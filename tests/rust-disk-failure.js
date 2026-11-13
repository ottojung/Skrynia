#!/usr/bin/env node
'use strict';
// Fail closed on directory I/O errors; recover a committed-but-unnotified write.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {mkdtempSync,rmSync,readFileSync,readdirSync,chmodSync}=fs;
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const crypto=require('node:crypto');
const {port,request,start,stop,credentials,sha}=require('./crash/fixture');
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
function ok(result,status) {
  assert.equal(result.status,status,result.body.toString());
  return result;
}
async function deniedMutation(root,portNumber,token,protectedDir) {
  const path=join(root,protectedDir);
  chmodSync(path,0o500);
  try {
    const denied=await request(portNumber,'PUT','/store/evidence',token,'replacement',
      {'X-Skrynia-If-SHA256':sha('original')});
    ok(denied,500);
    const failed=await request(portNumber,'GET','/health');
    ok(failed,503);
    assert.equal(JSON.parse(failed.body).error,'recovery_required');
    assert.equal(readFileSync(join(root,'store/evidence'),'utf8'),'original');
  } finally {chmodSync(path,0o700);}
}
async function main() {
  if(process.getuid?.()===0) {console.log('Permission fault test needs non-root execution');return;}
  const root=mkdtempSync(join(tmpdir(),'skrynia-disk-fault-'));
  let child;
  try {
    const p=await port();
    child=await start(binary,[],root,p);
    const grant=credentials(root);
    const user=JSON.parse(ok(await request(p,'POST',
      `/grants/${grant.id}/users`,grant.issuer),200).body).userToken;
    ok(await request(p,'POST','/store/evidence',user,'original'),201);
    await deniedMutation(root,p,user,'state/store-transactions');
    await stop(child,'SIGTERM');child=undefined;
    child=await start(binary,[],root,p);
    ok(await request(p,'GET','/store/evidence'),200);
    await deniedMutation(root,p,user,'store-tmp');
    await stop(child,'SIGTERM');child=undefined;
    child=await start(binary,[],root,p);
    assert.equal(ok(await request(p,'GET','/store/evidence'),200).body.toString(),'original');
    const ecdh=crypto.createECDH('prime256v1');ecdh.generateKeys();
    const subscription=JSON.parse(ok(await request(p,'POST','/push/subscriptions',user,
      JSON.stringify({locations:'evidence',events:['replace'],
        endpoint:'https://push.example.org/durable-notification',
        keys:{p256dh:ecdh.getPublicKey().toString('base64url'),
          auth:crypto.randomBytes(16).toString('base64url')}})),201).body);
    const outbox=join(root,'state/push/outbox');
    chmodSync(outbox,0o500);
    let written;
    try {
      written=ok(await request(p,'PUT','/store/evidence',user,'replacement',
        {'X-Skrynia-If-SHA256':sha('original')}),200);
    } finally {chmodSync(outbox,0o700);}
    assert.equal(JSON.parse(written.body).notificationPending,true);
    assert.equal(readFileSync(join(root,'store/evidence'),'utf8'),'replacement');
    assert.equal(readdirSync(join(root,'state/store-transactions')).length,1);
    await stop(child,'SIGTERM');child=undefined;
    child=await start(binary,[],root,p);
    assert.equal(ok(await request(p,'GET','/store/evidence'),200).body.toString(),'replacement');
    const notice=JSON.parse(readFileSync(join(outbox,subscription.id+'.json')));
    assert.equal(notice.generation,1);
    assert.deepEqual(readdirSync(join(root,'state/store-transactions')),[]);
    await stop(child,'SIGTERM');child=undefined;
    child=await start(process.execPath,[resolve('src/server.js')],root,p);
    assert.equal(ok(await request(p,'GET','/store/evidence'),200).body.toString(),'replacement');
    console.log('Disk fault injection passed: fail-closed journal/staging, recovered notification, Node restart');
  } finally {
    if(child)await stop(child,'SIGKILL');
    rmSync(root,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
