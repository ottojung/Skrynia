#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {execFileSync}=require('node:child_process');
const {createECDH,randomBytes}=require('node:crypto');
const {port,request,start,stop,credentials}=require('./crash/fixture');
const {manifest}=require('./snapshot/manifest');
async function main() {
  const root=fs.mkdtempSync(join(tmpdir(),'skrynia-snapshot-source-'));
  let child;
  try {
    const p=await port();child=await start(process.execPath,[resolve('src/server.js')],root,p);
    const grant=credentials(root);
    const response=await request(p,'POST',`/grants/${grant.id}/users`,grant.issuer);
    assert.equal(response.status,200);
    const user=JSON.parse(response.body).userToken;
    for(let i=0;i<7;i++) {
      const bytes=Buffer.alloc(4096+i*256,65+i);
      const result=await request(p,'POST',`/store/fixture_${i}.data`,user,bytes);
      assert.equal(result.status,201,result.body.toString());
    }
    const ec=createECDH('prime256v1');ec.generateKeys();
    const sub=await request(p,'POST','/push/subscriptions',user,
      JSON.stringify({locations:'fixture_*',events:['create','replace'],
        endpoint:'https://push.example.org/offline-snapshot',
        keys:{p256dh:ec.getPublicKey().toString('base64url'),
          auth:randomBytes(16).toString('base64url')}}));
    assert.equal(sub.status,201,sub.body.toString());
    await stop(child,'SIGTERM');child=undefined;
    const before=await manifest(root);
    execFileSync(process.execPath,[resolve('tests/rust-offline-snapshot.js'),root],
      {stdio:'inherit',timeout:60000});
    assert.deepEqual(await manifest(root),before,'snapshot source was modified');
    console.log('Offline-source immutability and credential/subscription replay: OK');
  } finally {
    if(child)await stop(child,'SIGKILL');
    fs.rmSync(root,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
