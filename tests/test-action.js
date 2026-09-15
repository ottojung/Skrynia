#!/usr/bin/env node
'use strict';

const http = require('http');
const { execFile } = require('child_process');
const fs = require('fs');
const path = require('path');

const ACTION = path.join(__dirname, '..', 'action', 'deploy', 'index.js');
const TMP = '/tmp/skrynia-action-test';
const TOKEN = 'test-token-abc123';

function rmrf(d) { try { fs.rmSync(d, { recursive: true, force: true }); } catch {} }
function assert(cond, msg) { if (!cond) throw new Error('ASSERT: ' + msg); }

let pass = 0;
let fail = 0;

function run(env) {
  return new Promise(function (resolve) {
    const outputDir = path.join(TMP, 'out-' + Math.random().toString(36).slice(2));
    fs.mkdirSync(outputDir, { recursive: true });
    const outputFile = path.join(outputDir, 'output');
    fs.writeFileSync(outputFile, '');

    const fullEnv = Object.assign({}, process.env, {
      GITHUB_OUTPUT: outputFile,
      GITHUB_REPOSITORY: '',
      GITHUB_SHA: '',
      INPUT_NAMESPACE: '',
      INPUT_TOKEN: '',
      'INPUT_SKRYNIA-URL': '',
      INPUT_REPO: '',
      INPUT_COMMIT: '',
      INPUT_SUBDIR: '',
      INPUT_BUILDER: '',
    }, env);

    execFile('node', [ACTION], { env: fullEnv, timeout: 10000 }, function (err, stdout, stderr) {
      var outputs = {};
      try {
        var content = fs.readFileSync(outputFile, 'utf8');
        for (var line of content.split('\n')) {
          var idx = line.indexOf('=');
          if (idx > 0) outputs[line.substring(0, idx)] = line.substring(idx + 1);
        }
      } catch {}
      rmrf(outputDir);
      resolve({
        exit: err ? err.code || 1 : 0,
        stdout: stdout,
        stderr: stderr,
        outputs: outputs,
      });
    });
  });
}

function startServer(handler) {
  return new Promise(function (resolve) {
    var server = http.createServer(handler);
    server.listen(0, '127.0.0.1', function () {
      resolve({ server: server, port: server.address().port });
    });
  });
}

function stopServer(s) {
  return new Promise(function (resolve) { s.close(resolve); });
}

async function test_missing_namespace() {
  var r = await run({
    INPUT_TOKEN: TOKEN,
    'INPUT_SKRYNIA-URL': 'http://127.0.0.1:1',
    INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
  });
  assert(r.exit !== 0, 'should fail without namespace');
  assert(r.stderr.includes('namespace is required'), 'error mentions namespace: ' + r.stderr);
  pass++; console.log('  missing_namespace ... ok');
}

async function test_missing_token() {
  var r = await run({
    INPUT_NAMESPACE: 'myapp',
    'INPUT_SKRYNIA-URL': 'http://127.0.0.1:1',
    INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
  });
  assert(r.exit !== 0, 'should fail without token');
  assert(r.stderr.includes('token is required'), 'error mentions token: ' + r.stderr);
  pass++; console.log('  missing_token ... ok');
}

async function test_missing_url() {
  var r = await run({
    INPUT_NAMESPACE: 'myapp',
    INPUT_TOKEN: TOKEN,
    INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
  });
  assert(r.exit !== 0, 'should fail without url');
  assert(r.stderr.includes('skrynia-url is required'), 'error mentions url: ' + r.stderr);
  pass++; console.log('  missing_url ... ok');
}

async function test_missing_commit_and_repo() {
  var r = await run({
    INPUT_NAMESPACE: 'myapp',
    INPUT_TOKEN: TOKEN,
    'INPUT_SKRYNIA-URL': 'http://127.0.0.1:1',
  });
  assert(r.exit !== 0, 'should fail without commit or GITHUB_SHA');
  assert(r.stderr.includes('repo is required') || r.stderr.includes('commit is required'),
    'error mentions repo or commit: ' + r.stderr);
  pass++; console.log('  missing_commit_and_repo ... ok');
}

