#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp(),delivered=[];
  let server;
  try {
    const started=await open(dir,{pushMaxOutboxItems:1,
      pushTransport:async sub=>{delivered.push(sub.endpoint);}});
    server=started.server;
    const p=started.port,root=server.skrynia.grants.root();
    const rootToken=(await expect(p,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    const child=(await expect(p,200,'POST','/grants',{
      locations:'news.*',operations:['subscribe'],maxBytes:10000,maxObjects:2
    },auth(rootToken))).json;
    const subscriptionToken=(await expect(p,200,'POST',
      '/grants/'+child.id+'/users',null,auth(child.issuerToken))).json.userToken;
    const sample={locations:'news.*',events:['replace'],
      endpoint:'https://push.example.org/u/1',keys:{p256dh:'public-key',auth:'auth-secret'}};
    await expect(p,403,'POST','/push/subscriptions',
      {...sample,locations:'other.*'},grant(subscriptionToken));
    await expect(p,400,'POST','/push/subscriptions',
      {...sample,events:['invalid']},grant(subscriptionToken));
    const registered=(await expect(p,201,'POST','/push/subscriptions',
      sample,grant(subscriptionToken))).json;
    assert.match(registered.capability,/^[a-f0-9]{64}$/);
    await expect(p,403,'POST','/push/subscriptions',sample,grant(subscriptionToken));
    const deduped=(await expect(p,201,'POST','/push/subscriptions',
      sample,{'X-Skrynia-Capability':registered.capability})).json;
    assert.equal(deduped.id,registered.id);
    assert.notEqual(deduped.capability,registered.capability);
    const parallel=(await expect(p,201,'POST','/push/subscriptions',{
      ...sample,locations:'news.item',events:['delete']
    },grant(rootToken))).json;
    assert.notEqual(parallel.id,registered.id,
      'one browser endpoint can subscribe independently to different scopes');
    await expect(p,403,'DELETE','/push/subscriptions/'+registered.id,null,
      {'X-Skrynia-Capability':registered.capability});

    await expect(p,201,'POST','/store/news.item','first',grant(rootToken));
    assert.equal(server.skrynia.push.outboxCount(),0,'create not subscribed');
    await expect(p,200,'PUT','/store/news.item','second',grant(rootToken));
    assert.equal(server.skrynia.push.outboxCount(),1);
    await expect(p,200,'PUT','/store/news.item','third',grant(rootToken));
    assert.equal(server.skrynia.push.outboxCount(),1,'pending changes coalesce');
    assert.equal(server.skrynia.push.listOutbox()[0].entry.generation,2);
    assert.equal((await expect(p,200,'GET','/store/news.item')).bytes.toString(),'third');
    assert.deepEqual(await server.skrynia.push.pumpOnce(),{delivered:1});
    assert.deepEqual(delivered,['https://push.example.org/u/1']);
    assert.equal(server.skrynia.push.outboxCount(),0);

    // The cap is still enforced when an update would need two distinct
    // outbox entries; the failed mutation must preserve the old value.
    const extra=(await expect(p,201,'POST','/push/subscriptions',{
      ...sample,endpoint:'https://push.example.org/other'
    },grant(rootToken))).json;
    await expect(p,507,'PUT','/store/news.item','blocked',grant(rootToken));
    assert.equal((await expect(p,200,'GET','/store/news.item')).bytes.toString(),'third');
    await expect(p,200,'DELETE','/push/subscriptions/'+extra.id,null,
      {'X-Skrynia-Capability':extra.capability});
    await expect(p,200,'PUT','/store/news.item','fourth',grant(rootToken));
    assert.equal(server.skrynia.push.outboxCount(),1);

    const changed=await expect(p,200,'PUT','/push/subscriptions/'+registered.id,
      {endpoint:'https://push.example.org/u/2'},
      {'X-Skrynia-Capability':deduped.capability});
    assert.equal(changed.json.ok,true);
    assert.deepEqual(await server.skrynia.push.pumpOnce(),{delivered:1});
    assert.deepEqual(delivered,['https://push.example.org/u/1','https://push.example.org/u/2']);

    const stored=server.skrynia.grants.usage(child.id);
    assert.equal(stored.objects,2,'matched object and subscription share the object-count budget');
    await expect(p,200,'DELETE','/grants/'+child.id,null,auth(child.issuerToken));
    assert.equal(server.skrynia.push.allSubs().length,1,
      'revocation does not remove another grant\'s subscription');
    await expect(p,200,'DELETE','/push/subscriptions/'+parallel.id,null,
      {'X-Skrynia-Capability':parallel.capability});
    assert.equal(server.skrynia.push.allSubs().length,0);
    await expect(p,403,'POST','/push/subscriptions',sample,grant(subscriptionToken));
    await expect(p,200,'PUT','/store/news.item','fifth',grant(rootToken));
    assert.equal(server.skrynia.push.outboxCount(),0);
    console.log('grant-scoped Web Push, events, quotas, revocation: OK');
  }finally{
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
