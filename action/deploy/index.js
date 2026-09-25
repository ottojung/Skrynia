'use strict';

const fs = require('fs');
const http = require('http');
const https = require('https');

const DEFAULT_TIMEOUT_SECONDS = 1800;
const MAX_TIMEOUT_SECONDS = 7200;

function input(name) {
  const val = process.env['INPUT_' + name.toUpperCase().replace(/ /g, '_')] || '';
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

function parseTimeoutSeconds() {
  const raw = input('timeout-seconds') || String(DEFAULT_TIMEOUT_SECONDS);
  if (!/^[0-9]+$/.test(raw)) fail('inputs.timeout-seconds must be an integer');
  const value = Number(raw);
  if (!Number.isSafeInteger(value) || value < 1 || value > MAX_TIMEOUT_SECONDS) {
    fail('inputs.timeout-seconds must be between 1 and ' + MAX_TIMEOUT_SECONDS);
  }
  return value;
}

function requestText(url, timeoutSeconds) {
  return new Promise(function (resolve, reject) {
    const transport = url.protocol === 'https:' ? https : url.protocol === 'http:' ? http : null;
    if (!transport) {
      reject(new Error('Skrynia URL must use http or https'));
      return;
    }

    let settled = false;
    let timer;
    const req = transport.get(url, { headers: { Accept: 'application/json' } }, function (res) {
      const chunks = [];
      res.setEncoding('utf8');
      res.on('data', function (chunk) { chunks.push(chunk); });
      res.on('end', function () {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        resolve({ status: res.statusCode || 0, body: chunks.join('') });
      });
    });

    timer = setTimeout(function () {
      req.destroy(new Error('deployment timed out after ' + timeoutSeconds + ' seconds'));
    }, timeoutSeconds * 1000);

    req.on('error', function (err) {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      reject(err);
    });
  });
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
  url.pathname = basePath + '/deploy';
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
  const timeoutSeconds = parseTimeoutSeconds();

  const params = { repo, commit, subdir, namespace, token };
  if (builder) params.builder = builder;

  const url = buildUrl(skryniaUrl, params);

  console.log('Deploying ' + repo + '@' + commit.substring(0, 8) + ' (subdir: ' + subdir + ') to ' + skryniaUrl + ' namespace ' + namespace);

  let response;
  try {
    response = await requestText(url, timeoutSeconds);
  } catch (err) {
    fail('Network error connecting to Skrynia: ' + redact(err.message, token));
  }

  const body = response.body;

  if (response.status < 200 || response.status >= 300) {
    console.error(redact('HTTP ' + response.status + ': ' + body, token));
    fail('Deploy failed with HTTP ' + response.status);
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
