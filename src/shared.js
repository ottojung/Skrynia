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

  return {
    dataDir, RELEASES_DIR, STORE_DIR, STORE_META_DIR, STORE_TMP_DIR, STATE_DIR,
    DEFAULT_QUOTA_BYTES, DEFAULT_MAX_OBJECTS,
    nsStoreDir, nsStoreMetaDir, nsObjPath, nsMetaPath, nsQuotaPath, nsCurrentLink, nsConfigPath, nsStagingDir,
    ensureStoreRoots, ensureStoreNamespace, readObjectMeta, writeObjectMeta,
  };
}

module.exports = { NS_RE, validNs, ensureDir, createShared };
