'use strict';

const fs = require('fs');
const path = require('path');

// --- Namespace validator ---

const NS_RE = /^[a-z0-9][a-z0-9_-]{0,63}$/;
function validNs(ns) {
  return typeof ns === 'string' && NS_RE.test(ns);
}

// --- Directory helpers ---

function ensureDir(dir) { fs.mkdirSync(dir, { recursive: true }); }

// --- Factory: creates shared helpers bound to a data directory ---

function createShared(dataDir) {
  dataDir = dataDir || process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
  const RELEASES_DIR = path.join(dataDir, 'releases');
  const STORAGE_DIR = path.join(dataDir, 'storage');
  const STATE_DIR = path.join(dataDir, 'state');

  const DEFAULT_QUOTA_BYTES = parseInt(process.env.SKRYNIA_DEFAULT_QUOTA_BYTES || '10485760', 10);
  const DEFAULT_MAX_OBJECTS = parseInt(process.env.SKRYNIA_MAX_OBJECT_COUNT || '10000', 10);

  function nsStorageDir(ns) { return path.join(STORAGE_DIR, ns); }
  function nsObjPath(ns, key) { return path.join(nsStorageDir(ns), key + '.dat'); }
  function nsMetaPath(ns, key) { return path.join(nsStorageDir(ns), key + '.meta'); }
  function nsCapPath(ns, key) { return path.join(nsStorageDir(ns), key + '.cap'); }
  function nsQuotaPath(ns) { return path.join(STATE_DIR, ns, 'quota.json'); }
  function nsCurrentLink(ns) { return path.join(RELEASES_DIR, ns, 'current'); }
  function nsConfigPath(ns) { return path.join(STATE_DIR, ns, 'config.json'); }
  function nsStagingDir(ns) { return path.join(RELEASES_DIR, ns, '.staging-' + process.pid); }

  // --- Quota ---

  function loadQuota(ns) {
    const p = nsQuotaPath(ns);
    if (!fs.existsSync(p)) return { bytes: 0, count: 0, quotaBytes: DEFAULT_QUOTA_BYTES, maxObjects: DEFAULT_MAX_OBJECTS };
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  }

  function recalcQuota(ns) {
    const dir = nsStorageDir(ns);
    let bytes = 0, count = 0;
    if (fs.existsSync(dir)) {
      for (const e of fs.readdirSync(dir)) {
        if (e.endsWith('.dat')) { bytes += fs.statSync(path.join(dir, e)).size; count++; }
      }
    }
    const q = loadQuota(ns);
    q.bytes = bytes; q.count = count;
    return q;
  }

  return {
    dataDir,
    RELEASES_DIR,
    STORAGE_DIR,
    STATE_DIR,
    DEFAULT_QUOTA_BYTES,
    DEFAULT_MAX_OBJECTS,
    nsStorageDir,
    nsObjPath,
    nsMetaPath,
    nsCapPath,
    nsQuotaPath,
    nsCurrentLink,
    nsConfigPath,
    nsStagingDir,
    loadQuota,
    recalcQuota,
  };
}

module.exports = {
  NS_RE,
  validNs,
  ensureDir,
  createShared,
};
