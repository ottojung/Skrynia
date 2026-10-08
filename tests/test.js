#!/usr/bin/env node
'use strict';

const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execFileSync } = require('child_process');
const { createServer } = require('../src/server.js');
const { normalizeBasePath } = require('../src/base-path.js');
const { createShared } = require('../src/shared.js');

const SRC = path.join(__dirname, '..');
const TMP = '/tmp/skrynia-test';
const FAKE_BIN = path.join(os.homedir(), '.skrynia-test-bin');
const TOKEN = 'test-management-token';
const REPO_MAP = path.join(TMP, 'repo.map');

function rmrf(d) { try { fs.rmSync(d, { recursive: true, force: true }); } catch {} }
function assert(cond, msg) { if (!cond) throw new Error('ASSERT: ' + msg); }

function setup() {
  rmrf(TMP);
  rmrf(FAKE_BIN);
  for (const name of ['releases', 'store', 'store-meta', 'store-tmp', 'storage', 'state', 'builds']) fs.mkdirSync(path.join(TMP, name), { recursive: true });
  fs.mkdirSync(FAKE_BIN, { recursive: true });
  installFakeGit();
}

function installFakeGit() {
  const realGit = execFileSync('which', ['git'], { encoding: 'utf8' }).trim();
  const script = path.join(FAKE_BIN, 'git');
  fs.writeFileSync(script, [
    '#!/bin/sh',
    'set -eu',
    'REAL_GIT=' + JSON.stringify(realGit),
    'REPO_MAP=' + JSON.stringify(REPO_MAP),
    'case "$1" in',
    '  clone)',
    '    shift; while [ "$1" = "--quiet" ] || [ "$1" = "-q" ]; do shift; done',
    '    repo=""; destdir=""',
    '    for arg in "$@"; do',
    '      if [ -z "$repo" ]; then repo="$arg"',
    '      elif [ -z "$destdir" ]; then destdir="$arg"; fi',
    '    done',
    '    case "$repo" in',
    '      /*) exec "$REAL_GIT" clone --quiet "$repo" "$destdir" ;;',
    '      *)',
    '        repobasename="${repo##*:}"',
    '        if [ -f "$REPO_MAP" ]; then',
    '          localpath=$(grep -F "$repobasename" "$REPO_MAP" 2>/dev/null | head -1 | cut -d: -f2-)',
    '        fi',
    '        if [ -n "${localpath:-}" ] && [ -d "$localpath" ]; then',
    '          exec "$REAL_GIT" clone --quiet "$localpath" "$destdir"',
    '        else',
    '          exec "$REAL_GIT" clone --quiet "$repo" "$destdir"',
    '        fi',
    '        ;;',
    '    esac',
    '    ;;',
    '  checkout)',
    '    shift; while [ "$1" = "--quiet" ] || [ "$1" = "-q" ]; do shift; done',
    '    exec "$REAL_GIT" "$@"',
    '    ;;',
    '  *)',
    '    exec "$REAL_GIT" "$@"',
    '    ;;',
    'esac',
  ].join('\n'));
  fs.chmodSync(script, 0o755);
}

function registerRepo(sshString, localPath) {
  const name = sshString.split(':')[1];
  let content = '';
  try { content = fs.readFileSync(REPO_MAP, 'utf8'); } catch {}
  const lines = content.split('\n').filter(l => l && !l.startsWith(name + ':'));
  lines.push(name + ':' + localPath);
  fs.writeFileSync(REPO_MAP, lines.join('\n') + '\n');
}

function createNs(ns, quotaBytes, maxObjects) {
  fs.mkdirSync(path.join(TMP, 'store', ns), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'store-meta', ns), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'state', ns), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', ns, 'quota.json'), JSON.stringify({
    quotaBytes: quotaBytes || 10485760,
    maxObjects: maxObjects || 10000,
  }));
}

function startServer(extraOpts) {
  const server = createServer(Object.assign({ dataDir: TMP, token: TOKEN, skryniaUrl: 'https://example.test/platform' }, extraOpts || {}));
  return new Promise(resolve => server.listen(0, '127.0.0.1', () => resolve({ server, port: server.address().port })));
}

function stopServer(server) { return new Promise(resolve => server.close(resolve)); }

function request(port, method, urlPath, data, headers) {
  return new Promise((resolve, reject) => {
    const opts = { hostname: '127.0.0.1', port, path: urlPath, method, headers: Object.assign({}, headers || {}), timeout: 10000 };
    if (data != null) opts.headers['Content-Length'] = Buffer.byteLength(data);
    const req = http.request(opts, res => {
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => {
        const body = Buffer.concat(chunks);
        resolve({ status: res.statusCode, body, text: body.toString(), headers: res.headers });
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('timeout')); });
    if (data != null) req.write(data);
    req.end();
  });
}

function get(port, urlPath) { return request(port, 'GET', urlPath); }

function managementUrl(endpoint, params, token) {
  const q = new URLSearchParams();
  for (const [key, value] of Object.entries(params || {})) {
    if (value != null) q.set(key, String(value));
  }
  q.set('token', token === undefined ? TOKEN : token);
  return '/platform/' + endpoint + '?' + q.toString();
}

function managementGet(port, endpoint, params, token) {
  return get(port, managementUrl(endpoint, params, token));
}

function jsonBody(response) { return JSON.parse(response.text); }

function makeFakeDocker(logPath) {
  const script = path.join(FAKE_BIN, 'docker');
  fs.writeFileSync(script, [
    '#!/bin/sh',
    'set -eu',
    logPath ? 'printf "%s\\n" "$*" >> ' + JSON.stringify(logPath) : ':',
    'if [ -n "${SKRYNIA_TEST_DOCKER_SLEEP:-}" ]; then sleep "$SKRYNIA_TEST_DOCKER_SLEEP"; fi',
    'repo=""',
    'work="/repo"',
    'while [ "$#" -gt 0 ]; do',
    '  case "$1" in',
    '    -v)',
    '      val="$2"',
    '      host="${val%%:*}"',
    '      cont="${val#*:}"',
    '      if [ "$cont" = "/repo" ]; then repo="$host"; fi',
    '      shift 2',
    '      ;;',
    '    -w)',
    '      work="$2"',
    '      shift 2',
    '      ;;',
    '    *) shift ;;',
    '  esac',
    'done',
    '[ -n "$repo" ]',
    'case "$work" in',
    '  /repo) app="$repo" ;;',
    '  /repo/) app="$repo" ;;',
    '  /repo/*) app="$repo/${work#/repo/}" ;;',
    '  *) exit 42 ;;',
    'esac',
    'mkdir -p "$app/build"',
    'cp "$app/index.html" "$app/build/index.html"',
  ].join('\n'));
  fs.chmodSync(script, 0o755);
}

function withFakeDocker(fn, logPath) {
  makeFakeDocker(logPath);
  const oldPath = process.env.PATH;
  process.env.PATH = FAKE_BIN + ':' + (oldPath || '/usr/bin:/bin');
  return Promise.resolve().then(fn).finally(() => { process.env.PATH = oldPath; });
}

function makeHangingGit(marker) {
  const script = path.join(FAKE_BIN, 'git');
  fs.writeFileSync(script, [
    '#!/bin/sh',
    'printf "clone still blocked\\n" >&2',
    'touch ' + JSON.stringify(marker),
    'sleep 10',
  ].join('\n'));
  fs.chmodSync(script, 0o755);
}

function makeHangingBuilderDocker(started, removed, logPath) {
  const script = path.join(FAKE_BIN, 'docker');
  fs.writeFileSync(script, [
    '#!/bin/sh',
    'set -eu',
    'printf "%s\\n" "$*" >> ' + JSON.stringify(logPath),
    'if [ "${1:-}" = "rm" ]; then',
    '  touch ' + JSON.stringify(removed),
    '  printf "removed %s\\n" "${3:-}"',
    '  exit 0',
    'fi',
    'container=""',
    'while [ "$#" -gt 0 ]; do',
    '  case "$1" in',
    '    --name) container="$2"; shift 2 ;;',
    '    *) shift ;;',
    '  esac',
    'done',
    'touch ' + JSON.stringify(started),
    'printf "build still blocked in %s\\n" "$container" >&2',
    'sleep 10',
  ].join('\n'));
  fs.chmodSync(script, 0o755);
}

function makeHangingCleanupDocker(started, cleanupStarted) {
  const script = path.join(FAKE_BIN, 'docker');
  fs.writeFileSync(script, [
    '#!/bin/sh',
    'set -eu',
    'if [ "${1:-}" = "rm" ]; then',
    '  touch ' + JSON.stringify(cleanupStarted),
    '  printf "cleanup secret %s\\n" "$SKRYNIA_DEPLOY_SECRET" >&2',
    '  sleep 10',
    'fi',
    'touch ' + JSON.stringify(started),
    'printf "builder cleanup is bounded\\n" >&2',
    'sleep 10',
  ].join('\n'));
  fs.chmodSync(script, 0o755);
}

function makeFailedCleanupDocker(started) {
  const script = path.join(FAKE_BIN, 'docker');
  fs.writeFileSync(script, [
    '#!/bin/sh',
    'set -eu',
    'if [ "${1:-}" = "rm" ]; then',
    '  printf "name removal secret %s failed\\n" "$SKRYNIA_DEPLOY_SECRET" >&2',
    '  exit 1',
    'fi',
    'touch ' + JSON.stringify(started),
    'printf "builder before failed cleanup\\n" >&2',
    'sleep 10',
  ].join('\n'));
  fs.chmodSync(script, 0o755);
}

function makeRepo(dir, files) {
  fs.mkdirSync(dir, { recursive: true });
  execFileSync('git', ['init', dir], { stdio: 'pipe' });
  execFileSync('git', ['-C', dir, 'config', 'user.email', 'test@test.com']);
  execFileSync('git', ['-C', dir, 'config', 'user.name', 'Test']);
  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, content);
  }
  execFileSync('git', ['-C', dir, 'add', '.']);
  execFileSync('git', ['-C', dir, 'commit', '-m', 'initial'], { stdio: 'pipe' });
  return execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD']).toString().trim();
}

