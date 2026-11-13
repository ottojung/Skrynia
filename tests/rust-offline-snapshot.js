#!/usr/bin/env node
'use strict';
// Read a coherent offline copy, never launch a server on the supplied path.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {port,start,stop}=require('./crash/fixture');
const {manifest,requireRegularTree}=require('./snapshot/manifest');
const {verify}=require('./snapshot/verify');
const source=resolve(process.argv[2]||'');
const binary=resolve(process.argv[3]||'target/debug/skrynia-server');
async function main() {
  assert(process.argv[2],'Usage: node tests/rust-offline-snapshot.js <offline-data-copy>');
  requireRegularTree(source);
  assert(fs.lstatSync(source).isDirectory(),'snapshot path must be a directory');
  for(const part of ['store','state/grants','state/push-subs','state/push',
    'state/store-transactions','state/grant-revocations','store-tmp']) {
    assert(fs.statSync(join(source,part)).isDirectory(),`missing ${part}`);
  }
  for(const part of ['state/store-transactions','state/grant-revocations','store-tmp']) {
    assert.deepEqual(fs.readdirSync(join(source,part)),[],
      `snapshot has pending recovery in ${part}; take a coherent offline copy`);
  }
  const dir=fs.mkdtempSync(join(tmpdir(),'skrynia-offline-rehearsal-'));
  const clone=join(dir,'data');
  let server;
  try {
    fs.cpSync(source,clone,{recursive:true,preserveTimestamps:true});
    const original=await manifest(clone);
    assert([...original.keys()].some(key=>key.startsWith('state/grants/')),
      'expected at least one grant record');
    const p=await port();
    server=await start(binary,[],clone,p,2400);
    await verify(p,original);
    await stop(server,'SIGTERM');server=undefined;
    assert.deepEqual(await manifest(clone),original,'Rust changed persistent snapshot data');
    server=await start(process.execPath,[resolve('src/server.js')],clone,p,2400);
    await verify(p,original);
    await stop(server,'SIGTERM');server=undefined;
    assert.deepEqual(await manifest(clone),original,'Node changed snapshot after Rust');
    console.log(`Offline snapshot rehearsal: ${original.size} files verified by Rust and Node; source untouched`);
  } finally {
    if(server)await stop(server,'SIGKILL');
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
