#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {spawn}=require('node:child_process');
const {once}=require('node:events');
const {createServer}=require('../src/server.js');
const {open,close,expect,auth,tmp}=require('./test-grants.js');

const options=dir=>({dataDir:dir,pushManual:true,
  skryniaUrl:'http://127.0.0.1:17380/platform'});

async function run() {
  const dir=tmp();
  let child,server;
  try {
    // A failed startup cannot retain ownership.
    assert.throws(()=>createServer({...options(dir),skryniaUrl:'bad-url'}),
      /SKRYNIA_URL/);
    const script='const {createServer}=require('+JSON.stringify(
      path.join(__dirname,'../src/server.js'))+');'+
      'const server=createServer({dataDir:process.argv[1],pushManual:true,'+
      'skryniaUrl:"http://127.0.0.1:17380/platform"});'+
      'server.listen(0,"127.0.0.1",()=>console.log("READY"));';
    child=spawn(process.execPath,['-e',script,dir],{
      stdio:['ignore','pipe','pipe']});
    const ready=await Promise.race([
      once(child.stdout,'data').then(([bytes])=>bytes.toString()),
      once(child,'exit').then(([status])=>{
        throw new Error('writer exited early: '+status);
      })
    ]);
    assert.match(ready,/READY/);
    const rootDir=path.join(dir,'state','grants');
    const before=fs.readdirSync(rootDir);
    assert.equal(before.length,1);
    assert.throws(()=>createServer(options(dir)),/another writer/);
    assert.deepEqual(fs.readdirSync(rootDir),before);
    assert.equal(fs.existsSync(path.join(dir,'state','writer.lock')),true);

    // The OS releases the lock automatically on an unclean process exit.
    const exited=once(child,'exit');
    child.kill('SIGKILL');
    await exited;
    child=null;
    let opened=await open(dir);
    server=opened.server;
    assert.deepEqual(fs.readdirSync(rootDir),before);
    assert.throws(()=>createServer(options(dir)),/another writer/);

    // Ongoing push sends can still change disk state after HTTP closes.
    // Keep the writer lock until the send finishes.
    const root=server.skrynia.grants.root();
    const token=(await expect(opened.port,200,'POST',
      '/grants/'+root.id+'/users',null,auth(root.rootIssuerToken))).json.userToken;
    await close(server);server=null;

    let release,sendStarted;
    const pending=new Promise(resolve=>{release=resolve;});
    const started=new Promise(resolve=>{sendStarted=resolve;});
    opened=await open(dir,{pushTransport:async()=>{sendStarted();await pending;}});
    server=opened.server;
    server.skrynia.push.createSub('*',['create'],
      'https://push.example.org/u',{p256dh:'key',auth:'secret'},null,token);
    server.skrynia.push.enqueue('object.first','create');
    const pumping=server.skrynia.push.pumpOnce();
    await started;
    await close(server);server=null;
    assert.throws(()=>createServer(options(dir)),/another writer/);
    release();
    await pumping;
    const restarted=await open(dir);
    server=restarted.server;
    await expect(restarted.port,200,'GET','/health');
    console.log('exclusive writer ownership, crash release and push drain: OK');
  } finally {
    if(child) {
      const exit=once(child,'exit');
      child.kill('SIGKILL');
      await exit;
    }
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