function commitRepo(dir, file, content, message) {
  fs.writeFileSync(path.join(dir, file), content);
  execFileSync('git', ['-C', dir, 'add', file]);
  execFileSync('git', ['-C', dir, 'commit', '-m', message], { stdio: 'pipe' });
  return execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD']).toString().trim();
}

async function test_health() {
  setup();
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/platform/health');
    assert(r.status === 200, 'health status');
    assert(jsonBody(r).ok === true, 'health body');
    r = await get(port, '/health');
    assert(r.status === 404, 'nested root rejects out-of-base health path');
  } finally { await stopServer(server); }
}


async function test_version() {
  setup();
  const buildInfo = {
    version: '1.2.3-4-gabcdef0',
    commit: '0123456789abcdef0123456789abcdef01234567',
  };
  const { server, port } = await startServer({ buildInfo });
  try {
    let r = await get(port, '/platform/version');
    assert(r.status === 200, 'version status');
    assert(jsonBody(r).version === buildInfo.version, 'version value');
    assert(jsonBody(r).commit === buildInfo.commit, 'commit value');

    r = await request(port, 'POST', '/platform/version');
    assert(r.status === 405, 'version is GET only');

    r = await get(port, '/version');
    assert(r.status === 404, 'nested root rejects out-of-base version path');
  } finally { await stopServer(server); }
}

async function test_root_url() {
  setup();
  const { server, port } = await startServer({ skryniaUrl: 'https://example.test/' });
  try {
    let r = await get(port, '/health');
    assert(r.status === 200 && jsonBody(r).ok === true, 'root URL health');
    r = await get(port, '/platform/health');
    assert(r.status === 404, 'root URL does not invent nested prefix');
  } finally { await stopServer(server); }
}

async function test_management_requires_configured_token() {
  setup();
  const { server, port } = await startServer({ token: '' });
  try {
    const r = await get(port, '/platform/ns/list?token=anything');
    assert(r.status === 503, 'disabled management status');
    assert(jsonBody(r).error === 'management_api_disabled', 'disabled management error');
  } finally { await stopServer(server); }
}

async function test_management_rejects_missing_or_bad_token() {
  setup();
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/platform/ns/list');
    assert(r.status === 401, 'missing token rejected');
    r = await managementGet(port, 'ns/list', {}, 'wrong');
    assert(r.status === 401, 'bad token rejected');
  } finally { await stopServer(server); }
}

async function test_management_get_only() {
  setup();
  const { server, port } = await startServer();
  try {
    const r = await request(port, 'POST', managementUrl('ns/list', {}));
    assert(r.status === 405, 'management POST rejected');
  } finally { await stopServer(server); }
}

async function test_namespace_http_api() {
  setup();
  const { server, port } = await startServer();
  try {
    let r = await managementGet(port, 'ns/create', { namespace: 'web', quota: 1234 });
    assert(r.status === 200, 'ns create');
    assert(jsonBody(r).created === true, 'created flag');
    assert(jsonBody(r).quota.quotaBytes === 1234, 'quota applied');

    r = await managementGet(port, 'ns/create', { namespace: 'web', quota: 9999 });
    assert(r.status === 200, 'repeat create');
    assert(jsonBody(r).created === false, 'repeat preserves namespace');
    assert(jsonBody(r).quota.quotaBytes === 1234, 'repeat preserves quota');

    r = await managementGet(port, 'ns/inspect', { namespace: 'web' });
    assert(r.status === 200, 'ns inspect');
    assert(jsonBody(r).namespace === 'web', 'inspect namespace');

    r = await managementGet(port, 'ns/list', {});
    assert(r.status === 200, 'ns list');
    assert(jsonBody(r).namespaces.length === 1, 'one namespace listed');

    r = await managementGet(port, 'ns/remove', { namespace: 'web' });
    assert(r.status === 200, 'ns remove');
    r = await managementGet(port, 'ns/inspect', { namespace: 'web' });
    assert(r.status === 404, 'removed namespace missing');
  } finally { await stopServer(server); }
}

async function test_store_requires_namespace() {
  setup();
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/platform/store/missing/x', 'x', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 409, 'missing namespace blocks create');
    await managementGet(port, 'ns/create', { namespace: 'created' });
    r = await request(port, 'POST', '/platform/store/created/x', 'x', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'HTTP-created namespace accepts store write');
  } finally { await stopServer(server); }
}

async function test_store_absence_does_not_disclose_namespace_existence() {
  setup(); createNs('present');
  const { server, port } = await startServer();
  try {
    const methods = [
      ['GET', null],
      ['PUT', 'body'],
      ['DELETE', null],
    ];
    for (const [method, data] of methods) {
      const headers = method === 'PUT' ? { 'X-Skrynia-Capability': '00'.repeat(32) } : {};
      const present = await request(port, method, '/platform/store/present/absentkey', data, headers);
      const absent = await request(port, method, '/platform/store/absent/absentkey', data, headers);
      assert(present.status === 404, method + ' on an existing namespace with an absent key is 404, got ' + present.status);
      assert(absent.status === present.status, method + ' status must not reveal namespace existence');
      assert(absent.text === present.text, method + ' body must not reveal namespace existence: ' + JSON.stringify(absent.text) + ' vs ' + JSON.stringify(present.text));
      assert(absent.headers['content-type'] === present.headers['content-type'], method + ' content type must not reveal namespace existence');
      assert(JSON.parse(absent.text).error === 'not_found', method + ' on an absent namespace answers not_found');
    }
  } finally { await stopServer(server); }
}

// A store object key is a bearer credential, so the URL that carries it must
// not survive anywhere the operator reads by eye or a script retains. The
// store path writes nothing to the process streams, and no store response
// reflects the namespace, the key or the capability back to the caller.
async function test_store_path_neither_logs_nor_reflects_the_secret_url() {
  setup();
  const ns = 'secret-ns';
  const key = 'secret-key-4d2e9b';
  const absentNs = 'no-such-ns';
  const absentKey = 'no-such-key';
const { server, port } = await startServer();
  try {
    await managementGet(port, 'ns/create', { namespace: ns });
    const created = JSON.parse((await request(port, 'POST', `/platform/store/${ns}/${key}`, 'v1', { 'X-Skrynia-Mode': 'capability-write' })).text);
    assert(created.capability, 'capability-write create returns a capability');
    const capability = created.capability;

    // One request per outcome the store path can produce, every one of them
    // addressed by the capability-bearing URL and by an absent namespace.
    const probes = [
      ['POST', `/platform/store/${ns}/${key}-fresh`, 'x', {}],
      ['POST', `/platform/store/${ns}/${key}`, 'x', {}],
      ['POST', `/platform/store/${absentNs}/${key}`, 'x', {}],
      ['POST', `/platform/store/${ns}/badmode`, 'x', {'X-Skrynia-Mode':'immutable'}],
      ['POST', `/platform/store/${ns}/unknownmode`, 'x', {'X-Skrynia-Mode':'nonsense'}],
      ['GET', `/platform/store/${ns}/${key}`, null, {}],
      ['GET', `/platform/store/${ns}/${absentKey}`, null, {}],
      ['GET', `/platform/store/${absentNs}/${key}`, null, {}],
      ['GET', `/platform/store/${ns}/%zz`, null, {}],
      ['GET', `/platform/store/${ns}/` + 'k'.repeat(300), null, {}],
      ['GET', `/platform/store/${ns}/bad%2Fslash`, null, {}],
      ['PUT', `/platform/store/${ns}/${key}`, 'x', {}],
      ['PUT', `/platform/store/${ns}/${key}`, 'x', {'X-Skrynia-Capability':'0'.repeat(64)}],
      ['PUT', `/platform/store/${ns}/${key}`, 'x', {'X-Skrynia-Capability':capability, 'If-Match':'"not-the-etag"'}],
      ['PUT', `/platform/store/${ns}/${key}`, 'x', {'X-Skrynia-Capability':capability}],
      ['PUT', `/platform/store/${ns}/${absentKey}`, 'x', {'X-Skrynia-Capability':capability}],
      ['PUT', `/platform/store/${absentNs}/${key}`, 'x', {'X-Skrynia-Capability':capability}],
      ['DELETE', `/platform/store/${ns}/${key}`, null, {}],
      ['DELETE', `/platform/store/${ns}/${key}`, null, {'X-Skrynia-Capability':'0'.repeat(64)}],
      ['DELETE', `/platform/store/${ns}/${key}`, null, {'X-Skrynia-Capability':capability}],
      ['DELETE', `/platform/store/${ns}/${key}`, null, {'X-Skrynia-Capability':capability}],
      ['PATCH', `/platform/store/${ns}/${key}`, 'x', {}],
    ];

    const statuses = new Set();
    let captured = '';
    const collect = chunk => { captured += chunk; return true; };
    const realOut = process.stdout.write;
    const realErr = process.stderr.write;
    process.stdout.write = collect;
    process.stderr.write = collect;
    let responses;
    try {
      responses = [];
      for (const [method, urlPath, data, headers] of probes) {
        responses.push([method + ' ' + urlPath.replace(new RegExp('^/platform/store/'), ''), await request(port, method, urlPath, data, headers)]);
      }
    } finally {
      process.stdout.write = realOut;
      process.stderr.write = realErr;
    }

    assert(captured === '', 'the store path writes nothing to stdout or stderr: ' + JSON.stringify(captured.slice(0, 400)));
    assert(probes.length >= 20, 'the probe set covers the store outcomes, not a token sample');

    for (const [label, r] of responses) {
      const haystacks = [[label + ' body', r.text]];
      for (const [name, value] of Object.entries(r.headers)) haystacks.push([label + ' header ' + name, String(value)]);
      for (const [what, text] of haystacks) {
        // The one legitimate exception: the create of a capability-write
        // object answers with the capability it just issued, once.
        if (text.includes(capability) && r.status === 201 && text.includes('"capability"')) continue;
        assert(!text.includes(ns), 'store response must not echo the namespace: ' + what);
        assert(!text.includes(key) && !text.includes(absentKey), 'store response must not echo the object key: ' + what);
        assert(!text.includes(capability), 'store response must not echo the capability: ' + what);
      }
      statuses.add(r.status);
    }
    assert(statuses.has(403) && statuses.has(404) && statuses.has(409) && statuses.has(412) && statuses.has(405),
      'the probes really reached the refusal outcomes, got ' + [...statuses].sort().join(','));
  } finally { await stopServer(server); }
}

