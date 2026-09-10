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

const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { existsSync, mkdirSync, readFileSync, writeFileSync, unlinkSync, statSync, readdirSync, realpathSync, renameSync, openSync, closeSync, createWriteStream } = fs;
const { O_CREAT, O_EXCL, O_WRONLY } = fs.constants || {};

function createServer(opts) {
  opts = opts || {};
  const DATA_DIR = opts.dataDir || process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
  const RELEASES_DIR = path.join(DATA_DIR, 'releases');
  const STORAGE_DIR = path.join(DATA_DIR, 'storage');
  const STATE_DIR = path.join(DATA_DIR, 'state');

  const DEFAULT_QUOTA_BYTES = parseInt(process.env.SKRYNIA_DEFAULT_QUOTA_BYTES || '10485760', 10);
  const MAX_OBJECT_COUNT = parseInt(process.env.SKRYNIA_MAX_OBJECT_COUNT || '10000', 10);
  const MAX_KEY_LENGTH = parseInt(process.env.SKRYNIA_MAX_KEY_LENGTH || '256', 10);
  const MAX_OBJECT_SIZE = parseInt(process.env.SKRYNIA_MAX_OBJECT_SIZE || '10485760', 10);

  function ensureDir(dir) { mkdirSync(dir, { recursive: true }); }

  function safeKey(key) {
    if (typeof key !== 'string' || key.length === 0 || key.length > MAX_KEY_LENGTH) return null;
    if (key.includes('\0') || key.includes('..') || key.startsWith('/') || key.includes('//')) return null;
    if (key === '.' || key === '..') return null;
    return key;
  }

  function nsDir(ns) { return path.join(STORAGE_DIR, ns); }
  function objPath(ns, key) { return path.join(nsDir(ns), key + '.dat'); }
  function metaPath(ns, key) { return path.join(nsDir(ns), key + '.meta'); }
  function capPath(ns, key) { return path.join(nsDir(ns), key + '.cap'); }
  function quotaFile(ns) { return path.join(STATE_DIR, ns, 'quota.json'); }
  function currentLink(ns) { return path.join(RELEASES_DIR, ns, 'current'); }

  function generateCapability() { return crypto.randomBytes(32).toString('hex'); }
  function sha256hex(data) { return crypto.createHash('sha256').update(data).digest('hex'); }

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

  // Exclusive create: returns true if created, false if exists
  function exclusiveCreate(filePath, data) {
    try {
      const fd = openSync(filePath, O_CREAT | O_EXCL | O_WRONLY);
      closeSync(fd);
      writeFileSync(filePath, data);
      return true;
    } catch (e) {
      if (e.code === 'EEXIST') return false;
      throw e;
    }
  }

  // Atomic replace: write to tmp, rename
  function atomicReplace(filePath, data) {
    const tmp = filePath + '.tmp.' + process.pid;
    writeFileSync(tmp, data);
    renameSync(tmp, filePath);
  }

  // --- Storage handlers (single-process event-loop serialized) ---

  function handleGet(ns, key, res) {
    const op = objPath(ns, key);
    if (!existsSync(op)) { res.writeHead(404, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'not_found'})); return; }
    const meta = JSON.parse(readFileSync(metaPath(ns, key), 'utf8'));
    const data = readFileSync(op);
    res.writeHead(200, {'Content-Type': meta.contentType||'application/octet-stream', 'X-Skrynia-Mode': meta.mode, 'X-Skrynia-Created': meta.created});
    res.end(data);
  }

  function handleCreate(ns, key, body, req, res) {
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

    // Exclusive create: atomic fail-if-exists
    const created = exclusiveCreate(objPath(ns, key), body);
    if (!created) {
      res.writeHead(409, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'already_exists'})); return;
    }

    const now = new Date().toISOString();
    const meta = { mode, contentType: req.headers['content-type']||'application/octet-stream', created: now, size: body.length };
    const resp = { ok: true, mode };

    if (mode === 'capability-write') {
      const cap = generateCapability();
      meta.capVerifier = sha256hex(cap);
      resp.capability = cap;
    }

    writeFileSync(metaPath(ns, key), JSON.stringify(meta, null, 2));
    if (mode === 'capability-write') writeFileSync(capPath(ns, key), meta.capVerifier);

    q.bytes += body.length; q.count++;
    saveQuota(ns, q);
    res.writeHead(201, {'Content-Type':'application/json'});
    res.end(JSON.stringify(resp));
  }

  function handlePut(ns, key, body, req, res) {
    const op = objPath(ns, key);
    if (!existsSync(op)) { res.writeHead(404, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'not_found'})); return; }
    const meta = JSON.parse(readFileSync(metaPath(ns, key), 'utf8'));
    if (meta.mode === 'immutable') { res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'immutable'})); return; }
    if (meta.mode === 'capability-write') {
      const cap = req.headers['x-skrynia-capability'];
      if (!cap) { res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'capability_required'})); return; }
      if (sha256hex(cap) !== readFileSync(capPath(ns, key), 'utf8')) {
        res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'invalid_capability'})); return;
      }
    }
    if (body.length > MAX_OBJECT_SIZE) {
      res.writeHead(413, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'object_too_large'})); return;
    }

    const q = recalcQuota(ns);
    const oldSize = meta.size;

    // Atomic replace: tmp + rename
    atomicReplace(op, body);

    meta.size = body.length;
    meta.contentType = req.headers['content-type'] || meta.contentType;
    meta.modified = new Date().toISOString();
    writeFileSync(metaPath(ns, key), JSON.stringify(meta, null, 2));

    q.bytes = q.bytes - oldSize + body.length;
    saveQuota(ns, q);
    res.writeHead(200, {'Content-Type':'application/json'});
    res.end(JSON.stringify({ok:true}));
  }

  function handleDelete(ns, key, req, res) {
    const op = objPath(ns, key);
    if (!existsSync(op)) { res.writeHead(404, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'not_found'})); return; }
    const meta = JSON.parse(readFileSync(metaPath(ns, key), 'utf8'));
    if (meta.mode === 'immutable') { res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'immutable'})); return; }
    if (meta.mode === 'capability-write') {
      const cap = req.headers['x-skrynia-capability'];
      if (!cap) { res.writeHead(403, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'capability_required'})); return; }
      if (sha256hex(cap) !== readFileSync(capPath(ns, key), 'utf8')) {
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

  function serveApp(ns, req, res) {
    const cur = currentLink(ns);
    if (!existsSync(cur)) { res.writeHead(503, {'Content-Type':'text/plain'}); res.end('Service unavailable'); return; }
    const releaseRoot = realpathSync(cur);
    let urlPath = req.url.replace(/^\/a\/[^/]+\/?/, '/') || '/index.html';
    if (urlPath === '/') urlPath = '/index.html';
    const safe = path.normalize(urlPath);
    if (safe.includes('..')) { res.writeHead(403); res.end('Forbidden'); return; }
    const filePath = path.join(releaseRoot, safe);
    if (!existsSync(filePath) || statSync(filePath).isDirectory()) {
      const tryIndex = path.join(filePath, 'index.html');
      if (existsSync(tryIndex)) { serveFile(tryIndex, res); return; }
      res.writeHead(404); res.end('Not found'); return;
    }
    serveFile(filePath, res);
  }

  function serveFile(p, res) {
    const types = {'.html':'text/html','.js':'application/javascript','.css':'text/css','.json':'application/json','.png':'image/png','.svg':'image/svg+xml'};
    const ct = types[path.extname(p)] || 'application/octet-stream';
    res.writeHead(200, {'Content-Type': ct});
    res.end(readFileSync(p));
  }

  // --- Router ---

  function readBody(req, cb) {
    const chunks = []; let size = 0;
    req.on('data', c => { size += c.length; if (size > MAX_OBJECT_SIZE+1024) { req.destroy(); return; } chunks.push(c); });
    req.on('end', () => cb(Buffer.concat(chunks)));
  }

  function route(req, res) {
    const p = new URL(req.url, 'http://'+req.headers.host).pathname;

    const storeMatch = req.url.match(/^\/_skrynia\/store\/([^/]+)\/(.+?)(?:\?.*)?$/);
    if (storeMatch) {
      const ns = decodeURIComponent(storeMatch[1]);
      const sk = safeKey(decodeURIComponent(storeMatch[2]));
      if (!sk) { res.writeHead(400, {'Content-Type':'application/json'}); res.end(JSON.stringify({error:'invalid_key'})); return; }
      if (req.method === 'GET') return handleGet(ns, sk, res);
      if (req.method === 'DELETE') return handleDelete(ns, sk, req, res);
      if (req.method === 'PUT' || req.method === 'POST') {
        return readBody(req, body => {
          if (req.method === 'POST') handleCreate(ns, sk, body, req, res);
          else handlePut(ns, sk, body, req, res);
        });
      }
      res.writeHead(405); res.end('Method not allowed'); return;
    }

    const appMatch = p.match(/^\/a\/([^/]+)(\/.*)?$/);
    if (appMatch) return serveApp(decodeURIComponent(appMatch[1]), req, res);

    if (p === '/_skrynia/health') { res.writeHead(200, {'Content-Type':'application/json'}); res.end(JSON.stringify({ok:true})); return; }
    res.writeHead(404); res.end('Not found');
  }

  ensureDir(RELEASES_DIR); ensureDir(STORAGE_DIR); ensureDir(STATE_DIR);

  const server = http.createServer(route);
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
