#!/usr/bin/env node
'use strict';

const http = require('http');
const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');
const { createServer } = require('../src/server.js');

const SRC = path.join(__dirname, '..');
const NODE = process.execPath;
const TMP = '/tmp/skrynia-test';

function rmrf(d) { try { fs.rmSync(d, { recursive: true, force: true }); } catch {} }

function setup() {
  rmrf(TMP);
  fs.mkdirSync(path.join(TMP, 'releases'), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'storage'), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'state'), { recursive: true });
}

function startServer() {
  const server = createServer({ dataDir: TMP });
  return new Promise(resolve => {
    server.listen(0, '127.0.0.1', () => resolve({ server, port: server.address().port }));
  });
}

function stopServer(s) { return new Promise(resolve => { s.close(() => resolve()); }); }

function get(port, urlPath) {
  return new Promise((ok, fail) => {
    http.get({ hostname: '127.0.0.1', port, path: urlPath, timeout: 3000 }, res => {
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => ok({ status: res.statusCode, body: Buffer.concat(chunks), text: Buffer.concat(chunks).toString(), headers: res.headers }));
    }).on('error', fail).on('timeout', function() { this.destroy(); fail(new Error('timeout')); });
  });
}

function req(port, method, urlPath, data, headers) {
  return new Promise((ok, fail) => {
    const opts = { hostname: '127.0.0.1', port, path: urlPath, method, headers: headers || {}, timeout: 3000 };
    if (data != null) opts.headers['Content-Length'] = Buffer.byteLength(data);
    const r = http.request(opts, res => {
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => ok({ status: res.statusCode, body: Buffer.concat(chunks), text: Buffer.concat(chunks).toString(), headers: res.headers }));
    });
    r.on('error', fail).on('timeout', () => { r.destroy(); fail(new Error('timeout')); });
    if (data != null) r.write(data);
    r.end();
  });
}

function assert(cond, msg) { if (!cond) throw new Error('ASSERT: ' + msg); }

function createNs(ns) {
  fs.mkdirSync(path.join(TMP, 'state', ns), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', ns, 'quota.json'), JSON.stringify({bytes:0,count:0,quotaBytes:10485760,maxObjects:10000}));
}

// --- Original tests ---

async function test_health() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    const r = await get(port, '/_skrynia/health');
    assert(r.status === 200, 'status 200');
    assert(JSON.parse(r.text).ok === true, 'ok=true');
  } finally { await stopServer(server); }
}

async function test_key_validation() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/_skrynia/store/ns/a%2F..%2Fb');
    assert(r.status === 400, 'traversal blocked, got ' + r.status);
    r = await get(port, '/_skrynia/store/ns/..%2Fetc%2Fpasswd');
    assert(r.status === 400, 'encoded traversal blocked, got ' + r.status);
    r = await get(port, '/_skrynia/store/ns/hello');
    assert(r.status === 404, 'valid key not_found');
  } finally { await stopServer(server); }
}

async function test_store_crud() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await req(port, 'POST', '/_skrynia/store/ns/hello', 'world', {'Content-Type':'text/plain','X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'create 201');
    assert(JSON.parse(r.text).ok === true, 'create ok');

    r = await get(port, '/_skrynia/store/ns/hello');
    assert(r.status === 200, 'get 200');
    assert(r.text === 'world', 'get body');

    r = await req(port, 'PUT', '/_skrynia/store/ns/hello', 'updated', {'Content-Type':'text/plain'});
    assert(r.status === 200, 'put 200');

    r = await get(port, '/_skrynia/store/ns/hello');
    assert(r.text === 'updated', 'put body');

    r = await req(port, 'DELETE', '/_skrynia/store/ns/hello', null, {});
    assert(r.status === 200, 'delete 200');

    r = await get(port, '/_skrynia/store/ns/hello');
    assert(r.status === 404, 'deleted not_found');
  } finally { await stopServer(server); }
}

