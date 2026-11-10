#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');

// nginx's conventional static-file ETag: mtime in whole seconds, then size.
function nginxEtag(file) {
  const stat=fs.statSync(file);
  return '"'+Math.floor(stat.mtimeMs/1000).toString(16)+'-'+stat.size.toString(16)+'"';
}

async function run() {
  const dir=tmp();let server;
  try {
    let started=await open(dir);server=started.server;
    let p=started.port;
    const root=server.skrynia.grants.root();
    const token=(await expect(p,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    const key='static.app.js';
    const file=path.join(dir,'store',key);
    const resource='/store/'+key;
    await expect(p,201,'POST',resource,'abcdef',grant(token));
    let response=await expect(p,200,'GET',resource);
    assert.equal(response.etag,nginxEtag(file));
    assert.deepEqual((await expect(p,304,'GET',resource,null,
      {'If-None-Match':nginxEtag(file)})).bytes,Buffer.alloc(0));
    assert.equal((await expect(p,200,'HEAD',resource)).etag,nginxEtag(file));
    assert.equal((await expect(p,206,'GET',resource,null,
      {Range:'bytes=1-3'})).etag,nginxEtag(file));
    let previous=response.etag;
    for(let i=0;i<12;i++) {
      await expect(p,200,'PUT',resource,'abcdef',grant(token));
      response=await expect(p,200,'GET',resource);
      assert.equal(response.etag,nginxEtag(file));
      assert.notEqual(response.etag,previous,
        'every same-sized replacement must advance nginx-compatible ETag');
      previous=response.etag;
    }
    await expect(p,200,'DELETE',resource,null,grant(token));
    await expect(p,201,'POST',resource,'abcdef',grant(token));
    response=await expect(p,200,'GET',resource);
    assert.equal(response.etag,nginxEtag(file));
    assert.notEqual(response.etag,previous);
    previous=response.etag;
    await close(server);server=null;
    started=await open(dir);server=started.server;p=started.port;
    assert.equal((await expect(p,200,'GET',resource)).etag,nginxEtag(file));
    assert.equal((await expect(p,200,'GET',resource)).etag,previous);
    assert.equal(fs.existsSync(path.join(dir,'state','etag-sequence')),true);
    assert.equal(fs.existsSync(path.join(dir,'state','etag-sequence-ms')),false);
    console.log('nginx ETag equality, CAS, ABA prevention and restart: OK');
  }finally{
    if(server) await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
