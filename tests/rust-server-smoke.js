#!/usr/bin/env node
'use strict';
const assert = require('node:assert/strict');
const {mkdtempSync, readdirSync, readFileSync, writeFileSync, existsSync, mkdirSync, rmSync} = require('node:fs');
const {tmpdir} = require('node:os');
const {join, resolve} = require('node:path');
const {spawn} = require('node:child_process');
const {createHash} = require('node:crypto');
const net = require('node:net');
const http = require('node:http');
const pause = ms => new Promise(done => setTimeout(done, ms));
const sha = value => createHash('sha256').update(value).digest('hex');
async function port() {
  const socket = net.createServer();
  await new Promise(done => socket.listen(0, '127.0.0.1', done));
  const value = socket.address().port;
  await new Promise(done => socket.close(done));
  return value;
}
function request(port,method,path,body,token,extra={}) {
  const bytes=body===undefined?null:Buffer.isBuffer(body)?body:Buffer.from(body);
  const headers={...extra,...(token?{Authorization:'Bearer '+token}:{}),...(bytes?{'Content-Length':bytes.length}:{})};
  return new Promise((resolve,reject)=>{
    const req=http.request({host:'127.0.0.1',port,method,path:'/platform'+path,headers},res=>{
      const parts=[];
      res.on('data',b=>parts.push(b));
      res.on('end',()=>{
        const raw=Buffer.concat(parts);
        let json;try{json=JSON.parse(raw.toString());}catch{}
        resolve({status:res.statusCode,body:raw,json,headers:res.headers});
      });
    });
    req.on('error',reject);
    req.end(bytes);
  });
}
async function start(command,args,dir,port) {
  const child=spawn(command,args,{env:{...process.env,SKRYNIA_DATA_DIR:dir,
    SKRYNIA_URL:'http://127.0.0.1:'+port+'/platform',SKRYNIA_PORT:String(port)},stdio:'ignore'});
  let exited=false;
  child.once('exit',()=>{exited=true;});
  for(let attempt=0;attempt<100;attempt++) {
    if(exited)throw Error('server exited unexpectedly');
    try {if((await request(port,'GET','/health')).status===200)return child;}catch{}
    await pause(30);
  }
  child.kill();throw Error('server did not start');
}
async function stop(child) {
  child.kill('SIGTERM');
  await Promise.race([new Promise(done=>child.once('exit',done)),pause(1500)]);
  if(child.exitCode===null)child.kill('SIGKILL');
}
async function cannotStart(binary,dir,port) {
  const child=spawn(binary,[],{env:{...process.env,SKRYNIA_DATA_DIR:dir,
    SKRYNIA_URL:'http://127.0.0.1:'+port+'/platform',SKRYNIA_PORT:String(port)},stdio:'ignore'});
  const code=await Promise.race([
    new Promise((resolve,reject)=>child.once('exit',value=>resolve(value))),
    pause(1500).then(()=>{child.kill();throw Error('unsafe Rust startup succeeded')})
  ]);
  assert.notEqual(code,0,'Rust must reject unsafe state or concurrent writer');
}
async function run() {
  const dir=mkdtempSync(join(tmpdir(),'skrynia-rust-protocol-'));
  const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
  let server;
  try {
    const p=await port();
    server=await start(process.execPath,[resolve('src/server.js')],dir,p);
    const grant=JSON.parse(readFileSync(join(dir,'state/grants',readdirSync(join(dir,'state/grants'))[0])));
    const root=grant.id;
    const issuer=grant.rootIssuerToken;
    const user=(await request(p,'POST','/grants/'+root+'/users',undefined,issuer)).json.userToken;
    assert.equal((await request(p,'POST','/store/legacy.txt','legacy',user)).status,201);
    const vapidBefore=JSON.parse(readFileSync(join(dir,'state/push/vapid.json')));
    await cannotStart(binary,dir,await port()); // Node already holds the writer lock.
    await stop(server);server=undefined;
    server=await start(binary,[],dir,p);
    assert.deepEqual(JSON.parse(readFileSync(join(dir,'state/push/vapid.json'))),vapidBefore);
    assert.equal((await request(p,'GET','/store/legacy.txt')).body.toString(),'legacy');
    const created=await request(p,'POST','/store/new.txt','abc',user);
    assert.equal(created.status,201);
    const get=await request(p,'GET','/store/new.txt');
    assert.equal(get.headers.etag,'"'+sha('abc')+'"');
    assert.equal((await request(p,'GET','/store/new.txt',undefined,undefined,{'If-None-Match':get.headers.etag})).status,304);
    assert.equal((await request(p,'GET','/store/new.txt',undefined,undefined,{Range:'bytes=1-2'})).body.toString(),'bc');
    assert.equal((await request(p,'PUT','/store/new.txt','changed',user,{'X-Skrynia-If-SHA256':sha('bad')})).status,412);
    assert.equal((await request(p,'PUT','/store/new.txt','changed',user,{'X-Skrynia-If-SHA256':sha('abc')})).status,200);
    assert.equal((await request(p,'GET','/store/new.txt')).body.toString(),'changed');
    assert.equal((await request(p,'PUT','/store/new.txt','error',user,{'If-Match':'*'})).status,400);
    const list=await request(p,'GET','/store?locations=*',undefined,user);
    assert.deepEqual(list.json.keys,['legacy.txt','new.txt']);
    const issued=await request(p,'POST','/grants/'+root+'/users',undefined,issuer);
    assert.equal(issued.status,200);
    const extra=issued.json;
    assert.equal((await request(p,'POST','/store/temporary','yes',extra.userToken)).status,201);
    assert.equal((await request(p,'DELETE','/grants/'+root+'/users/'+extra.userId,undefined,issuer)).status,200);
    assert.equal((await request(p,'POST','/store/denied','no',extra.userToken)).status,403);
    const child=(await request(p,'POST','/grants',JSON.stringify({locations:'team.*',
      operations:['create','list','subscribe'],maxBytes:30000,maxObjects:5}),user)).json;
    assert.equal(typeof child.issuerToken,'string');
    const contributor=(await request(p,'POST','/grants/'+child.id+'/users',undefined,child.issuerToken)).json.userToken;
    // Persisted browser subscriptions must survive both Rust and Node readers.
    const crypto=require('node:crypto');
    const ecdh=crypto.createECDH('prime256v1');ecdh.generateKeys();
    const keys={p256dh:ecdh.getPublicKey().toString('base64url'),
      auth:crypto.randomBytes(16).toString('base64url')};
    const subscription={locations:'team.*',events:['create','replace','delete'],
      endpoint:'https://push.example.com/some-id',keys};
    const createdSub=await request(p,'POST','/push/subscriptions',JSON.stringify(subscription),contributor);
    assert.equal(createdSub.status,201,JSON.stringify(createdSub.json));
    const subId=createdSub.json.id;
    const liveSubFile=join(dir,'state/push-subs',subId+'.json');
    assert.equal(JSON.parse(readFileSync(liveSubFile)).grantId,child.id);
    const vapid=JSON.parse(readFileSync(join(dir,'state/push/vapid.json')));
    assert.equal((await request(p,'GET','/push/vapid')).json.publicKey,vapid.publicKey);
    assert.equal((await request(p,'POST','/push/subscriptions',JSON.stringify(subscription),contributor)).status,403);
    const rotated=await request(p,'POST','/push/subscriptions',JSON.stringify(subscription),contributor,
      {'X-Skrynia-Capability':createdSub.json.capability});
    assert.equal(rotated.status,201);
    assert.equal(rotated.json.deduped,true);
    assert.notEqual(rotated.json.capability,createdSub.json.capability);
    assert.equal((await request(p,'PUT','/push/subscriptions/'+subId,
      JSON.stringify({endpoint:'https://push2.example.com/v1'}),undefined,
      {'X-Skrynia-Capability':rotated.json.capability})).status,200);
    const storedSub=JSON.parse(readFileSync(liveSubFile));
    assert.equal(storedSub.endpoint,'https://push2.example.com/v1');
    assert.equal(storedSub.capHash,sha(rotated.json.capability));
    const jsPush=require('../src/push.js').createPush({dataDir:dir,pushManual:true});
    assert.equal(jsPush.findSub(subId).capHash,storedSub.capHash);
    assert.equal(jsPush.getPublicKey(),vapid.publicKey);
    assert.equal((await request(p,'DELETE','/push/subscriptions/'+subId,undefined,undefined,
      {'X-Skrynia-Capability':createdSub.json.capability})).status,403);
    assert.equal((await request(p,'DELETE','/push/subscriptions/'+subId,undefined,undefined,
      {'X-Skrynia-Capability':rotated.json.capability})).status,200);
    assert.equal(existsSync(liveSubFile),false);
    assert.equal((await request(p,'POST','/store/team.a','member',contributor)).status,201);
    assert.equal((await request(p,'PUT','/store/team.a','wrong',contributor)).status,403);
    assert.equal((await request(p,'GET','/store?locations=team.*',undefined,contributor)).json.keys[0],'team.a');
    assert.equal((await request(p,'DELETE','/store/new.txt',undefined,user,{'X-Skrynia-If-SHA256':sha('changed')})).status,200);
    await stop(server);server=undefined;
    server=await start(process.execPath,[resolve('src/server.js')],dir,p);
    assert.equal((await request(p,'GET','/store/team.a')).body.toString(),'member');
    assert.equal((await request(p,'GET','/store/new.txt')).status,404);
    await stop(server);server=undefined;
    // A committed object with a legacy transaction marker must be recoverable.
    const txDir=join(dir,'state/store-transactions');
    const txFile=join(txDir,'f'.repeat(32)+'.json');
    writeFileSync(txFile,JSON.stringify({key:'team.a',kind:'replace',sha256:sha('member')}));
    server=await start(binary,[],dir,p);
    assert.equal(existsSync(txFile),false);
    await stop(server);server=undefined;
    // A pending grant revocation must finish before serving requests.
    const revokeDir=join(dir,'state/grant-revocations');
    mkdirSync(revokeDir,{recursive:true});
    const marker=join(revokeDir,child.id+'.json');
    writeFileSync(marker,JSON.stringify({id:child.id,ids:[child.id]}));
    server=await start(binary,[],dir,p);
    assert.equal(existsSync(marker),false);
    assert.equal((await request(p,'GET','/store/team.a')).body.toString(),'member');
    assert.equal((await request(p,'POST','/store/team.b','unauthorized',contributor)).status,403);
    await stop(server);server=undefined;
    // An unknown pending push subscription must not be silently discarded.
    const subs=join(dir,'state/push-subs');mkdirSync(subs,{recursive:true});
    const subFile=join(subs,'f'.repeat(32)+'.json');
    writeFileSync(subFile,'{}');
    await cannotStart(binary,dir,p);
    assert.equal(existsSync(subFile),true);
    rmSync(subFile);
    server=await start(process.execPath,[resolve('src/server.js')],dir,p);
    assert.equal((await request(p,'GET','/store/team.a')).body.toString(),'member');
    console.log('Rust/Node recovery, CAS, grant authority and startup safety: OK');
  } finally {if(server)await stop(server);rmSync(dir,{recursive:true,force:true});}
}
run().catch(e=>{console.error(e);process.exitCode=1;});
