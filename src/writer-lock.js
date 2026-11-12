'use strict';
const fs=require('node:fs');
const path=require('node:path');
const {spawnSync}=require('node:child_process');
const {ensureDir}=require('./shared.js');

// Keep the open file description in Node; a short-lived flock process acquires
// the kernel lock through an inherited descriptor. The kernel releases it on
// process exit, including crashes, without stale-PID cleanup.
function acquireWriterLock(stateDir) {
  ensureDir(stateDir,0o700);
  const file=path.join(stateDir,'writer.lock');
  const fd=fs.openSync(file,fs.constants.O_RDWR|fs.constants.O_CREAT,0o600);
  try {
    const result=spawnSync('flock',
      ['--nonblock','--conflict-exit-code','75','3'],
      {stdio:['ignore','ignore','pipe',fd],timeout:3000});
    if(result.error) {
      throw new Error('Skrynia requires util-linux flock: '+result.error.message);
    }
    if(result.status===75) {
      throw new Error('Skrynia data directory is held by another writer: '+stateDir);
    }
    if(result.status!==0) {
      throw new Error('Skrynia could not lock '+stateDir+': '+
        (result.stderr?.toString().trim()||'flock exited '+result.status));
    }
  } catch(e) {
    fs.closeSync(fd);
    throw e;
  }
  let released=false;
  return ()=>{
    if(!released) {released=true;fs.closeSync(fd);}
  };
}
module.exports={acquireWriterLock};
