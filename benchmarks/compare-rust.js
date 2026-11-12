#!/usr/bin/env node
'use strict';
// Same-host, same API workload comparison. Neither runtime is assumed faster.
const {mkdtempSync,rmSync,readFileSync,readdirSync} = require('node:fs');
const {tmpdir} = require('node:os');
const {join,resolve} = require('node:path');
const {spawn} = require('node:child_process');
const {createHash} = require('node:crypto');
const {performance} = require('node:perf_hooks');
const http = require('node:http');
const net = require('node:net');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const sleep = ms => new Promise(done => setTimeout(done,ms));
async function port() {
  const server=net.createServer();
  await new Promise(done=>server.listen(0,'127.0.0.1',done));
  const result=server.address().port;
  await new Promise(done=>server.close(done));
  return result;
}
function client(port) {
  const agent=new http.Agent({keepAlive:true,maxSockets:16});
  function call(method,path,token,body,headers={}) {
    const bytes=body==null?null:Buffer.isBuffer(body)?body:Buffer.from(body);
    return new Promise((resolve,reject)=>{
      const request=http.request({host:'127.0.0.1',port,agent,method,path:'/bench'+path,
        headers:{...headers,...(token?{Authorization:'Bearer '+token}:{}),
          ...(bytes?{'Content-Length':bytes.length}:{})}},response=>{
        const chunks=[];
        response.on('data',chunk=>chunks.push(chunk));
        response.on('error',reject);
        response.on('end',()=>resolve({status:response.statusCode,
          headers:response.headers,body:Buffer.concat(chunks)}));
      });
      request.on('error',reject);
      request.end(bytes);
    });
  }
  return {call,close:()=>agent.destroy()};
}
function ok(response,status) {
  if(response.status!==status) throw Error(`Expected ${status}, got ${response.status}: ${response.body}`);
  return response;
}
async function launch(binary,args,dir,port) {
  const process_=spawn(binary,args,{stdio:'ignore',env:{...process.env,
    SKRYNIA_DATA_DIR:dir,SKRYNIA_PORT:String(port),
    SKRYNIA_URL:`http://127.0.0.1:${port}/bench`,SKRYNIA_PUSH_MANUAL:'1'}});
  let exited=false;process_.once('exit',()=>{exited=true;});
  const link=client(port);
  for(let attempt=0;attempt<120;attempt++) {
    if(exited) throw Error(`${binary} exited during startup`);
    try {if((await link.call('GET','/health')).status===200)return {process_,link};}
    catch {}
    await sleep(25);
  }
  process_.kill();throw Error(`${binary} failed to start`);
}
async function stop(service) {
  service.link.close();service.process_.kill('SIGTERM');
  await Promise.race([new Promise(done=>service.process_.once('exit',done)),sleep(1000)]);
  if(service.process_.exitCode===null) service.process_.kill('SIGKILL');
}
function metric(label,samples,elapsed) {
  const sorted=samples.toSorted((a,b)=>a-b);
  return {operation:label,operations:sorted.length,
    opsPerSecond:Number((sorted.length*1000/elapsed).toFixed(1)),
    p50Ms:Number(sorted[Math.floor(sorted.length*0.50)].toFixed(2)),
    p95Ms:Number(sorted[Math.min(sorted.length-1,Math.floor(sorted.length*0.95))].toFixed(2))};
}
async function measure(label,items,concurrency,action) {
  const timings=[];let next=0;
  const started=performance.now();
  await Promise.all(Array.from({length:concurrency},async()=>{
    while(next<items.length) {
      const item=items[next++];const a=performance.now();
      await action(item);timings.push(performance.now()-a);
    }
  }));
  return metric(label,timings,performance.now()-started);
}
async function run(name,bin,args,objects=64) {
  const dir=mkdtempSync(join(tmpdir(),`skrynia-compare-${name}-`));
  let service;
  try {
    const p=await port();service=await launch(bin,args,dir,p);
    const {call}=service.link;
    const file=join(dir,'state/grants',readdirSync(join(dir,'state/grants'))[0]);
    const root=JSON.parse(readFileSync(file));
    const user=JSON.parse(ok(await call('POST',`/grants/${root.id}/users`,root.rootIssuerToken),200).body).userToken;
    const data=Buffer.alloc(32768,0x41);
    for(let i=0;i<objects;i++) ok(await call('POST',`/store/bench_${i}.bin`,user,data),201);
    const keys=Array.from({length:300},(_,i)=>i%objects);
    for(const key of keys.slice(0,20)) ok(await call('GET',`/store/bench_${key}.bin`),200);
    const results=[];
    results.push(await measure('public GET 32KiB',keys,8,async key=>{
      const response=ok(await call('GET',`/store/bench_${key}.bin`),200);
      if(response.body.length!==data.length) throw Error('short GET');
    }));
    const oldHash=sha(data);
    results.push(await measure('CAS PUT 8KiB',Array.from({length:32},(_,i)=>i),4,async i=>{
      ok(await call('PUT',`/store/bench_${i}.bin`,user,Buffer.alloc(8192,0x42),
        {'X-Skrynia-If-SHA256':oldHash}),200);
    }));
    results.push(await measure('child grant creation',Array.from({length:24},(_,i)=>i),4,async i=>{
      const spec={locations:`child_${i}.*`,maxBytes:25000,maxObjects:10,
        operations:['create','list']};
      const result=ok(await call('POST','/grants',user,JSON.stringify(spec)),200);
      if(!JSON.parse(result.body).issuerToken)throw Error('missing issuerToken');
    }));
    return {server:name,results};
  } finally {if(service)await stop(service);rmSync(dir,{recursive:true,force:true});}
}
async function main() {
  const repetitions=Number(process.env.SKRYNIA_BENCH_REPETITIONS||2);
  const objects=Number(process.env.SKRYNIA_BENCH_OBJECTS||64);
  if(!(repetitions>=1&&repetitions<=5&&Number.isInteger(repetitions)))throw Error('invalid repetitions');
  const configurations=[['Node',process.execPath,[resolve('src/server.js')]],
    ['Rust',resolve(process.argv[2]||'target/debug/skrynia-server'),[]]];
  const trials=[];
  for(let i=0;i<repetitions;i++) {
    for(const config of i%2 ? configurations.toReversed() : configurations) {
      const result=await run(...config,objects);
      console.log(`Trial ${i+1}: ${result.server}`,JSON.stringify(result.results));
      trials.push({trial:i+1,...result});
    }
  }
  console.log(JSON.stringify({schema:'skrynia-rust-node-comparison-v1',environment:{
    node:process.version,platform:process.platform,arch:process.arch},objects,repetitions,trials},null,2));
}
main().catch(error=>{console.error(error);process.exitCode=1;});
