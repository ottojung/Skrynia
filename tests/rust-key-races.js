#!/usr/bin/env node
'use strict';
// Competing mutations on one key must be linearizable under real HTTP contention.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {mkdtempSync,rmSync}=fs;
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {port,request,start,stop,credentials,sha}=require('./crash/fixture');
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
const rounds=Number(process.env.SKRYNIA_RACE_ROUNDS||6);
const width=Number(process.env.SKRYNIA_RACE_WIDTH||16);
async function phase(p,user,key,method,expected,values) {
  const requests=values.map(value=>request(p,method,`/store/${key}`,user,
    method==='DELETE'?undefined:value,
    expected?{'X-Skrynia-If-SHA256':expected}:{}));
  return Promise.all(requests);
}
function exactlyOne(outcomes,success,loser,context) {
  const statuses=outcomes.map(response=>response.status);
  assert.equal(statuses.filter(code=>code===success).length,1,
    `${context}: expected one ${success}, got ${statuses}`);
  assert.equal(statuses.filter(code=>code===loser).length,statuses.length-1,
    `${context}: unexpected statuses ${statuses}`);
  return statuses.indexOf(success);
}
async function run(name,command,args) {
  const root=mkdtempSync(join(tmpdir(),`skrynia-cas-race-${name}-`));
  let server;
  try {
    const p=await port();
    server=await start(command,args,root,p);
    const grant=credentials(root);
    const issued=await request(p,'POST',`/grants/${grant.id}/users`,grant.issuer);
    assert.equal(issued.status,200,issued.body.toString());
    const user=JSON.parse(issued.body).userToken;
    for(let round=0;round<rounds;round++) {
      const key=`race_${round}`;
      const creates=Array.from({length:width},(_,i)=>Buffer.from(`create-${round}-${i}`));
      const created=await phase(p,user,key,'POST',null,creates);
      const winner=exactlyOne(created,201,409,`${name} create ${round}`);
      const first=await request(p,'GET',`/store/${key}`);
      assert.equal(first.status,200);
      assert(first.body.equals(creates[winner]),'create winner bytes not durable');
      assert.equal(first.headers.etag,'"'+sha(first.body)+'"');
      const replacements=Array.from({length:width},(_,i)=>Buffer.from(`replace-${round}-${i}`));
      const updated=await phase(p,user,key,'PUT',sha(first.body),replacements);
      const replaceWinner=exactlyOne(updated,200,412,`${name} CAS ${round}`);
      const current=await request(p,'GET',`/store/${key}`);
      assert.equal(current.status,200);
      assert(current.body.equals(replacements[replaceWinner]),'CAS winner bytes not durable');
      assert.equal(current.headers.etag,'"'+sha(current.body)+'"');
      const deletions=await phase(p,user,key,'DELETE',sha(current.body),
        Array.from({length:width},()=>undefined));
      exactlyOne(deletions,200,404,`${name} delete ${round}`);
      assert.equal((await request(p,'GET',`/store/${key}`)).status,404);
    }
    await stop(server,'SIGTERM');server=undefined;
    server=await start(name==='Rust'?process.execPath:binary,
      name==='Rust'?[resolve('src/server.js')]:[],root,p);
    for(let round=0;round<rounds;round++) {
      assert.equal((await request(p,'GET',`/store/race_${round}`)).status,404);
    }
    console.log(`${name}: ${rounds} rounds x ${width} create/CAS/delete contenders, rollback check: OK`);
  } finally {
    if(server)await stop(server,'SIGKILL');
    rmSync(root,{recursive:true,force:true});
  }
}
async function main() {
  assert(Number.isInteger(rounds)&&rounds>=1&&rounds<=40);
  assert(Number.isInteger(width)&&width>=2&&width<=64);
  await run('Node',process.execPath,[resolve('src/server.js')]);
  await run('Rust',binary,[]);
}
main().catch(error=>{console.error(error);process.exitCode=1;});
