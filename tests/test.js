#!/usr/bin/env node
'use strict';

const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execFileSync } = require('child_process');
const { createServer } = require('../src/server.js');
const { normalizeBasePath } = require('../src/base-path.js');

const SRC = path.join(__dirname, '..');
const NODE = process.execPath;
const TMP = '/tmp/skrynia-test';
const FAKE_BIN = path.join(os.homedir(), '.skrynia-test-bin');

function rmrf(d) { try { fs.rmSync(d, { recursive: true, force: true }); } catch {} }

function setup() {
  rmrf(TMP);
  fs.mkdirSync(path.join(TMP, 'releases'), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'storage'), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'state'), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'builds'), { recursive: true });
  rmrf(FAKE_BIN);
  fs.mkdirSync(FAKE_BIN, { recursive: true });
}

function startServer(extraOpts) {
  const opts = Object.assign({ dataDir: TMP }, extraOpts || {});
  const server = createServer(opts);
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

function makeFakeDocker(body) {
  const script = path.join(FAKE_BIN, 'docker');
  fs.writeFileSync(script, [
    '#!/bin/sh',
    'REPO=""',
    'while [ $# -gt 0 ]; do',
    '  case "$1" in',
    '    -v)',
    '      val="$2"',
    '      host=$(echo "$val" | cut -d: -f1)',
    '      cont=$(echo "$val" | cut -d: -f2)',
    '      if [ "$cont" = "/repo" ]; then REPO="$host"; fi',
    '      shift 2',
    '      ;;',
    '    *)',
    '      shift',
    '      ;;',
    '  esac',
    'done',
  ].concat(body).join('\n'));
  fs.chmodSync(script, 0o755);
}

function deployEnv() {
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  env.PATH = FAKE_BIN + ':' + (env.PATH || '/usr/bin:/bin');
  return env;
}

function deployArgs(repo, commit, subdir, ns) {
  return [
    path.join(SRC, 'src', 'admin.js'), 'deploy',
    '--repo', repo, '--commit', commit, '--subdir', subdir, '--namespace', ns,
  ];
}

function makeRepo(dir, files) {
  fs.mkdirSync(dir, { recursive: true });
  execFileSync('git', ['init', dir], { timeout: 5000, stdio: 'pipe' });
  execFileSync('git', ['-C', dir, 'config', 'user.email', 'test@test.com'], { timeout: 3000, stdio: 'pipe' });
  execFileSync('git', ['-C', dir, 'config', 'user.name', 'Test'], { timeout: 3000, stdio: 'pipe' });
  for (const [name, content] of Object.entries(files)) {
    const fp = path.join(dir, name);
    fs.mkdirSync(path.dirname(fp), { recursive: true });
    fs.writeFileSync(fp, content);
  }
  execFileSync('git', ['-C', dir, 'add', '.'], { timeout: 3000, stdio: 'pipe' });
  execFileSync('git', ['-C', dir, 'commit', '-m', 'init'], { timeout: 5000, stdio: 'pipe' });
  return execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD'], { timeout: 3000, stdio: 'pipe' }).toString().trim();
}

// --- Server tests ---

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
  fs.mkdirSync(path.join(TMP, 'state', 'tq'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', 'tq', 'quota.json'), JSON.stringify({bytes:0,count:0,quotaBytes:10,maxObjects:2}));
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
    let r = await get(port, '/apps/webapp/');
    assert(r.status === 200, 'index 200');
    assert(r.text === '<h1>Hello</h1>', 'index body');
    r = await get(port, '/apps/webapp/assets/app.js');
    assert(r.status === 200, 'js 200');
    assert(r.headers['content-type'] === 'application/javascript', 'js type');
    r = await get(port, '/apps/webapp/../../etc/passwd');
    assert(r.status >= 400, 'traversal blocked');
  } finally { await stopServer(server); }
}

async function test_app_serving_query_string() {
  setup(); createNs('qsapp');
  const relBase = path.join(TMP, 'releases', 'qsapp');
  fs.mkdirSync(relBase, { recursive: true });
  fs.writeFileSync(path.join(relBase, 'app.js'), 'console.log("qs")');
  fs.symlinkSync(relBase, path.join(relBase, '..', 'qsapp', 'current'));
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/apps/qsapp/app.js?x=1');
    assert(r.status === 200, 'qs file 200');
    assert(r.text === 'console.log("qs")', 'qs body');
    assert(r.headers['content-type'] === 'application/javascript', 'qs content-type');
  } finally { await stopServer(server); }
}

