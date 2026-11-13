'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {join,resolve}=require('node:path');
const {port,request,start,stop}=require('./crash/fixture');
const root=process.env.SKRYNIA_CONTAINER_EXPECTED_ROOT;
async function main() {
  assert.deepEqual(fs.readdirSync(join(root,'store-tmp')),[]);
  assert.deepEqual(fs.readdirSync(join(root,'state/store-transactions')),[]);
  const number=await port();
  const server=await start(process.execPath,[resolve('src/server.js')],root,number);
  try {
    for(const name of fs.readdirSync(join(root,'store'))) {
      const match=/^pressure_([0-9]+)$/.exec(name);
      assert(match,`unexpected stored filename: ${name}`);
      const index=Number(match[1]);
      const length=Number(process.env.SKRYNIA_CONTAINER_PAYLOAD_BYTES||4194304);
      const expected=Buffer.alloc(length,65+index%25);
      assert(fs.readFileSync(join(root,'store',name)).equals(expected),name);
      const actual=await request(number,'GET','/store/'+name);
      assert.equal(actual.status,200,name);
      assert(actual.body.equals(expected),name);
    }
  } finally {await stop(server,'SIGTERM');}
}
main().catch(error=>{console.error(error);process.exitCode=1;});
