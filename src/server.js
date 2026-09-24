#!/usr/bin/env node
'use strict';

// Skrynia HTTP server.
//
// Runtime invariant: exactly one server process per data directory. Storage
// mutations remain serialized by the Node.js event loop. A deployment request
// stays open through clone, build, activation, and cleanup, but its child
// processes run asynchronously so health and unrelated storage traffic remain
// responsive.

const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { normalizeBasePath } = require('./base-path.js');
const { createManagement, ManagementError } = require('./management.js');
const { createPush } = require('./push.js');
const { NS_RE, validNs, ensureDir, createShared } = require('./shared.js');

const {
  existsSync,
  readFileSync,
  writeFileSync,
  unlinkSync,
  statSync,
  readdirSync,
  realpathSync,
  renameSync,
  openSync,
  closeSync,
} = fs;
const { O_CREAT, O_EXCL, O_WRONLY } = fs.constants || {};

function createServer(opts) {
  opts = opts || {};

  const shared = createShared(opts.dataDir);
  const { RELEASES_DIR, STORAGE_DIR, STATE_DIR } = shared;

  const CLIENT_PATH = path.join(__dirname, 'client.js');
  const APP_BASE_PATH = normalizeBasePath(opts.appBasePath || process.env.SKRYNIA_APP_BASE_PATH || '/apps');
  const APP_DIR = opts.appDir || process.env.SKRYNIA_APP_DIR || '';
  const TOKEN = opts.token !== undefined ? String(opts.token) : String(process.env.SKRYNIA_TOKEN || '');
  const SKRYNIA_URL = opts.skryniaUrl !== undefined ? String(opts.skryniaUrl) : String(process.env.SKRYNIA_URL || '');
  if (!SKRYNIA_URL) throw new Error('SKRYNIA_URL is required');
  let publicUrl;
  try { publicUrl = new URL(SKRYNIA_URL); }
  catch { throw new Error('SKRYNIA_URL must be an absolute URL'); }
  if (publicUrl.protocol !== 'http:' && publicUrl.protocol !== 'https:') throw new Error('SKRYNIA_URL must use http or https');
  if (publicUrl.search || publicUrl.hash) throw new Error('SKRYNIA_URL must not contain query or fragment components');
  const HTTP_BASE_PATH = normalizeBasePath(publicUrl.pathname || '/');

  const MAX_KEY_LENGTH = parseInt(process.env.SKRYNIA_MAX_KEY_LENGTH || '256', 10);
  const MAX_OBJECT_SIZE = parseInt(process.env.SKRYNIA_MAX_OBJECT_SIZE || '10485760', 10);

  const push = createPush({
    dataDir: shared.dataDir,
    skryniaUrl: SKRYNIA_URL,
    pushTransport: opts.pushTransport,
    pushPollMs: opts.pushPollMs,
    pushSubject: opts.pushSubject,
    pushManual: opts.pushManual,
    pushSendTimeoutMs: opts.pushSendTimeoutMs,
    maxKeyLength: MAX_KEY_LENGTH,
    maxSubsPerNamespace: opts.pushMaxSubsPerNamespace,
    maxOutboxItems: opts.pushMaxOutboxItems,
  });

  const management = createManagement({
    dataDir: shared.dataDir,
    appBasePath: APP_BASE_PATH,
    appDir: APP_DIR,
    builderImage: opts.builderImage,
    push,
  });

  function json(res, status, body) {
    res.writeHead(status, {'Content-Type':'application/json'});
    res.end(JSON.stringify(body));
  }

  function safeKey(key) {
    if (typeof key !== 'string' || key.length === 0 || key.length > MAX_KEY_LENGTH) return null;
    if (key.includes('\0') || key.includes('..') || key.startsWith('/') || key.includes('//')) return null;
    if (key.includes('/') || key === '.' || key === '..') return null;
    return key;
  }

  function nsExists(ns) { return existsSync(shared.nsQuotaPath(ns)); }

  function requireNs(ns, res) {
    if (!nsExists(ns)) {
      json(res, 404, {error:'namespace_not_found'});
      return false;
    }
    return true;
  }

  function requireNsCreate(ns, res) {
    if (!nsExists(ns)) {
      json(res, 409, {error:'namespace_not_created', detail:'create the namespace through the ns/create endpoint first'});
      return false;
    }
    return true;
  }

  function generateCapability() { return crypto.randomBytes(32).toString('hex'); }
  function sha256hex(data) { return crypto.createHash('sha256').update(data).digest('hex'); }

  function timingSafeEqualHex(a, b) {
    if (typeof a !== 'string' || typeof b !== 'string' || a.length !== b.length) return false;
    const bufA = Buffer.from(a, 'hex');
    const bufB = Buffer.from(b, 'hex');
    if (bufA.length !== bufB.length) return false;
    return crypto.timingSafeEqual(bufA, bufB);
  }

  function timingSafeEqualString(a, b) {
    if (typeof a !== 'string' || typeof b !== 'string') return false;
    const bufA = Buffer.from(a);
    const bufB = Buffer.from(b);
    if (bufA.length !== bufB.length) return false;
    return crypto.timingSafeEqual(bufA, bufB);
  }

  function exclusiveCreate(filePath, data) {
    try {
      const fd = openSync(filePath, O_CREAT | O_EXCL | O_WRONLY);
      try {
        if (data && data.length > 0) fs.writeSync(fd, data, 0, data.length, null);
      } finally {
        closeSync(fd);
      }
      return true;
    } catch (e) {
      if (e.code === 'EEXIST') return false;
      throw e;
    }
  }

  function atomicReplace(filePath, data) {
    const tmp = filePath + '.tmp.' + process.pid;
    writeFileSync(tmp, data);
    renameSync(tmp, filePath);
  }

  function handleGet(ns, key, res) {
    const op = shared.nsObjPath(ns, key);
    if (!existsSync(op)) { res.writeHead(404, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'not_found'})); return; }
    const meta = JSON.parse(readFileSync(shared.nsMetaPath(ns, key), 'utf8'));
    const data = readFileSync(op);
    res.writeHead(200, {
      'Content-Type': meta.contentType || 'application/octet-stream',
      'ETag': '"' + sha256hex(data) + '"',
      'X-Skrynia-Mode': meta.mode,
      'X-Skrynia-Created': meta.created,
    });
    res.end(data);
  }

  function cleanupIncompleteCreate(ns, key) {
    for (const p of [shared.nsObjPath(ns, key), shared.nsCapPath(ns, key)]) {
      try { unlinkSync(p); } catch (e) { if (e.code !== 'ENOENT') throw e; }
    }
  }

  function handleCreate(ns, key, body, req, res) {
    if (!requireNsCreate(ns, res)) return;

    const mode = req.headers['x-skrynia-mode'] || 'capability-write';
    if (!['immutable','capability-write','public-write'].includes(mode)) return json(res, 400, {error:'invalid_mode'});
    if (body.length > MAX_OBJECT_SIZE) return json(res, 413, {error:'object_too_large'});

    const q = shared.recalcQuota(ns);
    if (q.count >= q.maxObjects) return json(res, 507, {error:'namespace_full',detail:'max_objects'});
    if (q.bytes + body.length > q.quotaBytes) return json(res, 507, {error:'namespace_full',detail:'quota_bytes'});

    ensureDir(shared.nsStorageDir(ns));

    // A .meta file is the visibility/commit marker. If data exists without
    // metadata, it is residue from an interrupted create and can be reclaimed.
    if (!existsSync(shared.nsMetaPath(ns, key)) && (existsSync(shared.nsObjPath(ns, key)) || existsSync(shared.nsCapPath(ns, key)))) cleanupIncompleteCreate(ns, key);

    if (existsSync(shared.nsMetaPath(ns, key))) return json(res, 409, {error:'already_exists'});

    // Durability ordering: persist outbox item(s) BEFORE the mutation becomes
    // committed/visible (.meta rename below). A crash between outbox
    // persistence and commit causes at most a spurious wake-up (acceptable);
    // a crash after commit always leaves recoverable outbox state. The whole
    // handler is synchronous, so no other mutation can interleave.
    try {
      push.enqueue(ns, key, 'create');
    } catch (e) {
      if (e.code === 'outbox_full') return json(res, 507, {error:'push_outbox_full', detail:'durable push outbox at capacity; retry after it drains'});
      console.error('skrynia push enqueue error:', e && e.message ? e.message : e);
      return json(res, 500, {error:'push_enqueue_failed'});
    }

    if (!exclusiveCreate(shared.nsObjPath(ns, key), body)) return json(res, 409, {error:'already_exists'});

    const now = new Date().toISOString();
    const meta = {
      mode,
      contentType: req.headers['content-type'] || 'application/octet-stream',
      created: now,
      size: body.length,
    };
    const response = { ok: true, mode };

    try {
      if (mode === 'capability-write') {
        const capability = generateCapability();
        writeFileSync(shared.nsCapPath(ns, key), sha256hex(capability));
        response.capability = capability;
      }
      const tmp = shared.nsMetaPath(ns, key) + '.tmp.' + process.pid;
      writeFileSync(tmp, JSON.stringify(meta, null, 2));
      renameSync(tmp, shared.nsMetaPath(ns, key));
    } catch (e) {
      cleanupIncompleteCreate(ns, key);
      throw e;
    }

    json(res, 201, response);
  }

  function handlePut(ns, key, body, req, res) {
    if (!requireNs(ns, res)) return;

    const mp = shared.nsMetaPath(ns, key);
    if (!existsSync(mp)) return json(res, 404, {error:'not_found'});
    const meta = JSON.parse(readFileSync(mp, 'utf8'));
    if (meta.mode === 'immutable') return json(res, 403, {error:'immutable'});
    if (meta.mode === 'capability-write') {
      const capability = req.headers['x-skrynia-capability'];
      if (!capability) return json(res, 403, {error:'capability_required'});
      const storedHash = readFileSync(shared.nsCapPath(ns, key), 'utf8');
      if (!timingSafeEqualHex(sha256hex(capability), storedHash)) return json(res, 403, {error:'invalid_capability'});
    }
    const ifMatch = req.headers['if-match'];
    if (ifMatch !== undefined) {
      const current = readFileSync(shared.nsObjPath(ns, key));
      const currentEtag = '"' + sha256hex(current) + '"';
      if (ifMatch !== currentEtag) return json(res, 412, {error:'etag_mismatch'});
    }
    if (body.length > MAX_OBJECT_SIZE) return json(res, 413, {error:'object_too_large'});

    const q = shared.recalcQuota(ns);
    const newBytes = q.bytes - meta.size + body.length;
    if (newBytes > q.quotaBytes) return json(res, 507, {error:'namespace_full',detail:'quota_bytes'});

    // Outbox before commit: the .dat replace below is the visibility point
    // for current-data readers, so notification state goes to disk first.
    try {
      push.enqueue(ns, key, 'replace');
    } catch (e) {
      if (e.code === 'outbox_full') return json(res, 507, {error:'push_outbox_full', detail:'durable push outbox at capacity; retry after it drains'});
      console.error('skrynia push enqueue error:', e && e.message ? e.message : e);
      return json(res, 500, {error:'push_enqueue_failed'});
    }

    atomicReplace(shared.nsObjPath(ns, key), body);
    meta.size = body.length;
    meta.contentType = req.headers['content-type'] || meta.contentType;
    meta.modified = new Date().toISOString();
    writeFileSync(mp, JSON.stringify(meta, null, 2));
    json(res, 200, {ok:true});
  }

  function handleDelete(ns, key, req, res) {
    if (!requireNs(ns, res)) return;

    const mp = shared.nsMetaPath(ns, key);
    if (!existsSync(mp)) return json(res, 404, {error:'not_found'});
    const meta = JSON.parse(readFileSync(mp, 'utf8'));
    if (meta.mode === 'immutable') return json(res, 403, {error:'immutable'});
    if (meta.mode === 'capability-write') {
      const capability = req.headers['x-skrynia-capability'];
      if (!capability) return json(res, 403, {error:'capability_required'});
      if (!timingSafeEqualHex(sha256hex(capability), readFileSync(shared.nsCapPath(ns, key), 'utf8'))) return json(res, 403, {error:'invalid_capability'});
    }

    // Outbox before commit: the .meta unlink below is the visibility point
    // for deletion, so notification state goes to disk first.
    try {
      push.enqueue(ns, key, 'delete');
    } catch (e) {
      if (e.code === 'outbox_full') return json(res, 507, {error:'push_outbox_full', detail:'durable push outbox at capacity; retry after it drains'});
      console.error('skrynia push enqueue error:', e && e.message ? e.message : e);
      return json(res, 500, {error:'push_enqueue_failed'});
    }

    unlinkSync(mp);
    try { unlinkSync(shared.nsObjPath(ns, key)); } catch (e) { if (e.code !== 'ENOENT') throw e; }
    try { unlinkSync(shared.nsCapPath(ns, key)); } catch (e) { if (e.code !== 'ENOENT') throw e; }
    json(res, 200, {ok:true});
  }

  function activeLink(ns) { return APP_DIR ? path.join(APP_DIR, ns) : shared.nsCurrentLink(ns); }

  function serveFile(filePath, res) {
    const types = {
      '.html':'text/html',
      '.js':'application/javascript',
      '.css':'text/css',
      '.json':'application/json',
      '.png':'image/png',
      '.svg':'image/svg+xml',
    };
    res.writeHead(200, {'Content-Type': types[path.extname(filePath)] || 'application/octet-stream'});
    res.end(readFileSync(filePath));
  }

  function serveApp(ns, res, urlPath) {
    const link = activeLink(ns);
    if (!existsSync(link)) { res.writeHead(503, {'Content-Type':'text/plain'}); res.end('Service unavailable'); return; }

    let appRoot;
    try { appRoot = realpathSync(link); }
    catch { res.writeHead(503, {'Content-Type':'text/plain'}); res.end('Service unavailable'); return; }

    if (urlPath === '/') urlPath = '/index.html';
    const safe = path.normalize(urlPath);
    if (safe.includes('..')) { res.writeHead(403); res.end('Forbidden'); return; }
    const filePath = path.join(appRoot, safe);
    if (!existsSync(filePath)) { res.writeHead(404); res.end('Not found'); return; }

    let resolved;
    try { resolved = realpathSync(filePath); }
    catch { resolved = null; }
    if (!resolved || (resolved !== appRoot && !resolved.startsWith(appRoot + path.sep))) {
      res.writeHead(403);
      res.end('Forbidden');
      return;
    }

    if (statSync(resolved).isDirectory()) {
      const index = path.join(resolved, 'index.html');
      if (!existsSync(index)) { res.writeHead(404); res.end('Not found'); return; }
      const resolvedIndex = realpathSync(index);
      if (resolvedIndex !== appRoot && !resolvedIndex.startsWith(appRoot + path.sep)) { res.writeHead(403); res.end('Forbidden'); return; }
      return serveFile(resolvedIndex, res);
    }
    serveFile(resolved, res);
  }

  function readBody(req, res, cb) {
    const chunks = [];
    let size = 0;
    let oversized = false;
    req.on('data', chunk => {
      if (oversized) return;
      size += chunk.length;
      if (size > MAX_OBJECT_SIZE) {
        oversized = true;
        json(res, 413, {error:'request_too_large'});
        return;
      }
      chunks.push(chunk);
    });
    req.on('end', () => { if (!oversized) cb(Buffer.concat(chunks)); });
  }

  function requireManagementToken(url, res) {
    if (!TOKEN) {
      json(res, 503, {error:'management_api_disabled', detail:'SKRYNIA_TOKEN is not configured'});
      return false;
    }
    const supplied = url.searchParams.get('token');
    if (!supplied || !timingSafeEqualString(supplied, TOKEN)) {
      json(res, 401, {error:'invalid_token'});
      return false;
    }
    return true;
  }

  const managementRoutes = {
    '/deploy': params => management.deploy(params),
    '/undeploy': params => management.undeploy(params),
    '/rollback': params => management.rollback(params),
    '/releases': params => management.releases(params),
    '/inspect': params => management.inspect(params),
    '/ns/create': params => management.namespaceCreate(params),
    '/ns/remove': params => management.namespaceRemove(params),
    '/ns/inspect': params => management.namespaceInspect(params),
    '/ns/list': () => management.namespaceList(),
    '/push/rules/set': params => management.pushRuleSet(params),
    '/push/rules/get': params => management.pushRuleGet(params),
    '/push/rules/list': params => management.pushRuleList(params),
    '/push/rules/remove': params => management.pushRuleRemove(params),
  };

  // --- Web Push public API ---

  const PUSH_JSON_MAX = 8192;

  function readPushJson(req, res, cb) {
    const chunks = [];
    let size = 0;
    let done = false;
    req.on('data', chunk => {
      if (done) return;
      size += chunk.length;
      if (size > PUSH_JSON_MAX) {
        done = true;
        json(res, 413, {error:'request_too_large'});
        return;
      }
      chunks.push(chunk);
    });
    req.on('end', () => {
      if (done) return;
      let body;
      try { body = JSON.parse(Buffer.concat(chunks).toString('utf8')); }
      catch { return json(res, 400, {error:'invalid_json'}); }
      cb(body);
    });
  }

  function handlePushRegister(url, req, res, body) {
    const ns = url.searchParams.get('namespace');
    const channel = url.searchParams.get('channel');
    if (!validNs(ns)) return json(res, 400, {error:'invalid_namespace'});
    if (!nsExists(ns)) return json(res, 404, {error:'namespace_not_found'});
    if (!push.validChannel(channel)) return json(res, 400, {error:'invalid_channel'});
    const endpoint = body && body.endpoint;
    const keys = body && body.keys;
    if (!push.validEndpoint(endpoint)) return json(res, 400, {error:'invalid_subscription', detail:'endpoint must be an https:// URL'});
    if (!push.validKeys(keys)) return json(res, 400, {error:'invalid_subscription', detail:'keys.p256dh and keys.auth are required'});
    try {
      const sub = push.createSub(ns, channel, endpoint, keys, pushCapability(req));
      return json(res, 201, { ok: true, id: sub.id, capability: sub.capability, deduped: sub.deduped });
    } catch (e) {
      if (e.code === 'capability_required') return json(res, 403, {error:'capability_required'});
      if (e.code === 'invalid_capability') return json(res, 403, {error:'invalid_capability'});
      if (e.code === 'channel_full') return json(res, 507, {error:'channel_full'});
      if (e.code === 'namespace_full') return json(res, 507, {error:'namespace_full', detail:'subscription quota'});
      throw e;
    }
  }

  function pushCapability(req) {
    // Like store object capabilities: bearer secret in a header, never in
    // the URL, so it does not leak into proxy logs or history.
    const cap = req.headers['x-skrynia-capability'];
    return typeof cap === 'string' && cap ? cap : null;
  }

  function handlePushUpdate(id, req, res, body) {
    const capability = pushCapability(req);
    if (!capability) return json(res, 403, {error:'capability_required'});
    const patch = {};
    if (body && body.endpoint !== undefined) {
      if (!push.validEndpoint(body.endpoint)) return json(res, 400, {error:'invalid_subscription', detail:'endpoint must be an https:// URL'});
      patch.endpoint = body.endpoint;
    }
    if (body && body.keys !== undefined) {
      if (!push.validKeys(body.keys)) return json(res, 400, {error:'invalid_subscription', detail:'keys.p256dh and keys.auth are required'});
      patch.keys = body.keys;
    }
    if (patch.endpoint === undefined && patch.keys === undefined) return json(res, 400, {error:'nothing_to_update'});
    try {
      push.updateSubAnywhere(id, capability, patch);
      return json(res, 200, {ok:true});
    } catch (e) {
      if (e.code === 'not_found') return json(res, 404, {error:'not_found'});
      if (e.code === 'invalid_capability') return json(res, 403, {error:'invalid_capability'});
      if (e.code === 'endpoint_in_use') return json(res, 409, {error:'endpoint_in_use'});
      throw e;
    }
  }

  function handlePushDelete(id, req, res) {
    const capability = pushCapability(req);
    if (!capability) return json(res, 403, {error:'capability_required'});
    try {
      push.removeSubAnywhere(id, capability);
      return json(res, 200, {ok:true});
    } catch (e) {
      if (e.code === 'not_found') return json(res, 404, {error:'not_found'});
      if (e.code === 'invalid_capability') return json(res, 403, {error:'invalid_capability'});
      throw e;
    }
  }

  function routePath(pathname) {
    if (HTTP_BASE_PATH === '/') return pathname;
    if (pathname === HTTP_BASE_PATH) return '/';
    if (!pathname.startsWith(HTTP_BASE_PATH + '/')) return null;
    return pathname.slice(HTTP_BASE_PATH.length);
  }

  function route(req, res) {
    let url;
    try { url = new URL(req.url, 'http://' + req.headers.host); }
    catch { res.writeHead(400); res.end('Bad request'); return; }
    const p = routePath(url.pathname);

    if (p === '/health') return json(res, 200, {ok:true});

    if (p === '/client/skrynia.js') {
      if (!existsSync(CLIENT_PATH)) { res.writeHead(404); res.end('Not found'); return; }
      res.writeHead(200, {'Content-Type':'application/javascript'});
      res.end(readFileSync(CLIENT_PATH));
      return;
    }

    if (p === '/push/vapid' && req.method === 'GET') {
      return json(res, 200, { publicKey: push.getPublicKey() });
    }

    if (p === '/push/subscriptions' && req.method === 'POST') {
      return readPushJson(req, res, body => handlePushRegister(url, req, res, body));
    }

    const pushSubMatch = p === null ? null : p.match(/^\/push\/subscriptions\/([0-9a-zA-Z]+)$/);
    if (pushSubMatch) {
      const subId = pushSubMatch[1];
      if (req.method === 'PUT') {
        return readPushJson(req, res, body => handlePushUpdate(subId, req, res, body));
      }
      if (req.method === 'DELETE') {
        return handlePushDelete(subId, req, res);
      }
      res.writeHead(405);
      res.end('Method not allowed');
      return;
    }

    const managementHandler = p === null ? undefined : managementRoutes[p];
    if (managementHandler) {
      if (req.method !== 'GET') { res.writeHead(405); res.end('Method not allowed'); return; }
      if (!requireManagementToken(url, res)) return;
      const params = Object.fromEntries(url.searchParams.entries());
      delete params.token;
      Promise.resolve()
        .then(() => managementHandler(params))
        .then(result => json(res, 200, result))
        .catch(e => {
          if (e instanceof ManagementError) return json(res, e.status, {error:e.code, detail:e.detail});
          console.error('skrynia management error:', e && e.stack ? e.stack : e);
          return json(res, 500, {error:'internal_error'});
        });
      return;
    }

    const storeMatch = p === null ? null : p.match(/^\/store\/([^/]+)\/(.+)$/);
    if (storeMatch) {
      let ns;
      let key;
      try {
        ns = decodeURIComponent(storeMatch[1]);
        key = safeKey(decodeURIComponent(storeMatch[2]));
      } catch {
        return json(res, 400, {error:'invalid_percent_encoding'});
      }
      if (!validNs(ns)) return json(res, 400, {error:'invalid_namespace'});
      if (!key) return json(res, 400, {error:'invalid_key'});
      if (req.method === 'GET') {
        if (!requireNs(ns, res)) return;
        return handleGet(ns, key, res);
      }
      if (req.method === 'DELETE') return handleDelete(ns, key, req, res);
      if (req.method === 'PUT' || req.method === 'POST') {
        return readBody(req, res, body => {
          if (req.method === 'POST') handleCreate(ns, key, body, req, res);
          else handlePut(ns, key, body, req, res);
        });
      }
      res.writeHead(405);
      res.end('Method not allowed');
      return;
    }

    const baseRe = APP_BASE_PATH.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const appMatch = url.pathname.match(new RegExp('^' + baseRe + '/([^/]+)(/.*)?$'));
    if (appMatch) {
      let ns;
      try { ns = decodeURIComponent(appMatch[1]); }
      catch { res.writeHead(400); res.end('Bad namespace'); return; }
      if (!validNs(ns)) { res.writeHead(400); res.end('Bad namespace'); return; }
      return serveApp(ns, res, appMatch[2] || '/');
    }

    res.writeHead(404);
    res.end('Not found');
  }

  ensureDir(RELEASES_DIR);
  ensureDir(STORAGE_DIR);
  ensureDir(STATE_DIR);
  push.start();

  const server = http.createServer(route);
  server.skrynia = { HTTP_BASE_PATH, SKRYNIA_URL, APP_BASE_PATH, APP_DIR, DATA_DIR: shared.dataDir, managementEnabled: Boolean(TOKEN), push };
  server.on('close', () => push.close());
  return server;
}

if (require.main === module) {
  const PORT = parseInt(process.env.SKRYNIA_PORT || '17380', 10);
  const server = createServer();
  server.listen(PORT, '127.0.0.1', () => {
    console.log('Skrynia server listening on 127.0.0.1:' + PORT);
  });
}

module.exports = { createServer };
