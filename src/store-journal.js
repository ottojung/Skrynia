'use strict';

// Durable one-key intents integrate mutations with at-least-once Web Push.
const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const {ensureDir,syncDir,atomicJson}=require('./shared.js');
function createStoreJournal(shared,push,grants) {
  const dir=path.join(shared.STATE_DIR,'store-transactions');
  ensureDir(dir,0o700);
  function finalize(file) {fs.unlinkSync(file);syncDir(dir);}
  function applied(record) {
    const target=shared.objPath(record.key);
    if(record.kind==='delete') return !fs.existsSync(target);
    if(!fs.existsSync(target)) return false;
    return Math.floor(fs.statSync(target).mtimeMs/1000)===record.version;
  }
  function finish(file,record) {
    if(record.committed||applied(record)) push.enqueue(record.key,record.kind);
    finalize(file);
  }
  function commit(key,kind,version,change) {
    push.preflight(key,kind);
    const record={key,kind,version};
    const file=path.join(dir,crypto.randomBytes(16).toString('hex')+'.json');
    atomicJson(file,record);
    let committed=false;
    try {
      change();committed=true;syncDir(shared.STORE_DIR);
    } catch(e) {
      if(!committed&&!applied(record)) finalize(file);
      throw e;
    }
    try {
      record.committed=true;
      atomicJson(file,record);
      finish(file,record);
    } catch(e) {
      console.error('Skrynia: committed value, pending push outbox:',e);
      return {notificationPending:true};
    }
    return {};
  }
  function recover() {
    if(fs.existsSync(shared.STORE_TMP_DIR)) {
      for(const name of fs.readdirSync(shared.STORE_TMP_DIR)) {
        if(/^obj\.[0-9]+\.[a-f0-9]{24}$/.test(name))
          fs.unlinkSync(path.join(shared.STORE_TMP_DIR,name));
      }
      syncDir(shared.STORE_TMP_DIR);
    }
    for(const name of fs.readdirSync(dir)) {
      if(!/^[a-f0-9]{32}\.json$/.test(name)) throw new Error('invalid storage intent filename');
      const file=path.join(dir,name);
      const record=JSON.parse(fs.readFileSync(file,'utf8'));
      if(!grants.validKey(record.key)||!['create','replace','delete'].includes(record.kind)||
        !Number.isSafeInteger(record.version)||record.version<0) {
        throw new Error('invalid store mutation intent');
      }
      finish(file,record);
    }
  }
  return {commit,recover,syncDir};
}
module.exports={createStoreJournal,syncDir};
