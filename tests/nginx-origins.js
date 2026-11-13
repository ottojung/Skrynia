#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const http=require('node:http');
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {spawn,spawnSync}=require('node:child_process');
const {setTimeout:pause}=require('node:timers/promises');
const {port}=require('./crash/fixture');
const nginx=process.env.SKRYNIA_NGINX||'nginx';
async function get(p,host,path) {
  return new Promise((done,fail)=>{
    const req=http.get({host:'127.0.0.1',port:p,path,
      headers:{Host:host,Connection:'close'}},res=>{
      const chunks=[];res.on('data',b=>chunks.push(b));
      res.on('error',fail);res.on('end',()=>done({
        code:res.statusCode,body:Buffer.concat(chunks).toString()}));
    });
    req.on('error',fail);req.setTimeout(3000,()=>req.destroy(Error('nginx timeout')));
  });
}
async function ready(p,child) {
  for(let i=0;i<60;i++) {
    if(child.exitCode!=null)throw Error('nginx exited early');
    try {
      if((await get(p,'alice.apps.example.org','/')).code===200)return;
    } catch {}
    await pause(50);
  }
  throw Error('nginx did not become ready');
}
async function main() {
  const root=fs.mkdtempSync(join(tmpdir(),'skrynia-nginx-origins-'));
  let child;
  try {
    const store=join(root,'store');fs.mkdirSync(store);
    const state=join(root,'state');fs.mkdirSync(state);
    const privateFile=join(state,'private.json');
    fs.writeFileSync(privateFile,'SECRET-MUST-NOT-BE-SERVED',{mode:0o600});
    for(const name of ['alice','bob']) {
      fs.writeFileSync(join(store,name+'.index.html'),name+' home');
      fs.writeFileSync(join(store,name+'.app.js'),name+' script');
    }
    fs.symlinkSync(privateFile,join(store,'alice.leak'));
    fs.symlinkSync(join(store,'bob.app.js'),join(store,'alice.cross'));
    const p=await port();
    const source=fs.readFileSync(resolve('examples/nginx-origins.conf'),'utf8');
    const vhosts=source.replaceAll('listen 443 ssl;',`listen 127.0.0.1:${p};`)
      .replaceAll('root /var/lib/skrynia/store;',`root ${store};`)
      .replace(/^\s*ssl_certificate(?:_key)?\s+[^;]+;/gm,'');
    assert.equal((vhosts.match(/disable_symlinks on;/g)||[]).length,2);
    const config=join(root,'nginx.conf');
    fs.writeFileSync(config,`worker_processes 1;
pid ${root}/nginx.pid;
error_log ${root}/nginx.log warn;
events { worker_connections 128; }
http { access_log off; ${vhosts} }
`);
    const args=['-p',root+'/','-c',config];
    const syntax=spawnSync(nginx,['-t',...args],{encoding:'utf8',timeout:10000});
    assert.equal(syntax.status,0,syntax.error?.message||syntax.stderr);
    child=spawn(nginx,[...args,'-g','daemon off;'],{stdio:'ignore'});
    await ready(p,child);
    for(const name of ['alice','bob']) {
      const host=name+'.apps.example.org';
      assert.deepEqual(await get(p,host,'/'),{code:200,body:name+' home'});
      assert.deepEqual(await get(p,host,'/app.js'),{code:200,body:name+' script'});
      assert.equal((await get(p,host,'/'+(name==='alice'?'bob':'alice')+'.app.js')).code,404);
    }
    for(const path of ['/leak','/cross']) {
      const result=await get(p,'alice.apps.example.org',path);
      assert.notEqual(result.code,200,path+' served a symlink');
      assert(!result.body.includes('SECRET-MUST-NOT-BE-SERVED'));
      assert(!result.body.includes('bob script'));
    }
    assert.equal((await get(p,'bob.apps.example.org','/leak')).code,404);
    console.log('Real nginx origin isolation and private/sibling symlink denial: OK');
  } finally {
    if(child) {
      child.kill('SIGTERM');
      await Promise.race([
        new Promise(done=>child.once('exit',done)),pause(1200)]);
      if(child.exitCode==null&&child.signalCode==null)child.kill('SIGKILL');
    }
    fs.rmSync(root,{recursive:true,force:true});
  }
}
main().catch(e=>{console.error(e);process.exitCode=1;});