async function test_app_serving_custom_base_path() {
  setup(); createNs('myapp');
  const relBase = path.join(TMP, 'releases', 'myapp');
  fs.mkdirSync(relBase, { recursive: true });
  fs.writeFileSync(path.join(relBase, 'index.html'), '<p>custom</p>');
  fs.symlinkSync(relBase, path.join(relBase, '..', 'myapp', 'current'));
  const { server, port } = await startServer({ appBasePath: '/myapps' });
  try {
    let r = await get(port, '/myapps/myapp/');
    assert(r.status === 200, 'custom base 200');
    assert(r.text === '<p>custom</p>', 'custom base body');
    r = await get(port, '/apps/myapp/');
    assert(r.status === 404, 'default base 404');
  } finally { await stopServer(server); }
}

async function test_app_dir_serving() {
  setup(); createNs('dapp');
  const relBase = path.join(TMP, 'releases', 'dapp');
  fs.mkdirSync(relBase, { recursive: true });
  fs.writeFileSync(path.join(relBase, 'index.html'), '<p>from appdir</p>');
  const appDir = path.join(TMP, 'app-expose');
  fs.mkdirSync(appDir, { recursive: true });
  // Unified: active link lives in APP_DIR, not in releases/current
  fs.symlinkSync(relBase, path.join(appDir, 'dapp'));
  const { server, port } = await startServer({ appDir });
  try {
    let r = await get(port, '/apps/dapp/');
    assert(r.status === 200, 'appdir 200');
    assert(r.text === '<p>from appdir</p>', 'appdir body');
  } finally { await stopServer(server); }
}

async function test_app_dir_symlink_atomic() {
  setup(); createNs('atomicns');
  const relBase = path.join(TMP, 'releases', 'atomicns');
  fs.mkdirSync(path.join(relBase, 'r1'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r1', 'index.html'), 'v1');
  fs.mkdirSync(path.join(relBase, 'r2'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r2', 'index.html'), 'v2');
  const appDir = path.join(TMP, 'app-expose-atomic');
  fs.mkdirSync(appDir, { recursive: true });
  // Deploy: atomic symlink in APP_DIR (no current in releases)
  const appLink = path.join(appDir, 'atomicns');
  const tmpLink = appLink + '.tmp';
  fs.symlinkSync(path.join(relBase, 'r2'), tmpLink);
  fs.renameSync(tmpLink, appLink);
  assert(fs.readlinkSync(appLink).includes('r2'), 'appdir points to r2');
  assert(!fs.existsSync(path.join(relBase, 'current')), 'no current in releases');
  // Rollback: atomic swap in APP_DIR
  const tmpLink2 = appLink + '.tmp';
  fs.symlinkSync(path.join(relBase, 'r1'), tmpLink2);
  fs.renameSync(tmpLink2, appLink);
  assert(fs.readlinkSync(appLink).includes('r1'), 'appdir rolled back to r1');
}

async function test_rollback_sim() {
  setup();
  const relBase = path.join(TMP, 'releases', 'rbapp');
  fs.mkdirSync(path.join(relBase, 'r1'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r1', 'index.html'), 'v1');
  fs.mkdirSync(path.join(relBase, 'r2'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'r2', 'index.html'), 'v2');
  const link = path.join(relBase, 'current');
  let t = link + '.tmp';
  fs.symlinkSync(path.join(relBase, 'r2'), t); fs.renameSync(t, link);
  t = link + '.tmp';
  fs.symlinkSync(path.join(relBase, 'r1'), t); fs.renameSync(t, link);
  assert(fs.readlinkSync(link).includes('r1'), 'rolled back');
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
    assert(results.filter(r => r.status === 201).length === 1, 'one create');
    assert(results.filter(r => r.status === 409).length === 9, 'nine conflicts');
  } finally { await stopServer(server); }
}

async function test_concurrent_quota_boundary() {
  setup();
  fs.mkdirSync(path.join(TMP, 'state', 'cqb'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', 'cqb', 'quota.json'), JSON.stringify({bytes:0,count:0,quotaBytes:15,maxObjects:10}));
  const { server, port } = await startServer();
  try {
    const promises = [];
    for (let i = 0; i < 5; i++) {
      promises.push(req(port, 'POST', '/_skrynia/store/cqb/k' + i, 'bytes!', {'X-Skrynia-Mode':'public-write'}));
    }
    const results = await Promise.all(promises);
    const ok = results.filter(r => r.status === 201).length;
    const full = results.filter(r => r.status === 507).length;
    assert(ok + full === 5, 'all accounted for');
    assert(ok >= 1 && full >= 1, 'boundary hit');
  } finally { await stopServer(server); }
}

async function test_admin_help() {
  const out = execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'help'], { timeout: 3000 });
  assert(out.toString().includes('Usage:'), 'help has Usage');
}

async function test_admin_ns_crud() {
  setup();
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'ns', 'create', '--namespace', 'mytest', '--quota', '1024'], { timeout: 3000, env });
  assert(fs.existsSync(path.join(TMP, 'state', 'mytest', 'quota.json')), 'ns created');
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'ns', 'remove', '--namespace', 'mytest'], { timeout: 3000, env });
  assert(!fs.existsSync(path.join(TMP, 'storage', 'mytest')), 'ns removed');
}

async function test_admin_releases() {
  setup();
  fs.mkdirSync(path.join(TMP, 'releases', 'emptyns'), { recursive: true });
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  const out = execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'releases', '--namespace', 'emptyns'], { timeout: 3000, env });
  assert(out.toString().includes('Releases:'), 'has Releases header');
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
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'rollback', '--namespace', 'rbns', '--release', 'r1'], { timeout: 3000, env });
  assert(fs.readlinkSync(path.join(relBase, 'current')).includes('r1'), 'rolled back via admin');
}

