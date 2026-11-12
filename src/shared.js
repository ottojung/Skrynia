'use strict';
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

function ensureDir(dir, mode) {
  fs.mkdirSync(dir, {recursive:true,mode:mode || 0o777});
  if (mode != null) fs.chmodSync(dir,mode);
}

function syncDir(dir) {
  const fd=fs.openSync(dir,'r');
  try { fs.fsyncSync(fd); }
  catch(e) { if(e.code!=='EINVAL' && e.code!=='ENOSYS') throw e; }
  finally { fs.closeSync(fd); }
}

// Shared crash-safe replacement of one private JSON record.
function atomicJson(file,value) {
  const dir=path.dirname(file);
  ensureDir(dir,0o700);
  const tmp=file+'.'+crypto.randomBytes(12).toString('hex')+'.tmp';
  try {
    const fd=fs.openSync(tmp,'wx',0o600);
    try {
      fs.writeSync(fd,JSON.stringify(value));
      fs.fsyncSync(fd);
    } finally {fs.closeSync(fd);}
    fs.renameSync(tmp,file);
    syncDir(dir);
  } finally {
    try {fs.unlinkSync(tmp);}
    catch(e) {if(e.code!=='ENOENT') throw e;}
  }
}

function createShared(dataDir) {
  dataDir=dataDir || process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
  const STORE_DIR=path.join(dataDir,'store');
  const STORE_TMP_DIR=path.join(dataDir,'store-tmp');
  const STATE_DIR=path.join(dataDir,'state');
  function objPath(key) {return path.join(STORE_DIR,key);}
  function ensureStoreRoots() {
    ensureDir(STORE_DIR,0o755);
    ensureDir(STORE_TMP_DIR,0o700);
  }
  return {dataDir,STORE_DIR,STORE_TMP_DIR,STATE_DIR,
    objPath,ensureStoreRoots};
}
module.exports={ensureDir,syncDir,atomicJson,createShared};
