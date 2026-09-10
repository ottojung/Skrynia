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
    server.listen(0, '127.0.0.1', () => {
      resolve({ server, port: server.address().port });
    });
  });
}

function stopServer(s) {
  return new Promise(resolve => { s.close(() => resolve()); });
}

function get(port, urlPath) {
  return new Promise((ok, fail) => {
    http.get({ hostname: '127.0.0.1', port, path: urlPath, timeout: 3000 }, res => {
      let body = '';
      res.on('data', c => body += c);
      res.on('end', () => ok({ status: res.statusCode, body, headers: res.headers }));
    }).on('error', fail).on('timeout', function() { this.destroy(); fail(new Error('timeout')); });
  });
}

function req(port, method, urlPath, data, headers) {
  return new Promise((ok, fail) => {
    const opts = { hostname: '127.0.0.1', port, path: urlPath, method, headers: headers || {}, timeout: 3000 };
    if (data != null) opts.headers['Content-Length'] = Buffer.byteLength(data);
    const r = http.request(opts, res => {
      let body = '';
      res.on('data', c => body += c);
      res.on('end', () => ok({ status: res.statusCode, body, headers: res.headers }));
    });
    r.on('error', fail).on('timeout', () => { r.destroy(); fail(new Error('timeout')); });
    if (data != null) r.write(data);
    r.end();
  });
}

function assert(cond, msg) { if (!cond) throw new Error('ASSERT: ' + msg); }

// --- Tests ---

async function test_health() {
  setup();
  const { server, port } = await startServer();
  try {
    const r = await get(port, '/_skrynia/health');
    assert(r.status === 200, 'status 200');
    assert(JSON.parse(r.body).ok === true, 'ok=true');
  } finally { await stopServer(server); }
}

async function test_key_validation() {
  setup();
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/_skrynia/store/ns1/a%2F..%2Fb');
    assert(r.status === 400, 'traversal blocked, got ' + r.status);
    r = await get(port, '/_skrynia/store/ns1/..%2Fetc%2Fpasswd');
    assert(r.status === 400, 'encoded traversal blocked, got ' + r.status);
    r = await get(port, '/_skrynia/store/ns1/hello');
    assert(r.status === 404, 'valid key not_found');
  } finally { await stopServer(server); }
}

