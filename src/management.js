'use strict';

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { execFileSync } = require('child_process');
const { normalizeBasePath } = require('./base-path.js');

class ManagementError extends Error {
  constructor(status, code, detail) {
    super(detail || code);
    this.status = status;
    this.code = code;
    this.detail = detail || code;
  }
}

function createManagement(opts) {
  opts = opts || {};

  const DATA_DIR = opts.dataDir || process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
  const RELEASES_DIR = path.join(DATA_DIR, 'releases');
  const STORAGE_DIR = path.join(DATA_DIR, 'storage');
  const STATE_DIR = path.join(DATA_DIR, 'state');
  const BUILDS_DIR = path.join(DATA_DIR, 'builds');
  const BUILDER_IMAGE = opts.builderImage || process.env.SKRYNIA_BUILDER_IMAGE || 'skrynia-builder:0.1.0';
  const APP_BASE_PATH = normalizeBasePath(opts.appBasePath || process.env.SKRYNIA_APP_BASE_PATH || '/apps');
  const APP_DIR = opts.appDir || process.env.SKRYNIA_APP_DIR || '';
  const DEFAULT_QUOTA_BYTES = parseInt(process.env.SKRYNIA_DEFAULT_QUOTA_BYTES || '10485760', 10);
  const MAX_OBJECT_COUNT = parseInt(process.env.SKRYNIA_MAX_OBJECT_COUNT || '10000', 10);

  const NS_RE = /^[a-z0-9][a-z0-9_-]{0,63}$/;
  const HEX40 = /^[0-9a-f]{40}$/;
  const HEX64 = /^[0-9a-f]{64}$/;

  function ensureDir(dir) { fs.mkdirSync(dir, { recursive: true }); }
  function rmrfDir(dir) { if (fs.existsSync(dir)) fs.rmSync(dir, { recursive: true, force: true }); }
  function fail(status, code, detail) { throw new ManagementError(status, code, detail); }
  function validNs(ns) { return typeof ns === 'string' && NS_RE.test(ns); }
  function validCommit(commit) { return typeof commit === 'string' && (HEX40.test(commit) || HEX64.test(commit)); }
  function configPath(ns) { return path.join(STATE_DIR, ns, 'config.json'); }
  function quotaPath(ns) { return path.join(STATE_DIR, ns, 'quota.json'); }
  function currentLink(ns) { return path.join(RELEASES_DIR, ns, 'current'); }
  function activeLink(ns) { return APP_DIR ? path.join(APP_DIR, ns) : currentLink(ns); }

  function loadConfig(ns) {
    const p = configPath(ns);
    return fs.existsSync(p) ? JSON.parse(fs.readFileSync(p, 'utf8')) : null;
  }

  function saveConfig(ns, cfg) {
    ensureDir(path.dirname(configPath(ns)));
    fs.writeFileSync(configPath(ns), JSON.stringify(cfg, null, 2));
  }

  function loadQuota(ns) {
    const p = quotaPath(ns);
    if (!fs.existsSync(p)) return null;
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  }

  function recalcQuota(ns) {
    const q = loadQuota(ns);
    if (!q) fail(404, 'namespace_not_found', 'namespace ' + ns + ' not found');

    const dir = path.join(STORAGE_DIR, ns);
    let bytes = 0;
    let count = 0;
    if (fs.existsSync(dir)) {
      for (const entry of fs.readdirSync(dir)) {
        if (!entry.endsWith('.meta')) continue;
        const data = path.join(dir, entry.slice(0, -5) + '.dat');
        if (!fs.existsSync(data)) continue;
        bytes += fs.statSync(data).size;
        count++;
      }
    }
    q.bytes = bytes;
    q.count = count;
    fs.writeFileSync(quotaPath(ns), JSON.stringify(q, null, 2));
    return q;
  }

  function ensureNamespace(ns, quotaBytes) {
    ensureDir(path.join(STORAGE_DIR, ns));
    ensureDir(path.join(STATE_DIR, ns));
    const p = quotaPath(ns);
    if (fs.existsSync(p)) return { created: false, quota: JSON.parse(fs.readFileSync(p, 'utf8')) };
    const q = {
      bytes: 0,
      count: 0,
      quotaBytes: quotaBytes || DEFAULT_QUOTA_BYTES,
      maxObjects: MAX_OBJECT_COUNT,
    };
    fs.writeFileSync(p, JSON.stringify(q, null, 2));
    return { created: true, quota: q };
  }

  function listReleases(ns) {
    const base = path.join(RELEASES_DIR, ns);
    if (!fs.existsSync(base)) return [];
    return fs.readdirSync(base)
      .filter(name => {
        if (name.startsWith('.') || name === 'current') return false;
        try { return fs.statSync(path.join(base, name)).isDirectory(); }
        catch { return false; }
      })
      .sort();
  }

  function activateRelease(ns, releaseDir) {
    if (APP_DIR) ensureDir(APP_DIR);
    else ensureDir(path.join(RELEASES_DIR, ns));
    const link = activeLink(ns);
    const tmp = link + '.tmp.' + process.pid;
    try { fs.unlinkSync(tmp); } catch {}
    fs.symlinkSync(releaseDir, tmp);
    fs.renameSync(tmp, link);
  }

  function deactivateRelease(ns) {
    try { fs.unlinkSync(activeLink(ns)); } catch (e) {
      if (e.code !== 'ENOENT') throw e;
    }
  }

  function currentReleaseId(ns) {
    try { return path.basename(fs.readlinkSync(activeLink(ns))); }
    catch { return null; }
  }

  function validateNamespace(ns) {
    if (!ns) fail(400, 'missing_namespace', 'namespace is required');
    if (!validNs(ns)) fail(400, 'invalid_namespace', 'namespace must match ' + NS_RE);
  }

  function validateSubdir(subdir) {
    if (!subdir) fail(400, 'missing_subdir', 'subdir is required');
    if (path.isAbsolute(subdir)) fail(400, 'invalid_subdir', 'subdir must be relative');
    if (subdir.includes('..')) fail(400, 'invalid_subdir', 'subdir must not contain ..');
    if (subdir.includes('\0')) fail(400, 'invalid_subdir', 'subdir must not contain null bytes');
  }

  function validateBuildOutput(buildDir) {
    const issues = [];
    function walk(dir, rel) {
      for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        const item = rel ? rel + '/' + entry.name : entry.name;
        if (entry.isSymbolicLink()) issues.push('symlink not allowed: ' + item);
        else if (entry.isBlockDevice() || entry.isCharacterDevice() || entry.isFIFO() || entry.isSocket()) issues.push('special file not allowed: ' + item);
        else if (entry.isDirectory()) walk(full, item);
      }
    }
    walk(buildDir, '');
    return issues;
  }

  function deploy(params) {
    const repo = params.repo;
    const commit = params.commit;
    const subdir = params.subdir;
    const ns = params.namespace;
    const builder = params.builder || BUILDER_IMAGE;

    if (!repo) fail(400, 'missing_repo', 'repo is required');
    if (!commit) fail(400, 'missing_commit', 'commit is required');
    validateNamespace(ns);
    validateSubdir(subdir);
    if (!validCommit(commit)) fail(400, 'invalid_commit', 'commit must be a full 40 or 64 hex git object id');

    ensureDir(BUILDS_DIR);
    const workDir = fs.mkdtempSync(path.join(BUILDS_DIR, 'build-'));
    let stageDir = null;

    try {
      const repoDir = path.join(workDir, 'repo');
      try {
        execFileSync('git', ['clone', '--quiet', repo, repoDir], { stdio: 'inherit' });
        execFileSync('git', ['-C', repoDir, 'checkout', '--quiet', commit], { stdio: 'inherit' });
      } catch (e) {
        fail(500, 'git_failed', 'clone or checkout failed');
      }

      const head = execFileSync('git', ['-C', repoDir, 'rev-parse', 'HEAD'], { stdio: 'pipe' }).toString().trim();
      if (head !== commit) fail(400, 'commit_mismatch', 'checked-out HEAD does not match requested commit');

      const appDir = path.resolve(repoDir, subdir);
      let repoReal;
      let appReal;
      try {
        repoReal = fs.realpathSync(repoDir);
        appReal = fs.realpathSync(appDir);
      } catch {
        fail(400, 'subdir_not_found', 'subdir not found');
      }
      if (appReal !== repoReal && !appReal.startsWith(repoReal + path.sep)) fail(400, 'invalid_subdir', 'subdir escapes repository');

      const releaseBase = path.join(RELEASES_DIR, ns);
      ensureDir(releaseBase);
      stageDir = fs.mkdtempSync(path.join(releaseBase, '.staging-'));
      const owner = fs.statSync(repoDir);
      const absSubdir = path.relative(repoDir, appDir);
      const dockerArgs = [
        'run', '--rm',
        '--user', `${owner.uid}:${owner.gid}`,
        '--env', 'HOME=/tmp',
        '-v', repoDir + ':/repo',
        '-w', '/repo/' + absSubdir,
        builder,
        'make', 'build',
      ];
      try {
        execFileSync('docker', dockerArgs, { stdio: 'inherit' });
      } catch (e) {
        fail(500, 'build_failed', 'build failed' + (e.status == null ? '' : ' (exit ' + e.status + ')'));
      }

      const buildOutput = path.join(appDir, 'build');
      if (!fs.existsSync(buildOutput)) fail(500, 'build_output_missing', 'build output directory not found: build/');
      fs.cpSync(buildOutput, stageDir, { recursive: true });

      const issues = validateBuildOutput(stageDir);
      if (issues.length) fail(500, 'invalid_build_output', issues.join('; '));

      ensureNamespace(ns);

      const ts = new Date().toISOString().replace(/[^0-9]/g, '').slice(0, 17);
      const releaseId = ts + '-' + crypto.randomBytes(3).toString('hex');
      const releaseDir = path.join(RELEASES_DIR, ns, releaseId);
      ensureDir(path.dirname(releaseDir));
      fs.renameSync(stageDir, releaseDir);
      stageDir = null;
      activateRelease(ns, releaseDir);

      const cfg = loadConfig(ns) || {};
      Object.assign(cfg, {
        namespace: ns,
        repo,
        commit,
        subdir,
        builder,
        currentReleaseId: releaseId,
        deployedAt: new Date().toISOString(),
      });
      saveConfig(ns, cfg);

      const releases = listReleases(ns);
      for (const old of releases.slice(0, Math.max(0, releases.length - 3))) rmrfDir(path.join(RELEASES_DIR, ns, old));

      return { ok: true, namespace: ns, release: releaseId, path: APP_BASE_PATH + '/' + ns + '/' };
    } finally {
      rmrfDir(workDir);
      try { if (stageDir && fs.existsSync(stageDir) && fs.lstatSync(stageDir).isDirectory()) rmrfDir(stageDir); } catch {}
    }
  }

  function undeploy(params) {
    const ns = params.namespace;
    validateNamespace(ns);
    deactivateRelease(ns);
    rmrfDir(path.join(RELEASES_DIR, ns));
    rmrfDir(path.join(STORAGE_DIR, ns));
    rmrfDir(path.join(STATE_DIR, ns));
    return { ok: true, namespace: ns };
  }

  function rollback(params) {
    const ns = params.namespace;
    validateNamespace(ns);
    const releases = listReleases(ns);
    if (!releases.length) fail(404, 'no_releases', 'no releases for namespace ' + ns);

    let target = params.release;
    if (target) {
      if (!releases.includes(target)) fail(404, 'release_not_found', 'release ' + target + ' not found');
    } else {
      const current = currentReleaseId(ns);
      const currentIndex = current ? releases.indexOf(current) : -1;
      if (currentIndex > 0) target = releases[currentIndex - 1];
      else if (currentIndex === -1 && releases.length >= 2) target = releases[releases.length - 2];
      else fail(409, 'nothing_to_rollback', 'no previous release available');
    }

    activateRelease(ns, path.join(RELEASES_DIR, ns, target));
    const cfg = loadConfig(ns);
    if (cfg) {
      cfg.currentReleaseId = target;
      cfg.lastRollbackAt = new Date().toISOString();
      saveConfig(ns, cfg);
    }
    return { ok: true, namespace: ns, release: target };
  }

  function releases(params) {
    const ns = params.namespace;
    validateNamespace(ns);
    const items = listReleases(ns);
    if (!items.length && !fs.existsSync(path.join(RELEASES_DIR, ns))) fail(404, 'no_releases', 'no releases for namespace ' + ns);
    return { namespace: ns, current: currentReleaseId(ns), releases: items };
  }

  function inspect(params) {
    const ns = params.namespace;
    validateNamespace(ns);
    const cfg = loadConfig(ns);
    if (!cfg) fail(404, 'namespace_not_found', 'namespace ' + ns + ' not found');
    return cfg;
  }

  function namespaceCreate(params) {
    const ns = params.namespace;
    validateNamespace(ns);
    let quota = DEFAULT_QUOTA_BYTES;
    if (params.quota != null && params.quota !== '') {
      quota = Number(params.quota);
      if (!Number.isSafeInteger(quota) || quota <= 0) fail(400, 'invalid_quota', 'quota must be a positive integer');
    }
    const result = ensureNamespace(ns, quota);
    return { ok: true, namespace: ns, created: result.created, quota: result.quota };
  }

  function namespaceRemove(params) {
    const ns = params.namespace;
    validateNamespace(ns);
    rmrfDir(path.join(STORAGE_DIR, ns));
    rmrfDir(path.join(STATE_DIR, ns));
    return { ok: true, namespace: ns };
  }

  function namespaceInspect(params) {
    const ns = params.namespace;
    validateNamespace(ns);
    return { namespace: ns, quota: recalcQuota(ns), deployment: loadConfig(ns) };
  }

  function namespaceList() {
    if (!fs.existsSync(STATE_DIR)) return { namespaces: [] };
    const namespaces = fs.readdirSync(STATE_DIR).filter(ns => validNs(ns) && fs.existsSync(quotaPath(ns))).sort().map(ns => ({
      namespace: ns,
      quota: recalcQuota(ns),
      deployment: loadConfig(ns),
    }));
    return { namespaces };
  }

  return {
    deploy,
    undeploy,
    rollback,
    releases,
    inspect,
    namespaceCreate,
    namespaceRemove,
    namespaceInspect,
    namespaceList,
  };
}

module.exports = { createManagement, ManagementError };
