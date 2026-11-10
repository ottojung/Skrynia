'use strict';

// One global authority tree. Objects never carry grant ownership.
const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const {ensureDir,syncDir}=require('./shared.js');
const {createResourceIndex}=require('./resource-index.js');

const GRANT_COST=2048,OBJECT_COST=512,PUSH_SUB_COST=4096,EXTRA_USER_COST=128;
const ID_RE=/^[a-f0-9]{32}$/;
const KEY_RE=/^[A-Za-z0-9_][A-Za-z0-9_.]*$/;
const LOCATION_RE=/^(?:[A-Za-z0-9_][A-Za-z0-9_.]*\*?|\*)?$/;
const OPERATIONS=Object.freeze(['write','list','delegate','subscribe']);

function validKey(key) {return typeof key==='string'&&KEY_RE.test(key);}
function validLocations(loc) {return typeof loc==='string'&&LOCATION_RE.test(loc);}
function matches(loc,key) {
  return validLocations(loc)&&validKey(key)&&
    (loc.endsWith('*')?key.startsWith(loc.slice(0,-1)):key===loc);
}
function subset(child,parent) {
  if(!validLocations(child)||!validLocations(parent)) return false;
  if(child==='') return true;
  return parent.endsWith('*') ? child.startsWith(parent.slice(0,-1)) :
    (!child.endsWith('*')&&child===parent);
}
function fault(status,code) {
  const e=new Error(code);e.status=status;e.code=code;return e;
}
function randomId() {return crypto.randomBytes(16).toString('hex');}
function token() {return crypto.randomBytes(32).toString('hex');}
function digest(s) {return crypto.createHash('sha256').update(s).digest('hex');}
function same(a,b) {
  return typeof a==='string'&&typeof b==='string'&&
    /^[a-f0-9]{64}$/.test(a)&&/^[a-f0-9]{64}$/.test(b)&&
    crypto.timingSafeEqual(Buffer.from(a,'hex'),Buffer.from(b,'hex'));
}
function normalizeOperations(value) {
  if(!Array.isArray(value)||value.some(x=>!OPERATIONS.includes(x))||
    new Set(value).size!==value.length) throw fault(400,'invalid_operations');
  return OPERATIONS.filter(x=>value.includes(x));
}
function validFiniteLimit(v) {return Number.isSafeInteger(v)&&v>0;}
function within(usage,max) {return max===null||usage<=max;}

