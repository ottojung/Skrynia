#!/usr/bin/env node
'use strict';

// Web Push facility tests: rules, subscriptions, durability, delivery.
// Deterministic and offline: delivery uses an injectable fake transport and
// tests drive server.skrynia.push.pumpOnce() explicitly (pushManual:true).

const http = require('http');
const fs = require('fs');
const path = require('path');
const { createServer } = require('../src/server.js');

const TMP = '/tmp/skrynia-push-test';
const TOKEN = 'push-test-token';

function rmrf(d) { try { fs.rmSync(d, { recursive: true, force: true }); } catch {} }
function assert(cond, msg) { if (!cond) throw new Error('ASSERT: ' + msg); }

function setup() {
  rmrf(TMP);
  for (const name of ['releases', 'storage', 'state', 'builds']) fs.mkdirSync(path.join(TMP, name), { recursive: true });
}

function createNs(ns) {
  fs.mkdirSync(path.join(TMP, 'storage', ns), { recursive: true });
  fs.mkdirSync(path.join(TMP, 'state', ns), { recursive: true });
  fs.writeFileSync(path.join(TMP, 'state', ns, 'quota.json'), JSON.stringify({ quotaBytes: 10485760, maxObjects: 10000 }));
}

function fakeTransport(fn) {
  const calls = [];
  const transport = async (sub, channel) => {
    calls.push({ endpoint: sub.endpoint, channel });
    if (fn) await fn(sub, channel, calls.length);
  };
  transport.calls = calls;
  return transport;
}

function transientError(statusCode) {
  const e = new Error('transient');
  e.statusCode = statusCode;
  return e;
}

function startServer(extra, transport) {
  const server = createServer(Object.assign(
    { dataDir: TMP, token: TOKEN, skryniaUrl: 'https://example.test/platform', pushManual: true, pushTransport: transport || fakeTransport() },
    extra || {}
  ));
  return new Promise(resolve => server.listen(0, '127.0.0.1', () => resolve({ server, port: server.address().port })));
}

function stopServer(server) {
  server.skrynia.push.close();
  return new Promise(resolve => server.close(resolve));
}

function request(port, method, urlPath, data, headers) {
  return new Promise((resolve, reject) => {
    const body = typeof data === 'string' || Buffer.isBuffer(data) ? data : (data == null ? null : JSON.stringify(data));
    const opts = { hostname: '127.0.0.1', port, path: urlPath, method, headers: Object.assign({}, headers || {}), timeout: 10000 };
    if (body != null) {
      opts.headers['Content-Length'] = Buffer.byteLength(body);
      if (!opts.headers['Content-Type']) opts.headers['Content-Type'] = 'application/json';
    }
    const req = http.request(opts, res => {
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => {
        const raw = Buffer.concat(chunks);
        resolve({ status: res.statusCode, body: raw, text: raw.toString(), headers: res.headers });
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('timeout')); });
    if (body != null) req.write(body);
    req.end();
  });
}

function get(port, p) { return request(port, 'GET', p); }
function jsonBody(r) { return JSON.parse(r.text); }

function mgmt(endpoint, params, token) {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(params || {})) if (v != null) q.set(k, String(v));
  q.set('token', token === undefined ? TOKEN : token);
  return '/platform/' + endpoint + '?' + q.toString();
}

const SUB_KEYS = { p256dh: 'BNcRdreALRFXTK0zOz1wGo2gib20', auth: 'tBHItJIx-Xp9x7Vuo' };
function subBody(endpoint) { return { endpoint, keys: SUB_KEYS }; }
function subUrl(ns, channel) { return '/platform/push/subscriptions?namespace=' + encodeURIComponent(ns) + '&channel=' + encodeURIComponent(channel); }

async function register(port, ns, channel, endpoint, capability) {
  const headers = capability ? { 'X-Skrynia-Capability': capability } : undefined;
  const r = await request(port, 'POST', subUrl(ns, channel), subBody(endpoint || ('https://push.example/' + ns + '/' + channel + '/1')), headers);
  assert(r.status === 201, 'register status, got ' + r.status + ': ' + r.text);
  return jsonBody(r);
}

