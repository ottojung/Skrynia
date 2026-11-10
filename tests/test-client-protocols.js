#!/usr/bin/env node
'use strict';
// A logical transaction touches two distinct objects and a manifest;
// one ETag CAS on db_head is its only commit point.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();let server;
  try {
    const started=await open(dir);server=started.server;
    const p=started.port,root=server.skrynia.grants.root();
    const token=(await expect(p,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    async function stage(label,alice,bob) {
      const a='acct_a_'+label,b='acct_b_'+label,m='manifest_'+label;
      await expect(p,201,'POST','/store/'+a,{balance:alice},grant(token));
      await expect(p,201,'POST','/store/'+b,{balance:bob},grant(token));
      await expect(p,201,'POST','/store/'+m,{alice:a,bob:b},grant(token));
      return m;
    }
    async function get(key) {
      return expect(p,200,'GET','/store/'+key);
    }
    async function read(key) {
      return JSON.parse((await get(key)).bytes.toString());
    }
    async function snapshot(head) {
      const m=await read(head.manifest);
      return {alice:(await read(m.alice)).balance,
        bob:(await read(m.bob)).balance};
    }
    const initial=await stage('initial',10,10);
    await expect(p,201,'POST','/store/db_head',{revision:0,manifest:initial},grant(token));
    const old=await get('db_head');
    const oldHead=JSON.parse(old.bytes.toString());
    assert.deepEqual(await snapshot(oldHead),{alice:10,bob:10});
    const first=await stage('first',8,12);
    const second=await stage('second',7,13);
    assert.deepEqual(await read('db_head'),oldHead);
    await expect(p,200,'PUT','/store/db_head',{revision:1,manifest:first},
      {...grant(token),'If-Match':old.etag});
    await expect(p,412,'PUT','/store/db_head',{revision:1,manifest:second},
      {...grant(token),'If-Match':old.etag});
    const committed=await get('db_head');
    const committedHead=JSON.parse(committed.bytes.toString());
    assert.deepEqual(await snapshot(committedHead),{alice:8,bob:12});
    assert.deepEqual(await snapshot(oldHead),{alice:10,bob:10});
    const retry=await stage('retry',5,15);
    await expect(p,200,'PUT','/store/db_head',{revision:2,manifest:retry},
      {...grant(token),'If-Match':committed.etag});
    assert.deepEqual(await snapshot(await read('db_head')),{alice:5,bob:15});
    assert.deepEqual(await snapshot(committedHead),{alice:8,bob:12});
    console.log('client-constructed atomic logical transactions: OK');
  }finally{
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