function createGrants(shared) {
  const index=createResourceIndex(shared,validKey,matches);
  const dir=path.join(shared.STATE_DIR,'grants');
  const subdir=path.join(shared.STATE_DIR,'push-subs');
  const pendingDir=path.join(shared.STATE_DIR,'grant-revocations');
  let grantsCache=null,subscriptionsCache=null;
  function recordPath(id) {return path.join(dir,id+'.json');}
  function atomicJson(file,record) {
    ensureDir(path.dirname(file),0o700);
    const tmp=file+'.'+randomId()+'.tmp';
    try {
      const fd=fs.openSync(tmp,'wx',0o600);
      try {fs.writeSync(fd,JSON.stringify(record));fs.fsyncSync(fd);}
      finally {fs.closeSync(fd);}
      fs.renameSync(tmp,file);syncDir(path.dirname(file));
    } finally {
      try {fs.unlinkSync(tmp);}catch(e){if(e.code!=='ENOENT')throw e;}
    }
  }
  function grantMap() {
    if(grantsCache) return grantsCache;
    const result=new Map();
    if(fs.existsSync(dir)) for(const filename of fs.readdirSync(dir)) {
      if(!/^[a-f0-9]{32}\.json$/.test(filename)) throw new Error('invalid grant filename');
      const record=JSON.parse(fs.readFileSync(path.join(dir,filename),'utf8'));
      if(record.id+'.json'!==filename) throw new Error('corrupt grant filename');
      result.set(record.id,record);
    }
    grantsCache=result;return result;
  }
  function all() {return [...grantMap().values()];}
  function root() {return all().find(g=>g.parent===null)||null;}
  function writeGrant(g) {
    atomicJson(recordPath(g.id),g);
    grantMap().set(g.id,g);
  }
  function find(id) {
    if(!ID_RE.test(id||'')) throw fault(400,'invalid_grant');
    const g=grantMap().get(id);
    if(!g) throw fault(404,'grant_not_found');
    return g;
  }
  function validateTree() {
    const map=new Map(all().map(g=>[g.id,g]));
    const roots=[];
    for(const g of map.values()) {
      if(!ID_RE.test(g.id)||!validLocations(g.locations)||
        !Array.isArray(g.operations)||g.operations.some(op=>!OPERATIONS.includes(op))||
        !Array.isArray(g.users)||g.users.some(u=>
          !ID_RE.test(u.id)||!/^[a-f0-9]{64}$/.test(u.hash))) {
        throw new Error('invalid grant record');
      }
      if(g.parent===null) {
        roots.push(g);
        if(g.locations!=='*'||g.maxBytes!==null||g.maxObjects!==null||
          g.operations.length!==OPERATIONS.length||
          OPERATIONS.some(op=>!g.operations.includes(op))||
          !/^[a-f0-9]{64}$/.test(g.rootIssuerToken||'')||
          !same(g.issuerHash,digest(g.rootIssuerToken))) {
          throw new Error('invalid root grant');
        }
      } else {
        if(!validFiniteLimit(g.maxBytes)||!validFiniteLimit(g.maxObjects)||
           g.maxBytes<GRANT_COST||g.rootIssuerToken!==undefined) {
          throw new Error('invalid child grant limits');
        }
        const parent=map.get(g.parent);
        if(!parent||!subset(g.locations,parent.locations)||
          !g.operations.every(op=>parent.operations.includes(op))) {
          throw new Error('invalid grant ancestry');
        }
      }
    }
    if(map.size&&roots.length!==1) throw new Error('invalid grant roots');
    for(const g of map.values()) {
      const seen=new Set();let x=g;
      while(x) {
        if(seen.has(x.id)) throw new Error('cyclic grant ancestry');
        seen.add(x.id);
        x=x.parent===null?null:map.get(x.parent);
      }
    }
    return map;
  }
  function isDescendant(map,child,ancestor) {
    let g=map.get(child);
    while(g) {
      if(g.id===ancestor) return true;
      g=g.parent===null?null:map.get(g.parent);
    }
    return false;
  }
  function ancestors(map,id) {
    const result=[];let g=map.get(id);
    if(!g) throw fault(404,'grant_not_found');
    while(g) {result.push(g);g=g.parent===null?null:map.get(g.parent);}
    return result;
  }
  function subscriptionEntries() {
    if(subscriptionsCache) return subscriptionsCache;
    const list=[];
    if(fs.existsSync(subdir)) for(const filename of fs.readdirSync(subdir)) {
      if(!/^[a-f0-9]{32}\.json$/.test(filename)) throw new Error('invalid subscription filename');
      const file=path.join(subdir,filename);
      list.push({file,data:JSON.parse(fs.readFileSync(file,'utf8'))});
    }
    subscriptionsCache=list;return list;
  }
  function subscriptionChanged() {subscriptionsCache=null;}
  function usageWith(g,map,subs) {
    const stats=index.stats(g.locations);
    let bytes=stats.bytes+stats.objects*OBJECT_COST;
    let objects=stats.objects;
    for(const h of map.values()) if(isDescendant(map,h.id,g.id)) {
      bytes+=GRANT_COST+Math.max(0,h.users.length-1)*EXTRA_USER_COST;
    }
    for(const sub of subs) {
      if(!map.has(sub.data.grantId)) throw new Error('orphaned subscription');
      if(isDescendant(map,sub.data.grantId,g.id)) {
        bytes+=PUSH_SUB_COST;objects++;
      }
    }
    return {bytes,objects};
  }
  function usage(id) {
    const map=validateTree(),g=map.get(id);
    if(!g) throw fault(404,'grant_not_found');
    return usageWith(g,map,subscriptionEntries());
  }
  function assertCapacity(id,extraBytes,extraObjects) {
    const map=validateTree(),subs=subscriptionEntries();
    for(const g of ancestors(map,id)) {
      const u=usageWith(g,map,subs);
      if(!within(u.bytes+extraBytes,g.maxBytes)||
        !within(u.objects+extraObjects,g.maxObjects)) {
        throw fault(507,'grant_quota_exceeded');
      }
    }
  }
  function requireUser(secret,id) {
    if(typeof secret!=='string') throw fault(401,'grant_user_token_required');
    const hash=digest(secret);
    const g=all().find(x=>x.users.some(u=>same(hash,u.hash))&&(!id||x.id===id));
    if(!g) throw fault(403,'invalid_grant_user_token');
    return g;
  }
  function requireIssuer(id,secret) {
    const g=find(id);
    if(typeof secret!=='string'||!same(g.issuerHash,digest(secret))) {
      throw fault(403,'invalid_grant_issuer_token');
    }
    return g;
  }
  function requireOperation(g,op) {
    if(!g.operations.includes(op)) throw fault(403,'grant_operation_forbidden');
  }
  function authorizeMutation(secret,key,kind,newBytes) {
    const g=requireUser(secret);
    requireOperation(g,'write');
    if(!matches(g.locations,key)) throw fault(403,'location_not_authorized');
    if(!['create','replace','delete'].includes(kind)) throw fault(400,'invalid_mutation');
    if(kind==='delete') return g.id;
    if(!Number.isSafeInteger(newBytes)||newBytes<0)
      throw fault(400,'invalid_object_size');
    const map=validateTree(),subs=subscriptionEntries();
    const old=index.value(key);
    const delta=newBytes-(old===null?0:old)+(old===null?OBJECT_COST:0);
    for(const ancestor of ancestors(map,g.id)) {
      const u=usageWith(ancestor,map,subs);
      if(!within(u.bytes+delta,ancestor.maxBytes)||
        !within(u.objects+(old===null?1:0),ancestor.maxObjects)) {
        throw fault(507,'grant_quota_exceeded');
      }
    }
    return g.id;
  }
  // Listings are constrained by the calling grant, not by creator identity.
  function listObjects(secret,locations) {
    const g=requireUser(secret);
    requireOperation(g,'list');
    if(!validLocations(locations)) throw fault(400,'invalid_locations');
    if(!subset(locations,g.locations)) throw fault(403,'locations_outside_grant');
    return {keys:index.entries().filter(({key})=>matches(locations,key))
      .map(({key})=>key).sort()};
  }
  function recordObject(key,bytes) {index.replace(key,bytes);}
  function ensureRoot() {
    recoverRevocations();
    validateTree();
    const existing=root();
    if(existing) return {created:false,id:existing.id};
    if(all().length||subscriptionEntries().length) {
      throw new Error('orphaned grant or subscription records');
    }
    const secret=token(),id=randomId();
    const record={id,parent:null,locations:'*',operations:[...OPERATIONS],
      maxBytes:null,maxObjects:null,issuerHash:digest(secret),
      rootIssuerToken:secret,users:[]};
    writeGrant(record);
    return {created:true,id};
  }
  function createChild(secret,maxBytes,maxObjects,locations,operations) {
    const parent=requireUser(secret);
    requireOperation(parent,'delegate');
    if(!validFiniteLimit(maxBytes)||maxBytes<GRANT_COST||
      !validFiniteLimit(maxObjects)) throw fault(400,'invalid_grant_limits');
    const loc=locations===undefined?parent.locations:locations;
    const ops=operations===undefined?parent.operations:normalizeOperations(operations);
    if(!validLocations(loc)) throw fault(400,'invalid_locations');
    if(!subset(loc,parent.locations)) throw fault(403,'locations_outside_parent');
    if(!ops.every(op=>parent.operations.includes(op))) throw fault(403,'operations_outside_parent');
    assertCapacity(parent.id,GRANT_COST,0);
    const issuerToken=token();
    const record={id:randomId(),parent:parent.id,locations:loc,operations:[...ops],
      maxBytes,maxObjects,issuerHash:digest(issuerToken),users:[]};
    writeGrant(record);
    return {id:record.id,issuerToken,locations:loc,operations:ops,maxBytes,maxObjects};
  }
  function createUser(id,issuerToken) {
    const g=requireIssuer(id,issuerToken);
    if(g.users.length) assertCapacity(g.id,EXTRA_USER_COST,0);
    const userToken=token(),userId=randomId();
    const next={...g,users:[...g.users,{id:userId,hash:digest(userToken)}]};
    const charged=GRANT_COST+Math.max(0,next.users.length-1)*EXTRA_USER_COST;
    if(Buffer.byteLength(JSON.stringify(next))>charged) throw fault(507,'grant_quota_exceeded');
    writeGrant(next);
    return {grantId:id,userId,userToken};
  }
  function revokeUser(id,issuerToken,userId) {
    const g=requireIssuer(id,issuerToken);
    if(!ID_RE.test(userId||'')) throw fault(400,'invalid_user_id');
    const next=g.users.filter(u=>u.id!==userId);
    if(next.length===g.users.length) throw fault(404,'user_not_found');
    writeGrant({...g,users:next});
    return {ok:true,grantId:id,userId};
  }
  function accountSubscription(secret,locations) {
    const g=requireUser(secret);
    requireOperation(g,'subscribe');
    if(!validLocations(locations)) throw fault(400,'invalid_locations');
    if(!subset(locations,g.locations)) throw fault(403,'locations_outside_parent');
    assertCapacity(g.id,PUSH_SUB_COST,1);
    return g.id;
  }
  function runRevocation(id,ids) {
    ensureDir(pendingDir,0o700);
    const marker=path.join(pendingDir,id+'.json');
    if(!fs.existsSync(marker)) atomicJson(marker,{id,ids});
    const work=JSON.parse(fs.readFileSync(marker,'utf8'));
    const revoked=new Set(work.ids);
    for(const sub of subscriptionEntries()) if(revoked.has(sub.data.grantId)) {
      try {fs.unlinkSync(sub.file);}catch(e){if(e.code!=='ENOENT')throw e;}
    }
    if(fs.existsSync(subdir)) syncDir(subdir);
    subscriptionChanged();
    // Delete the root last. The root credential lives in its own private
    // grant record, so a crash cannot leave a live root without its credential.
    const records=work.ids.slice().sort((a,b)=>
      (grantMap().get(a)?.parent===null?1:0)-(grantMap().get(b)?.parent===null?1:0));
    for(const grantId of records) {
      try {fs.unlinkSync(recordPath(grantId));}
      catch(e){if(e.code!=='ENOENT')throw e;}
      grantMap().delete(grantId);
    }
    if(fs.existsSync(dir)) syncDir(dir);
    fs.unlinkSync(marker);syncDir(pendingDir);
  }
  function revokeGrant(id,issuerToken) {
    requireIssuer(id,issuerToken);
    const map=validateTree();
    const ids=[...map.keys()].filter(candidate=>isDescendant(map,candidate,id));
    runRevocation(id,ids);
    return {ok:true,grantId:id};
  }
  function recoverRevocations() {
    ensureDir(pendingDir,0o700);
    for(const filename of fs.readdirSync(pendingDir)) {
      if(!/^[a-f0-9]{32}\.json$/.test(filename)) throw new Error('invalid revocation marker');
      const work=JSON.parse(fs.readFileSync(path.join(pendingDir,filename),'utf8'));
      if(!ID_RE.test(work.id)||filename!==work.id+'.json'||!Array.isArray(work.ids)||
        !work.ids.includes(work.id)||new Set(work.ids).size!==work.ids.length||
        work.ids.some(id=>!ID_RE.test(id))) throw new Error('corrupt revocation marker');
      runRevocation(work.id,work.ids);
    }
  }
  function recover() {
    recoverRevocations();
    const map=validateTree();
    const subs=subscriptionEntries();
    if(!map.size&&subs.length) throw new Error('subscriptions without grants');
    for(const sub of subs) if(!map.has(sub.data.grantId)) {
      throw new Error('subscription requires live grant');
    }
  }
  function inspect(id,secret) {
    requireUser(secret,id);
    const {issuerHash,rootIssuerToken,users,...safe}=find(id);
    return {...safe,userCount:users.length,usage:usage(id)};
  }
  return {GRANT_COST,OBJECT_COST,PUSH_SUB_COST,OPERATIONS,validKey,
    validLocations,matches,subset,root,ensureRoot,createChild,createUser,
    revokeUser,revokeGrant,recover,recoverRevocations,requireUser,
    authorizeMutation,listObjects,recordObject,accountSubscription,subscriptionChanged,usage,inspect};
}
module.exports={createGrants,KEY_RE,LOCATION_RE,OPERATIONS,validKey,validLocations,matches,subset};