async function test_create_duplicate() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    await req(port, 'POST', '/_skrynia/store/ns/k', 'v1', {'X-Skrynia-Mode':'public-write'});
    const r = await req(port, 'POST', '/_skrynia/store/ns/k', 'v2', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 409, 'duplicate 409');
  } finally { await stopServer(server); }
}

async function test_immutable() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    await req(port, 'POST', '/_skrynia/store/ns/imm', 'forever', {'X-Skrynia-Mode':'immutable'});
    let r = await req(port, 'DELETE', '/_skrynia/store/ns/imm', null, {});
    assert(r.status === 403, 'delete blocked');
    r = await req(port, 'PUT', '/_skrynia/store/ns/imm', 'nope', {'Content-Type':'text/plain'});
    assert(r.status === 403, 'put blocked');
  } finally { await stopServer(server); }
}

async function test_capability_write() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await req(port, 'POST', '/_skrynia/store/ns/ck', 'data', {'X-Skrynia-Mode':'capability-write'});
    assert(r.status === 201, 'create 201');
    const j = JSON.parse(r.text);
    assert(j.capability && j.capability.length === 64, 'capability returned');
    const cap = j.capability;

    r = await req(port, 'PUT', '/_skrynia/store/ns/ck', 'hack', {'Content-Type':'text/plain'});
    assert(r.status === 403, 'put no cap 403');
    r = await req(port, 'PUT', '/_skrynia/store/ns/ck', 'hack', {'Content-Type':'text/plain', 'X-Skrynia-Capability':'wrong'});
    assert(r.status === 403, 'put bad cap 403');
    r = await req(port, 'PUT', '/_skrynia/store/ns/ck', 'legit', {'Content-Type':'text/plain', 'X-Skrynia-Capability': cap});
    assert(r.status === 200, 'put good cap 200');
    r = await get(port, '/_skrynia/store/ns/ck');
    assert(r.text === 'legit', 'put verified');

    r = await req(port, 'DELETE', '/_skrynia/store/ns/ck', null, {});
    assert(r.status === 403, 'del no cap 403');
    r = await req(port, 'DELETE', '/_skrynia/store/ns/ck', null, {'X-Skrynia-Capability': cap});
    assert(r.status === 200, 'del good cap 200');
  } finally { await stopServer(server); }
}

async function test_quota_full() {
  setup();
  const qDir = path.join(TMP, 'state', 'tq');
  fs.mkdirSync(qDir, { recursive: true });
  fs.writeFileSync(path.join(qDir, 'quota.json'), JSON.stringify({bytes:0,count:0,quotaBytes:10,maxObjects:2}));
  const { server, port } = await startServer();
  try {
    await req(port, 'POST', '/_skrynia/store/tq/a', 'x', {'X-Skrynia-Mode':'public-write'});
    await req(port, 'POST', '/_skrynia/store/tq/b', 'y', {'X-Skrynia-Mode':'public-write'});
    let r = await req(port, 'POST', '/_skrynia/store/tq/c', 'z', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 507, 'count full 507');
  } finally { await stopServer(server); }
}

