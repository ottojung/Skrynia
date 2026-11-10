#!/usr/bin/env node
'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const http = require('node:http');
const {createServer} = require('../src/server.js');
const {matches, subset, KEY_RE, LOCATION_RE} = require('../src/grants.js');
const mk = () => fs.mkdtempSync(path.join(os.tmpdir(), 'skrynia-spec-'));
async function open(dir) {
  const server = createServer({dataDir:dir,token:'admin-token',
    skryniaUrl:'http://127.0.0.1:17880/platform',pushManual:true});
  await new Promise(ok=>server.listen(0,'127.0.0.1',ok));
  return {server,port:server.address().port};
}
const close = server=>new Promise(ok=>server.close(ok));
const user = token=>({Authorization:'Bearer '+token});
const grant = token=>({'X-Skrynia-Grant':token});
function request(port, method, endpoint, body, headers={}) {
  return new Promise((resolve,reject)=>{
    const data = body == null ? null : Buffer.isBuffer(body) ? body :
      typeof body === 'object' ? Buffer.from(JSON.stringify(body)) : Buffer.from(body);
    const req = http.request({hostname:'127.0.0.1',port,method,
      path:'/platform'+endpoint,headers:{...headers,...(data?{'Content-Length':data.length}:{})}},res=>{
      const chunks=[]; res.on('data',c=>chunks.push(c)); res.on('end',()=>{
        const bytes=Buffer.concat(chunks), raw=bytes.toString('utf8');
        let json;try{json=JSON.parse(raw);}catch{}
        resolve({status:res.statusCode,body:raw,json,etag:res.headers.etag,bytes});
      });
    });
    req.on('error',reject);
    if(data)req.write(data);
    req.end();
  });
}
async function demand(port, status, method, url, body, headers) {
  const r=await request(port,method,url,body,headers);
  assert.equal(r.status,status,method+' '+url+': '+r.body);
  return r;
}
function checkGrammar() {
  for(const key of ['report','report12','REPORT_3','_x','A0']) assert(KEY_RE.test(key));
  for(const key of ['','report-1','a:b','ab*','a\\b','a/b','é']) assert(!KEY_RE.test(key));
  for(const pattern of ['','*','report','report*','A_1*']) assert(LOCATION_RE.test(pattern));
  for(const pattern of ['**','*a','a**','abc:','a-b','a\\b']) assert(!LOCATION_RE.test(pattern));
  assert(matches('*','A_1'));
  assert(matches('report*','report') && matches('report*','report_1'));
  assert(!matches('report','report1') && !matches('','report'));
  assert(subset('', 'report'));
  assert(subset('report_1', 'report*'));
  assert(subset('report_1*','report*'));
  assert(!subset('report*','report_1') && !subset('*','report*'));
  assert(!subset('foo*','foo'));
  assert(subset('report','report'));
  console.log('matchers: OK');
}
async function testHttp() {
  const dir=mk(); let server,port;
  try {
    ({server,port}=await open(dir));
    const engine=server.skrynia.grants;
    const root=engine.createRoot('forum',120000,100);
    assert.equal(root.locations,'*');
    assert.deepEqual(root.operations,['write','delegate','subscribe']);
    assert.equal(engine.createRoot('forum',120000,100).created,false);
    assert.equal(root.userToken,undefined,'creation must return issuer token only');
    assert.equal(engine.usage('forum',root.id).bytes,2048,'unassigned grant has base cost');

    await demand(port,401,'POST','/store/forum/report1','x');
    await demand(port,401,'POST','/grants',{namespace:'forum',maxBytes:3500,maxObjects:5});
    await demand(port,403,'POST','/grants/'+root.id+'/users?namespace=forum');
    const firstUser = await demand(port,200,'POST',
      '/grants/'+root.id+'/users?namespace=forum',null,user(root.issuerToken));
    root.userToken=firstUser.json.userToken;
    assert.match(root.userToken,/^[a-f0-9]{64}$/);
    assert.equal(engine.inspect('forum',root.id,root.userToken).userCount,1);
    assert.equal(engine.usage('forum',root.id).bytes,2048,'first user included in grant metadata');
    const tiny=engine.createRoot('tiny',2048,1);
    await demand(port,200,'POST','/grants/'+tiny.id+'/users?namespace=tiny',
      null,user(tiny.issuerToken));
    await demand(port,507,'POST','/grants/'+tiny.id+'/users?namespace=tiny',
      null,user(tiny.issuerToken));
    await demand(port,403,'POST','/grants/'+root.id+'/users?namespace=forum',
      null,user(root.userToken));
    await demand(port,403,'POST','/grants/'+root.id+'/users?namespace=forum',
      null,user('f'.repeat(64)));
    await demand(port,400,'POST','/store/forum/invalid-name','x',grant(root.userToken));
    await demand(port,400,'POST','/store/forum/a%2Ab','x',grant(root.userToken));
    await demand(port,400,'POST','/store/forum/a%3Ab','x',grant(root.userToken));
    await demand(port,400,'POST','/store/forum/a%5Cb','x',grant(root.userToken));

    const newGrant = async (p,token,params)=>{
      const issued=await demand(p,200,'POST','/grants',
        {namespace:'forum',maxBytes:params.maxBytes,maxObjects:params.maxObjects,
         ...(params.locations===undefined?{}:{locations:params.locations}),
         ...(params.operations===undefined?{}:{operations:params.operations})},user(token));
      assert.equal(issued.json.userToken,undefined,'child issuance returns no user token');
      assert.match(issued.json.issuerToken,/^[a-f0-9]{64}$/);
      const minted=await demand(p,200,'POST','/grants/'+issued.json.id+'/users?namespace=forum',
        null,user(issued.json.issuerToken));
      return {json:{...issued.json,userToken:minted.json.userToken}};
    };
    const A=(await newGrant(port,root.userToken,
      {maxBytes:3560,maxObjects:5,locations:'report*',operations:['write','delegate']})).json;
    const B=(await newGrant(port,root.userToken,
      {maxBytes:4688,maxObjects:5,locations:'report*',operations:['write']})).json;

    assert.equal(A.locations,'report*');
    assert.deepEqual(A.operations,['write','delegate']);
    await demand(port,403,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1,locations:'wrong*'},user(A.userToken));
    await demand(port,403,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1,locations:'report*',
       operations:['write','subscribe']},user(A.userToken));
    await demand(port,403,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1},user(B.userToken));
    await demand(port,400,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1,locations:'**'},user(root.userToken));
    await demand(port,400,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1,operations:['write','write']},user(root.userToken));
    await demand(port,403,'POST','/grants/'+B.id+'/users?namespace=forum',null,user(B.userToken));
    await demand(port,403,'POST','/grants/'+A.id+'/users?namespace=forum',null,user(A.userToken));
    await demand(port,403,'POST','/grants/'+B.id+'/users?namespace=forum',null,user(root.issuerToken));
    // The issuer may mint users even if the grant has no 'delegate' operation.
    const secondBUser=await demand(port,200,'POST',
      '/grants/'+B.id+'/users?namespace=forum',null,user(B.issuerToken));
    assert.notEqual(secondBUser.json.userToken,B.userToken);
    assert.equal(engine.inspect('forum',B.id,B.userToken).userCount,2);
    await demand(port,400,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1,operations:['unrecognized']},
      user(root.userToken));

    // B can fill its own budget; A's lower ceiling is not a global invariant.
    await demand(port,201,'POST','/store/forum/report1','x'.repeat(2000),grant(B.userToken));
    const g=await demand(port,200,'GET','/store/forum/report1');
    assert.equal(g.body.length,2000);
    assert(!fs.existsSync(path.join(dir,'store-meta')),'object metadata is unnecessary');
    assert.equal(engine.usage('forum',A.id).bytes,2048+512+2000);
    assert.equal(engine.usage('forum',B.id).bytes,2048+512+2000+128);
    await demand(port,507,'PUT','/store/forum/report1','a'.repeat(1500),
      {...grant(A.userToken),'If-Match':g.etag});
    assert.equal((await demand(port,200,'GET','/store/forum/report1')).body.length,2000);
    await demand(port,200,'PUT','/store/forum/report1','a'.repeat(500),
      {...grant(A.userToken),'If-Match':g.etag});
    const after=await demand(port,200,'GET','/store/forum/report1');
    assert.equal(after.body.length,500);
    assert.notEqual(after.etag,g.etag);
    await demand(port,412,'PUT','/store/forum/report1','no', {...grant(B.userToken),'If-Match':g.etag});
    await demand(port,400,'PUT','/store/forum/report1','no',
      {...grant(B.userToken),'X-Skrynia-Mode':'public-write'});
    await demand(port,401,'DELETE','/store/forum/report1');
    await demand(port,200,'DELETE','/store/forum/report1',null,grant(A.userToken));
    await demand(port,201,'POST','/store/forum/report1','again',grant(B.userToken));
    const recreated=await demand(port,200,'GET','/store/forum/report1');
    assert.notEqual(recreated.etag,after.etag,'no ETag ABA');
    await demand(port,403,'POST','/store/forum/secret1','no',grant(A.userToken));

    // Child scope and operations cannot exceed their parent, and omitted fields inherit.
    const X=(await newGrant(port,root.userToken,
      {maxBytes:30000,maxObjects:20,locations:'secret*',
       operations:['write','delegate','subscribe']})).json;
    const C=(await newGrant(port,X.userToken,
      {maxBytes:15000,maxObjects:10,locations:'secret1'})).json;
    assert.equal(C.locations,'secret1');
    assert.deepEqual(C.operations,X.operations);
    const N=(await newGrant(port,root.userToken,
      {maxBytes:8000,maxObjects:10,locations:'secret1',
       operations:['write']})).json;
    await demand(port,201,'POST','/store/forum/secret1','sibling',grant(N.userToken));
    await demand(port,201,'POST','/store/forum/secret2','local',grant(X.userToken));
    await demand(port,201,'POST','/push/subscriptions?namespace=forum&channel=alerts',
      {endpoint:'https://push.example.test/a1',keys:{p256dh:'sample-key',auth:'sample-auth'}},
      grant(X.userToken));
    await demand(port,403,'POST','/push/subscriptions?namespace=forum&channel=alerts',
      {endpoint:'https://push.example.test/a2',keys:{p256dh:'sample-key',auth:'sample-auth'}},
      grant(N.userToken));

    // Revocation removes the subtree's authority and subscriptions, not stored values.
    await demand(port,200,'DELETE','/grants/'+X.id+'?namespace=forum',
      null,user(X.issuerToken));
    assert.equal((await demand(port,200,'GET','/store/forum/secret1')).body,'sibling');
    assert.equal((await demand(port,200,'GET','/store/forum/secret2')).body,'local');
    await demand(port,403,'PUT','/store/forum/secret2','denied',grant(X.userToken));
    await demand(port,403,'PUT','/store/forum/secret1','denied',grant(C.userToken));
    assert.throws(()=>engine.inspect('forum',C.id,C.userToken),/invalid_grant_user_token/);
    assert.equal(engine.inspect('forum',N.id,N.userToken).id,N.id);
    const pushDir=path.join(dir,'state','forum','push-subs');
    assert.equal(fs.readdirSync(pushDir).filter(x=>x.endsWith('.json')).length,0);

    // Exact matcher permits precisely one key, empty matcher permits none.
    const exact=(await newGrant(port,root.userToken,
      {maxBytes:4096,maxObjects:1,locations:'EXACT_1',operations:['write']})).json;
    await demand(port,201,'POST','/store/forum/EXACT_1','ok',grant(exact.userToken));
    await demand(port,403,'POST','/store/forum/EXACT_12','no',grant(exact.userToken));
    const empty=(await newGrant(port,root.userToken,
      {maxBytes:2048,maxObjects:1,locations:'',operations:['write']})).json;
    await demand(port,403,'PUT','/store/forum/EXACT_1','no',grant(empty.userToken));

    // Root revocation invalidates all delegated credentials without deleting objects.
    await demand(port,200,'DELETE','/grants/'+root.id+'?namespace=forum',null,user(root.issuerToken));
    assert.equal((await demand(port,200,'GET','/store/forum/report1')).body,'again');
    assert.equal((await demand(port,200,'GET','/store/forum/EXACT_1')).body,'ok');
    await demand(port,404,'POST','/store/forum/reborn','no',grant(root.userToken));
    // A replacement root can administer the retained namespace.
    const nextRoot=engine.createRoot('forum',120000,100);
    const nextUser=engine.createUser('forum',nextRoot.id,nextRoot.issuerToken);
    await demand(port,200,'DELETE','/store/forum/report1',null,grant(nextUser.userToken));
    assert.equal((await demand(port,200,'GET','/store/forum/EXACT_1')).body,'ok');
    console.log('HTTP and recursive grant behavior: OK');
  } finally {
    if(server) await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
async function testGrantLifetime() {
  const dir=mk();let server,port;
  try {
    ({server,port}=await open(dir));
    const grants=server.skrynia.grants, ns='lifetime';
    const root=grants.createRoot(ns,30000,20);
    root.userToken=grants.createUser(ns,root.id,root.issuerToken).userToken;
    const parent=grants.createChild(ns,root.userToken,12000,10,'record*',['write','delegate']);
    parent.userToken=grants.createUser(ns,parent.id,parent.issuerToken).userToken;
    const child=grants.createChild(ns,parent.userToken,4096,2,'record1',[]);
    const sibling=grants.createChild(ns,root.userToken,2048,1,'record1',[]);
    await demand(port,201,'POST','/store/lifetime/record1','parent',grant(parent.userToken));
    await demand(port,201,'POST','/store/lifetime/record2','parent2',grant(parent.userToken));
    await demand(port,200,'DELETE','/grants/'+child.id+'?namespace=lifetime',
      null,user(child.issuerToken));
    assert.equal((await demand(port,200,'GET','/store/lifetime/record1')).body,'parent');
    await demand(port,200,'DELETE','/grants/'+parent.id+'?namespace=lifetime',
      null,user(parent.issuerToken));
    assert.equal((await demand(port,200,'GET','/store/lifetime/record1')).body,'parent');
    assert.equal((await demand(port,200,'GET','/store/lifetime/record2')).body,'parent2');
    assert.equal(grants.usage(ns,root.id).objects,2);
    await demand(port,200,'DELETE','/grants/'+sibling.id+'?namespace=lifetime',
      null,user(sibling.issuerToken));
    assert.equal((await demand(port,200,'GET','/store/lifetime/record1')).body,'parent');
    await demand(port,200,'DELETE','/grants/'+root.id+'?namespace=lifetime',
      null,user(root.issuerToken));
    assert.equal((await demand(port,200,'GET','/store/lifetime/record1')).body,'parent');
    assert.equal((await demand(port,200,'GET','/store/lifetime/record2')).body,'parent2');
    await close(server);server=null;
    ({server,port}=await open(dir));
    assert.equal((await demand(port,200,'GET','/store/lifetime/record1')).body,'parent');
    const nextRoot=server.skrynia.grants.createRoot(ns,30000,20);
    const nextUser=server.skrynia.grants.createUser(ns,nextRoot.id,nextRoot.issuerToken);
    await demand(port,200,'DELETE','/store/lifetime/record1',null,grant(nextUser.userToken));
    await demand(port,404,'GET','/store/lifetime/record1');
    await demand(port,200,'GET','/store/lifetime/record2');
    console.log('recursive revocation, data persistence, root reissue: OK');
  } finally {
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
async function testRecovery() {
  const dir=mk();let server,port;
  try {
    ({server,port}=await open(dir));
    const ns='recovery', g=server.skrynia.grants;
    const root=g.createRoot(ns,30000,20);
    root.userToken=g.createUser(ns,root.id,root.issuerToken).userToken;
    const parent=g.createChild(ns,root.userToken,12000,10,'logs*',['write','delegate']);
    parent.userToken=g.createUser(ns,parent.id,parent.issuerToken).userToken;
    const child=g.createChild(ns,parent.userToken,4096,2,'logs1',[]);
    await demand(port,201,'POST','/store/recovery/logs1','lasting1',grant(parent.userToken));
    await demand(port,201,'POST','/store/recovery/logs2','lasting2',grant(parent.userToken));
    const marker=path.join(dir,'state','_grant-revocations','recovery-'+parent.id+'.json');
    const childFile=path.join(dir,'state',ns,'grants',child.id+'.json');
    const unlink=fs.unlinkSync;
    try {
      let injected=false;
      fs.unlinkSync=function(file) {
        const result=unlink.call(fs,file);
        if(!injected && file===childFile) {
          injected=true;
          throw new Error('injected_revocation_crash');
        }
        return result;
      };
      assert.throws(()=>g.revokeGrant(ns,parent.id,parent.issuerToken),
        /injected_revocation_crash/);
      assert(injected,'fault injection reached a grant file');
    } finally {fs.unlinkSync=unlink;}
    assert(fs.existsSync(marker));
    const pending=JSON.parse(fs.readFileSync(marker,'utf8'));
    assert.deepEqual(new Set(pending.ids),new Set([parent.id,child.id]));
    await close(server);server=null;
    ({server,port}=await open(dir));
    assert.equal(fs.existsSync(marker),false);
    assert.equal((await demand(port,200,'GET','/store/recovery/logs1')).body,'lasting1');
    assert.equal((await demand(port,200,'GET','/store/recovery/logs2')).body,'lasting2');
    assert.throws(()=>server.skrynia.grants.usage(ns,parent.id),/grant_not_found/);
    assert.throws(()=>server.skrynia.grants.usage(ns,child.id),/grant_not_found/);
    assert.equal(server.skrynia.grants.usage(ns,root.id).objects,2);
    await demand(port,403,'PUT','/store/recovery/logs1','denied',grant(parent.userToken));
    console.log('interrupted revocation recovery: OK');
  } finally {
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
(async()=>{checkGrammar();await testHttp();await testGrantLifetime();await testRecovery();})()
  .catch(e=>{console.error(e);process.exitCode=1;});
