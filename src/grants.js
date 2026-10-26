'use strict';

// Grant authority and persistent resource accounting. One process owns each
// Skrynia data directory; every public mutation is synchronous and serialized.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { ensureDir, validNs } = require('./shared.js');

const DAY = 24 * 60 * 60 * 1000;
// Fixed upper-bound accounting for bounded private records. This prevents a
// tiny grant from minting unlimited empty descendants or object metadata.
const GRANT_COST = 2048;
const OBJECT_COST = 512;
const PUSH_SUB_COST = 4096;
const EXTRA_USER_COST = 128;
const ID_RE = /^[a-f0-9]{32}$/;

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
function boundedInteger(v) {
  return Number.isSafeInteger(v) && v > 0;
}
function createGrants(shared) {
  const { dataDir } = shared;
  const stateDir = path.join(dataDir, 'state');
  const pendingDir = path.join(stateDir, '_grant-deletions');

  function dir(ns) { return path.join(stateDir, ns, 'grants'); }
  function recordPath(ns, id) { return path.join(dir(ns), id + '.json'); }
  function atomicJson(p, obj) {
    ensureDir(path.dirname(p), 0o700);
    const tmp = p + '.' + crypto.randomBytes(12).toString('hex') + '.tmp';
    try {
      fs.writeFileSync(tmp, JSON.stringify(obj), { mode: 0o600 });
      fs.renameSync(tmp, p);
    } finally {
      try { fs.unlinkSync(tmp); } catch (e) { if (e.code !== 'ENOENT') throw e; }
    }
  }
  function all(ns) {
    if (!validNs(ns)) throw fault(400, 'invalid_namespace');
    if (!fs.existsSync(dir(ns))) return [];
    return fs.readdirSync(dir(ns)).filter(x => ID_RE.test(x.replace(/\.json$/, '')) && x.endsWith('.json'))
      .map(x => JSON.parse(fs.readFileSync(path.join(dir(ns), x), 'utf8')));
  }
  function find(ns, id) {
    if (!ID_RE.test(id || '')) throw fault(400, 'invalid_grant');
    const g = all(ns).find(x => x.id === id);
    if (!g) throw fault(404, 'grant_not_found');
    return g;
  }
  function root(ns) { return all(ns).find(g => g.parent === null) || null; }
  function validateTree(grants) {
    const map = new Map(grants.map(g => [g.id, g]));
    for (const g of grants) {
      const visited = new Set([g.id]);
      let cur = g;
      while (cur.parent) {
        if (!map.has(cur.parent) || visited.has(cur.parent)) throw new Error('corrupt grant ancestry');
        visited.add(cur.parent);
        cur = map.get(cur.parent);
      }
    }
    return map;
  }
  function isDescendant(map, child, ancestor) {
    let g = map.get(child);
    while (g) {
      if (g.id === ancestor) return true;
      g = g.parent ? map.get(g.parent) : null;
    }
    return false;
  }
  function requireUser(ns, secret, id) {
    if (typeof secret !== 'string') throw fault(401, 'grant_user_token_required');
    const h = digest(secret);
    const g = all(ns).find(x => x.userHashes.some(u => same(u, h)) && (!id || x.id === id));
    if (!g) throw fault(403, 'invalid_grant_user_token');
    return g;
  }
  function requireIssuer(ns, id, secret) {
    const g = find(ns, id);
    if (typeof secret !== 'string' || !same(g.issuerHash, digest(secret))) {
      throw fault(403, 'invalid_grant_issuer_token');
    }
    return g;
  }
  function objectEntries(ns) {
    const metaDir = shared.nsStoreMetaDir(ns);
    if (!fs.existsSync(metaDir)) return [];
    return fs.readdirSync(metaDir).filter(x => x.endsWith('.json')).map(x => {
      const key = x.slice(0, -5);
      const meta = JSON.parse(fs.readFileSync(path.join(metaDir, x), 'utf8'));
      const file = shared.nsObjPath(ns, key);
      if (!fs.existsSync(file)) return null; // tombstone for deleted object
      if (!meta.grantId) throw new Error('unsupported old object schema: ' + ns + '/' + key);
      return { key, grantId: meta.grantId, bytes: fs.statSync(file).size };
    }).filter(Boolean);
  }
  function subscriptionEntries(ns) {
    const p = path.join(stateDir, ns, 'push-subs');
    if (!fs.existsSync(p)) return [];
    return fs.readdirSync(p).filter(x => x.endsWith('.json')).map(x => ({
      file: path.join(p, x),
      data: JSON.parse(fs.readFileSync(path.join(p, x), 'utf8')),
    }));
  }
  function usage(ns, grantId) {
    const grants = all(ns), map = validateTree(grants);
    let bytes = 0, objects = 0;
    for (const g of grants) if (isDescendant(map, g.id, grantId)) {
      bytes += GRANT_COST + (g.userHashes.length - 1) * EXTRA_USER_COST;
    }
    for (const o of objectEntries(ns)) {
      if (!map.has(o.grantId)) throw new Error('orphaned object: ' + ns + '/' + o.key);
      if (isDescendant(map, o.grantId, grantId)) {
        bytes += o.bytes + OBJECT_COST;
        objects++;
      }
    }
    for (const sub of subscriptionEntries(ns)) {
      if (!sub.data.grantId) throw new Error('unsupported old push subscription schema');
      if (!map.has(sub.data.grantId)) throw new Error('orphaned push subscription');
      if (isDescendant(map, sub.data.grantId, grantId)) {
        bytes += PUSH_SUB_COST;
        objects++;
      }
    }
    return { bytes, objects };
  }
  function assertCapacity(ns, grantId, extraBytes, extraObjects) {
    const map = validateTree(all(ns));
    let g = map.get(grantId);
    if (!g) throw fault(404, 'grant_not_found');
    while (g) {
      const u = usage(ns, g.id);
      if (u.bytes + extraBytes > g.maxBytes || u.objects + extraObjects > g.maxObjects) {
        throw fault(507, 'grant_quota_exceeded');
      }
      g = g.parent ? map.get(g.parent) : null;
    }
  }
  function makeRecord(ns, parent, maxBytes, maxObjects) {
    const issuerToken = token(), userToken = token();
    const now = Date.now();
    const id = crypto.randomBytes(16).toString('hex');
    const record = { id, namespace: ns, parent, maxBytes, maxObjects,
      issuerHash: digest(issuerToken), userHashes: [digest(userToken)],
      windowStart: now, nextCheckAt: now + DAY, keepAt: null, releaseAt: null };
    if (Buffer.byteLength(JSON.stringify(record)) > GRANT_COST) throw new Error('grant record cost exceeded');
    atomicJson(recordPath(ns, id), record);
    return { id, issuerToken, userToken, maxBytes, maxObjects, nextCheckAt: record.nextCheckAt };
  }
  function createRoot(ns, maxBytes, maxObjects) {
    if (!validNs(ns) || !boundedInteger(maxBytes) || !boundedInteger(maxObjects) || maxBytes < GRANT_COST) {
      throw fault(400, 'invalid_grant_limits');
    }
    if (root(ns)) return { created: false, id: root(ns).id };
    if (all(ns).length || objectEntries(ns).length) throw new Error('unsupported old namespace data');
    const out = makeRecord(ns, null, maxBytes, maxObjects);
    return { ...out, created: true };
  }
  function createChild(ns, secret, maxBytes, maxObjects) {
    sweep();
    const parent = requireUser(ns, secret);
    if (!boundedInteger(maxBytes) || !boundedInteger(maxObjects) || maxBytes < GRANT_COST) {
      throw fault(400, 'invalid_grant_limits');
    }
    assertCapacity(ns, parent.id, GRANT_COST, 0);
    return makeRecord(ns, parent.id, maxBytes, maxObjects);
  }
  function accountCreate(ns, secret, length) {
    sweep();
    const g = requireUser(ns, secret);
    assertCapacity(ns, g.id, length + OBJECT_COST, 1);
    return g.id;
  }
  function accountSubscription(ns, secret) {
    return accountCreate(ns, secret, PUSH_SUB_COST - OBJECT_COST);
  }
  function accountReplace(ns, grantId, previousSize, nextSize) {
    sweep();
    find(ns, grantId);
    if (nextSize > previousSize) assertCapacity(ns, grantId, nextSize - previousSize, 0);
  }
  function createUser(ns, id, secret) {
    sweep();
    const g = requireUser(ns, secret, id);
    assertCapacity(ns, id, EXTRA_USER_COST, 0);
    const userToken = token();
    g.userHashes.push(digest(userToken));
    const cost = GRANT_COST + (g.userHashes.length - 1) * EXTRA_USER_COST;
    if (Buffer.byteLength(JSON.stringify(g)) > cost) throw new Error('grant metadata exceeds charged cost');
    atomicJson(recordPath(ns, id), g);
    return {grantId: id, userToken};
  }
  function confirmation(ns, id, secret, need) {
    sweep();
    const g = requireUser(ns, secret, id);
    if (typeof need !== 'boolean') throw fault(400, 'invalid_confirmation');
    const now = Date.now();
    if (need) g.keepAt = now;
    else g.releaseAt = now;
    atomicJson(recordPath(ns, id), g);
    return { ok: true, grantId: id, need, nextCheckAt: g.nextCheckAt };
  }
  function deleteData(ns, ids) {
    const owned = new Set(ids);
    for (const sub of subscriptionEntries(ns)) {
      if (owned.has(sub.data.grantId)) fs.unlinkSync(sub.file);
    }
    for (const obj of objectEntries(ns)) {
      if (!owned.has(obj.grantId)) continue;
      fs.unlinkSync(shared.nsObjPath(ns, obj.key));
      fs.unlinkSync(shared.nsMetaPath(ns, obj.key));
    }
  }
  function runDeletion(ns, id, action, ids) {
    // Persist the full original subtree before deleting anything. Recovery
    // cannot reconstruct it from grant records after some have been removed.
    const marker = path.join(pendingDir, id + '.json');
    if (!fs.existsSync(marker)) atomicJson(marker, {ns, id, action, ids});
    const work = JSON.parse(fs.readFileSync(marker, 'utf8'));
    deleteData(work.ns, work.ids);
    for (const descendantId of work.ids) {
      if (descendantId !== work.id || work.action === 'revoke') {
        try { fs.unlinkSync(recordPath(work.ns, descendantId)); }
        catch (e) { if (e.code !== 'ENOENT') throw e; }
      }
    }
    if (work.action === 'clean' && fs.existsSync(recordPath(work.ns, work.id))) {
      const g = find(work.ns, work.id), now = Date.now();
      g.windowStart = now;
      g.nextCheckAt = now + DAY;
      g.keepAt = null;
      g.releaseAt = null;
      atomicJson(recordPath(work.ns, work.id), g);
    }
    fs.unlinkSync(marker);
  }
  function removeSubtree(ns, id, action) {
    const grants = all(ns), map = validateTree(grants);
    if (!map.has(id)) throw fault(404, 'grant_not_found');
    const ids = grants.filter(g => isDescendant(map, g.id, id)).map(g => g.id);
    runDeletion(ns, id, action, ids);
  }
  function issuerAction(ns, id, secret, action) {
    sweep();
    requireIssuer(ns, id, secret);
    removeSubtree(ns, id, action);
    return {ok: true, grantId: id, action};
  }
  function sweep(now = Date.now()) {
    if (!fs.existsSync(stateDir)) return;
    for (const ns of fs.readdirSync(stateDir).filter(validNs)) {
      let grants = all(ns);
      if (!grants.length) continue;
      const map = validateTree(grants);
      const due = grants.filter(g => g.nextCheckAt <= now).sort((a,b) => {
        const depth = x => { let n=0; while(x.parent){n++;x=map.get(x.parent);} return n; };
        return depth(a)-depth(b);
      });
      for (const g of due) {
        if (!fs.existsSync(recordPath(ns, g.id))) continue;
        // All descendants can protect an ancestor. Positive confirmations do
        // not propagate sideways or downward. Negative confirmations are local.
        const keep = grants.some(h => isDescendant(map, h.id, g.id) &&
          h.keepAt !== null && h.keepAt >= g.windowStart && h.keepAt < g.nextCheckAt);
        const release = g.releaseAt !== null && g.releaseAt >= g.windowStart && g.releaseAt < g.nextCheckAt;
        if (release && !keep) {
          removeSubtree(ns, g.id, 'clean');
          grants = all(ns);
          continue;
        }
        const fresh = find(ns, g.id);
        fresh.windowStart = now;
        fresh.nextCheckAt = now + DAY;
        fresh.releaseAt = null;
        // Retain the latest keep timestamp for ancestors with later checks.
        atomicJson(recordPath(ns, g.id), fresh);
      }
    }
  }
  function recover() {
    ensureDir(pendingDir, 0o700);
    for (const name of fs.readdirSync(pendingDir).filter(x => x.endsWith('.json'))) {
      const op = JSON.parse(fs.readFileSync(path.join(pendingDir, name), 'utf8'));
      runDeletion(op.ns, op.id, op.action, op.ids);
    }
    // No compatibility or migration: refuse old persisted namespaces.
    if (!fs.existsSync(stateDir)) return;
    for (const ns of fs.readdirSync(stateDir).filter(validNs)) {
      const files = objectEntries(ns);
      if ((files.length || fs.existsSync(shared.nsQuotaPath(ns))) && !root(ns)) {
        throw new Error('unsupported old namespace schema: ' + ns);
      }
      if (fs.existsSync(shared.nsQuotaPath(ns))) throw new Error('old quota schema is unsupported: ' + ns);
      validateTree(all(ns));
    }
  }
  function inspect(ns, id, secret) {
    sweep();
    requireUser(ns, secret, id);
    const g = find(ns, id);
    const {issuerHash, userHashes, ...safe} = g;
    safe.userCount = userHashes.length;
    return {...safe, usage: usage(ns, id)};
  }
  return { DAY, GRANT_COST, OBJECT_COST, root, createRoot, createChild, accountCreate,
    accountReplace, accountSubscription, requireUser, createUser, confirmation, issuerAction, inspect, usage,
    recover, sweep, assertCapacity, fault };
}
module.exports = {createGrants};