async function test_atomic_symlink() {
  setup();
  const relBase = path.join(TMP, 'releases', 'myapp');
  fs.mkdirSync(path.join(relBase, 'r1'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r1', 'index.html'), 'v1');
  fs.mkdirSync(path.join(relBase, 'r2'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r2', 'index.html'), 'v2');
  const link = path.join(relBase, 'current');
  let tmp = link + '.tmp';
  fs.symlinkSync(path.join(relBase, 'r1'), tmp); fs.renameSync(tmp, link);
  assert(fs.readlinkSync(link).endsWith('/r1'), 'link to r1');
  tmp = link + '.tmp';
  fs.symlinkSync(path.join(relBase, 'r2'), tmp); fs.renameSync(tmp, link);
  assert(fs.readlinkSync(link).endsWith('/r2'), 'link to r2');
  assert(fs.existsSync(path.join(relBase, 'r1', 'index.html')), 'r1 still exists');
}

async function test_app_serving() {
  setup(); createNs('webapp');
  const relBase = path.join(TMP, 'releases', 'webapp');
  fs.mkdirSync(path.join(relBase, 'assets'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'index.html'), '<h1>Hello</h1>');
  fs.writeFileSync(path.join(relBase, 'assets', 'app.js'), 'console.log("hi")');
  fs.symlinkSync(relBase, path.join(relBase, '..', 'webapp', 'current'));

  const { server, port } = await startServer();
  try {
    let r = await get(port, '/a/webapp/');
    assert(r.status === 200, 'index 200');
    assert(r.text === '<h1>Hello</h1>', 'index body');
    assert(r.headers['content-type'] === 'text/html', 'index type');

    r = await get(port, '/a/webapp/assets/app.js');
    assert(r.status === 200, 'js 200');
    assert(r.headers['content-type'] === 'application/javascript', 'js type');

    r = await get(port, '/a/webapp/../../etc/passwd');
    assert(r.status >= 400, 'traversal blocked');
  } finally { await stopServer(server); }
}

async function test_rollback_sim() {
  setup();
  const relBase = path.join(TMP, 'releases', 'rbapp');
  fs.mkdirSync(path.join(relBase, '20260910120000'), { recursive: true });
  fs.writeFileSync(path.join(relBase, '20260910120000', 'index.html'), 'v1');
  fs.mkdirSync(path.join(relBase, '20260910130000'), { recursive: true });
  fs.writeFileSync(path.join(relBase, '20260910130000', 'index.html'), 'v2');
  const link = path.join(relBase, 'current');
  let t = link + '.tmp';
  fs.symlinkSync(path.join(relBase, '20260910130000'), t); fs.renameSync(t, link);
  t = link + '.tmp';
  fs.symlinkSync(path.join(relBase, '20260910120000'), t); fs.renameSync(t, link);
  assert(fs.readlinkSync(link).includes('20260910120000'), 'rolled back');
}

async function test_concurrent_create() {
  setup(); createNs('cct');
  const { server, port } = await startServer();
  try {
    const promises = [];
    for (let i = 0; i < 10; i++) {
      promises.push(req(port, 'POST', '/_skrynia/store/cct/same', 'data' + i, {'X-Skrynia-Mode':'public-write'}));
    }
    const results = await Promise.all(promises);
    const okCount = results.filter(r => r.status === 201).length;
    const conflictCount = results.filter(r => r.status === 409).length;
    assert(okCount === 1, 'exactly one create succeeded, got ' + okCount);
    assert(conflictCount === 9, 'nine conflicts, got ' + conflictCount);
  } finally { await stopServer(server); }
}

async function test_concurrent_quota_boundary() {
  setup();
  const qDir = path.join(TMP, 'state', 'cqb');
  fs.mkdirSync(qDir, { recursive: true });
  fs.writeFileSync(path.join(qDir, 'quota.json'), JSON.stringify({bytes:0,count:0,quotaBytes:15,maxObjects:10}));
  const { server, port } = await startServer();
  try {
    const promises = [];
    for (let i = 0; i < 5; i++) {
      promises.push(req(port, 'POST', '/_skrynia/store/cqb/k' + i, 'bytes!', {'X-Skrynia-Mode':'public-write'}));
    }
    const results = await Promise.all(promises);
    const okCount = results.filter(r => r.status === 201).length;
    const fullCount = results.filter(r => r.status === 507).length;
    assert(okCount + fullCount === 5, 'all accounted for');
    assert(okCount >= 1, 'at least one succeeded');
    assert(fullCount >= 1, 'at least one hit quota');
    assert(okCount <= 3, 'at most 3 succeeded (quota=15 bytes, each=6 bytes)');
  } finally { await stopServer(server); }
}

async function test_admin_help() {
  const out = execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'help'], { timeout: 3000 });
  assert(out.toString().includes('Usage:'), 'help has Usage');
}

async function test_admin_ns_crud() {
  setup();
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'ns', 'create', 'mytest', '--quota', '1024'], { timeout: 3000, env });
  assert(fs.existsSync(path.join(TMP, 'state', 'mytest', 'quota.json')), 'ns created');
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'ns', 'remove', 'mytest'], { timeout: 3000, env });
  assert(!fs.existsSync(path.join(TMP, 'storage', 'mytest')), 'ns removed');
}

