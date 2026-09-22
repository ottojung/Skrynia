'use strict';

// Durable generic Web Push facility driven by store mutations.
//
// Layout under the shared data dir (same process, same data dir):
//   STATE_DIR/_push/vapid.json            installation-owned VAPID keypair (0600)
//   STATE_DIR/_push/outbox/{oid}.json     durable at-least-once delivery outbox
//   STATE_DIR/{ns}/push-rules.json        mutation -> channel rules for ns
//   STATE_DIR/{ns}/push-subs/{id}.json    private subscription records for ns
//
// Push payload is exactly the channel name; never namespace/key/value data.

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { ensureDir, validNs } = require('./shared.js');

const CHANNEL_RE = /^[a-z0-9][a-z0-9_-]{0,63}$/;
const KINDS = ['create', 'replace', 'delete'];
const MAX_CHANNELS_PER_RULE = 8;
const MAX_SUBS_PER_CHANNEL = 500;
const MAX_SUBS_PER_NAMESPACE = 2000;
const MAX_OUTBOX_ITEMS = 10000;
const DEFAULT_SEND_TIMEOUT_MS = 10000;
const SUB_ID_BYTES = 16;
const CAP_BYTES = 32;

function validChannel(name) {
  return typeof name === 'string' && CHANNEL_RE.test(name);
}

function parseList(raw) {
  if (raw == null || raw === '') return [];
  return String(raw).split(',').map(s => s.trim()).filter(s => s.length > 0);
}

function randomHex(n) { return crypto.randomBytes(n).toString('hex'); }
function sha256hex(data) { return crypto.createHash('sha256').update(data).digest('hex'); }

function timingSafeEqualHex(a, b) {
  if (typeof a !== 'string' || typeof b !== 'string' || a.length !== b.length) return false;
  const bufA = Buffer.from(a, 'hex');
  const bufB = Buffer.from(b, 'hex');
  if (bufA.length !== bufB.length) return false;
  return crypto.timingSafeEqual(bufA, bufB);
}

// Durable atomic write for all private push state: temp file is created
// with mode 0600 (no 0644 window), fsynced before rename so the entry
// survives OS-level crashes, then the containing directory is fsynced so
// the rename itself is durable.
function atomicWrite(filePath, data) {
  ensureDir(path.dirname(filePath));
  const tmp = filePath + '.tmp.' + process.pid;
  const fd = fs.openSync(tmp, 'w', 0o600);
  try {
    fs.writeFileSync(fd, data);
    fs.fsyncSync(fd);
  } finally {
    fs.closeSync(fd);
  }
  fs.renameSync(tmp, filePath);
  fsyncDir(path.dirname(filePath));
}

function fsyncDir(dir) {
  // The target is Linux: real open/I/O/permission errors propagate. Only
  // explicitly known unsupported directory-fsync cases are tolerated.
  const fd = fs.openSync(dir, 'r');
  try {
    fs.fsyncSync(fd);
  } catch (e) {
    if (e.code !== 'EINVAL' && e.code !== 'ENOSYS') throw e;
  } finally {
    try { fs.closeSync(fd); } catch {}
  }
}

function backoffMs(attempts) {
  const ms = 1000 * Math.pow(2, Math.max(0, attempts - 1));
  return Math.min(ms, 30000);
}

