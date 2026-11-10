#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {createPush}=require('../src/push.js');
const {validEndpoint,publicAddress,safeAgent}=require('../src/push-endpoint.js');
const {tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();let push;
  try {
    for(const url of [
      'http://localhost:1234/u','https://localhost/u',
      'https://127.0.0.1/u','https://[::1]/u',
      'https://172.20.2.2/u','https://host.internal/path',
      'https://user:pass@push.example.org/u'
    ]) assert.equal(validEndpoint(url),false,url);
    assert.equal(validEndpoint('https://push.example.org/u'),true);
    for(const address of [
      '127.0.0.1','10.1.2.3','172.16.1.1','192.168.1.1',
      '169.254.1.1','::1','::ffff:127.0.0.1','2002:0a00:0001::'
    ]) assert.equal(publicAddress(address),false,address);
    assert.equal(publicAddress('8.8.8.8'),true);
    assert.equal(publicAddress('2606:4700:4700::1111'),true);

    async function lookup(address) {
      const agent=safeAgent((name,opts,callback)=>
        callback(null,[{address,family:address.includes(':')?6:4}]));
      return new Promise(resolve=>agent.options.lookup('push.example.org',
        {family:0},(error,value)=>resolve(error?.code||value)));
    }
    assert.equal(await lookup('127.0.0.1'),'EHOSTUNREACH');
    assert.equal(await lookup('192.168.2.1'),'EHOSTUNREACH');
    assert.equal(await lookup('8.8.8.8'),'8.8.8.8');

    let active=0,maxActive=0;
    const started=[];
    let release;
    const hold=new Promise(resolve=>{release=resolve;});
    const reached=new Promise(resolve=>{
      const notify=()=>{
        if(started.length===2) resolve();
      };
      push=createPush({dataDir:dir,pushManual:true,pushConcurrency:2,
        onSubscriptionCreate:()=> 'a'.repeat(32),
        pushTransport:async sub=>{
          active++;maxActive=Math.max(maxActive,active);
          started.push(sub.endpoint);notify();
          await hold;
          active--;
        }});
    });
    push.start();
    const events=['https://one.example.org/a','https://two.example.org/b',
      'https://three.example.org/c'];
    for(const endpoint of events) {
      push.createSub('*',['create'],endpoint,{p256dh:'test',auth:'test'},
        null,'ignored');
    }
    push.enqueue('events.entry','create');
    const pumping=push.pumpOnce();
    let timer;
    try {
      await Promise.race([reached,new Promise((_,reject)=>{
        timer=setTimeout(()=>reject(new Error('push delivery remained sequential')),1000);
      })]);
    } finally {clearTimeout(timer);}
    assert.equal(started.length,2);
    assert.equal(maxActive,2);
    release();
    assert.deepEqual(await pumping,{delivered:3});
    assert.equal(push.outboxCount(),0);

    const previous=process.env.SKRYNIA_PUSH_SEND_TIMEOUT_MS;
    try {
      process.env.SKRYNIA_PUSH_SEND_TIMEOUT_MS='not-a-number';
      assert.throws(()=>createPush({dataDir:dir,pushManual:true}),
        /invalid push timeout/);
      process.env.SKRYNIA_PUSH_SEND_TIMEOUT_MS='42';
      const next=createPush({dataDir:dir,pushManual:true});
      next.close();
    } finally {
      if(previous===undefined)delete process.env.SKRYNIA_PUSH_SEND_TIMEOUT_MS;
      else process.env.SKRYNIA_PUSH_SEND_TIMEOUT_MS=previous;
    }
    console.log('push SSRF protection, DNS pinning, timeout and parallel delivery: OK');
  } finally {
    if(push)push.close();
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