// A filesystem failure on the store path must still not retain the bearer URL.
// The cleanup catch used to log e.message, and filesystem errors embed the
// failing path, which is the namespace and the object key.
async function test_store_error_logging_is_path_free() {
  setup();
  const ns = 'error-ns';
  const key = 'error-key-9f3a';
  const { server, port } = await startServer();
  try {
    await managementGet(port, 'ns/create', { namespace: ns });
    const created = JSON.parse((await request(port, 'POST', `/platform/store/${ns}/${key}`, 'v1', { 'X-Skrynia-Mode': 'capability-write' })).text);
    const metaPath = path.join(TMP, 'store-meta', ns, key + '.json');

    // Simulate the filesystem error the way ENOSPC/EIO presents: a message that
    // embeds the meta path (namespace + key). Only the meta rename fails.
    const realRename = fs.renameSync;
    fs.renameSync = function (from, to) {
      if (to === metaPath) {
        const e = new Error("ENOSPC: no space left on device, rename '" + from + "' -> '" + to + "'");
        e.code = 'ENOSPC';
        throw e;
      }
      return realRename.apply(fs, arguments);
    };
    let captured = '';
    const realErr = process.stderr.write;
    process.stderr.write = chunk => { captured += chunk; return true; };
    let del;
    try {
      del = await request(port, 'DELETE', `/platform/store/${ns}/${key}`, null, { 'X-Skrynia-Capability': created.capability });
    } finally {
      process.stderr.write = realErr;
      fs.renameSync = realRename;
    }

    assert(del.status === 200, 'the delete still commits when metadata cleanup fails, got ' + del.status);
    assert(captured.includes('metadata cleanup'), 'the failure is still surfaced to the operator: ' + JSON.stringify(captured));
    assert(!captured.includes(ns), 'store error output must not contain the namespace: ' + JSON.stringify(captured));
    assert(!captured.includes(key), 'store error output must not contain the object key: ' + JSON.stringify(captured));
  } finally { await stopServer(server); }
}

// The legacy storage migration runs at startup, where a throw is an uncaught
// exception whose message reaches stderr. A store object key is a bearer
// credential, so the migration refusal must not name the namespace or key.
function test_legacy_migration_failure_is_path_free() {
  setup();
  const ns = 'migr-ns';
  const key = 'migr-key-7c21';
  fs.mkdirSync(path.join(TMP, 'storage', ns), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'storage', ns, key + '.meta'), JSON.stringify({ mode: 'immutable' }));
  let caught = null;
  try {
    createServer({ dataDir: TMP, token: TOKEN, skryniaUrl: 'https://example.test/platform' });
  } catch (e) {
    caught = e;
  }
  assert(caught, 'inconsistent legacy state fails the migration');
  assert(!caught.message.includes(ns), 'migration refusal must not name the namespace: ' + caught.message);
  assert(!caught.message.includes(key), 'migration refusal must not name the object key: ' + caught.message);
}

// A filesystem failure during the migration must also reach stderr without
// the failing path, so it is rethrown with the error code only.
function test_legacy_migration_filesystem_failure_is_path_free() {
  setup();
  fs.mkdirSync(path.join(TMP, 'storage', 'migr-ns'), { recursive: true });
  const realReaddir = fs.readdirSync;
  fs.readdirSync = function (p) {
    if (typeof p === 'string' && p.endsWith('storage')) {
      const e = new Error("EIO: input/output error, scandir '" + p + "'");
      e.code = 'EIO';
      throw e;
    }
    return realReaddir.apply(fs, arguments);
  };
  let caught = null;
  try {
    createServer({ dataDir: TMP, token: TOKEN, skryniaUrl: 'https://example.test/platform' });
  } catch (e) {
    caught = e;
  } finally {
    fs.readdirSync = realReaddir;
  }
  assert(caught, 'a filesystem failure during migration fails startup');
  assert(caught.message.includes('EIO'), 'the failure is still surfaced as a code: ' + caught.message);
  assert(!caught.message.includes(TMP), 'migration filesystem failure must not name the path: ' + caught.message);
}

// A filesystem failure on the store read path must answer a stable 500 and
// keep the namespace and key out of stderr. The object path is a directory,
// so the read fails the way EIO or EACCES would: a message that embeds the
// failing path.
async function test_store_read_failure_is_path_free_and_answered() {
  setup();
  const ns = 'read-ns';
  const key = 'read-key-5b1f';
  const { server, port } = await startServer();
  try {
    await managementGet(port, 'ns/create', { namespace: ns });
    await request(port, 'POST', `/platform/store/${ns}/${key}`, 'v1', { 'X-Skrynia-Mode': 'public-write' });
    fs.rmSync(path.join(TMP, 'store', ns, key));
    fs.mkdirSync(path.join(TMP, 'store', ns, key));

    let captured = '';
    const realErr = process.stderr.write;
    process.stderr.write = chunk => { captured += chunk; return true; };
    let r;
    try {
      r = await get(port, `/platform/store/${ns}/${key}`);
    } finally {
      process.stderr.write = realErr;
    }
    assert(r.status === 500, 'a store read filesystem failure answers 500, got ' + r.status);
    assert(captured.includes('store error'), 'the failure is still surfaced to the operator: ' + JSON.stringify(captured));
    assert(!captured.includes(ns), 'store read failure must not name the namespace: ' + JSON.stringify(captured));
    assert(!captured.includes(key), 'store read failure must not name the object key: ' + JSON.stringify(captured));
  } finally { await stopServer(server); }
}

// An unexpected management failure used to log e.stack, whose first line is
// the error message, and a filesystem error message embeds the failing path,
// which for namespace state is the namespace. Those failures now log a code
// only.
async function test_management_filesystem_failure_is_path_free() {
  setup();
  const ns = 'mgmt-ns';
  const { server, port } = await startServer();
  try {
    await managementGet(port, 'ns/create', { namespace: ns });
    const configPath = path.join(TMP, 'state', ns, 'config.json');
    fs.writeFileSync(configPath, '{}');
    const realRead = fs.readFileSync;
    fs.readFileSync = function (p) {
      if (typeof p === 'string' && p === configPath) {
        const e = new Error("EACCES: permission denied, open '" + p + "'");
        e.code = 'EACCES';
        throw e;
      }
      return realRead.apply(fs, arguments);
    };
    let captured = '';
    const realErr = process.stderr.write;
    process.stderr.write = chunk => { captured += chunk; return true; };
    let r;
    try {
      r = await managementGet(port, 'ns/inspect', { namespace: ns });
    } finally {
      process.stderr.write = realErr;
      fs.readFileSync = realRead;
    }
    assert(r.status === 500, 'a management filesystem failure answers 500, got ' + r.status);
    assert(captured.includes('management error'), 'the failure is still surfaced to the operator: ' + JSON.stringify(captured));
    assert(!captured.includes(ns), 'management failure must not name the namespace: ' + JSON.stringify(captured));
  } finally { await stopServer(server); }
}

// The push subscription handlers rethrew unexpected errors, so a filesystem
// failure reading a subscription record crashed the process with the record
// path, which contains the namespace, in stderr. Those failures now answer a
// stable 500 and log a code only.
async function test_push_subscription_failure_is_path_free() {
  setup();
  const ns = 'push-ns';
  const { server, port } = await startServer();
  try {
    await managementGet(port, 'ns/create', { namespace: ns });
    const realRead = fs.readFileSync;
    fs.readFileSync = function (p) {
      if (typeof p === 'string' && p.includes('push-subs')) {
        const e = new Error("EACCES: permission denied, open '" + p + "'");
        e.code = 'EACCES';
        throw e;
      }
      return realRead.apply(fs, arguments);
    };
    let captured = '';
    const realErr = process.stderr.write;
    process.stderr.write = chunk => { captured += chunk; return true; };
    let r;
    try {
      r = await request(port, 'PUT', '/platform/push/subscriptions/' + 'a'.repeat(32), '{"endpoint":"https://push.example/other"}', { 'X-Skrynia-Capability': '0'.repeat(64) });
    } finally {
      process.stderr.write = realErr;
      fs.readFileSync = realRead;
    }
    assert(r.status === 500, 'a push subscription filesystem failure answers 500, got ' + r.status);
    assert(captured.includes('push error'), 'the failure is still surfaced to the operator: ' + JSON.stringify(captured));
    assert(!captured.includes(ns), 'push failure must not name the namespace: ' + JSON.stringify(captured));
  } finally { await stopServer(server); }
}

// A plain-http public root would put every bearer store URL on the wire in the
// clear. Skrynia cannot terminate TLS, so it refuses the misconfiguration and
// requires an explicit override.
function test_plain_http_public_root_is_rejected() {
  setup();
  let rejected = false;
  try {
    createServer({ dataDir: TMP, token: TOKEN, skryniaUrl: 'http://example.test/platform' });
  } catch (e) {
    rejected = /plain http/.test(e.message);
  }
  assert(rejected, 'a non-loopback plain-http public root is rejected');

  // Loopback and https roots are unaffected, and the override is explicit.
  const loop = createServer({ dataDir: TMP, token: TOKEN, skryniaUrl: 'http://127.0.0.1:17380/platform' });
  loop.close();
  const https = createServer({ dataDir: TMP, token: TOKEN, skryniaUrl: 'https://example.test/platform' });
  https.close();
  const forced = createServer({ dataDir: TMP, token: TOKEN, skryniaUrl: 'http://example.test/platform', allowInsecureHttp: true });
  forced.close();
}

