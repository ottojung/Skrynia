#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();let server;
  try {
    let started=await open(dir);server=started.server;
    let p=started.port;
    const root=server.skrynia.grants.root();
    const user=(await expect(p,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json;
    const child=(await expect(p,200,'POST','/grants',{
      locations:'valid.*',maxBytes:8000,maxObjects:3,
      operations:['write','delegate']},auth(user.userToken))).json;
    const childToken=(await expect(p,200,'POST','/grants/'+child.id+'/users',
      null,auth(child.issuerToken))).json;
    await expect(p,401,'POST','/store/valid.first','no');
    await expect(p,403,'POST','/store/valid.first','no',grant('f'.repeat(64)));
    await expect(p,400,'POST','/store/invalid%2Fslash','no',grant(childToken.userToken));
    await expect(p,400,'POST','/store/valid%20space','no',grant(childToken.userToken));
    await expect(p,201,'POST','/store/valid.first','initial',grant(childToken.userToken));
    const before=await expect(p,200,'GET','/store/valid.first');
    await expect(p,412,'DELETE','/store/valid.first',null,
      {...grant(childToken.userToken),'If-Match':'"bad"'});
    await expect(p,200,'DELETE','/store/valid.first',null,
      {...grant(childToken.userToken),'If-Match':before.etag});
    await expect(p,201,'POST','/store/valid.first','recreated',grant(childToken.userToken));
    const after=await expect(p,200,'GET','/store/valid.first');
    assert.notEqual(after.etag,before.etag,'ETag generation prevents ABA after deletion');
    assert.equal(after.headers['x-content-type-options'],'nosniff');
    await expect(p,200,'DELETE','/grants/'+child.id+'/users/'+childToken.userId,
      null,auth(child.issuerToken));
    await expect(p,403,'POST','/store/valid.another','no',grant(childToken.userToken));
    await expect(p,200,'GET','/store/valid.first');

    // A durable revocation marker must finish removing the grant subtree
    // during startup recovery, without deleting object values.
    const marker=path.join(dir,'state','grant-revocations',child.id+'.json');
    fs.writeFileSync(marker,JSON.stringify({id:child.id,ids:[child.id]}),{mode:0o600});
    await close(server);server=null;
    started=await open(dir);server=started.server;p=started.port;
    assert.equal(server.skrynia.grants.root().id,root.id);
    assert.equal(server.skrynia.grants.root().rootIssuerToken,root.rootIssuerToken);
    assert.equal(fs.existsSync(marker),false);
    await expect(p,403,'GET','/grants/'+child.id,null,auth(user.userToken));
    await expect(p,200,'GET','/store/valid.first');

    // A crash or storage fault after commit cannot convert a successful
    // object mutation into a rollback. The journal is replayed at restart.
    const original=server.skrynia.push.enqueue;
    server.skrynia.push.enqueue=()=>{throw new Error('injected_outbox_failure');};
    const originalError=console.error;
    let expectedLog='';
    console.error=(...args)=>{expectedLog+=String(args[0]);};
    let write;
    try {
      write=await expect(p,201,'POST','/store/valid.journal','durable',
        grant(user.userToken));
    } finally {
      console.error=originalError;
      server.skrynia.push.enqueue=original;
    }
    assert.match(expectedLog,/committed value, pending push outbox/);
    assert.equal(write.json.notificationPending,true);
    assert.equal((await expect(p,200,'GET','/store/valid.journal')).bytes.toString(),'durable');
    await close(server);server=null;
    started=await open(dir);server=started.server;p=started.port;
    assert.equal((await expect(p,200,'GET','/store/valid.journal')).bytes.toString(),'durable');
    assert.equal(fs.readdirSync(path.join(dir,'state','store-transactions')).length,0);

    // Private root credential corruption must fail rather than silently
    // minting unrelated authority over a still-live grant hierarchy.
    await close(server);server=null;
    const rootFile=path.join(dir,'state','grants',root.id+'.json');
    const record=JSON.parse(fs.readFileSync(rootFile));
    record.rootIssuerToken='0'.repeat(64);
    fs.writeFileSync(rootFile,JSON.stringify(record));
    assert.throws(()=>require('../src/server.js').createServer({
      dataDir:dir,skryniaUrl:'http://127.0.0.1:17380/platform',pushManual:true}),
      /invalid root grant/);
    console.log('authorization, ETags, crash recovery and bootstrap integrity: OK');
  }finally {
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
