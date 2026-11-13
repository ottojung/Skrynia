#!/usr/bin/env node
'use strict';
// Drive a separately started memory-limited Rust container over real HTTP.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const http=require('node:http');
const {join,resolve}=require('node:path');
const {createHash}=require('node:crypto');
const {execFileSync}=require('node:child_process');
const root=resolve(process.argv[2]);
const p=Number(process.argv[3]);
const workers=Number(process.env.SKRYNIA_CONTAINER_WORKERS||24);
const bytes=Number(process.env.SKRYNIA_CONTAINER_PAYLOAD_BYTES||4194304);
function ask(method,path,token,body) {
  return new Promise((done,fail)=>{
    const req=http.request({host:'127.0.0.1',port:p,method,path:'/crash'+path,
      agent:false,headers:{...(token?{Authorization:'Bearer '+token}:{}),
        ...(body?{'Content-Length':body.length}:{})}},res=>{
      const chunks=[];res.on('data',b=>chunks.push(b));
      res.on('error',fail);
      res.on('end',()=>done({status:res.statusCode,body:Buffer.concat(chunks),
        headers:res.headers}));
    });
    req.setTimeout(15000,()=>req.destroy(Error('container request timeout')));
    req.on('error',fail);req.end(body);
  });
}
function hash(body) {return createHash('sha256').update(body).digest('hex');}
async function main() {
  assert(workers>=2&&workers<=64&&bytes>=4096&&bytes<=10485760);
  const raw=execFileSync(process.env.SKRYNIA_CONTAINER_ENGINE||'podman',
    ['exec','-u','0',process.argv[4],'sh','-c',
      'cat /var/lib/skrynia/state/grants/*.json'],{encoding:'utf8'});
  const record=JSON.parse(raw);
  const grant={id:record.id,issuer:record.rootIssuerToken};
  const issued=await ask('POST',`/grants/${grant.id}/users`,grant.issuer);
  assert.equal(issued.status,200,issued.body.toString());
  const user=JSON.parse(issued.body).userToken;
  const bodies=Array.from({length:workers},(_,i)=>Buffer.alloc(bytes,65+i%25));
  const outcomes=await Promise.all(bodies.map((body,i)=>
    ask('POST',`/store/pressure_${i}`,user,body).catch(error=>{
      // Early 503 can close the upload socket before the sender finishes.
      if(['EPIPE','ECONNRESET'].includes(error.code))return {status:0,body:Buffer.alloc(0)};
      throw error;
    })));
  const accepted=[];
  outcomes.forEach((r,i)=>{
    assert([0,201,503].includes(r.status),`upload ${i}: ${r.status} ${r.body}`);
    if(r.status===503)assert.equal(JSON.parse(r.body).error,'upload_memory_full');
    else if(r.status===201)accepted.push(i);
  });
  assert(accepted.length>0,'no successful upload despite available memory');
  assert.equal((await ask('GET','/health')).status,200);
  for(const i of accepted) {
    const response=await ask('GET',`/store/pressure_${i}`);
    assert.equal(response.status,200);
    assert(response.body.equals(bodies[i]),`torn stored object ${i}`);
    assert.equal(response.headers.etag,'"'+hash(bodies[i])+'"');
  }
  // A lost HTTP reply is ambiguous: the write might have committed.
  // All names, including rejected or disconnected ones, must be absent or
  // contain the complete original upload bytes and its SHA-256 ETag.
  for(let i=0;i<workers;i++) {
    const response=await ask('GET',`/store/pressure_${i}`);
    if(outcomes[i].status===503)assert.equal(response.status,404);
    else if(response.status===200) {
      assert(response.body.equals(bodies[i]),`torn or unexpected bytes ${i}`);
      assert.equal(response.headers.etag,'"'+hash(bodies[i])+'"');
    } else assert.equal(response.status,404);
    if(outcomes[i].status===201)assert.equal(response.status,200);
  }
  assert.equal((await ask('GET','/health')).status,200);
  console.log(`Container memory pressure: ${workers} x ${bytes}B uploads, `+
    `${accepted.length} accepted, ${outcomes.filter(r=>r.status===503).length} HTTP 503, `+
    `${outcomes.filter(r=>r.status===0).length} early socket closures, `+
    'all surviving objects and journal valid: OK');
}
main().catch(e=>{console.error(e);process.exitCode=1;});
