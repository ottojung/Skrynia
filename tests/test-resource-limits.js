#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const http=require('node:http');
const {createShared}=require('../src/shared.js');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();let server,pending;
  try {
    const shared=createShared(dir);
    shared.ensureStoreRoots();
    for(let i=0;i<1005;i++) {
      fs.writeFileSync(path.join(shared.STORE_DIR,
        'entry.'+String(i).padStart(4,'0')),'x');
    }
    const started=await open(dir,{maxBufferedUploadBytes:1024,maxConnections:128});
    server=started.server;
    assert.equal(server.maxConnections,128);
    const p=started.port,root=server.skrynia.grants.root();
    const token=(await expect(p,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    const list=async suffix=>(await expect(p,200,'GET',
      '/store?locations=entry.*'+suffix,null,grant(token))).json;

    const first=await list('');
    assert.equal(first.keys.length,1000);
    assert.equal(first.next,'entry.0999');
    const second=await list('&after='+first.next);
    assert.deepEqual(second.keys,['entry.1000','entry.1001','entry.1002',
      'entry.1003','entry.1004']);
    assert.equal(second.next,undefined);
    const custom=await list('&limit=2');
    assert.deepEqual(custom,{keys:['entry.0000','entry.0001'],next:'entry.0001'});
    await expect(p,400,'GET','/store?locations=entry.*&limit=1001',
      null,grant(token));

    // Hold a partial request in memory while a competing upload is refused.
    pending=http.request({host:'127.0.0.1',port:p,
      path:'/platform/store/buffer.first',method:'POST',
      headers:{...auth(token),'Content-Length':1000}});
    const completed=new Promise((resolve,reject)=>{
      pending.on('response',res=>{
        res.resume();res.on('end',()=>resolve(res.statusCode));
      });
      pending.on('error',reject);
    });
    pending.write(Buffer.alloc(800));
    const deadline=Date.now()+2500;
    while(server.skrynia.bufferedUploadBytes()<800) {
      if(Date.now()>deadline) throw new Error('upload did not reserve memory');
      await new Promise(resolve=>setTimeout(resolve,5));
    }
    await expect(p,503,'POST','/store/buffer.second',
      Buffer.alloc(800),grant(token));
    assert.equal(server.skrynia.bufferedUploadBytes(),800);
    pending.end(Buffer.alloc(200));
    assert.equal(await completed,201);
    assert.equal(server.skrynia.bufferedUploadBytes(),0);
    await expect(p,201,'POST','/store/buffer.second',
      Buffer.alloc(800),grant(token));
    assert.equal(server.skrynia.bufferedUploadBytes(),0);

    // The declared length can be rejected from headers alone.
    await new Promise((resolve,reject)=>{
      const req=http.request({host:'127.0.0.1',port:p,
        path:'/platform/store/too.big',method:'POST',
        headers:{...auth(token),'Content-Length':1500}},res=>{
        assert.equal(res.statusCode,503);
        res.resume();res.on('end',resolve);
      });
      req.on('error',reject);
      req.setTimeout(1500,()=>req.destroy(new Error('early refusal timed out')));
      req.flushHeaders();
    });
    assert.equal(server.skrynia.bufferedUploadBytes(),0);
    console.log('bounded listing, upload memory and connection limits: OK');
  } finally {
    if(pending) pending.destroy();
    if(server) await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