async function test_namespace_not_created() {
  setup();
  const { server, port } = await startServer();
  try {
    let r = await req(port, 'POST', '/_skrynia/store/ghost/k', 'v', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 409, 'POST uncreated ns 409, got ' + r.status);
    r = await get(port, '/_skrynia/store/ghost/k');
    assert(r.status === 404, 'GET uncreated ns 404');
  } finally { await stopServer(server); }
}

async function test_put_quota_enforced() {
  setup();
  fs.mkdirSync(path.join(TMP, 'state', 'pqt'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', 'pqt', 'quota.json'), JSON.stringify({bytes:0,count:0,quotaBytes:10,maxObjects:100}));
  const { server, port } = await startServer();
  try {
    await req(port, 'POST', '/_skrynia/store/pqt/k', 'hi', {'X-Skrynia-Mode':'public-write'});
    let r = await req(port, 'PUT', '/_skrynia/store/pqt/k', 'x'.repeat(20), {'Content-Type':'text/plain'});
    assert(r.status === 507, 'PUT exceeds quota 507');
    r = await get(port, '/_skrynia/store/pqt/k');
    assert(r.text === 'hi', 'original preserved');
  } finally { await stopServer(server); }
}

async function test_capability_not_in_meta() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    await req(port, 'POST', '/_skrynia/store/ns/ck2', 'data', {'X-Skrynia-Mode':'capability-write'});
    const meta = JSON.parse(fs.readFileSync(path.join(TMP, 'storage', 'ns', 'ck2.meta'), 'utf8'));
    assert(!meta.capVerifier, 'verifier not in meta');
    assert(fs.existsSync(path.join(TMP, 'storage', 'ns', 'ck2.cap')), '.cap file exists');
  } finally { await stopServer(server); }
}

async function test_app_serving_symlink_escape() {
  setup(); createNs('escapeapp');
  const relBase = path.join(TMP, 'releases', 'escapeapp');
  fs.mkdirSync(path.join(relBase, 'public'), { recursive: true });
  fs.writeFileSync(path.join(relBase, 'public', 'ok.html'), 'safe');
  // Symlink escape under public/
  fs.symlinkSync('/etc', path.join(relBase, 'public', 'escape'));
  // Symlink escape at root level
  fs.symlinkSync('/etc', path.join(relBase, 'escape'));
  fs.symlinkSync(relBase, path.join(relBase, '..', 'escapeapp', 'current'));
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/apps/escapeapp/public/ok.html');
    assert(r.status === 200, 'normal file 200');
    r = await get(port, '/apps/escapeapp/public/escape/passwd');
    assert(r.status === 403, 'symlink escape 403, got ' + r.status);
    r = await get(port, '/apps/escapeapp/public/escape');
    assert(r.status === 403, 'symlink itself 403, got ' + r.status);
    // Root-level escape
    r = await get(port, '/apps/escapeapp/escape/passwd');
    assert(r.status === 403, 'root escape 403, got ' + r.status);
    r = await get(port, '/apps/escapeapp/escape');
    assert(r.status === 403, 'root symlink itself 403, got ' + r.status);
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
  const cfgPath = path.join(TMP, 'state', 'rbmeta', 'config.json');
  fs.mkdirSync(path.dirname(cfgPath), { recursive: true });
  fs.writeFileSync(cfgPath, JSON.stringify({namespace:'rbmeta',currentReleaseId:'r2'}, null, 2));
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'rollback', '--namespace', 'rbmeta', '--release', 'r1'], { timeout: 3000, env });
  const cfg = JSON.parse(fs.readFileSync(cfgPath, 'utf8'));
  assert(cfg.currentReleaseId === 'r1', 'config updated to r1');
  assert(cfg.lastRollbackAt, 'rollback timestamp recorded');
}

