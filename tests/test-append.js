#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();let server;
  try {
    let started=await open(dir);server=started.server;
    let p=started.port;
    const root=server.skrynia.grants.root();
    assert(root.operations.includes('append'));
    const rootToken=(await expect(p,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    const append=(await expect(p,200,'POST','/grants',{
      locations:'ledger.*',operations:['append','delegate'],
      maxBytes:9000,maxObjects:2},auth(rootToken))).json;
    const token=(await expect(p,200,'POST','/grants/'+append.id+'/users',
      null,auth(append.issuerToken))).json.userToken;
    assert.deepEqual(append.operations,['append','delegate']);

    const first=Buffer.from([0x00,0xff,0x0a,0x00]);
    await expect(p,201,'POST','/store/ledger.log',first,grant(token));
    assert.deepEqual((await expect(p,200,'GET','/store/ledger.log')).bytes,first);

    const extended=Buffer.concat([first,Buffer.from([0x00,0xff,0x41])]);
    const original=await expect(p,200,'GET','/store/ledger.log');
    await expect(p,200,'PUT','/store/ledger.log',extended,
      {...grant(token),'If-Match':original.etag});
    const current=await expect(p,200,'GET','/store/ledger.log');
    assert.deepEqual(current.bytes,extended);

    const stale=Buffer.concat([first,Buffer.from('different append')]);
    await expect(p,409,'PUT','/store/ledger.log',stale,grant(token));
    await expect(p,409,'PUT','/store/ledger.log',first,grant(token));
    await expect(p,409,'PUT','/store/ledger.log',
      Buffer.from([0x00,0xfe,0x0a,0x00,0x00,0xff,0x41]),grant(token));
    await expect(p,409,'PUT','/store/ledger.log',
      Buffer.concat([first,Buffer.from([0x00,0xff,0x42])]),grant(token));
    await expect(p,412,'PUT','/store/ledger.log',
      Buffer.concat([extended,Buffer.from('!')]),
      {...grant(token),'If-Match':original.etag});
    assert.deepEqual((await expect(p,200,'GET','/store/ledger.log')).bytes,extended);

    await expect(p,200,'PUT','/store/ledger.log',extended,grant(token));
    const updated=await expect(p,200,'GET','/store/ledger.log');
    assert.notEqual(updated.etag,current.etag);
    await expect(p,403,'DELETE','/store/ledger.log',null,grant(token));
    await expect(p,403,'POST','/store/other.log',first,grant(token));
    await expect(p,507,'PUT','/store/ledger.log',
      Buffer.concat([extended,Buffer.alloc(9000)]),grant(token));

    await expect(p,403,'POST','/grants',{
      locations:'ledger.*',operations:['write'],maxBytes:4096,maxObjects:1
    },auth(token));
    const nested=(await expect(p,200,'POST','/grants',{
      locations:'ledger.child.*',operations:['append'],
      maxBytes:4096,maxObjects:1
    },auth(token))).json;
    const nestedToken=(await expect(p,200,'POST','/grants/'+nested.id+'/users',
      null,auth(nested.issuerToken))).json.userToken;
    await expect(p,201,'POST','/store/ledger.child.log','hi',grant(nestedToken));
    await expect(p,200,'PUT','/store/ledger.child.log','hi!',grant(nestedToken));
    await expect(p,409,'PUT','/store/ledger.child.log','hi?',grant(nestedToken));

    // 'write' alone implies append authority when attenuating grants.
    const writer=(await expect(p,200,'POST','/grants',{
      locations:'records.*',operations:['write','delegate'],
      maxBytes:20000,maxObjects:3
    },auth(rootToken))).json;
    const writerToken=(await expect(p,200,'POST','/grants/'+writer.id+'/users',
      null,auth(writer.issuerToken))).json.userToken;
    const narrowed=(await expect(p,200,'POST','/grants',{
      locations:'records.log',operations:['append'],
      maxBytes:4096,maxObjects:1
    },auth(writerToken))).json;
    const narrowedToken=(await expect(p,200,'POST','/grants/'+narrowed.id+'/users',
      null,auth(narrowed.issuerToken))).json.userToken;
    await expect(p,201,'POST','/store/records.log','first',grant(narrowedToken));
    await expect(p,409,'PUT','/store/records.log','replaced',grant(narrowedToken));
    await expect(p,200,'PUT','/store/records.log','replaced',grant(writerToken));

    await close(server);server=null;
    started=await open(dir);server=started.server;p=started.port;
    assert.deepEqual((await expect(p,200,'GET','/store/ledger.log')).bytes,extended);
    await expect(p,409,'PUT','/store/ledger.log',first,grant(token));
    await expect(p,200,'PUT','/store/ledger.log',
      Buffer.concat([extended,Buffer.from('after restart')]),grant(token));
    console.log('append-only binary prefix, delegation, quotas, restart: OK');
  }finally {
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