async function test_successful_deploy() {
  var srv = await startServer(function (req, res) {
    var url = new URL(req.url, 'http://localhost');
    assert(url.pathname === '/platform/deploy', 'correct path');
    assert(url.searchParams.get('repo') === 'git@github.com:myorg/myapp.git', 'repo param');
    assert(url.searchParams.get('commit') === 'aabbccddeeff0011223344556677889900112233', 'commit param');
    assert(url.searchParams.get('subdir') === '.', 'subdir param');
    assert(url.searchParams.get('namespace') === 'myapp', 'namespace param');
    assert(url.searchParams.get('token') === TOKEN, 'token param');
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, namespace: 'myapp', release: '20260910120000000-abc', path: '/apps/myapp/' }));
  });
  try {
    var r = await run({
      INPUT_NAMESPACE: 'myapp',
      INPUT_TOKEN: TOKEN,
      'INPUT_SKRYNIA-URL': 'http://127.0.0.1:' + srv.port + '/platform',
      INPUT_REPO: 'git@github.com:myorg/myapp.git',
      INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
    });
    assert(r.exit === 0, 'should succeed: ' + r.stderr);
    assert(r.outputs.release === '20260910120000000-abc', 'release output');
    assert(r.outputs.path === '/apps/myapp/', 'path output');
    assert(r.stdout.includes('Deploy succeeded'), 'success message');
    assert(!r.stdout.includes(TOKEN), 'token not printed');
    pass++; console.log('  successful_deploy ... ok');
  } finally { await stopServer(srv.server); }
}


async function test_root_url_deploy() {
  var srv = await startServer(function (req, res) {
    var url = new URL(req.url, 'http://localhost');
    assert(url.pathname === '/deploy', 'root URL deploy path');
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, release: 'root' }));
  });
  try {
    var r = await run({
      INPUT_NAMESPACE: 'ns',
      INPUT_TOKEN: TOKEN,
      'INPUT_SKRYNIA-URL': 'http://127.0.0.1:' + srv.port,
      INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
      INPUT_REPO: 'git@github.com:org/repo.git',
    });
    assert(r.exit === 0, 'root URL should succeed: ' + r.stderr);
    pass++; console.log('  root_url_deploy ... ok');
  } finally { await stopServer(srv.server); }
}

async function test_default_repo_from_github_repository() {
  var srv = await startServer(function (req, res) {
    var url = new URL(req.url, 'http://localhost');
    assert(url.searchParams.get('repo') === 'git@github.com:org/repo.git', 'default repo');
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, release: 'r1' }));
  });
  try {
    var r = await run({
      INPUT_NAMESPACE: 'ns',
      INPUT_TOKEN: TOKEN,
      'INPUT_SKRYNIA-URL': 'http://127.0.0.1:' + srv.port + '/platform',
      INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
      GITHUB_REPOSITORY: 'org/repo',
    });
    assert(r.exit === 0, 'should succeed: ' + r.stderr);
    assert(r.outputs.release === 'r1', 'release output');
    pass++; console.log('  default_repo ... ok');
  } finally { await stopServer(srv.server); }
}

async function test_http_error() {
  var srv = await startServer(function (req, res) {
    res.writeHead(400, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'invalid_commit' }));
  });
  try {
    var r = await run({
      INPUT_NAMESPACE: 'ns',
      INPUT_TOKEN: TOKEN,
      'INPUT_SKRYNIA-URL': 'http://127.0.0.1:' + srv.port + '/platform',
      INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
      INPUT_REPO: 'git@github.com:org/repo.git',
    });
    assert(r.exit !== 0, 'should fail on 400');
    assert(r.stderr.includes('400'), 'mentions HTTP 400');
    assert(r.stderr.includes('invalid_commit'), 'includes error body');
    pass++; console.log('  http_error ... ok');
  } finally { await stopServer(srv.server); }
}