async function test_rule_management() {
  setup(); createNs('n1');
  const { server, port } = await startServer();
  try {
    let r = await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'orders', channels: 'shop,ops', on: 'create,replace' }));
    assert(r.status === 200, 'rule set: ' + r.text);
    assert(jsonBody(r).rule.channels.join(',') === 'shop,ops', 'channels stored');

    r = await get(port, mgmt('push/rules/get', { namespace: 'n1', key: 'orders' }));
    assert(r.status === 200 && jsonBody(r).rule.kinds.join(',') === 'create,replace', 'rule get');

    r = await get(port, mgmt('push/rules/list', { namespace: 'n1' }));
    assert(r.status === 200 && jsonBody(r).rules.length === 1, 'rule list');

    r = await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'orders', channels: 'shop', on: 'delete' }));
    assert(r.status === 200, 'rule replace');
    r = await get(port, mgmt('push/rules/get', { namespace: 'n1', key: 'orders' }));
    assert(jsonBody(r).rule.channels.join(',') === 'shop' && jsonBody(r).rule.kinds.join(',') === 'delete', 'rule replaced');

    r = await get(port, mgmt('push/rules/remove', { namespace: 'n1', key: 'orders' }));
    assert(r.status === 200, 'rule remove');
    r = await get(port, mgmt('push/rules/get', { namespace: 'n1', key: 'orders' }));
    assert(r.status === 404, 'removed rule missing');

    r = await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'Bad Channel!' }));
    assert(r.status === 400, 'bad channel rejected');
    r = await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'ok', on: 'explode' }));
    assert(r.status === 400, 'bad kind rejected');
    r = await get(port, mgmt('push/rules/set', { namespace: 'missing', key: 'k', channels: 'ok' }));
    assert(r.status === 404, 'rule for missing namespace rejected');
    r = await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'ok' }, 'wrong'));
    assert(r.status === 401, 'bad token rejected');
  } finally { await stopServer(server); }
}

async function test_register_dedupe_validation() {
  setup(); createNs('n1');
  const { server, port } = await startServer();
  try {
    const first = await register(port, 'n1', 'news', 'https://push.example/sub-1');
    assert(first.id && first.id.length === 32, 'opaque id');
    assert(first.capability && first.capability.length === 64, 'capability');
    assert(first.deduped === false, 'not deduped');

    let r = await request(port, 'POST', subUrl('n1', 'news'), subBody('https://push.example/sub-1'));
    assert(r.status === 403 && jsonBody(r).error === 'capability_required', 'duplicate registration requires existing capability');
    r = await request(port, 'POST', subUrl('n1', 'news'), subBody('https://push.example/sub-1'), { 'X-Skrynia-Capability': 'wrong' });
    assert(r.status === 403 && jsonBody(r).error === 'invalid_capability', 'duplicate registration rejects wrong capability');
    r = await request(port, 'POST', subUrl('n1', 'news'), subBody('https://push.example/sub-1'), { 'X-Skrynia-Capability': first.capability });
    assert(r.status === 201, 'authorized re-register');
    const second = jsonBody(r);
    assert(second.deduped === true && second.id === first.id, 'same endpoint deduped to same id');
    assert(second.capability !== first.capability, 'authorized re-register rotates capability');

    let capCheck = await request(port, 'PUT', '/platform/push/subscriptions/' + first.id, { keys: SUB_KEYS }, { 'X-Skrynia-Capability': first.capability });
    assert(capCheck.status === 403, 'old capability invalid after authorized rotation');
    capCheck = await request(port, 'PUT', '/platform/push/subscriptions/' + first.id, { keys: SUB_KEYS }, { 'X-Skrynia-Capability': second.capability });
    assert(capCheck.status === 200, 'new capability controls deduped registration');

    const bad1 = await request(port, 'POST', subUrl('n1', 'news'), subBody('http://insecure.example/x'));
    assert(bad1.status === 400, 'non-https endpoint rejected');
    const bad2 = await request(port, 'POST', subUrl('n1', 'news'), { endpoint: 'https://push.example/x', keys: {} });
    assert(bad2.status === 400, 'missing keys rejected');
    const bad3 = await request(port, 'POST', subUrl('n1', 'Bad!'), subBody('https://push.example/x'));
    assert(bad3.status === 400, 'bad channel rejected');
    const bad4 = await request(port, 'POST', subUrl('n1', 'news'), subBody('https://push.example/x'));
    // same-endpoint different-case path check via invalid namespace:
    void bad4;
    const bad5 = await request(port, 'POST', '/platform/push/subscriptions?namespace=Bad&channel=news', subBody('https://push.example/x'));
    assert(bad5.status === 400, 'bad namespace rejected');
    const bad6 = await request(port, 'POST', subUrl('nosuchns', 'news'), subBody('https://push.example/x'));
    assert(bad6.status === 404, 'missing namespace rejected');
  } finally { await stopServer(server); }
}