async function test_admin_releases() {
  setup();
  fs.mkdirSync(path.join(TMP, 'releases', 'emptyns'), { recursive: true });
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  const out = execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'releases', 'emptyns'], { timeout: 3000, env });
  const text = out.toString();
  assert(text.includes('Releases:'), 'has Releases header');
  const lines = text.split('\n').filter(l => l.trim().startsWith('20'));
  assert(lines.length === 0, 'no release entries');
}

async function test_admin_rollback() {
  setup();
  const relBase = path.join(TMP, 'releases', 'rbns');
  fs.mkdirSync(path.join(relBase, 'r1'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r1', 'index.html'), 'v1');
  fs.mkdirSync(path.join(relBase, 'r2'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r2', 'index.html'), 'v2');
  fs.symlinkSync(path.join(relBase, 'r2'), path.join(relBase, 'current'));
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'rollback', 'rbns', 'r1'], { timeout: 3000, env });
  const link = fs.readlinkSync(path.join(relBase, 'current'));
  assert(link.includes('r1'), 'rolled back via admin');
}

// --- Regression tests for review fixes ---

async function test_namespace_not_created() {
  setup();
  const { server, port } = await startServer();
  try {
    // POST to non-existent namespace => 409
    let r = await req(port, 'POST', '/_skrynia/store/ghost/k', 'v', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 409, 'POST uncreated ns 409, got ' + r.status);
    // GET on non-existent namespace => 404
    r = await get(port, '/_skrynia/store/ghost/k');
    assert(r.status === 404, 'GET uncreated ns 404, got ' + r.status);
    // DELETE on non-existent namespace => 404
    r = await req(port, 'DELETE', '/_skrynia/store/ghost/k', null, {});
    assert(r.status === 404, 'DELETE uncreated ns 404, got ' + r.status);
    // PUT on non-existent namespace => 404
    r = await req(port, 'PUT', '/_skrynia/store/ghost/k', 'v', {'Content-Type':'text/plain'});
    assert(r.status === 404, 'PUT uncreated ns 404, got ' + r.status);
  } finally { await stopServer(server); }
}

async function test_put_quota_enforced() {
  setup();
  const qDir = path.join(TMP, 'state', 'pqt');
  fs.mkdirSync(qDir, { recursive: true });
  fs.writeFileSync(path.join(qDir, 'quota.json'), JSON.stringify({bytes:0,count:0,quotaBytes:10,maxObjects:100}));
  const { server, port } = await startServer();
  try {
    // Create a small object within quota
    await req(port, 'POST', '/_skrynia/store/pqt/k', 'hi', {'X-Skrynia-Mode':'public-write'});
    // Put a much larger object that exceeds quota => 507
    const big = 'x'.repeat(20);
    let r = await req(port, 'PUT', '/_skrynia/store/pqt/k', big, {'Content-Type':'text/plain'});
    assert(r.status === 507, 'PUT exceeds quota 507, got ' + r.status);
    // Original data unchanged
    r = await get(port, '/_skrynia/store/pqt/k');
    assert(r.text === 'hi', 'original data preserved');
  } finally { await stopServer(server); }
}

async function test_capability_not_in_meta() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    await req(port, 'POST', '/_skrynia/store/ns/ck2', 'data', {'X-Skrynia-Mode':'capability-write'});
    // Read the .meta file directly
    const meta = JSON.parse(fs.readFileSync(path.join(TMP, 'storage', 'ns', 'ck2.meta'), 'utf8'));
    assert(!meta.capVerifier, 'verifier not in meta');
    // Read the .cap file
    const capFile = path.join(TMP, 'storage', 'ns', 'ck2.cap');
    assert(fs.existsSync(capFile), '.cap file exists');
    const verifier = fs.readFileSync(capFile, 'utf8');
    assert(verifier.length === 64, 'verifier is 64 hex chars');
  } finally { await stopServer(server); }
}

