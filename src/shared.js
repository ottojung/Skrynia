'use strict';
const fs = require('fs');
const path = require('path');

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

function createShared(dataDir) {
  dataDir=dataDir || process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
  const STORE_DIR=path.join(dataDir,'store');
  const STORE_TMP_DIR=path.join(dataDir,'store-tmp');
  const STATE_DIR=path.join(dataDir,'state');
  const VERSION_PATH=path.join(STATE_DIR,'etag-sequence');
  function objPath(key) { return path.join(STORE_DIR,key); }
  function ensureStoreRoots() {
    ensureDir(STORE_DIR,0o755);
    ensureDir(STORE_TMP_DIR,0o700);
  }
  function nextVersion() {
    let previous=0;
    try { previous=Number(fs.readFileSync(VERSION_PATH,'utf8')); }
    catch(e) { if(e.code!=='ENOENT') throw e; }
    if(!Number.isSafeInteger(previous)||previous<0) throw new Error('invalid ETag sequence');
    const next=Math.max(Math.floor(Date.now()/1000),previous+1);
    if(!Number.isSafeInteger(next)) throw new Error('ETag sequence exhausted');
    ensureDir(STATE_DIR,0o700);
    const tmp=VERSION_PATH+'.'+process.pid+'.tmp';
    const fd=fs.openSync(tmp,'w',0o600);
    try { fs.writeSync(fd,String(next));fs.fsyncSync(fd); }
    finally { fs.closeSync(fd); }
    fs.renameSync(tmp,VERSION_PATH);
    syncDir(STATE_DIR);
    return next;
  }
  return {dataDir,STORE_DIR,STORE_TMP_DIR,STATE_DIR,VERSION_PATH,
    objPath,ensureStoreRoots,nextVersion};
}
module.exports={ensureDir,syncDir,createShared};