async function test_secret_url_responses_forbid_referrer_and_retention() {
  setup();
  const { server, port } = await startServer();
  try {
    await managementGet(port, 'ns/create', { namespace: 'refns' });
    const created = JSON.parse((await request(port, 'POST', '/platform/store/refns/secret-key-77aa21', 'v1', { 'X-Skrynia-Mode': 'capability-write' })).text);
    assert(created.capability, 'capability-write create returns a capability');
    const withCapability = { 'X-Skrynia-Capability': created.capability };

    // Each probe carries whether the response is served for a bearer URL (or a
    // management token URL) and must therefore never be retained.
    const probes = [
      ['store read', await get(port, '/platform/store/refns/secret-key-77aa21'), true],
      ['store absence', await get(port, '/platform/store/refns/absent-key'), true],
      ['store refusal', await request(port, 'PUT', '/platform/store/refns/secret-key-77aa21', 'x', withCapability), true],
      ['store create refusal', await request(port, 'POST', '/platform/store/refns/secret-key-77aa21', 'x'), true],
      // A method refusal is a store response too. 405 is heuristically
      // cacheable, so it needs no-store like every other store answer.
      ['store method refusal', await request(port, 'PATCH', '/platform/store/refns/secret-key-77aa21', 'x'), true],
      ['management answer', await managementGet(port, 'ns/inspect', { namespace: 'refns' }), true],
      ['management refusal', await managementGet(port, 'ns/inspect', { namespace: 'refns' }, 'wrong-token'), true],
      ['management method refusal', await request(port, 'POST', managementUrl('ns/list', {})), true],
      // Refusals and plain-text answers are responses a browser can render
      // too, so the guarantee cannot depend on a route remembering to add it.
      ['method refusal', await request(port, 'POST', '/platform/health', 'x'), false],
      ['unknown route', await get(port, '/platform/nothing-here'), false],
      ['client library', await get(port, '/platform/client/skrynia.js'), false],
    ];

    for (const [label, r, retained] of probes) {
      assert(r.headers['referrer-policy'] === 'no-referrer',
        label + ' forbids the referrer, got ' + JSON.stringify(r.headers['referrer-policy']));
      if (retained) {
        assert(r.headers['cache-control'] === 'no-store',
          label + ' is not retained by an intermediary, got ' + JSON.stringify(r.headers['cache-control']));
      }
    }

    // The app document is the response a browser renders and can navigate
    // away from, so it is where a leaked Referer would matter most. Stage a
    // release directly rather than deploying, so this needs no builder.
    const relDir = path.join(TMP, 'releases', 'v1');
    fs.mkdirSync(relDir, { recursive: true });
    fs.writeFileSync(path.join(relDir, 'index.html'), '<h1>app</h1>');
    fs.mkdirSync(path.join(TMP, 'releases', 'refapp'), { recursive: true });
    fs.symlinkSync(relDir, path.join(TMP, 'releases', 'refapp', 'current'));
    const appDoc = await get(port, '/apps/refapp/');
    assert(appDoc.status === 200, 'app document served, got ' + appDoc.status);
    assert(appDoc.headers['referrer-policy'] === 'no-referrer',
      'app document forbids the referrer, got ' + JSON.stringify(appDoc.headers['referrer-policy']));
  } finally { await stopServer(server); }
}

async function test_store_crud() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/platform/store/ns/hello', 'world', {'Content-Type':'text/plain','X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'create');
    r = await get(port, '/platform/store/ns/hello');
    assert(r.status === 200 && r.text === 'world', 'read');
    r = await request(port, 'PUT', '/platform/store/ns/hello', 'updated', {'Content-Type':'text/plain'});
    assert(r.status === 200, 'put');
    r = await get(port, '/platform/store/ns/hello');
    assert(r.text === 'updated', 'updated read');
    r = await request(port, 'DELETE', '/platform/store/ns/hello');
    assert(r.status === 200, 'delete');
    r = await get(port, '/platform/store/ns/hello');
    assert(r.status === 404, 'deleted');
  } finally { await stopServer(server); }
}

async function test_etag_and_conditional_replace() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/platform/store/ns/k', 'one', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'create for etag');
    r = await get(port, '/platform/store/ns/k');
    const firstEtag = r.headers.etag;
    const firstStat = fs.statSync(path.join(TMP, 'store', 'ns', 'k'));
    const nginxEtag = '"' + Math.floor(firstStat.mtimeMs / 1000).toString(16) + '-' + firstStat.size.toString(16) + '"';
    assert(r.status === 200 && firstEtag === nginxEtag, 'read exposes nginx-compatible static-file etag');
    assert(r.headers['content-length'] === String(Buffer.byteLength('one')), 'read exposes exact content length');

    r = await get(port, '/platform/store/ns/k');
    assert(r.headers.etag === firstEtag, 'unchanged reread has same etag');

    r = await request(port, 'PUT', '/platform/store/ns/k', 'two', {'If-Match': firstEtag});
    assert(r.status === 200, 'matching etag replaces');
    r = await get(port, '/platform/store/ns/k');
    const secondEtag = r.headers.etag;
    assert(r.text === 'two' && secondEtag !== firstEtag, 'replacement advances etag');

    r = await request(port, 'PUT', '/platform/store/ns/k', 'one', {'If-Match': secondEtag});
    assert(r.status === 200, 'second matching etag replaces');
    r = await get(port, '/platform/store/ns/k');
    const thirdEtag = r.headers.etag;
    assert(r.text === 'one' && thirdEtag !== secondEtag && thirdEtag !== firstEtag, 'A-B-A still advances etag');

    r = await request(port, 'PUT', '/platform/store/ns/k', 'wrong', {'If-Match': firstEtag});
    assert(r.status === 412 && jsonBody(r).error === 'etag_mismatch', 'stale etag rejected after A-B-A');

    r = await request(port, 'DELETE', '/platform/store/ns/k');
    assert(r.status === 200, 'delete before recreation');
    r = await request(port, 'POST', '/platform/store/ns/k', 'uno', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'recreate');
    r = await get(port, '/platform/store/ns/k');
    const recreatedEtag = r.headers.etag;
    assert(r.text === 'uno' && recreatedEtag !== thirdEtag && recreatedEtag !== firstEtag, 'delete-recreate advances etag');

    r = await request(port, 'PUT', '/platform/store/ns/k', 'stale', {'If-Match': firstEtag});
    assert(r.status === 412, 'pre-delete etag cannot replace recreated object');
  } finally { await stopServer(server); }
}

async function test_conditional_replace_authorization_and_race() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/platform/store/ns/k', 'one', {'X-Skrynia-Mode':'capability-write'});
    const cap = jsonBody(r).capability;
    r = await get(port, '/platform/store/ns/k');
    const etag = r.headers.etag;
    r = await request(port, 'PUT', '/platform/store/ns/k', 'no-cap', {'If-Match': etag});
    assert(r.status === 403, 'conditional put still requires capability');
    const writers = await Promise.all([
      request(port, 'PUT', '/platform/store/ns/k', 'a', {'X-Skrynia-Capability': cap, 'If-Match': etag}),
      request(port, 'PUT', '/platform/store/ns/k', 'b', {'X-Skrynia-Capability': cap, 'If-Match': etag}),
    ]);
    assert(writers.filter(x => x.status === 200).length === 1, 'one competing conditional writer succeeds');
    assert(writers.filter(x => x.status === 412).length === 1, 'one competing conditional writer fails precondition');
  } finally { await stopServer(server); }
}

async function test_capability_write() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/platform/store/ns/k', 'one', {'X-Skrynia-Mode':'capability-write'});
    const cap = jsonBody(r).capability;
    assert(r.status === 201 && cap && cap.length === 64, 'capability returned');
    r = await request(port, 'PUT', '/platform/store/ns/k', 'two');
    assert(r.status === 403, 'capability required');
    r = await request(port, 'PUT', '/platform/store/ns/k', 'two', {'X-Skrynia-Capability':cap});
    assert(r.status === 200, 'capability permits put');
    r = await request(port, 'DELETE', '/platform/store/ns/k', null, {'X-Skrynia-Capability':cap});
    assert(r.status === 200, 'capability permits delete');
  } finally { await stopServer(server); }
}

async function test_immutable_mode_is_not_accepted() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    const r = await request(port, 'POST', '/platform/store/ns/k', 'one', {'X-Skrynia-Mode':'immutable'});
    assert(r.status === 400 && jsonBody(r).error === 'mode_removed', 'immutable creation is rejected explicitly');
    assert(!fs.existsSync(path.join(TMP, 'store', 'ns', 'k')), 'rejected creation writes no object');
    const read = await get(port, '/platform/store/ns/k');
    assert(read.status === 404, 'rejected creation leaves the key absent');
  } finally { await stopServer(server); }
}

async function test_legacy_immutable_object_is_writable_in_place() {
  setup(); createNs('ns');
  fs.writeFileSync(path.join(TMP, 'store', 'ns', 'old'), 'legacy');
  fs.writeFileSync(path.join(TMP, 'store-meta', 'ns', 'old.json'), JSON.stringify({mode:'immutable', version:1}));
  const { server, port } = await startServer();
  try {
    const read = await get(port, '/platform/store/ns/old');
    assert(read.status === 200 && read.text === 'legacy', 'legacy immutable object remains readable');
    const put = await request(port, 'PUT', '/platform/store/ns/old', 'replaced');
    assert(put.status === 200, 'legacy immutable object is writable in place, with no prerequisite');
    assert((await get(port, '/platform/store/ns/old')).text === 'replaced', 'the in-place replace is visible on a subsequent read');
    const again = await request(port, 'PUT', '/platform/store/ns/old', 'replaced-again');
    assert(again.status === 200, 'the stored immutable marker grants no lasting privilege');
    const del = await request(port, 'DELETE', '/platform/store/ns/old');
    assert(del.status === 200, 'legacy immutable object can be reclaimed');
    const after = await get(port, '/platform/store/ns/old');
    assert(after.status === 404, 'reclaimed object is gone');
  } finally { await stopServer(server); }
}

