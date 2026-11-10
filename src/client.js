/** Browser helper for the global Skrynia object store. */
(function(root) {
  'use strict';
  function detectRoot() {
    if(typeof document!=='undefined'&&document.currentScript&&document.currentScript.src) {
      const url=new URL(document.currentScript.src,root.location&&root.location.href);
      const suffix='/client/skrynia.js';
      if(url.pathname.endsWith(suffix)) {
        url.pathname=url.pathname.slice(0,-suffix.length)||'/';
        url.search='';url.hash='';
        return url.href.replace(/\/+$/,'');
      }
    }
    return '';
  }
  const API_ROOT=detectRoot();
  function request(method,path,body,headers={}) {
    return new Promise((resolve,reject)=>{
      const xhr=new XMLHttpRequest();
      xhr.open(method,API_ROOT+path,true);
      xhr.responseType='arraybuffer';
      for(const [name,value] of Object.entries(headers)) xhr.setRequestHeader(name,value);
      xhr.onload=()=>{
        const data=new Uint8Array(xhr.response||new ArrayBuffer(0));
        const text=()=>new TextDecoder().decode(data);
        const json=()=>JSON.parse(text());
        if(xhr.status<200||xhr.status>=300) {
          const e=new Error(method+' '+path+': HTTP '+xhr.status);
          e.status=xhr.status;
          reject(e);return;
        }
        resolve({bytes:data,text,json,etag:xhr.getResponseHeader('ETag'),status:xhr.status});
      };
      xhr.onerror=()=>reject(new Error('network error'));
      xhr.send(body==null?null:body);
    });
  }
  function bearer(token) {return {'Authorization':'Bearer '+token};}
  function userHeader(token) {return token?bearer(token):{};}
  function Store(token) {this.token=token;}
  Store.prototype.url=function(key) {return '/store/'+encodeURIComponent(key);};
  Store.prototype.get=async function(key) {
    try {const result=await request('GET',this.url(key));return {
      bytes:result.bytes,text:result.text,json:result.json,
      meta:{etag:result.etag}
    };} catch(e){if(e.status===404)return null;throw e;}
  };
  Store.prototype.create=async function(key,value,options) {
    const r=await request('POST',this.url(key),value,
      userHeader(options?.grantUserToken||this.token));return r.json();
  };
  Store.prototype.put=async function(key,value,options) {
    const h=userHeader(options?.grantUserToken||this.token);
    if(options?.etag)h['If-Match']=options.etag;
    const r=await request('PUT',this.url(key),value,h);return r.json();
  };
  Store.prototype.delete=async function(key,options) {
    const h=userHeader(options?.grantUserToken||this.token);
    if(options?.etag)h['If-Match']=options.etag;
    const r=await request('DELETE',this.url(key),null,h);return r.json();
  };
  async function grantRequest(method,path,token,data) {
    const h={...bearer(token)};
    if(data!==undefined) h['Content-Type']='application/json';
    const r=await request(method,path,
      data===undefined?null:JSON.stringify(data),h);return r.json();
  }
  const Skrynia={
    store:userToken=>new Store(userToken),
    listObjects:async (locations,grantUserToken,{limit,after}={})=>{
      const params=new URLSearchParams({locations});
      if(limit!==undefined)params.set('limit',String(limit));
      if(after!==undefined)params.set('after',after);
      const r=await request('GET','/store?'+params.toString(),
        null,userHeader(grantUserToken));
      return r.json();
    },
    createGrant:(parentUserToken,options)=>
      grantRequest('POST','/grants',parentUserToken,options),
    inspectGrant:(id,userToken)=>
      grantRequest('GET','/grants/'+encodeURIComponent(id),userToken),
    issueUser:(id,issuerToken)=>
      grantRequest('POST','/grants/'+encodeURIComponent(id)+'/users',issuerToken),
    revokeUser:(id,userId,issuerToken)=>
      grantRequest('DELETE','/grants/'+encodeURIComponent(id)+'/users/'+encodeURIComponent(userId),issuerToken),
    revokeGrant:(id,issuerToken)=>
      grantRequest('DELETE','/grants/'+encodeURIComponent(id),issuerToken),
    subscribe:async (grantUserToken,locations,events,subscription,capability)=>{
      const headers=userHeader(grantUserToken);
      if(capability)headers['X-Skrynia-Capability']=capability;
      headers['Content-Type']='application/json';
      const r=await request('POST','/push/subscriptions',
        JSON.stringify({locations,events,endpoint:subscription.endpoint,
          keys:subscription.toJSON().keys}),headers);
      return r.json();
    },
  };
  root.Skrynia=Skrynia;
})(typeof window!=='undefined'?window:this);
