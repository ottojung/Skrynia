#!/usr/bin/env node
'use strict';
// The GET representation and ETag must describe the same pinned inode.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {mkdtempSync,rmSync}=fs;
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const http=require('node:http');
const {port,request,start,stop,credentials,sha}=require('./crash/fixture');
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
const first=Buffer.alloc(4*1024*1024,0x41);
const second=Buffer.alloc(4*1024*1024,0x42);
function streamDuringReplace(portNumber,user) {
  return new Promise((resolve,reject)=>{
    const req=http.get({host:'127.0.0.1',port:portNumber,
      path:'/crash/store/large.data',agent:false},res=>{
      const chunks=[];
      let replacing;
      res.on('error',reject);
      res.on('data',chunk=>{
        chunks.push(chunk);
        if(!replacing) {
          res.pause();
          replacing=request(portNumber,'PUT','/store/large.data',user,second,
            {'X-Skrynia-If-SHA256':sha(first)});
          replacing.then(response=>{
            assert.equal(response.status,200,response.body.toString());
            res.resume();
          }).catch(reject);
        }
      });
      res.on('end',async()=>{
        try {
          await replacing;
          resolve({status:res.statusCode,headers:res.headers,body:Buffer.concat(chunks)});
        }catch(error){reject(error);}
      });
    });
    req.on('error',reject);
    req.setTimeout(15000,()=>req.destroy(Error('stream timed out')));
  });
}
async function main() {
  const root=mkdtempSync(join(tmpdir(),'skrynia-pinned-read-'));
  let child;
  try {
    const p=await port();
    child=await start(binary,[],root,p);
    const grant=credentials(root);
    const issued=await request(p,'POST',`/grants/${grant.id}/users`,grant.issuer);
    assert.equal(issued.status,200);
    const user=JSON.parse(issued.body).userToken;
    assert.equal((await request(p,'POST','/store/large.data',user,first)).status,201);
    const streamed=await streamDuringReplace(p,user);
    assert.equal(streamed.status,200);
    assert(streamed.body.equals(first),'pinned GET was torn by concurrent PUT');
    assert.equal(streamed.headers.etag,'"'+sha(first)+'"');
    const newer=await request(p,'GET','/store/large.data');
    assert.equal(newer.status,200);
    assert(newer.body.equals(second),'later GET did not observe replacement');
    assert.equal(newer.headers.etag,'"'+sha(second)+'"');
    const range=await request(p,'GET','/store/large.data',undefined,undefined,
      {Range:'bytes=100-199'});
    assert.equal(range.status,206);
    assert(range.body.equals(second.subarray(100,200)));
    await stop(child,'SIGTERM');child=undefined;
    child=await start(process.execPath,[resolve('src/server.js')],root,p);
    const reopened=await request(p,'GET','/store/large.data');
    assert.equal(reopened.status,200);
    assert(reopened.body.equals(second),'Node cannot read large Rust replacement');
    console.log('Pinned 4MiB GET, overlapping CAS PUT, ETags, range, Node restart: OK');
  } finally {
    if(child)await stop(child,'SIGKILL');
    rmSync(root,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
