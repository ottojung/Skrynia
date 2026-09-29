'use strict';

const fs = require('fs');
const path = require('path');

const NS_RE = /^[a-z0-9][a-z0-9_-]{0,63}$/;
function validNs(ns) {
  return typeof ns === 'string' && NS_RE.test(ns);
}

function ensureDir(dir, mode) {
  fs.mkdirSync(dir, { recursive: true, mode: mode || 0o777 });
  if (mode != null) fs.chmodSync(dir, mode);
}

let atomicSequence = 0;
function atomicWrite(filePath, data, mode) {
  ensureDir(path.dirname(filePath), 0o700);
  const tmp = filePath + '.tmp.' + process.pid + '.' + (++atomicSequence);
  fs.writeFileSync(tmp, data, { mode: mode == null ? 0o600 : mode });
  fs.renameSync(tmp, filePath);
}

function createShared(dataDir) {
  dataDir = dataDir || process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
  const RELEASES_DIR = path.join(dataDir, 'releases');
  const STORE_DIR = path.join(dataDir, 'store');
  const STORE_META_DIR = path.join(dataDir, 'store-meta');
  const STORE_TMP_DIR = path.join(dataDir, 'store-tmp');
  const LEGACY_STORAGE_DIR = path.join(dataDir, 'storage');
  const STATE_DIR = path.join(dataDir, 'state');

  const DEFAULT_QUOTA_BYTES = parseInt(process.env.SKRYNIA_DEFAULT_QUOTA_BYTES || '10485760', 10);
  const DEFAULT_MAX_OBJECTS = parseInt(process.env.SKRYNIA_MAX_OBJECT_COUNT || '10000', 10);

  function nsStoreDir(ns) { return path.join(STORE_DIR, ns); }
  function nsStoreMetaDir(ns) { return path.join(STORE_META_DIR, ns); }
  function nsObjPath(ns, key) { return path.join(nsStoreDir(ns), key); }
  function nsMetaPath(ns, key) { return path.join(nsStoreMetaDir(ns), key + '.json'); }
  function nsQuotaPath(ns) { return path.join(STATE_DIR, ns, 'quota.json'); }
  function nsCurrentLink(ns) { return path.join(RELEASES_DIR, ns, 'current'); }
  function nsConfigPath(ns) { return path.join(STATE_DIR, ns, 'config.json'); }
  function nsStagingDir(ns) { return path.join(RELEASES_DIR, ns, '.staging-' + process.pid); }

  function ensureStoreRoots() {
    ensureDir(STORE_DIR, 0o755);
    ensureDir(STORE_META_DIR, 0o700);
    ensureDir(STORE_TMP_DIR, 0o700);
  }

  function ensureStoreNamespace(ns) {
    ensureStoreRoots();
    ensureDir(nsStoreDir(ns), 0o755);
    ensureDir(nsStoreMetaDir(ns), 0o700);
  }

  function readObjectMeta(ns, key) {
    const p = nsMetaPath(ns, key);
    if (!fs.existsSync(p)) return null;
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  }

  function writeObjectMeta(ns, key, meta) {
    ensureStoreNamespace(ns);
    atomicWrite(nsMetaPath(ns, key), JSON.stringify(meta, null, 2), 0o600);
  }

  function validStoredVersion(value) {
    return Number.isSafeInteger(value) && value >= 0 ? value : 0;
  }

  function migrateLegacyStorage() {
    ensureStoreRoots();
    if (!fs.existsSync(LEGACY_STORAGE_DIR)) return;

    for (const ns of fs.readdirSync(LEGACY_STORAGE_DIR)) {
      if (!validNs(ns)) continue;
      const legacyDir = path.join(LEGACY_STORAGE_DIR, ns);
      let dirStat;
      try { dirStat = fs.statSync(legacyDir); } catch { continue; }
      if (!dirStat.isDirectory()) continue;

      ensureStoreNamespace(ns);
      const entries = fs.readdirSync(legacyDir);

      for (const name of entries.filter(x => x.endsWith('.meta'))) {
        const key = name.slice(0, -5);
        const oldMetaPath = path.join(legacyDir, name);
        const oldDataPath = path.join(legacyDir, key + '.dat');
        const oldCapPath = path.join(legacyDir, key + '.cap');
        const oldVersionPath = path.join(legacyDir, key + '.ver');
        const newDataPath = nsObjPath(ns, key);
        const newMetaPath = nsMetaPath(ns, key);

        if (!fs.existsSync(oldDataPath)) {
          if (fs.existsSync(newDataPath) && fs.existsSync(newMetaPath)) continue;
          throw new Error('legacy committed object missing data: ' + ns + '/' + key);
        }
        if (fs.existsSync(newDataPath)) throw new Error('legacy migration target already exists: ' + ns + '/' + key);

        const oldMeta = JSON.parse(fs.readFileSync(oldMetaPath, 'utf8'));
        const dataVersion = Math.floor(fs.statSync(oldDataPath).mtimeMs / 1000);
        let storedVersion = 0;
        if (fs.existsSync(oldVersionPath)) storedVersion = Number(fs.readFileSync(oldVersionPath, 'utf8').trim());

        const mode = ['immutable', 'capability-write', 'public-write'].includes(oldMeta.mode) ? oldMeta.mode : 'public-write';
        const meta = { mode, version: Math.max(dataVersion, validStoredVersion(storedVersion)) };
        if (mode === 'capability-write') {
          if (!fs.existsSync(oldCapPath)) throw new Error('legacy capability object missing verifier: ' + ns + '/' + key);
          meta.capHash = fs.readFileSync(oldCapPath, 'utf8').trim();
        }

        atomicWrite(newMetaPath, JSON.stringify(meta, null, 2), 0o600);
        fs.chmodSync(oldDataPath, 0o644);
        fs.renameSync(oldDataPath, newDataPath);
      }

      for (const name of entries.filter(x => x.endsWith('.ver'))) {
        const key = name.slice(0, -4);
        if (fs.existsSync(nsObjPath(ns, key))) continue;
        const version = validStoredVersion(Number(fs.readFileSync(path.join(legacyDir, name), 'utf8').trim()));
        if (!version) continue;
        const existing = readObjectMeta(ns, key) || {};
        if (validStoredVersion(existing.version) < version) writeObjectMeta(ns, key, { version });
      }

      fs.rmSync(legacyDir, { recursive: true, force: true });
    }

    try {
      if (fs.readdirSync(LEGACY_STORAGE_DIR).length === 0) fs.rmdirSync(LEGACY_STORAGE_DIR);
    } catch {}
  }

  function loadQuota(ns) {
    const p = nsQuotaPath(ns);
    if (!fs.existsSync(p)) return { quotaBytes: DEFAULT_QUOTA_BYTES, maxObjects: DEFAULT_MAX_OBJECTS };
    const q = JSON.parse(fs.readFileSync(p, 'utf8'));
    let dirty = false;
    if ('bytes' in q) { delete q.bytes; dirty = true; }
    if ('count' in q) { delete q.count; dirty = true; }
    if (dirty) fs.writeFileSync(p, JSON.stringify(q, null, 2));
    return q;
  }

  function recalcQuota(ns) {
    const dir = nsStoreDir(ns);
    let bytes = 0, count = 0;
    if (fs.existsSync(dir)) {
      for (const e of fs.readdirSync(dir)) {
        const p = path.join(dir, e);
        let st;
        try { st = fs.statSync(p); } catch { continue; }
        if (!st.isFile()) continue;
        bytes += st.size;
        count++;
      }
    }
    const q = loadQuota(ns);
    q.bytes = bytes; q.count = count;
    return q;
  }

  return {
    dataDir, RELEASES_DIR, STORE_DIR, STORE_META_DIR, STORE_TMP_DIR, LEGACY_STORAGE_DIR, STATE_DIR,
    DEFAULT_QUOTA_BYTES, DEFAULT_MAX_OBJECTS,
    nsStoreDir, nsStoreMetaDir, nsObjPath, nsMetaPath, nsQuotaPath, nsCurrentLink, nsConfigPath, nsStagingDir,
    ensureStoreRoots, ensureStoreNamespace, readObjectMeta, writeObjectMeta, migrateLegacyStorage,
    loadQuota, recalcQuota,
  };
}

module.exports = { NS_RE, validNs, ensureDir, createShared };