async function test_undeploy_removes_everything() {
  setup(); createNs('udns');
  const relDir = path.join(TMP, 'releases', 'udns');
  fs.mkdirSync(path.join(relDir, 'r1'), { recursive: true });
  fs.writeFileSync(path.join(relDir, 'r1', 'index.html'), 'hi');
  fs.symlinkSync(path.join(relDir, 'r1'), path.join(relDir, 'current'));
  fs.mkdirSync(path.join(TMP, 'storage', 'udns'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'storage', 'udns', 'data.bin'), 'payload');
  fs.writeFileSync(path.join(TMP, 'state', 'udns', 'config.json'), JSON.stringify({namespace:'udns',currentReleaseId:'r1'}));
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'undeploy', '--namespace', 'udns'], { timeout: 3000, env });
  assert(!fs.existsSync(relDir), 'releases removed');
  assert(!fs.existsSync(path.join(TMP, 'storage', 'udns')), 'storage removed');
  assert(!fs.existsSync(path.join(TMP, 'state', 'udns')), 'state removed');
}

async function test_binary_store() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    const binData = Buffer.alloc(256);
    for (let i = 0; i < 256; i++) binData[i] = i;
    let r = await req(port, 'POST', '/_skrynia/store/ns/raw', binData, {'Content-Type':'application/octet-stream','X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'create binary 201');
    r = await get(port, '/_skrynia/store/ns/raw');
    assert(r.status === 200 && r.body.length === 256, 'binary 256 bytes');
    for (let i = 0; i < 256; i++) assert(r.body[i] === i, 'byte ' + i);
  } finally { await stopServer(server); }
}

async function test_key_rejects_slash() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/_skrynia/store/ns/foo/bar');
    assert(r.status === 400, 'slash rejected, got ' + r.status);
    r = await get(port, '/_skrynia/store/ns/a%2Fb');
    assert(r.status === 400, 'encoded slash rejected');
  } finally { await stopServer(server); }
}

async function test_namespace_validation() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/_skrynia/store/INVALID/key');
    assert(r.status === 400, 'uppercase ns rejected');
    r = await get(port, '/_skrynia/store/my%20ns/key');
    assert(r.status === 400, 'space in ns rejected');
    r = await get(port, '/_skrynia/store/' + 'a'.repeat(100) + '/key');
    assert(r.status === 400, 'long ns rejected');
    r = await get(port, '/_skrynia/store/ns/key');
    assert(r.status === 404, 'valid ns accepted');
  } finally { await stopServer(server); }
}

async function test_malformed_pct_encoding() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    const r = await get(port, '/_skrynia/store/ns/%zz');
    assert(r.status === 400, 'malformed pct-enc returns 400, got ' + r.status);
  } finally { await stopServer(server); }
}

async function test_client_serving() {
  setup();
  const { server, port } = await startServer();
  try {
    // Client path may not exist in test env (not installed), but should return 404 not crash
    const r = await get(port, '/_skrynia/client/skrynia.js');
    assert(r.status === 404 || r.status === 200, 'client endpoint returns 404 or 200');
  } finally { await stopServer(server); }
}

async function test_missing_static_returns_404() {
  setup(); createNs('ns');
  const relBase = path.join(TMP, 'releases', 'ns');
  fs.mkdirSync(relBase, { recursive: true });
  fs.writeFileSync(path.join(relBase, 'index.html'), 'ok');
  fs.symlinkSync(relBase, path.join(relBase, '..', 'ns', 'current'));
  const { server, port } = await startServer();
  try {
    const r = await get(port, '/apps/ns/nonexistent.html');
    assert(r.status === 404, 'missing static 404, got ' + r.status);
  } finally { await stopServer(server); }
}

async function test_deploy_rejects_positional_args() {
  setup();
  const env = deployEnv();
  let threw = false;
  try {
    execFileSync(NODE, [
      path.join(SRC, 'src', 'admin.js'), 'deploy',
      'file:///repo', 'abc123', '.', 'myapp',
    ], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('missing required flags'), 'positional rejected'); }
  assert(threw, '4-positional threw');
}

async function test_deploy_requires_all_four_flags() {
  setup();
  const env = deployEnv();
  const combos = [
    ['--repo', 'x', '--commit', 'x', '--subdir', '.'],
    ['--repo', 'x', '--commit', 'x', '--namespace', 'ns'],
    ['--repo', 'x', '--subdir', '.', '--namespace', 'ns'],
    ['--commit', 'x', '--subdir', '.', '--namespace', 'ns'],
  ];
  for (const combo of combos) {
    let threw = false;
    try {
      execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'deploy', ...combo], { timeout: 3000, env, stdio: 'pipe' });
    } catch (e) { threw = true; assert(e.stderr.toString().includes('missing required flags'), 'flags combo rejected'); }
    assert(threw, 'combo threw: ' + combo.join(' '));
  }
}

