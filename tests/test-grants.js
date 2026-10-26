#!/usr/bin/env node
'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const http = require('node:http');
const {createServer} = require('../src/server.js');

const mk = () => fs.mkdtempSync(path.join(os.tmpdir(), 'skrynia-grants-'));
async function open(tmp) {
  const server = createServer({dataDir: tmp, token:'admin-token',
    skryniaUrl:'http://127.0.0.1:17880/platform', pushManual:true});
  await new Promise(ok => server.listen(0,'127.0.0.1',ok));
  return {server,port:server.address().port};
}
function close(server) { return new Promise(ok => server.close(ok)); }
function request(port, method, route, body, headers={}) {
  return new Promise((resolve,reject)=>{
    const data = body == null ? null : Buffer.isBuffer(body) ? body :
      typeof body === 'object' ? Buffer.from(JSON.stringify(body)) : Buffer.from(body);
    const req = http.request({hostname:'127.0.0.1',port,method,
      path:'/platform'+route, headers:{...headers,
        ...(data ? {'Content-Length':data.length} : {})}},res=>{
      const chunks=[]; res.on('data',c=>chunks.push(c));
      res.on('end',()=>{
        const raw=Buffer.concat(chunks).toString('utf8');
        let json=null; try { json=JSON.parse(raw); } catch {}
        resolve({status:res.statusCode,body:raw,json,etag:res.headers.etag});
      });
    });
    req.on('error',reject);
    if(data)req.write(data);
    req.end();
  });
}
const user = token=>({Authorization:'Bearer '+token});
const grant = token=>({'X-Skrynia-Grant':token});
const issuer = (ns,id,token,kind)=>({ns,id,token,route:'/grants/'+id+(kind==null?'':'/'+kind)+'?namespace='+ns});
async function test() {
  const dir=mk(); let server,port;
  try {
    ({server,port}=await open(dir));
    const grants=server.skrynia.grants;
    const root=grants.createRoot('forum', 60000,100);
    assert.equal(root.created,true);
    assert.match(root.userToken,/^[a-f0-9]{64}$/);
    assert.notEqual(root.userToken,root.issuerToken);
    assert.equal(grants.createRoot('forum',60000,100).created,false);

    let r=await request(port,'POST','/store/forum/noauth','x');
    assert.equal(r.status,401,'anonymous create forbidden');
    r=await request(port,'POST','/grants',{namespace:'forum',maxBytes:20000,maxObjects:10});
    assert.equal(r.status,401,'anonymous subgrant forbidden');

    r=await request(port,'POST','/grants',{namespace:'forum',maxBytes:12000,maxObjects:5},user(root.issuerToken));
    assert.equal(r.status,403,'issuer is not a user');
    r=await request(port,'POST','/grants',{namespace:'forum',maxBytes:20000,maxObjects:10},user(root.userToken));
    assert.equal(r.status,200);
    const child=r.json;
    r=await request(port,'POST','/grants/'+child.id+'/users?namespace=forum',
      null,user(child.userToken));
    assert.equal(r.status,200,'grant users may mint more user tokens');
    const otherUser=r.json.userToken;
    assert.notEqual(otherUser,child.userToken);
    assert.equal(grants.inspect('forum',child.id,otherUser).userCount,2);
    r=await request(port,'POST','/grants/'+child.id+'/users?namespace=forum',
      null,user(child.issuerToken));
    assert.equal(r.status,403,'issuer cannot mint user tokens');
    r=await request(port,'POST','/grants',{namespace:'forum',maxBytes:6000,maxObjects:5},user(child.userToken));
    assert.equal(r.status,200);
    const grand=r.json;
    r=await request(port,'POST','/grants',{namespace:'forum',maxBytes:6000,maxObjects:5},user(root.userToken));
    const sibling=r.json; assert.equal(r.status,200);

    r=await request(port,'POST','/push/subscriptions?namespace=forum&channel=updates',
      {endpoint:'https://push.example.test/p/1',keys:{p256dh:'sample-key',auth:'auth-key'}});
    assert.equal(r.status,401,'push subscription cannot be created anonymously');
    r=await request(port,'POST','/push/subscriptions?namespace=forum&channel=updates',
      {endpoint:'https://push.example.test/p/1',keys:{p256dh:'sample-key',auth:'auth-key'}},
      grant(child.userToken));
    assert.equal(r.status,201,'push subscription grant sponsorship');

    r=await request(port,'POST','/store/forum/post','hello',grant(child.userToken));
    assert.equal(r.status,201);
    const writeCap=r.json.capability;
    r=await request(port,'GET','/store/forum/post');
    assert.equal(r.body,'hello');
    const etag=r.etag;
    r=await request(port,'PUT','/store/forum/post','world',
      {'X-Skrynia-Capability':writeCap,'If-Match':etag});
    assert.equal(r.status,200);
    r=await request(port,'PUT','/store/forum/post','invalid',
      {'X-Skrynia-Capability':writeCap,'If-Match':etag});
    assert.equal(r.status,412,'CAS semantics retained');

    r=await request(port,'POST','/store/forum/grand','child data',grant(grand.userToken));
    assert.equal(r.status,201);
    r=await request(port,'POST','/store/forum/sibling','sibling data',grant(sibling.userToken));
    assert.equal(r.status,201);

    // A descendant's keep protects its ancestors, not sibling grants.
    r=await request(port,'POST','/grants/'+root.id+'/confirmation?namespace=forum',
      {need:false},user(root.userToken));
    assert.equal(r.status,200);
    r=await request(port,'POST','/grants/'+grand.id+'/confirmation?namespace=forum',
      {need:true},user(grand.userToken));
    assert.equal(r.status,200);
    function due(ns,id) {
      const p=path.join(dir,'state',ns,'grants',id+'.json');
      const g=JSON.parse(fs.readFileSync(p,'utf8'));
      g.windowStart=Date.now()-grants.DAY-1000;
      g.nextCheckAt=Date.now()-1;
      fs.writeFileSync(p,JSON.stringify(g));
    }
    due('forum',root.id);
    grants.sweep(Date.now());
    assert.equal(grants.root('forum').id,root.id);
    assert.equal((await request(port,'GET','/store/forum/grand')).status,200);

    // Issuer cleanup bypasses votes but preserves child authority.
    r=await request(port,'DELETE','/grants/'+child.id+'/space?namespace=forum',null,
      user(child.issuerToken));
    assert.equal(r.status,200);
    assert.equal((await request(port,'GET','/store/forum/post')).status,404);
    assert.equal((await request(port,'GET','/store/forum/grand')).status,404);
    assert.equal((await request(port,'GET','/store/forum/sibling')).status,200);
    assert.equal(grants.inspect('forum',child.id,child.userToken).id,child.id);
    assert.throws(()=>grants.inspect('forum',grand.id,grand.userToken),/grant_not_found/);
    r=await request(port,'POST','/store/forum/new','new',grant(child.userToken));
    assert.equal(r.status,201,'cleaned grant remains usable');

    // Negative vote in a new window collects descendants and data, not root.
    r=await request(port,'POST','/grants/'+root.id+'/confirmation?namespace=forum',
      {need:false},user(root.userToken));
    assert.equal(r.status,200);
    due('forum',root.id);
    grants.sweep(Date.now());
    assert.equal((await request(port,'GET','/store/forum/sibling')).status,404);
    assert.equal(grants.root('forum').id,root.id);
    r=await request(port,'POST','/store/forum/reborn','yes',grant(root.userToken));
    assert.equal(r.status,201);

    // A bad or issuer-only token cannot vote or create storage.
    r=await request(port,'POST','/grants/'+root.id+'/confirmation?namespace=forum',
      {need:true},user(root.issuerToken));
    assert.equal(r.status,403);
    r=await request(port,'DELETE','/grants/'+root.id+'?namespace=forum',null,user(root.issuerToken));
    assert.equal(r.status,200);
    assert.equal(grants.root('forum'),null);
    assert.equal((await request(port,'GET','/store/forum/reborn')).status,404);
    r=await request(port,'POST','/store/forum/forbidden','no',grant(root.userToken));
    assert.equal(r.status,409);

    await close(server);server=null;
    // Grant metadata is charged, even when no objects exist.
    ({server,port}=await open(dir));
    const little=server.skrynia.grants.createRoot('small',4096,100);
    const one=await request(port,'POST','/grants',{namespace:'small',maxBytes:2048,maxObjects:1},user(little.userToken));
    assert.equal(one.status,200);
    const two=await request(port,'POST','/grants',{namespace:'small',maxBytes:2048,maxObjects:1},user(little.userToken));
    assert.equal(two.status,507);

    // Interrupted recursive deletion resumes from its persisted subtree list.
    const smallChild=one.json;
    fs.mkdirSync(path.join(dir,'state','_grant-deletions'),{recursive:true});
    fs.writeFileSync(path.join(dir,'state','_grant-deletions',little.id+'.json'),
      JSON.stringify({ns:'small',id:little.id,action:'clean',ids:[little.id,smallChild.id]}));
    await close(server);server=null;
    ({server,port}=await open(dir));
    assert.equal(server.skrynia.grants.root('small').id,little.id);
    assert.throws(()=>server.skrynia.grants.inspect('small',smallChild.id,smallChild.userToken),/grant_not_found/);
    assert.equal(fs.readdirSync(path.join(dir,'state','_grant-deletions')).length,0);
    console.log('recursive grants: all assertions passed');
  } finally {
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
async function testDeployment() {
  const dir=mk(), bin=path.join(dir,'bin'), source=path.join(dir,'source');
  fs.mkdirSync(bin); fs.mkdirSync(source);
  fs.writeFileSync(path.join(source,'index.html'),'<h1>test</h1>');
  const {execFileSync}=require('node:child_process');
  execFileSync('git',['init',source]);
  execFileSync('git',['-C',source,'config','user.name','Test']);
  execFileSync('git',['-C',source,'config','user.email','test@example.test']);
  execFileSync('git',['-C',source,'add','.']);
  execFileSync('git',['-C',source,'commit','-m','initial']);
  const sha=execFileSync('git',['-C',source,'rev-parse','HEAD'],{encoding:'utf8'}).trim();
  const fakeGit = '#!/bin/sh\\nif [ "$1" = "clone" ]; then exec /usr/bin/git clone --quiet "$SKRYNIA_TEST_SOURCE_REPO" "$4"; fi\\nexec /usr/bin/git "$@"\\n';
  const fakeDocker = '#!/bin/sh\\nset -eu\\nrepo=""\\nwhile [ "$#" -gt 0 ]; do case "$1" in -v) repo="${2%%:*}"; shift 2;; *) shift;; esac; done\\n[ -n "$repo" ]\\nmkdir -p "$repo/build"\\ncp "$repo/index.html" "$repo/build/index.html"\\n';
  fs.writeFileSync(path.join(bin,'git'),fakeGit,{mode:0o755});
  fs.writeFileSync(path.join(bin,'docker'),fakeDocker,{mode:0o755});
  const oldPath=process.env.PATH, oldRepo=process.env.SKRYNIA_TEST_SOURCE_REPO;
  process.env.PATH=bin+':'+oldPath;
  process.env.SKRYNIA_TEST_SOURCE_REPO=source;
  let server;
  try {
    const running=await open(path.join(dir,'data')); server=running.server;
    const route='/deploy?'+new URLSearchParams({repo:'git@github.com:test/mock.git',
      commit:sha,subdir:'.',namespace:'demo',token:'admin-token',
      quota:'12000',maxObjects:'40'}).toString();
    let r=await request(running.port,'GET',route);
    assert.equal(r.status,200,'deployment should bootstrap root');
    assert.equal(r.json.rootGrant.created,true);
    assert.equal(r.json.rootGrant.maxBytes,12000);
    assert.match(r.json.rootGrant.userToken,/^[a-f0-9]{64}$/);
    const initial=r.json.rootGrant;
    r=await request(running.port,'POST','/store/demo/first','ok',grant(initial.userToken));
    assert.equal(r.status,201,'newly-issued root token must be functional');
    r=await request(running.port,'GET',route);
    assert.equal(r.status,200);
    assert.equal(r.json.rootGrant.created,false);
    assert.equal(r.json.rootGrant.id,initial.id);
    assert.equal(r.json.rootGrant.userToken,undefined,'deployment must not re-expose token');
    assert.equal((await request(running.port,'GET','/store/demo/first')).status,200);
    console.log('deployment bootstrapping: all assertions passed');
  } finally {
    if(server) await close(server);
    process.env.PATH=oldPath;
    if(oldRepo===undefined)delete process.env.SKRYNIA_TEST_SOURCE_REPO;
    else process.env.SKRYNIA_TEST_SOURCE_REPO=oldRepo;
    fs.rmSync(dir,{recursive:true,force:true});
  }
}

test().then(testDeployment).catch(e=>{console.error(e);process.exitCode=1;});
