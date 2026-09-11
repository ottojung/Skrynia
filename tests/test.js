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
  for (const name of ['releases', 'storage', 'state', 'builds']) fs.mkdirSync(path.join(TMP, name), { recursive: true });
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
  fs.mkdirSync(path.join(TMP, 'storage', ns), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'state', ns), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', ns, 'quota.json'), JSON.stringify({
    quotaBytes: quotaBytes || 10485760,
    maxObjects: maxObjects || 10000,
  }));
}

function startServer(extraOpts) {
  const server = createServer(Object.assign({ dataDir: TMP, token: TOKEN }, extraOpts || {}));
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
  return '/_skrynia/' + endpoint + '?' + q.toString();
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
    const r = await get(port, '/_skrynia/health');
    assert(r.status === 200, 'health status');
    assert(jsonBody(r).ok === true, 'health body');
  } finally { await stopServer(server); }
}

async function test_management_requires_configured_token() {
  setup();
  const { server, port } = await startServer({ token: '' });
  try {
    const r = await get(port, '/_skrynia/ns/list?token=anything');
    assert(r.status === 503, 'disabled management status');
    assert(jsonBody(r).error === 'management_api_disabled', 'disabled management error');
  } finally { await stopServer(server); }
}

async function test_management_rejects_missing_or_bad_token() {
  setup();
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/_skrynia/ns/list');
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
    let r = await request(port, 'POST', '/_skrynia/store/missing/x', 'x', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 409, 'missing namespace blocks create');
    await managementGet(port, 'ns/create', { namespace: 'created' });
    r = await request(port, 'POST', '/_skrynia/store/created/x', 'x', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'HTTP-created namespace accepts store write');
  } finally { await stopServer(server); }
}

async function test_store_crud() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/_skrynia/store/ns/hello', 'world', {'Content-Type':'text/plain','X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'create');
    r = await get(port, '/_skrynia/store/ns/hello');
    assert(r.status === 200 && r.text === 'world', 'read');
    r = await request(port, 'PUT', '/_skrynia/store/ns/hello', 'updated', {'Content-Type':'text/plain'});
    assert(r.status === 200, 'put');
    r = await get(port, '/_skrynia/store/ns/hello');
    assert(r.text === 'updated', 'updated read');
    r = await request(port, 'DELETE', '/_skrynia/store/ns/hello');
    assert(r.status === 200, 'delete');
    r = await get(port, '/_skrynia/store/ns/hello');
    assert(r.status === 404, 'deleted');
  } finally { await stopServer(server); }
}

async function test_capability_write() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/_skrynia/store/ns/k', 'one', {'X-Skrynia-Mode':'capability-write'});
    const cap = jsonBody(r).capability;
    assert(r.status === 201 && cap && cap.length === 64, 'capability returned');
    r = await request(port, 'PUT', '/_skrynia/store/ns/k', 'two');
    assert(r.status === 403, 'capability required');
    r = await request(port, 'PUT', '/_skrynia/store/ns/k', 'two', {'X-Skrynia-Capability':cap});
    assert(r.status === 200, 'capability permits put');
    r = await request(port, 'DELETE', '/_skrynia/store/ns/k', null, {'X-Skrynia-Capability':cap});
    assert(r.status === 200, 'capability permits delete');
  } finally { await stopServer(server); }
}

async function test_immutable() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    await request(port, 'POST', '/_skrynia/store/ns/k', 'one', {'X-Skrynia-Mode':'immutable'});
    let r = await request(port, 'PUT', '/_skrynia/store/ns/k', 'two');
    assert(r.status === 403, 'immutable put blocked');
    r = await request(port, 'DELETE', '/_skrynia/store/ns/k');
    assert(r.status === 403, 'immutable delete blocked');
  } finally { await stopServer(server); }
}

async function test_key_validation() {
  setup(); createNs('ns');
  const { server, port } = await startServer();
  try {
    let r = await get(port, '/_skrynia/store/ns/a%2Fb');
    assert(r.status === 400, 'slash rejected');
    r = await get(port, '/_skrynia/store/ns/..%2Fetc');
    assert(r.status === 400, 'traversal rejected');
    r = await get(port, '/_skrynia/store/INVALID/key');
    assert(r.status === 400, 'namespace rejected');
  } finally { await stopServer(server); }
}

