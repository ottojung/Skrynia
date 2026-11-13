'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {join}=require('node:path');
const {createHash}=require('node:crypto');
async function hash(file) {
  const digest=createHash('sha256');
  for await(const chunk of fs.createReadStream(file))digest.update(chunk);
  return digest.digest('hex');
}
async function manifest(root) {
  const result=new Map();
  async function walk(dir) {
    for(const entry of fs.readdirSync(join(root,dir),{withFileTypes:true})) {
      const file=join(dir,entry.name);
      if(entry.isDirectory())await walk(file);
      else {
        assert(entry.isFile(),`nonregular item in snapshot: ${file}`);
        result.set(file,await hash(join(root,file)));
      }
    }
  }
  for(const dir of ['store','state/grants','state/push-subs','state/push'])await walk(dir);
  return result;
}
module.exports={hash,manifest};
