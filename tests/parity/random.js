'use strict';
// Reproducible differential state-machine exercise, with no fuzzy oracle.
const assert=require('node:assert/strict');
const {createHash}=require('node:crypto');
const digest=text=>createHash('sha256').update(text).digest('hex');
function seeded(seed) {
  let state=seed>>>0;
  return ()=>{
    state^=state<<13;state^=state>>>17;state^=state<<5;
    return state>>>0;
  };
}
module.exports=async function run(compare,count=200) {
  const next=seeded(0x7259ac31);
  const known=new Map();
  for(let step=0;step<count;step++) {
    const id=next()%23;
    const key=`fuzz_${id}`;
    const choice=next()%10;
    const body=`value_${step}_${(next()%100000).toString(16)}`;
    const previous=known.get(key);
    const bad=step%17===0;
    let method,token=bad?'bad':'user',headers={},payload,compareBody=false;
    if(choice<=1) {method='GET';token='none';compareBody=true;}
    else if(choice===2) {method='HEAD';token='none';compareBody=true;}
    else if(choice===3) {
      method='GET';token='none';compareBody=true;
      headers.Range=next()%2?'bytes=0-0':'bytes=-1';
    }
    else if(choice===4) {method='DELETE';}
    else if(choice===5) {method='POST';payload=body;}
    else if(choice===6) {method='PUT';payload=body;}
    else if(choice===7) {
      method='PUT';payload=body;
      headers['X-Skrynia-If-SHA256']=previous?digest(previous):'f'.repeat(64);
    }
    else if(choice===8) {
      method='DELETE';
      headers['X-Skrynia-If-SHA256']=previous?digest(previous):'f'.repeat(64);
    }
    else {
      method='PUT';payload=body;
      headers['X-Skrynia-If-SHA256']='a'.repeat(64);
    }
    const [node,rust]=await compare(`seeded ${step} ${method} ${key}`,method,
      `/store/${key}`,{token,headers,body:payload,compareBody});
    if(node.status!==rust.status)continue; // compare() collects mismatch detail
    if(!bad && node.status>=200 && node.status<300) {
      if(method==='DELETE')known.delete(key);
      else if(method==='PUT'||method==='POST')known.set(key,body);
    }
    if(step%11===0) {
      const candidate=`fuzz_${next()%23}`;
      const [a,b]=await compare(`seeded ${step} consistency`, 'GET',
        `/store/${candidate}`,{token:'none',compareBody:true});
      const expected=known.get(candidate);
      if(expected===undefined)assert.equal(a.status,404);
      else {
        assert.equal(a.status,200);
        assert.equal(a.body,expected);
      }
      assert.equal(b.status,a.status);
    }
  }
};
