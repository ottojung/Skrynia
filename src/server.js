#!/usr/bin/env node
'use strict';

// One object store, one delegable authority tree, one owning server process.
const http=require('http');
const {pipeline}=require('node:stream');
const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const {normalizeBasePath}=require('./base-path.js');
const {createShared,ensureDir}=require('./shared.js');
const {acquireWriterLock}=require('./writer-lock.js');
const {createGrants}=require('./grants.js');
const {createPush}=require('./push.js');
const {createStoreJournal}=require('./store-journal.js');
const {sha256Bytes,sha256Fd,sha256File}=require('./content-hash.js');

const DEFAULT_BUILD_INFO=Object.freeze({version:'development',commit:'development'});
function loadBuildInfo(dir) {
  const root=dir||path.join(__dirname,'..');
  const ver=path.join(root,'VERSION'),commit=path.join(root,'COMMIT');
  if(!fs.existsSync(ver)&&!fs.existsSync(commit)) return DEFAULT_BUILD_INFO;
  if(!fs.existsSync(ver)||!fs.existsSync(commit)) throw new Error('incomplete build info');
  const version=fs.readFileSync(ver,'utf8').trim();
  const sha=fs.readFileSync(commit,'utf8').trim();
  if(!version||!(version==='development'&&sha==='development')&&
    !/^[0-9a-f]{40,64}$/i.test(sha)) throw new Error('invalid build info');
  return {version,commit:sha};
}
function mimeType(key) {
  const extension=path.extname(key).toLowerCase();
  const types={'.html':'text/html; charset=utf-8',
    '.js':'text/javascript; charset=utf-8','.mjs':'text/javascript; charset=utf-8',
    '.css':'text/css; charset=utf-8','.json':'application/json',
    '.webmanifest':'application/manifest+json','.txt':'text/plain; charset=utf-8',
    '.svg':'image/svg+xml','.png':'image/png','.jpg':'image/jpeg',
    '.jpeg':'image/jpeg','.gif':'image/gif','.webp':'image/webp',
    '.ico':'image/x-icon','.wasm':'application/wasm','.pdf':'application/pdf'};
  return types[extension]||'application/octet-stream';
}
function createServer(opts={}) {
  const shared=createShared(opts.dataDir);
  const releaseWriter=acquireWriterLock(shared.STATE_DIR);
  try {
  const grants=createGrants(shared);
  const buildInfo=opts.buildInfo||loadBuildInfo(opts.buildInfoDir);
  const external=opts.skryniaUrl??process.env.SKRYNIA_URL;
  if(!external) throw new Error('SKRYNIA_URL is required');
  let publicUrl;
  try {publicUrl=new URL(external);}
  catch {throw new Error('SKRYNIA_URL must be an absolute URL');}
  if(!['http:','https:'].includes(publicUrl.protocol)||publicUrl.search||publicUrl.hash) {
    throw new Error('invalid SKRYNIA_URL');
  }
  const base=normalizeBasePath(publicUrl.pathname||'/');
  const maxKeyLength=Number(process.env.SKRYNIA_MAX_KEY_LENGTH||256);
  const maxObjectSize=Number(process.env.SKRYNIA_MAX_OBJECT_SIZE||10485760);
  const maxBufferedBytes=Number(opts.maxBufferedUploadBytes??
    process.env.SKRYNIA_MAX_BUFFERED_UPLOAD_BYTES??41943040);
  const maxConnections=Number(opts.maxConnections??
    process.env.SKRYNIA_MAX_CONNECTIONS??512);
  const idleTimeoutMs=Number(opts.socketIdleTimeoutMs??
    process.env.SKRYNIA_SOCKET_IDLE_TIMEOUT_MS??30000);
  if(!Number.isSafeInteger(maxKeyLength)||maxKeyLength<1||
    !Number.isSafeInteger(maxObjectSize)||maxObjectSize<1||
    !Number.isSafeInteger(maxBufferedBytes)||maxBufferedBytes<1||
    !Number.isSafeInteger(maxConnections)||maxConnections<1||
    !Number.isSafeInteger(idleTimeoutMs)||idleTimeoutMs<1) {
    throw new Error('invalid storage limits');
  }
  let bufferedBytes=0;
  const push=createPush({dataDir:shared.dataDir,skryniaUrl:external,
    pushTransport:opts.pushTransport,pushPollMs:opts.pushPollMs,
    pushSubject:opts.pushSubject,pushManual:opts.pushManual,
    pushSendTimeoutMs:opts.pushSendTimeoutMs,
    maxSubscriptions:opts.pushMaxSubscriptions,maxOutboxItems:opts.pushMaxOutboxItems,
    pushConcurrency:opts.pushConcurrency,
    pushAllowedHosts:opts.pushAllowedHosts,
    pushMaxAttempts:opts.pushMaxAttempts,
    onSubscriptionCreate:(secret,locations)=>grants.accountSubscription(secret,locations),
    onSubscriptionMutation:()=>grants.subscriptionChanged()});
  grants.setOnSubscriptionsRevoked(()=>push.invalidateSubscriptions());
  const journal=createStoreJournal(shared,push,grants);
  grants.recover();
  shared.ensureStoreRoots();
  ensureDir(shared.STATE_DIR,0o700);
  journal.recover();
  grants.ensureRoot();
  push.start();

  function json(res,status,value) {
    res.writeHead(status,{'Content-Type':'application/json','Cache-Control':'no-store'});
    res.end(JSON.stringify(value));
  }
  function error(res,e) {
    if(e.status) return json(res,e.status,{error:e.code});
    if(e.code==='outbox_full'||e.code==='subscriptions_full')
      return json(res,507,{error:e.code});
    if(e.code==='invalid_subscription')
      return json(res,400,{error:e.code});
    if(['capability_required','invalid_capability'].includes(e.code))
      return json(res,403,{error:e.code});
    if(e.code==='endpoint_in_use') return json(res,409,{error:e.code});
    if(e.code==='not_found') return json(res,404,{error:e.code});
    console.error('Skrynia request failed:',e);
    return json(res,500,{error:'internal_error'});
  }
  function validKey(key) {
    return key.length<=maxKeyLength&&grants.validKey(key);
  }
  function tempObject(body) {
    const file=path.join(shared.STORE_TMP_DIR,
      'obj.'+process.pid+'.'+crypto.randomBytes(12).toString('hex'));
    fs.writeFileSync(file,body,{mode:0o600});
    fs.chmodSync(file,0o644);
    const fd=fs.openSync(file,'r');
    try {fs.fsyncSync(fd);}finally{fs.closeSync(fd);}
    journal.syncDir(shared.STORE_TMP_DIR);
    return file;
  }
  function removeTemp(file) {
    try{fs.unlinkSync(file);}catch(e){if(e.code!=='ENOENT')throw e;}
  }
  function get(key,req,res) {
    const file=shared.objPath(key);
    let fd=null;
    try {
      // Open once: validators and body must describe the same inode.
      fd=fs.openSync(file,'r');
      const stat=fs.fstatSync(fd),tag='"'+sha256Fd(fd)+'"';
      const headers={
        'Content-Type':mimeType(key),
        'ETag':tag,
        'Cache-Control':'no-cache',
        'Accept-Ranges':'bytes',
        'X-Content-Type-Options':'nosniff'
      };
      const validators=(req.headers['if-none-match']||'').split(',').map(x=>x.trim());
      if(validators.some(x=>x==='*'||x===tag||x==='W/'+tag)) {
        res.writeHead(304,headers);
        return res.end();
      }
      let start=0,end=stat.size-1,status=200;
      const range=req.headers.range;
      // If-Range must match the current strong representation tag.
      if(range&&(!req.headers['if-range']||req.headers['if-range']===tag)) {
        const match=/^bytes=(\d*)-(\d*)$/.exec(range);
        if(match&&(match[1]||match[2])&&stat.size>0) {
          if(!match[1]) {
            const suffix=Number(match[2]);
            if(Number.isSafeInteger(suffix)&&suffix>0) {
              start=Math.max(0,stat.size-suffix);
            } else status=416;
          } else {
            start=Number(match[1]);
            end=match[2]?Number(match[2]):end;
            if(!Number.isSafeInteger(start)||!Number.isSafeInteger(end)||
              end<start||start>=stat.size) status=416;
            end=Math.min(end,stat.size-1);
          }
          if(status!==416) status=206;
        } else status=416;
      }
      if(status===416) {
        res.writeHead(416,{...headers,'Content-Range':'bytes */'+stat.size});
        return res.end();
      }
      if(status===206) {
        headers['Content-Range']='bytes '+start+'-'+end+'/'+stat.size;
      }
      headers['Content-Length']=status===206?end-start+1:stat.size;
      res.writeHead(status,headers);
      if(req.method==='HEAD'||headers['Content-Length']===0) return res.end();

      // pipeline destroys the source on disconnect, closing its descriptor
      // even when an anonymous client aborts a large download.
      const source=fs.createReadStream(null,{fd,autoClose:true,start,end});
      fd=null; // ownership is transferred to the stream
      pipeline(source,res,e=>{
        if(e&&!['ERR_STREAM_PREMATURE_CLOSE','ECONNRESET'].includes(e.code)) {
          console.error('Skrynia download stream failed:',e);
        }
      });
    } catch(e) {
      if(e.code==='ENOENT') return json(res,404,{error:'not_found'});
      throw e;
    } finally {
      if(fd!==null) fs.closeSync(fd);
    }
  }
  function validMutationPrecondition(req,res,method) {
    for(const name of [
      'if-match','if-none-match','if-modified-since',
      'if-unmodified-since','if-range'
    ]) {
      if(req.headers[name]!==undefined) {
        json(res,400,{error:'invalid_precondition'});
        return false;
      }
    }
    const digest=req.headers['x-skrynia-if-sha256'];
    if(digest!==undefined&&
      (method==='POST'||typeof digest!=='string'||!/^[a-f0-9]{64}$/.test(digest))) {
      json(res,400,{error:'invalid_precondition'});
      return false;
    }
    return true;
  }
  function contentMatches(file,req,res) {
    const expected=req.headers['x-skrynia-if-sha256'];
    if(expected===undefined) return true;
    if(sha256File(file)===expected) return true;
    json(res,412,{error:'content_mismatch'});
    return false;
  }
  function create(key,body,req,res) {
    const file=shared.objPath(key);
    if(fs.existsSync(file)) return json(res,409,{error:'already_exists'});
    grants.authorizeMutation(bearer(req),key,'create',body.length);
    const digest=sha256Bytes(body),tmp=tempObject(body);
    try {
      const outcome=journal.commit(key,'create',digest,()=>{
        fs.linkSync(tmp,file);grants.recordObject(key,body.length);
      });
      return json(res,201,{ok:true,...outcome});
    } catch(e) {
      if(e.code==='EEXIST') return json(res,409,{error:'already_exists'});
      throw e;
    } finally {removeTemp(tmp);}
  }
  function replace(key,body,req,res) {
    const file=shared.objPath(key);
    if(!fs.existsSync(file)) return json(res,404,{error:'not_found'});
    if(!contentMatches(file,req,res)) return;
    grants.authorizeMutation(bearer(req),key,'replace',body.length);
    const digest=sha256Bytes(body),tmp=tempObject(body);
    try {
      const outcome=journal.commit(key,'replace',digest,()=>{
        fs.renameSync(tmp,file);grants.recordObject(key,body.length);
      });
      return json(res,200,{ok:true,...outcome});
    } finally {removeTemp(tmp);}
  }
  function deleteObject(key,req,res) {
    const file=shared.objPath(key);
    if(!fs.existsSync(file)) return json(res,404,{error:'not_found'});
    grants.authorizeMutation(bearer(req),key,'delete');
    if(!contentMatches(file,req,res)) return;
    const outcome=journal.commit(key,'delete',null,()=>{
      fs.unlinkSync(file);grants.recordObject(key,null);
    });
    return json(res,200,{ok:true,...outcome});
  }
  function body(req,res,limit,callback) {
    // The cap covers actual resident request chunks across connections.
    // Failed, interrupted and completed uploads release their reservation.
    const length=req.headers['content-length'];
    if(length!==undefined&&Number(length)>limit) {
      res.setHeader('Connection','close');
      return json(res,413,{error:'request_too_large'});
    }
    if(length!==undefined&&Number(length)>maxBufferedBytes) {
      res.setHeader('Connection','close');
      return json(res,503,{error:'upload_memory_full'});
    }
    let count=0,held=0,finished=false;
    const chunks=[];
    function release() {
      bufferedBytes-=held;
      held=0;
    }
    function stop(status,reason) {
      if(finished) return;
      finished=true;
      release();
      res.setHeader('Connection','close');
      json(res,status,{error:reason});
    }
    req.on('data',chunk=>{
      if(finished) return;
      if(count+chunk.length>limit)
        return stop(413,'request_too_large');
      if(bufferedBytes+chunk.length>maxBufferedBytes)
        return stop(503,'upload_memory_full');
      count+=chunk.length;
      held+=chunk.length;
      bufferedBytes+=chunk.length;
      chunks.push(chunk);
    });
    req.on('end',()=>{
      if(finished) return;
      finished=true;
      const bytes=Buffer.concat(chunks);
      release();
      callback(bytes);
    });
    req.on('error',()=>stop(400,'request_interrupted'));
    req.on('close',release);
  }
  function jsonBody(req,res,callback) {
    return body(req,res,8192,raw=>{
      let value;
      try{value=JSON.parse(raw.toString('utf8'));}
      catch{return json(res,400,{error:'invalid_json'});}
      if(!value||typeof value!=='object'||Array.isArray(value))
        return json(res,400,{error:'invalid_json_object'});
      callback(value);
    });
  }
  function bearer(req) {
    const found=/^Bearer ([a-f0-9]{64})$/.exec(req.headers.authorization||'');
    return found?found[1]:null;
  }
  function capability(req) {
    return typeof req.headers['x-skrynia-capability']==='string'?
      req.headers['x-skrynia-capability']:null;
  }
  function routePath(pathname) {
    if(base==='/') return pathname;
    if(pathname===base) return '/';
    return pathname.startsWith(base+'/')?pathname.slice(base.length):null;
  }
  function route(req,res) {
    // No cookies or ambient credentials: every mutation requires a bearer secret.
    res.setHeader('Access-Control-Allow-Origin','*');
    res.setHeader('Access-Control-Expose-Headers','ETag');
    if(req.method==='OPTIONS') {
      res.writeHead(204,{'Access-Control-Allow-Methods':'GET, POST, PUT, DELETE, OPTIONS',
        'Access-Control-Allow-Headers':'Authorization, Content-Type, X-Skrynia-Capability, X-Skrynia-If-SHA256, If-None-Match, If-Range, Range',
        'Access-Control-Max-Age':'3600'});
      return res.end();
    }
    let url;
    try {url=new URL(req.url,'http://127.0.0.1');}
    catch {return json(res,400,{error:'invalid_url'});}
    const p=routePath(url.pathname);
    if(p==='/health'&&req.method==='GET') return json(res,200,{ok:true});
    if(p==='/version'&&req.method==='GET') return json(res,200,buildInfo);
    if(p==='/client/skrynia.js'&&req.method==='GET') {
      const file=path.join(__dirname,'client.js');
      res.writeHead(200,{'Content-Type':'text/javascript','Cache-Control':'no-cache'});
      return res.end(fs.readFileSync(file));
    }
    if(p==='/push/vapid'&&req.method==='GET')
      return json(res,200,{publicKey:push.getPublicKey()});
    if(p==='/push/subscriptions'&&req.method==='POST')
      return jsonBody(req,res,value=>{
        try {
          const result=push.createSub(value.locations,
            value.events??['create','replace','delete'],
            value.endpoint,value.keys,capability(req),bearer(req));
          json(res,201,{ok:true,...result});
        }catch(e){error(res,e);}
      });
    const sub=p?.match(/^\/push\/subscriptions\/([a-f0-9]{32})$/);
    if(sub) {
      if(req.method==='DELETE') {
        try{return json(res,200,push.removeSub(sub[1],capability(req)));}
        catch(e){return error(res,e);}
      }
      if(req.method==='PUT') return jsonBody(req,res,value=>{
        if(value.endpoint===undefined&&value.keys===undefined)
          return json(res,400,{error:'nothing_to_update'});
        if(Object.keys(value).some(k=>!['endpoint','keys'].includes(k)))
          return json(res,400,{error:'invalid_subscription_patch'});
        try{return json(res,200,push.updateSub(sub[1],capability(req),value));}
        catch(e){return error(res,e);}
      });
      return json(res,405,{error:'method_not_allowed'});
    }
    if(p==='/grants'&&req.method==='POST') return jsonBody(req,res,value=>{
      try{return json(res,200,grants.createChild(bearer(req),value.maxBytes,
        value.maxObjects,value.locations,value.operations));}
      catch(e){return error(res,e);}
    });
    const grant=p?.match(/^\/grants\/([a-f0-9]{32})(?:\/(users))?$/);
    if(grant) {
      const id=grant[1],action=grant[2];
      if(req.method==='GET'&&!action) {
        try{return json(res,200,grants.inspect(id,bearer(req)));}
        catch(e){return error(res,e);}
      }
      if(req.method==='POST'&&action==='users') {
        if(Number(req.headers['content-length']||0)>0||req.headers['transfer-encoding'])
          return json(res,400,{error:'unexpected_body'});
        try{return json(res,200,grants.createUser(id,bearer(req)));}
        catch(e){return error(res,e);}
      }
      if(req.method==='DELETE'&&!action) {
        try{return json(res,200,grants.revokeGrant(id,bearer(req)));}
        catch(e){return error(res,e);}
      }
      return json(res,405,{error:'method_not_allowed'});
    }
    const user=p?.match(/^\/grants\/([a-f0-9]{32})\/users\/([a-f0-9]{32})$/);
    if(user) {
      if(req.method!=='DELETE') return json(res,405,{error:'method_not_allowed'});
      try{return json(res,200,grants.revokeUser(user[1],bearer(req),user[2]));}
      catch(e){return error(res,e);}
    }
    if(p==='/store'&&req.method==='GET') {
      const params=url.searchParams;
      const patterns=params.getAll('locations');
      if(patterns.length!==1||
        [...params.keys()].some(k=>!['locations','limit','after'].includes(k))||
        [...params.keys()].some(k=>params.getAll(k).length!==1)) {
        return json(res,400,{error:'invalid_locations'});
      }
      const limit=params.has('limit')?Number(params.get('limit')):1000;
      const after=params.has('after')?params.get('after'):null;
      try {return json(res,200,grants.listObjects(bearer(req),patterns[0],
        {limit,after}));}
      catch(e){return error(res,e);}
    }
    if(p==='/store') return json(res,405,{error:'method_not_allowed'});
    const keyMatch=p?.match(/^\/store\/(.+)$/);
    if(keyMatch) {
      let key;
      try {key=decodeURIComponent(keyMatch[1]);}
      catch{return json(res,400,{error:'invalid_percent_encoding'});}
      if(!validKey(key)) return json(res,400,{error:'invalid_key'});
      if(req.method==='GET'||req.method==='HEAD') return get(key,req,res);
      if(req.method==='DELETE') {
        if(!validMutationPrecondition(req,res,'DELETE')) return;
        try{return deleteObject(key,req,res);}
        catch(e){return error(res,e);}
      }
      if(req.method==='POST'||req.method==='PUT') {
        if(!validMutationPrecondition(req,res,req.method)) return;
        // Reject anonymous and unauthorized uploads before buffering bytes.
        try {
          grants.requireMutation(bearer(req),key,
            req.method==='POST'?'create':'replace');
        } catch(e) {return error(res,e);}
        return body(req,res,maxObjectSize,bytes=>{
          try {
            if(req.method==='POST') return create(key,bytes,req,res);
            return replace(key,bytes,req,res);
          } catch(e){return error(res,e);}
        });
      }
      return json(res,405,{error:'method_not_allowed'});
    }
    return json(res,404,{error:'not_found'});
  }
  const recoveryTimer=setInterval(()=>{
    try{grants.recoverRevocations();journal.recover();}
    catch(e){console.error('Skrynia recovery pending:',e);}
  },60000);
  recoveryTimer.unref();
  const server=http.createServer((req,res)=>{
    // A public GET can fail during open/stat or stream setup. No unhandled
    // synchronous exception may terminate the entire storage process.
    try {route(req,res);}
    catch(e) {
      console.error('Skrynia unhandled request error:',e);
      if(res.headersSent) res.destroy(e);
      else {
        try {error(res,e);}
        catch(replyError) {res.destroy(replyError);}
      }
    }
  });
  server.maxConnections=maxConnections;
  server.requestTimeout=60000;
  server.headersTimeout=15000;
  // requestTimeout only bounds receiving request data. An inactive
  // *response* socket must also be closed to avoid exhausting connections.
  server.setTimeout(idleTimeoutMs,socket=>socket.destroy());
  server.skrynia={HTTP_BASE_PATH:base,SKRYNIA_URL:external,DATA_DIR:shared.dataDir,
    grants,push,storeJournal:journal,buildInfo,
    bufferedUploadBytes:()=>bufferedBytes};
  server.on('close',()=>{
    clearInterval(recoveryTimer);
    try {push.close(releaseWriter);}
    catch(e) {releaseWriter();throw e;}
  });
  return server;
  } catch(e) {
    releaseWriter();
    throw e;
  }
}
if(require.main===module) {
  const port=Number(process.env.SKRYNIA_PORT||17380);
  const host=process.env.SKRYNIA_HOST||'127.0.0.1';
  createServer().listen(port,host,()=>console.log('Skrynia listening on '+host+':'+port));
}
module.exports={createServer};
