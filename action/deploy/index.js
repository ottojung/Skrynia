'use strict';

const fs = require('fs');

function input(name) {
  const val = process.env['INPUT_' + name.toUpperCase().replace(/-/g, '_')] || '';
  return val.trim();
}

function redact(text, token) {
  if (!token || !text) return text;
  var safe = text.split(token).join('***');
  try {
    safe = safe.split(encodeURIComponent(token)).join('***');
  } catch {}
  return safe;
}

function fail(message) {
  console.error('::error::' + message);
  process.exit(1);
}

function setOutput(name, value) {
  const file = process.env.GITHUB_OUTPUT;
  if (file && value) {
    fs.appendFileSync(file, name + '=' + value + '\n');
  }
}

function deriveRepo() {
  const explicit = input('repo');
  if (explicit) return explicit;

  const ghRepo = process.env.GITHUB_REPOSITORY;
  if (!ghRepo) {
    fail('inputs.repo is required when GITHUB_REPOSITORY is not available');
  }
  return 'git@github.com:' + ghRepo + '.git';
}

function deriveCommit() {
  const explicit = input('commit');
  if (explicit) return explicit;

  const sha = process.env.GITHUB_SHA;
  if (!sha) {
    fail('inputs.commit is required when GITHUB_SHA is not available');
  }
  return sha;
}

function buildUrl(baseUrl, params) {
  const url = new URL(baseUrl);
  const basePath = url.pathname.replace(/\/+$/, '');
  url.pathname = basePath.endsWith('/_skrynia')
    ? basePath + '/deploy'
    : basePath + '/_skrynia/deploy';
  url.search = '';
  url.hash = '';
  for (const [key, value] of Object.entries(params)) {
    if (value) url.searchParams.set(key, value);
  }
  return url;
}

async function deploy() {
  const namespace = input('namespace');
  const token = input('token');
  const skryniaUrl = input('skrynia-url');

  if (!namespace) fail('inputs.namespace is required');
  if (!token) fail('inputs.token is required');
  if (!skryniaUrl) fail('inputs.skrynia-url is required');

  const repo = deriveRepo();
  const commit = deriveCommit();
  const subdir = input('subdir') || '.';
  const builder = input('builder');

  const params = { repo, commit, subdir, namespace, token };
  if (builder) params.builder = builder;

  const url = buildUrl(skryniaUrl, params);

  console.log('Deploying ' + repo + '@' + commit.substring(0, 8) + ' (subdir: ' + subdir + ') to ' + skryniaUrl + ' namespace ' + namespace);

  let res;
  try {
    res = await fetch(url.href);
  } catch (err) {
    fail('Network error connecting to Skrynia: ' + redact(err.message, token));
  }

  const body = await res.text();

  if (!res.ok) {
    console.error(redact('HTTP ' + res.status + ': ' + body, token));
    fail('Deploy failed with HTTP ' + res.status);
  }

  let data;
  try {
    data = JSON.parse(body);
  } catch {
    fail('Invalid JSON response from Skrynia');
  }

  if (!data.ok) {
    fail('Deploy returned ok=false: ' + redact(JSON.stringify(data), token));
  }

  setOutput('release', data.release);
  setOutput('path', data.path);

  if (data.release) console.log('Release: ' + data.release);
  if (data.path) console.log('Path: ' + data.path);
  console.log('Deploy succeeded.');
}

deploy().catch(function (err) {
  fail(err.message || String(err));
});