async function test_legacy_immutable_object_is_not_migrated() {
  setup(); createNs('ns');
  fs.writeFileSync(path.join(TMP, 'store', 'ns', 'old'), 'legacy');
  fs.writeFileSync(path.join(TMP, 'store-meta', 'ns', 'old.json'), JSON.stringify({mode:'immutable', version:1}));
  const { server, port } = await startServer();
  try {
    const before = JSON.parse(fs.readFileSync(path.join(TMP, 'store-meta', 'ns', 'old.json'), 'utf8'));
    assert(before.mode === 'immutable', 'the stored mode is left as it was found');
    const put = await request(port, 'PUT', '/platform/store/ns/old', 'replaced');
    assert(put.status === 200, 'the object is writable without any prior migration step');
    const after = JSON.parse(fs.readFileSync(path.join(TMP, 'store-meta', 'ns', 'old.json'), 'utf8'));
    assert(after.mode === 'immutable', 'a replace does not re-key, copy or rewrite the stored mode');
    assert(fs.existsSync(path.join(TMP, 'store', 'ns', 'old')), 'the object keeps its key and is not relocated');
    assert(!fs.existsSync(path.join(TMP, 'store', 'ns', 'old.rekeyed')), 'no re-keyed copy is produced');
  } finally { await stopServer(server); }
}

async function test_capability_write_is_unaffected_by_legacy_mode() {
  setup(); createNs('ns');
  fs.writeFileSync(path.join(TMP, 'store', 'ns', 'old'), 'legacy');
  fs.writeFileSync(path.join(TMP, 'store-meta', 'ns', 'old.json'), JSON.stringify({mode:'immutable', version:1}));
  const { server, port } = await startServer();
  try {
    const created = await request(port, 'POST', '/platform/store/ns/cw', 'one', {'X-Skrynia-Mode':'capability-write'});
    const cap = jsonBody(created).capability;
    assert(created.status === 201 && cap, 'capability-write object created');
    assert((await request(port, 'PUT', '/platform/store/ns/cw', 'two')).status === 403, 'capability-write still refuses an uncapped replace');
    assert((await request(port, 'DELETE', '/platform/store/ns/cw')).status === 403, 'capability-write still refuses an uncapped delete');
    assert((await request(port, 'PUT', '/platform/store/ns/cw', 'two', {'X-Skrynia-Capability':cap})).status === 200, 'the capability still permits replace');
    assert((await request(port, 'DELETE', '/platform/store/ns/cw', null, {'X-Skrynia-Capability':'0'.repeat(64)})).status === 403, 'a wrong capability is still refused');
    assert((await get(port, '/platform/store/ns/cw')).text === 'two', 'capability-write object still holds the replaced bytes');
  } finally { await stopServer(server); }
}

function plantLegacyObject(ns, key, meta) {
  fs.mkdirSync(path.join(TMP, 'storage', ns), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'storage', ns, key + '.dat'), 'legacy-' + key);
  fs.writeFileSync(path.join(TMP, 'storage', ns, key + '.meta'), JSON.stringify(meta));
  fs.writeFileSync(path.join(TMP, 'storage', ns, key + '.ver'), '100');
}

function migratedMeta(ns, key) {
  return JSON.parse(fs.readFileSync(path.join(TMP, 'store-meta', ns, key + '.json'), 'utf8'));
}

async function test_legacy_storage_migration_yields_writable_modes() {
  setup(); createNs('ns');
  plantLegacyObject('ns', 'kept', {mode:'immutable'});
  plantLegacyObject('ns', 'odd', {mode:'write-once-forever'});
  plantLegacyObject('ns', 'bare', {version:7});
  const { server, port } = await startServer();
  try {
    assert(migratedMeta('ns', 'kept').mode === 'immutable', 'a genuine legacy immutable entry migrates as itself, not promoted');
    assert(migratedMeta('ns', 'odd').mode === 'public-write', 'an unrecognised legacy mode migrates to a writable mode, not to immutable');
    assert(migratedMeta('ns', 'bare').mode === 'public-write', 'a legacy entry with no mode migrates to a writable mode, not to immutable');
    for (const key of ['kept', 'odd', 'bare']) {
      assert(fs.readFileSync(path.join(TMP, 'store', 'ns', key), 'utf8') === 'legacy-' + key, key + ' bytes migrated to the public object path');
    }
    for (const key of ['odd', 'bare']) {
      const put = await request(port, 'PUT', '/platform/store/ns/' + key, 'rewritten');
      assert(put.status === 200, key + ' is writable in place by an anonymous caller');
    }
    const keptPut = await request(port, 'PUT', '/platform/store/ns/kept', 'rewritten');
    assert(keptPut.status === 200, 'a genuine legacy immutable entry keeps its stored mode and is still writable in place');
    assert(migratedMeta('ns', 'kept').mode === 'immutable', 'a replace leaves the migrated mode as itself rather than rewriting it');
    for (const key of ['kept', 'odd', 'bare']) {
      const del = await request(port, 'DELETE', '/platform/store/ns/' + key);
      assert(del.status === 200, key + ' is reclaimable, so migration cannot mint an undeletable object');
      const create = await request(port, 'POST', '/platform/store/ns/' + key, 'replacement', {'X-Skrynia-Mode':'public-write'});
      assert(create.status === 201, key + ' key is reusable after reclaim');
      const read = await get(port, '/platform/store/ns/' + key);
      assert(read.text === 'replacement', key + ' holds the replacement bytes');
    }
  } finally { await stopServer(server); }
}

async function test_key_validation() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/platform/store/ns/a%2Fb');
    assert(r.status === 400, 'slash rejected');
    r = await get(port, '/platform/store/ns/..%2Fetc');
    assert(r.status === 400, 'traversal rejected');
    r = await get(port, '/platform/store/INVALID/key');
    assert(r.status === 400, 'namespace rejected');
  } finally { await stopServer(server); }
}

async function test_quota_and_count_limits() {
  setup(); createNs('small', 4, 1);
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/platform/store/small/a', '1234', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'first object fits');
    r = await request(port, 'POST', '/platform/store/small/b', 'x', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 507, 'object count blocks second object');
    r = await request(port, 'PUT', '/platform/store/small/a', '12345');
    assert(r.status === 507, 'quota blocks growth');
  } finally { await stopServer(server); }
}

async function test_incomplete_create_is_reclaimed() {
  setup(); createNs('ns');
  fs.writeFileSync(path.join(TMP, 'store-meta', 'ns', 'k.json'), JSON.stringify({mode:'capability-write', capHash:'orphan', version:123}));
  const { server, port } = await startServer();
  try {
    const r = await request(port, 'POST', '/platform/store/ns/k', 'fresh', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'private residue without public file does not block create');
    const read = await get(port, '/platform/store/ns/k');
    assert(read.text === 'fresh', 'fresh data visible');
    const meta = JSON.parse(fs.readFileSync(path.join(TMP, 'store-meta', 'ns', 'k.json'), 'utf8'));
    assert(meta.mode === 'public-write' && !meta.capHash, 'new metadata replaces stale residue');
  } finally { await stopServer(server); }
}

async function test_client_serving() {
  setup();
  const { server, port } = await startServer();
  try {
    const r = await get(port, '/platform/client/skrynia.js');
    assert(r.status === 200, 'client served');
    assert(r.text.includes('Skrynia'), 'client content');
  } finally { await stopServer(server); }
}


async function test_client_derives_root_from_script_url() {
  const vm = require('vm');
  const source = fs.readFileSync(path.join(SRC, 'src', 'client.js'), 'utf8');
  const opened = [];

  function FakeXHR() {}
  FakeXHR.prototype.open = function(method, url) { opened.push({ method, url }); };
  FakeXHR.prototype.setRequestHeader = function() {};
  FakeXHR.prototype.getResponseHeader = function() { return null; };
  FakeXHR.prototype.send = function() {
    this.status = 404;
    this.response = new ArrayBuffer(0);
    this.onload();
  };

  const context = {
    URL,
    Promise,
    TextDecoder,
    Uint8Array,
    ArrayBuffer,
    XMLHttpRequest: FakeXHR,
    document: { currentScript: { src: 'https://storage.example/custom/root/client/skrynia.js' } },
    location: { href: 'https://app.example/apps/demo/' },
  };
  context.window = context;
  vm.runInNewContext(source, context);
  await context.Skrynia.store('ns').get('key');
  assert(opened.length === 1, 'client made one request');
  assert(
    opened[0].url === 'https://storage.example/custom/root/store/ns/key',
    'client derives nested cross-origin root from script URL: ' + opened[0].url
  );
}

async function test_client_etag_and_conditional_headers() {
  const vm = require('vm');
  const source = fs.readFileSync(path.join(SRC, 'src', 'client.js'), 'utf8');
  const requests = [];

  function FakeXHR() { this.requestHeaders = {}; requests.push(this); }
  FakeXHR.prototype.open = function(method, url) { this.method = method; this.url = url; };
  FakeXHR.prototype.setRequestHeader = function(name, value) { this.requestHeaders[name] = value; };
  FakeXHR.prototype.getResponseHeader = function(name) { return name === 'etag' ? '"version-1"' : null; };
  FakeXHR.prototype.send = function() {
    this.status = 200;
    this.response = new ArrayBuffer(0);
    this.onload();
  };

  const context = { URL, Promise, TextDecoder, Uint8Array, ArrayBuffer, XMLHttpRequest: FakeXHR };
  context.window = context;
  vm.runInNewContext(source, context);
  const store = context.Skrynia.store('ns');
  const result = await store.get('key');
  assert(result.meta.etag === '"version-1"', 'client exposes response etag');
  await store.put('key', 'data', { capability: 'cap', etag: result.meta.etag });
  assert(requests[1].requestHeaders['X-Skrynia-Capability'] === 'cap', 'client sends capability');
  assert(requests[1].requestHeaders['If-Match'] === '"version-1"', 'client sends conditional etag');
  await store.put('key', 'data');
  assert(!('If-Match' in requests[2].requestHeaders), 'unconditional put omits etag');
}

