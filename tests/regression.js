#!/usr/bin/env node
'use strict';

const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execFileSync } = require('child_process');
const { createServer } = require('../src/server.js');

const TMP = '/tmp/skrynia-regression';
const FAKE_BIN = path.join(os.homedir(), '.skrynia-regression-bin');
const TOKEN = 'regression-token';

function rmrf(p) { try { fs.rmSync(p, { recursive: true, force: true }); } catch {} }
function assert(cond, msg) { if (!cond) throw new Error('ASSERT: ' + msg); }

function setup() {
  rmrf(TMP);
  rmrf(FAKE_BIN);
  for (const name of ['releases', 'storage', 'state', 'builds']) fs.mkdirSync(path.join(TMP, name), { recursive: true });
  fs.mkdirSync(FAKE_BIN, { recursive: true });
}

function createNs(ns, quotaBytes, maxObjects) {
  fs.mkdirSync(path.join(TMP, 'storage', ns), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'state', ns), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', ns, 'quota.json'), JSON.stringify({
    bytes: 0,
    count: 0,
    quotaBytes: quotaBytes || 10485760,
    maxObjects: maxObjects || 10000,
  }));
}

function startServer(extra) {
  const server = createServer(Object.assign({ dataDir: TMP, token: TOKEN }, extra || {}));
  return new Promise(resolve => server.listen(0, '127.0.0.1', () => resolve({ server, port: server.address().port })));
}
function stopServer(server) { return new Promise(resolve => server.close(resolve)); }

function request(port, method, urlPath, data, headers) {
  return new Promise((resolve, reject) => {
    const opts = { hostname:'127.0.0.1', port, path:urlPath, method, headers:Object.assign({}, headers || {}), timeout:15000 };
    if (data != null) opts.headers['Content-Length'] = Buffer.byteLength(data);
    const req = http.request(opts, res => {
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => {
        const body = Buffer.concat(chunks);
        resolve({ status:res.statusCode, body, text:body.toString(), headers:res.headers });
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('timeout')); });
    if (data != null) req.write(data);
    req.end();
  });
}
function get(port, p) { return request(port, 'GET', p); }

function managementUrl(endpoint, params) {
  const q = new URLSearchParams(params || {});
  q.set('token', TOKEN);
  return '/_skrynia/' + endpoint + '?' + q.toString();
}

function makeRepo(dir) {
  fs.mkdirSync(dir, { recursive:true });
  fs.writeFileSync(path.join(dir, 'index.html'), '<h1>app</h1>');
  fs.writeFileSync(path.join(dir, 'Makefile'), 'build:\n\tmkdir -p build && cp index.html build/index.html\n');
  execFileSync('git', ['init', dir], { stdio:'pipe' });
  execFileSync('git', ['-C', dir, 'config', 'user.email', 'test@test.com']);
  execFileSync('git', ['-C', dir, 'config', 'user.name', 'Test']);
  execFileSync('git', ['-C', dir, 'add', '.']);
  execFileSync('git', ['-C', dir, 'commit', '-m', 'init'], { stdio:'pipe' });
  return execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD']).toString().trim();
}

function installFakeDocker(lines) {
  const p = path.join(FAKE_BIN, 'docker');
  fs.writeFileSync(p, ['#!/bin/sh', 'set -eu'].concat(lines).join('\n'));
  fs.chmodSync(p, 0o755);
}

async function withFakeDocker(lines, fn) {
  installFakeDocker(lines);
  const old = process.env.PATH;
  process.env.PATH = FAKE_BIN + ':' + (old || '/usr/bin:/bin');
  try { return await fn(); } finally { process.env.PATH = old; }
}

async function test_concurrent_create() {
  setup(); createNs('c');
  const { server, port } = await startServer();
  try {
    const rs = await Promise.all(Array.from({length:10}, (_, i) => request(port, 'POST', '/_skrynia/store/c/same', 'v' + i, {'X-Skrynia-Mode':'public-write'})));
    assert(rs.filter(r => r.status === 201).length === 1, 'one concurrent create wins');
    assert(rs.filter(r => r.status === 409).length === 9, 'other concurrent creates conflict');
  } finally { await stopServer(server); }
}

