#!/usr/bin/env node
'use strict';
// Integration proof of concept: multi-object logical commit via one-key CAS.
// The server knows nothing about account balances or manifests.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const http = require('node:http');
const {createServer} = require('../src/server.js');

function request(port, method, key, data, userToken, etag) {
  const bytes = data === undefined ? null : Buffer.from(JSON.stringify(data));
  const headers = {};
  if (userToken) headers['X-Skrynia-Grant'] = userToken;
  if (etag) headers['If-Match'] = etag;
  if (bytes) headers['Content-Length'] = String(bytes.length);
  return new Promise((resolve, reject) => {
    const req = http.request({hostname:'127.0.0.1', port,
      path:'/platform/store/atomic/' + key, method, headers}, res => {
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => {
        const body = Buffer.concat(chunks).toString('utf8');
        resolve({status:res.statusCode, etag:res.headers.etag,
          value:res.statusCode===200 ? JSON.parse(body) : null, body});
      });
    });
    req.on('error', reject);
    if (bytes) req.write(bytes);
    req.end();
  });
}
async function expect(port, status, method, key, data, token, etag) {
  const response = await request(port, method, key, data, token, etag);
  assert.equal(response.status,status,
    method + ' ' + key + ' got HTTP ' + response.status + ': ' + response.body);
  return response;
}
async function run() {
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),'skrynia-client-protocol-'));
  const server=createServer({dataDir:dir,token:'management-token',
    skryniaUrl:'http://127.0.0.1:17880/platform',pushManual:true});
  try {
    await new Promise(ok => server.listen(0,'127.0.0.1',ok));
    const port=server.address().port;
    const root=server.skrynia.grants.createRoot('atomic',100000,100);
    const token=server.skrynia.grants.createUser('atomic',root.id,
      root.issuerToken).userToken;
    const get=key=>expect(port,200,'GET',key);

    // These are three distinct physical objects per snapshot: two values
    // and a manifest. Only the head is mutable in this client protocol.
    async function stage(label,alice,bob) {
      const a='acct_a_'+label,b='acct_b_'+label,m='manifest_'+label;
      await expect(port,201,'POST',a,{balance:alice},token);
      await expect(port,201,'POST',b,{balance:bob},token);
      await expect(port,201,'POST',m,{alice:a,bob:b},token);
      return m;
    }
    async function snapshot(head) {
      const manifest=(await get(head.manifest)).value;
      const a=(await get(manifest.alice)).value.balance;
      const b=(await get(manifest.bob)).value.balance;
      return {alice:a,bob:b};
    }
    const initial=await stage('initial',10,10);
    await expect(port,201,'POST','db_head',
      {revision:0,manifest:initial},token);
    const observed=await get('db_head');
    assert.deepEqual(await snapshot(observed.value),{alice:10,bob:10});

    // Two writers independently read the same version and prepare candidates.
    const first=await stage('first',8,12);
    const second=await stage('second',7,13);
    assert.deepEqual((await get('db_head')).value,observed.value,
      'staging cannot publish a partial transaction');
    await expect(port,200,'PUT','db_head',
      {revision:1,manifest:first},token,observed.etag);
    await expect(port,412,'PUT','db_head',
      {revision:1,manifest:second},token,observed.etag);

    const committed=await get('db_head');
    assert.deepEqual(await snapshot(committed.value),{alice:8,bob:12},
      'only the CAS winner is visible');
    assert.deepEqual(await snapshot(observed.value),{alice:10,bob:10},
      'old readers retain a coherent immutable snapshot');
    assert.notEqual(committed.etag,observed.etag);

    // The losing writer must recompute against the new head.
    const retried=await stage('retry',5,15);
    await expect(port,200,'PUT','db_head',
      {revision:2,manifest:retried},token,committed.etag);
    assert.deepEqual(await snapshot((await get('db_head')).value),
      {alice:5,bob:15});
    assert.deepEqual(await snapshot(committed.value),{alice:8,bob:12});
    assert.equal((await get('manifest_second')).value.alice,'acct_a_second',
      'failed candidates can exist without becoming committed');

    console.log('client-side atomic multi-object snapshots via CAS: OK');
  } finally {
    if (server.listening) await new Promise(ok => server.close(ok));
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
