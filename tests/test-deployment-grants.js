#!/usr/bin/env node
'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const http = require('node:http');
const {execFileSync} = require('node:child_process');
const {createServer} = require('../src/server.js');

const temp=()=>fs.mkdtempSync(path.join(os.tmpdir(),'skrynia-deploy-grants-'));
async function start(dir) {
  const server=createServer({dataDir:dir,skryniaUrl:'http://127.0.0.1:17880/platform',
    token:'management-secret',pushManual:true});
  await new Promise(ok=>server.listen(0,'127.0.0.1',ok));
  return {server,port:server.address().port};
}
function get(port,route,method='GET',body=null,headers={}) {
  return new Promise((resolve,reject)=>{
    const req=http.request({hostname:'127.0.0.1',port,method,path:'/platform'+route,
      headers:{...headers,...(body?{'Content-Length':Buffer.byteLength(body)}:{})}},res=>{
      const chunks=[];res.on('data',c=>chunks.push(c));res.on('end',()=>{
        const raw=Buffer.concat(chunks).toString('utf8');
        let json;try{json=JSON.parse(raw);}catch{}
        resolve({status:res.statusCode,json,raw});
      });
    });
    req.on('error',reject);if(body)req.write(body);req.end();
  });
}
async function verify() {
  const dir=temp(), bin=path.join(dir,'bin'), source=path.join(dir,'source');
  fs.mkdirSync(bin);fs.mkdirSync(source);
  fs.writeFileSync(path.join(source,'index.html'),'<h1>Hello grants</h1>');
  execFileSync('git',['init','-q',source]);
  execFileSync('git',['-C',source,'config','user.name','Test']);
  execFileSync('git',['-C',source,'config','user.email','test@example.test']);
  execFileSync('git',['-C',source,'add','.']);
  execFileSync('git',['-C',source,'commit','-qm','initial']);
  const sha=execFileSync('git',['-C',source,'rev-parse','HEAD'],{encoding:'utf8'}).trim();
  const realGit=execFileSync('which',['git'],{encoding:'utf8'}).trim();
  fs.writeFileSync(path.join(bin,'git'),[
    '#!/bin/sh','if [ "$1" = "clone" ]; then',
    '  exec "'+realGit+'" clone --quiet "$SKRYNIA_TEST_SOURCE_REPO" "$4"',
    'fi','exec "'+realGit+'" "$@"',''
  ].join('\n'),{mode:0o755});
  fs.writeFileSync(path.join(bin,'docker'),[
    '#!/bin/sh','set -eu','repo=""',
    'while [ "$#" -gt 0 ]; do',
    '  case "$1" in',
    '    -v) repo="$(printf "%s" "$2" | cut -d: -f1)"; shift 2;;',
    '    *) shift;;',
    '  esac',
    'done',
    '[ -n "$repo" ]','mkdir -p "$repo/build"',
    'cp "$repo/index.html" "$repo/build/index.html"',''
  ].join('\n'),{mode:0o755});
  const originalPath=process.env.PATH, originalRepo=process.env.SKRYNIA_TEST_SOURCE_REPO;
  process.env.PATH=bin+':'+originalPath;
  process.env.SKRYNIA_TEST_SOURCE_REPO=source;
  let server;
  try {
    const opened=await start(path.join(dir,'data'));
    server=opened.server;const port=opened.port;
    const route='/deploy';
    const payload=JSON.stringify({
      repo:'git@github.com:example/deploy-grants.git',
      commit:sha,subdir:'.',namespace:'demo',
      quota:16000,maxObjects:5,locations:'demo*',operations:'write,delegate',
    });
    const admin={'Authorization':'Bearer management-secret',
      'Content-Type':'application/json'};
    let r=await get(port,route);
    assert.equal(r.status,405,'deployment must not mutate via GET');
    r=await get(port,route,'POST',payload);
    assert.equal(r.status,401,'deployment requires management bearer header');
    r=await get(port,route,'POST',payload,admin);
    assert.equal(r.status,200,'deploy: '+r.raw);
    const initial=r.json.rootGrant;
    assert(initial && initial.created);
    assert.equal(initial.locations,'demo*');
    assert.deepEqual(initial.operations,['write','delegate']);
    assert.equal(initial.maxBytes,16000);
    assert.equal(initial.userToken,undefined);
    assert.match(initial.issuerToken,/^[a-f0-9]{64}$/);
    r=await get(port,'/grants/'+initial.id+'/users?namespace=demo','POST',null,
      {Authorization:'Bearer '+initial.issuerToken});
    assert.equal(r.status,200,'issuer can mint first user token: '+r.raw);
    const rootUserToken=r.json.userToken;
    assert.match(rootUserToken,/^[a-f0-9]{64}$/);
    r=await get(port,'/store/demo/demo1','POST','first',
      {'X-Skrynia-Grant':rootUserToken});
    assert.equal(r.status,201);
    r=await get(port,'/store/demo/outside1','POST','blocked',
      {'X-Skrynia-Grant':rootUserToken});
    assert.equal(r.status,403);
    r=await get(port,'/store/demo/demo1');
    assert.equal(r.status,200);
    assert.equal(r.raw,'first');
    r=await get(port,route,'POST',payload,admin);
    assert.equal(r.status,200,'repeat deployment: '+r.raw);
    assert.equal(r.json.rootGrant.created,false);
    assert.equal(r.json.rootGrant.id,initial.id);
    assert.equal(r.json.rootGrant.userToken,undefined);
    assert.equal(r.json.rootGrant.issuerToken,undefined);
    assert.equal((await get(port,'/store/demo/demo1')).raw,'first');
    r=await get(port,'/ns/inspect?namespace=demo', 'GET', null, admin);
    assert.equal(r.status,200);
    assert.equal(r.json.rootGrant.id,initial.id);
    assert.equal(r.json.rootGrant.usage.objects,1);
    console.log('deployment and root authority: OK');
  } finally {
    if(server)await new Promise(ok=>server.close(ok));
    process.env.PATH=originalPath;
    if(originalRepo===undefined)delete process.env.SKRYNIA_TEST_SOURCE_REPO;
    else process.env.SKRYNIA_TEST_SOURCE_REPO=originalRepo;
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
verify().catch(e=>{console.error(e);process.exitCode=1;});