async function test_binary_roundtrip() {
  setup(); createNs('bin');
  const { server, port } = await startServer();
  try {
    const data = Buffer.from(Array.from({length:256}, (_, i) => i));
    let r = await request(port, 'POST', '/_skrynia/store/bin/raw', data, {'Content-Type':'application/octet-stream','X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'binary create');
    r = await get(port, '/_skrynia/store/bin/raw');
    assert(r.status === 200 && r.body.equals(data), 'binary bytes round-trip');
  } finally { await stopServer(server); }
}

async function test_malformed_percent_encoding() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    const r = await get(port, '/_skrynia/store/ns/%zz');
    assert(r.status === 400, 'bad percent encoding rejected');
  } finally { await stopServer(server); }
}

async function test_oversized_body_413() {
  setup(); createNs('big');
  const { server, port } = await startServer();
  try {
    const r = await request(port, 'POST', '/_skrynia/store/big/x', Buffer.alloc(10485761), {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 413, 'oversized request rejected');
    assert(JSON.parse(r.text).error === 'request_too_large', 'oversized error code');
  } finally { await stopServer(server); }
}

async function test_static_symlink_escape() {
  setup();
  const release = path.join(TMP, 'releases', 'web', 'r1');
  fs.mkdirSync(release, { recursive:true });
  fs.writeFileSync(path.join(release, 'index.html'), 'safe');
  fs.symlinkSync('/etc', path.join(release, 'escape'));
  fs.symlinkSync(release, path.join(TMP, 'releases', 'web', 'current'));
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/apps/web/');
    assert(r.status === 200 && r.text === 'safe', 'normal static file works');
    r = await get(port, '/apps/web/escape/passwd');
    assert(r.status === 403, 'symlink escape blocked');
  } finally { await stopServer(server); }
}

async function test_custom_base_path() {
  setup();
  const release = path.join(TMP, 'releases', 'web', 'r1');
  fs.mkdirSync(release, { recursive:true });
  fs.writeFileSync(path.join(release, 'index.html'), 'custom');
  fs.symlinkSync(release, path.join(TMP, 'releases', 'web', 'current'));
  const { server, port } = await startServer({ appBasePath:'/custom/' });
  try {
    let r = await get(port, '/custom/web/');
    assert(r.status === 200 && r.text === 'custom', 'custom base serves');
    r = await get(port, '/apps/web/');
    assert(r.status === 404, 'default base no longer serves');
  } finally { await stopServer(server); }
}

async function test_deploy_failure_cleanup_and_no_namespace() {
  setup();
  const repo = path.join(TMP, 'repo');
  const commit = makeRepo(repo);
  const staleDir = path.join(TMP, 'releases', 'fail', '.staging-stale');
  fs.mkdirSync(staleDir, { recursive:true });
  fs.writeFileSync(path.join(staleDir, 'sentinel'), 'keep');
  await withFakeDocker(['exit 17'], async () => {
    const { server, port } = await startServer();
    try {
      const r = await get(port, managementUrl('deploy', { repo:'file://' + repo, commit, subdir:'.', namespace:'fail' }));
      assert(r.status === 500, 'failed build returns 500');
      assert(JSON.parse(r.text).error === 'build_failed', 'failed build error code');
      assert(!fs.existsSync(path.join(TMP, 'state', 'fail', 'quota.json')), 'namespace not created on failed deploy');
      assert(fs.readdirSync(path.join(TMP, 'builds')).filter(x => x.startsWith('build-')).length === 0, 'build workspace cleaned');
      assert(fs.existsSync(path.join(staleDir, 'sentinel')), 'failed deploy cleanup leaves other staging directories alone');
      const staging = fs.readdirSync(path.join(TMP, 'releases', 'fail')).filter(x => x.startsWith('.staging-'));
      assert(staging.length === 1 && staging[0] === '.staging-stale', 'failed deploy removes only its own staging directory');
    } finally { await stopServer(server); }
  });
}

