#!/usr/bin/env node
'use strict';
// Differential HTTP contract for Web Push subscription/capability lifecycle.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {mkdtempSync,rmSync,readFileSync}=fs;
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const crypto=require('node:crypto');
const {port,request,start,stop,credentials}=require('./crash/fixture');
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
function subscription(endpoint='https://push.example.org/one',events) {
  const ecdh=crypto.createECDH('prime256v1');ecdh.generateKeys();
  return {locations:'notice.*',endpoint,
    ...(events===undefined?{}:{events}),
    keys:{p256dh:ecdh.getPublicKey().toString('base64url'),
      auth:crypto.randomBytes(16).toString('base64url')}};
}
function normalize(reply) {
  const data=JSON.parse(reply.body);
  const result={status:reply.status,...data};
  if(data.id) {assert.match(data.id,/^[a-f0-9]{32}$/);result.id='<opaque-id>';}
  if(data.capability) {
    assert.match(data.capability,/^[a-f0-9]{64}$/);
    result.capability='<opaque-capability>';
  }
  if(data.publicKey) {assert(data.publicKey.length>60);result.publicKey='<P256-public>';}
  return result;
}
async function main() {
  const states=[];
  try {
    for(const [name,command,args] of [['Node',process.execPath,[resolve('src/server.js')]],
      ['Rust',binary,[]]]) {
      const dir=mkdtempSync(join(tmpdir(),'skrynia-push-parity-'+name+'-'));
      const number=await port();
      const child=await start(command,args,dir,number);
      const grant=credentials(dir);
      const issued=await request(number,'POST',`/grants/${grant.id}/users`,grant.issuer);
      assert.equal(issued.status,200);
      states.push({name,child,dir,p:number,user:JSON.parse(issued.body).userToken});
    }
    let checked=0;
    async function compare(label,method,route,data,token='user',cap='none') {
      const responses=await Promise.all(states.map(async state=>{
        const url=typeof route==='function'?route(state):route;
        const tokenValue=token==='user'?state.user:token==='bad'?'f'.repeat(64):undefined;
        const capValue=cap==='current'?state.cap:cap==='prior'?state.oldCap:
          cap==='wrong'?'0'.repeat(64):undefined;
        const headers=capValue?{'X-Skrynia-Capability':capValue}:{};
        return request(state.p,method,url,tokenValue,
          data===undefined?undefined:JSON.stringify(data),headers);
      }));
      const parsed=responses.map(normalize);
      assert.deepEqual(parsed[1],parsed[0],label);
      checked++;
      return responses.map(response=>JSON.parse(response.body));
    }
    const normal=subscription();
    await compare('public VAPID','GET','/push/vapid',undefined,'none');
    await compare('not a JSON object','POST','/push/subscriptions',[]);
    await compare('wrong events','POST','/push/subscriptions',{...normal,events:['create','create']});
    await compare('wrong HTTPS endpoint','POST','/push/subscriptions',{...normal,endpoint:'https://127.0.0.1/private'});
    await compare('missing keys','POST','/push/subscriptions',{...normal,keys:null});
    await compare('unauthorized registration','POST','/push/subscriptions',normal,'none');
    await compare('invalid bearer','POST','/push/subscriptions',normal,'bad');
    const created=await compare('valid registration','POST','/push/subscriptions',normal);
    for(let i=0;i<states.length;i++) {
      states[i].id=created[i].id;
      states[i].cap=created[i].capability;
    }
    await compare('dedup missing capability','POST','/push/subscriptions',normal);
    await compare('dedup wrong capability','POST','/push/subscriptions',normal,'user','wrong');
    const rotated=await compare('dedup rotation','POST','/push/subscriptions',normal,'user','current');
    for(let i=0;i<states.length;i++) {
      states[i].oldCap=states[i].cap;
      states[i].cap=rotated[i].capability;
    }
    const current=state=>`/push/subscriptions/${state.id}`;
    await compare('empty patch','PUT',current,{},'none','current');
    await compare('unknown patch fields','PUT',current,{unknown:1},'none','current');
    await compare('unknown patch no cap','PUT',current,{unknown:1},'none');
    await compare('empty patch no cap','PUT',current,{},'none');
    await compare('missing capability patch','PUT',current,{endpoint:'https://push.example.org/changed'},'none');
    await compare('old capability patch','PUT',current,{endpoint:'https://push.example.org/changed'},'none','prior');
    await compare('invalid keys patch','PUT',current,{keys:{auth:'',p256dh:'bad'}},'none','current');
    await compare('valid endpoint patch','PUT',current,{endpoint:'https://push.example.org/changed'},'none','current');
    await compare('old capability delete','DELETE',current,undefined,'none','prior');
    const secondInput=subscription('https://push.example.org/other');
    const second=await compare('second registration','POST','/push/subscriptions',secondInput);
    for(let i=0;i<states.length;i++) {
      states[i].otherId=second[i].id;
      states[i].otherCap=second[i].capability;
    }
    const competing=state=>`/push/subscriptions/${state.otherId}`;
    const clash=await Promise.all(states.map(state=>request(state.p,'PUT',
      competing(state),undefined,JSON.stringify({endpoint:'https://push.example.org/changed'}),
      {'X-Skrynia-Capability':state.otherCap})));
    assert.deepEqual(clash.map(normalize),[{status:409,error:'endpoint_in_use'},
      {status:409,error:'endpoint_in_use'}]);
    checked++;
    await compare('delete main subscription','DELETE',current,undefined,'none','current');
    await compare('delete already gone','DELETE',current,undefined,'none','current');
    for(const state of states) {
      const vapid=JSON.parse(readFileSync(join(state.dir,'state/push/vapid.json')));
      assert(vapid.publicKey.length>60);
      assert(!fs.existsSync(join(state.dir,'state/push-subs',state.id+'.json')));
    }
    console.log(`Verified ${checked} paired Node/Rust Web Push lifecycle requests`);
  } finally {
    for(const state of states) {
      await stop(state.child,'SIGTERM');
      rmSync(state.dir,{recursive:true,force:true});
    }
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