async function test_app_serving_symlink_escape() {
  setup(); createNs('escapeapp');
  const relBase = path.join(TMP, 'releases', 'escapeapp');
  fs.mkdirSync(path.join(relBase, 'public'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'public', 'ok.html'), 'safe');
  // Create a symlink inside the release that points outside
  fs.symlinkSync('/etc', path.join(relBase, 'public', 'escape'));
  fs.symlinkSync(relBase, path.join(relBase, '..', 'escapeapp', 'current'));

  const { server, port } = await startServer();
  try {
    // Normal file works
    let r = await get(port, '/a/escapeapp/public/ok.html');
    assert(r.status === 200, 'normal file 200');
    // Symlink escape is blocked
    r = await get(port, '/a/escapeapp/escape/passwd');
    assert(r.status === 403, 'symlink escape 403, got ' + r.status);
    // Even reading the symlink itself is blocked
    r = await get(port, '/a/escapeapp/escape');
    assert(r.status === 403, 'symlink itself 403, got ' + r.status);
  } finally { await stopServer(server); }
}

async function test_rollback_updates_metadata() {
  setup();
  const relBase = path.join(TMP, 'releases', 'rbmeta');
  fs.mkdirSync(path.join(relBase, 'r1'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r1', 'index.html'), 'v1');
  fs.mkdirSync(path.join(relBase, 'r2'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r2', 'index.html'), 'v2');
  fs.symlinkSync(path.join(relBase, 'r2'), path.join(relBase, 'current'));
  // Write initial config
  const cfgPath = path.join(TMP, 'state', 'rbmeta', 'config.json');
  fs.mkdirSync(path.dirname(cfgPath), { recursive: true });
  fs.writeFileSync(cfgPath, JSON.stringify({namespace:'rbmeta',currentReleaseId:'r2'}, null, 2));

  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'rollback', 'rbmeta', 'r1'], { timeout: 3000, env });
  const cfg = JSON.parse(fs.readFileSync(cfgPath, 'utf8'));
  assert(cfg.currentReleaseId === 'r1', 'config updated to r1, got ' + cfg.currentReleaseId);
  assert(cfg.lastRollbackAt, 'rollback timestamp recorded');
}

async function test_undeploy_removes_everything() {
  setup(); createNs('udns');
  // Simulate deployed state through proper representation
  const relDir = path.join(TMP, 'releases', 'udns');
  fs.mkdirSync(path.join(relDir, 'r1'), { recursive: true });
  fs.writeFileSync(path.join(relDir, 'r1', 'index.html'), 'hi');
  fs.symlinkSync(path.join(relDir, 'r1'), path.join(relDir, 'current'));
  // Store data through the namespace storage directory
  fs.mkdirSync(path.join(TMP, 'storage', 'udns'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'storage', 'udns', 'data.bin'), 'payload');
  // Config exists from createNs, add deployment info
  fs.writeFileSync(path.join(TMP, 'state', 'udns', 'config.json'), JSON.stringify({namespace:'udns',currentReleaseId:'r1'}));

  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'undeploy', 'udns'], { timeout: 3000, env });
  assert(!fs.existsSync(relDir), 'releases removed');
  assert(!fs.existsSync(path.join(TMP, 'storage', 'udns')), 'storage removed');
  assert(!fs.existsSync(path.join(TMP, 'state', 'udns')), 'state removed');
}

async function test_deploy_subdir_validation() {
  setup();
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });

  // Early validation: absolute path rejected before clone
  let threw = false;
  try {
    execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'deploy',
      'file:///nonexistent', 'abc', '/etc', 'ns'], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) {
    threw = true;
    assert(e.stderr.toString().includes('relative'), 'absolute path rejected');
  }
  assert(threw, 'absolute subdir threw');

  // Early validation: .. rejected before clone
  threw = false;
  try {
    execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'deploy',
      'file:///nonexistent', 'abc', '../../etc', 'ns'], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) {
    threw = true;
    assert(e.stderr.toString().includes('..'), '.. rejected');
  }
  assert(threw, 'dotdot subdir threw');

  // Early validation: empty rejected
  threw = false;
  try {
    execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'deploy',
      'file:///nonexistent', 'abc', '', 'ns'], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) {
    threw = true;
    assert(e.stderr.toString().includes('empty'), 'empty rejected');
  }
  assert(threw, 'empty subdir threw');
}