async function test_deploy_rejects_non_hex_commit() {
  setup();
  const env = deployEnv();
  let threw = false;
  try {
    execFileSync(NODE, [
      path.join(SRC, 'src', 'admin.js'), 'deploy',
      '--repo', 'x', '--commit', 'not-a-sha', '--subdir', '.', '--namespace', 'ns',
    ], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('--commit must be'), 'commit validated'); }
  assert(threw, 'bad commit threw');
}

async function test_deploy_rejects_short_commit() {
  setup();
  const env = deployEnv();
  let threw = false;
  try {
    execFileSync(NODE, [
      path.join(SRC, 'src', 'admin.js'), 'deploy',
      '--repo', 'x', '--commit', 'abc123', '--subdir', '.', '--namespace', 'ns',
    ], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('--commit must be'), 'short commit rejected'); }
  assert(threw, 'short commit threw');
}

async function test_deploy_subdir_validation() {
  setup();
  const env = deployEnv();
  // Absolute path rejected
  let threw = false;
  try {
    execFileSync(NODE, [...deployArgs('file:///x', 'a'.repeat(40), '/etc', 'ns')], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('relative'), 'absolute rejected'); }
  assert(threw, 'absolute threw');
  // .. rejected
  threw = false;
  try {
    execFileSync(NODE, [...deployArgs('file:///x', 'a'.repeat(40), '../../etc', 'ns')], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('..'), '.. rejected'); }
  assert(threw, 'dotdot threw');
  // Empty rejected (flag present but empty value)
  threw = false;
  try {
    execFileSync(NODE, [...deployArgs('file:///x', 'a'.repeat(40), '', 'ns')], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('missing required flags') || e.stderr.toString().includes('empty'), 'empty rejected'); }
  assert(threw, 'empty threw');
}

async function test_build_output_rejects_symlinks() {
  const buildDir = path.join(TMP, 'badbuild');
  fs.mkdirSync(buildDir, { recursive: true });
  fs.writeFileSync(path.join(buildDir, 'ok.txt'), 'fine');
  fs.symlinkSync('/etc', path.join(buildDir, 'badlink'));
  const issues = [];
  function walk(dir, rel) {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const r = rel ? rel + '/' + e.name : e.name;
      if (e.isSymbolicLink()) issues.push('symlink: ' + r);
      else if (e.isDirectory()) walk(path.join(dir, e.name), r);
    }
  }
  walk(buildDir, '');
  assert(issues.length === 1 && issues[0].includes('badlink'), 'symlink rejected');
}

async function test_deploy_success_full() {
  setup();
  const env = deployEnv();
  const repoDir = path.join(TMP, 'gitrepo');
  const h = makeRepo(repoDir, { 'Makefile': 'build:\n\tmkdir -p build && echo hello > build/index.html\n' });
  makeFakeDocker(['cd "$REPO" && mkdir -p build && echo hello > build/index.html']);

  execFileSync(NODE, deployArgs('file://' + repoDir, h, '.', 'myapp'), { timeout: 30000, env });

  const relsBase = path.join(TMP, 'releases', 'myapp');
  assert(fs.existsSync(relsBase), 'releases dir exists');
  const releases = fs.readdirSync(relsBase).filter(d => !d.startsWith('.') && d !== 'current');
  assert(releases.length === 1, 'one release');
  assert(fs.readlinkSync(path.join(relsBase, 'current')).includes(releases[0]), 'current points to release');
  assert(fs.existsSync(path.join(relsBase, releases[0], 'index.html')), 'release has index.html');

  const cfg = JSON.parse(fs.readFileSync(path.join(TMP, 'state', 'myapp', 'config.json'), 'utf8'));
  assert(cfg.namespace === 'myapp' && cfg.commit === h, 'config correct');

  // Second deploy: unique release, namespace preserved
  fs.writeFileSync(path.join(repoDir, 'Makefile'), 'build:\n\tmkdir -p build && echo updated > build/index.html\n');
  execFileSync('git', ['-C', repoDir, 'add', '.'], { timeout: 3000, stdio: 'pipe' });
  execFileSync('git', ['-C', repoDir, 'commit', '-m', 'update'], { timeout: 5000, stdio: 'pipe' });
  const h2 = execFileSync('git', ['-C', repoDir, 'rev-parse', 'HEAD'], { timeout: 3000, stdio: 'pipe' }).toString().trim();
  execFileSync(NODE, deployArgs('file://' + repoDir, h2, '.', 'myapp'), { timeout: 30000, env });
  const releases2 = fs.readdirSync(relsBase).filter(d => !d.startsWith('.') && d !== 'current');
  assert(releases2.length === 2, 'two releases');
  assert(releases2[0] !== releases2[1], 'unique release IDs');
}

