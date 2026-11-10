#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();let server;
  try {
    const started=await open(dir);server=started.server;
    const p=started.port,root=server.skrynia.grants.root();
    assert.deepEqual(root.operations,
      ['create','modify','delete','list','delegate','subscribe']);
    const rootToken=(await expect(p,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    async function child(operations,locations='roles.*',parent=rootToken) {
      return (await expect(p,200,'POST','/grants',{
        operations,locations,maxBytes:10000,maxObjects:6
      },auth(parent))).json;
    }
    async function user(g) {
      return (await expect(p,200,'POST','/grants/'+g.id+'/users',
        null,auth(g.issuerToken))).json.userToken;
    }
    const creator=await child(['create','delegate']);
    const createToken=await user(creator);
    const modifier=await child(['modify']);
    const modifyToken=await user(modifier);
    const remover=await child(['delete']);
    const deleteToken=await user(remover);
    const lister=await child(['list']);
    const listToken=await user(lister);
    const key='/store/roles.event';
    await expect(p,201,'POST',key,Buffer.from([0,255,1]),grant(createToken));
    const before=await expect(p,200,'GET',key);
    assert.deepEqual(before.bytes,Buffer.from([0,255,1]));
    await expect(p,409,'POST',key,'cannot overwrite',grant(createToken));
    await expect(p,403,'PUT',key,'cannot modify',grant(createToken));
    await expect(p,403,'DELETE',key,null,grant(createToken));

    await expect(p,403,'POST','/store/roles.another','no',grant(modifyToken));
    await expect(p,403,'DELETE',key,null,grant(modifyToken));
    await expect(p,412,'PUT',key,'wrong',
      {...grant(modifyToken),'If-Match':'"wrong"'});
    await expect(p,200,'PUT',key,'updated',
      {...grant(modifyToken),'If-Match':before.etag});
    assert.equal((await expect(p,200,'GET',key)).bytes.toString(),'updated');

    await expect(p,403,'POST','/store/roles.another','no',grant(deleteToken));
    await expect(p,403,'PUT',key,'cannot modify',grant(deleteToken));
    await expect(p,200,'DELETE',key,null,grant(deleteToken));
    await expect(p,404,'GET',key);
    await expect(p,201,'POST',key,'recreated',grant(createToken));
    await expect(p,403,'POST','/store/roles.another','no',grant(listToken));
    await expect(p,403,'PUT',key,'no',grant(listToken));
    await expect(p,403,'DELETE',key,null,grant(listToken));
    assert.deepEqual((await expect(p,200,'GET',
      '/store?locations=roles.*',null,grant(listToken))).json.keys,['roles.event']);

    // Child authority is the intersection of the delegated operation set
    // and its location scope. Create never implies modify or delete.
    const nested=await child(['create'],'roles.child.*',createToken);
    const nestedToken=await user(nested);
    await expect(p,201,'POST','/store/roles.child.entry','child',grant(nestedToken));
    await expect(p,403,'PUT','/store/roles.child.entry','no',grant(nestedToken));
    await expect(p,403,'DELETE','/store/roles.child.entry',null,grant(nestedToken));
    await expect(p,403,'POST','/grants',{
      locations:'roles.child.*',operations:['modify'],
      maxBytes:4096,maxObjects:1},auth(createToken));
    await expect(p,403,'POST','/grants',{
      locations:'roles.child.*',operations:['delete'],
      maxBytes:4096,maxObjects:1},auth(createToken));
    await expect(p,400,'POST','/grants',{
      locations:'roles.*',operations:['write'],
      maxBytes:4096,maxObjects:1},auth(rootToken));

    // A finite allocator can promise quota headroom to independent children
    // by assigning disjoint prefixes and bounding the sum of their limits.
    // Parent: 10000 bytes, 2 objects; own grant overhead: 2048 bytes;
    // each child: 3000 bytes (2048 grant + 512 object + 440 payload).
    const allocator=(await expect(p,200,'POST','/grants',{
      locations:'capacity.*',operations:['create','delegate'],
      maxBytes:10000,maxObjects:2},auth(rootToken))).json;
    const allocatorToken=await user(allocator);
    async function allocate(prefix) {
      return (await expect(p,200,'POST','/grants',{
        locations:'capacity.'+prefix+'.*',operations:['create'],
        maxBytes:3000,maxObjects:1},auth(allocatorToken))).json;
    }
    const alice=await allocate('alice');
    const bob=await allocate('bob');
    const aliceToken=await user(alice);
    const bobToken=await user(bob);
    await expect(p,201,'POST','/store/capacity.bob.first',
      Buffer.alloc(440),grant(bobToken));
    await expect(p,201,'POST','/store/capacity.alice.first',
      Buffer.alloc(440),grant(aliceToken));
    const parentUsage=await expect(p,200,'GET','/grants/'+allocator.id,
      null,auth(allocatorToken));
    assert.equal(parentUsage.json.usage.bytes,8048);
    assert.equal(parentUsage.json.usage.objects,2);
    await expect(p,507,'POST','/store/capacity.alice.second',
      'quota full',grant(aliceToken));
    // Other siblings cannot consume Alice's allocation through their token.
    await expect(p,403,'POST','/store/capacity.alice.second',
      'unauthorized scope',grant(bobToken));

    // Role restrictions survive startup and work against previously
    // created objects irrespective of which grant authored them.
    await close(server);server=null;
    const reopened=await open(dir);server=reopened.server;
    await expect(server.address().port,200,'PUT',key,'again',grant(modifyToken));
    await expect(server.address().port,403,'PUT',key,'no',grant(createToken));
    console.log('independent create, modify, delete and delegation: OK');
  } finally {
    if(server) await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
