'use strict';

// Grant authority is location-scoped; object files do not carry grant identity.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { ensureDir, validNs } = require('./shared.js');
const { createResourceIndex } = require('./resource-index.js');
const { syncDir } = require('./store-journal.js');

const GRANT_COST = 2048;
const OBJECT_COST = 512;
const PUSH_SUB_COST = 4096;
const EXTRA_USER_COST = 128;
const ID_RE = /^[a-f0-9]{32}$/;
const KEY_RE = /^[A-Za-z0-9_]+$/;
const LOCATION_RE = /^[A-Za-z0-9_]*\*?$/;
const OPERATIONS = Object.freeze(['write', 'delegate', 'subscribe']);

function fault(status, code) {
  const e = new Error(code);
  e.status = status;
  e.code = code;
  return e;
}
function token() { return crypto.randomBytes(32).toString('hex'); }
function digest(s) { return crypto.createHash('sha256').update(s).digest('hex'); }
function same(a, b) {
  return typeof a === 'string' && typeof b === 'string' &&
    /^[a-f0-9]{64}$/.test(a) && /^[a-f0-9]{64}$/.test(b) &&
    crypto.timingSafeEqual(Buffer.from(a, 'hex'), Buffer.from(b, 'hex'));
}
function boundedInteger(v) { return Number.isSafeInteger(v) && v > 0; }
function validKey(key) { return typeof key === 'string' && KEY_RE.test(key); }
function validLocations(loc) { return typeof loc === 'string' && LOCATION_RE.test(loc); }
function matches(loc, key) {
  if (!validLocations(loc) || !validKey(key)) return false;
  return loc.endsWith('*') ? key.startsWith(loc.slice(0, -1)) : key === loc;
}
function subset(child, parent) {
  if (!validLocations(child) || !validLocations(parent)) return false;
  // Empty exact matcher contains no valid keys and is contained everywhere.
  if (child === '') return true;
  const childWild = child.endsWith('*'), parentWild = parent.endsWith('*');
  if (parentWild) {
    const prefix = parent.slice(0, -1);
    return child.startsWith(prefix);
  }
  return !childWild && child === parent;
}
function normalizeOperations(value) {
  if (!Array.isArray(value) || value.some(x => !OPERATIONS.includes(x)) ||
      new Set(value).size !== value.length) throw fault(400, 'invalid_operations');
  return OPERATIONS.filter(x => value.includes(x));
}
function subsetOperations(child, parent) {
  return child.every(op => parent.includes(op));
}

