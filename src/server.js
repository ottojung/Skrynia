#!/usr/bin/env node
'use strict';

// Skrynia storage server.
//
// Runtime invariant: exactly ONE server process per data directory.
// Node.js event-loop serialization makes concurrent request mutations
// safe within a single process. No cross-process locking is provided;
// the admin CLI and server must not modify the same data directory
// simultaneously.
//
// Atomic filesystem primitives:
//   - create: exclusive O_CREAT on .dat file (fails if exists)
//   - put: write to .tmp, fs.rename to .dat (atomic on POSIX)
//   - delete: unlink .dat, .meta, .cap (individual unlinks)
//
// Namespace policy: a namespace must exist (have a quota.json) before
// the public store API will accept mutations.
//
// Capability verifier: the .cap file is the single source of truth
// containing the SHA-256 hash of the capability. The .meta file does
// not store the verifier.

const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { existsSync, mkdirSync, readFileSync, writeFileSync, unlinkSync, statSync, readdirSync, realpathSync, renameSync, openSync, closeSync } = fs;
const { O_CREAT, O_EXCL, O_WRONLY } = fs.constants || {};

function createServer(opts) {
  opts = opts || {};
  const DATA_DIR = opts.dataDir || process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
  const RELEASES_DIR = path.join(DATA_DIR, 'releases');
  const STORAGE_DIR = path.join(DATA_DIR, 'storage');
  const STATE_DIR = path.join(DATA_DIR, 'state');
  const CLIENT_PATH = path.join(__dirname, 'client.js');

  // Configurable URL base path for app serving (default "/apps").
  // There is no canonical prefix; the deployer sets this to match the
  // reverse proxy or web server configuration (e.g. "/a", "/apps", "/s").
  const APP_BASE_PATH = (opts.appBasePath || process.env.SKRYNIA_APP_BASE_PATH || '/apps').replace(/\/+$/, '');

  // Optional separate filesystem directory where active app symlinks are
  // exposed for direct serving by an external web server (e.g. nginx).
  // When set, admin deploy creates APP_DIR/{ns} -> release dir symlinks.
  const APP_DIR = opts.appDir || process.env.SKRYNIA_APP_DIR || '';

  const DEFAULT_QUOTA_BYTES = parseInt(process.env.SKRYNIA_DEFAULT_QUOTA_BYTES || '10485760', 10);
  const MAX_OBJECT_COUNT = parseInt(process.env.SKRYNIA_MAX_OBJECT_COUNT || '10000', 10);
  const MAX_KEY_LENGTH = parseInt(process.env.SKRYNIA_MAX_KEY_LENGTH || '256', 10);
  const MAX_OBJECT_SIZE = parseInt(process.env.SKRYNIA_MAX_OBJECT_SIZE || '10485760', 10);

  function ensureDir(dir) { mkdirSync(dir, { recursive: true }); }

  // --- Strict namespace validator ---
  const NS_RE = /^[a-z0-9][a-z0-9_-]{0,63}$/;
  function validNs(ns) {
    return typeof ns === 'string' && NS_RE.test(ns);
  }

  function safeKey(key) {
    if (typeof key !== 'string' || key.length === 0 || key.length > MAX_KEY_LENGTH) return null;
    if (key.includes('\0') || key.includes('..') || key.startsWith('/') || key.includes('//')) return null;
    if (key.includes('/')) return null;
    if (key === '.' || key === '..') return null;
    return key;
  }

  function nsDir(ns) { return path.join(STORAGE_DIR, ns); }
  function objPath(ns, key) { return path.join(nsDir(ns), key + '.dat'); }
  function metaPath(ns, key) { return path.join(nsDir(ns), key + '.meta'); }
  function capPath(ns, key) { return path.join(nsDir(ns), key + '.cap'); }
  function quotaFile(ns) { return path.join(STATE_DIR, ns, 'quota.json'); }
  function currentLink(ns) { return path.join(RELEASES_DIR, ns, 'current'); }

  // --- Namespace validation ---

  function nsExists(ns) { return existsSync(quotaFile(ns)); }

  function requireNs(ns, res) {
    if (!nsExists(ns)) {
      res.writeHead(404, {'Content-Type':'application/json'});
      res.end(JSON.stringify({error:'namespace_not_found'}));
      return false;
    }
    return true;
  }

  function requireNsCreate(ns, res) {
    if (!nsExists(ns)) {
      res.writeHead(409, {'Content-Type':'application/json'});
      res.end(JSON.stringify({error:'namespace_not_created', detail:'use admin CLI to create namespace first'}));
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

  // --- Quota ---

  function loadQuota(ns) {
    const p = quotaFile(ns);
    if (!existsSync(p)) return { bytes: 0, count: 0, quotaBytes: DEFAULT_QUOTA_BYTES, maxObjects: MAX_OBJECT_COUNT };
    return JSON.parse(readFileSync(p, 'utf8'));
  }

  function saveQuota(ns, q) {
    ensureDir(path.dirname(quotaFile(ns)));
    writeFileSync(quotaFile(ns), JSON.stringify(q, null, 2));
  }

  function recalcQuota(ns) {
    const dir = nsDir(ns);
    let bytes = 0, count = 0;
    if (existsSync(dir)) {
      for (const e of readdirSync(dir)) {
        if (e.endsWith('.dat')) { bytes += statSync(path.join(dir, e)).size; count++; }
      }
    }
    const q = loadQuota(ns);
    q.bytes = bytes; q.count = count;
    saveQuota(ns, q);
    return q;
  }

  // --- Atomic file helpers ---

  function exclusiveCreate(filePath, data) {
    try {
      const fd = openSync(filePath, O_CREAT | O_EXCL | O_WRONLY);
      try {
        if (data && data.length > 0) {
          fs.writeSync(fd, data, 0, data.length, null);
        }
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

  // --- Storage handlers ---

  function handleGet(ns, key, res) {
    const op = objPath(ns, key);
    if (!existsSync(op)) { res.writeHead(404, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'not_found'})); return; }
    const meta = JSON.parse(readFileSync(metaPath(ns, key), 'utf8'));
    const data = readFileSync(op);
    res.writeHead(200, {'Content-Type': meta.contentType||'application/octet-stream', 'X-Skrynia-Mode': meta.mode, 'X-Skrynia-Created': meta.created});
    res.end(data);
  }

  function handleCreate(ns, key, body, req, res) {
    if (!requireNsCreate(ns, res)) return;

    const mode = req.headers['x-skrynia-mode'] || 'capability-write';
    if (!['immutable','capability-write','public-write'].includes(mode)) {
      res.writeHead(400, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'invalid_mode'})); return;
    }
    if (body.length > MAX_OBJECT_SIZE) {
      res.writeHead(413, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'object_too_large'})); return;
    }

    const q = recalcQuota(ns);
    if (q.count >= q.maxObjects) {
      res.writeHead(507, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'namespace_full',detail:'max_objects'})); return;
    }
    if (q.bytes + body.length > q.quotaBytes) {
      res.writeHead(507, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'namespace_full',detail:'quota_bytes'})); return;
    }

    ensureDir(nsDir(ns));

    const created = exclusiveCreate(objPath(ns, key), body);
    if (!created) {
      res.writeHead(409, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'already_exists'})); return;
    }

    const now = new Date().toISOString();
    const meta = { mode, contentType: req.headers['content-type']||'application/octet-stream', created: now, size: body.length };
    const resp = { ok: true, mode };

    if (mode === 'capability-write') {
      const cap = generateCapability();
      const verifier = sha256hex(cap);
      writeFileSync(capPath(ns, key), verifier);
      resp.capability = cap;
    }

    writeFileSync(metaPath(ns, key), JSON.stringify(meta, null, 2));

    q.bytes += body.length; q.count++;
    saveQuota(ns, q);
    res.writeHead(201, {'Content-Type':'application/json'});
    res.end(JSON.stringify(resp));
  }

  function handlePut(ns, key, body, req, res) {
    if (!requireNs(ns, res)) return;

    const op = objPath(ns, key);
    if (!existsSync(op)) { res.writeHead(404, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'not_found'})); return; }
    const meta = JSON.parse(readFileSync(metaPath(ns, key), 'utf8'));
    if (meta.mode === 'immutable') { res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'immutable'})); return; }
    if (meta.mode === 'capability-write') {
      const cap = req.headers['x-skrynia-capability'];
      if (!cap) { res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'capability_required'})); return; }
      const storedHash = readFileSync(capPath(ns, key), 'utf8');
      if (!timingSafeEqualHex(sha256hex(cap), storedHash)) {
        res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'invalid_capability'})); return;
      }
    }
    if (body.length > MAX_OBJECT_SIZE) {
      res.writeHead(413, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'object_too_large'})); return;
    }

    const q = recalcQuota(ns);
    const oldSize = meta.size;
    const newBytes = q.bytes - oldSize + body.length;

    if (newBytes > q.quotaBytes) {
      res.writeHead(507, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'namespace_full',detail:'quota_bytes'})); return;
    }

    atomicReplace(op, body);

    meta.size = body.length;
    meta.contentType = req.headers['content-type'] || meta.contentType;
    meta.modified = new Date().toISOString();
    writeFileSync(metaPath(ns, key), JSON.stringify(meta, null, 2));

    q.bytes = newBytes;
    saveQuota(ns, q);
    res.writeHead(200, {'Content-Type':'application/json'});
    res.end(JSON.stringify({ok:true}));
  }

  function handleDelete(ns, key, req, res) {
    if (!requireNs(ns, res)) return;

    const op = objPath(ns, key);
    if (!existsSync(op)) { res.writeHead(404, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'not_found'})); return; }
    const meta = JSON.parse(readFileSync(metaPath(ns, key), 'utf8'));
    if (meta.mode === 'immutable') { res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'immutable'})); return; }
    if (meta.mode === 'capability-write') {
      const cap = req.headers['x-skrynia-capability'];
      if (!cap) { res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'capability_required'})); return; }
      if (!timingSafeEqualHex(sha256hex(cap), readFileSync(capPath(ns, key), 'utf8'))) {
        res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'invalid_capability'})); return;
      }
    }

    const q = recalcQuota(ns);
    unlinkSync(op);
    unlinkSync(metaPath(ns, key));
    const cp = capPath(ns, key);
    if (existsSync(cp)) unlinkSync(cp);
    q.bytes -= meta.size; q.count--;
    saveQuota(ns, q);
    res.writeHead(200, {'Content-Type':'application/json'});
    res.end(JSON.stringify({ok:true}));
  }

  // --- Static app serving ---

  function activeLink(ns) {
    return APP_DIR ? path.join(APP_DIR, ns) : path.join(RELEASES_DIR, ns, 'current');
  }

  function serveApp(ns, req, res, urlPath) {
    const link = activeLink(ns);
    if (!existsSync(link)) { res.writeHead(503, {'Content-Type':'text/plain'}); res.end('Service unavailable'); return; }
    let appRoot;
    try { appRoot = realpathSync(link); } catch { res.writeHead(503, {'Content-Type':'text/plain'}); res.end('Service unavailable'); return; }
    if (urlPath === '/') urlPath = '/index.html';
    const safe = path.normalize(urlPath);
    if (safe.includes('..')) { res.writeHead(403); res.end('Forbidden'); return; }
    const filePath = path.join(appRoot, safe);

    // For missing files (no realpath possible), return 404.
    // For existing files/symlinks, realpath to detect escapes.
    if (!existsSync(filePath)) {
      res.writeHead(404); res.end('Not found'); return;
    }

    let resolved;
    try { resolved = realpathSync(filePath); } catch { resolved = null; }
    if (!resolved || (resolved !== appRoot && !resolved.startsWith(appRoot + path.sep))) {
      res.writeHead(403); res.end('Forbidden'); return;
    }

    if (statSync(resolved).isDirectory()) {
      const tryIndex = path.join(resolved, 'index.html');
      if (existsSync(tryIndex)) {
        const tryResolved = realpathSync(tryIndex);
        if (tryResolved === appRoot || tryResolved.startsWith(appRoot + path.sep)) {
          serveFile(tryIndex, res); return;
        }
      }
      res.writeHead(404); res.end('Not found'); return;
    }
    serveFile(resolved, res);
  }

  function serveFile(p, res) {
    const types = {'.html':'text/html','.js':'application/javascript','.css':'text/css','.json':'application/json','.png':'image/png','.svg':'image/svg+xml'};
    const ct = types[path.extname(p)] || 'application/octet-stream';
    res.writeHead(200, {'Content-Type': ct});
    res.end(readFileSync(p));
  }

  // --- Router ---

  function readBody(req, res, cb) {
    const chunks = []; let size = 0;
    let oversized = false;
    req.on('data', c => {
      if (oversized) return;
      size += c.length;
      if (size > MAX_OBJECT_SIZE) {
        oversized = true;
        res.writeHead(413, {'Content-Type':'application/json'});
        res.end(JSON.stringify({error:'request_too_large'}));
        return;
      }
      chunks.push(c);
    });
    req.on('end', () => { if (!oversized) cb(Buffer.concat(chunks)); });
  }

  function route(req, res) {
    let p;
    try {
      p = new URL(req.url, 'http://'+req.headers.host).pathname;
    } catch {
      res.writeHead(400); res.end('Bad request'); return;
    }

    // Client library
    if (p === '/_skrynia/client/skrynia.js') {
      if (existsSync(CLIENT_PATH)) {
        res.writeHead(200, {'Content-Type':'application/javascript'});
        res.end(readFileSync(CLIENT_PATH));
      } else {
        res.writeHead(404); res.end('Not found');
      }
      return;
    }

    const storeMatch = req.url.match(/^\/_skrynia\/store\/([^/]+)\/(.+?)(?:\?.*)?$/);
    if (storeMatch) {
      let ns, sk;
      try {
        ns = decodeURIComponent(storeMatch[1]);
        sk = safeKey(decodeURIComponent(storeMatch[2]));
      } catch {
        res.writeHead(400, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'invalid_percent_encoding'})); return;
      }
      if (!validNs(ns)) { res.writeHead(400, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'invalid_namespace'})); return; }
      if (!sk) { res.writeHead(400, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'invalid_key'})); return; }
      if (req.method === 'GET') { if (!requireNs(ns, res)) return; return handleGet(ns, sk, res); }
      if (req.method === 'DELETE') return handleDelete(ns, sk, req, res);
      if (req.method === 'PUT' || req.method === 'POST') {
        return readBody(req, res, body => {
          if (req.method === 'POST') handleCreate(ns, sk, body, req, res);
          else handlePut(ns, sk, body, req, res);
        });
      }
      res.writeHead(405); res.end('Method not allowed'); return;
    }

    // Build app route regex from configurable base path.
    // Escapes regex metacharacters in the prefix, then matches /{ns}/{path}.
    const baseRe = APP_BASE_PATH.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const appRe = new RegExp('^' + baseRe + '/([^/]+)(/.*)?$');
    const appMatch = p.match(appRe);
    if (appMatch) {
      let ns;
      try { ns = decodeURIComponent(appMatch[1]); } catch { res.writeHead(400); res.end('Bad namespace'); return; }
      if (!validNs(ns)) { res.writeHead(400); res.end('Bad namespace'); return; }
      return serveApp(ns, req, res, appMatch[2] || '/');
    }

    if (p === '/_skrynia/health') { res.writeHead(200, {'Content-Type':'application/json'}); res.end(JSON.stringify({ok:true})); return; }
    res.writeHead(404); res.end('Not found');
  }

  ensureDir(RELEASES_DIR); ensureDir(STORAGE_DIR); ensureDir(STATE_DIR);

  const server = http.createServer(route);
  server._skrynia = { APP_BASE_PATH, APP_DIR, DATA_DIR };
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
