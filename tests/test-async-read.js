#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {sha256Bytes}=require('../src/content-hash.js');
const {open,close,expect,request,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp(),read=fs.read;
  let server,resume=()=>{};
  try {
    const started=await open(dir);
    server=started.server;
    const port=started.port,root=server.skrynia.grants.root();
    const token=(await expect(port,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    const old=Buffer.alloc(192*1024,0x61);
    const updated=Buffer.alloc(192*1024,0x62);
    await expect(port,201,'POST','/store/atomic.bin',old,grant(token));

    // Suspend one disk-read callback. Other requests must remain serviceable
    // and the eventual response must use the inode opened before replacement.
    let entered;
    const hashing=new Promise(resolve=>{entered=resolve;});
    const paused=new Promise(resolve=>{resume=resolve;});
    let delayed=false;
    fs.read=function(fd,buffer,offset,length,position,callback) {
      if(!delayed) {
        delayed=true;entered();
        return read(fd,buffer,offset,length,position,(...result)=>{
          paused.then(()=>callback(...result));
        });
      }
      return read(fd,buffer,offset,length,position,callback);
    };
    const pending=request(port,'GET','/store/atomic.bin');
    await hashing;
    await expect(port,200,'GET','/health');
    await expect(port,200,'PUT','/store/atomic.bin',updated,grant(token));
    resume();
    const response=await pending;
    assert.equal(response.status,200);
    assert.deepEqual(response.bytes,old);
    assert.equal(response.etag,'"'+sha256Bytes(old)+'"');
    const current=await expect(port,200,'GET','/store/atomic.bin');
    assert.deepEqual(current.bytes,updated);
    assert.equal(current.etag,'"'+sha256Bytes(updated)+'"');
    console.log('public hashing yields to requests and serves pinned bytes: OK');
  } finally {
    resume();
    fs.read=read;
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