function createGrants(shared) {
  const { dataDir } = shared;
  const objectsIndex = createResourceIndex(shared, validKey, matches);
  const grantsCache=new Map(), subscriptionsCache=new Map();
  const stateDir = path.join(dataDir, 'state');
  const pendingDir = path.join(stateDir, '_grant-revocations');
  function dir(ns) { return path.join(stateDir, ns, 'grants'); }
  function recordPath(ns, id) { return path.join(dir(ns), id + '.json'); }

  function atomicJson(p, obj) {
    ensureDir(path.dirname(p), 0o700);
    const tmp = p + '.' + crypto.randomBytes(12).toString('hex') + '.tmp';
    try {
      const fd=fs.openSync(tmp,'wx',0o600);
      try {
        fs.writeSync(fd,JSON.stringify(obj));
        fs.fsyncSync(fd);
      } finally { fs.closeSync(fd); }
      fs.renameSync(tmp,p);
      syncDir(path.dirname(p));
    } finally {
      try { fs.unlinkSync(tmp); } catch (e) { if (e.code !== 'ENOENT') throw e; }
    }
  }

  function grantMap(ns) {
    if (!validNs(ns)) throw fault(400, 'invalid_namespace');
    if(grantsCache.has(ns)) return grantsCache.get(ns);
    const result=new Map();
    if(fs.existsSync(dir(ns))) {
      for(const filename of fs.readdirSync(dir(ns))) {
        if (!filename.endsWith('.json') || !ID_RE.test(filename.slice(0,-5))) continue;
        const g=JSON.parse(fs.readFileSync(path.join(dir(ns),filename),'utf8'));
        result.set(g.id,g);
      }
    }
    grantsCache.set(ns,result);
    return result;
  }
  function cloneGrant(g) {
    return {...g, operations:[...g.operations],
      users:g.users.map(u=>({...u}))};
  }
  function all(ns) { return [...grantMap(ns).values()].map(cloneGrant); }
  function writeGrant(ns,g) {
    atomicJson(recordPath(ns,g.id),g);
    grantMap(ns).set(g.id,cloneGrant(g));
  }
  function reloadGrants(ns) { grantsCache.delete(ns); }
  function find(ns, id) {
    if (!ID_RE.test(id || '')) throw fault(400, 'invalid_grant');
    const g = all(ns).find(x => x.id === id);
    if (!g) throw fault(404, 'grant_not_found');
    return g;
  }
  function root(ns) { return all(ns).find(g => g.parent === null) || null; }
  function validateTree(grants) {
    const map = new Map(grants.map(g => [g.id, g]));
    if (map.size !== grants.length) throw new Error('duplicate grant ID');
    const children = new Map(grants.map(g => [g.id, []]));
    const roots = [];
    for (const g of grants) {
      if (!ID_RE.test(g.id) || !validNs(g.namespace) ||
          !validLocations(g.locations) ||
          !Array.isArray(g.operations) ||
          g.operations.some(op => !OPERATIONS.includes(op)) ||
          !boundedInteger(g.maxBytes) || !boundedInteger(g.maxObjects) ||
          !Array.isArray(g.users) ||
          g.users.some(u => !ID_RE.test(u.id) || !/^[a-f0-9]{64}$/.test(u.hash))) {
        throw new Error('invalid grant record');
      }
      if (g.parent === null) roots.push(g);
      else {
        const p = map.get(g.parent);
        if (!p) throw new Error('corrupt grant ancestry');
        if (!subset(g.locations, p.locations) ||
            !subsetOperations(g.operations, p.operations)) {
          throw new Error('grant authority escalation');
        }
        children.get(g.parent).push(g);
      }
    }
    if (grants.length && roots.length !== 1) throw new Error('invalid grant roots');
    const order = roots.slice(), depth = new Map(roots.map(g => [g.id, 0]));
    for (let i = 0; i < order.length; i++) {
      const g = order[i];
      for (const c of children.get(g.id)) {
        depth.set(c.id, depth.get(g.id) + 1);
        order.push(c);
      }
    }
    if (order.length !== grants.length) throw new Error('cyclic grant ancestry');
    map.order = order;
    map.depth = depth;
    return map;
  }
  function isDescendant(map, child, ancestor) {
    let g = map.get(child);
    while (g) {
      if (g.id === ancestor) return true;
      g = g.parent === null ? null : map.get(g.parent);
    }
    return false;
  }

  function requireUser(ns, secret, id) {
    if (typeof secret !== 'string') throw fault(401, 'grant_user_token_required');
    const hash = digest(secret);
    const g = all(ns).find(x =>
      x.users.some(u => same(u.hash, hash)) && (!id || x.id === id));
    if (!g) throw fault(403, 'invalid_grant_user_token');
    return g;
  }
  function requireOperation(g, op) {
    if (!g.operations.includes(op)) throw fault(403, 'grant_operation_forbidden');
  }
  function requireIssuer(ns, id, secret) {
    const g = find(ns, id);
    if (typeof secret !== 'string' || !same(g.issuerHash, digest(secret))) {
      throw fault(403, 'invalid_grant_issuer_token');
    }
    return g;
  }
  function requireWrite(ns, secret, key) {
    const g = requireUser(ns, secret);
    requireOperation(g, 'write');
    if (!matches(g.locations, key)) throw fault(403, 'location_not_authorized');
    return g;
  }

  function objectEntries(ns) {
    return objectsIndex.entries(ns);
  }
  function recordObject(ns, key, bytes) {
    objectsIndex.replace(ns,key,bytes);
  }
  function dropNamespace(ns) {
    objectsIndex.forget(ns);
    grantsCache.delete(ns);
    subscriptionsCache.delete(ns);
  }
  function subscriptionEntries(ns) {
    if(subscriptionsCache.has(ns)) return subscriptionsCache.get(ns);
    const p = path.join(stateDir, ns, 'push-subs');
    const result = fs.existsSync(p) ? fs.readdirSync(p)
      .filter(x=>x.endsWith('.json')).map(x=>({
        file:path.join(p,x),
        data:JSON.parse(fs.readFileSync(path.join(p,x),'utf8')),
      })) : [];
    subscriptionsCache.set(ns,result);
    return result;
  }
  function subscriptionChanged(ns) { subscriptionsCache.delete(ns); }
  function usageWith(ns, g, map, objects, subscriptions) {
    const objectStats = objectsIndex.stats(ns, g.locations);
    let bytes = objectStats.bytes + objectStats.objects * OBJECT_COST;
    let objectCount = objectStats.objects;
    for (const h of map.values()) {
      if (!isDescendant(map, h.id, g.id)) continue;
      bytes += GRANT_COST + Math.max(0, h.users.length - 1) * EXTRA_USER_COST;
    }
    for (const sub of subscriptions) {
      if (!sub.data.grantId || !map.has(sub.data.grantId)) {
        throw new Error('orphaned push subscription');
      }
      if (!isDescendant(map, sub.data.grantId, g.id)) continue;
      bytes += PUSH_SUB_COST;
      objectCount++;
    }
    return {bytes, objects:objectCount};
  }
  function usage(ns, id) {
    const map = validateTree(all(ns)), g = map.get(id);
    if (!g) throw fault(404, 'grant_not_found');
    return usageWith(ns, g, map, objectEntries(ns), subscriptionEntries(ns));
  }

  function ancestors(map, id) {
    const result = [];
    let g = map.get(id);
    if (!g) throw fault(404, 'grant_not_found');
    while (g) {
      result.push(g);
      g = g.parent === null ? null : map.get(g.parent);
    }
    return result;
  }
  function assertCapacity(ns, id, extraBytes, extraObjects) {
    const map = validateTree(all(ns));
    const objects = objectEntries(ns), subs = subscriptionEntries(ns);
    for (const g of ancestors(map, id)) {
      const u = usageWith(ns, g, map, objects, subs);
      if (u.bytes + extraBytes > g.maxBytes ||
          u.objects + extraObjects > g.maxObjects) {
        throw fault(507, 'grant_quota_exceeded');
      }
    }
  }
  function authorizeWrite(ns, secret, key, newBytes, oldBytes) {
    const g = requireWrite(ns, secret, key);
    if (newBytes !== null) {
      if (!Number.isSafeInteger(newBytes) || newBytes < 0) throw fault(400, 'invalid_object_size');
      const map = validateTree(all(ns));
      const subs = subscriptionEntries(ns);
      const old = objectsIndex.value(ns, key);
      const changedBytes = newBytes - (old===null ? 0 : old) +
        (old===null ? OBJECT_COST : 0);
      const changedObjects = old===null ? 1 : 0;
      // The old object is logically removed before creation is checked.
      // Equivalent to evaluating the full post-edit state, but O(ancestors).
      for (const a of ancestors(map, g.id)) {
        const current = usageWith(ns, a, map, null, subs);
        if (current.bytes + changedBytes > a.maxBytes ||
            current.objects + changedObjects > a.maxObjects) {
          throw fault(507, 'grant_quota_exceeded');
        }
      }
    }
    return g.id;
  }

  function makeRecord(ns, parent, maxBytes, maxObjects, locations, operations) {
    const issuerToken = token();
    const id = crypto.randomBytes(16).toString('hex');
    const record = {id, namespace:ns, parent, maxBytes, maxObjects, locations,
      operations, issuerHash:digest(issuerToken), users:[]};
    if (Buffer.byteLength(JSON.stringify(record)) > GRANT_COST) {
      throw new Error('grant record exceeds charged bytes');
    }
    writeGrant(ns,record);
    return {id, issuerToken, maxBytes, maxObjects, locations, operations};
  }
  function limits(maxBytes, maxObjects) {
    if (!boundedInteger(maxBytes) || !boundedInteger(maxObjects) ||
        maxBytes < GRANT_COST) throw fault(400, 'invalid_grant_limits');
  }
  function createRoot(ns, maxBytes, maxObjects, locations = '*', operations = OPERATIONS) {
    if (!validNs(ns)) throw fault(400, 'invalid_namespace');
    limits(maxBytes, maxObjects);
    if (!validLocations(locations)) throw fault(400, 'invalid_locations');
    const ops = normalizeOperations(operations);
    if (root(ns)) return {created:false, id:root(ns).id};
    if (all(ns).length || subscriptionEntries(ns).length) {
      throw new Error('namespace contains grant metadata without a root grant');
    }
    return {...makeRecord(ns, null, maxBytes, maxObjects, locations, ops), created:true};
  }
  function createChild(ns, secret, maxBytes, maxObjects, locations, operations) {
    const parent = requireUser(ns, secret);
    requireOperation(parent, 'delegate');
    limits(maxBytes, maxObjects);
    const loc = locations === undefined ? parent.locations : locations;
    const ops = operations === undefined ? parent.operations : normalizeOperations(operations);
    if (!validLocations(loc)) throw fault(400, 'invalid_locations');
    if (!subset(loc, parent.locations)) throw fault(403, 'locations_outside_parent');
    if (!subsetOperations(ops, parent.operations)) throw fault(403, 'operations_outside_parent');
    assertCapacity(ns, parent.id, GRANT_COST, 0);
    return makeRecord(ns, parent.id, maxBytes, maxObjects, loc, ops);
  }
  function createUser(ns, id, secret) {
    const g = requireIssuer(ns, id, secret);
    if (g.users.length !== 0) assertCapacity(ns, g.id, EXTRA_USER_COST, 0);
    const userToken = token();
    const userId = crypto.randomBytes(16).toString('hex');
    g.users.push({id:userId, hash:digest(userToken)});
    const cost = GRANT_COST + Math.max(0, g.users.length - 1) * EXTRA_USER_COST;
    if (Buffer.byteLength(JSON.stringify(g)) > cost) throw fault(507, 'grant_quota_exceeded');
    writeGrant(ns,g);
    return {grantId:id, userId, userToken};
  }
  function revokeUser(ns, id, issuerSecret, userId) {
    const g = requireIssuer(ns, id, issuerSecret);
    if (!ID_RE.test(userId || '')) throw fault(400, 'invalid_user_id');
    const next = g.users.filter(u => u.id !== userId);
    if (next.length === g.users.length) throw fault(404, 'user_not_found');
    g.users = next;
    writeGrant(ns,g);
    return {ok:true, grantId:id, userId};
  }
  function rotateRootIssuer(ns) {
    const g = root(ns);
    if (!g) throw fault(404, 'namespace_not_found');
    const issuerToken = token();
    g.issuerHash = digest(issuerToken);
    writeGrant(ns,g);
    return {grantId:g.id, issuerToken};
  }
  function accountSubscription(ns, secret) {
    const g = requireUser(ns, secret);
    requireOperation(g, 'subscribe');
    assertCapacity(ns, g.id, PUSH_SUB_COST, 1);
    return g.id;
  }
  function runRevocation(ns, id, ids) {
    const marker = path.join(pendingDir, ns + '-' + id + '.json');
    if (!fs.existsSync(marker)) atomicJson(marker, {ns, id, ids});
    const work = JSON.parse(fs.readFileSync(marker, 'utf8'));
    const revoked = new Set(work.ids);
    const subdir = path.join(stateDir, ns, 'push-subs');
    for (const sub of subscriptionEntries(work.ns)) {
      if (!revoked.has(sub.data.grantId)) continue;
      try { fs.unlinkSync(sub.file); }
      catch (e) { if (e.code !== 'ENOENT') throw e; }
    }
    if (fs.existsSync(subdir)) syncDir(subdir);
    subscriptionChanged(work.ns);
    for (const grantId of work.ids) {
      try { fs.unlinkSync(recordPath(work.ns, grantId)); }
      catch (e) { if (e.code !== 'ENOENT') throw e; }
      grantMap(work.ns).delete(grantId);
    }
    if (fs.existsSync(dir(work.ns))) syncDir(dir(work.ns));
    fs.unlinkSync(marker);
    syncDir(pendingDir);
  }
  function revokeGrant(ns, id, secret) {
    requireIssuer(ns, id, secret);
    const map = validateTree(all(ns));
    const ids = [...map.values()]
      .filter(g => isDescendant(map, g.id, id)).map(g => g.id);
    runRevocation(ns, id, ids);
    return {ok:true, grantId:id};
  }
  function recoverRevocations() {
    ensureDir(pendingDir, 0o700);
    for (const name of fs.readdirSync(pendingDir).filter(x => x.endsWith('.json'))) {
      const op = JSON.parse(fs.readFileSync(path.join(pendingDir, name), 'utf8'));
      if (!validNs(op.ns) || !ID_RE.test(op.id) ||
          !Array.isArray(op.ids) || op.ids.some(id => !ID_RE.test(id)) ||
          !op.ids.includes(op.id) || op.ids.length !== new Set(op.ids).size ||
          name !== op.ns + '-' + op.id + '.json') {
        throw new Error('invalid pending grant revocation');
      }
      runRevocation(op.ns, op.id, op.ids);
    }
  }
  function recover() {
    recoverRevocations();
    if (!fs.existsSync(stateDir)) return;
    const namespaces = new Set();
    for (const rootDir of [stateDir, shared.STORE_DIR]) {
      if (fs.existsSync(rootDir)) for (const ns of fs.readdirSync(rootDir).filter(validNs)) {
        namespaces.add(ns);
      }
    }
    for (const ns of namespaces) {
      const grants = all(ns);
      const map = validateTree(grants);
      const subs = subscriptionEntries(ns);
      if (!grants.length && subs.length) {
        throw new Error('subscriptions require a root grant: ' + ns);
      }
      for (const sub of subs) if (!map.has(sub.data.grantId)) {
        throw new Error('subscription requires a live funding grant');
      }
    }
  }
  // Administrative discovery reveals no credential hashes.
  function listGrants(ns) {
    const records=all(ns), map=validateTree(records);
    const objects=objectEntries(ns), subs=subscriptionEntries(ns);
    return {namespace:ns, grants:records.map(g => ({
      id:g.id, parent:g.parent, locations:g.locations,
      operations:g.operations, maxBytes:g.maxBytes, maxObjects:g.maxObjects,
      userCount:g.users.length,
      usage:usageWith(ns,g,map,objects,subs),
    }))};
  }
  function listKeys(ns, prefix='', after='', limit=100) {
    if (!validNs(ns)) throw fault(400, 'invalid_namespace');
    if (typeof prefix!=='string' || (prefix && !KEY_RE.test(prefix)) ||
        typeof after!=='string' || (after && !KEY_RE.test(after))) {
      throw fault(400, 'invalid_key_filter');
    }
    const pageSize=Number(limit);
    if (!Number.isSafeInteger(pageSize) || pageSize<1 || pageSize>200) {
      throw fault(400, 'invalid_page_limit');
    }
    const keys=objectEntries(ns).filter(o =>
      o.key.startsWith(prefix) && o.key>after)
      .sort((a,b)=>a.key.localeCompare(b.key,'en'));
    // Key alphabet is ASCII; use deterministic codepoint comparison for cursors.
    keys.sort((a,b)=>a.key<b.key?-1:a.key>b.key?1:0);
    const page=keys.slice(0,pageSize);
    return {namespace:ns, keys:page, nextAfter:keys.length>pageSize?
      page[page.length-1].key:null};
  }
  function inspect(ns, id, secret) {
    requireUser(ns, secret, id);
    const g = find(ns, id), {issuerHash, users, ...safe} = g;
    return {...safe, userCount:users.length, usage:usage(ns, id)};
  }

  return {GRANT_COST, OBJECT_COST, PUSH_SUB_COST, OPERATIONS,
    root, createRoot, createChild, createUser, revokeUser,
    rotateRootIssuer, requireUser, requireWrite, recordObject, dropNamespace,
    subscriptionChanged, reloadGrants,
    accountSubscription, authorizeWrite, revokeGrant, inspect,
    usage, recover, recoverRevocations, listGrants, listKeys,
    matches, subset, validKey, validLocations};
}
module.exports = {createGrants, KEY_RE, LOCATION_RE, OPERATIONS, matches, subset};