async function test_deploy_validation() {
  setup();
  const { server, port } = await startServer();
  try {
    let r = await managementGet(port, 'deploy', { repo: 'git@github.com:org/repo.git', commit: 'abc', subdir: '.', namespace: 'app' });
    assert(r.status === 400 && jsonBody(r).error === 'invalid_commit', 'short commit rejected');
    r = await managementGet(port, 'deploy', { repo: 'git@github.com:org/repo.git', commit: 'a'.repeat(40), subdir: '../x', namespace: 'app' });
    assert(r.status === 400 && jsonBody(r).error === 'invalid_subdir', 'traversal subdir rejected');
    r = await managementGet(port, 'deploy', { repo: 'git@github.com:org/repo.git', commit: 'a'.repeat(40), subdir: '.', namespace: 'Bad' });
    assert(r.status === 400 && jsonBody(r).error === 'invalid_namespace', 'invalid namespace rejected');
  } finally { await stopServer(server); }
}

async function test_deploy_rejects_non_ssh_repo() {
  setup();
  const { server, port } = await startServer();
  try {
    const cases = [
      ['/tmp/repo', 'absolute local path'],
      ['file:///tmp/repo', 'file:// URL'],
      ['http://github.com/org/repo.git', 'http:// URL'],
      ['https://github.com/org/repo.git', 'https:// URL'],
      ['ssh://git@github.com/org/repo.git', 'ssh:// URL'],
      ['ftp://example.com/repo.git', 'ftp:// URL'],
      ['../relative/path', 'relative local path'],
      ['repo.git', 'bare name without @host:'],
      ['user@host', 'scp-like without path after colon'],
    ];
    for (const [repo, desc] of cases) {
      const r = await managementGet(port, 'deploy', {
        repo,
        commit: 'a'.repeat(40),
        subdir: '.',
        namespace: 'ns',
      });
      assert(r.status === 400 && jsonBody(r).error === 'invalid_repo', desc + ' rejected, got ' + r.status + ': ' + r.text);
    }
  } finally { await stopServer(server); }
}

async function test_deploy_accepts_ssh_repo() {
  setup();
  const repo = path.join(TMP, 'repo-accept');
  const commit = makeRepo(repo, {
    'index.html': '<h1>accept</h1>',
    'Makefile': 'build:\n\tmkdir -p build && cp index.html build/index.html\n',
  });
  const sshRepo = 'git@github.com:org/repo.git';
  registerRepo(sshRepo, repo);

  await withFakeDocker(async () => {
    const { server, port } = await startServer({ appBasePath: '/apps' });
    try {
      let r = await managementGet(port, 'deploy', {
        repo: sshRepo,
        commit,
        subdir: '.',
        namespace: 'accept',
      });
      assert(r.status === 200, 'deploy via SSH-style repo: ' + r.text);
      const deployed = jsonBody(r);
      assert(deployed.ok && deployed.release, 'deploy result');
      assert(deployed.path === '/apps/accept/', 'deploy path');

      r = await get(port, '/apps/accept/');
      assert(r.status === 200 && r.text.includes('accept'), 'app served');
    } finally { await stopServer(server); }
  });
}

async function test_deploy_inspect_releases_and_serving() {
  setup();
  const repo = path.join(TMP, 'repo');
  const commit = makeRepo(repo, {
    'index.html': '<h1>version one</h1>',
    'Makefile': 'build:\n\tmkdir -p build && cp index.html build/index.html\n',
  });
  const sshRepo = 'git@test.invalid:myrepo.git';
  registerRepo(sshRepo, repo);

  await withFakeDocker(async () => {
    const { server, port } = await startServer({ appBasePath: '/site' });
    try {
      let r = await managementGet(port, 'deploy', {
        repo: sshRepo,
        commit,
        subdir: '.',
        namespace: 'birthday-list',
      });
      assert(r.status === 200, 'deploy status: ' + r.text);
      const deployed = jsonBody(r);
      assert(deployed.ok && deployed.release, 'deploy result');
      assert(deployed.path === '/site/birthday-list/', 'deploy path');

      r = await managementGet(port, 'inspect', { namespace: 'birthday-list' });
      assert(r.status === 200, 'inspect');
      assert(jsonBody(r).commit === commit, 'inspect commit');

      r = await managementGet(port, 'releases', { namespace: 'birthday-list' });
      assert(r.status === 200, 'releases');
      assert(jsonBody(r).current === deployed.release, 'current release');
      assert(jsonBody(r).releases.length === 1, 'one release');

      r = await get(port, '/site/birthday-list/');
      assert(r.status === 200 && r.text.includes('version one'), 'deployed app served');
    } finally { await stopServer(server); }
  });
}

async function test_deploy_uses_disposable_writable_builder_shape() {
  setup();
  const repo = path.join(TMP, 'repo');
  const commit = makeRepo(repo, {
    'index.html': '<h1>x</h1>',
    'Makefile': 'build:\n\tmkdir -p build && cp index.html build/index.html\n',
  });
  const sshRepo = 'git@test.invalid:shape.git';
  registerRepo(sshRepo, repo);
  const log = path.join(TMP, 'docker.log');

  await withFakeDocker(async () => {
    const { server, port } = await startServer();
    try {
      const r = await managementGet(port, 'deploy', { repo: sshRepo, commit, subdir: '.', namespace: 'shape' });
      assert(r.status === 200, 'deploy succeeds');
    } finally { await stopServer(server); }
  }, log);

  const args = fs.readFileSync(log, 'utf8');
  assert(args.includes('run --rm --pull=always'), 'builder always refreshes the requested image');
  assert(args.includes('--env HOME=/tmp'), 'builder sets HOME');
  assert(args.includes('-v ') && args.includes(':/repo'), 'builder mounts repo');
  assert(!args.includes('--read-only'), 'builder root is writable');
  assert(args.includes('--network host'), 'builder uses Docker host networking');
}

async function test_deploy_keeps_service_responsive() {
  setup(); createNs('live');
  const repo = path.join(TMP, 'repo-responsive');
  const commit = makeRepo(repo, {
    'index.html': '<h1>slow build</h1>',
    'Makefile': 'build:\n\tmkdir -p build && cp index.html build/index.html\n',
  });
  const sshRepo = 'git@test.invalid:responsive.git';
  registerRepo(sshRepo, repo);
  const log = path.join(TMP, 'docker-responsive.log');

  await withFakeDocker(async () => {
    process.env.SKRYNIA_TEST_DOCKER_SLEEP = '0.4';
    const { server, port } = await startServer();
    try {
      let r = await request(port, 'POST', '/platform/store/live/k', 'alive', {'X-Skrynia-Mode':'public-write'});
      assert(r.status === 201, 'seed storage object');

      let deployDone = false;
      const deployment = managementGet(port, 'deploy', {
        repo: sshRepo,
        commit,
        subdir: '.',
        namespace: 'slow',
      }).then(result => {
        deployDone = true;
        return result;
      });

      const deadline = Date.now() + 1000;
      while (!fs.existsSync(log) && Date.now() < deadline) {
        await new Promise(resolve => setTimeout(resolve, 5));
      }
      assert(fs.existsSync(log), 'deployment reached builder');

      const health = await get(port, '/platform/health');
      assert(health.status === 200 && jsonBody(health).ok === true, 'health responds during build');

      const stored = await get(port, '/platform/store/live/k');
      assert(stored.status === 200 && stored.text === 'alive', 'storage responds during build');

      const conflict = await managementGet(port, 'rollback', { namespace: 'slow' });
      assert(conflict.status === 409 && jsonBody(conflict).error === 'deployment_in_progress',
        'conflicting lifecycle mutation rejected during build');

      assert(!deployDone, 'deployment still running while health and storage requests completed');

      r = await deployment;
      assert(r.status === 200, 'slow deployment succeeds: ' + r.text);
    } finally {
      delete process.env.SKRYNIA_TEST_DOCKER_SLEEP;
      await stopServer(server);
    }
  }, log);
}

async function test_deploy_git_timeout_recovers() {
  setup(); createNs('git-live');
  const marker = path.join(TMP, 'hung-git');
  makeHangingGit(marker);
  const oldPath = process.env.PATH;
  process.env.PATH = FAKE_BIN + ':' + (oldPath || '/usr/bin:/bin');
  const { server, port } = await startServer({ gitTimeoutMs: 150 });
  try {
    await request(port, 'POST', '/platform/store/git-live/k', 'alive', {'X-Skrynia-Mode':'public-write'});
    const deployment = managementGet(port, 'deploy', {
      repo: 'git@test.invalid:hung.git', commit: 'a'.repeat(40), subdir: '.', namespace: 'hung-git',
    });
    const deadline = Date.now() + 1000;
    while (!fs.existsSync(marker) && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 5));
    assert(fs.existsSync(marker), 'hung git started');

    const health = await get(port, '/platform/health');
    assert(health.status === 200 && jsonBody(health).ok, 'health responds while git is hung');
    const stored = await get(port, '/platform/store/git-live/k');
    assert(stored.status === 200 && stored.text === 'alive', 'storage responds while git is hung');

    const result = await deployment;
    assert(result.status === 500 && jsonBody(result).error === 'git_failed', 'git timeout fails deployment');
    assert(jsonBody(result).detail.includes('clone failed timed out: stderr: clone still blocked'), 'git timeout has bounded diagnostic');
    assert(fs.readdirSync(path.join(TMP, 'builds')).length === 0, 'git timeout removes workspace');

    const recovered = await managementGet(port, 'ns/create', { namespace: 'after-git-timeout' });
    assert(recovered.status === 200, 'deployment lock recovers after git timeout');
  } finally {
    process.env.PATH = oldPath;
    await stopServer(server);
  }
}

