#!/usr/bin/env node
'use strict';
// Rehearse an offline upgrade and rollback on a realistic delegated data tree.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {mkdtempSync,rmSync,readdirSync,readFileSync}=fs;
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {createECDH,randomBytes}=require('node:crypto');
const {port,request,start,stop,credentials,sha}=require('./crash/fixture');
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
const leaves=4,perLeaf=Number(process.env.SKRYNIA_MIGRATION_OBJECTS_PER_LEAF||25);
function good(reply,code) {
  assert.equal(reply.status,code,reply.body.toString());
  try {return JSON.parse(reply.body);}catch{return reply.body;}
}
function payload(group,leaf,index,version) {
  const length=8192+((group*11+leaf*17+index*23)%10)*4096;
  return Buffer.alloc(length,65+(group+leaf+index+version)%25);
}
function key(group,leaf,index) {return `department_${group}.team_${leaf}.object_${index}`;}
async function createUser(portNumber,id,issuer) {
  return good(await request(portNumber,'POST',`/grants/${id}/users`,issuer),200).userToken;
}
async function makeGrant(portNumber,token,locations,operations,maxBytes,maxObjects) {
  return good(await request(portNumber,'POST','/grants',token,JSON.stringify({
    locations,operations,maxBytes,maxObjects})),200);
}
async function makeSubscription(portNumber,token,group) {
  const key=createECDH('prime256v1');key.generateKeys();
  return good(await request(portNumber,'POST','/push/subscriptions',token,
    JSON.stringify({locations:`department_${group}.*`,
      events:['create','replace','delete'],
      endpoint:`https://push.example.org/migration-${group}`,
      keys:{p256dh:key.getPublicKey().toString('base64url'),
        auth:randomBytes(16).toString('base64url')}})),201);
}
async function build(portNumber,user,expected,scopes,subs) {
  for(let group=0;group<leaves;group++) {
    const child=await makeGrant(portNumber,user,`department_${group}.*`,
      ['create','modify','delete','list','delegate','subscribe'],15000000,200);
    const groupUser=await createUser(portNumber,child.id,child.issuerToken);
    subs.push(await makeSubscription(portNumber,groupUser,group));
    const department=await makeGrant(portNumber,groupUser,
      `department_${group}.team_${group}.*`,
      ['create','modify','delete','list'],10000000,perLeaf+15);
    const teamUser=await createUser(portNumber,department.id,department.issuerToken);
    scopes.push({group,leaf:group,token:teamUser,grantId:department.id});
    for(let index=0;index<perLeaf;index++) {
      const name=key(group,group,index),body=payload(group,group,index,0);
      good(await request(portNumber,'POST','/store/'+name,teamUser,body),201);
      expected.set(name,body);
    }
  }
}
async function verify(portNumber,expected) {
  for(const [name,bytes] of expected) {
    const reply=await request(portNumber,'GET','/store/'+name);
    assert.equal(reply.status,200,name);
    assert(reply.body.equals(bytes),`mismatched bytes for ${name}`);
    assert.equal(reply.headers.etag,'"'+sha(bytes)+'"',name);
  }
}
async function mutate(portNumber,expected,scopes) {
  for(const {group,leaf,token} of scopes) {
    for(let index=0;index<perLeaf;index++) {
      const name=key(group,leaf,index),old=expected.get(name);
      if(index%7===0) {
        good(await request(portNumber,'DELETE','/store/'+name,token,undefined,
          {'X-Skrynia-If-SHA256':sha(old)}),200);
        expected.delete(name);
      } else if(index%3===0) {
        const next=payload(group,leaf,index,1);
        good(await request(portNumber,'PUT','/store/'+name,token,next,
          {'X-Skrynia-If-SHA256':sha(old)}),200);
        expected.set(name,next);
      }
    }
    const fresh=`department_${group}.team_${leaf}.added`;
    const body=Buffer.from(`new-after-Rust-upgrade-${group}`);
    good(await request(portNumber,'POST','/store/'+fresh,token,body),201);
    expected.set(fresh,body);
  }
}
async function main() {
  assert(Number.isInteger(perLeaf)&&perLeaf>=5&&perLeaf<=40);
  const root=mkdtempSync(join(tmpdir(),'skrynia-migrate-rehearse-'));
  const expected=new Map(),scopes=[],subs=[];
  let child;
  try {
    const portNumber=await port();
    child=await start(process.execPath,[resolve('src/server.js')],root,portNumber);
    const grant=credentials(root);
    const rootUser=await createUser(portNumber,grant.id,grant.issuer);
    await build(portNumber,rootUser,expected,scopes,subs);
    const originalVapid=readFileSync(join(root,'state/push/vapid.json'));
    const objects=expected.size;
    await verify(portNumber,expected);
    await stop(child,'SIGTERM');child=undefined;
    child=await start(binary,[],root,portNumber);
    assert.equal((await request(portNumber,'GET','/health')).status,200);
    await verify(portNumber,expected);
    for(const scope of scopes) {
      const result=good(await request(portNumber,'GET',
        '/store?locations='+encodeURIComponent(`department_${scope.group}.team_${scope.leaf}.*`),
        scope.token),200);
      assert.equal(result.keys.length,perLeaf);
    }
    assert.equal(readFileSync(join(root,'state/push/vapid.json')).toString(),originalVapid.toString());
    await mutate(portNumber,expected,scopes);
    await stop(child,'SIGTERM');child=undefined;
    child=await start(process.execPath,[resolve('src/server.js')],root,portNumber);
    await verify(portNumber,expected);
    for(const sub of subs) {
      assert(fs.existsSync(join(root,'state/push-subs',sub.id+'.json')));
      assert(fs.existsSync(join(root,'state/push/outbox',sub.id+'.json')));
    }
    assert.deepEqual(readdirSync(join(root,'state/store-transactions')),[]);
    console.log(`Offline Node → Rust → Node migration: ${objects} original objects, `+
      `${expected.size} final objects, ${scopes.length} delegated teams, `+
      `${subs.length} Push subscriptions, VAPID and credentials preserved: OK`);
  } finally {
    if(child)await stop(child,'SIGKILL');
    rmSync(root,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
