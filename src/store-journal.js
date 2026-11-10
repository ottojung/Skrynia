'use strict';

// Crash-recoverable intent for one-key mutations and their Web Push outbox.
// Mutations are atomic per object file. Push delivery is at-least-once;
// replay can enqueue duplicates after a crash between enqueue and finalization.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const {ensureDir,validNs}=require('./shared.js');

function syncDir(dir) {
  const fd=fs.openSync(dir,'r');
  try { fs.fsyncSync(fd); }
  catch(e){if(e.code!=='EINVAL' && e.code!=='ENOSYS') throw e;}
  finally {fs.closeSync(fd);}
}
function durableJson(file, value) {
  ensureDir(path.dirname(file),0o700);
  const tmp=file+'.tmp.'+crypto.randomBytes(6).toString('hex');
  try {
    const fd=fs.openSync(tmp,'wx',0o600);
    try {fs.writeSync(fd,JSON.stringify(value));fs.fsyncSync(fd);}
    finally {fs.closeSync(fd);}
    fs.renameSync(tmp,file);
    syncDir(path.dirname(file));
  } finally {
    try{fs.unlinkSync(tmp);}catch(e){if(e.code!=='ENOENT')throw e;}
  }
}
function createStoreJournal(shared,push,grants) {
  const pendingDir=path.join(shared.STATE_DIR,'_store-transactions');
  ensureDir(pendingDir,0o700);
  function intentFile() {
    return path.join(pendingDir,crypto.randomBytes(16).toString('hex')+'.json');
  }
  function finalize(file) {
    fs.unlinkSync(file);
    syncDir(pendingDir);
  }
  function applied(record) {
    const target=shared.nsObjPath(record.ns,record.key);
    if(record.kind==='delete') return !fs.existsSync(target);
    if(!fs.existsSync(target)) return false;
    const stat=fs.statSync(target);
    return Math.floor(stat.mtimeMs/1000)===record.version;
  }
  function finish(file,record) {
    // A committed notification remains owed even if a later mutation has
    // changed the same key before this intent is replayed.
    if(record.committed || applied(record)) {
      push.enqueue(record.ns,record.key,record.kind);
    }
    finalize(file);
  }
  function commit(ns,key,kind,version,change) {
    push.preflight(ns,key,kind);
    const record={ns,key,kind,version};
    const file=intentFile();
    durableJson(file,record);
    let committed=false;
    try {
      change();
      committed=true;
      syncDir(shared.nsStoreDir(ns));
    } catch(e) {
      if(!committed) {
        // If filesystem mutation succeeded but directory fsync failed, we
        // cannot safely call this aborted: retain journal for recovery.
        if(!applied(record)) finalize(file);
      }
      throw e;
    }
    try {
      record.committed=true;
      durableJson(file,record);
      finish(file,record);
    } catch(e) {
      // The value is already committed. Leave the journal for replay
      // instead of misleading callers that their mutation was rolled back.
      console.error('Skrynia: committed value, pending push outbox:',e);
      return {notificationPending:true};
    }
    return {};
  }
  function recover() {
    // The single owning process may discard abandoned staging files left
    // behind by a crash. Committed objects are already in the store dir.
    if(fs.existsSync(shared.STORE_TMP_DIR)) {
      for(const name of fs.readdirSync(shared.STORE_TMP_DIR)) {
        if(/^[a-z0-9_-]+\.[0-9]+\.[a-f0-9]{24}$/.test(name)) {
          fs.unlinkSync(path.join(shared.STORE_TMP_DIR,name));
        }
      }
      syncDir(shared.STORE_TMP_DIR);
    }
    for(const name of fs.readdirSync(pendingDir).filter(n=>/^[a-f0-9]{32}\.json$/.test(n))) {
      const file=path.join(pendingDir,name);
      const record=JSON.parse(fs.readFileSync(file,'utf8'));
      if(!validNs(record.ns)||!grants.validKey(record.key)||
         !['create','replace','delete'].includes(record.kind)||
         !Number.isSafeInteger(record.version)||record.version<0) {
        throw new Error('invalid store mutation intent');
      }
      finish(file,record);
    }
  }
  function forgetNamespace(ns) {
    for(const name of fs.readdirSync(pendingDir).filter(x=>/^[a-f0-9]{32}\.json$/.test(x))) {
      const file=path.join(pendingDir,name);
      const record=JSON.parse(fs.readFileSync(file,'utf8'));
      if(record.ns===ns) fs.unlinkSync(file);
    }
    syncDir(pendingDir);
  }
  return {commit,recover,forgetNamespace,syncDir};
}
module.exports={createStoreJournal,syncDir};