async function test_deploy_monorepo_subdir() {
  setup();
  const env = deployEnv();
  const repoDir = path.join(TMP, 'monorepo');
  const h = makeRepo(repoDir, { 'frontend/Makefile': 'build:\n\tmkdir -p build && echo fe > build/index.html\n' });
  makeFakeDocker(['cd "$REPO/frontend" && mkdir -p build && echo fe > build/index.html']);
  execFileSync(NODE, deployArgs('file://' + repoDir, h, 'frontend', 'webfront'), { timeout: 30000, env });
  const relsBase = path.join(TMP, 'releases', 'webfront');
  assert(fs.existsSync(relsBase), 'monorepo releases dir exists');
  assert(fs.readdirSync(relsBase).filter(d => !d.startsWith('.') && d !== 'current').length === 1, 'one release');
}

async function test_deploy_failure_cleanup() {
  setup();
  const env = deployEnv();
  const repoDir = path.join(TMP, 'failrepo');
  const h = makeRepo(repoDir, { 'Makefile': 'build:\n\tmkdir -p build && echo ok > build/index.html\n' });
  makeFakeDocker(['exit 1']);
  let threw = false;
  try {
    execFileSync(NODE, deployArgs('file://' + repoDir, h, '.', 'failns'), { timeout: 30000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('build failed'), 'build failed'); }
  assert(threw, 'deploy threw');
  assert(fs.readdirSync(path.join(TMP, 'builds')).filter(e => e.startsWith('build-')).length === 0, 'temp cleaned');
}

async function test_undeploy_destructive() {
  setup(); createNs('delpns');
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  const relDir = path.join(TMP, 'releases', 'delpns');
  fs.mkdirSync(path.join(relDir, 'r1'), { recursive: true });
  fs.writeFileSync(path.join(relDir, 'r1', 'index.html'), 'page');
  fs.symlinkSync(path.join(relDir, 'r1'), path.join(relDir, 'current'));
  fs.mkdirSync(path.join(TMP, 'storage', 'delpns'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'storage', 'delpns', 'file.bin'), 'data');
  fs.writeFileSync(path.join(TMP, 'state', 'delpns', 'config.json'), JSON.stringify({namespace:'delpns'}));
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'undeploy', '--namespace', 'delpns'], { timeout: 5000, env });
  assert(!fs.existsSync(relDir), 'releases gone');
  assert(!fs.existsSync(path.join(TMP, 'storage', 'delpns')), 'storage gone');
  assert(!fs.existsSync(path.join(TMP, 'state', 'delpns')), 'state gone');
}

async function test_deploy_auto_create_after_build() {
  setup();
  const env = deployEnv();
  const repoDir = path.join(TMP, 'autonsrepo');
  const h = makeRepo(repoDir, { 'Makefile': 'build:\n\tmkdir -p build && echo ok > build/index.html\n' });
  makeFakeDocker(['cd "$REPO" && mkdir -p build && echo ok > build/index.html']);

  // Namespace should NOT exist before deploy
  assert(!fs.existsSync(path.join(TMP, 'state', 'autons', 'quota.json')), 'ns does not exist before deploy');

  execFileSync(NODE, deployArgs('file://' + repoDir, h, '.', 'autons'), { timeout: 30000, env });

  // Namespace should exist after successful deploy
  assert(fs.existsSync(path.join(TMP, 'state', 'autons', 'quota.json')), 'ns auto-created after build');

  // Redeploy preserves quota
  const q1 = JSON.parse(fs.readFileSync(path.join(TMP, 'state', 'autons', 'quota.json'), 'utf8'));
  execFileSync(NODE, deployArgs('file://' + repoDir, h, '.', 'autons'), { timeout: 30000, env });
  const q2 = JSON.parse(fs.readFileSync(path.join(TMP, 'state', 'autons', 'quota.json'), 'utf8'));
  assert(q2.quotaBytes === q1.quotaBytes, 'quota preserved on redeploy');
}

async function test_deploy_failure_no_ns_created() {
  setup();
  const env = deployEnv();
  const repoDir = path.join(TMP, 'failnsrepo');
  const h = makeRepo(repoDir, { 'Makefile': 'build:\n\tmkdir -p build && echo ok > build/index.html\n' });
  makeFakeDocker(['exit 1']);

  let threw = false;
  try {
    execFileSync(NODE, deployArgs('file://' + repoDir, h, '.', 'failns'), { timeout: 30000, env, stdio: 'pipe' });
  } catch (e) { threw = true; }
  assert(threw, 'deploy threw');

  // Namespace should NOT exist (auto-create is after build validation)
  assert(!fs.existsSync(path.join(TMP, 'state', 'failns', 'quota.json')), 'ns not created on failure');
}

