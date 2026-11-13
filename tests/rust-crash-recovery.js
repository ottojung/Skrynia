#!/usr/bin/env node
'use strict';
// Crash-test a real Rust process in the middle of concurrent durable mutations.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const crypto=require('node:crypto');
const {mkdtempSync,rmSync,readdirSync,readFileSync}=fs;
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {port,request,start,stop,credentials,sha,pause}=require('./crash/fixture');
const rounds=Number(process.env.SKRYNIA_CRASH_ROUNDS||6);
const writes=Number(process.env.SKRYNIA_CRASH_WRITES||8);
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
const sample=(round,index,version)=>Buffer.alloc(24*1024,65+(round+index+version)%25);
const expected=(status,code,context)=>assert.equal(status,code,context);
async function main() {
  assert(Number.isInteger(rounds)&&rounds>=1&&rounds<=30);
  assert(Number.isInteger(writes)&&writes>=1&&writes<=32);
  const root=mkdtempSync(join(tmpdir(),'skrynia-kill-test-'));
  let child;
  try {
    const p=await port();
    child=await start(process.execPath,[resolve('src/server.js')],root,p);
    const grant=credentials(root);
    const issued=await request(p,'POST',`/grants/${grant.id}/users`,grant.issuer);
    expected(issued.status,200,'issue user');
    const user=JSON.parse(issued.body).userToken;
    const subscriber=crypto.createECDH('prime256v1');subscriber.generateKeys();
    const registration=await request(p,'POST','/push/subscriptions',user,JSON.stringify({
      locations:'fresh_*',events:['create'],endpoint:'https://push.example.org/crash-test',
      keys:{p256dh:subscriber.getPublicKey().toString('base64url'),
        auth:crypto.randomBytes(16).toString('base64url')}
    }));
    expected(registration.status,201,'register durable crash wake-up');
    const subId=JSON.parse(registration.body).id;
    const noticeFile=join(root,'state/push/outbox',subId+'.json');
    for(let round=0;round<rounds;round++) {
      for(let j=0;j<writes;j++) {
        expected((await request(p,'POST',`/store/base_${round}_${j}`,user,sample(round,j,0))).status,
          201,'seed base object');
      }
    }
    await stop(child,'SIGTERM');child=undefined;
    let confirmed=0,uncertain=0,applied=0,created=0;
    for(let round=0;round<rounds;round++) {
      child=await start(binary,[],root,p);
      const calls=[];
      for(let j=0;j<writes;j++) {
        calls.push({key:`fresh_${round}_${j}`,old:null,body:sample(round,j,1),
          request:request(p,'POST',`/store/fresh_${round}_${j}`,user,sample(round,j,1)).catch(()=>null)});
        calls.push({key:`base_${round}_${j}`,old:sample(round,j,0),body:sample(round,j,2),
          request:request(p,'PUT',`/store/base_${round}_${j}`,user,sample(round,j,2),
            {'X-Skrynia-If-SHA256':sha(sample(round,j,0))}).catch(()=>null)});
      }
      await pause([8,25,60,120,240,450][round%6]);
      await stop(child,'SIGKILL');child=undefined;
      const outcomes=await Promise.all(calls.map(async call=>{
        try{return await call.request;}catch{return null;}
      }));
      child=await start(binary,[],root,p);
      for(let j=0;j<calls.length;j++) {
        const call=calls[j],outcome=outcomes[j];
        const actual=await request(p,'GET',`/store/${call.key}`);
        const got=actual.status===200?actual.body:null;
        assert(actual.status===200||actual.status===404,`unexpected GET ${call.key}: ${actual.status}`);
        assert(got===null || got.equals(call.body) || (call.old&&got.equals(call.old)),
          `torn/unknown object: ${call.key}`);
        if(outcome)assert([200,201].includes(outcome.status),`unexpected write response ${outcome.status} on ${call.key}`);
        if(outcome) {
          confirmed++;
          assert.equal(actual.status,200,`acknowledged mutation missing: ${call.key}`);
          assert(got.equals(call.body),`acknowledged mutation lost: ${call.key}`);
        } else {uncertain++;}
        if(got?.equals(call.body)) {
          applied++;
          if(call.old===null)created++;
        }
      }
      if(created>0) {
        const pending=JSON.parse(readFileSync(noticeFile));
        assert.equal(pending.subscriptionId,subId);
        assert(pending.generation>=created,
          `lost Push wake-up: ${pending.generation} generations for ${created} applied creates`);
      }
      assert.deepEqual(readdirSync(join(root,'store-tmp')),[],
        'abandoned staged files not cleaned after crash recovery');
      const journal=join(root,'state/store-transactions');
      assert.deepEqual(readdirSync(journal),[],`journal not fully recovered in round ${round}`);
      await stop(child,'SIGTERM');child=undefined;
      console.log(`SIGKILL round ${round+1}/${rounds}: verified ${calls.length} objects`);
    }
    child=await start(process.execPath,[resolve('src/server.js')],root,p);
    for(let round=0;round<rounds;round++) {
      for(let j=0;j<writes;j++) {
        for(const key of [`fresh_${round}_${j}`,`base_${round}_${j}`]) {
          const result=await request(p,'GET',`/store/${key}`);
          assert([200,404].includes(result.status),`Node cannot read recovered ${key}`);
        }
      }
    }
    console.log(`SIGKILL recovery passed: ${rounds} crashes, ${rounds*writes*2} objects, `+
      `${confirmed} acknowledged, ${uncertain} uncertain, ${applied} new values visible, `+
      `${created} recoverable Push creates; Node reader OK`);
  } finally {
    if(child)await stop(child,'SIGKILL');
    rmSync(root,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