async function test_quota_and_count_limits() {
  setup(); createNs('small', 4, 1);
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/_skrynia/store/small/a', '1234', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'first object fits');
    r = await request(port, 'POST', '/_skrynia/store/small/b', 'x', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 507, 'object count blocks second object');
    r = await request(port, 'PUT', '/_skrynia/store/small/a', '12345');
    assert(r.status === 507, 'quota blocks growth');
  } finally { await stopServer(server); }
}

async function test_incomplete_create_is_reclaimed() {
  setup(); createNs('ns');
  const dir = path.join(TMP, 'storage', 'ns');
  fs.writeFileSync(path.join(dir, 'k.dat'), 'orphan');
  fs.writeFileSync(path.join(dir, 'k.cap'), 'orphan');
  const { server, port } = await startServer();
  try {
    const r = await request(port, 'POST', '/_skrynia/store/ns/k', 'fresh', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'incomplete create reclaimed');
    const read = await get(port, '/_skrynia/store/ns/k');
    assert(read.text === 'fresh', 'fresh data visible');
  } finally { await stopServer(server); }
}

async function test_client_serving() {
  setup();
  const { server, port } = await startServer();
  try {
    const r = await get(port, '/_skrynia/client/skrynia.js');
    assert(r.status === 200, 'client served');
    assert(r.text.includes('Skrynia'), 'client content');
  } finally { await stopServer(server); }
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
  assert(args.includes('run --rm'), 'builder uses --rm');
  assert(args.includes('--env HOME=/tmp'), 'builder sets HOME');
  assert(args.includes('-v ') && args.includes(':/repo'), 'builder mounts repo');
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
  fs.mkdirSync(path.join(TMP, 'storage', 'dq'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'storage', 'dq', 'a.dat'), '12345');
  fs.writeFileSync(path.join(TMP, 'storage', 'dq', 'a.meta'), '{}');
  fs.writeFileSync(path.join(TMP, 'storage', 'dq', 'b.dat'), '67890');
  fs.writeFileSync(path.join(TMP, 'storage', 'dq', 'b.meta'), '{}');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/_skrynia/store/dq/k', 'x'.repeat(90), {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 201, 'fits within remaining 90 bytes, got ' + r.status);
    r = await request(port, 'POST', '/_skrynia/store/dq/k2', 'y', {'X-Skrynia-Mode':'public-write'});
    assert(r.status === 507, 'exceeds quota after derived usage, got ' + r.status);
  } finally { await stopServer(server); }
}

async function test_stale_counters_ignored() {
  setup();
  fs.mkdirSync(path.join(TMP, 'state', 'sc'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', 'sc', 'quota.json'), JSON.stringify({bytes:999999,count:9999,quotaBytes:100,maxObjects:100}));
  fs.mkdirSync(path.join(TMP, 'storage', 'sc'), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'storage', 'sc', 'only.dat'), 'data');
  fs.writeFileSync(path.join(TMP, 'storage', 'sc', 'only.meta'), '{}');
  const { server, port } = await startServer();
  try {
    let r = await request(port, 'POST', '/_skrynia/store/sc/k', 'hi', {'X-Skrynia-Mode':'public-write'});
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
  ['management_requires_configured_token', test_management_requires_configured_token],
  ['management_rejects_missing_or_bad_token', test_management_rejects_missing_or_bad_token],
  ['management_get_only', test_management_get_only],
  ['namespace_http_api', test_namespace_http_api],
  ['store_requires_namespace', test_store_requires_namespace],
  ['store_crud', test_store_crud],
  ['capability_write', test_capability_write],
  ['immutable', test_immutable],
  ['key_validation', test_key_validation],
  ['quota_and_count_limits', test_quota_and_count_limits],
  ['incomplete_create_is_reclaimed', test_incomplete_create_is_reclaimed],
  ['client_serving', test_client_serving],
  ['deploy_validation', test_deploy_validation],
  ['deploy_rejects_non_ssh_repo', test_deploy_rejects_non_ssh_repo],
  ['deploy_accepts_ssh_repo', test_deploy_accepts_ssh_repo],
  ['deploy_inspect_releases_and_serving', test_deploy_inspect_releases_and_serving],
  ['deploy_uses_disposable_writable_builder_shape', test_deploy_uses_disposable_writable_builder_shape],
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
