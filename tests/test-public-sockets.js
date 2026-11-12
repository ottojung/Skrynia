#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const http=require('node:http');
const net=require('node:net');
const {createShared}=require('../src/shared.js');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();
  let server;
  try {
    const shared=createShared(dir);
    shared.ensureStoreRoots();
    const large=path.join(shared.STORE_DIR,'large.binary');
    fs.writeFileSync(large,Buffer.alloc(16*1024*1024,0x41));
    const started=await open(dir,{socketIdleTimeoutMs:150});
    server=started.server;
    const port=started.port;
    const root=server.skrynia.grants.root();
    const token=(await expect(port,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    const countFD=()=>fs.readdirSync('/proc/self/fd').length;
    const baseline=countFD();

    for(let i=0;i<50;i++) {
      await new Promise((resolve,reject)=>{
        const req=http.get({
          host:'127.0.0.1',port,
          path:'/platform/store/large.binary',
          agent:false
        },response=>{
          assert.equal(response.statusCode,200);
          // Destroy before receiving the entire 16 MiB response.
          req.destroy();
          resolve();
        });
        req.on('error',e=>{
          if(e.code!=='ECONNRESET') reject(e);
        });
        req.setTimeout(1500,()=>req.destroy(new Error('download timed out')));
      });
    }

    let count=countFD(),deadline=Date.now()+2500;
    while(count>baseline+4&&Date.now()<deadline) {
      await new Promise(resolve=>setTimeout(resolve,20));
      count=countFD();
    }
    assert(count<=baseline+4,
      'aborted GETs leaked descriptors: initial '+baseline+', now '+count);

    // Synchronous read errors must return 500 rather than killing Node.
    const original=fs.openSync,originalError=console.error;
    console.error=()=>{};
    const forbidden=path.join(shared.STORE_DIR,'large.binary');
    fs.openSync=function(file,...args) {
      if(file===forbidden) {
        const e=new Error('simulated EACCES');e.code='EACCES';
        throw e;
      }
      return original.call(this,file,...args);
    };
    try {
      await expect(port,500,'GET','/store/large.binary');
    } finally {
      fs.openSync=original;
      console.error=originalError;
    }
    await expect(port,200,'GET','/health');
    await expect(port,200,'PUT','/store/large.binary','still alive',grant(token));

    // HTTP connections that send no request bytes cannot remain forever.
    const socket=net.createConnection({host:'127.0.0.1',port});
    await new Promise(resolve=>socket.once('connect',resolve));
    await Promise.race([
      new Promise(resolve=>socket.once('close',resolve)),
      new Promise((_,reject)=>setTimeout(()=>
        reject(new Error('idle connection was not closed')),1500))
    ]);
    console.log('aborted downloads close file handles, errors survive, idle sockets close: OK');
  } finally {
    if(server) await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
