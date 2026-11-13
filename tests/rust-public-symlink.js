#!/usr/bin/env node
'use strict';
// A new public-store symlink must never expose private server state.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {join,resolve}=require('node:path');
const {tmpdir}=require('node:os');
const {port,request,start,stop,credentials,sha}=require('./crash/fixture');
const binary=resolve(process.argv[2]||'target/debug/skrynia-server');
async function main() {
  const root=fs.mkdtempSync(join(tmpdir(),'skrynia-public-link-'));
  let child;
  try {
    const p=await port();child=await start(binary,[],root,p);
    const grant=credentials(root);
    const privateFile=join(root,'state/grants',grant.id+'.json');
    const publicLink=join(root,'store','exfiltration');
    fs.symlinkSync(privateFile,publicLink);
    const response=await request(p,'GET','/store/exfiltration');
    assert.equal(response.status,500,response.body.toString());
    assert(!response.body.includes(Buffer.from(grant.issuer)),
      'private grant issuer leaked through public symlink');
    assert.equal((await request(p,'HEAD','/store/exfiltration')).status,500);
    assert.equal((await request(p,'GET','/health')).status,200);
    fs.unlinkSync(publicLink);
    const user=JSON.parse((await request(p,'POST',
      `/grants/${grant.id}/users`,grant.issuer)).body).userToken;
    assert.equal((await request(p,'POST','/store/exfiltration',user,'safe')).status,201);
    assert.equal((await request(p,'GET','/store/exfiltration')).body.toString(),'safe');
    // Replacing an existing object with a symlink must not bypass CAS either.
    fs.unlinkSync(publicLink);
    fs.symlinkSync(privateFile,publicLink);
    const cas=await request(p,'PUT','/store/exfiltration',user,'replacement',
      {'X-Skrynia-If-SHA256':sha('safe')});
    assert.equal(cas.status,500,cas.body.toString());
    assert(!cas.body.includes(Buffer.from(grant.issuer)));
    assert.equal((await request(p,'GET','/health')).status,200);
    fs.unlinkSync(publicLink);
    fs.writeFileSync(publicLink,'safe');
    assert.equal((await request(p,'PUT','/store/exfiltration',user,'replacement',
      {'X-Skrynia-If-SHA256':sha('safe')})).status,200);
    assert.equal((await request(p,'GET','/store/exfiltration')).body.toString(),
      'replacement');
    console.log('Public-store GET/HEAD and CAS reject injected symlink; private state protected: OK');
  } finally {
    if(child)await stop(child,'SIGKILL');
    fs.rmSync(root,{recursive:true,force:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
