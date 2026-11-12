#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const crypto=require('node:crypto');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
const {open,close,expect,request,auth,grant,tmp}=require('./test-grants.js');

const sha256=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const hashHeader=digest=>({'X-Skrynia-If-SHA256':digest});
async function run() {
  const dir=tmp();let server;
  try {
    const started=await open(dir);server=started.server;
    const port=started.port,root=server.skrynia.grants.root();
    const token=(await expect(port,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    const key='bytes.bin',url='/store/'+key,file=path.join(dir,'store',key);
    const a=Buffer.from([0,255,16,13,0,127]),b=Buffer.from([0,255,16,13,0,126]);
    const ah=sha256(a),bh=sha256(b);
    await expect(port,201,'POST',url,a,grant(token));
    assert.equal(sha256(fs.readFileSync(file)),ah);
    assert(Math.abs(fs.statSync(file).mtimeMs-Date.now())<10000);
    const first=await expect(port,200,'GET',url);
    assert.deepEqual(first.bytes,a);
    assert.equal(first.etag,'"'+ah+'"');
    assert.equal((await expect(port,200,'HEAD',url)).etag,'"'+ah+'"');
    await expect(port,304,'GET',url,null,{'If-None-Match':'"'+ah+'"'});
    await expect(port,304,'GET',url,null,{'If-None-Match':'W/"'+ah+'"'});
    await expect(port,206,'GET',url,null,{
      Range:'bytes=1-3','If-Range':'"'+ah+'"'
    });
    assert.deepEqual((await expect(port,200,'GET',url,null,{
      Range:'bytes=1-3','If-Range':'"'+bh+'"'
    })).bytes,a);

    // A token read directly from the public file is sufficient for CAS.
    await expect(port,412,'PUT',url,b,{
      ...grant(token),...hashHeader(bh)
    }).then(r=>assert.equal(r.json.error,'content_mismatch'));
    assert.deepEqual(fs.readFileSync(file),a);
    await expect(port,200,'PUT',url,b,{
      ...grant(token),...hashHeader(sha256(fs.readFileSync(file)))
    });
    assert.equal((await expect(port,200,'GET',url)).etag,'"'+bh+'"');
    await expect(port,200,'PUT',url,b,{
      ...grant(token),...hashHeader(bh)
    });
    assert.equal((await expect(port,200,'GET',url)).etag,'"'+bh+'"');
    await expect(port,200,'PUT',url,a,{...grant(token),...hashHeader(bh)});
    assert.equal((await expect(port,200,'GET',url)).etag,'"'+ah+'"');
    // Equal bytes are indistinguishable, including A -> B -> A.
    await expect(port,200,'PUT',url,b,{...grant(token),...hashHeader(ah)});
    await expect(port,412,'DELETE',url,null,{
      ...grant(token),...hashHeader(ah)
    }).then(r=>assert.equal(r.json.error,'content_mismatch'));

    for(const invalid of ['','A'.repeat(64),'f'.repeat(63),'f'.repeat(65),
      'g'.repeat(64),' '.repeat(64),'f'.repeat(64)+', '+ah]) {
      const r=await expect(port,400,'PUT',url,a,{
        ...grant(token),...hashHeader(invalid)
      });
      assert.equal(r.json.error,'invalid_precondition');
      const d=await expect(port,400,'DELETE',url,null,{
        ...grant(token),...hashHeader(invalid)
      });
      assert.equal(d.json.error,'invalid_precondition');
    }
    for(const name of ['If-Match','If-None-Match',
      'If-Unmodified-Since','If-Modified-Since','If-Range']) {
      for(const method of ['POST','PUT','DELETE']) {
        const r=await expect(port,400,method,url,method==='DELETE'?null:'x',
          {...grant(token),[name]:'*'});
        assert.equal(r.json.error,'invalid_precondition');
      }
    }
    await expect(port,400,'POST','/store/other',a,{
      ...grant(token),...hashHeader(ah)
    });
    assert.deepEqual(fs.readFileSync(file),b);

    // Recovery sends a wake-up only when the recorded bytes are present.
    const journalDir=path.join(dir,'state','store-transactions');
    const makeIntent=(id,kind,digest)=>fs.writeFileSync(path.join(journalDir,
      id.repeat(32)+'.json'),JSON.stringify({kind,key,sha256:digest}));
    let wakes=0;
    const enqueue=server.skrynia.push.enqueue;
    server.skrynia.push.enqueue=()=>{wakes++;return [];};
    try {
      makeIntent('a','create',bh);
      makeIntent('b','replace',bh);
      makeIntent('c','replace',ah);
      server.skrynia.storeJournal.recover();
      assert.equal(wakes,2);
      assert.deepEqual(fs.readdirSync(journalDir),[]);
    } finally {server.skrynia.push.enqueue=enqueue;}

    const cors=await expect(port,204,'OPTIONS',url,null,{
      Origin:'https://app.example.org',
      'Access-Control-Request-Method':'PUT',
      'Access-Control-Request-Headers':'x-skrynia-if-sha256,authorization'
    });
    assert(cors.headers['access-control-allow-headers'].includes('X-Skrynia-If-SHA256'));
    assert.equal(cors.headers['access-control-expose-headers'],'ETag');

    // The distributed browser helper hashes *received bytes* using WebCrypto.
    class XHR {
      constructor() {this.headers={};}
      open(method,pathname) {this.method=method;this.pathname=pathname;}
      setRequestHeader(name,value) {this.headers[name]=value;}
      send(body) {
        request(port,this.method,this.pathname,body,this.headers).then(r=>{
          this.status=r.status;
          this.response=r.bytes.buffer.slice(r.bytes.byteOffset,
            r.bytes.byteOffset+r.bytes.byteLength);
          this.onload();
        },e=>{this.onerror(e);});
      }
    }
    const window={};
    vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../src/client.js'),'utf8'),
      {window,crypto:crypto.webcrypto,XMLHttpRequest:XHR,TextDecoder,URL,
        URLSearchParams,Uint8Array,ArrayBuffer});
    const store=window.Skrynia.store(token);
    const got=await store.get(key);
    assert.equal(got.meta.sha256,bh);
    assert.deepEqual(Buffer.from(got.bytes),b);
    await store.put(key,a,{sha256:got.meta.sha256});
    const updated=await store.get(key);
    assert.equal(updated.meta.sha256,ah);
    await store.delete(key,{sha256:updated.meta.sha256});
    assert.equal(await store.get(key),null);
    console.log('byte SHA-256 CAS, external-file hash, journal, headers, browser: OK');
  }finally {
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