async function test_deploy_invalid_namespace() {
  setup();
  const env = deployEnv();
  let threw = false;
  try {
    execFileSync(NODE, [...deployArgs('file:///x', 'a'.repeat(40), '.', 'INVALID_NS')], { timeout: 5000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('invalid namespace'), 'invalid ns'); }
  assert(threw, 'invalid ns threw');
}

async function test_admin_ns_preserves_quota() {
  setup();
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'ns', 'create', '--namespace', 'pqns', '--quota', '999'], { timeout: 3000, env });
  assert(JSON.parse(fs.readFileSync(path.join(TMP, 'state', 'pqns', 'quota.json'), 'utf8')).quotaBytes === 999, 'first create 999');
  execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'ns', 'create', '--namespace', 'pqns', '--quota', '5000'], { timeout: 3000, env });
  assert(JSON.parse(fs.readFileSync(path.join(TMP, 'state', 'pqns', 'quota.json'), 'utf8')).quotaBytes === 999, 'preserved 999');
}

async function test_examples_hello_make_build() {
  // Verify examples/hello can produce build/index.html
  const helloDir = path.join(SRC, 'examples', 'hello');
  const buildDir = path.join(helloDir, 'build');
  try { rmrf(buildDir); } catch {}
  execFileSync('make', ['build'], { cwd: helloDir, timeout: 5000, stdio: 'pipe' });
  assert(fs.existsSync(path.join(buildDir, 'index.html')), 'examples/hello build/index.html exists');
  const content = fs.readFileSync(path.join(buildDir, 'index.html'), 'utf8');
  assert(content.includes('Hello Skrynia'), 'build/index.html has expected content');
  rmrf(buildDir);
}

async function test_undeploy_rejects_positional() {
  setup();
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  let threw = false;
  try {
    execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'undeploy', 'myns'], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('missing required flag') || e.stderr.toString().includes('positional'), 'positional rejected'); }
  assert(threw, 'positional undeploy threw');
}

async function test_rollback_rejects_positional() {
  setup();
  const env = Object.assign({}, process.env, { SKRYNIA_DATA_DIR: TMP });
  let threw = false;
  try {
    execFileSync(NODE, [path.join(SRC, 'src', 'admin.js'), 'rollback', 'myns'], { timeout: 3000, env, stdio: 'pipe' });
  } catch (e) { threw = true; assert(e.stderr.toString().includes('missing required flag') || e.stderr.toString().includes('positional'), 'positional rejected'); }
  assert(threw, 'positional rollback threw');
}

async function test_readBody_oversized_413() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    const big = Buffer.alloc(10485761, 0x41);
    const r = await req(port, 'POST', '/_skrynia/store/ns/big', big, {'Content-Type':'application/octet-stream','X-Skrynia-Mode':'public-write'});
    assert(r.status === 413, 'oversized returns 413, got ' + r.status);
    const j = JSON.parse(r.text);
    assert(j.error === 'request_too_large', 'error key correct');
  } finally { await stopServer(server); }
}

async function test_release_timestamp_millis() {
  setup();
  const env = deployEnv();
  const repoDir = path.join(TMP, 'tsrepo');
  const h = makeRepo(repoDir, { 'Makefile': 'build:\n\tmkdir -p build && echo ok > build/index.html\n' });
  makeFakeDocker(['cd "$REPO" && mkdir -p build && echo ok > build/index.html']);
  execFileSync(NODE, deployArgs('file://' + repoDir, h, '.', 'tsns'), { timeout: 30000, env });
  const releases = fs.readdirSync(path.join(TMP, 'releases', 'tsns')).filter(d => !d.startsWith('.') && d !== 'current');
  assert(releases.length === 1, 'one release');
  const rid = releases[0];
  assert(/^\d{17}-[0-9a-f]{6}$/.test(rid), 'release id has 17-digit timestamp + 6-hex random, got: ' + rid);
  const tsPart = rid.split('-')[0];
  assert(tsPart.length === 17, 'timestamp part 17 chars');
  assert(/^\d{8}\d{6}\d{3}$/.test(tsPart), 'YYYYMMDDHHmmssSSS format');
}

async function test_docker_security_opts() {
  // Verify admin.js uses correct Docker security flags:
  // --security-opt no-new-privileges (not standalone --no-new-privileges)
  const src = fs.readFileSync(path.join(SRC, 'src', 'admin.js'), 'utf8');
  // Must NOT have standalone --no-new-privileges (would be flag with dash prefix alone)
  assert(!src.includes("'--no-new-privileges'"), 'must not use standalone --no-new-privileges');
  // Must have --security-opt followed by no-new-privileges
  assert(src.includes("'--security-opt', 'no-new-privileges'"), 'must use --security-opt no-new-privileges');
  // Must have --cap-drop ALL
  assert(src.includes("'--cap-drop', 'ALL'"), 'must have --cap-drop ALL');
  // Must have --read-only
  assert(src.includes("'--read-only'"), 'must have --read-only');
  // Must have --user with owner uid:gid
  assert(src.includes("'--user'"), 'must have --user flag');
  assert(src.includes('owner.uid') && src.includes('owner.gid'), 'must reference owner uid and gid');
}