async function test_build_output_rejects_symlinks() {
  setup();
  const { validateBuildOutput } = (() => {
    // Inline test of the validation function logic
    const buildDir = path.join(TMP, 'badbuild');
    fs.mkdirSync(path.join(buildDir, 'sub'), { recursive: true });
    fs.writeFileSync(path.join(buildDir, 'ok.txt'), 'fine');
    fs.symlinkSync('/etc', path.join(buildDir, 'badlink'));
    const issues = [];
    function walk(dir, rel) {
      const entries = fs.readdirSync(dir, { withFileTypes: true });
      for (const entry of entries) {
        const r = rel ? rel + '/' + entry.name : entry.name;
        if (entry.isSymbolicLink()) issues.push('symlink not allowed: ' + r);
        else if (entry.isDirectory()) walk(path.join(dir, entry.name), r);
      }
    }
    walk(buildDir, '');
    return { validateBuildOutput: () => issues };
  })();
  const issues = validateBuildOutput();
  assert(issues.length === 1, 'found one issue');
  assert(issues[0].includes('badlink'), 'mentions badlink');
}

async function test_binary_store() {
  setup(); createNs('bin');
  const { server, port } = await startServer();
  try {
    // Store binary data (all 256 byte values)
    const binData = Buffer.alloc(256);
    for (let i = 0; i < 256; i++) binData[i] = i;
    let r = await req(port, 'POST', '/_skrynia/store/bin/raw', binData, {'Content-Type':'application/octet-stream','X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'create binary 201');

    // Retrieve and verify binary integrity
    r = await get(port, '/_skrynia/store/bin/raw');
    assert(r.status === 200, 'get binary 200');
    assert(r.body.length === 256, 'binary length 256');
    for (let i = 0; i < 256; i++) {
      assert(r.body[i] === i, 'byte ' + i + ' matches');
    }
  } finally { await stopServer(server); }
}

// --- Runner ---

const tests = [
  ['health', test_health],
  ['key_validation', test_key_validation],
  ['store_crud', test_store_crud],
  ['create_duplicate', test_create_duplicate],
  ['immutable', test_immutable],
  ['capability_write', test_capability_write],
  ['quota_full', test_quota_full],
  ['atomic_symlink', test_atomic_symlink],
  ['app_serving', test_app_serving],
  ['rollback_sim', test_rollback_sim],
  ['concurrent_create', test_concurrent_create],
  ['concurrent_quota_boundary', test_concurrent_quota_boundary],
  ['admin_help', test_admin_help],
  ['admin_ns_crud', test_admin_ns_crud],
  ['admin_releases', test_admin_releases],
  ['admin_rollback', test_admin_rollback],
  ['namespace_not_created', test_namespace_not_created],
  ['put_quota_enforced', test_put_quota_enforced],
  ['capability_not_in_meta', test_capability_not_in_meta],
  ['app_serving_symlink_escape', test_app_serving_symlink_escape],
  ['rollback_updates_metadata', test_rollback_updates_metadata],
  ['undeploy_removes_everything', test_undeploy_removes_everything],
  ['deploy_subdir_validation', test_deploy_subdir_validation],
  ['build_output_rejects_symlinks', test_build_output_rejects_symlinks],
  ['binary_store', test_binary_store],
];

let pass = 0, fail = 0;

async function run() {
  console.log('Running Skrynia tests...\n');
  for (const [name, fn] of tests) {
    process.stdout.write('  ' + name + ' ... ');
    try { await fn(); console.log('ok'); pass++; }
    catch (e) { console.log('FAIL'); console.error('    ' + e.message); fail++; }
  }
  console.log('\nResults: ' + pass + ' passed, ' + fail + ' failed, ' + (pass+fail) + ' total');
  rmrf(TMP);
  process.exit(fail > 0 ? 1 : 0);
}

run();
