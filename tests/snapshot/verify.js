'use strict';
const assert=require('node:assert/strict');
const http=require('node:http');
async function requestDigest(port,key) {
  return new Promise((resolve,reject)=>{
    const sha=require('node:crypto').createHash('sha256');
    const req=http.get({host:'127.0.0.1',port,path:'/crash/store/'+encodeURIComponent(key)},res=>{
      const code=res.statusCode;
      res.on('data',chunk=>sha.update(chunk));
      res.on('error',reject);
      res.on('end',()=>resolve({code,etag:res.headers.etag,hash:sha.digest('hex')}));
    });
    req.on('error',reject);
    req.setTimeout(30000,()=>req.destroy(Error('snapshot HTTP GET timed out')));
  });
}
async function verify(port,files) {
  for(const [file,digest] of files) {
    if(!file.startsWith('store/'))continue;
    const response=await requestDigest(port,file.slice('store/'.length));
    assert.equal(response.code,200,file);
    assert.equal(response.hash,digest,file);
    assert.equal(response.etag,'"'+digest+'"',file);
  }
}
module.exports={verify};
