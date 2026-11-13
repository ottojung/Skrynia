#!/usr/bin/env node
'use strict';
// Mixed concurrent read/write durability and resident-memory load probe.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {mkdtempSync,rmSync,readdirSync}=fs;
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const http=require('node:http');
const {performance}=require('node:perf_hooks');
const {port,request,start,stop,credentials,sha}=require('../tests/crash/fixture');
const seconds=Number(process.env.SKRYNIA_SOAK_SECONDS||12);
const workers=Number(process.env.SKRYNIA_SOAK_WORKERS||16);
const initial=Buffer.alloc(4096,0x41);
const updated=Buffer.alloc(4096,0x42);
const binary=resolve(process.argv[2]||'target/release/skrynia-server');
const useNode=process.env.SKRYNIA_SOAK_SERVER==='node';
function transport(port) {
  const agent=new http.Agent({keepAlive:true,maxSockets:workers});
  function call(method,key,user,body,headers={}) {
    return new Promise((resolve,reject)=>{
      const args={host:'127.0.0.1',port,method,path:'/crash/store/'+key,agent,
        headers:{...headers,...(user?{Authorization:'Bearer '+user}:{}),
          ...(body?{'Content-Length':body.length}:{})}};
      const req=http.request(args,res=>{
        const chunks=[];
        res.on('data',chunk=>chunks.push(chunk));
        res.on('error',reject);
        res.on('end',()=>resolve({status:res.statusCode,
          headers:res.headers,body:Buffer.concat(chunks)}));
      });
      req.on('error',reject);
      req.setTimeout(4000,()=>req.destroy(Error('HTTP timeout')));
      req.end(body);
    });
  }
  return {call,close:()=>agent.destroy()};
}
function rss(pid) {
  try {
    const line=fs.readFileSync(`/proc/${pid}/status`,'utf8').match(/^VmRSS:\s+(\d+) kB/m);
    return line?Number(line[1])*1024:null;
  }catch{return null;}
}
function percentile(numbers,fraction) {
  if(numbers.length===0)return null;
  const sorted=numbers.toSorted((a,b)=>a-b);
  return Number(sorted[Math.min(sorted.length-1,Math.floor(sorted.length*fraction))].toFixed(2));
}
async function main() {
  assert(Number.isFinite(seconds)&&seconds>=1&&seconds<=3600);
  assert(Number.isInteger(workers)&&workers>=1&&workers<=256);
  const root=mkdtempSync(join(tmpdir(),'skrynia-soak-'));
  let child,client,monitor;
  try {
    const p=await port();
    child=await start(useNode?process.execPath:binary,
      useNode?[resolve('src/server.js')]:[],root,p);
    const grant=credentials(root);
    const response=await request(p,'POST',`/grants/${grant.id}/users`,grant.issuer);
    assert.equal(response.status,200);
    const user=JSON.parse(response.body).userToken;
    client=transport(p);
    const old=Array.from({length:workers},()=>initial);
    for(let i=0;i<workers;i++) {
      const result=await client.call('POST',`stress_${i}`,user,initial);
      assert.equal(result.status,201);
    }
    const samples=[],readLat=[],writeLat=[];
    let peakRSS=rss(child.pid)||0,reads=0,writes=0;
    monitor=setInterval(()=>{peakRSS=Math.max(peakRSS,rss(child.pid)||0);},100);
    const started=performance.now(),end=started+seconds*1000;
    await Promise.all(Array.from({length:workers},async(_,index)=>{
      for(let iteration=0;performance.now()<end;iteration++) {
        const writing=iteration%10===0;
        const key=writing?index:(index*17+iteration*13)%workers;
        const before=performance.now();
        if(writing) {
          const body=old[index]===initial?updated:initial;
          const result=await client.call('PUT',`stress_${key}`,user,body,
            {'X-Skrynia-If-SHA256':sha(old[index])});
          assert.equal(result.status,200,`PUT stress_${key}: ${result.body}`);
          old[index]=body;writes++;writeLat.push(performance.now()-before);
        } else {
          const result=await client.call('GET',`stress_${key}`);
          assert.equal(result.status,200,`GET stress_${key}: ${result.body}`);
          assert(result.body.equals(initial)||result.body.equals(updated),
            `torn read stress_${key}`);
          assert.equal(result.headers.etag,'"'+sha(result.body)+'"');
          reads++;readLat.push(performance.now()-before);
        }
      }
    }));
    const elapsed=(performance.now()-started)/1000;
    clearInterval(monitor);monitor=undefined;
    peakRSS=Math.max(peakRSS,rss(child.pid)||0);
    assert.deepEqual(readdirSync(join(root,'state/store-transactions')),[],
      'journal not drained after sustained mixed load');
    await stop(child,'SIGTERM');child=undefined;client.close();client=undefined;
    child=await start(process.execPath,[resolve('src/server.js')],root,p);
    for(let index=0;index<workers;index++) {
      const value=await request(p,'GET',`/store/stress_${index}`);
      assert.equal(value.status,200);
      assert(value.body.equals(old[index]),`Node restart sees wrong payload stress_${index}`);
    }
    console.log(JSON.stringify({server:useNode?'Node':'Rust',seconds:Number(elapsed.toFixed(2)),workers,reads,writes,
      opsPerSecond:Number(((reads+writes)/elapsed).toFixed(1)),
      readP95ms:percentile(readLat,0.95),readP99ms:percentile(readLat,0.99),
      writeP95ms:percentile(writeLat,0.95),writeP99ms:percentile(writeLat,0.99),
      peakRssMiB:Number((peakRSS/1024/1024).toFixed(1)),
      verified:'complete bytes, matching ETags, drained journal, Node restart'},null,2));
  } finally {
    if(monitor)clearInterval(monitor);
    if(client)client.close();
    if(child)await stop(child,'SIGKILL');
    rmSync(root,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
