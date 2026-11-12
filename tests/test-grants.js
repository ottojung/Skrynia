#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const {sha256Bytes:sha256}=require('../src/content-hash.js');
const fs=require('node:fs');
const os=require('node:os');
const path=require('node:path');
const http=require('node:http');
const {execFileSync}=require('node:child_process');
const {createServer}=require('../src/server.js');
const {validKey,validLocations,matches,subset}=require('../src/grants.js');

const tmp=()=>fs.mkdtempSync(path.join(os.tmpdir(),'skrynia-global-test-'));
const auth=t=>({Authorization:'Bearer '+t});
const grant=t=>auth(t);
async function open(dataDir,options={}) {
  const server=createServer({dataDir,skryniaUrl:'http://127.0.0.1:17380/platform',
    pushManual:true,...options});
  await new Promise(ok=>server.listen(0,'127.0.0.1',ok));
  return {server,port:server.address().port};
}
async function close(server) {
  await new Promise(ok=>server.close(ok));
}
function request(port,method,url,body=null,headers={}) {
  const bytes=body===null?null:Buffer.isBuffer(body)?body:
    typeof body==='object'?Buffer.from(JSON.stringify(body)):Buffer.from(body);
  return new Promise((resolve,reject)=>{
    const req=http.request({port,host:'127.0.0.1',
      path:'/platform'+url,method,headers:{...headers,...(bytes?{'Content-Length':bytes.length}:{})}},res=>{
      const chunks=[];
      res.on('data',b=>chunks.push(b));
      res.on('end',()=>{
        const raw=Buffer.concat(chunks);
        let json=null;try{json=JSON.parse(raw.toString());}catch{}
        resolve({status:res.statusCode,json,bytes:raw,etag:res.headers.etag,
          type:res.headers['content-type'],headers:res.headers});
      });
    });
    req.on('error',reject);
    if(bytes)req.write(bytes);
    req.end();
  });
}
async function expect(port,status,method,url,body,headers) {
  const r=await request(port,method,url,body,headers);
  assert.equal(r.status,status,method+' '+url+' got '+r.status+': '+JSON.stringify(r.json));
  return r;
}
function grammar() {
  for(const key of ['index.html','app.js','alice.settings.json','some..name','name...','name.','A_0']) {
    assert(validKey(key),key);
  }
  for(const key of ['','.','..','...','.hidden','..hidden','a/b','a-b','é','a*']) assert(!validKey(key),key);
  for(const scope of ['','*','app.js','alice.*','alice..*','alice...*','name.'])
    assert(validLocations(scope));
  for(const scope of ['.hidden','.hidden*','..*','.','..','...','.*'])
    assert(!validLocations(scope),scope);
  assert(validLocations('a'.repeat(256)+'*'));
  assert(!validLocations('a'.repeat(258)));
  assert(!validLocations('a'.repeat(257)+'*'));
  assert(matches('alice.*','alice.index.html'));
  assert(matches('alice...*','alice...script.js'));
  assert(!matches('alice.*','.hidden'));
  assert(!matches('alice.*','bob.index.html'));
  assert(subset('alice.config.json','alice.*'));
  assert(!subset('alice.*','alice.config.json'));
  assert(subset('','alice.*'));
}
async function run() {
  grammar();
  const dir=tmp();
  let first,second,port,oldRoot;
  try {
    ({server:first,port}=await open(dir));
    oldRoot=first.skrynia.grants.root();
    assert.equal(oldRoot.locations,'*');
    assert.equal(oldRoot.maxBytes,null);
    assert.equal(oldRoot.maxObjects,null);
    assert.deepEqual(oldRoot.operations,['create','modify','delete','list','delegate','subscribe']);
    assert.match(oldRoot.rootIssuerToken,/^[a-f0-9]{64}$/);
    const recordPath=path.join(dir,'state','grants',oldRoot.id+'.json');
    assert.equal(fs.statSync(recordPath).mode & 0o777,0o600);
    assert.equal(JSON.parse(fs.readFileSync(recordPath)).rootIssuerToken,
      oldRoot.rootIssuerToken);
    assert.equal(fs.statSync(path.join(dir,'state')).mode & 0o777,0o700);
    const credentials=JSON.parse(execFileSync(process.execPath,
      [path.join(__dirname,'../scripts/root-credentials.js')],
      {env:{...process.env,SKRYNIA_DATA_DIR:dir}}).toString());
    assert.deepEqual(credentials,{id:oldRoot.id,issuerToken:oldRoot.rootIssuerToken});

    const a=await expect(port,200,'POST','/grants/'+oldRoot.id+'/users',null,
      auth(oldRoot.rootIssuerToken));
    const rootToken=a.json.userToken;
    const alice=await expect(port,200,'POST','/grants',{
      locations:'alice.*',operations:['create','modify','delete','delegate','subscribe'],
      maxBytes:20000,maxObjects:5},auth(rootToken));
    const aliceId=alice.json.id;
    const aliceUser=(await expect(port,200,'POST','/grants/'+aliceId+'/users',
      null,auth(alice.json.issuerToken))).json.userToken;
    const bob=await expect(port,200,'POST','/grants',{
      locations:'bob.*',operations:['create','modify','delete'],maxBytes:6000,maxObjects:2
    },auth(rootToken));
    const bobToken=(await expect(port,200,'POST','/grants/'+bob.json.id+'/users',
      null,auth(bob.json.issuerToken))).json.userToken;
    await expect(port,403,'POST','/store/bob.secret','forbidden',grant(aliceUser));
    await expect(port,400,'POST','/store/.hidden','not allowed',grant(rootToken));
    await expect(port,400,'POST','/grants',{
      locations:'.hidden*',operations:['create','modify','delete'],maxBytes:4096,maxObjects:2
    },auth(rootToken));
    await expect(port,201,'POST','/store/alice.index.html','<h1>Hello</h1>',grant(aliceUser));
    const html=await expect(port,200,'GET','/store/alice.index.html');
    assert.match(html.type,/^text\/html/);
    assert.equal(html.bytes.toString(),'<h1>Hello</h1>');
    assert(html.headers['access-control-allow-origin']==='*');
    await expect(port,412,'PUT','/store/alice.index.html','wrong',
      {...grant(aliceUser),'X-Skrynia-If-SHA256':'0'.repeat(64)});
    await expect(port,200,'PUT','/store/alice.index.html','<h1>Updated</h1>',
      {...grant(aliceUser),'X-Skrynia-If-SHA256':sha256(html.bytes)});
    await expect(port,201,'POST','/store/bob.file.json','{}',grant(bobToken));
    assert.equal((await expect(port,200,'GET','/store/bob.file.json')).type,'application/json');
    await expect(port,403,'POST','/grants',{
      locations:'bob.*',operations:['create','modify','delete'],maxBytes:4096,maxObjects:2
    },auth(aliceUser));
    await expect(port,403,'POST','/grants',{
      locations:'alice.*',operations:['create','modify','delete'],maxBytes:4096,maxObjects:2
    },auth(bobToken));
    const subgrant=await expect(port,200,'POST','/grants',{
      locations:'alice.child.*',operations:['create','modify','delete'],
      maxBytes:4096,maxObjects:1
    },auth(aliceUser));
    assert.equal(subgrant.json.locations,'alice.child.*');
    const subToken=(await expect(port,200,'POST','/grants/'+subgrant.json.id+'/users',
      null,auth(subgrant.json.issuerToken))).json.userToken;
    await expect(port,201,'POST','/store/alice.child.a','x',grant(subToken));
    await expect(port,507,'POST','/store/alice.child.b','x',grant(subToken));
    const usage=await expect(port,200,'GET','/grants/'+subgrant.json.id,
      null,auth(subToken));
    assert.equal(usage.json.usage.objects,1);
    assert.equal(usage.json.maxObjects,1);
    assert.equal(usage.json.rootIssuerToken,undefined);
    const tight=await expect(port,200,'POST','/grants',{
      locations:'shared.*',operations:['create','modify','delete'],maxBytes:2500,maxObjects:1
    },auth(rootToken));
    const wide=await expect(port,200,'POST','/grants',{
      locations:'shared.*',operations:['create','modify','delete'],maxBytes:10000,maxObjects:3
    },auth(rootToken));
    const tightToken=(await expect(port,200,'POST','/grants/'+tight.json.id+
      '/users',null,auth(tight.json.issuerToken))).json.userToken;
    const wideToken=(await expect(port,200,'POST','/grants/'+wide.json.id+
      '/users',null,auth(wide.json.issuerToken))).json.userToken;
    await expect(port,201,'POST','/store/shared.item','overlapping',grant(wideToken));
    const tightUsage=await expect(port,200,'GET','/grants/'+tight.json.id,
      null,auth(tightToken));
    assert(tightUsage.json.usage.bytes>2500,'independent writes can exceed a sibling budget');
    await expect(port,200,'DELETE','/store/shared.item',null,grant(tightToken));
    await expect(port,200,'DELETE','/grants/'+aliceId,null,auth(alice.json.issuerToken));
    await expect(port,403,'PUT','/store/alice.index.html','disallowed',grant(aliceUser));
    await expect(port,200,'GET','/store/alice.index.html');
    await expect(port,200,'GET','/store/alice.child.a');
    await expect(port,200,'DELETE','/grants/'+oldRoot.id,null,auth(oldRoot.rootIssuerToken));
    await expect(port,403,'PUT','/store/bob.file.json','no',grant(bobToken));
    await expect(port,200,'GET','/store/bob.file.json');
    assert.equal(first.skrynia.grants.root(),null,'no root until restart');
    await close(first);first=null;

    ({server:second,port}=await open(dir));
    const newRoot=second.skrynia.grants.root();
    assert.notEqual(newRoot.id,oldRoot.id);
    assert.notEqual(newRoot.rootIssuerToken,oldRoot.rootIssuerToken);
    assert.equal(newRoot.maxBytes,null);
    await expect(port,200,'GET','/store/alice.index.html');
    await expect(port,200,'GET','/store/bob.file.json');
    const newUser=(await expect(port,200,'POST','/grants/'+newRoot.id+'/users',
      null,auth(newRoot.rootIssuerToken))).json.userToken;
    await expect(port,200,'DELETE','/store/bob.file.json',null,grant(newUser));
    await expect(port,404,'GET','/store/bob.file.json');
    for(const uri of ['/not-an-endpoint','/unknown/anything']) {
      const r=await request(port,'GET',uri);
      assert.equal(r.status,404,uri);
    }
    const cors=await expect(port,204,'OPTIONS','/store/alice.index.html',null,{
      Origin:'https://example.org',
      'Access-Control-Request-Method':'PUT',
      'Access-Control-Request-Headers':'authorization,if-match'});
    assert.match(cors.headers['access-control-allow-headers'],/X-Skrynia-If-SHA256/);
    console.log('global store, dot keys, delegation, quotas, root regeneration: OK');
  }finally{
    if(first)await close(first);
    if(second)await close(second);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
if(require.main===module)run().catch(e=>{console.error(e);process.exitCode=1;});
module.exports={open,close,request,expect,auth,grant,tmp};