async function test_no_public_enumeration() {
  setup(); createNs('n1');
  const { server, port } = await startServer();
  try {
    await register(port, 'n1', 'news', 'https://push.example/secret-1');
    const probes = [
      await get(port, '/platform/push/subscriptions'),
      await get(port, '/platform/push/subscriptions?namespace=n1'),
      await get(port, '/platform/push/subs'),
    ];
    for (const r of probes) assert(r.status === 404, 'no public listing, got ' + r.status);
    // Management rule listing exists but requires the management token.
    const gated = await get(port, '/platform/push/rules/list');
    assert(gated.status === 401, 'rule listing needs management token, got ' + gated.status);
    const r = await request(port, 'DELETE', '/platform/push/subscriptions/0123456789abcdef0123456789abcdef', null, { 'X-Skrynia-Capability': 'a'.repeat(64) });
    assert(r.status === 404, 'unknown id is 404, not 403');
  } finally { await stopServer(server); }
}

async function test_capability_update_delete() {
  setup(); createNs('n1');
  const { server, port } = await startServer();
  try {
    const sub = await register(port, 'n1', 'news', 'https://push.example/u-1');
    const url = id => '/platform/push/subscriptions/' + id;
    const cap = c => ({ 'X-Skrynia-Capability': c });

    let r = await request(port, 'PUT', url(sub.id), { keys: SUB_KEYS }, cap('wrong'));
    assert(r.status === 403, 'wrong capability rejected');
    r = await request(port, 'PUT', url(sub.id), { keys: SUB_KEYS });
    assert(r.status === 403, 'missing capability rejected');
    // Capability in the query string is not honored (would leak into logs).
    r = await request(port, 'PUT', url(sub.id) + '?capability=' + sub.capability, { keys: SUB_KEYS });
    assert(r.status === 403, 'query capability ignored');
    const other = await register(port, 'n1', 'news', 'https://push.example/u-2');
    r = await request(port, 'PUT', url(sub.id), { endpoint: 'https://push.example/u-2' }, cap(sub.capability));
    assert(r.status === 409 && jsonBody(r).error === 'endpoint_in_use', 'update cannot create duplicate endpoint in one channel');
    assert(other.id !== sub.id, 'second registration is distinct');

    r = await request(port, 'PUT', url(sub.id), { keys: { p256dh: 'newp', auth: 'newa' } }, cap(sub.capability));
    assert(r.status === 200, 'update with capability');
    const stored = JSON.parse(fs.readFileSync(path.join(TMP, 'state', 'n1', 'push-subs', sub.id + '.json'), 'utf8'));
    assert(stored.keys.p256dh === 'newp', 'keys updated');

    r = await request(port, 'DELETE', url(sub.id), null, cap('wrong'));
    assert(r.status === 403, 'delete with wrong capability rejected');
    r = await request(port, 'DELETE', url(sub.id), null, cap(sub.capability));
    assert(r.status === 200, 'delete with capability');
    assert(!fs.existsSync(path.join(TMP, 'state', 'n1', 'push-subs', sub.id + '.json')), 'record removed');
    r = await request(port, 'DELETE', url(sub.id), null, cap(sub.capability));
    assert(r.status === 404, 'deleted id is 404');
  } finally { await stopServer(server); }
}

async function test_vapid_stable_private_never_exposed() {
  setup(); createNs('n1');
  const t = fakeTransport();
  const first = await startServer({}, t);
  let key1;
  try {
    const r = await get(first.port, '/platform/push/vapid');
    assert(r.status === 200, 'vapid served');
    key1 = jsonBody(r).publicKey;
    assert(typeof key1 === 'string' && key1.length > 0, 'public key present');
    assert(!('privateKey' in jsonBody(r)), 'private key never over HTTP');
    assert(!r.text.includes('privateKey'), 'no private key leakage');
  } finally { await stopServer(first.server); }

  const second = await startServer({}, fakeTransport());
  try {
    const r = await get(second.port, '/platform/push/vapid');
    assert(jsonBody(r).publicKey === key1, 'VAPID key stable across restart');
  } finally { await stopServer(second.server); }

  const stored = JSON.parse(fs.readFileSync(path.join(TMP, 'state', '_push', 'vapid.json'), 'utf8'));
  assert(stored.privateKey && stored.publicKey === key1, 'keypair persisted');
  const mode = fs.statSync(path.join(TMP, 'state', '_push', 'vapid.json')).mode & 0o777;
  assert(mode === 0o600, 'vapid file is 0600, got ' + mode.toString(8));
}

