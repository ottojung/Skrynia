#!/usr/bin/env node
'use strict';
// Compare the existing Node contract and the Rust preview, on isolated data.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const os=require('node:os');
const path=require('node:path');
const http=require('node:http');
const net=require('node:net');
const crypto=require('node:crypto');
const {spawn}=require('node:child_process');
const sha=value=>crypto.createHash('sha256').update(value).digest('hex');
const sleep=ms=>new Promise(done=>setTimeout(done,ms));
async function freePort() {
  const server=net.createServer();
  await new Promise(done=>server.listen(0,'127.0.0.1',done));
  const port=server.address().port;
  await new Promise(done=>server.close(done));
  return port;
}
function request(port,method,route,token,body,extra={}) {
  const bytes=body==null?null:Buffer.from(body);
  return new Promise((resolve,reject)=>{
    const req=http.request({host:'127.0.0.1',port,method,path:'/parity'+route,agent:false,
      headers:{...extra,...(token?{Authorization:'Bearer '+token}:{}),
        ...(bytes?{'Content-Length':bytes.length}:{})}},res=>{
      const chunks=[];
      res.on('data',chunk=>chunks.push(chunk));
      res.on('error',reject);
      res.on('end',()=>{
        const body=Buffer.concat(chunks).toString();
        let json;try{json=JSON.parse(body);}catch{}
        resolve({status:res.statusCode,body,headers:res.headers,json});
      });
    });
    req.on('error',reject);req.end(bytes);
  });
}
async function start(name,command,args) {
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),'skrynia-parity-'+name+'-'));
  const port=await freePort();
  const child=spawn(command,args,{stdio:'ignore',env:{...process.env,SKRYNIA_DATA_DIR:dir,
    SKRYNIA_URL:`http://127.0.0.1:${port}/parity`,SKRYNIA_PORT:String(port),
    SKRYNIA_PUSH_MANUAL:'1'}});
  let dead=false;child.once('exit',()=>{dead=true;});
  for(let i=0;i<100;i++) {
    if(dead)throw Error(name+' exited before ready');
    try {if((await request(port,'GET','/health')).status===200)break;}
    catch {}
    await sleep(20);
    if(i===99)throw Error(name+' did not start');
  }
  const file=path.join(dir,'state/grants',fs.readdirSync(path.join(dir,'state/grants'))[0]);
  const root=JSON.parse(fs.readFileSync(file));
  const issued=await request(port,'POST',`/grants/${root.id}/users`,root.rootIssuerToken);
  assert.equal(issued.status,200);
  return {name,child,dir,port,root,user:issued.json.userToken};
}
async function stop(state) {
  state.child.kill('SIGTERM');
  await Promise.race([new Promise(done=>state.child.once('exit',done)),sleep(1200)]);
  if(state.child.exitCode===null&&state.child.signalCode===null)state.child.kill('SIGKILL');
  fs.rmSync(state.dir,{recursive:true,force:true});
}
function normalize(result,compareBody) {
  const output={status:result.status,error:result.json?.error};
  if(compareBody) {
    output.body=result.body;
    const keys=['etag','content-range','cache-control'];
    if(result.status>=200&&result.status<300)keys.push('content-length');
    for(const key of keys) {
      if(result.headers[key]!=null)output[key]=result.headers[key];
    }
  }
  return output;
}
async function main() {
  const states=[];const failures=[];
  try {
    states.push(await start('Node',process.execPath,[path.resolve('src/server.js')]));
    states.push(await start('Rust',path.resolve(process.argv[2]||'target/debug/skrynia-server'),[]));
    let checked=0;
    async function compare(label,method,route,{token='user',body,headers={},compareBody=false}={}) {
      const results=await Promise.all(states.map(async state=>{
        const routeFor=typeof route==='function'?route(state):route;
        const auth=token==='user'?state.user:token==='issuer'?state.root.rootIssuerToken:
          token==='bad'?'f'.repeat(64):null;
        return request(state.port,method,routeFor,auth,body,headers);
      }));
      const normalized=results.map(value=>normalize(value,compareBody));
      try {assert.deepEqual(normalized[1],normalized[0]);}
      catch {failures.push({label,node:normalized[0],rust:normalized[1]});}
      checked++;
      return results;
    }
    await compare('health','GET','/health');
    await compare('missing object','GET','/store/missing', {compareBody:true});
    await compare('leading dot','GET','/store/.hidden');
    await compare('bad escape','GET','/store/%ZZ');
    await compare('raw ampersand','POST','/store/alias&x=other',{body:'attack'});
    await compare('encoded ampersand','POST','/store/alias%26x%3Dother',{body:'attack'});
    await compare('anonymous upload','POST','/store/x',{body:'abc',token:'none'});
    await compare('wrong token upload','POST','/store/x',{body:'abc',token:'bad'});
    await compare('disallowed conditional creation','POST','/store/x',{
      body:'abc',headers:{'If-Match':'*'}});
    await compare('malformed SHA conditional creation','POST','/store/x',{
      body:'abc',headers:{'X-Skrynia-If-SHA256':'bad'}});
    await compare('create','POST','/store/x',{body:'abc'});
    await compare('duplicate','POST','/store/x',{body:'abc'});
    await compare('read','GET','/store/x',{compareBody:true});
    await compare('head','HEAD','/store/x',{compareBody:true});
    const readEdges=[
      ['head-range','HEAD','/store/x',{Range:'bytes=1-1'}],
      ['head-not-modified','HEAD','/store/x',{'If-None-Match':'"'+sha('abc')+'"'}],
      ['range first','GET','/store/x',{Range:'bytes=0-0'}],
      ['range last','GET','/store/x',{Range:'bytes=2-2'}],
      ['range reversed','GET','/store/x',{Range:'bytes=2-1'}],
      ['range empty','GET','/store/x',{Range:'bytes='}],
      ['range invalid alphabet','GET','/store/x',{Range:'bytes=abc'}],
      ['range unusual unit','GET','/store/x',{Range:'pixels=1-2'}],
      ['range uppercase','GET','/store/x',{Range:'Bytes=0-1'}],
      ['range overlapping','GET','/store/x',{Range:'bytes=0-1,1-2'}],
      ['if-range weak','GET','/store/x',{Range:'bytes=0-1','If-Range':'W/"'+sha('abc')+'"'}],
      ['conditional weak','GET','/store/x',{'If-None-Match':'W/"'+sha('abc')+'"'}],
      ['conditional wildcard','GET','/store/x',{'If-None-Match':'*'}],
      ['conditional mismatch','GET','/store/x',{'If-None-Match':'"no"'}],
    ];
    for(const [label,method,route,headers] of readEdges) {
      await compare(label,method,route,{headers,compareBody:true});
    }
    await compare('cache','GET','/store/x',{headers:{'If-None-Match':'"'+sha('abc')+'"'},compareBody:true});
    await compare('range','GET','/store/x',{headers:{Range:'bytes=1-2'},compareBody:true});
    await compare('suffix range','GET','/store/x',{headers:{Range:'bytes=-1'},compareBody:true});
    await compare('open range','GET','/store/x',{headers:{Range:'bytes=1-'},compareBody:true});
    await compare('invalid range','GET','/store/x',{headers:{Range:'bytes=100-'},compareBody:true});
    await compare('multi range','GET','/store/x',{headers:{Range:'bytes=0-0,2-2'},compareBody:true});
    await compare('if-range matching','GET','/store/x',{headers:{Range:'bytes=0-0','If-Range':'"'+sha('abc')+'"'},compareBody:true});
    await compare('if-range mismatched','GET','/store/x',{headers:{Range:'bytes=0-0','If-Range':'"wrong"'},compareBody:true});
    await compare('put precondition fail','PUT','/store/x',{
      body:'def',headers:{'X-Skrynia-If-SHA256':sha('wrong')}});
    await compare('put precondition malformed','PUT','/store/x',{
      body:'def',headers:{'X-Skrynia-If-SHA256':'ABC'}});
    await compare('put valid','PUT','/store/x',{
      body:'def',headers:{'X-Skrynia-If-SHA256':sha('abc')}});
    await compare('updated read','GET','/store/x',{compareBody:true});
    await compare('delete precondition fail','DELETE','/store/x',{
      headers:{'X-Skrynia-If-SHA256':sha('abc')}});
    await compare('delete valid','DELETE','/store/x',{
      headers:{'X-Skrynia-If-SHA256':sha('def')}});
    await compare('deleted read','GET','/store/x',{compareBody:true});
    const negativePaths=[
      '/store/.dot','/store/%2Ehidden','/store/abc%2Fdef','/store/a%5Cb',
      '/store/%C3%A9','/store/hello%252Eworld','/store/%00','/store/abc%20def',
      '/store/%2F','/store/a%3Fb','/store/a%23b',
    ];
    for(const route of negativePaths) {
      await compare('invalid-key '+route,'GET',route);
    }
    const writeEdges=[
      ['head missing','HEAD','/store/not_there',{}],
      ['delete missing','DELETE','/store/not_there',{}],
      ['delete malformed precondition','DELETE','/store/not_there',{'X-Skrynia-If-SHA256':'BAD'}],
      ['put missing','PUT','/store/not_there',{}],
      ['get unsupported','PATCH','/store/x',{}],
      ['unknown endpoint','GET','/not_an_endpoint',{}],
      ['push unsupported GET','GET','/push/subscriptions',{}],
    ];
    for(const [label,method,route,headers] of writeEdges) {
      await compare(label,method,route,{headers,body:method==='PUT'?'data':undefined});
    }
    await compare('anonymous list','GET','/store?locations=*',{token:'none'});
    await compare('bad-token list','GET','/store?locations=*',{token:'bad'});
    await compare('anonymous grant inspect','GET',state=>'/grants/'+state.root.id,{token:'none'});
    await compare('invalid issuer grant inspect','GET',state=>'/grants/'+state.root.id,{token:'bad'});
    await compare('bad issuer user issuance','POST',state=>'/grants/'+state.root.id+'/users',{token:'bad'});
    await compare('anonymous user issuance','POST',state=>'/grants/'+state.root.id+'/users',{token:'none'});
    await compare('list all','GET','/store?locations=*');
    await compare('list missing matcher','GET','/store');
    await compare('list duplicate matcher','GET','/store?locations=*&locations=*');
    await compare('list invalid limit','GET','/store?locations=*&limit=0');
    await compare('list invalid cursor','GET','/store?locations=*&after=.hidden');
    await compare('grant missing limits','POST','/grants',{body:JSON.stringify({locations:'team.*'})});
    await compare('grant invalid scope','POST','/grants',{body:JSON.stringify({locations:'../bad',maxBytes:5000,maxObjects:2})});
    await compare('grant outside operations','POST','/grants',{body:JSON.stringify({locations:'*',maxBytes:5000,maxObjects:2,operations:['unknown']})});
    const children=await compare('grant create valid','POST','/grants',{
      body:JSON.stringify({locations:'team.*',maxBytes:20000,maxObjects:5,
        operations:['create','list']})});
    for(let i=0;i<states.length;i++) {
      states[i].childGrant=children[i].json;
      const result=await request(states[i].port,'POST',`/grants/${children[i].json.id}/users`,
        children[i].json.issuerToken);
      assert.equal(result.status,200);
      states[i].childUser=result.json.userToken;
    }
    for(const [name,route] of [['child create','/store/team.first'],['child out of scope','/store/elsewhere']]) {
      const status=await Promise.all(states.map(state=>request(state.port,'POST',route,state.childUser,'one')));
      try {assert.equal(status[1].status,status[0].status);}catch {failures.push({label:name,node:status[0].status,rust:status[1].status});}
      checked++;
    }
    const second=await Promise.all(states.map(state=>request(state.port,'POST',
      '/store/team.second',state.childUser,'second')));
    assert.deepEqual(second.map(x=>x.status),[201,201]);
    const paged=await Promise.all(states.map(state=>request(state.port,'GET',
      '/store?locations=team.*&limit=1',state.user)));
    assert.deepEqual(paged.map(x=>x.status),[200,200]);
    assert.deepEqual(paged[1].json,paged[0].json);
    assert.deepEqual(paged[0].json.keys,['team.first']);
    assert.equal(paged[0].json.next,'team.first');
    checked++;
    const remaining=await Promise.all(states.map(state=>request(state.port,'GET',
      '/store?locations=team.*&limit=1&after=team.first',state.user)));
    assert.deepEqual(remaining[1].json,remaining[0].json);
    assert.deepEqual(remaining[0].json.keys,['team.second']);
    checked++;
    await compare('list after','GET','/store?locations=*&after=team.first');
    console.log(`Verified ${checked} paired Node/Rust request outcomes`);
    if(failures.length) {
      for(const failure of failures)console.error('PROTOCOL MISMATCH',JSON.stringify(failure));
      throw Error(`${failures.length} protocol mismatches`);
    }
  } finally {await Promise.all(states.map(stop));}
}
main().catch(error=>{console.error(error);process.exitCode=1;});
