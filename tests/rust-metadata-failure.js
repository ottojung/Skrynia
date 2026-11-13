'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {createECDH,randomBytes}=require('node:crypto');
const {port,request,start,stop,credentials}=require('./crash/fixture');
const binary=resolve('target/debug/skrynia-server');
async function main() {
  if(process.getuid?.()===0)return;
  const root=fs.mkdtempSync(join(tmpdir(),'skrynia-metadata-test-'));
  let child;
  try {
    const p=await port();child=await start(binary,[],root,p);
    const grant=credentials(root);
    const issued=await request(p,'POST',`/grants/${grant.id}/users`,grant.issuer);
    assert.equal(issued.status,200);
    const user=JSON.parse(issued.body).userToken;
    async function inject(directory,method,route,secret,body,headers) {
      const dir=join(root,directory);
      fs.chmodSync(dir,0o500);
      try {
        const response=await request(p,method,route,secret,body,headers);
        assert.equal(response.status,500,response.body.toString());
        const health=await request(p,'GET','/health');
        assert.equal(health.status,503,health.body.toString());
        assert.equal(JSON.parse(health.body).error,'recovery_required');
      } finally {fs.chmodSync(dir,0o700);}
      await stop(child,'SIGTERM');child=undefined;
      child=await start(binary,[],root,p);
      assert.equal((await request(p,'GET','/health')).status,200);
    }
    await inject('state/grants','POST','/grants',user,
      JSON.stringify({locations:'test.*',maxBytes:10000,maxObjects:4}));
    await inject('state/grants','POST',`/grants/${grant.id}/users`,grant.issuer);
    await inject('state/grants','DELETE',
      `/grants/${grant.id}/users/${JSON.parse(issued.body).userId}`,grant.issuer);
    assert.equal((await request(p,'POST','/store/still_authorized',user,'yes')).status,201);
    const ecdh=createECDH('prime256v1');ecdh.generateKeys();
    const sub={locations:'*',events:['create'],
      endpoint:'https://push.example.org/metadata-failure',
      keys:{p256dh:ecdh.getPublicKey().toString('base64url'),
        auth:randomBytes(16).toString('base64url')}};
    await inject('state/push-subs','POST','/push/subscriptions',user,JSON.stringify(sub));
    const registered=await request(p,'POST','/push/subscriptions',user,JSON.stringify(sub));
    assert.equal(registered.status,201,registered.body.toString());
    const {id,capability}=JSON.parse(registered.body);
    const file=join(root,'state/push-subs',id+'.json');
    const before=fs.readFileSync(file).toString();
    const route='/push/subscriptions/'+id;
    const header={'X-Skrynia-Capability':capability};
    await inject('state/push-subs','PUT',route,undefined,
      JSON.stringify({endpoint:'https://push.example.org/new-location'}),header);
    assert.equal(fs.readFileSync(file).toString(),before);
    await inject('state/push-subs','DELETE',route,undefined,undefined,header);
    assert(fs.existsSync(file));
    assert.equal((await request(p,'DELETE',route,undefined,undefined,header)).status,200);
    assert(!fs.existsSync(file));
    console.log('Grant and Push metadata I/O faults fail closed; restarts recover: OK');
  } finally {
    if(child)await stop(child,'SIGKILL');
    fs.rmSync(root,{recursive:true,force:true});
  }
}
main().catch(e=>{console.error(e);process.exitCode=1;});
