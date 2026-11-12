'use strict';

// Grant-authorized, location-scoped browser notifications.
// Every subscription names its own object scope and events.
const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const {ensureDir,syncDir,atomicJson}=require('./shared.js');
const {matches,validLocations}=require('./grants.js');
const {validEndpoint,safeAgent}=require('./push-endpoint.js');

const KINDS=Object.freeze(['create','replace','delete']);
const MAX_SUBSCRIPTIONS=2000,MAX_OUTBOX_ITEMS=10000;
const DEFAULT_SEND_TIMEOUT_MS=10000;
const DEFAULT_CONCURRENCY=8;
const DEFAULT_MAX_ATTEMPTS=12;
function randomId() {return crypto.randomBytes(16).toString('hex');}
function randomSecret() {return crypto.randomBytes(32).toString('hex');}
function hash(x) {return crypto.createHash('sha256').update(x).digest('hex');}
function safeMatch(a,b) {
  return typeof a==='string'&&typeof b==='string'&&
    /^[a-f0-9]{64}$/.test(a)&&/^[a-f0-9]{64}$/.test(b)&&
    crypto.timingSafeEqual(Buffer.from(a,'hex'),Buffer.from(b,'hex'));
}
function fault(code) {const e=new Error(code);e.code=code;return e;}
function readJson(file) {
  try {return JSON.parse(fs.readFileSync(file,'utf8'));}
  catch(e){if(e.code==='ENOENT')return null;throw e;}
}
function validKeys(keys) {
  return keys&&typeof keys==='object'&&
    ['p256dh','auth'].every(k=>typeof keys[k]==='string'&&keys[k].length>0&&keys[k].length<=256);
}
function validEvents(events) {
  return Array.isArray(events)&&events.length>0&&
    new Set(events).size===events.length&&events.every(x=>KINDS.includes(x));
}
function backoffMs(attempts) {
  // Slow repeated failures instead of continuously probing arbitrary hosts.
  return Math.min(30*60*1000,1000*Math.pow(2,Math.max(0,attempts-1)));
}
function createPush(opts={}) {
  const stateDir=path.join(opts.dataDir||process.env.SKRYNIA_DATA_DIR||'/var/lib/skrynia','state');
  const pushDir=path.join(stateDir,'push');
  const subdir=path.join(stateDir,'push-subs');
  const outboxDir=path.join(pushDir,'outbox');
  const vapidPath=path.join(pushDir,'vapid.json');
  const subject=opts.pushSubject||process.env.SKRYNIA_PUSH_SUBJECT||opts.skryniaUrl||'https://skrynia.local/';
  const maxSubs=opts.maxSubscriptions??MAX_SUBSCRIPTIONS;
  const maxOutbox=opts.maxOutboxItems??MAX_OUTBOX_ITEMS;
  const concurrency=opts.pushConcurrency??Number(process.env.SKRYNIA_PUSH_CONCURRENCY||DEFAULT_CONCURRENCY);
  const maxAttempts=opts.pushMaxAttempts??
    Number(process.env.SKRYNIA_PUSH_MAX_ATTEMPTS||DEFAULT_MAX_ATTEMPTS);
  const hosts=opts.pushAllowedHosts??process.env.SKRYNIA_PUSH_ALLOWED_HOSTS??'';
  if(typeof hosts!=='string') throw new Error('invalid push allowed hosts');
  const hostnames=hosts.split(',').map(host=>host.trim().toLowerCase())
    .filter(Boolean);
  if(hostnames.some(host=>!validEndpoint('https://'+host+'/'))) {
    throw new Error('invalid push allowed hosts');
  }
  const allowedHosts=new Set(hostnames);
  function endpointAllowed(endpoint) {
    return validEndpoint(endpoint)&&
      (allowedHosts.size===0||allowedHosts.has(new URL(endpoint).hostname.toLowerCase()));
  }
  const timeoutMs=opts.pushSendTimeoutMs??Number(process.env.SKRYNIA_PUSH_SEND_TIMEOUT_MS||DEFAULT_SEND_TIMEOUT_MS);
  const manual=Boolean(opts.pushManual);
  const pollMs=opts.pushPollMs??1000;
  const transport=opts.pushTransport||null;
  const agent=safeAgent();
  const webPush=()=>opts.webPushLib||require('web-push');
  const authorize=opts.onSubscriptionCreate||(()=>{throw new Error('grant authorization required');});
  const onMutation=opts.onSubscriptionMutation||(()=>{});
  if(!Number.isSafeInteger(timeoutMs)||timeoutMs<1) throw new Error('invalid push timeout');
  if(!Number.isSafeInteger(maxSubs)||maxSubs<1) throw new Error('invalid subscription limit');
  if(!Number.isSafeInteger(maxOutbox)||maxOutbox<1) throw new Error('invalid outbox limit');
  if(!Number.isSafeInteger(concurrency)||concurrency<1||concurrency>100)
    throw new Error('invalid push concurrency');
  if(!Number.isSafeInteger(maxAttempts)||maxAttempts<1||maxAttempts>100)
    throw new Error('invalid push max attempts');
  let timer=null,pumping=false,cachedSubs=null;
  const inFlight=new Set();
  function subPath(id) {return path.join(subdir,id+'.json');}
  function invalidateSubscriptions() {cachedSubs=null;}
  function allSubs() {
    if(cachedSubs) return cachedSubs;
    const values=fs.existsSync(subdir)?
      fs.readdirSync(subdir).filter(n=>/^[a-f0-9]{32}\.json$/.test(n))
        .map(n=>readJson(path.join(subdir,n))).filter(Boolean):[];
    cachedSubs=values;
    return values;
  }
  function findSub(id) {
    if(!/^[a-f0-9]{32}$/.test(id||'')) return null;
    const sub=readJson(subPath(id));return sub&&sub.id===id?sub:null;
  }
  function getVapidKeys() {
    const existing=readJson(vapidPath);
    if(existing) {
      if(typeof existing.publicKey!=='string'||typeof existing.privateKey!=='string'||
        !existing.publicKey||!existing.privateKey) throw new Error('invalid VAPID state');
      return existing;
    }
    const keys=webPush().generateVAPIDKeys();
    atomicJson(vapidPath,keys);
    return keys;
  }
  function createSub(locations,events,endpoint,keys,existingCapability,grantToken) {
    if(!validLocations(locations)||!validEvents(events)||!endpointAllowed(endpoint)||!validKeys(keys)) {
      throw fault('invalid_subscription');
    }
    const subs=allSubs();
    const sameEvents=(a,b)=>
      JSON.stringify([...a].sort())===JSON.stringify([...b].sort());
    const existing=subs.find(s=>s.endpoint===endpoint&&
      s.locations===locations&&sameEvents(s.events,events));
    if(existing) {
      if(!existingCapability) throw fault('capability_required');
      if(!safeMatch(existing.capHash,hash(existingCapability))) throw fault('invalid_capability');
      const capability=randomSecret();
      existing.capHash=hash(capability);
      existing.keys={p256dh:keys.p256dh,auth:keys.auth};
      atomicJson(subPath(existing.id),existing);
      invalidateSubscriptions();
      return {id:existing.id,capability,deduped:true};
    }
    if(subs.length>=maxSubs) throw fault('subscriptions_full');
    // Validation and resource charging are performed against the same grant.
    const grantId=authorize(grantToken,locations);
    const id=randomId(),capability=randomSecret();
    const record={id,grantId,locations,events:[...events],
      endpoint,keys:{p256dh:keys.p256dh,auth:keys.auth},
      capHash:hash(capability)};
    atomicJson(subPath(id),record);
    invalidateSubscriptions();
    onMutation();
    return {id,capability,deduped:false};
  }
  function updateSub(id,capability,patch) {
    const sub=findSub(id);
    if(!sub) throw fault('not_found');
    if(!safeMatch(sub.capHash,hash(capability||''))) throw fault('invalid_capability');
    if(patch.endpoint!==undefined) {
      if(!endpointAllowed(patch.endpoint)) throw fault('invalid_subscription');
      if(allSubs().some(s=>s.id!==id&&s.endpoint===patch.endpoint&&
        s.locations===sub.locations&&
        JSON.stringify([...s.events].sort())===JSON.stringify([...sub.events].sort()))) {
        throw fault('endpoint_in_use');
      }
      sub.endpoint=patch.endpoint;
    }
    if(patch.keys!==undefined) {
      if(!validKeys(patch.keys)) throw fault('invalid_subscription');
      sub.keys={p256dh:patch.keys.p256dh,auth:patch.keys.auth};
    }
    atomicJson(subPath(id),sub);
    invalidateSubscriptions();
    return {ok:true};
  }
  function deleteRecord(id) {
    fs.unlinkSync(subPath(id));syncDir(subdir);invalidateSubscriptions();onMutation();
  }
  function removeSub(id,capability) {
    const sub=findSub(id);
    if(!sub) throw fault('not_found');
    if(!safeMatch(sub.capHash,hash(capability||''))) throw fault('invalid_capability');
    deleteRecord(id);return {ok:true};
  }
  function removeSubIfUnchanged(old) {
    const current=findSub(old.id);
    if(current&&current.endpoint===old.endpoint&&current.capHash===old.capHash&&
      current.keys.p256dh===old.keys.p256dh&&current.keys.auth===old.keys.auth) {
      deleteRecord(old.id);
      return true;
    }
    return false;
  }
  function matching(key,kind) {
    return allSubs().filter(s=>s.events.includes(kind)&&matches(s.locations,key));
  }
  function outboxCount() {
    return fs.existsSync(outboxDir)?fs.readdirSync(outboxDir).filter(x=>x.endsWith('.json')).length:0;
  }
  // A notification is a wake-up to reread current state. One durable
  // pending wake-up per subscription is sufficient, even if its endpoint
  // remains offline while thousands of mutations occur.
  function notificationPath(subId) {return path.join(outboxDir,subId+'.json');}
  function preflight(key,kind) {
    const needed=matching(key,kind).filter(s=>!fs.existsSync(notificationPath(s.id))).length;
    if(outboxCount()+needed>maxOutbox) throw fault('outbox_full');
    return needed;
  }
  function enqueue(key,kind) {
    const subscribers=matching(key,kind);
    const needed=subscribers.filter(s=>!fs.existsSync(notificationPath(s.id))).length;
    if(outboxCount()+needed>maxOutbox) throw fault('outbox_full');
    for(const sub of subscribers) {
      const file=notificationPath(sub.id);
      const pending=readJson(file);
      if(pending&&pending.subscriptionId!==sub.id) throw new Error('corrupt push outbox');
      atomicJson(file,{subscriptionId:sub.id,
        generation:(pending?.generation||0)+1,
        // An unrelated mutation must not bypass retry backoff or reset the
        // failed-send budget of an unresponsive subscription.
        attempts:pending?.attempts||0,nextAt:pending?.nextAt||0});
    }
    if(subscribers.length) schedule();
    return subscribers.map(s=>s.id);
  }
  function listOutbox() {
    if(!fs.existsSync(outboxDir)) return [];
    return fs.readdirSync(outboxDir).filter(n=>/^[a-f0-9]{32}\.json$/.test(n))
      .map(n=>({file:path.join(outboxDir,n),entry:readJson(path.join(outboxDir,n))}));
  }
  function isGoneError(e) {return e&&(e.statusCode===404||e.statusCode===410||e.status===404||e.status===410);}
  async function sendBounded(sub) {
    if(!endpointAllowed(sub.endpoint)) throw fault('push_endpoint_not_allowed');
    const send=transport||((s)=>webPush().sendNotification(
      {endpoint:s.endpoint,keys:s.keys},'changed',
      {timeout:timeoutMs,agent,vapidDetails:{subject,publicKey:getVapidKeys().publicKey,privateKey:getVapidKeys().privateKey}}));
    let timeout;
    try {
      return await Promise.race([
        Promise.resolve().then(()=>send(sub)),
        new Promise((_,reject)=>{timeout=setTimeout(()=>reject(fault('push_timeout')),timeoutMs);}),
      ]);
    } finally {clearTimeout(timeout);}
  }
  async function pumpOne(file,ignoreBackoff) {
    if(inFlight.has(file)) return false;
    inFlight.add(file);
    try {
      const fresh=readJson(file);
      if(!fresh||!ignoreBackoff&&fresh.nextAt>Date.now()) return false;
      const sub=findSub(fresh.subscriptionId);
      if(!sub) {fs.unlinkSync(file);return false;}
      try {
        await sendBounded(sub);
        const current=readJson(file);
        // Never acknowledge a newer mutation that happened during the send.
        if(current&&current.generation===fresh.generation) fs.unlinkSync(file);
        return true;
      } catch(e) {
        const current=readJson(file);
        if(!current) return false;
        if(!findSub(fresh.subscriptionId)) {
          fs.unlinkSync(file);
          return false;
        }
        const attempts=Math.max(current.attempts||0,fresh.attempts||0)+1;
        if(isGoneError(e)||e.code==='push_endpoint_not_allowed'||
          attempts>=maxAttempts) {
          // Permanently dead endpoints must not be retried every 30 seconds
          // forever. A refreshed subscription is preserved and retried.
          if(removeSubIfUnchanged(sub)||!findSub(fresh.subscriptionId)) {
            fs.unlinkSync(file);
          } else {
            atomicJson(file,{...current,attempts:0,nextAt:0});
          }
        } else {
          // Update the latest generation too: a write racing with this
          // failed send must not clear the failed-send budget.
          atomicJson(file,{...current,attempts,
            nextAt:Date.now()+backoffMs(attempts)});
        }
        return false;
      }
    } finally {
      inFlight.delete(file);
    }
  }
  async function pumpOnce({ignoreBackoff=false}={}) {
    const files=listOutbox().map(x=>x.file);
    let next=0,delivered=0;
    async function worker() {
      while(next<files.length) {
        const file=files[next++];
        if(await pumpOne(file,ignoreBackoff)) delivered++;
      }
    }
    await Promise.all(Array.from({length:Math.min(concurrency,files.length)},worker));
    return {delivered};
  }
  async function pumpTick() {
    if(pumping) return;
    pumping=true;
    try {await pumpOnce();}
    catch(e){console.error('skrynia push pump error:',e);}
    finally {pumping=false;}
  }
  function schedule() {if(!manual) setImmediate(pumpTick);}
  function start() {
    ensureDir(outboxDir,0o700);getVapidKeys();
    if(manual) return;
    pumpTick();
    if(pollMs>0) {timer=setInterval(pumpTick,pollMs);timer.unref();}
  }
  function close() {if(timer) clearInterval(timer);timer=null;}
  return {KINDS,validEndpoint,validKeys,validEvents,getPublicKey:()=>getVapidKeys().publicKey,
    getVapidKeys,allSubs,invalidateSubscriptions,findSub,createSub,updateSub,removeSub,preflight,enqueue,
    listOutbox,outboxCount,pumpOnce,start,close,outboxDir};
}
module.exports={createPush,KINDS,MAX_SUBSCRIPTIONS,MAX_OUTBOX_ITEMS};
