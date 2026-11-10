'use strict';

// Memory-only index; reconstruct from object files at first use.
const fs=require('fs');
const path=require('path');
function createResourceIndex(shared,validKey,matches) {
  let entries=null;
  const patterns=new Map();
  function state() {
    if(entries) return entries;
    entries=new Map();
    if(fs.existsSync(shared.STORE_DIR)) {
      for(const ent of fs.readdirSync(shared.STORE_DIR,{withFileTypes:true})) {
        if(!ent.isFile()||!validKey(ent.name)) {
          throw new Error('invalid stored object key: '+ent.name);
        }
        entries.set(ent.name,fs.statSync(path.join(shared.STORE_DIR,ent.name)).size);
      }
    }
    return entries;
  }
  function value(key) { const s=state();return s.has(key)?s.get(key):null; }
  function stats(locations) {
    const s=state();
    if(!patterns.has(locations)) {
      let bytes=0,objects=0;
      for(const [key,size] of s) if(matches(locations,key)) {bytes+=size;objects++;}
      patterns.set(locations,{bytes,objects});
    }
    return {...patterns.get(locations)};
  }
  function replace(key,size) {
    const s=state();
    const old=s.has(key)?s.get(key):null;
    if(old===size) return;
    if(size===null) s.delete(key);
    else {
      if(!Number.isSafeInteger(size)||size<0) throw new Error('invalid indexed size');
      s.set(key,size);
    }
    const delta=(size===null?0:size)-(old===null?0:old);
    const count=(size===null?0:1)-(old===null?0:1);
    for(const [pattern,totals] of patterns) if(matches(pattern,key)) {
      totals.bytes+=delta;totals.objects+=count;
      if(totals.bytes<0||totals.objects<0) throw new Error('corrupt object index');
    }
  }
  function list() {return [...state()].map(([key,bytes])=>({key,bytes}));}
  return {value,stats,replace,entries:list};
}
module.exports={createResourceIndex};