async function test_private_files_are_0600() {
  setup(); createNs('n1');
  const { server, port } = await startServer({}, fakeTransport());
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'c' }));
    const sub = await register(port, 'n1', 'c', 'https://push.example/m-1');
    await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
    const files = [
      path.join(TMP, 'state', '_push', 'vapid.json'),
      path.join(TMP, 'state', 'n1', 'push-rules.json'),
      path.join(TMP, 'state', 'n1', 'push-subs', sub.id + '.json'),
    ].concat(server.skrynia.push.listOutbox().map(i => i.file));
    assert(files.length === 4, 'all private files present, got ' + files.length);
    for (const f of files) {
      const mode = fs.statSync(f).mode & 0o777;
      assert(mode === 0o600, 'private file is 0600: ' + f + ' got ' + mode.toString(8));
    }
  } finally { await stopServer(server); }
}

async function test_no_rule_no_push() {
  setup(); createNs('n1');
  const t = fakeTransport();
  const { server, port } = await startServer({}, t);
  try {
    await register(port, 'n1', 'news', 'https://push.example/s-1');
    const r = await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'create without rule');
    assert(server.skrynia.push.listOutbox().length === 0, 'no outbox without rule');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 0, 'no delivery without rule');
  } finally { await stopServer(server); }
}

async function test_kinds_routing_and_payload_exact() {
  setup(); createNs('n1');
  const t = fakeTransport();
  const { server, port } = await startServer({}, t);
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'doc', channels: 'alpha,beta', on: 'create,delete' }));
    await register(port, 'n1', 'alpha', 'https://push.example/a-1');
    await register(port, 'n1', 'beta', 'https://push.example/b-1');

    let r = await request(port, 'POST', '/platform/store/n1/doc', 'v1', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'create');
    assert(server.skrynia.push.listOutbox().length === 2, 'one outbox item per channel');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 2, 'both channels delivered');
    const payloads = t.calls.map(c => c.channel).sort();
    assert(payloads.join(',') === 'alpha,beta', 'payloads are exactly channel names');
    for (const c of t.calls) assert(typeof c.channel === 'string' && c.channel.length > 0 && !c.channel.includes('n1') && !c.channel.includes('doc'), 'no ns/key data in payload');
    assert(server.skrynia.push.listOutbox().length === 0, 'outbox drained');

    t.calls.length = 0;
    r = await request(port, 'PUT', '/platform/store/n1/doc', 'v2');
    assert(r.status === 200, 'replace not covered by rule');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 0, 'replace kind excluded');

    r = await request(port, 'DELETE', '/platform/store/n1/doc');
    assert(r.status === 200, 'delete');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 2, 'delete delivered to both channels');
  } finally { await stopServer(server); }
}

async function test_enqueue_failure_blocks_commit() {
  setup(); createNs('n1');
  const { server, port } = await startServer({}, fakeTransport());
  const push = server.skrynia.push;
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'c' }));
    const orig = push.enqueue;
    push.enqueue = () => { throw new Error('disk full'); };
    try {
      let r = await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
      assert(r.status === 500 && jsonBody(r).error === 'push_enqueue_failed', 'create blocked without durable outbox');
      r = await get(port, '/platform/store/n1/k');
      assert(r.status === 404, 'failed create left no visible object');

      push.enqueue = orig;
      r = await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
      assert(r.status === 201, 'create works again');

      push.enqueue = () => { throw new Error('disk full'); };
      r = await request(port, 'PUT', '/platform/store/n1/k', 'v2');
      assert(r.status === 500, 'replace blocked without durable outbox');
      r = await get(port, '/platform/store/n1/k');
      assert(r.text === 'v', 'failed replace left old value');

      r = await request(port, 'DELETE', '/platform/store/n1/k');
      assert(r.status === 500, 'delete blocked without durable outbox');
      r = await get(port, '/platform/store/n1/k');
      assert(r.status === 200, 'failed delete left object');
    } finally {
      push.enqueue = orig;
    }
  } finally { await stopServer(server); }
}

