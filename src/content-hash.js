'use strict';
const fs=require('fs');
const crypto=require('crypto');

function sha256Bytes(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function sha256Fd(fd) {
  const hasher=crypto.createHash('sha256');
  const block=Buffer.allocUnsafe(65536);
  let position=0;
  while(true) {
    const count=fs.readSync(fd,block,0,block.length,position);
    if(count===0) break;
    hasher.update(block.subarray(0,count));
    position+=count;
  }
  return hasher.digest('hex');
}

// Hash public reads asynchronously so large anonymous GETs cannot monopolize
// the event loop. The descriptor pins the same inode for hashing and serving.
function sha256FdAsync(fd,cancelled=()=>false) {
  const hash=crypto.createHash('sha256');
  const block=Buffer.allocUnsafe(65536);
  let position=0;
  return new Promise((resolve,reject)=>{
    function next() {
      if(cancelled()) return resolve(null);
      fs.read(fd,block,0,block.length,position,(e,count)=>{
        if(e) return reject(e);
        if(cancelled()) return resolve(null);
        if(count===0) return resolve(hash.digest('hex'));
        hash.update(block.subarray(0,count));
        position+=count;
        next();
      });
    }
    next();
  });
}

function sha256File(file) {
  const fd=fs.openSync(file,'r');
  try {return sha256Fd(fd);}
  finally {fs.closeSync(fd);}
}

module.exports={sha256Bytes,sha256Fd,sha256FdAsync,sha256File};
