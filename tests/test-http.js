#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const http=require('node:http');
const {createShared}=require('../src/shared.js');
const {createResourceIndex}=require('../src/resource-index.js');
const {validKey,matches}=require('../src/grants.js');
const {open,close,expect,request,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();let server;
  try {
    const started=await open(dir);server=started.server;
    const p=started.port,root=server.skrynia.grants.root();
    const token=(await expect(p,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;

    await expect(p,201,'POST','/store/doc.alpha','0123456789',grant(token));
    const original=await expect(p,200,'GET','/store/doc.alpha');
    const tag=original.etag;
    assert.equal(original.headers['accept-ranges'],'bytes');
    const head=await expect(p,200,'HEAD','/store/doc.alpha');
    assert.equal(head.bytes.length,0);
    assert.equal(head.headers['content-length'],'10');
    assert.equal(head.etag,tag);
    const fresh=await expect(p,304,'GET','/store/doc.alpha',null,{
      'If-None-Match':tag});
    assert.equal(fresh.bytes.length,0);
    await expect(p,304,'GET','/store/doc.alpha',null,{
      'If-None-Match':'W/'+tag});
    await expect(p,304,'HEAD','/store/doc.alpha',null,{
      'If-None-Match':'"other", '+tag});
    const partial=await expect(p,206,'GET','/store/doc.alpha',null,{
      Range:'bytes=2-4'});
    assert.equal(partial.bytes.toString(),'234');
    assert.equal(partial.headers['content-range'],'bytes 2-4/10');
    assert.equal(partial.headers['content-length'],'3');
    assert.equal((await expect(p,206,'GET','/store/doc.alpha',null,{
      Range:'bytes=-3'})).bytes.toString(),'789');
    assert.equal((await expect(p,206,'GET','/store/doc.alpha',null,{
      Range:'bytes=8-'})).bytes.toString(),'89');
    assert.equal((await expect(p,200,'GET','/store/doc.alpha',null,{
      Range:'bytes=2-4','If-Range':'"stale"'})).bytes.toString(),'0123456789');
    const unsat=await expect(p,416,'GET','/store/doc.alpha',null,{
      Range:'bytes=12-15'});
    assert.equal(unsat.headers['content-range'],'bytes */10');
    assert.equal(unsat.bytes.length,0);

    for(const key of ['doc.beta','doc.gamma','doc.delta']) {
      await expect(p,201,'POST','/store/'+key,key,grant(token));
    }
    let batch=await expect(p,200,'GET','/store?locations=doc.*&limit=2',
      null,grant(token));
    assert.deepEqual(batch.json,{keys:['doc.alpha','doc.beta'],next:'doc.beta'});
    batch=await expect(p,200,'GET',
      '/store?locations=doc.*&limit=2&after=doc.beta',null,grant(token));
    assert.deepEqual(batch.json,{keys:['doc.delta','doc.gamma']});
    assert.deepEqual((await expect(p,200,'GET',
      '/store?locations=doc.*&after=doc.zzz',null,grant(token))).json,
      {keys:[]});
    await expect(p,400,'GET','/store?locations=doc.*&limit=0',
      null,grant(token));
    await expect(p,400,'GET','/store?locations=doc.*&limit=1001',
      null,grant(token));
    await expect(p,400,'GET','/store?locations=doc.*&after=.hidden',
      null,grant(token));
    await expect(p,400,'GET','/store?locations=doc.*&limit=1&limit=2',
      null,grant(token));

    // The response must arrive based only on headers. We deliberately never
    // transmit the advertised megabyte of unauthenticated body data.
    await new Promise((resolve,reject)=>{
      const req=http.request({hostname:'127.0.0.1',port:p,method:'POST',
        path:'/platform/store/doc.unauthorized',
        headers:{'Content-Length':1024*1024}},res=>{
        assert.equal(res.statusCode,401);
        req.destroy();resolve();
      });
      req.on('error',error=>{if(error.code!=='ECONNRESET')reject(error);});
      req.setTimeout(1800,()=>{req.destroy();reject(new Error('late rejection'));});
      req.flushHeaders();
    });
    assert.equal((await expect(p,404,'GET','/store/doc.unauthorized')).status,404);

    // A failed scan must not leave a partial cache that understates quotas.
    const scanDir=tmp();
    try {
      const shared=createShared(scanDir);
      shared.ensureStoreRoots();
      fs.writeFileSync(path.join(shared.STORE_DIR,'good.one'),'one');
      fs.writeFileSync(path.join(shared.STORE_DIR,'invalid-name'),'bad');
      fs.writeFileSync(path.join(shared.STORE_DIR,'good.two'),'two');
      const index=createResourceIndex(shared,validKey,matches);
      assert.throws(()=>index.stats('*'),/invalid stored object key/);
      assert.throws(()=>index.stats('*'),/invalid stored object key/);
      fs.unlinkSync(path.join(shared.STORE_DIR,'invalid-name'));
      assert.deepEqual(index.stats('*'),{bytes:6,objects:2});
      assert.deepEqual(index.listKeys('good.*'),{keys:['good.one','good.two']});
    } finally {fs.rmSync(scanDir,{recursive:true,force:true});}

    const cors=await expect(p,204,'OPTIONS','/store/doc.alpha',null,{
      Origin:'https://frontend.example.org',
      'Access-Control-Request-Method':'GET',
      'Access-Control-Request-Headers':'if-none-match,if-range,range'
    });
    for(const name of ['If-None-Match','If-Range','Range']) {
      assert(cors.headers['access-control-allow-headers'].includes(name));
    }

    await expect(p,201,'POST','/store/conditions','first',grant(token));
    const firstTag=(await expect(p,200,'GET','/store/conditions')).etag;
    await expect(p,200,'PUT','/store/conditions','second',{
      ...grant(token),'If-Match':'"unknown", '+firstTag
    });
    const secondTag=(await expect(p,200,'GET','/store/conditions')).etag;
    await expect(p,412,'PUT','/store/conditions','weak',{
      ...grant(token),'If-Match':'W/'+secondTag
    });
    await expect(p,200,'PUT','/store/conditions','third',{
      ...grant(token),'If-Match':'*'
    });
    await expect(p,412,'DELETE','/store/conditions',null,{
      ...grant(token),'If-Match':'"another", *'
    });
    await expect(p,200,'DELETE','/store/conditions',null,{
      ...grant(token),'If-Match':'*'
    });
    await expect(p,404,'PUT','/store/conditions','cannot reappear',{
      ...grant(token),'If-Match':'*'
    });

    // Cached per-matcher statistics are bounded even after many scopes.
    const boundedIndex=createResourceIndex(createShared(dir),validKey,matches);
    for(let i=0;i<1500;i++)boundedIndex.stats('unused.'+i+'*');
    assert(boundedIndex.statsCacheSize()<=1024);
    boundedIndex.clearStats();
    assert.equal(boundedIndex.statsCacheSize(),0);

    console.log('conditional GET, HEAD, Range, pagination and fail-closed loading: OK');
  } finally {
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