async function test_builder_param() {
  var srv = await startServer(function (req, res) {
    var url = new URL(req.url, 'http://localhost');
    assert(url.searchParams.get('builder') === 'custom:latest', 'builder param');
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, release: 'r2' }));
  });
  try {
    var r = await run({
      INPUT_NAMESPACE: 'ns',
      INPUT_TOKEN: TOKEN,
      'INPUT_SKRYNIA-URL': 'http://127.0.0.1:' + srv.port + '/platform',
      INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
      INPUT_REPO: 'git@github.com:org/repo.git',
      INPUT_BUILDER: 'custom:latest',
    });
    assert(r.exit === 0, 'should succeed: ' + r.stderr);
    assert(r.outputs.release === 'r2', 'release output');
    pass++; console.log('  builder_param ... ok');
  } finally { await stopServer(srv.server); }
}

async function test_subdir_param() {
  var srv = await startServer(function (req, res) {
    var url = new URL(req.url, 'http://localhost');
    assert(url.searchParams.get('subdir') === 'example/app', 'subdir param');
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, release: 'r3' }));
  });
  try {
    var r = await run({
      INPUT_NAMESPACE: 'ns',
      INPUT_TOKEN: TOKEN,
      'INPUT_SKRYNIA-URL': 'http://127.0.0.1:' + srv.port + '/platform',
      INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
      INPUT_REPO: 'git@github.com:org/repo.git',
      INPUT_SUBDIR: 'example/app',
    });
    assert(r.exit === 0, 'should succeed: ' + r.stderr);
    pass++; console.log('  subdir_param ... ok');
  } finally { await stopServer(srv.server); }
}

async function test_token_not_in_output() {
  var srv = await startServer(function (req, res) {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, release: 'r4' }));
  });
  try {
    var r = await run({
      INPUT_NAMESPACE: 'ns',
      INPUT_TOKEN: TOKEN,
      'INPUT_SKRYNIA-URL': 'http://127.0.0.1:' + srv.port + '/platform',
      INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
      INPUT_REPO: 'git@github.com:org/repo.git',
    });
    assert(!r.stdout.includes(TOKEN), 'token not in stdout');
    assert(!r.stderr.includes(TOKEN), 'token not in stderr');
    pass++; console.log('  token_not_in_output ... ok');
  } finally { await stopServer(srv.server); }
}

async function test_error_body_token_redacted() {
  var srv = await startServer(function (req, res) {
    res.writeHead(400, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'invalid_token', detail: 'token ' + TOKEN + ' rejected' }));
  });
  try {
    var r = await run({
      INPUT_NAMESPACE: 'ns',
      INPUT_TOKEN: TOKEN,
      'INPUT_SKRYNIA-URL': 'http://127.0.0.1:' + srv.port + '/platform',
      INPUT_COMMIT: 'aabbccddeeff0011223344556677889900112233',
      INPUT_REPO: 'git@github.com:org/repo.git',
    });
    assert(r.exit !== 0, 'should fail on 400');
    assert(!r.stdout.includes(TOKEN), 'token not in stdout');
    assert(!r.stderr.includes(TOKEN), 'token not in stderr');
    assert(r.stderr.includes('***'), 'redacted token present');
    pass++; console.log('  error_body_token_redacted ... ok');
  } finally { await stopServer(srv.server); }
}

rmrf(TMP);
fs.mkdirSync(TMP, { recursive: true });

(async function () {
  var tests = [
    test_missing_namespace,
    test_missing_token,
    test_missing_url,
    test_missing_commit_and_repo,
    test_successful_deploy,
    test_root_url_deploy,
    test_default_repo_from_github_repository,
    test_http_error,
    test_builder_param,
    test_subdir_param,
    test_token_not_in_output,
    test_error_body_token_redacted,
  ];

  console.log('Action tests:');
  for (var fn of tests) {
    try {
      await fn();
    } catch (e) {
      fail++;
      console.log('  ' + fn.name.replace('test_', '') + ' ... FAIL');
      console.error(e.stack || e);
    }
  }
  rmrf(TMP);
  console.log('\nResults: ' + pass + ' passed, ' + fail + ' failed, ' + (pass + fail) + ' total');
  process.exit(fail ? 1 : 0);
})().catch(function (e) {
  console.error(e);
  process.exit(1);
});