async function test_deploy_builder_timeout_removes_container_and_recovers() {
  setup(); createNs('build-live');
  const repo = path.join(TMP, 'repo-hung-builder');
  const commit = makeRepo(repo, {
    'index.html': '<h1>hung</h1>',
    'Makefile': 'build:\n\tmkdir -p build && cp index.html build/index.html\n',
  });
  const sshRepo = 'git@test.invalid:hung-builder.git';
  registerRepo(sshRepo, repo);
  const started = path.join(TMP, 'container-started');
  const removed = path.join(TMP, 'container-removed');
  const log = path.join(TMP, 'hung-builder.log');
  makeHangingBuilderDocker(started, removed, log);
  const oldPath = process.env.PATH;
  process.env.PATH = FAKE_BIN + ':' + (oldPath || '/usr/bin:/bin');
  const { server, port } = await startServer({ buildTimeoutMs: 150 });
  try {
    await request(port, 'POST', '/platform/store/build-live/k', 'alive', {'X-Skrynia-Mode':'public-write'});
    const deployment = managementGet(port, 'deploy', { repo: sshRepo, commit, subdir: '.', namespace: 'hung-build' });
    const deadline = Date.now() + 1000;
    while (!fs.existsSync(started) && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 5));
    assert(fs.existsSync(started), 'hung builder started');

    const health = await get(port, '/platform/health');
    assert(health.status === 200 && jsonBody(health).ok, 'health responds while builder is hung');
    const stored = await get(port, '/platform/store/build-live/k');
    assert(stored.status === 200 && stored.text === 'alive', 'storage responds while builder is hung');

    const result = await deployment;
    assert(result.status === 500 && jsonBody(result).error === 'build_failed', 'builder timeout fails deployment');
    assert(jsonBody(result).detail.includes('build failed timed out: stderr: build still blocked in skrynia-build-'), 'builder timeout has bounded diagnostic');
    assert(fs.existsSync(removed), 'timed-out container is force-removed');
    const dockerLog = fs.readFileSync(log, 'utf8');
    assert(/run .*--name skrynia-build-\d+-1 .*--cidfile /.test(dockerLog), 'builder has deterministic name and cidfile');
    assert(dockerLog.includes('rm -f skrynia-build-'), 'cleanup uses the actual container identity');
    assert(fs.readdirSync(path.join(TMP, 'builds')).length === 0, 'builder timeout removes workspace');
    const releaseBase = path.join(TMP, 'releases', 'hung-build');
    assert(!fs.existsSync(releaseBase) || fs.readdirSync(releaseBase).every(name => !name.startsWith('.staging-')), 'builder timeout removes staging');

    const recovered = await managementGet(port, 'ns/create', { namespace: 'after-build-timeout' });
    assert(recovered.status === 200, 'deployment lock recovers after builder timeout');
  } finally {
    process.env.PATH = oldPath;
    await stopServer(server);
  }
}

async function test_builder_cleanup_timeout_is_bounded() {
  setup();
  const repo = path.join(TMP, 'repo-hung-cleanup');
  const commit = makeRepo(repo, {
    'index.html': '<h1>hung cleanup</h1>',
    'Makefile': 'build:\n\tmkdir -p build && cp index.html build/index.html\n',
  });
  const sshRepo = 'git@test.invalid:hung-cleanup.git';
  registerRepo(sshRepo, repo);
  const started = path.join(TMP, 'cleanup-builder-started');
  const cleanupStarted = path.join(TMP, 'cleanup-started');
  makeHangingCleanupDocker(started, cleanupStarted);
  const oldPath = process.env.PATH;
  process.env.PATH = FAKE_BIN + ':' + (oldPath || '/usr/bin:/bin');
  process.env.SKRYNIA_DEPLOY_SECRET = 'cleanup-secret-value';
  const { server, port } = await startServer({ buildTimeoutMs: 100 });
  try {
    const deployment = managementGet(port, 'deploy', { repo: sshRepo, commit, subdir: '.', namespace: 'hung-cleanup' });
    const deadline = Date.now() + 1000;
    while (!fs.existsSync(cleanupStarted) && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 5));
    assert(fs.existsSync(cleanupStarted), 'container cleanup started');

    const health = await get(port, '/platform/health');
    assert(health.status === 200 && jsonBody(health).ok, 'health responds while cleanup is hung');
    const conflict = await managementGet(port, 'ns/create', { namespace: 'during-cleanup' });
    assert(conflict.status === 409 && jsonBody(conflict).error === 'deployment_in_progress',
      'deployment lock remains held during bounded container cleanup');

    const result = await deployment;
    const detail = jsonBody(result).detail;
    assert(result.status === 500 && jsonBody(result).error === 'build_failed', 'builder cleanup timeout fails deployment');
    assert(detail.includes('build failed timed out: stderr: builder cleanup is bounded'), 'builder timeout identifies subprocess');
    assert(detail.includes('cleanup: stderr: docker timed out') && detail.includes('stderr: cleanup secret [redacted]'), 'cleanup failure is specific and redacted: ' + detail);
    assert(detail.length < 20000, 'builder timeout diagnostic stays bounded');
    assert(fs.readdirSync(path.join(TMP, 'builds')).length === 0, 'cleanup timeout removes workspace');

    const recovered = await managementGet(port, 'ns/create', { namespace: 'after-cleanup-timeout' });
    assert(recovered.status === 200, 'deployment lock recovers after bounded cleanup failure');
  } finally {
    delete process.env.SKRYNIA_DEPLOY_SECRET;
    process.env.PATH = oldPath;
    await stopServer(server);
  }
}

async function test_builder_cleanup_failure_is_reported() {
  setup();
  const repo = path.join(TMP, 'repo-failed-cleanup');
  const commit = makeRepo(repo, {
    'index.html': '<h1>failed cleanup</h1>',
    'Makefile': 'build:\n\tmkdir -p build && cp index.html build/index.html\n',
  });
  const sshRepo = 'git@test.invalid:failed-cleanup.git';
  registerRepo(sshRepo, repo);
  const started = path.join(TMP, 'failed-cleanup-builder-started');
  makeFailedCleanupDocker(started);
  const oldPath = process.env.PATH;
  process.env.PATH = FAKE_BIN + ':' + (oldPath || '/usr/bin:/bin');
  process.env.SKRYNIA_DEPLOY_SECRET = 'cleanup-name-secret';
  const { server, port } = await startServer({ buildTimeoutMs: 100 });
  try {
    const result = await managementGet(port, 'deploy', { repo: sshRepo, commit, subdir: '.', namespace: 'failed-cleanup' });
    const detail = jsonBody(result).detail;
    assert(result.status === 500 && jsonBody(result).error === 'build_failed', 'cleanup failure remains a build timeout error');
    assert(detail.includes('build failed timed out: stderr: builder before failed cleanup'), 'builder timeout identifies subprocess');
    assert(detail.includes('cleanup: stderr: docker failed') && detail.includes('stderr: name removal secret [redacted] failed'), 'failed name removal is reported and redacted: ' + detail);
    assert(!detail.includes('cleanup succeeded'), 'failed cleanup is never reported as successful');
    assert(detail.length < 20000, 'failed cleanup diagnostic stays bounded');
    assert(fs.readdirSync(path.join(TMP, 'builds')).length === 0, 'failed cleanup removes workspace');

    const health = await get(port, '/platform/health');
    assert(health.status === 200 && jsonBody(health).ok, 'health responds after failed cleanup');
    const recovered = await managementGet(port, 'ns/create', { namespace: 'after-failed-cleanup' });
    assert(recovered.status === 200, 'deployment lock recovers after failed cleanup');
  } finally {
    delete process.env.SKRYNIA_DEPLOY_SECRET;
    process.env.PATH = oldPath;
    await stopServer(server);
  }
}

async function test_rollback_and_undeploy_http() {
  setup();
  const repo = path.join(TMP, 'repo');
  const first = makeRepo(repo, {
    'index.html': '<h1>one</h1>',
    'Makefile': 'build:\n\tmkdir -p build && cp index.html build/index.html\n',
  });
  const sshRepo = 'git@test.invalid:rb.git';
  registerRepo(sshRepo, repo);

  await withFakeDocker(async () => {
    const { server, port } = await startServer({ appBasePath: '/apps' });
    try {
      let r = await managementGet(port, 'deploy', { repo: sshRepo, commit: first, subdir: '.', namespace: 'rb' });
      assert(r.status === 200, 'first deploy');
      const firstRelease = jsonBody(r).release;

      const second = commitRepo(repo, 'index.html', '<h1>two</h1>', 'two');
      r = await managementGet(port, 'deploy', { repo: sshRepo, commit: second, subdir: '.', namespace: 'rb' });
      assert(r.status === 200, 'second deploy');
      const secondRelease = jsonBody(r).release;
      assert(firstRelease !== secondRelease, 'release ids differ');

      r = await get(port, '/apps/rb/');
      assert(r.text.includes('two'), 'second active');

      r = await managementGet(port, 'rollback', { namespace: 'rb' });
      assert(r.status === 200, 'rollback');
      assert(jsonBody(r).release === firstRelease, 'rolled back to first');
      r = await get(port, '/apps/rb/');
      assert(r.text.includes('one'), 'first active again');

      r = await managementGet(port, 'undeploy', { namespace: 'rb' });
      assert(r.status === 200, 'undeploy');
      r = await get(port, '/apps/rb/');
      assert(r.status === 503, 'undeployed app unavailable');
      assert(!fs.existsSync(path.join(TMP, 'state', 'rb')), 'state removed');
      assert(!fs.existsSync(path.join(TMP, 'storage', 'rb')), 'storage removed');
    } finally { await stopServer(server); }
  });
}

async function test_external_app_dir_activation() {
  setup();
  const appDir = path.join(TMP, 'public-apps');
  fs.mkdirSync(appDir, { recursive: true });
  const repo = path.join(TMP, 'repo');
  const commit = makeRepo(repo, {
    'index.html': '<h1>external</h1>',
    'Makefile': 'build:\n\tmkdir -p build && cp index.html build/index.html\n',
  });
  const sshRepo = 'git@test.invalid:external.git';
  registerRepo(sshRepo, repo);

  await withFakeDocker(async () => {
    const { server, port } = await startServer({ appDir });
    try {
      const r = await managementGet(port, 'deploy', { repo: sshRepo, commit, subdir: '.', namespace: 'external' });
      assert(r.status === 200, 'external deploy');
      const link = path.join(appDir, 'external');
      assert(fs.lstatSync(link).isSymbolicLink(), 'external active link exists');
      assert(fs.realpathSync(link).includes(path.join('releases', 'external')), 'external link points to release');
    } finally { await stopServer(server); }
  });
}