async function test_crash_restart_recovery() {
  setup(); createNs('n1');
  const failing = fakeTransport(async () => { throw transientError(500); });
  const first = await startServer({}, failing);
  try {
    await get(first.port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'c' }));
    await register(first.port, 'n1', 'c', 'https://push.example/r-1');
    const r = await request(first.port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'mutation acked with durable outbox');
    // Delivery fails transiently while the mutation is already committed.
    await first.server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(failing.calls.length === 1, 'attempt recorded');
    assert(first.server.skrynia.push.listOutbox().length === 1, 'entry survives failed delivery');
    const stored = JSON.parse(fs.readFileSync(path.join(TMP, 'state', 'n1', 'push-subs', fs.readdirSync(path.join(TMP, 'state', 'n1', 'push-subs'))[0]), 'utf8'));
    assert(stored.id, 'subscription intact');
  } finally { await stopServer(first.server); }

  // "Restart": new process on the same data dir recovers pending outbox state.
  const delivered = fakeTransport();
  const second = await startServer({}, delivered);
  try {
    await second.server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(delivered.calls.length === 1, 'pending notification redelivered after restart');
    assert(delivered.calls[0].channel === 'c', 'payload is the channel');
    assert(second.server.skrynia.push.listOutbox().length === 0, 'outbox drained after recovery');
  } finally { await stopServer(second.server); }
}

async function test_spurious_wakeup_safe_and_idempotent() {
  setup(); createNs('n1');
  const t = fakeTransport();
  const { server, port } = await startServer({}, t);
  try {
    await register(port, 'n1', 'c', 'https://push.example/sp-1');
    // Simulate a crash between outbox persistence and object commit: an
    // outbox item exists for an object that was never committed.
    server.skrynia.push.setRule('n1', 'ghost', ['create'], ['c']);
    const fs2 = require('fs');
    const outboxDir = server.skrynia.push.outboxDir;
    const entry = { id: 'ghost-1', ns: 'n1', key: 'ghost', kind: 'create', channel: 'c', attempts: 0, nextAt: new Date(0).toISOString(), created: new Date(0).toISOString() };
    fs2.writeFileSync(path.join(outboxDir, 'ghost-1.json'), JSON.stringify(entry));
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 1 && t.calls[0].channel === 'c', 'spurious wake-up delivers safely');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 1, 'reprocessing is safe (no duplicates)');
  } finally { await stopServer(server); }
}

async function test_transient_retry_backoff_bounded() {
  setup(); createNs('n1');
  let failures = 1;
  const t = fakeTransport(async () => {
    if (failures > 0) { failures--; throw transientError(503); }
  });
  const { server, port } = await startServer({}, t);
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'c' }));
    await register(port, 'n1', 'c', 'https://push.example/t-1');
    const r = await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'create');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 1, 'first attempt');
    let items = server.skrynia.push.listOutbox();
    assert(items.length === 1 && items[0].entry.attempts === 1, 'entry kept with attempts=1');
    assert(Date.parse(items[0].entry.nextAt) > Date.now(), 'backoff defers next attempt');

    const due = await server.skrynia.push.pumpOnce();
    assert(due.delivered === 0 && t.calls.length === 1, 'backoff respected (no early retry)');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 2, 'retried');
    assert(server.skrynia.push.listOutbox().length === 0, 'delivered entry removed');
  } finally { await stopServer(server); }
}

async function test_stuck_delivery_times_out_and_isolates() {
  setup(); createNs('n1');
  let first = true;
  const t = fakeTransport(async () => {
    if (first) {
      first = false;
      await new Promise(() => {});
    }
  });
  const { server, port } = await startServer({ pushSendTimeoutMs: 20 }, t);
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'c' }));
    await register(port, 'n1', 'c', 'https://push.example/timeout-1');
    await register(port, 'n1', 'c', 'https://push.example/timeout-2');
    const r = await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'create');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 2, 'stuck first send does not prevent later subscription attempt');
    const items = server.skrynia.push.listOutbox();
    assert(items.length === 1 && items[0].entry.attempts === 1, 'timeout is transient and retained for retry');
  } finally { await stopServer(server); }
}

