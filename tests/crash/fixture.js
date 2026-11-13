'use strict';
const fs=require('node:fs');
const {join}=require('node:path');
const {createHash}=require('node:crypto');
const {spawn}=require('node:child_process');
const net=require('node:net');
const http=require('node:http');
const pause=ms=>new Promise(done=>setTimeout(done,ms));
const sha=buffer=>createHash('sha256').update(buffer).digest('hex');
async function port() {
  const listener=net.createServer();
  await new Promise(done=>listener.listen(0,'127.0.0.1',done));
  const port=listener.address().port;
  await new Promise(done=>listener.close(done));
  return port;
}
function request(port,method,route,token,data,headers={}) {
  return new Promise((resolve,reject)=>{
    const body=data===undefined?null:Buffer.from(data);
    const req=http.request({host:'127.0.0.1',port,method,path:'/crash'+route,agent:false,
      headers:{...headers,...(token?{Authorization:'Bearer '+token}:{}),
        ...(body?{'Content-Length':body.length}:{})}},res=>{
        const chunks=[];
        res.on('data',chunk=>chunks.push(chunk));
        res.on('error',reject);
        res.on('end',()=>resolve({status:res.statusCode,headers:res.headers,body:Buffer.concat(chunks)}));
      });
    req.on('error',reject);
    req.setTimeout(2500,()=>req.destroy(Error('request timed out')));
    req.end(body);
  });
}
async function start(command,args,root,port,tries=100) {
  const child=spawn(command,args,{stdio:'ignore',env:{...process.env,
    SKRYNIA_DATA_DIR:root,SKRYNIA_PORT:String(port),
    SKRYNIA_URL:`http://127.0.0.1:${port}/crash`,SKRYNIA_PUSH_MANUAL:'1'}});
  let dead=false;child.once('exit',()=>{dead=true;});
  for(let attempt=0;attempt<tries;attempt++) {
    if(dead)throw Error('server exited unexpectedly');
    try {if((await request(port,'GET','/health')).status===200)return child;}catch{}
    await pause(25);
  }
  child.kill('SIGKILL');throw Error('server failed to start');
}
async function stop(child,signal) {
  if(child.exitCode!=null || child.signalCode!=null)return;
  child.kill(signal);
  await Promise.race([new Promise(done=>child.once('exit',done)),pause(1200)]);
  if(child.exitCode==null&&child.signalCode==null)child.kill('SIGKILL');
}
function credentials(root) {
  const dir=join(root,'state/grants');
  const files=fs.readdirSync(dir);
  const grant=JSON.parse(fs.readFileSync(join(dir,files[0])));
  return {id:grant.id,issuer:grant.rootIssuerToken};
}
module.exports={port,request,start,stop,credentials,sha,pause};
