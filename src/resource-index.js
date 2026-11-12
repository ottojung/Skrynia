'use strict';

// Memory-only index rebuilt from public object files. Failed loads are never
// cached; sorted key snapshots are rebuilt lazily after creates/deletes.
const fs=require('fs');
const path=require('path');

function createResourceIndex(shared,validKey,matches) {
  let entries=null,sortedKeys=null;
  const patterns=new Map();
  const MAX_PATTERN_STATS=1024;

  function state() {
    if(entries) return entries;
    const loaded=new Map();
    if(fs.existsSync(shared.STORE_DIR)) {
      for(const ent of fs.readdirSync(shared.STORE_DIR,{withFileTypes:true})) {
        if(!ent.isFile()||!validKey(ent.name))
          throw new Error('invalid stored object key: '+ent.name);
        loaded.set(ent.name,fs.statSync(path.join(shared.STORE_DIR,ent.name)).size);
      }
    }
    // Assign only after the entire scan succeeds: partial quotas are unsafe.
    entries=loaded;
    return entries;
  }

  function value(key) {
    const s=state();
    return s.has(key)?s.get(key):null;
  }

  function stats(locations) {
    const s=state();
    let result=patterns.get(locations);
    if(result) patterns.delete(locations);
    else {
      let bytes=0,objects=0;
      for(const [key,size] of s) {
        if(matches(locations,key)) {bytes+=size;objects++;}
      }
      result={bytes,objects};
    }
    // LRU cap prevents the stats cache from retaining every formerly
    // delegated pattern forever. A missed pattern is rebuilt from state.
    patterns.set(locations,result);
    if(patterns.size>MAX_PATTERN_STATS) {
      patterns.delete(patterns.keys().next().value);
    }
    return {...result};
  }

  function replace(key,size) {
    const s=state(),old=s.has(key)?s.get(key):null;
    if(old===size) return;
    if(size===null) s.delete(key);
    else {
      if(!Number.isSafeInteger(size)||size<0)
        throw new Error('invalid indexed size');
      s.set(key,size);
    }
    if((old===null)!==(size===null)) sortedKeys=null;
    const delta=(size===null?0:size)-(old===null?0:old);
    const count=Number(size!==null)-Number(old!==null);
    for(const [pattern,totals] of patterns) {
      if(!matches(pattern,key)) continue;
      totals.bytes+=delta;
      totals.objects+=count;
      if(totals.bytes<0||totals.objects<0) throw new Error('corrupt object index');
    }
  }

  function sorted() {
    if(sortedKeys===null) sortedKeys=[...state().keys()].sort();
    return sortedKeys;
  }

  function lowerBound(items,value,strict=false) {
    let lo=0,hi=items.length;
    while(lo<hi) {
      const mid=(lo+hi)>>>1;
      if(items[mid]<value||strict&&items[mid]===value) lo=mid+1;
      else hi=mid;
    }
    return lo;
  }

  function listKeys(locations,{after=null,limit=null}={}) {
    if(locations==='') return {keys:[]};
    const items=sorted();
    const wildcard=locations.endsWith('*');
    const prefix=wildcard?locations.slice(0,-1):locations;
    let index=lowerBound(items,prefix);
    if(after!==null) index=Math.max(index,lowerBound(items,after,true));
    const keys=[];
    while(index<items.length) {
      const key=items[index++];
      if(wildcard?!key.startsWith(prefix):key!==locations) break;
      if(limit!==null&&keys.length===limit) {
        return {keys,next:keys[keys.length-1]};
      }
      keys.push(key);
    }
    return {keys};
  }

  function list() {
    return [...state()].map(([key,bytes])=>({key,bytes}));
  }
  return {value,stats,replace,entries:list,listKeys,
    clearStats:()=>patterns.clear(),statsCacheSize:()=>patterns.size};
}
module.exports={createResourceIndex};
