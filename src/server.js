#!/usr/bin/env node
'use strict';

// One object store, one delegable authority tree, one owning server process.
const http=require('http');
const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const {normalizeBasePath}=require('./base-path.js');
const {createShared,ensureDir}=require('./shared.js');
const {createGrants}=require('./grants.js');
const {createPush}=require('./push.js');
const {createStoreJournal}=require('./store-journal.js');

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
  if(!Number.isSafeInteger(maxKeyLength)||maxKeyLength<1||
    !Number.isSafeInteger(maxObjectSize)||maxObjectSize<1) {
    throw new Error('invalid storage limits');
  }
  const push=createPush({dataDir:shared.dataDir,skryniaUrl:external,
    pushTransport:opts.pushTransport,pushPollMs:opts.pushPollMs,
    pushSubject:opts.pushSubject,pushManual:opts.pushManual,
    pushSendTimeoutMs:opts.pushSendTimeoutMs,
    maxSubscriptions:opts.pushMaxSubscriptions,maxOutboxItems:opts.pushMaxOutboxItems,
    onSubscriptionCreate:(secret,locations)=>grants.accountSubscription(secret,locations),
    onSubscriptionMutation:()=>grants.subscriptionChanged()});
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
  function etag(file) {
    const st=fs.statSync(file);
    return '"'+Math.floor(st.mtimeMs/1000).toString(16)+'-'+st.size.toString(16)+'"';
  }
  function setVersion(file,version) {
    const time=new Date(version*1000);
    fs.utimesSync(file,time,time);
    if(Math.floor(fs.statSync(file).mtimeMs/1000)!==version) {
      throw new Error('filesystem did not preserve object version');
    }
  }
  function tempObject(body,version) {
    const file=path.join(shared.STORE_TMP_DIR,
      'obj.'+process.pid+'.'+crypto.randomBytes(12).toString('hex'));
    fs.writeFileSync(file,body,{mode:0o600});
    setVersion(file,version);
    fs.chmodSync(file,0o644);
    const fd=fs.openSync(file,'r');
    try {fs.fsyncSync(fd);}finally{fs.closeSync(fd);}
    journal.syncDir(shared.STORE_TMP_DIR);
    return file;
  }
  function removeTemp(file) {
    try{fs.unlinkSync(file);}catch(e){if(e.code!=='ENOENT')throw e;}
  }
  function get(key,res) {
    const file=shared.objPath(key);
    if(!fs.existsSync(file)) return json(res,404,{error:'not_found'});
    const data=fs.readFileSync(file);
    res.writeHead(200,{'Content-Type':mimeType(key),'Content-Length':data.length,
      'ETag':etag(file),'Cache-Control':'no-cache',
      'X-Content-Type-Options':'nosniff'});
    res.end(data);
  }
  function create(key,body,req,res) {
    const file=shared.objPath(key);
    if(fs.existsSync(file)) return json(res,409,{error:'already_exists'});
    grants.authorizeMutation(req.headers['x-skrynia-grant'],key,'create',body.length);
    const version=shared.nextVersion(),tmp=tempObject(body,version);
    try {
      const outcome=journal.commit(key,'create',version,()=>{
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
    if(req.headers['if-match']!==undefined&&req.headers['if-match']!==etag(file))
      return json(res,412,{error:'etag_mismatch'});
    grants.authorizeMutation(req.headers['x-skrynia-grant'],key,'replace',body.length);
    const version=shared.nextVersion(),tmp=tempObject(body,version);
    try {
      const outcome=journal.commit(key,'replace',version,()=>{
        fs.renameSync(tmp,file);grants.recordObject(key,body.length);
      });
      return json(res,200,{ok:true,...outcome});
    } finally {removeTemp(tmp);}
  }
  function deleteObject(key,req,res) {
    const file=shared.objPath(key);
    if(!fs.existsSync(file)) return json(res,404,{error:'not_found'});
    grants.authorizeMutation(req.headers['x-skrynia-grant'],key,'delete');
    if(req.headers['if-match']!==undefined&&req.headers['if-match']!==etag(file))
      return json(res,412,{error:'etag_mismatch'});
    const version=shared.nextVersion();
    const outcome=journal.commit(key,'delete',version,()=>{
      fs.unlinkSync(file);grants.recordObject(key,null);
    });
    return json(res,200,{ok:true,...outcome});
  }
  function body(req,res,limit,callback) {
    let count=0,finished=false;
    const chunks=[];
    req.on('data',chunk=>{
      if(finished) return;
      count+=chunk.length;
      if(count>limit) {finished=true;json(res,413,{error:'request_too_large'});return;}
      chunks.push(chunk);
    });
    req.on('end',()=>{if(!finished) callback(Buffer.concat(chunks));});
    req.on('error',()=>{if(!finished) {finished=true;json(res,400,{error:'request_interrupted'});}});
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
        'Access-Control-Allow-Headers':'Authorization, Content-Type, X-Skrynia-Grant, X-Skrynia-Capability, If-Match',
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
            value.endpoint,value.keys,capability(req),req.headers['x-skrynia-grant']);
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
      const patterns=url.searchParams.getAll('locations');
      if(patterns.length!==1||url.searchParams.size!==1)
        return json(res,400,{error:'invalid_locations'});
      try {return json(res,200,grants.listObjects(req.headers['x-skrynia-grant'],patterns[0]));}
      catch(e){return error(res,e);}
    }
    if(p==='/store') return json(res,405,{error:'method_not_allowed'});
    const keyMatch=p?.match(/^\/store\/(.+)$/);
    if(keyMatch) {
      let key;
      try {key=decodeURIComponent(keyMatch[1]);}
      catch{return json(res,400,{error:'invalid_percent_encoding'});}
      if(!validKey(key)) return json(res,400,{error:'invalid_key'});
      if(req.method==='GET') return get(key,res);
      if(req.method==='DELETE') {
        try{return deleteObject(key,req,res);}
        catch(e){return error(res,e);}
      }
      if(req.method==='POST'||req.method==='PUT') return body(req,res,maxObjectSize,
        bytes=>{
          try {
            if(req.method==='POST') return create(key,bytes,req,res);
            return replace(key,bytes,req,res);
          } catch(e){return error(res,e);}
        });
      return json(res,405,{error:'method_not_allowed'});
    }
    return json(res,404,{error:'not_found'});
  }
  const recoveryTimer=setInterval(()=>{
    try{grants.recoverRevocations();journal.recover();}
    catch(e){console.error('Skrynia recovery pending:',e);}
  },60000);
  recoveryTimer.unref();
  const server=http.createServer(route);
  server.skrynia={HTTP_BASE_PATH:base,SKRYNIA_URL:external,DATA_DIR:shared.dataDir,
    grants,push,storeJournal:journal,buildInfo};
  server.on('close',()=>{clearInterval(recoveryTimer);push.close();});
  return server;
}
if(require.main===module) {
  const port=Number(process.env.SKRYNIA_PORT||17380);
  createServer().listen(port,'127.0.0.1',()=>console.log('Skrynia listening on 127.0.0.1:'+port));
}
module.exports={createServer};
