#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {createPush}=require('../src/push.js');
const {tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();let push;
  try {
    let calls=0,started;
    let releaseFirst;
    const firstStarted=new Promise(resolve=>{started=resolve;});
    const firstFailed=new Promise((_,reject)=>{
      releaseFirst=()=>reject(new Error('endpoint unavailable'));
    });
    push=createPush({
      dataDir:dir,pushManual:true,pushMaxAttempts:3,
      pushAllowedHosts:'first.example.org,second.example.org',
      onSubscriptionCreate:()=> 'a'.repeat(32),
      pushTransport:async ()=>{
        calls++;
        if(calls===1) {started();await firstFailed;}
        else throw new Error('endpoint unavailable');
      }
    });
    push.start();
    assert.throws(()=>push.createSub('*',['create'],
      'https://third.example.org/u',{p256dh:'a',auth:'b'},null,'grant'),
      e=>e.code==='invalid_subscription');
    assert.throws(()=>createPush({dataDir:dir,pushManual:true,
      pushAllowedHosts:'localhost'}),/invalid push allowed hosts/);

    const sub=push.createSub('*',['create'],
      'https://first.example.org/u',{p256dh:'a',auth:'b'},null,'grant');
    push.enqueue('events.first','create');
    const sending=push.pumpOnce();
    await firstStarted;
    push.enqueue('events.second','create');
    assert.equal(push.listOutbox()[0].entry.generation,2);
    releaseFirst();
    assert.deepEqual(await sending,{delivered:0});
    assert.equal(push.listOutbox()[0].entry.attempts,1);
    assert(push.listOutbox()[0].entry.nextAt>Date.now());

    // More writes must not reset backoff or the failed-send budget.
    push.enqueue('events.third','create');
    assert.equal(push.listOutbox()[0].entry.attempts,1);
    assert.equal((await push.pumpOnce()).delivered,0);
    assert.equal(calls,1);
    await push.pumpOnce({ignoreBackoff:true});
    assert.equal(push.listOutbox()[0].entry.attempts,2);
    assert.equal(calls,2);
    push.enqueue('events.fourth','create');
    assert.equal(push.listOutbox()[0].entry.attempts,2);
    await push.pumpOnce({ignoreBackoff:true});
    assert.equal(calls,3);
    assert.equal(push.findSub(sub.id),null,
      'exhausted failing subscription should be retired');
    assert.equal(push.outboxCount(),0);
    push.enqueue('events.fifth','create');
    assert.equal(push.outboxCount(),0);

    const restored=createPush({
      dataDir:dir,pushManual:true,pushMaxAttempts:3,
      pushAllowedHosts:'second.example.org',
      onSubscriptionCreate:()=> 'a'.repeat(32)
    });
    const next=restored.createSub('*',['create'],
      'https://second.example.org/u',{p256dh:'a',auth:'b'},null,'grant');
    assert.equal(restored.findSub(next.id)?.id,next.id);
    assert.throws(()=>restored.updateSub(next.id,next.capability,{
      endpoint:'https://first.example.org/u'
    }),e=>e.code==='invalid_subscription');
    restored.close();
    console.log('push retry cap, shared backoff, allowed hosts and retirement: OK');
  } finally {
    if(push)push.close();
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