async function test_store_crud() {
  setup();
  const { server, port } = await startServer();
  try {
    let r = await req(port, 'POST', '/_skrynia/store/ns1/hello', 'world', {'Content-Type':'text/plain','X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'create status 201');
    assert(JSON.parse(r.body).ok === true, 'create ok');

    r = await get(port, '/_skrynia/store/ns1/hello');
    assert(r.status === 200, 'get status 200');
    assert(r.body === 'world', 'get body matches');

    r = await req(port, 'PUT', '/_skrynia/store/ns1/hello', 'updated', {'Content-Type':'text/plain'});
    assert(r.status === 200, 'put status 200');

    r = await get(port, '/_skrynia/store/ns1/hello');
    assert(r.body === 'updated', 'put body matches');

    r = await req(port, 'DELETE', '/_skrynia/store/ns1/hello', null, {});
    assert(r.status === 200, 'delete status 200');

    r = await get(port, '/_skrynia/store/ns1/hello');
    assert(r.status === 404, 'deleted not_found');
  } finally { await stopServer(server); }
}

async function test_create_duplicate() {
  setup();
  const { server, port } = await startServer();
  try {
    await req(port, 'POST', '/_skrynia/store/ns/k', 'v1', {'X-Skrynia-Mode':'public-write'});
    const r = await req(port, 'POST', '/_skrynia/store/ns/k', 'v2', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 409, 'duplicate 409');
  } finally { await stopServer(server); }
}

async function test_immutable() {
  setup();
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
  setup();
  const { server, port } = await startServer();
  try {
    let r = await req(port, 'POST', '/_skrynia/store/ns/ck', 'data', {'X-Skrynia-Mode':'capability-write'});
    assert(r.status === 201, 'create 201');
    const j = JSON.parse(r.body);
    assert(j.capability && j.capability.length === 64, 'capability returned');
    const cap = j.capability;

    r = await req(port, 'PUT', '/_skrynia/store/ns/ck', 'hack', {'Content-Type':'text/plain'});
    assert(r.status === 403, 'put no cap 403');

    r = await req(port, 'PUT', '/_skrynia/store/ns/ck', 'hack', {'Content-Type':'text/plain', 'X-Skrynia-Capability':'wrong'});
    assert(r.status === 403, 'put bad cap 403');

    r = await req(port, 'PUT', '/_skrynia/store/ns/ck', 'legit', {'Content-Type':'text/plain', 'X-Skrynia-Capability': cap});
    assert(r.status === 200, 'put good cap 200');

    r = await get(port, '/_skrynia/store/ns/ck');
    assert(r.body === 'legit', 'put verified');

    r = await req(port, 'DELETE', '/_skrynia/store/ns/ck', null, {});
    assert(r.status === 403, 'del no cap 403');

    r = await req(port, 'DELETE', '/_skrynia/store/ns/ck', null, {'X-Skrynia-Capability': cap});
    assert(r.status === 200, 'del good cap 200');
  } finally { await stopServer(server); }
}

async function test_quota_full() {
  setup();
  const { server, port } = await startServer();
  try {
    const qDir = path.join(TMP, 'state', 'tq');
    fs.mkdirSync(qDir, { recursive: true });
    fs.writeFileSync(path.join(qDir, 'quota.json'), JSON.stringify({bytes:0, count:0, quotaBytes:10, maxObjects:2}));
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
  const tmp = link + '.tmp';
  fs.symlinkSync(path.join(relBase, 'r1'), tmp);
  fs.renameSync(tmp, link);
  assert(fs.readlinkSync(link).endsWith('/r1'), 'link to r1');

  const tmp2 = link + '.tmp';
  fs.symlinkSync(path.join(relBase, 'r2'), tmp2);
  fs.renameSync(tmp2, link);
  assert(fs.readlinkSync(link).endsWith('/r2'), 'link to r2');
  assert(fs.existsSync(path.join(relBase, 'r1', 'index.html')), 'r1 still exists');
}

async function test_app_serving() {
  setup();
  const relBase = path.join(TMP, 'releases', 'webapp');
  fs.mkdirSync(relBase, { recursive: true });
  fs.mkdirSync(path.join(relBase, 'assets'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'index.html'), '<h1>Hello</h1>');
  fs.writeFileSync(path.join(relBase, 'assets', 'app.js'), 'console.log("hi")');
  fs.symlinkSync(relBase, path.join(relBase, '..', 'webapp', 'current'));

  const { server, port } = await startServer();
  try {
    let r = await get(port, '/a/webapp/');
    assert(r.status === 200, 'index 200');
    assert(r.body === '<h1>Hello</h1>', 'index body');
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
  const t1 = link + '.tmp';
  fs.symlinkSync(path.join(relBase, '20260910130000'), t1);
  fs.renameSync(t1, link);

  const t2 = link + '.tmp';
  fs.symlinkSync(path.join(relBase, '20260910120000'), t2);
  fs.renameSync(t2, link);
  assert(fs.readlinkSync(link).includes('20260910120000'), 'rolled back');
}

async function test_concurrent_create() {
  setup();
  const { server, port } = await startServer();
  try {
    // Fire 10 concurrent creates to the same key.
    // Exactly one must succeed (201), rest must be 409 (already_exists).
    const promises = [];
    for (let i = 0; i < 10; i++) {
      promises.push(req(port, 'POST', '/_skrynia/store/cct/same', 'data' + i, {'X-Skrynia-Mode':'public-write'}));
    }
    const results = await Promise.all(promises);
    const statuses = results.map(r => r.status).sort();
    const okCount = statuses.filter(s => s === 201).length;
    const conflictCount = statuses.filter(s => s === 409).length;
    assert(okCount === 1, 'exactly one create succeeded, got ' + okCount);
    assert(conflictCount === 9, 'nine conflicts, got ' + conflictCount);
    // Verify data is valid (one of the submitted values)
    const r = await get(port, '/_skrynia/store/cct/same');
    assert(r.status === 200, 'object exists');
    assert(r.body.startsWith('data'), 'data present');
  } finally { await stopServer(server); }
}

async function test_concurrent_quota_boundary() {
  setup();
  const { server, port } = await startServer();
  try {
    // Set quota to exactly allow 3 objects of 5 bytes each
    const qDir = path.join(TMP, 'state', 'cqb');
    fs.mkdirSync(qDir, { recursive: true });
    fs.writeFileSync(path.join(qDir, 'quota.json'), JSON.stringify({bytes:0, count:0, quotaBytes:15, maxObjects:10}));

    // Fire 5 concurrent creates (each 5 bytes = 25 bytes total, quota is 15)
    // Some must succeed, some must fail with quota exceeded
    const promises = [];
    for (let i = 0; i < 5; i++) {
      promises.push(req(port, 'POST', '/_skrynia/store/cqb/k' + i, 'bytes!', {'X-Skrynia-Mode':'public-write'}));
    }
    const results = await Promise.all(promises);
    const statuses = results.map(r => r.status).sort();
    const okCount = statuses.filter(s => s === 201).length;
    const fullCount = statuses.filter(s => s === 507).length;
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
  // No release directories, so only the header
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
];

let pass = 0, fail = 0;

async function run() {
  console.log('Running Skrynia tests...\n');
  for (const [name, fn] of tests) {
    process.stdout.write('  ' + name + ' ... ');
    try {
      await fn();
      console.log('ok');
      pass++;
    } catch (e) {
      console.log('FAIL');
      console.error('    ' + e.message);
      fail++;
    }
  }
  console.log('\nResults: ' + pass + ' passed, ' + fail + ' failed, ' + (pass+fail) + ' total');
  rmrf(TMP);
  process.exit(fail > 0 ? 1 : 0);
}

run();