async function test_expired_old_endpoint_does_not_delete_update() {
  setup(); createNs('n1');
  let releaseOld;
  let startedOld;
  const oldStarted = new Promise(resolve => { startedOld = resolve; });
  const oldGate = new Promise(resolve => { releaseOld = resolve; });
  const t = fakeTransport(async sub => {
    if (sub.endpoint.endsWith('/old')) {
      startedOld();
      await oldGate;
      throw transientError(410);
    }
  });
  const { server, port } = await startServer({ pushSendTimeoutMs: 1000 }, t);
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'c' }));
    const sub = await register(port, 'n1', 'c', 'https://push.example/old');
    const r = await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'create');

    const pumping = server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    await oldStarted;
    const update = await request(
      port,
      'PUT',
      '/platform/push/subscriptions/' + sub.id,
      { endpoint: 'https://push.example/new' },
      { 'X-Skrynia-Capability': sub.capability }
    );
    assert(update.status === 200, 'registration updated while old endpoint send is in flight');
    releaseOld();
    await pumping;

    const storedPath = path.join(TMP, 'state', 'n1', 'push-subs', sub.id + '.json');
    assert(fs.existsSync(storedPath), '410 from old endpoint does not delete newer registration');
    const stored = JSON.parse(fs.readFileSync(storedPath, 'utf8'));
    assert(stored.endpoint === 'https://push.example/new', 'new endpoint survives old 410');
  } finally { await stopServer(server); }
}

async function test_expired_removed_and_isolation() {
  setup(); createNs('n1');
  const t = fakeTransport(async sub => {
    if (sub.endpoint.endsWith('/dead')) throw transientError(410);
  });
  const { server, port } = await startServer({}, t);
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'c' }));
    const dead = await register(port, 'n1', 'c', 'https://push.example/dead');
    await register(port, 'n1', 'c', 'https://push.example/healthy');
    const r = await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'create');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    const healthy = t.calls.filter(c => c.endpoint.endsWith('/healthy'));
    assert(healthy.length === 1, 'healthy subscription still notified');
    assert(!fs.existsSync(path.join(TMP, 'state', 'n1', 'push-subs', dead.id + '.json')), '410 subscription removed');
    assert(server.skrynia.push.listOutbox().length === 0, 'outbox drained');
  } finally { await stopServer(server); }
}

async function test_broken_subscription_isolation() {
  setup(); createNs('n1');
  const t = fakeTransport(async sub => {
    if (sub.endpoint.endsWith('/broken')) throw transientError(500);
  });
  const { server, port } = await startServer({}, t);
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'c' }));
    await register(port, 'n1', 'c', 'https://push.example/broken');
    await register(port, 'n1', 'c', 'https://push.example/fine');
    const r = await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'create');
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.filter(c => c.endpoint.endsWith('/fine')).length === 1, 'healthy notified despite broken sibling');
    const items = server.skrynia.push.listOutbox();
    assert(items.length === 1 && items[0].entry.attempts === 1, 'entry retained for retry');
    assert(fs.readdirSync(path.join(TMP, 'state', 'n1', 'push-subs')).length === 2, 'broken subscription kept for retry (not 404/410)');
  } finally { await stopServer(server); }
}

async function test_namespace_subscription_cap() {
  setup(); createNs('n1');
  const { server, port } = await startServer({ pushMaxSubsPerNamespace: 3 }, fakeTransport());
  try {
    // Arbitrary channel names cannot bypass the per-namespace total.
    const first = await register(port, 'n1', 'chan-a', 'https://push.example/cap-1');
    await register(port, 'n1', 'chan-b', 'https://push.example/cap-2');
    await register(port, 'n1', 'chan-c', 'https://push.example/cap-3');
    const r = await request(port, 'POST', subUrl('n1', 'chan-d'), subBody('https://push.example/cap-4'));
    assert(r.status === 507 && jsonBody(r).error === 'namespace_full', 'namespace cap enforced, got ' + r.status + ': ' + r.text);
    // Dedupe of an existing record still works at cap (rotates capability).
    const d = await request(port, 'POST', subUrl('n1', 'chan-a'), subBody('https://push.example/cap-1'), { 'X-Skrynia-Capability': first.capability });
    assert(d.status === 201 && jsonBody(d).deduped === true, 'authorized dedupe works at cap');
  } finally { await stopServer(server); }
}

