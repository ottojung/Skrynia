#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');
async function run() {
  const dir=tmp();
  let server,failed=0;
  try {
    let started=await open(dir,{pushTransport:async()=>{failed++;throw Error('network unavailable');}});
    server=started.server;let p=started.port;
    const root=server.skrynia.grants.root();
    const token=(await expect(p,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    const vapid=(await expect(p,200,'GET','/push/vapid')).json.publicKey;
    await expect(p,201,'POST','/push/subscriptions',{
      locations:'feed.*',events:['create'],
      endpoint:'https://push.example.org/test',
      keys:{p256dh:'public',auth:'auth'}
    },grant(token));
    await expect(p,201,'POST','/store/feed.entry','one',grant(token));
    assert.equal(server.skrynia.push.outboxCount(),1);
    assert.deepEqual(await server.skrynia.push.pumpOnce(),{delivered:0});
    assert.equal(failed,1);
    assert.equal(server.skrynia.push.outboxCount(),1);
    await close(server);server=null;

    const delivered=[];
    started=await open(dir,{pushTransport:async sub=>delivered.push(sub.endpoint)});
    server=started.server;p=started.port;
    assert.equal(server.skrynia.grants.root().id,root.id);
    assert.equal((await expect(p,200,'GET','/push/vapid')).json.publicKey,vapid);
    assert.deepEqual(await server.skrynia.push.pumpOnce({ignoreBackoff:true}),
      {delivered:1});
    assert.deepEqual(delivered,['https://push.example.org/test']);
    assert.equal(server.skrynia.push.outboxCount(),0);
    await expect(p,200,'DELETE','/grants/'+root.id,null,auth(root.rootIssuerToken));
    assert.equal(server.skrynia.push.allSubs().length,0);
    await close(server);server=null;

    started=await open(dir);server=started.server;p=started.port;
    assert.notEqual(server.skrynia.grants.root().id,root.id);
    assert.equal(server.skrynia.push.allSubs().length,0);
    assert.equal((await expect(p,200,'GET','/store/feed.entry')).bytes.toString(),'one');
    console.log('Web Push durable retry and root revocation: OK');
  }finally{
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
