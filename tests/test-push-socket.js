#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const https=require('node:https');
const crypto=require('node:crypto');
const {execFileSync}=require('node:child_process');
const webpush=require('web-push');
const {safeAgent}=require('../src/push-endpoint.js');
const {createPush}=require('../src/push.js');
const {tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();
  let server,push;
  try {
    const key=path.join(dir,'key.pem'),cert=path.join(dir,'cert.pem');
    execFileSync('openssl',['req','-x509','-nodes','-newkey','rsa:2048',
      '-keyout',key,'-out',cert,'-days','1',
      '-subj','/CN=push.example.org'],{stdio:'ignore'});
    let received=0;
    server=https.createServer({
      key:fs.readFileSync(key),cert:fs.readFileSync(cert)
    },(req,res)=>{
      assert.equal(req.method,'POST');
      assert.match(req.headers.authorization||'',/^(vapid|WebPush) /i);
      const chunks=[];
      req.on('data',part=>chunks.push(part));
      req.on('end',()=>{
        received++;
        assert(Buffer.concat(chunks).length>0);
        res.writeHead(201);res.end();
      });
    });
    await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
    const lookup=(hostname,options,callback)=>{
      assert.equal(hostname,'push.example.org');
      assert.equal(options.all,true);
      callback(null,[{address:'127.0.0.1',family:4}]);
    };
    const agent=safeAgent(lookup,()=>true);
    // This explicit, local-only test bypasses the public-address check
    // to exercise the exact TLS agent used in production.
    agent.options.rejectUnauthorized=false;
    const ecdh=crypto.createECDH('prime256v1');
    ecdh.generateKeys();
    let usedProductionAgent=false;
    // Exercise createPush -> web-push -> real HTTPS socket, substituting a
    // permissive test-only loopback agent after inspecting the production one.
    const pushLib={
      generateVAPIDKeys:()=>webpush.generateVAPIDKeys(),
      sendNotification(subscription,payload,options) {
        assert(options.agent instanceof https.Agent);
        usedProductionAgent=true;
        return webpush.sendNotification(subscription,payload,
          {...options,agent});
      }
    };
    push=createPush({dataDir:dir,pushManual:true,
      pushAllowedHosts:'push.example.org',webPushLib:pushLib,
      pushSubject:'mailto:ops@example.org',
      onSubscriptionCreate:()=> 'a'.repeat(32)});
    push.start();
    push.createSub('*',['create'],
      'https://push.example.org:'+server.address().port+'/endpoint',{
        p256dh:ecdh.getPublicKey().toString('base64url'),
        auth:crypto.randomBytes(16).toString('base64url')
      },null,'grant');
    push.enqueue('events.hello','create');
    assert.deepEqual(await push.pumpOnce(),{delivered:1});
    assert(usedProductionAgent);
    assert.equal(received,1);

    // The default production agent must still reject the same loopback
    // destination when DNS resolves to it.
    const blocked=safeAgent(lookup);
    assert.equal(await new Promise(resolve=>
      blocked.options.lookup('push.example.org',{all:true},e=>resolve(e.code))),
      'EHOSTUNREACH');
    console.log('real Web Push HTTPS socket, Node lookup(all), private DNS blocking: OK');
  } finally {
    if(push)push.close();
    if(server)await new Promise(resolve=>server.close(resolve));
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
