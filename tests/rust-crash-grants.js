#!/usr/bin/env node
'use strict';
// Crash while recursively revoking delegated authority and pending Push records.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {mkdtempSync,rmSync,existsSync,readdirSync}=fs;
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const crypto=require('node:crypto');
const {port,request,start,stop,credentials,pause}=require('./crash/fixture');
const rounds=Number(process.env.SKRYNIA_REVOKE_CRASH_ROUNDS||4);
const width=Number(process.env.SKRYNIA_REVOKE_CRASH_WIDTH||10);
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
function json(response,status) {
  assert.equal(response.status,status,response.body.toString());
  return JSON.parse(response.body);
}
async function build(p,rootUser,round) {
  const prefix=`revoke_${round}.`;
  const parent=json(await request(p,'POST','/grants',rootUser,JSON.stringify({
    locations:prefix+'*',operations:['create','modify','delete','list','delegate','subscribe'],
    maxBytes:1000000,maxObjects:200})),200);
  const delegated=json(await request(p,'POST',`/grants/${parent.id}/users`,parent.issuerToken),200);
  const ids=[parent.id],users=[];
  for(let i=0;i<width;i++) {
    const child=json(await request(p,'POST','/grants',delegated.userToken,JSON.stringify({
      locations:`${prefix}leaf_${i}.*`,operations:['create','list'],maxBytes:6000,maxObjects:2})),200);
    ids.push(child.id);
    const user=json(await request(p,'POST',`/grants/${child.id}/users`,child.issuerToken),200);
    users.push(user.userToken);
    json(await request(p,'POST',`/store/${prefix}leaf_${i}.data`,user.userToken,'safe'),201);
  }
  const ecdh=crypto.createECDH('prime256v1');ecdh.generateKeys();
  const sub=json(await request(p,'POST','/push/subscriptions',delegated.userToken,
    JSON.stringify({locations:prefix+'*',events:['create','replace','delete'],
      endpoint:`https://push.example.org/grant-test-${round}`,
      keys:{p256dh:ecdh.getPublicKey().toString('base64url'),
        auth:crypto.randomBytes(16).toString('base64url')}})),201);
  json(await request(p,'POST',`/store/${prefix}trigger`,rootUser,'notice'),201);
  return {prefix,parent,ids,users,sub};
}
async function killDuringRevocation(p,child,root,tree) {
  const marker=join(root,'state/grant-revocations',`${tree.parent.id}.json`);
  const sent=request(p,'DELETE',`/grants/${tree.parent.id}`,tree.parent.issuerToken)
    .catch(()=>null);
  let sawMarker=false;
  for(let i=0;i<100;i++) {
    if(existsSync(marker)) {sawMarker=true;break;}
    await pause(2);
  }
  await stop(child,'SIGKILL');
  const response=await sent;
  if(response)assert.equal(response.status,200,response.body.toString());
  return {sawMarker,acknowledged:Boolean(response)};
}
async function verify(p,root,tree,result) {
  const grants=join(root,'state/grants');
  const surviving=tree.ids.filter(id=>existsSync(join(grants,id+'.json')));
  assert(surviving.length===0||surviving.length===tree.ids.length,
    `partially revoked grant tree: ${surviving.length}/${tree.ids.length}`);
  if(result.sawMarker||result.acknowledged) assert.equal(surviving.length,0);
  const subPath=join(root,'state/push-subs',tree.sub.id+'.json');
  const outboxPath=join(root,'state/push/outbox',tree.sub.id+'.json');
  if(surviving.length===0) {
    assert(!existsSync(subPath),'orphaned subscription after revocation');
    assert(!existsSync(outboxPath),'orphaned durable notification after revocation');
    for(const token of tree.users) {
      const answer=await request(p,'POST',`/store/${tree.prefix}forbidden`,token,'no');
      assert.equal(answer.status,403,'revoked descendant token still authorized');
    }
  } else {
    assert(existsSync(subPath),'surviving parent subscription missing');
    assert(existsSync(outboxPath),'surviving parent outbox missing');
  }
  for(let i=0;i<width;i++) {
    const content=await request(p,'GET',`/store/${tree.prefix}leaf_${i}.data`);
    assert.equal(content.status,200,'revocation must not delete object data');
    assert.equal(content.body.toString(),'safe');
  }
  assert.deepEqual(readdirSync(join(root,'state/grant-revocations')),[],
    'revocation marker not cleared after recovery');
  return surviving.length===0;
}
async function main() {
  assert(Number.isInteger(rounds)&&rounds>=1&&rounds<=30);
  assert(Number.isInteger(width)&&width>=1&&width<=50);
  const root=mkdtempSync(join(tmpdir(),'skrynia-revoke-kill-'));
  let child;
  try {
    const p=await port();
    child=await start(process.execPath,[resolve('src/server.js')],root,p);
    const grant=credentials(root);
    const rootUser=json(await request(p,'POST',`/grants/${grant.id}/users`,grant.issuer),200).userToken;
    await stop(child,'SIGTERM');child=undefined;
    let revoked=0,markers=0;
    for(let round=0;round<rounds;round++) {
      child=await start(binary,[],root,p);
      const tree=await build(p,rootUser,round);
      const outcome=await killDuringRevocation(p,child,root,tree);
      child=undefined;
      child=await start(binary,[],root,p);
      if(outcome.sawMarker)markers++;
      if(await verify(p,root,tree,outcome))revoked++;
      await stop(child,'SIGTERM');child=undefined;
      console.log(`Revocation kill ${round+1}/${rounds}: marker=${outcome.sawMarker}, revoked=${revoked}`);
    }
    child=await start(process.execPath,[resolve('src/server.js')],root,p);
    assert.equal((await request(p,'GET','/health')).status,200);
    console.log(`Revocation crash recovery passed: ${rounds} rounds, ${revoked} completed, `+
      `${markers} journal markers observed; Node startup OK`);
  } finally {
    if(child)await stop(child,'SIGKILL');
    rmSync(root,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
