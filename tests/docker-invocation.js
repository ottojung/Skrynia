#!/usr/bin/env node
'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const { createManagement } = require('../src/management.js');

const TMP = '/tmp/skrynia-docker-argv-test';
const BIN = path.join(TMP, 'bin');
const FIXTURE = path.join(TMP, 'fixture');
const LOG = path.join(TMP, 'docker.argv');
const COMMIT = 'a'.repeat(40);

function assert(cond, msg) {
  if (!cond) throw new Error('ASSERT: ' + msg);
}

function writeExecutable(name, lines) {
  const file = path.join(BIN, name);
  fs.writeFileSync(file, lines.join('\n') + '\n');
  fs.chmodSync(file, 0o755);
}

function setup() {
  fs.rmSync(TMP, { recursive: true, force: true });
  fs.mkdirSync(BIN, { recursive: true });
  fs.mkdirSync(FIXTURE, { recursive: true });
  fs.writeFileSync(path.join(FIXTURE, 'index.html'), '<h1>test</h1>');
  fs.writeFileSync(path.join(FIXTURE, 'Makefile'), 'build:\n\tmkdir -p build && cp index.html build/index.html\n');

  writeExecutable('git', [
    '#!/bin/sh',
    'set -eu',
    'fixture=' + JSON.stringify(FIXTURE),
    'commit=' + JSON.stringify(COMMIT),
    'if [ "$1" = clone ]; then',
    '  dest=""',
    '  for arg in "$@"; do dest="$arg"; done',
    '  mkdir -p "$dest"',
    '  cp -R "$fixture"/. "$dest"/',
    '  exit 0',
    'fi',
    'if [ "$1" = -C ]; then',
    '  shift 2',
    '  if [ "$1" = checkout ]; then exit 0; fi',
    '  if [ "$1" = rev-parse ] && [ "$2" = HEAD ]; then printf "%s\\n" "$commit"; exit 0; fi',
    'fi',
    'exit 64',
  ]);

  writeExecutable('docker', [
    '#!/bin/sh',
    'set -eu',
    ': > ' + JSON.stringify(LOG),
    'for arg in "$@"; do printf "%s\\n" "$arg" >> ' + JSON.stringify(LOG) + '; done',
    'repo=""',
    'work="/repo"',
    'while [ "$#" -gt 0 ]; do',
    '  case "$1" in',
    '    -v) repo="${2%:/repo}"; shift 2 ;;',
    '    -w) work="$2"; shift 2 ;;',
    '    *) shift ;;',
    '  esac',
    'done',
    '[ -n "$repo" ]',
    'case "$work" in',
    '  /repo|/repo/) app="$repo" ;;',
    '  /repo/*) app="$repo/${work#/repo/}" ;;',
    '  *) exit 65 ;;',
    'esac',
    'mkdir -p "$app/build"',
    'cp "$app/index.html" "$app/build/index.html"',
  ]);
}

function valueAfter(args, flag) {
  const i = args.indexOf(flag);
  assert(i >= 0, 'missing ' + flag);
  assert(i + 1 < args.length, 'missing value after ' + flag);
  return args[i + 1];
}

setup();
const oldPath = process.env.PATH;
process.env.PATH = BIN + ':' + (oldPath || '/usr/bin:/bin');
try {
  const management = createManagement({ dataDir: TMP, builderImage: 'builder:test' });
  const result = management.deploy({
    repo: 'git@test.invalid:repo.git',
    commit: COMMIT,
    subdir: '.',
    namespace: 'argv',
  });
  assert(result.ok === true, 'deploy succeeds');

  const args = fs.readFileSync(LOG, 'utf8').trimEnd().split('\n');
  assert(args[0] === 'run', 'docker run');
  assert(args.includes('--rm'), 'builder is disposable');
  assert(valueAfter(args, '--tmpfs') === '/tmp:size=256m', 'tmpfs configuration');
  assert(valueAfter(args, '--security-opt') === 'no-new-privileges', 'no-new-privileges');
  assert(valueAfter(args, '--cap-drop') === 'ALL', 'all capabilities dropped');
  if (process.getuid && process.getgid) {
    assert(valueAfter(args, '--user') === process.getuid() + ':' + process.getgid(), 'checkout owner uid/gid');
  }
  assert(valueAfter(args, '--env') === 'HOME=/tmp', 'HOME configured');
  const mount = valueAfter(args, '-v');
  assert(mount.endsWith(':/repo'), 'repository bind mount');
  assert(valueAfter(args, '-w') === '/repo/', 'working directory');
  assert(args[args.length - 3] === 'builder:test', 'builder image');
  assert(args[args.length - 2] === 'make' && args[args.length - 1] === 'build', 'make build command');

  // Root-filesystem writability and network access are deliberately not asserted.
  console.log('docker invocation behavior ... ok');
} finally {
  process.env.PATH = oldPath;
  fs.rmSync(TMP, { recursive: true, force: true });
}
