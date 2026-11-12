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

function sha256File(file) {
  const fd=fs.openSync(file,'r');
  try {return sha256Fd(fd);}
  finally {fs.closeSync(fd);}
}

module.exports={sha256Bytes,sha256Fd,sha256File};