async function test_stale_release_staging_is_not_reused() {
  setup();
  const repo = path.join(TMP, 'repo');
  const commit = makeRepo(repo);
  const releaseBase = path.join(TMP, 'releases', 'fresh');
  const staleDir = path.join(releaseBase, '.staging-stale');
  fs.mkdirSync(staleDir, { recursive:true });
  fs.writeFileSync(path.join(staleDir, 'stale.txt'), 'leftover');

  await withFakeDocker([
    'repo=""',
    'while [ "$#" -gt 0 ]; do if [ "$1" = "-v" ]; then repo="${2%%:*}"; shift 2; else shift; fi; done',
    'mkdir -p "$repo/build"',
    'cp "$repo/index.html" "$repo/build/index.html"',
  ], async () => {
    const { server, port } = await startServer();
    try {
      let r = await get(port, managementUrl('deploy', { repo:'file://' + repo, commit, subdir:'.', namespace:'fresh' }));
      assert(r.status === 200, 'deploy succeeds with stale staging present: ' + r.text);
      const deployed = JSON.parse(r.text);
      const releaseDir = path.join(releaseBase, deployed.release);
      assert(fs.existsSync(path.join(releaseDir, 'index.html')), 'new release contains build output');
      assert(!fs.existsSync(path.join(releaseDir, 'stale.txt')), 'stale staging content is absent from release');

      r = await get(port, managementUrl('releases', { namespace:'fresh' }));
      assert(r.status === 200, 'release listing succeeds');
      const releases = JSON.parse(r.text).releases;
      assert(releases.length === 1 && releases[0] === deployed.release, 'staging directories are excluded from release listing');
    } finally { await stopServer(server); }
  });
}

async function test_invalid_build_output_rejected() {
  setup();
  const repo = path.join(TMP, 'repo');
  const commit = makeRepo(repo);
  await withFakeDocker([
    'repo=""',
    'while [ "$#" -gt 0 ]; do if [ "$1" = "-v" ]; then repo="${2%%:*}"; shift 2; else shift; fi; done',
    'mkdir -p "$repo/build"',
    'ln -s /etc "$repo/build/bad"',
  ], async () => {
    const { server, port } = await startServer();
    try {
      const r = await get(port, managementUrl('deploy', { repo:'file://' + repo, commit, subdir:'.', namespace:'badbuild' }));
      assert(r.status === 500, 'invalid output returns 500');
      assert(JSON.parse(r.text).error === 'invalid_build_output', 'invalid output error code');
      assert(!fs.existsSync(path.join(TMP, 'state', 'badbuild', 'quota.json')), 'namespace not created for invalid output');
    } finally { await stopServer(server); }
  });
}

async function test_capability_verifier_not_in_meta() {
  setup(); createNs('cap');
  const { server, port } = await startServer();
  try {
    const r = await request(port, 'POST', '/_skrynia/store/cap/k', 'v', {'X-Skrynia-Mode':'capability-write'});
    assert(r.status === 201, 'capability create');
    const meta = JSON.parse(fs.readFileSync(path.join(TMP, 'storage', 'cap', 'k.meta')));
    assert(!Object.prototype.hasOwnProperty.call(meta, 'capVerifier'), 'verifier absent from metadata');
    assert(fs.existsSync(path.join(TMP, 'storage', 'cap', 'k.cap')), 'verifier sidecar exists');
  } finally { await stopServer(server); }
}

const tests = [
  ['concurrent_create', test_concurrent_create],
  ['binary_roundtrip', test_binary_roundtrip],
  ['malformed_percent_encoding', test_malformed_percent_encoding],
  ['oversized_body_413', test_oversized_body_413],
  ['static_symlink_escape', test_static_symlink_escape],
  ['custom_base_path', test_custom_base_path],
  ['deploy_failure_cleanup_and_no_namespace', test_deploy_failure_cleanup_and_no_namespace],
  ['stale_release_staging_is_not_reused', test_stale_release_staging_is_not_reused],
  ['invalid_build_output_rejected', test_invalid_build_output_rejected],
  ['capability_verifier_not_in_meta', test_capability_verifier_not_in_meta],
];

(async () => {
  let pass = 0;
  let fail = 0;
  for (const [name, fn] of tests) {
    process.stdout.write('  regression ' + name + ' ... ');
    try { await fn(); console.log('ok'); pass++; }
    catch (e) { console.log('FAIL'); console.error(e && e.stack ? e.stack : e); fail++; }
  }
  console.log('\nRegression results: ' + pass + ' passed, ' + fail + ' failed, ' + (pass + fail) + ' total');
  process.exit(fail ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