async function test_retry_indefinite_no_drop() {
  setup(); createNs('n1');
  let failures = 15;
  const t = fakeTransport(async () => {
    if (failures > 0) { failures--; throw transientError(503); }
  });
  const { server, port } = await startServer({}, t);
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'k', channels: 'c' }));
    await register(port, 'n1', 'c', 'https://push.example/ld-1');
    const r = await request(port, 'POST', '/platform/store/n1/k', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'create');
    for (let i = 0; i < 15; i++) {
      await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
      const items = server.skrynia.push.listOutbox();
      assert(items.length === 1, 'entry survives attempt ' + (i + 1) + ' (no max-attempt drop)');
    }
    const kept = server.skrynia.push.listOutbox()[0].entry;
    assert(kept.attempts === 15, 'attempts keep counting, got ' + kept.attempts);
    await server.skrynia.push.pumpOnce({ ignoreBackoff: true });
    assert(t.calls.length === 16, 'delivered after 15 transient failures');
    assert(server.skrynia.push.listOutbox().length === 0, 'entry drained on success');
  } finally { await stopServer(server); }
}

async function test_outbox_full_blocks_mutation() {
  setup(); createNs('n1');
  const { server, port } = await startServer({ pushMaxOutboxItems: 2 }, fakeTransport());
  try {
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'a', channels: 'c1,c2' }));
    await get(port, mgmt('push/rules/set', { namespace: 'n1', key: 'b', channels: 'c1' }));
    let r = await request(port, 'POST', '/platform/store/n1/a', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 201, 'first mutation fills the outbox');
    assert(server.skrynia.push.listOutbox().length === 2, 'outbox at capacity');
    r = await request(port, 'POST', '/platform/store/n1/b', 'v', { 'X-Skrynia-Mode': 'public-write' });
    assert(r.status === 507 && jsonBody(r).error === 'push_outbox_full', 'outbox-full backpressure, got ' + r.status + ': ' + r.text);
    r = await get(port, '/platform/store/n1/b');
    assert(r.status === 404, 'blocked mutation left no visible object');
    assert(server.skrynia.push.listOutbox().length === 2, 'no partial entries added');
  } finally { await stopServer(server); }
}

const tests = [
  ['rule_management', test_rule_management],
  ['register_dedupe_validation', test_register_dedupe_validation],
  ['no_public_enumeration', test_no_public_enumeration],
  ['capability_update_delete', test_capability_update_delete],
  ['vapid_stable_private_never_exposed', test_vapid_stable_private_never_exposed],
  ['private_files_are_0600', test_private_files_are_0600],
  ['no_rule_no_push', test_no_rule_no_push],
  ['kinds_routing_and_payload_exact', test_kinds_routing_and_payload_exact],
  ['enqueue_failure_blocks_commit', test_enqueue_failure_blocks_commit],
  ['crash_restart_recovery', test_crash_restart_recovery],
  ['spurious_wakeup_safe_and_idempotent', test_spurious_wakeup_safe_and_idempotent],
  ['transient_retry_backoff_bounded', test_transient_retry_backoff_bounded],
  ['retry_indefinite_no_drop', test_retry_indefinite_no_drop],
  ['outbox_full_blocks_mutation', test_outbox_full_blocks_mutation],
  ['namespace_subscription_cap', test_namespace_subscription_cap],
  ['stuck_delivery_times_out_and_isolates', test_stuck_delivery_times_out_and_isolates],
  ['expired_old_endpoint_does_not_delete_update', test_expired_old_endpoint_does_not_delete_update],
  ['expired_removed_and_isolation', test_expired_removed_and_isolation],
  ['broken_subscription_isolation', test_broken_subscription_isolation],
];

(async () => {
  let pass = 0;
  let fail = 0;
  for (const [name, fn] of tests) {
    process.stdout.write('  push ' + name + ' ... ');
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
  console.log('\nPush results: ' + pass + ' passed, ' + fail + ' failed, ' + (pass + fail) + ' total');
  process.exit(fail ? 1 : 0);
})().catch(e => {
  console.error(e && e.stack ? e.stack : e);
  process.exit(1);
});
