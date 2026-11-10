'use strict';

// Ephemeral, authoritative-in-process index. The filesystem is the durable
// source of truth and is scanned once per namespace at startup/first access.
// All storage mutations must call replace() after their filesystem commit.
const fs = require('fs');
const path = require('path');
const {validNs} = require('./shared.js');

function createResourceIndex(shared, validKey, matches) {
  const loaded = new Map();

  function state(ns) {
    if (!validNs(ns)) throw new Error('invalid_namespace');
    if (loaded.has(ns)) return loaded.get(ns);
    const entries = new Map(), patterns = new Map(), dir = shared.nsStoreDir(ns);
    if (fs.existsSync(dir)) {
      for (const ent of fs.readdirSync(dir, {withFileTypes:true})) {
        if (!ent.isFile() || !validKey(ent.name)) {
          throw new Error('invalid stored object key: ' + ns);
        }
        entries.set(ent.name, fs.statSync(path.join(dir, ent.name)).size);
      }
    }
    const result = {entries, patterns};
    loaded.set(ns, result);
    return result;
  }

  function value(ns, key) {
    const m = state(ns).entries;
    return m.has(key) ? m.get(key) : null;
  }

  function stats(ns, locations) {
    const s=state(ns);
    if (!s.patterns.has(locations)) {
      let bytes=0, objects=0;
      for (const [key, size] of s.entries) {
        if (!matches(locations,key)) continue;
        bytes+=size;
        objects++;
      }
      s.patterns.set(locations,{bytes,objects});
    }
    const x=s.patterns.get(locations);
    return {bytes:x.bytes,objects:x.objects};
  }

  function replace(ns, key, size) {
    const s=state(ns);
    const old=s.entries.has(key) ? s.entries.get(key) : null;
    if (old===size) return;
    if (size===null) s.entries.delete(key);
    else {
      if (!Number.isSafeInteger(size) || size<0) throw new Error('invalid indexed object size');
      s.entries.set(key,size);
    }
    const delta=(size===null?0:size)-(old===null?0:old);
    const count=(size===null?0:1)-(old===null?0:1);
    for (const [pattern, totals] of s.patterns) {
      if (!matches(pattern,key)) continue;
      totals.bytes+=delta;
      totals.objects+=count;
      if (totals.bytes<0 || totals.objects<0) throw new Error('corrupt object index');
    }
  }

  function entries(ns) {
    return [...state(ns).entries].map(([key,bytes])=>({key,bytes}));
  }
  function forget(ns) { loaded.delete(ns); }
  return {value,stats,replace,entries,forget};
}
module.exports = {createResourceIndex};