async function test_base_path_helpers() {
  assert(normalizeBasePath('/apps/') === '/apps', 'trailing slash');
  assert(normalizeBasePath('/') === '/', 'root base path');
  let failed = false;
  try { normalizeBasePath('apps'); } catch { failed = true; }
  assert(failed, 'base path requires leading slash');
}

async function test_examples_build() {
  const hello = path.join(SRC, 'example', 'hello');
  const birthday = path.join(SRC, 'example', 'birthday-list');
  rmrf(path.join(hello, 'build'));
  rmrf(path.join(birthday, 'build'));
  execFileSync('make', ['build'], { cwd: hello, stdio: 'pipe' });
  execFileSync('npm', ['run', 'build'], { cwd: birthday, stdio: 'pipe' });
  assert(fs.existsSync(path.join(hello, 'build', 'index.html')), 'hello builds');
  assert(fs.existsSync(path.join(birthday, 'build', 'index.html')), 'birthday list builds');
  rmrf(path.join(hello, 'build'));
  rmrf(path.join(birthday, 'build'));
}

async function test_cli_deleted() {
  assert(!fs.existsSync(path.join(SRC, 'src', 'admin.js')), 'admin.js is deleted');
}

async function test_createShared_observes_env_at_call_time() {
  const saved1 = process.env.SKRYNIA_DEFAULT_QUOTA_BYTES;
  const saved2 = process.env.SKRYNIA_MAX_OBJECT_COUNT;
  try {
    process.env.SKRYNIA_DEFAULT_QUOTA_BYTES = '2048';
    process.env.SKRYNIA_MAX_OBJECT_COUNT = '50';
    const s1 = createShared('/tmp/skrynia-test-env1');
    assert(s1.DEFAULT_QUOTA_BYTES === 2048, 'first call reads 2048, got ' + s1.DEFAULT_QUOTA_BYTES);
    assert(s1.DEFAULT_MAX_OBJECTS === 50, 'first call reads 50, got ' + s1.DEFAULT_MAX_OBJECTS);

    process.env.SKRYNIA_DEFAULT_QUOTA_BYTES = '4096';
    process.env.SKRYNIA_MAX_OBJECT_COUNT = '99';
    const s2 = createShared('/tmp/skrynia-test-env2');
    assert(s2.DEFAULT_QUOTA_BYTES === 4096, 'second call reads 4096, got ' + s2.DEFAULT_QUOTA_BYTES);
    assert(s2.DEFAULT_MAX_OBJECTS === 99, 'second call reads 99, got ' + s2.DEFAULT_MAX_OBJECTS);
  } finally {
    if (saved1 === undefined) delete process.env.SKRYNIA_DEFAULT_QUOTA_BYTES;
    else process.env.SKRYNIA_DEFAULT_QUOTA_BYTES = saved1;
    if (saved2 === undefined) delete process.env.SKRYNIA_MAX_OBJECT_COUNT;
    else process.env.SKRYNIA_MAX_OBJECT_COUNT = saved2;
    rmrf('/tmp/skrynia-test-env1');
    rmrf('/tmp/skrynia-test-env2');
  }
}

async function test_quota_derived_from_filesystem() {
  setup();
  fs.mkdirSync(path.join(TMP, 'state', 'dq'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', 'dq', 'quota.json'), JSON.stringify({quotaBytes:100,maxObjects:100}));
  fs.mkdirSync(path.join(TMP, 'store', 'dq'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'store', 'dq', 'a'), '12345');
  fs.writeFileSync(path.join(TMP, 'store', 'dq', 'b'), '67890');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/platform/store/dq/k', 'x'.repeat(90), {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'fits within remaining 90 bytes, got ' + r.status);
    r = await request(port, 'POST', '/platform/store/dq/k2', 'y', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 507, 'exceeds quota after derived usage, got ' + r.status);
  } finally { await stopServer(server); }
}

async function test_stale_counters_ignored() {
  setup();
  fs.mkdirSync(path.join(TMP, 'state', 'sc'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', 'sc', 'quota.json'), JSON.stringify({bytes:999999,count:9999,quotaBytes:100,maxObjects:100}));
  fs.mkdirSync(path.join(TMP, 'store', 'sc'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'store', 'sc', 'only'), 'data');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/platform/store/sc/k', 'hi', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'stale counts ignored, create succeeds, got ' + r.status);
    const onDisk = JSON.parse(fs.readFileSync(path.join(TMP, 'state', 'sc', 'quota.json'), 'utf8'));
    assert(!('bytes' in onDisk), 'legacy bytes removed from persisted file');
    assert(!('count' in onDisk), 'legacy count removed from persisted file');
    assert(onDisk.quotaBytes === 100, 'quotaBytes preserved');
    assert(onDisk.maxObjects === 100, 'maxObjects preserved');
  } finally { await stopServer(server); }
}

async function test_ns_create_never_persists_derived() {
  setup();
  const { server, port } = await startServer();
  try {
    await managementGet(port, 'ns/create', { namespace: 'nderived', quota: 500 });
    const onDisk = JSON.parse(fs.readFileSync(path.join(TMP, 'state', 'nderived', 'quota.json'), 'utf8'));
    assert(!('bytes' in onDisk), 'ns/create does not persist bytes');
    assert(!('count' in onDisk), 'ns/create does not persist count');
    assert(onDisk.quotaBytes === 500, 'quotaBytes set correctly');
  } finally { await stopServer(server); }
}

const tests = [
  ['health', test_health],
  ['version', test_version],
  ['root_url', test_root_url],
  ['management_requires_configured_token', test_management_requires_configured_token],
  ['management_rejects_missing_or_bad_token', test_management_rejects_missing_or_bad_token],
  ['management_get_only', test_management_get_only],
  ['namespace_http_api', test_namespace_http_api],
  ['store_requires_namespace', test_store_requires_namespace],
  ['store_crud', test_store_crud],
  ['store_absence_does_not_disclose_namespace_existence', test_store_absence_does_not_disclose_namespace_existence],
  ['store_path_neither_logs_nor_reflects_the_secret_url', test_store_path_neither_logs_nor_reflects_the_secret_url],
  ['store_error_logging_is_path_free', test_store_error_logging_is_path_free],
  ['legacy_migration_failure_is_path_free', test_legacy_migration_failure_is_path_free],
  ['legacy_migration_filesystem_failure_is_path_free', test_legacy_migration_filesystem_failure_is_path_free],
  ['store_read_failure_is_path_free_and_answered', test_store_read_failure_is_path_free_and_answered],
  ['management_filesystem_failure_is_path_free', test_management_filesystem_failure_is_path_free],
  ['push_subscription_failure_is_path_free', test_push_subscription_failure_is_path_free],
  ['secret_url_responses_forbid_referrer_and_retention', test_secret_url_responses_forbid_referrer_and_retention],
  ['plain_http_public_root_is_rejected', test_plain_http_public_root_is_rejected],
  ['etag_and_conditional_replace', test_etag_and_conditional_replace],
  ['conditional_replace_authorization_and_race', test_conditional_replace_authorization_and_race],
  ['capability_write', test_capability_write],
  ['immutable_mode_is_not_accepted', test_immutable_mode_is_not_accepted],
  ['legacy_immutable_object_is_writable_in_place', test_legacy_immutable_object_is_writable_in_place],
  ['legacy_immutable_object_is_not_migrated', test_legacy_immutable_object_is_not_migrated],
  ['capability_write_is_unaffected_by_legacy_mode', test_capability_write_is_unaffected_by_legacy_mode],
  ['legacy_storage_migration_yields_writable_modes', test_legacy_storage_migration_yields_writable_modes],
  ['key_validation', test_key_validation],
  ['quota_and_count_limits', test_quota_and_count_limits],
  ['incomplete_create_is_reclaimed', test_incomplete_create_is_reclaimed],
  ['client_serving', test_client_serving],
  ['client_derives_root_from_script_url', test_client_derives_root_from_script_url],
  ['client_etag_and_conditional_headers', test_client_etag_and_conditional_headers],
  ['deploy_validation', test_deploy_validation],
  ['deploy_rejects_non_ssh_repo', test_deploy_rejects_non_ssh_repo],
  ['deploy_accepts_ssh_repo', test_deploy_accepts_ssh_repo],
  ['deploy_inspect_releases_and_serving', test_deploy_inspect_releases_and_serving],
  ['deploy_uses_disposable_writable_builder_shape', test_deploy_uses_disposable_writable_builder_shape],
  ['deploy_keeps_service_responsive', test_deploy_keeps_service_responsive],
  ['deploy_git_timeout_recovers', test_deploy_git_timeout_recovers],
  ['deploy_builder_timeout_removes_container_and_recovers', test_deploy_builder_timeout_removes_container_and_recovers],
  ['builder_cleanup_timeout_is_bounded', test_builder_cleanup_timeout_is_bounded],
  ['builder_cleanup_failure_is_reported', test_builder_cleanup_failure_is_reported],
  ['rollback_and_undeploy_http', test_rollback_and_undeploy_http],
  ['external_app_dir_activation', test_external_app_dir_activation],
  ['base_path_helpers', test_base_path_helpers],
  ['examples_build', test_examples_build],
  ['cli_deleted', test_cli_deleted],
  ['createShared_observes_env', test_createShared_observes_env_at_call_time],
  ['quota_derived_from_filesystem', test_quota_derived_from_filesystem],
  ['stale_counters_ignored', test_stale_counters_ignored],
  ['ns_create_never_persists_derived', test_ns_create_never_persists_derived],
];

(async () => {
  let pass = 0;
  let fail = 0;
  for (const [name, fn] of tests) {
    process.stdout.write('  ' + name + ' ... ');
    try {
      await fn();
      console.log('ok');
      pass++;
    } catch (e) {
      console.log('FAIL');
      console.error(e && e.stack ? e.stack : e);
      fail++;
    }
  }
  console.log('\nResults: ' + pass + ' passed, ' + fail + ' failed, ' + (pass + fail) + ' total');
  process.exit(fail ? 1 : 0);
})().catch(e => {
  console.error(e && e.stack ? e.stack : e);
  process.exit(1);
});
