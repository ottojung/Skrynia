#!/usr/bin/env node
'use strict';
// Repeat on the intended filesystem; numbers are observations, not targets.
const fs=require('node:fs');
const os=require('node:os');
const path=require('node:path');
const http=require('node:http');
const {performance,monitorEventLoopDelay}=require('node:perf_hooks');
const {sha256Bytes}=require('../src/content-hash.js');
const {createServer}=require('../src/server.js');

function count(name,defaultValue,max=10000) {
  const value=Number(process.env[name]||defaultValue);
  if(!Number.isSafeInteger(value)||value<1||value>max)
    throw new Error('invalid '+name);
  return value;
}
const objects=count('SKRYNIA_BENCH_OBJECTS',64);
const grants=count('SKRYNIA_BENCH_GRANTS',48);
const reads=count('SKRYNIA_BENCH_READS',100);
const writes=count('SKRYNIA_BENCH_WRITES',24);
const size=count('SKRYNIA_BENCH_BYTES',32768,10485760);

function quantile(numbers,p) {
  const sorted=[...numbers].sort((a,b)=>a-b);
  return sorted[Math.min(sorted.length-1,Math.ceil(p*sorted.length)-1)].toFixed(2);
}
async function sample(label,n,fn,parallel=1) {
  const values=[];
  const began=performance.now();
  let next=0;
  async function worker() {
    while(next<n) {
      const index=next++;
      const start=performance.now();
      await fn(index);
      values.push(performance.now()-start);
    }
  }
  await Promise.all(Array.from({length:Math.min(parallel,n)},worker));
  const seconds=(performance.now()-began)/1000;
  console.log(label+': '+n+' ops, '+(n/seconds).toFixed(1)+
    '/s, p50 '+quantile(values,0.5)+'ms, p95 '+quantile(values,0.95)+'ms');
}
async function run() {
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),'skrynia-benchmark-'));
  let server;
  try {
    server=createServer({dataDir:dir,pushManual:true,
      skryniaUrl:'http://127.0.0.1:17380/platform'});
    await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
    const port=server.address().port;
    async function request(method,url,body=null,token=null,extra={}) {
      const bytes=body===null?null:Buffer.isBuffer(body)?body:
        Buffer.from(JSON.stringify(body));
      const headers={...extra};
      if(token)headers.Authorization='Bearer '+token;
      if(bytes)headers['Content-Length']=bytes.length;
      return new Promise((resolve,reject)=>{
        const req=http.request({host:'127.0.0.1',port,
          path:'/platform'+url,method,headers},res=>{
          const parts=[];
          res.on('data',part=>parts.push(part));
          res.on('end',()=>{
            const data=Buffer.concat(parts);
            if(res.statusCode<200||res.statusCode>=300)
              return reject(new Error(method+' '+url+': '+res.statusCode+
                ' '+data.toString()));
            let json=null;
            try {json=JSON.parse(data.toString());}catch {}
            resolve({bytes:data,json});
          });
        });
        req.on('error',reject);
        req.end(bytes);
      });
    }
    const root=server.skrynia.grants.root();
    const token=(await request('POST','/grants/'+root.id+'/users',
      null,root.rootIssuerToken)).json.userToken;
    const blob=Buffer.alloc(size,0x41);
    for(let i=0;i<objects;i++)
      await request('POST','/store/bench.'+i,blob,token);
    await request('POST','/store/bench.head',Buffer.from('initial'),token);
    const monitor=monitorEventLoopDelay({resolution:10});
    monitor.enable();
    console.log('objects='+objects+', bytes/object='+size+
      ', grants='+grants+', Node='+process.version);
    await sample('Public GET',reads,async i=>{
      const r=await request('GET','/store/bench.'+(i%objects));
      if(r.bytes.length!==size)throw new Error('short GET');
    },8);
    await sample('Child grant creation',grants,async i=>{
      await request('POST','/grants',{
        locations:'scope_'+i+'.*',
        operations:['create','modify','list'],
        maxBytes:100000,maxObjects:10
      },token);
    });
    await sample('Grant quota inspection',reads,async()=>{
      await request('GET','/grants/'+root.id,null,token);
    });
    let head=Buffer.from('initial');
    await sample('Conditional PUT',writes,async i=>{
      const next=Buffer.from('revision '+i);
      await request('PUT','/store/bench.head',next,token,{
        'X-Skrynia-If-SHA256':sha256Bytes(head)
      });
      head=next;
    });
    monitor.disable();
    console.log('event-loop lag: p95 '+
      (monitor.percentile(95)/1e6).toFixed(2)+'ms, max '+
      (monitor.max/1e6).toFixed(2)+'ms');
  } finally {
    if(server)await new Promise(resolve=>server.close(resolve));
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