function createPush(opts) {
  opts = opts || {};
  const dataDir = opts.dataDir || process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
  const stateDir = path.join(dataDir, 'state');
  const pushDir = path.join(stateDir, '_push');
  const outboxDir = path.join(pushDir, 'outbox');
  const vapidPath = path.join(pushDir, 'vapid.json');
  const subject = opts.pushSubject || process.env.SKRYNIA_PUSH_SUBJECT || opts.skryniaUrl || process.env.SKRYNIA_URL || 'https://skrynia.local/';
  const getWebPush = () => opts.webPushLib || require('web-push');

  let transport = opts.pushTransport || null;
  let pumping = false;
  let timer = null;
  const pollMs = opts.pushPollMs != null ? Number(opts.pushPollMs) : 1000;
  // Manual mode (tests only): no automatic pumps; tests drive pumpOnce().
  const manual = Boolean(opts.pushManual);
  const maxSubsPerNamespace = opts.maxSubsPerNamespace != null ? Number(opts.maxSubsPerNamespace) : MAX_SUBS_PER_NAMESPACE;
  const maxOutboxItems = opts.maxOutboxItems != null ? Number(opts.maxOutboxItems) : MAX_OUTBOX_ITEMS;
  const maxKeyLength = opts.maxKeyLength != null ? Number(opts.maxKeyLength) : 256;
  const sendTimeoutMs = opts.pushSendTimeoutMs != null
    ? Number(opts.pushSendTimeoutMs)
    : Number(process.env.SKRYNIA_PUSH_SEND_TIMEOUT_MS || DEFAULT_SEND_TIMEOUT_MS);
  if (!Number.isFinite(sendTimeoutMs) || sendTimeoutMs <= 0) throw new Error('push send timeout must be a positive number');

  // --- paths ---

  function rulesPath(ns) { return path.join(stateDir, ns, 'push-rules.json'); }
  function subsDir(ns) { return path.join(stateDir, ns, 'push-subs'); }
  function subPath(ns, id) { return path.join(subsDir(ns), id + '.json'); }

  // --- VAPID (installation-owned, stable across restarts) ---

  function getVapidKeys() {
    try {
      const raw = JSON.parse(fs.readFileSync(vapidPath, 'utf8'));
      if (raw && typeof raw.publicKey === 'string' && typeof raw.privateKey === 'string' && raw.publicKey && raw.privateKey) {
        return raw;
      }
    } catch (e) {
      if (e.code !== 'ENOENT') throw e;
    }
    ensureDir(pushDir);
    // The pinned web-push library is mandatory for VAPID generation: if it
    // is unavailable, startup fails loudly rather than minting keys with
    // hand-rolled cryptography.
    const webpush = getWebPush();
    const keys = webpush.generateVAPIDKeys();
    atomicWrite(vapidPath, JSON.stringify(keys, null, 2));
    return keys;
  }

  function getPublicKey() { return getVapidKeys().publicKey; }

  // --- rules (managed through the existing management authority) ---

  function loadRules(ns) {
    try {
      const raw = JSON.parse(fs.readFileSync(rulesPath(ns), 'utf8'));
      if (raw && Array.isArray(raw.rules)) return raw.rules;
      return [];
    } catch (e) {
      if (e.code === 'ENOENT') return [];
      throw e;
    }
  }

  function saveRules(ns, rules) {
    ensureDir(path.dirname(rulesPath(ns)));
    atomicWrite(rulesPath(ns), JSON.stringify({ rules }, null, 2));
  }

  function validateRuleParts(key, kinds, channels) {
    if (typeof key !== 'string' || key.length === 0 || key.length > maxKeyLength) return 'key must be a non-empty string up to ' + maxKeyLength + ' chars';
    if (key.includes('\0') || key.includes('/') || key === '.' || key === '..' || key.includes('..')) return 'key must be an exact plain object key';
    for (const k of kinds) {
      if (!KINDS.includes(k)) return 'kind must be one of ' + KINDS.join(',');
    }
    if (!kinds.length) return 'at least one kind is required';
    if (!channels.length) return 'at least one channel is required';
    if (channels.length > MAX_CHANNELS_PER_RULE) return 'at most ' + MAX_CHANNELS_PER_RULE + ' channels per rule';
    const seen = new Set();
    for (const c of channels) {
      if (!validChannel(c)) return 'invalid channel name: ' + c;
      if (seen.has(c)) return 'duplicate channel: ' + c;
      seen.add(c);
    }
    return null;
  }

  function setRule(ns, key, kinds, channels) {
    const rules = loadRules(ns);
    const idx = rules.findIndex(r => r.key === key);
    const entry = { key, kinds: [...kinds].sort(), channels: [...channels] };
    if (idx >= 0) rules[idx] = entry;
    else rules.push(entry);
    rules.sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
    saveRules(ns, rules);
    return entry;
  }

  function getRule(ns, key) {
    return loadRules(ns).find(r => r.key === key) || null;
  }

  function removeRule(ns, key) {
    const rules = loadRules(ns);
    const next = rules.filter(r => r.key !== key);
    if (next.length === rules.length) return false;
    saveRules(ns, next);
    return true;
  }

  function matchChannels(ns, key, kind) {
    const rule = getRule(ns, key);
    if (!rule || !rule.kinds.includes(kind)) return [];
    return [...rule.channels];
  }

  // --- subscriptions (private records, never enumerated publicly) ---

  function validEndpoint(endpoint) {
    if (typeof endpoint !== 'string' || endpoint.length === 0 || endpoint.length > 2048) return false;
    let u;
    try { u = new URL(endpoint); } catch { return false; }
    if (u.protocol === 'https:') return true;
    if (u.protocol === 'http:') {
      const h = u.hostname;
      return h === 'localhost' || h === '127.0.0.1' || h === '[::1]' || h === '::1';
    }
    return false;
  }

  function validKeys(keys) {
    if (!keys || typeof keys !== 'object') return false;
    const { p256dh, auth } = keys;
    if (typeof p256dh !== 'string' || typeof auth !== 'string') return false;
    if (!p256dh || !auth || p256dh.length > 256 || auth.length > 256) return false;
    return true;
  }

  function readSubFile(file) {
    try { return JSON.parse(fs.readFileSync(file, 'utf8')); }
    catch { return null; }
  }

  function allSubs(ns) {
    const dir = subsDir(ns);
    let files = [];
    try { files = fs.readdirSync(dir).filter(f => f.endsWith('.json')); }
    catch (e) { if (e.code !== 'ENOENT') throw e; return []; }
    const out = [];
    for (const f of files) {
      const sub = readSubFile(path.join(dir, f));
      if (sub && sub.id && sub.endpoint) out.push(sub);
    }
    return out;
  }

  function channelCount(ns, channel) {
    return allSubs(ns).filter(s => s.channel === channel).length;
  }

  function findSub(ns, id) {
    if (!/^[0-9a-f]{32}$/.test(id || '')) return null;
    const sub = readSubFile(subPath(ns, id));
    return sub && sub.id === id ? sub : null;
  }

  // Searches every namespace dir for the id. Ids are unguessable 128-bit
  // hex; this avoids leaking namespace existence via per-ns endpoints.
  function findSubAnywhere(id) {
    if (!/^[0-9a-f]{32}$/.test(id || '')) return null;
    let dirs = [];
    try { dirs = fs.readdirSync(stateDir, { withFileTypes: true }).filter(d => d.isDirectory()).map(d => d.name); }
    catch { return null; }
    for (const ns of dirs) {
      if (!validNs(ns)) continue;
      const sub = readSubFile(path.join(subsDir(ns), id + '.json'));
      if (sub && sub.id === id) return sub;
    }
    return null;
  }

  function createSub(ns, channel, endpoint, keys, existingCapability) {
    const subs = allSubs(ns);
    const existing = subs.find(s => s.channel === channel && s.endpoint === endpoint);
    if (existing) {
      if (!existingCapability) {
        const err = new Error('capability_required');
        err.code = 'capability_required';
        throw err;
      }
      if (!checkCap(existing, existingCapability)) {
        const err = new Error('invalid_capability');
        err.code = 'invalid_capability';
        throw err;
      }
      const capability = randomHex(CAP_BYTES);
      existing.keys = { p256dh: keys.p256dh, auth: keys.auth };
      existing.capHash = sha256hex(capability);
      atomicWrite(subPath(ns, existing.id), JSON.stringify(existing, null, 2));
      return { id: existing.id, capability, deduped: true };
    }
    // Durable per-namespace total cap: arbitrary channel names cannot be
    // used to grow subscriptions without bound.
    if (subs.length >= maxSubsPerNamespace) {
      const err = new Error('namespace_full');
      err.code = 'namespace_full';
      throw err;
    }
    if (channelCount(ns, channel) >= MAX_SUBS_PER_CHANNEL) {
      const err = new Error('channel_full');
      err.code = 'channel_full';
      throw err;
    }
    const id = randomHex(SUB_ID_BYTES);
    const capability = randomHex(CAP_BYTES);
    const record = {
      id,
      ns,
      channel,
      endpoint,
      keys: { p256dh: keys.p256dh, auth: keys.auth },
      capHash: sha256hex(capability),
      created: new Date().toISOString(),
    };
    ensureDir(subsDir(ns));
    atomicWrite(subPath(ns, id), JSON.stringify(record, null, 2));
    return { id, capability, deduped: false };
  }

  function checkCap(sub, capability) {
    if (!sub || typeof capability !== 'string') return false;
    return timingSafeEqualHex(sha256hex(capability), sub.capHash);
  }

  function updateSub(ns, id, capability, patch) {
    const sub = findSub(ns, id);
    if (!sub) {
      const err = new Error('not_found');
      err.code = 'not_found';
      throw err;
    }
    if (!checkCap(sub, capability)) {
      const err = new Error('invalid_capability');
      err.code = 'invalid_capability';
      throw err;
    }
    if (patch.endpoint !== undefined) sub.endpoint = patch.endpoint;
    if (patch.keys !== undefined) sub.keys = { p256dh: patch.keys.p256dh, auth: patch.keys.auth };
    atomicWrite(subPath(ns, id), JSON.stringify(sub, null, 2));
    return sub;
  }

  function removeSub(ns, id, capability) {
    const sub = findSub(ns, id);
    if (!sub) {
      const err = new Error('not_found');
      err.code = 'not_found';
      throw err;
    }
    if (!checkCap(sub, capability)) {
      const err = new Error('invalid_capability');
      err.code = 'invalid_capability';
      throw err;
    }
    try { fs.unlinkSync(subPath(ns, id)); } catch (e) { if (e.code !== 'ENOENT') throw e; }
    return true;
  }

  function removeSubRecord(ns, id) {
    try { fs.unlinkSync(subPath(ns, id)); } catch (e) { if (e.code !== 'ENOENT') throw e; }
  }

  function removeSubRecordIfUnchanged(ns, snapshot) {
    const current = findSub(ns, snapshot.id);
    if (!current) return;
    if (
      current.endpoint === snapshot.endpoint &&
      current.capHash === snapshot.capHash &&
      current.keys &&
      snapshot.keys &&
      current.keys.p256dh === snapshot.keys.p256dh &&
      current.keys.auth === snapshot.keys.auth
    ) {
      removeSubRecord(ns, snapshot.id);
    }
  }

  function notFoundError() {
    const err = new Error('not_found');
    err.code = 'not_found';
    return err;
  }

  function capError() {
    const err = new Error('invalid_capability');
    err.code = 'invalid_capability';
    return err;
  }

  // Namespace-blind variants for the public update/delete endpoints: the URL
  // carries no namespace so existence of namespaces is never leaked; ids are
  // unguessable 128-bit hex and the capability is bearer authority.
  function updateSubAnywhere(id, capability, patch) {
    const sub = findSubAnywhere(id);
    if (!sub) throw notFoundError();
    if (!checkCap(sub, capability)) throw capError();
    if (patch.endpoint !== undefined) {
      const duplicate = allSubs(sub.ns).find(s => s.id !== id && s.channel === sub.channel && s.endpoint === patch.endpoint);
      if (duplicate) {
        const err = new Error('endpoint_in_use');
        err.code = 'endpoint_in_use';
        throw err;
      }
      sub.endpoint = patch.endpoint;
    }
    if (patch.keys !== undefined) sub.keys = { p256dh: patch.keys.p256dh, auth: patch.keys.auth };
    atomicWrite(subPath(sub.ns, id), JSON.stringify(sub, null, 2));
    return sub;
  }

  function removeSubAnywhere(id, capability) {
    const sub = findSubAnywhere(id);
    if (!sub) throw notFoundError();
    if (!checkCap(sub, capability)) throw capError();
    removeSubRecord(sub.ns, id);
    return true;
  }

  // --- durable outbox: one entry per (mutation x channel) ---

  function outboxCount() {
    try { return fs.readdirSync(outboxDir).filter(f => f.endsWith('.json')).length; }
    catch (e) { if (e.code === 'ENOENT') return 0; throw e; }
  }

  function enqueue(ns, key, kind) {
    const channels = matchChannels(ns, key, kind);
    if (!channels.length) return [];
    ensureDir(outboxDir);
    // Backpressure before commit: capacity for ALL matching channels is
    // verified synchronously before any entry is written, so a configured
    // mutation never commits without complete durable notification state.
    if (outboxCount() + channels.length > maxOutboxItems) {
      const err = new Error('outbox_full');
      err.code = 'outbox_full';
      throw err;
    }
    const now = new Date().toISOString();
    const ids = [];
    for (const channel of channels) {
      const oid = Date.now().toString(36) + '-' + randomHex(8);
      const entry = { id: oid, ns, key, kind, channel, attempts: 0, nextAt: now, created: now };
      atomicWrite(path.join(outboxDir, oid + '.json'), JSON.stringify(entry, null, 2));
      ids.push(oid);
    }
    schedule();
    return ids;
  }

  function listOutbox() {
    let files = [];
    try { files = fs.readdirSync(outboxDir).filter(f => f.endsWith('.json')).sort(); }
    catch (e) { if (e.code !== 'ENOENT') throw e; return []; }
    const out = [];
    for (const f of files) {
      const p = path.join(outboxDir, f);
      const entry = readSubFile(p);
      if (entry && entry.id) out.push({ file: p, entry });
      else { try { fs.unlinkSync(p); } catch {} }
    }
    return out;
  }

  function defaultTransport() {
    const webpush = getWebPush();
    const keys = getVapidKeys();
    return async (sub, channel) => {
      await webpush.sendNotification(
        { endpoint: sub.endpoint, keys: sub.keys },
        channel,
        {
          timeout: sendTimeoutMs,
          vapidDetails: { subject, publicKey: keys.publicKey, privateKey: keys.privateKey },
        }
      );
    };
  }

  function isGoneError(err) {
    const status = err && (err.statusCode || err.status);
    return status === 404 || status === 410;
  }

  async function sendBounded(send, sub, channel) {
    let timer;
    const timeout = new Promise((_, reject) => {
      timer = setTimeout(() => {
        const err = new Error('push delivery timed out');
        err.code = 'push_timeout';
        reject(err);
      }, sendTimeoutMs);
    });
    try {
      return await Promise.race([
        Promise.resolve().then(() => send(sub, channel)),
        timeout,
      ]);
    } finally {
      clearTimeout(timer);
    }
  }

  // At-least-once delivery. Safe to run repeatedly: a successfully
  // delivered entry is removed, and duplicates are acceptable because the
  // payload is only a channel-name wake-up (clients reread store state).
  // Web Push sends are NOT idempotent; the outbox entry is the deduplicating
  // record, and it is removed only after every subscription send succeeds
  // (or the subscription proves expired). Transient failures retry
  // indefinitely with capped exponential backoff; storage is bounded by the
  // outbox capacity check in enqueue(), never by dropping entries here.
  async function pumpOnce(opts) {
    opts = opts || {};
    const now = Date.now();
    const items = listOutbox();
    let delivered = 0;
    for (const { file, entry } of items) {
      let fresh;
      try { fresh = JSON.parse(fs.readFileSync(file, 'utf8')); }
      catch { continue; }
      if (!opts.ignoreBackoff && fresh.nextAt && Date.parse(fresh.nextAt) > now) continue;
      const subs = allSubs(fresh.ns).filter(s => s.channel === fresh.channel);
      if (!subs.length) {
        try { fs.unlinkSync(file); } catch {}
        continue;
      }
      const send = transport || defaultTransport();
      let failed = false;
      for (const sub of subs) {
        try {
          await sendBounded(send, { endpoint: sub.endpoint, keys: sub.keys }, fresh.channel);
          delivered++;
        } catch (err) {
          if (isGoneError(err)) {
            // Delivery awaited external I/O, so the registration may have
            // changed while the send was in flight. Never delete a newer
            // endpoint because an older endpoint returned 404/410.
            removeSubRecordIfUnchanged(sub.ns, sub);
          } else {
            failed = true; // one broken subscription never blocks healthy ones
          }
        }
      }
      if (!failed) {
        try { fs.unlinkSync(file); } catch {}
      } else {
        // Transient failure: retry indefinitely with capped exponential
        // backoff. Entries are never dropped here; see enqueue() capacity.
        fresh.attempts = (fresh.attempts || 0) + 1;
        fresh.nextAt = new Date(Date.now() + backoffMs(fresh.attempts)).toISOString();
        try { atomicWrite(file, JSON.stringify(fresh, null, 2)); } catch {}
      }
    }
    return { delivered };
  }

  function schedule() {
    if (manual || timer) return;
    setImmediate(() => { pumpTick(); });
  }

  async function pumpTick() {
    if (pumping) return;
    pumping = true;
    try { await pumpOnce(); }
    catch (e) { console.error('skrynia push pump error:', e && e.message ? e.message : e); }
    finally { pumping = false; }
  }

  function start() {
    ensureDir(outboxDir);
    getVapidKeys();
    if (manual) return;
    pumpTick(); // startup recovery: redeliver anything durable in the outbox
    if (pollMs > 0) {
      timer = setInterval(pumpTick, pollMs);
      if (timer.unref) timer.unref();
    }
  }

  function close() {
    if (timer) { clearInterval(timer); timer = null; }
  }

  function setTransport(fn) { transport = fn; }

  return {
    getPublicKey,
    getVapidKeys,
    loadRules,
    setRule,
    getRule,
    removeRule,
    matchChannels,
    validateRuleParts,
    parseList,
    validChannel,
    validEndpoint,
    validKeys,
    createSub,
    updateSub,
    removeSub,
    updateSubAnywhere,
    removeSubAnywhere,
    findSub,
    findSubAnywhere,
    allSubs,
    enqueue,
    listOutbox,
    outboxCount,
    pumpOnce,
    setTransport,
    start,
    close,
    outboxDir,
  };
}

module.exports = { createPush, CHANNEL_RE, KINDS, MAX_SUBS_PER_CHANNEL, MAX_SUBS_PER_NAMESPACE, MAX_OUTBOX_ITEMS };
