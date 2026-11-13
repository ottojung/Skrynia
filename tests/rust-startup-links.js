#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {join,dirname,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {spawnSync}=require('node:child_process');
const {port,request,start,stop}=require('./crash/fixture');
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
function mustRefuse(data,pattern) {
  const run=spawnSync(binary,[],{encoding:'utf8',timeout:5000,env:{
    ...process.env,SKRYNIA_DATA_DIR:data,
    SKRYNIA_URL:'http://127.0.0.1:17380',SKRYNIA_PORT:'17380',
    SKRYNIA_PUSH_MANUAL:'1'}});
  assert(!run.error,run.error?.message);
  assert.notEqual(run.status,0,'untrusted filesystem entry allowed server startup');
  assert.match(run.stderr,pattern);
}
async function main() {
  const root=fs.mkdtempSync(join(tmpdir(),'skrynia-startup-links-'));
  try {
    for(const part of ['store','store-tmp','state','state/grants',
      'state/store-transactions','state/grant-revocations','state/push',
      'state/push/outbox','state/push-subs']) {
      const data=join(root,'data');fs.mkdirSync(data,{recursive:true});
      const external=join(root,'outside');fs.mkdirSync(external,{recursive:true,mode:0o700});
      const sentinel=join(external,'sentinel');
      fs.writeFileSync(sentinel,'unaltered');
      const link=join(data,part);
      fs.mkdirSync(dirname(link),{recursive:true});
      fs.symlinkSync(external,link,'dir');
      mustRefuse(data,/storage directory must be a real directory/);
      assert.equal(fs.readFileSync(sentinel,'utf8'),'unaltered');
      assert.equal(fs.statSync(external).mode&0o777,0o700);
      fs.rmSync(data,{recursive:true,force:true});
      fs.rmSync(external,{recursive:true,force:true});
    }
    const data=join(root,'lock-data');
    fs.mkdirSync(join(data,'state'),{recursive:true});
    const outside=join(root,'external-lock');fs.writeFileSync(outside,'private');
    fs.symlinkSync(outside,join(data,'state/writer.lock'));
    mustRefuse(data,/Too many levels of symbolic links|symlink|Symbolic link/);
    assert.equal(fs.readFileSync(outside,'utf8'),'private');
    const clean=join(root,'clean');const p=await port();
    const child=await start(binary,[],clean,p);
    try {assert.equal((await request(p,'GET','/health')).status,200);}
    finally {await stop(child,'SIGTERM');}
    console.log('Storage directories and writer lock reject symlink injection: OK');
  } finally {fs.rmSync(root,{recursive:true,force:true});}
}
main().catch(error=>{console.error(error);process.exitCode=1;});
