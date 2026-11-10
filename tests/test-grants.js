#!/usr/bin/env node
'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const http = require('node:http');
const {createServer} = require('../src/server.js');
const {matches, subset, KEY_RE, LOCATION_RE} = require('../src/grants.js');
const DAY = 86400000;
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
function due(dir,ns,id) {
  const p=path.join(dir,'state',ns,'grants',id+'.json');
  const g=JSON.parse(fs.readFileSync(p,'utf8'));
  g.nextCheckAt=Date.now()-1;
  fs.writeFileSync(p,JSON.stringify(g));
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
    assert.deepEqual(root.operations,['write','delegate','keep','release','subscribe']);
    assert.equal(engine.createRoot('forum',120000,100).created,false);

    await demand(port,401,'POST','/store/forum/report1','x');
    await demand(port,401,'POST','/grants',{namespace:'forum',maxBytes:3500,maxObjects:5});
    await demand(port,400,'POST','/store/forum/invalid-name','x',grant(root.userToken));
    await demand(port,400,'POST','/store/forum/a%2Ab','x',grant(root.userToken));
    await demand(port,400,'POST','/store/forum/a%3Ab','x',grant(root.userToken));
    await demand(port,400,'POST','/store/forum/a%5Cb','x',grant(root.userToken));

    const newGrant = (p,token,params)=>demand(p,200,'POST','/grants',
      {namespace:'forum',maxBytes:params.maxBytes,maxObjects:params.maxObjects,
       ...(params.locations===undefined?{}:{locations:params.locations}),
       ...(params.operations===undefined?{}:{operations:params.operations})},user(token));
    const A=(await newGrant(port,root.userToken,
      {maxBytes:3560,maxObjects:5,locations:'report*',operations:['write','delegate']})).json;
    const B=(await newGrant(port,root.userToken,
      {maxBytes:4560,maxObjects:5,locations:'report*',operations:['write']})).json;

    assert.equal(A.locations,'report*');
    assert.deepEqual(A.operations,['write','delegate']);
    await demand(port,403,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1,locations:'wrong*'},user(A.userToken));
    await demand(port,403,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1,locations:'report*',
       operations:['write','keep']},user(A.userToken));
    await demand(port,403,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1},user(B.userToken));
    await demand(port,400,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1,locations:'**'},user(root.userToken));
    await demand(port,400,'POST','/grants',
      {namespace:'forum',maxBytes:2048,maxObjects:1,operations:['write','write']},user(root.userToken));
    await demand(port,403,'POST','/grants/'+B.id+'/users?namespace=forum',null,user(B.userToken));
    await demand(port,403,'POST','/grants/'+B.id+'/confirmation?namespace=forum',
      {need:true},user(B.userToken));

    // B can fill its own budget; A's lower ceiling is not a global invariant.
    await demand(port,201,'POST','/store/forum/report1','x'.repeat(2000),grant(B.userToken));
    const g=await demand(port,200,'GET','/store/forum/report1');
    assert.equal(g.body.length,2000);
    assert(!fs.existsSync(path.join(dir,'store-meta')),'object metadata is unnecessary');
    assert.equal(engine.usage('forum',A.id).bytes,2048+512+2000);
    assert.equal(engine.usage('forum',B.id).bytes,2048+512+2000);
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
       operations:['write','delegate','keep','release','subscribe']})).json;
    const C=(await newGrant(port,X.userToken,
      {maxBytes:15000,maxObjects:10,locations:'secret1'})).json;
    assert.equal(C.locations,'secret1');
    assert.deepEqual(C.operations,X.operations);
    const N=(await newGrant(port,root.userToken,
      {maxBytes:8000,maxObjects:10,locations:'secret*',
       operations:['write']})).json;
    await demand(port,201,'POST','/store/forum/secret1','sibling',grant(N.userToken));
    await demand(port,201,'POST','/store/forum/secret2','local',grant(X.userToken));
    await demand(port,201,'POST','/push/subscriptions?namespace=forum&channel=alerts',
      {endpoint:'https://push.example.test/a1',keys:{p256dh:'sample-key',auth:'sample-auth'}},
      grant(X.userToken));
    await demand(port,403,'POST','/push/subscriptions?namespace=forum&channel=alerts',
      {endpoint:'https://push.example.test/a2',keys:{p256dh:'sample-key',auth:'sample-auth'}},
      grant(N.userToken));

    await demand(port,200,'POST','/grants/'+X.id+'/confirmation?namespace=forum',
      {need:false},user(X.userToken));
    await demand(port,200,'POST','/grants/'+C.id+'/confirmation?namespace=forum',
      {need:true},user(C.userToken));
    await new Promise(ok=>setTimeout(ok,12));
    due(dir,'forum',X.id);
    engine.sweep();
    await demand(port,200,'GET','/store/forum/secret1');
    assert.equal(engine.inspect('forum',C.id,C.userToken).id,C.id);

    // A new release without keep triggers revoke + reissue using identical tokens.
    await demand(port,200,'POST','/grants/'+X.id+'/confirmation?namespace=forum',
      {need:false},user(X.userToken));
    await new Promise(ok=>setTimeout(ok,12));
    due(dir,'forum',X.id);
    engine.sweep();
    await demand(port,404,'GET','/store/forum/secret1');
    await demand(port,404,'GET','/store/forum/secret2');
    assert.equal(engine.inspect('forum',X.id,X.userToken).id,X.id);
    assert.equal(engine.inspect('forum',X.id,X.userToken).locations,'secret*');
    assert.deepEqual(engine.inspect('forum',X.id,X.userToken).operations,X.operations);
    assert.throws(()=>engine.inspect('forum',C.id,C.userToken),/invalid_grant_user_token/);
    assert.equal(engine.inspect('forum',N.id,N.userToken).id,N.id,'unrelated grant survives');
    const pushDir=path.join(dir,'state','forum','push-subs');
    assert.equal(fs.readdirSync(pushDir).filter(x=>x.endsWith('.json')).length,0);
    await demand(port,201,'POST','/store/forum/secret1','reused',grant(X.userToken));
    await demand(port,200,'DELETE','/grants/'+X.id+'/space?namespace=forum',null,user(X.issuerToken));
    await demand(port,404,'GET','/store/forum/secret1');
    await demand(port,201,'POST','/store/forum/secret1','reused-again',grant(X.userToken));
    await demand(port,200,'DELETE','/grants/'+X.id+'?namespace=forum',null,user(X.issuerToken));
    await demand(port,404,'GET','/store/forum/secret1');
    assert.throws(()=>engine.inspect('forum',X.id,X.userToken),/invalid_grant_user_token/);
    assert.equal(engine.inspect('forum',N.id,N.userToken).id,N.id);

    // Exact matcher permits precisely one key, empty matcher permits none.
    const exact=(await newGrant(port,root.userToken,
      {maxBytes:4096,maxObjects:1,locations:'EXACT_1',operations:['write']})).json;
    await demand(port,201,'POST','/store/forum/EXACT_1','ok',grant(exact.userToken));
    await demand(port,403,'POST','/store/forum/EXACT_12','no',grant(exact.userToken));
    const empty=(await newGrant(port,root.userToken,
      {maxBytes:2048,maxObjects:1,locations:'',operations:['write']})).json;
    await demand(port,403,'PUT','/store/forum/EXACT_1','no',grant(empty.userToken));

    // Root revocation removes all namespace keys and invalidates every user token.
    await demand(port,200,'DELETE','/grants/'+root.id+'?namespace=forum',null,user(root.issuerToken));
    await demand(port,404,'GET','/store/forum/report1');
    await demand(port,404,'GET','/store/forum/EXACT_1');
    await demand(port,404,'POST','/store/forum/reborn','no',grant(root.userToken));
    console.log('HTTP and recursive grant behavior: OK');
  } finally {
    if(server) await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
async function testRecovery() {
  const dir=mk();let server,port;
  try{
    ({server,port}=await open(dir));
    const g=server.skrynia.grants;
    const root=g.createRoot('recovery',20000,20);
    const child=g.createChild('recovery',root.userToken,9000,10,'logs*',['write','keep']);
    await demand(port,201,'POST','/store/recovery/logs1','data',grant(child.userToken));
    const marker=path.join(dir,'state','_grant-deletions','recovery-'+child.id+'.json');
    fs.mkdirSync(path.dirname(marker),{recursive:true});
    fs.writeFileSync(marker,JSON.stringify({ns:'recovery',id:child.id,
      action:'clean',ids:[child.id],locations:'logs*'}));
    await close(server);server=null;
    ({server,port}=await open(dir));
    await demand(port,404,'GET','/store/recovery/logs1');
    assert.equal(fs.existsSync(marker),false);
    assert.equal(server.skrynia.grants.inspect('recovery',child.id,child.userToken).id,child.id);
    await demand(port,201,'POST','/store/recovery/logs1','new',grant(child.userToken));
    console.log('interrupted cleanup recovery: OK');
  }finally{
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
(async()=>{checkGrammar();await testHttp();await testRecovery();})()
  .catch(e=>{console.error(e);process.exitCode=1;});
