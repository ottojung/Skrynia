#!/usr/bin/env node
'use strict';
// Root revocation must rotate authority, not destroy public objects.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {mkdtempSync,rmSync,readdirSync}=fs;
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {port,request,start,stop,credentials}=require('./crash/fixture');
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
function good(response,status) {
  assert.equal(response.status,status,response.body.toString());
  return JSON.parse(response.body);
}
async function main() {
  const root=mkdtempSync(join(tmpdir(),'skrynia-root-rotate-'));
  let child;
  try {
    const number=await port();
    child=await start(process.execPath,[resolve('src/server.js')],root,number);
    const original=credentials(root);
    const oldUser=good(await request(number,'POST',
      `/grants/${original.id}/users`,original.issuer),200).userToken;
    good(await request(number,'POST','/store/evidence',oldUser,'retain-on-root-rotation'),201);
    await stop(child,'SIGTERM');child=undefined;
    child=await start(binary,[],root,number);
    good(await request(number,'DELETE',`/grants/${original.id}`,original.issuer),200);
    assert.equal((await request(number,'POST','/store/denied',oldUser,'no')).status,403);
    assert.equal((await request(number,'GET','/store/evidence')).body.toString(),
      'retain-on-root-rotation');
    await stop(child,'SIGTERM');child=undefined;
    child=await start(binary,[],root,number);
    const renewed=credentials(root);
    assert.notEqual(renewed.id,original.id);
    assert.notEqual(renewed.issuer,original.issuer);
    assert.deepEqual(readdirSync(join(root,'state/grant-revocations')),[]);
    assert.equal((await request(number,'POST','/store/denied',oldUser,'no')).status,403);
    assert.equal((await request(number,'POST',`/grants/${renewed.id}/users`,original.issuer)).status,403);
    const user=good(await request(number,'POST',
      `/grants/${renewed.id}/users`,renewed.issuer),200).userToken;
    good(await request(number,'POST','/store/after_rotation',user,'new-root-authorized'),201);
    await stop(child,'SIGTERM');child=undefined;
    child=await start(process.execPath,[resolve('src/server.js')],root,number);
    assert.equal((await request(number,'GET','/store/evidence')).body.toString(),
      'retain-on-root-rotation');
    assert.equal((await request(number,'GET','/store/after_rotation')).body.toString(),
      'new-root-authorized');
    assert.equal((await request(number,'POST','/store/old_credential',oldUser,'no')).status,403);
    console.log('Root revocation, credential rotation, preserved public data and Node rollback: OK');
  } finally {
    if(child)await stop(child,'SIGKILL');
    rmSync(root,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
