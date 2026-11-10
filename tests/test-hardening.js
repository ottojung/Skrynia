#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const os=require('node:os');
const http=require('node:http');
const {execFile}=require('node:child_process');
const {promisify}=require('node:util');
const runCli=promisify(execFile);
const {createServer}=require('../src/server.js');

const tmp=()=>fs.mkdtempSync(path.join(os.tmpdir(),'skrynia-hardening-'));
const auth=t=>({Authorization:'Bearer '+t});
const grant=t=>({'X-Skrynia-Grant':t});
async function open(dir) {
  const server=createServer({dataDir:dir,token:'admin',
    skryniaUrl:'http://127.0.0.1:17380/platform',pushManual:true});
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  return {server,port:server.address().port};
}
async function close(server) {
  await new Promise(resolve=>server.close(resolve));
}
function req(port,method,route,body,headers={}) {
  return new Promise((resolve,reject)=>{
    let data=null;
    if(body!==undefined && body!==null){
      data=Buffer.isBuffer(body)?body:
        typeof body==='string'?Buffer.from(body):Buffer.from(JSON.stringify(body));
    }
    const request=http.request({hostname:'127.0.0.1',port,
      method,path:'/platform'+route,
      headers:{...headers,...(data?{'Content-Length':data.length}:{})}},response=>{
        const parts=[];
        response.on('data',x=>parts.push(x));
        response.on('end',()=>{
          const text=Buffer.concat(parts).toString('utf8');
          let json;
          try{json=JSON.parse(text);}catch{}
          resolve({status:response.statusCode,json,text,headers:response.headers});
        });
      });
    request.on('error',reject);
    if(data) request.write(data);
    request.end();
  });
}
async function expect(port,status,method,url,body,headers) {
  const r=await req(port,method,url,body,headers);
  assert.equal(r.status,status,method+' '+url+': '+r.text);
  return r;
}
async function run() {
  const dir=tmp();
  let server,port;
  try {
    ({server,port}=await open(dir));
    const g=server.skrynia.grants;
    const root=g.createRoot('audit',100000,100);
    root.userToken=g.createUser('audit',root.id,root.issuerToken).userToken;
    const second=await expect(port,200,'POST',
      '/grants/'+root.id+'/users?namespace=audit',null,auth(root.issuerToken));
    const guest=second.json;
    assert.match(guest.userId,/^[a-f0-9]{32}$/);
    assert.match(guest.userToken,/^[a-f0-9]{64}$/);
    assert.deepEqual(Object.keys(guest).sort(),['grantId','userId','userToken']);
    await expect(port,400,'POST',
      '/grants/'+root.id+'/users?namespace=audit',{label:'guest'},auth(root.issuerToken));
    await expect(port,405,'GET','/grants/'+root.id+'/users?namespace=audit',
      null,auth(root.issuerToken));
    const records=g.listGrants('audit').grants;
    assert.equal(records.find(x=>x.id===root.id).userCount,2);
    const rootRecord=JSON.parse(fs.readFileSync(
      path.join(dir,'state','audit','grants',root.id+'.json'),'utf8'));
    assert(rootRecord.users.every(u=>
      Object.keys(u).sort().join(',')==='hash,id'),
      'Skrynia stores only opaque user IDs and verifiers');

    // The CLI can issue and revoke a token with no request body or metadata.
    const cliEnv={...process.env,
      SKRYNIA_URL:'http://127.0.0.1:'+port+'/platform',
      SKRYNIA_ISSUER_TOKEN:root.issuerToken};
    const cliPath=path.join(__dirname,'..','scripts','cli.py');
    const {stdout:issuedOutput}=await runCli('python3',
      [cliPath,'grant','issue-user','--namespace','audit','--grant-id',root.id],
      {env:cliEnv});
    const cliUser=JSON.parse(issuedOutput);
    assert.match(cliUser.userId,/^[a-f0-9]{32}$/);
    const {stdout:revokedOutput}=await runCli('python3',
      [cliPath,'grant','revoke-user','--namespace','audit','--grant-id',root.id,
       '--user-id',cliUser.userId],{env:cliEnv});
    assert.equal(JSON.parse(revokedOutput).userId,cliUser.userId);
    await expect(port,403,'POST','/store/audit/cli_revoked','x',
      grant(cliUser.userToken));

    await expect(port,403,'DELETE',
      '/grants/'+root.id+'/users/'+guest.userId+'?namespace=audit',{},auth(guest.userToken));
    // Management authentication is header-only; GET may never mutate state.
    await expect(port,405,'GET','/ns/issuer/rotate?namespace=audit&token=admin');
    await expect(port,401,'POST','/ns/issuer/rotate',{namespace:'audit'});
    await expect(port,401,'POST','/ns/issuer/rotate',{namespace:'audit'},auth('wrong'));
    const rotated=await expect(port,200,'POST','/ns/issuer/rotate',
      {namespace:'audit'},auth('admin'));
    assert.equal(rotated.json.grantId,root.id);
    const nextIssuer=rotated.json.issuerToken;
    assert.notEqual(nextIssuer,root.issuerToken);
    await expect(port,403,'POST','/grants/'+root.id+'/users?namespace=audit',
      null,auth(root.issuerToken));
    await expect(port,200,'DELETE',
      '/grants/'+root.id+'/users/'+guest.userId+'?namespace=audit',
      {},auth(nextIssuer));
    await expect(port,403,'POST','/store/audit/revoked','x',grant(guest.userToken));
    await expect(port,201,'POST','/store/audit/item','survives',grant(root.userToken));

    const D=g.createChild('audit',root.userToken,10000,10,'item',['delegate']);
    D.userToken=g.createUser('audit',D.id,D.issuerToken).userToken;
    const E=g.createChild('audit',D.userToken,4096,1,'item',[]);
    // Issuer cleanup is independent of operations. Even an empty-operations
    // grant can revoke its space; ancestors do not protect matching objects.
    await expect(port,200,'DELETE','/grants/'+E.id+'?namespace=audit',
      null,auth(E.issuerToken));
    await expect(port,404,'GET','/store/audit/item');
    await expect(port,201,'POST','/store/audit/item','recreated',grant(root.userToken));
    // A grant with just 'delegate' has exactly the same cleanup power.
    await expect(port,200,'DELETE','/grants/'+D.id+'/space?namespace=audit',
      null,auth(D.issuerToken));
    await expect(port,404,'GET','/store/audit/item');
    assert.equal(g.inspect('audit',D.id,D.userToken).id,D.id,
      'issuer clean reissues the same grant without changing credentials');

    // Revocation with 'release' behaves identically; the operation is
    // relevant to voting, not to issuer cleanup.
    await expect(port,201,'POST','/store/audit/temp','throwaway',grant(root.userToken));
    const releasing=g.createChild('audit',root.userToken,4096,1,'temp',['release']);
    await expect(port,200,'DELETE','/grants/'+releasing.id+'?namespace=audit',
      null,auth(releasing.issuerToken));
    await expect(port,404,'GET','/store/audit/temp');

    // A release vote also triggers cleanup without granting ordinary write.
    await expect(port,201,'POST','/store/audit/voteonly','voted',grant(root.userToken));
    const voting=g.createChild('audit',root.userToken,4096,1,'voteonly',['release']);
    const voter=g.createUser('audit',voting.id,voting.issuerToken).userToken;
    await expect(port,200,'POST','/grants/'+voting.id+'/confirmation?namespace=audit',
      {need:false},auth(voter));
    const recordPath=path.join(dir,'state','audit','grants',voting.id+'.json');
    const record=JSON.parse(fs.readFileSync(recordPath,'utf8'));
    record.nextCheckAt=Math.max(Date.now()+4,record.releaseAt+4);
    fs.writeFileSync(recordPath,JSON.stringify(record));
    g.reloadGrants('audit');
    await new Promise(resolve=>setTimeout(resolve,12));
    g.sweep();
    await expect(port,404,'GET','/store/audit/voteonly');
    assert.equal(g.inspect('audit',voting.id,voter).id,voting.id);

    // Use and mutate the incremental index without rescanning the object dir.
    g.usage('audit',root.id);
    const storeDir=path.join(dir,'store','audit');
    const grantsDir=path.join(dir,'state','audit','grants');
    const subsDir=path.join(dir,'state','audit','push-subs');
    const originalReaddir=fs.readdirSync;
    fs.readdirSync=function(file,...rest){
      if(file===storeDir || file===grantsDir || file===subsDir) {
        throw new Error('resource directory scanned during quota check');
      }
      return originalReaddir.call(fs,file,...rest);
    };
    try {
      for(let i=0;i<30;i++) {
        await expect(port,201,'POST','/store/audit/key'+String(i).padStart(2,'0'),
          'x'.repeat(i),grant(root.userToken));
      }
      await expect(port,200,'PUT','/store/audit/key02','y'.repeat(90),
        grant(root.userToken));
      await expect(port,200,'DELETE','/store/audit/key04',null,
        grant(root.userToken));
      const usage=g.usage('audit',root.id);
      assert.equal(usage.objects,29);
    } finally {fs.readdirSync=originalReaddir;}
    const keys=await expect(port,200,'GET','/ns/keys?namespace=audit&prefix=key&limit=5',
      null,auth('admin'));
    assert.deepEqual(keys.json.keys.map(x=>x.key),[
      'key00','key01','key02','key03','key05']);
    assert.equal(keys.json.nextAfter,'key05');
    const next=await expect(port,200,'GET',
      '/ns/keys?namespace=audit&prefix=key&after=key05&limit=5',
      null,auth('admin'));
    assert.equal(next.json.keys[0].key,'key06');
    const grants=await expect(port,200,'GET','/ns/grants?namespace=audit',
      null,auth('admin'));
    assert(grants.json.grants.some(x=>x.id===root.id));
    assert(!JSON.stringify(grants.json).includes(nextIssuer));

    // After the object commits, a push enqueue fault leaves a durable intent
    // and success-with-pending response, then restart replays the event.
    server.skrynia.push.setRule('audit','event',['create'],['updates']);
    const originalEnqueue=server.skrynia.push.enqueue;
    server.skrynia.push.enqueue=function(){
      throw new Error('injected_notification_disk_failure');
    };
    const changed=await expect(port,201,'POST','/store/audit/event','durable',
      grant(root.userToken));
    assert.equal(changed.json.notificationPending,true);
    assert.equal((await expect(port,200,'GET','/store/audit/event')).text,'durable');
    const markerDir=path.join(dir,'state','_store-transactions');
    assert.equal(fs.readdirSync(markerDir).filter(x=>x.endsWith('.json')).length,1);
    server.skrynia.push.enqueue=originalEnqueue;
    // A newer replacement must not cancel notification of the first commit.
    await expect(port,200,'PUT','/store/audit/event','updated',grant(root.userToken));
    const orphan=path.join(dir,'store-tmp','audit.999.'+'f'.repeat(24));
    fs.writeFileSync(orphan,'abandoned');
    await close(server);server=null;
    ({server,port}=await open(dir));
    assert.equal(fs.existsSync(orphan),false,'restart reclaims abandoned staging files');
    assert.equal(fs.readdirSync(markerDir).filter(x=>x.endsWith('.json')).length,0);
    assert.equal(server.skrynia.push.listOutbox().length,1);
    assert.equal((await expect(port,200,'GET','/store/audit/event')).text,'updated');
    assert.equal(server.skrynia.grants.usage('audit',root.id).objects,30);

    // Undeploy must remove pending namespace events and commit intents too.
    server.skrynia.push.enqueue('audit','event','create');
    const stale=path.join(markerDir,'f'.repeat(32)+'.json');
    fs.writeFileSync(stale,JSON.stringify({ns:'audit',key:'gone',
      kind:'delete',version:123}));
    await expect(port,200,'DELETE','/undeploy',{namespace:'audit'},auth('admin'));
    assert.equal(fs.readdirSync(markerDir).filter(x=>x.endsWith('.json')).length,0);
    assert.equal(server.skrynia.push.listOutbox().length,0);
    const fresh=server.skrynia.grants.createRoot('audit',4096,2);
    assert.equal(server.skrynia.grants.usage('audit',fresh.id).objects,0);
    console.log('P0/P1 lifecycle, authority, indexed usage, discovery, crash replay: OK');
  } finally {
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
