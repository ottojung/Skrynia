#!/usr/bin/env node
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {open,close,expect,auth,grant,tmp}=require('./test-grants.js');

async function run() {
  const dir=tmp();let server;
  try {
    let started=await open(dir);server=started.server;
    let port=started.port;
    const root=server.skrynia.grants.root();
    assert.deepEqual(root.operations,['write','list','delegate','subscribe']);
    const rootToken=(await expect(port,200,'POST','/grants/'+root.id+'/users',
      null,auth(root.rootIssuerToken))).json.userToken;
    const scoped=(await expect(port,200,'POST','/grants',{
      locations:'board.*',operations:['list','delegate'],
      maxBytes:10000,maxObjects:5},auth(rootToken))).json;
    const scopedToken=(await expect(port,200,'POST','/grants/'+scoped.id+'/users',
      null,auth(scoped.issuerToken))).json.userToken;
    const readOnly=(await expect(port,200,'POST','/grants',{
      locations:'board.*',operations:[],maxBytes:5000,maxObjects:3
    },auth(rootToken))).json;
    const plainToken=(await expect(port,200,'POST','/grants/'+readOnly.id+'/users',
      null,auth(readOnly.issuerToken))).json.userToken;
    const writer=(await expect(port,200,'POST','/grants',{
      locations:'board.*',operations:['write'],maxBytes:10000,maxObjects:5
    },auth(rootToken))).json;
    const writerToken=(await expect(port,200,'POST','/grants/'+writer.id+'/users',
      null,auth(writer.issuerToken))).json.userToken;

    await expect(port,201,'POST','/store/board.one','first',grant(writerToken));
    await expect(port,201,'POST','/store/board.two','second',grant(rootToken));
    await expect(port,201,'POST','/store/board.deep.three','third',grant(rootToken));
    await expect(port,201,'POST','/store/other.key','other',grant(rootToken));
    await expect(port,201,'POST','/store/boardz','not-in-prefix',grant(rootToken));

    const listing=(pattern,token,status=200)=>expect(port,status,'GET',
      '/store?locations='+encodeURIComponent(pattern),null,token?grant(token):{});
    assert.deepEqual((await listing('board.*',scopedToken)).json.keys,
      ['board.deep.three','board.one','board.two']);
    assert.deepEqual((await listing('board.one',scopedToken)).json.keys,['board.one']);
    assert.deepEqual((await listing('board.missing',scopedToken)).json.keys,[]);
    assert.deepEqual((await listing('board.deep.*',scopedToken)).json.keys,['board.deep.three']);
    assert.deepEqual((await listing('',scopedToken)).json.keys,[]);
    assert.deepEqual((await listing('*',rootToken)).json.keys,
      ['board.deep.three','board.one','board.two','boardz','other.key']);
    assert.deepEqual((await listing('board.*',rootToken)).json.keys,
      ['board.deep.three','board.one','board.two']);
    // Lists return *all* matching keys regardless of which grant created
    // them, and grants without 'list' cannot enumerate.
    await listing('board.*',plainToken,403);
    await listing('board.*',writerToken,403);
    await listing('board.*',scoped.issuerToken,403);
    await listing('board.*',null,401);
    await listing('other.*',scopedToken,403);
    await listing('*',scopedToken,403);
    await listing('boardz',scopedToken,403);
    await listing('board',scopedToken,403);
    await listing('.hidden*',scopedToken,400);
    await expect(port,400,'GET','/store');
    await expect(port,400,'GET','/store?locations=board.*&locations=other.*',null,grant(scopedToken));
    await expect(port,400,'GET','/store?locations=board.*&junk=1',null,grant(scopedToken));
    await expect(port,405,'POST','/store',null,grant(scopedToken));

    // Delegated list is narrower than parent list; the parent has no
    // implicit write authority, and the child cannot invent operations.
    const child=(await expect(port,200,'POST','/grants',{
      locations:'board.deep.*',operations:['list'],
      maxBytes:4096,maxObjects:1},auth(scopedToken))).json;
    const childToken=(await expect(port,200,'POST','/grants/'+child.id+'/users',
      null,auth(child.issuerToken))).json.userToken;
    assert.deepEqual((await listing('board.deep.*',childToken)).json.keys,
      ['board.deep.three']);
    await listing('board.*',childToken,403);
    await expect(port,403,'POST','/grants',{
      locations:'board.deep.*',operations:['write'],
      maxBytes:4096,maxObjects:1},auth(scopedToken));
    await expect(port,403,'POST','/store/board.four','forbidden',grant(scopedToken));

    await expect(port,200,'PUT','/store/board.one','changed',grant(writerToken));
    assert.deepEqual((await listing('board.*',scopedToken)).json.keys,
      ['board.deep.three','board.one','board.two']);
    await expect(port,200,'DELETE','/store/board.two',null,grant(rootToken));
    assert.deepEqual((await listing('board.*',scopedToken)).json.keys,
      ['board.deep.three','board.one']);
    await expect(port,201,'POST','/store/board.two','again',grant(writerToken));
    assert.deepEqual((await listing('board.*',scopedToken)).json.keys,
      ['board.deep.three','board.one','board.two']);
    // Independent of root/subgrant revocation if the list grant stays live.
    await expect(port,200,'DELETE','/grants/'+writer.id,null,auth(writer.issuerToken));
    assert.deepEqual((await listing('board.*',scopedToken)).json.keys,
      ['board.deep.three','board.one','board.two']);
    await close(server);server=null;
    started=await open(dir);server=started.server;port=started.port;
    assert.deepEqual((await listing('board.*',scopedToken)).json.keys,
      ['board.deep.three','board.one','board.two']);
    console.log('grant-scoped object listing, attenuation, updates, restart: OK');
  } finally {
    if(server)await close(server);
    fs.rmSync(dir,{recursive:true,force:true});
  }
}
run().catch(e=>{console.error(e);process.exitCode=1;});
