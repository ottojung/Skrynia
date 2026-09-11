#!/usr/bin/env node
'use strict';

// Skrynia HTTP server.
//
// Runtime invariant: exactly one server process per data directory. Storage and
// management mutations are serialized by the Node.js event loop. Deployment is
// intentionally synchronous: the HTTP request remains open while git, build,
// release activation, and cleanup run.

const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { normalizeBasePath } = require('./base-path.js');
const { createManagement, ManagementError } = require('./management.js');
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

  const MAX_KEY_LENGTH = parseInt(process.env.SKRYNIA_MAX_KEY_LENGTH || '256', 10);
  const MAX_OBJECT_SIZE = parseInt(process.env.SKRYNIA_MAX_OBJECT_SIZE || '10485760', 10);

  const management = createManagement({
    dataDir: shared.dataDir,
    appBasePath: APP_BASE_PATH,
    appDir: APP_DIR,
    builderImage: opts.builderImage,
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
      json(res, 409, {error:'namespace_not_created', detail:'create the namespace through /_skrynia/ns/create first'});
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
    if (body.length > MAX_OBJECT_SIZE) return json(res, 413, {error:'object_too_large'});

    const q = shared.recalcQuota(ns);
    const newBytes = q.bytes - meta.size + body.length;
    if (newBytes > q.quotaBytes) return json(res, 507, {error:'namespace_full',detail:'quota_bytes'});

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
    '/_skrynia/deploy': params => management.deploy(params),
    '/_skrynia/undeploy': params => management.undeploy(params),
    '/_skrynia/rollback': params => management.rollback(params),
    '/_skrynia/releases': params => management.releases(params),
    '/_skrynia/inspect': params => management.inspect(params),
    '/_skrynia/ns/create': params => management.namespaceCreate(params),
    '/_skrynia/ns/remove': params => management.namespaceRemove(params),
    '/_skrynia/ns/inspect': params => management.namespaceInspect(params),
    '/_skrynia/ns/list': () => management.namespaceList(),
  };

  function route(req, res) {
    let url;
    try { url = new URL(req.url, 'http://' + req.headers.host); }
    catch { res.writeHead(400); res.end('Bad request'); return; }
    const p = url.pathname;

    if (p === '/_skrynia/health') return json(res, 200, {ok:true});

    if (p === '/_skrynia/client/skrynia.js') {
      if (!existsSync(CLIENT_PATH)) { res.writeHead(404); res.end('Not found'); return; }
      res.writeHead(200, {'Content-Type':'application/javascript'});
      res.end(readFileSync(CLIENT_PATH));
      return;
    }

    const managementHandler = managementRoutes[p];
    if (managementHandler) {
      if (req.method !== 'GET') { res.writeHead(405); res.end('Method not allowed'); return; }
      if (!requireManagementToken(url, res)) return;
      const params = Object.fromEntries(url.searchParams.entries());
      delete params.token;
      try {
        return json(res, 200, managementHandler(params));
      } catch (e) {
        if (e instanceof ManagementError) return json(res, e.status, {error:e.code, detail:e.detail});
        console.error('skrynia management error:', e && e.stack ? e.stack : e);
        return json(res, 500, {error:'internal_error'});
      }
    }

    const storeMatch = p.match(/^\/_skrynia\/store\/([^/]+)\/(.+)$/);
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
    const appMatch = p.match(new RegExp('^' + baseRe + '/([^/]+)(/.*)?$'));
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

  const server = http.createServer(route);
  server._skrynia = { APP_BASE_PATH, APP_DIR, DATA_DIR: shared.dataDir, managementEnabled: Boolean(TOKEN) };
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