async function test_base_path_strips_trailing_slashes() {
  assert(normalizeBasePath('/apps/') === '/apps', 'strips single trailing slash');
  assert(normalizeBasePath('/apps//') === '/apps', 'strips multiple trailing slashes');
  assert(normalizeBasePath('/apps///') === '/apps', 'strips many trailing slashes');
}

async function test_base_path_keeps_root() {
  assert(normalizeBasePath('/') === '/', 'root kept as-is');
  assert(normalizeBasePath('///') === '/', 'multiple slashes on root normalizes to root');
}

async function test_base_path_rejects_no_leading_slash() {
  let threw = false;
  try { normalizeBasePath('apps'); } catch { threw = true; }
  assert(threw, 'rejects missing leading slash');
  threw = false;
  try { normalizeBasePath('relative/path'); } catch { threw = true; }
  assert(threw, 'rejects relative path');
}

async function test_base_path_rejects_empty() {
  let threw = false;
  try { normalizeBasePath(''); } catch { threw = true; }
  assert(threw, 'rejects empty string');
  threw = false;
  try { normalizeBasePath(undefined); } catch { threw = true; }
  assert(threw, 'rejects undefined');
}

async function test_base_path_server_uses_helper() {
  setup(); createNs('bpns');
  const relBase = path.join(TMP, 'releases', 'bpns');
  fs.mkdirSync(relBase, { recursive: true });
  fs.writeFileSync(path.join(relBase, 'index.html'), 'bp');
  fs.symlinkSync(relBase, path.join(relBase, '..', 'bpns', 'current'));
  const { server, port } = await startServer({ appBasePath: '/custom/' });
  try {
    let r = await get(port, '/custom/bpns/');
    assert(r.status === 200, 'trailing-slash normalized, serves at /custom/bpns/');
    assert(r.text === 'bp', 'body correct');
    r = await get(port, '/custom/bpns/index.html');
    assert(r.status === 200, 'explicit file works');
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
  ['app_serving_query_string', test_app_serving_query_string],
  ['app_serving_custom_base_path', test_app_serving_custom_base_path],
  ['app_dir_serving', test_app_dir_serving],
  ['app_dir_symlink_atomic', test_app_dir_symlink_atomic],
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
  ['binary_store', test_binary_store],
  ['key_rejects_slash', test_key_rejects_slash],
  ['namespace_validation', test_namespace_validation],
  ['malformed_pct_encoding', test_malformed_pct_encoding],
  ['client_serving', test_client_serving],
  ['missing_static_404', test_missing_static_returns_404],
  ['deploy_rejects_positional', test_deploy_rejects_positional_args],
  ['deploy_requires_all_flags', test_deploy_requires_all_four_flags],
  ['deploy_rejects_non_hex_commit', test_deploy_rejects_non_hex_commit],
  ['deploy_rejects_short_commit', test_deploy_rejects_short_commit],
  ['deploy_subdir_validation', test_deploy_subdir_validation],
  ['build_output_rejects_symlinks', test_build_output_rejects_symlinks],
  ['deploy_success_full', test_deploy_success_full],
  ['deploy_monorepo_subdir', test_deploy_monorepo_subdir],
  ['deploy_failure_cleanup', test_deploy_failure_cleanup],
  ['undeploy_destructive', test_undeploy_destructive],
  ['deploy_auto_create_after_build', test_deploy_auto_create_after_build],
  ['deploy_failure_no_ns_created', test_deploy_failure_no_ns_created],
  ['deploy_invalid_namespace', test_deploy_invalid_namespace],
  ['admin_ns_preserves_quota', test_admin_ns_preserves_quota],
  ['examples_hello_make_build', test_examples_hello_make_build],
  ['undeploy_rejects_positional', test_undeploy_rejects_positional],
  ['rollback_rejects_positional', test_rollback_rejects_positional],
  ['readBody_oversized_413', test_readBody_oversized_413],
  ['release_timestamp_millis', test_release_timestamp_millis],
  ['docker_security_opts', test_docker_security_opts],
  ['base_path_strips_trailing_slashes', test_base_path_strips_trailing_slashes],
  ['base_path_keeps_root', test_base_path_keeps_root],
  ['base_path_rejects_no_leading_slash', test_base_path_rejects_no_leading_slash],
  ['base_path_rejects_empty', test_base_path_rejects_empty],
  ['base_path_server_uses_helper', test_base_path_server_uses_helper],
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
  rmrf(FAKE_BIN);
  process.exit(fail > 0 ? 1 : 0);
}

run();
