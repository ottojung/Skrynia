'use strict';

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { spawn } = require('child_process');
const { normalizeBasePath } = require('./base-path.js');
const { validNs, ensureDir, createShared } = require('./shared.js');
const { createPush } = require('./push.js');

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

  const shared = createShared(opts.dataDir);
  const { RELEASES_DIR, STORAGE_DIR, STATE_DIR } = shared;

  const BUILDS_DIR = path.join(shared.dataDir, 'builds');
  const BUILDER_IMAGE = opts.builderImage || process.env.SKRYNIA_BUILDER_IMAGE || 'skrynia-builder:0.1.0';
  const APP_BASE_PATH = normalizeBasePath(opts.appBasePath || process.env.SKRYNIA_APP_BASE_PATH || '/apps');
  const APP_DIR = opts.appDir || process.env.SKRYNIA_APP_DIR || '';
  const gitTimeoutMs = opts.gitTimeoutMs != null
    ? Number(opts.gitTimeoutMs)
    : Number(process.env.SKRYNIA_GIT_TIMEOUT_MS || 120000);
  const buildTimeoutMs = opts.buildTimeoutMs != null
    ? Number(opts.buildTimeoutMs)
    : Number(process.env.SKRYNIA_BUILD_TIMEOUT_MS || 900000);
  if (!Number.isFinite(gitTimeoutMs) || gitTimeoutMs <= 0) throw new Error('git timeout must be a positive number');
  if (!Number.isFinite(buildTimeoutMs) || buildTimeoutMs <= 0) throw new Error('build timeout must be a positive number');
  const push = opts.push || createPush({ dataDir: shared.dataDir });

  const HEX40 = /^[0-9a-f]{40}$/;
  const HEX64 = /^[0-9a-f]{64}$/;

  const SCP_REPO = /^[a-zA-Z0-9._-]+@[a-zA-Z0-9._-]+:[^ \t\n\r\x00]+$/;

  function rmrfDir(dir) { if (fs.existsSync(dir)) fs.rmSync(dir, { recursive: true, force: true }); }
  function fail(status, code, detail) { throw new ManagementError(status, code, detail); }

  const OUTPUT_TAIL_BYTES = 8192;
  const SENSITIVE_ENV_NAME = /(?:SECRET|TOKEN|PASSWORD|PASSWD|CREDENTIAL|COOKIE|PRIVATE_KEY|API_KEY|AUTH)/i;

  function appendTail(current, chunk, limit) {
    if (!chunk.length) return current;
    const combined = Buffer.concat([current, chunk]);
    return combined.length > limit ? combined.subarray(combined.length - limit) : combined;
  }

  function decodeTail(tail) {
    let value = tail.toString('utf8');
    if (tail.length && tail[0] >= 0x80) {
      while (Buffer.from(value, 'utf8').length < tail.length && value.length) value = value.slice(1);
    }
    return value.replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, ' ');
  }

  function diagnosticText(stdout, stderr) {
    let value = '';
    if (stdout.length) value += 'stdout: ' + decodeTail(stdout) + '\n';
    if (stderr.length) value += 'stderr: ' + decodeTail(stderr);
    for (const [name, secret] of Object.entries(process.env)) {
      if (name && secret && secret.length >= 4 && SENSITIVE_ENV_NAME.test(name)) {
        value = value.split(secret).join('[redacted]');
      }
    }
    return value.trim();
  }

  function terminateProcessGroup(child, signal) {
    try { process.kill(-child.pid, signal); }
    catch { try { child.kill(signal); } catch {} }
  }

  function runFile(command, args, captureStdout, timeoutMs, onTimeout) {
    return new Promise((resolve, reject) => {
      const child = spawn(command, args, { detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
      let stdout = Buffer.alloc(0);
      let stderr = Buffer.alloc(0);
      let settled = false;
      let timedOut = false;
      let cleanup = Promise.resolve();
      let forceTimer = null;
      const timer = setTimeout(() => {
        timedOut = true;
        cleanup = Promise.resolve(onTimeout ? onTimeout() : undefined).catch(() => {});
        terminateProcessGroup(child, 'SIGTERM');
        forceTimer = setTimeout(() => terminateProcessGroup(child, 'SIGKILL'), 1000);
      }, timeoutMs);

      function finish(fn, value) {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        if (forceTimer) clearTimeout(forceTimer);
        fn(value);
      }

      child.stdout.on('data', chunk => {
        stdout = appendTail(stdout, chunk, OUTPUT_TAIL_BYTES);
        if (!captureStdout) process.stdout.write(chunk);
      });
      child.stderr.on('data', chunk => {
        stderr = appendTail(stderr, chunk, OUTPUT_TAIL_BYTES);
        process.stderr.write(chunk);
      });
      child.on('error', err => finish(reject, err));
      child.on('close', (code, signal) => {
        if (timedOut) {
          cleanup.then(() => {
            const err = new Error(command + ' timed out');
            err.timedOut = true;
            const diagnostic = diagnosticText(stdout, stderr);
            if (diagnostic) err.diagnostic = diagnostic;
            finish(reject, err);
          });
          return;
        }
        if (code === 0) return finish(resolve, captureStdout ? decodeTail(stdout).trim() : undefined);
        const err = new Error(command + ' failed');
        err.status = code;
        err.signal = signal;
        const diagnostic = diagnosticText(stdout, stderr);
        if (diagnostic) err.diagnostic = diagnostic;
        finish(reject, err);
      });
    });
  }

  function removeBuilderContainer(name, cidFile) {
    return runFile('docker', ['rm', '-f', name], false, 10000, null).catch(() => {
      if (!fs.existsSync(cidFile)) return;
      const cid = fs.readFileSync(cidFile, 'utf8').trim();
      if (cid) return runFile('docker', ['rm', '-f', cid], false, 10000, null);
    });
  }

  let deploymentInProgress = false;
  function requireNoDeployment() {
    if (deploymentInProgress) fail(409, 'deployment_in_progress', 'another deployment is in progress');
  }

  function validCommit(commit) { return typeof commit === 'string' && (HEX40.test(commit) || HEX64.test(commit)); }
  function validRepo(repo) { return typeof repo === 'string' && SCP_REPO.test(repo); }

  function loadConfig(ns) {
    const p = shared.nsConfigPath(ns);
    return fs.existsSync(p) ? JSON.parse(fs.readFileSync(p, 'utf8')) : null;
  }

  function saveConfig(ns, cfg) {
    ensureDir(path.dirname(shared.nsConfigPath(ns)));
    fs.writeFileSync(shared.nsConfigPath(ns), JSON.stringify(cfg, null, 2));
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
    const link = APP_DIR ? path.join(APP_DIR, ns) : shared.nsCurrentLink(ns);
    const tmp = link + '.tmp.' + process.pid;
    try { fs.unlinkSync(tmp); } catch {}
    fs.symlinkSync(releaseDir, tmp);
    fs.renameSync(tmp, link);
  }

  function deactivateRelease(ns) {
    const link = APP_DIR ? path.join(APP_DIR, ns) : shared.nsCurrentLink(ns);
    try { fs.unlinkSync(link); } catch (e) {
      if (e.code !== 'ENOENT') throw e;
    }
  }

  function currentReleaseId(ns) {
    const link = APP_DIR ? path.join(APP_DIR, ns) : shared.nsCurrentLink(ns);
    try { return path.basename(fs.readlinkSync(link)); }
    catch { return null; }
  }

  function validateNamespace(ns) {
    if (!ns) fail(400, 'missing_namespace', 'namespace is required');
    if (!validNs(ns)) fail(400, 'invalid_namespace', 'namespace must match /^[a-z0-9][a-z0-9_-]{0,63}$/');
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

  function canonicalizeTreePermissions(dir) {
    fs.chmodSync(dir, 0o755);
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        canonicalizeTreePermissions(full);
        fs.chmodSync(full, 0o755);
      } else if (entry.isFile()) {
        fs.chmodSync(full, 0o644);
      }
    }
  }

  function validateTreePermissions(dir) {
    const issues = [];
    function walk(d, rel) {
      const st = fs.statSync(d);
      if ((st.mode & 0o777) !== 0o755) issues.push('dir mode ' + octal(st.mode) + ': ' + (rel || '.'));
      for (const entry of fs.readdirSync(d, { withFileTypes: true })) {
        const full = path.join(d, entry.name);
        const item = rel ? rel + '/' + entry.name : entry.name;
        if (entry.isDirectory()) walk(full, item);
        else if (entry.isFile()) {
          const s = fs.statSync(full);
          if ((s.mode & 0o777) !== 0o644) issues.push('file mode ' + octal(s.mode) + ': ' + item);
        }
      }
    }
    walk(dir, '');
    return issues;
  }

  function octal(mode) { return '0' + (mode & 0o777).toString(8); }

  function subprocessFailureDetail(label, err) {
    const status = err.timedOut ? ' timed out' : (err.status == null ? '' : ' (exit ' + err.status + ')');
    const diagnostic = err.diagnostic ? ': ' + err.diagnostic : '';
    return label + status + diagnostic;
  }

  async function deploy(params) {
    const repo = params.repo;
    const commit = params.commit;
    const subdir = params.subdir;
    const ns = params.namespace;
    const builder = params.builder || BUILDER_IMAGE;

    if (!repo) fail(400, 'missing_repo', 'repo is required');
    if (!validRepo(repo)) fail(400, 'invalid_repo', 'repo must be an SSH git URL in scp-like form user@host:path');
    if (!commit) fail(400, 'missing_commit', 'commit is required');
    validateNamespace(ns);
    validateSubdir(subdir);
    if (!validCommit(commit)) fail(400, 'invalid_commit', 'commit must be a full 40 or 64 hex git object id');
    requireNoDeployment();
    deploymentInProgress = true;

    let workDir = null;
    let stageDir = null;
    let builderSequence = 0;

    try {
      ensureDir(BUILDS_DIR);
      workDir = fs.mkdtempSync(path.join(BUILDS_DIR, 'build-'));
      const repoDir = path.join(workDir, 'repo');
      let head;
      let gitOperation = 'clone';
      try {
        await runFile('git', ['clone', '--quiet', repo, repoDir], false, gitTimeoutMs);
        gitOperation = 'checkout';
        await runFile('git', ['-C', repoDir, 'checkout', '--quiet', commit], false, gitTimeoutMs);
        gitOperation = 'verify';
        head = await runFile('git', ['-C', repoDir, 'rev-parse', 'HEAD'], true, gitTimeoutMs);
      } catch (e) {
        fail(500, 'git_failed', subprocessFailureDetail(gitOperation + ' failed', e));
      }
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
      const containerName = 'skrynia-build-' + process.pid + '-' + (++builderSequence);
      const cidFile = path.join(workDir, 'builder.cid');
      const dockerArgs = [
        'run', '--rm', '--pull=always',
        '--name', containerName,
        '--cidfile', cidFile,
        '--user', `${owner.uid}:${owner.gid}`,
        '--env', 'HOME=/tmp',
        '-v', repoDir + ':/repo',
        '-w', '/repo/' + absSubdir,
        builder,
        'make', 'build',
      ];
      try {
        await runFile('docker', dockerArgs, false, buildTimeoutMs, () => removeBuilderContainer(containerName, cidFile));
      } catch (e) {
        fail(500, 'build_failed', subprocessFailureDetail('build failed', e));
      }

      const buildOutput = path.join(appDir, 'build');
      if (!fs.existsSync(buildOutput)) fail(500, 'build_output_missing', 'build output directory not found: build/');
      fs.cpSync(buildOutput, stageDir, { recursive: true });

      const issues = validateBuildOutput(stageDir);
      if (issues.length) fail(500, 'invalid_build_output', issues.join('; '));

      canonicalizeTreePermissions(stageDir);

      const permIssues = validateTreePermissions(stageDir);
      if (permIssues.length) fail(500, 'tree_permissions_invalid', permIssues.join('; '));

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
      deploymentInProgress = false;
      if (workDir) rmrfDir(workDir);
      try { if (stageDir && fs.existsSync(stageDir) && fs.lstatSync(stageDir).isDirectory()) rmrfDir(stageDir); } catch {}
    }
  }

  function undeploy(params) {
    requireNoDeployment();
    const ns = params.namespace;
    validateNamespace(ns);
    deactivateRelease(ns);
    rmrfDir(path.join(RELEASES_DIR, ns));
    rmrfDir(path.join(STORAGE_DIR, ns));
    rmrfDir(path.join(STATE_DIR, ns));
    return { ok: true, namespace: ns };
  }

  function rollback(params) {
    requireNoDeployment();
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

  function ensureNamespace(ns, quotaBytes) {
    ensureDir(shared.nsStorageDir(ns));
    ensureDir(path.join(STATE_DIR, ns));
    const p = shared.nsQuotaPath(ns);
    if (fs.existsSync(p)) {
      shared.loadQuota(ns);
      return { created: false, quota: shared.recalcQuota(ns) };
    }
    const q = {
      quotaBytes: quotaBytes || shared.DEFAULT_QUOTA_BYTES,
      maxObjects: shared.DEFAULT_MAX_OBJECTS,
    };
    fs.writeFileSync(p, JSON.stringify(q, null, 2));
    return { created: true, quota: q };
  }

  function namespaceCreate(params) {
    requireNoDeployment();
    const ns = params.namespace;
    validateNamespace(ns);
    let quota = shared.DEFAULT_QUOTA_BYTES;
    if (params.quota != null && params.quota !== '') {
      quota = Number(params.quota);
      if (!Number.isSafeInteger(quota) || quota <= 0) fail(400, 'invalid_quota', 'quota must be a positive integer');
    }
    const result = ensureNamespace(ns, quota);
    return { ok: true, namespace: ns, created: result.created, quota: result.quota };
  }

  function namespaceRemove(params) {
    requireNoDeployment();
    const ns = params.namespace;
    validateNamespace(ns);
    rmrfDir(shared.nsStorageDir(ns));
    rmrfDir(path.join(STATE_DIR, ns));
    return { ok: true, namespace: ns };
  }

  function namespaceInspect(params) {
    const ns = params.namespace;
    validateNamespace(ns);
    if (!fs.existsSync(shared.nsQuotaPath(ns))) fail(404, 'namespace_not_found', 'namespace ' + ns + ' not found');
    return { namespace: ns, quota: shared.recalcQuota(ns), deployment: loadConfig(ns) };
  }

  function namespaceList() {
    if (!fs.existsSync(STATE_DIR)) return { namespaces: [] };
    const namespaces = fs.readdirSync(STATE_DIR).filter(ns => validNs(ns) && fs.existsSync(shared.nsQuotaPath(ns))).sort().map(ns => ({
      namespace: ns,
      quota: shared.recalcQuota(ns),
      deployment: loadConfig(ns),
    }));
    return { namespaces };
  }

  // --- Web Push rules: exact namespace + exact key + kinds -> channels ---

  function requirePushNs(ns) {
    validateNamespace(ns);
    if (!fs.existsSync(shared.nsQuotaPath(ns))) fail(404, 'namespace_not_found', 'namespace ' + ns + ' not found');
  }

  function pushRuleSet(params) {
    const ns = params.namespace;
    requirePushNs(ns);
    if (!params.key) fail(400, 'missing_key', 'key is required');
    const kinds = push.parseList(params.on == null || params.on === '' ? 'create,replace,delete' : params.on);
    const channels = push.parseList(params.channels);
    const problem = push.validateRuleParts(params.key, kinds, channels);
    if (problem) fail(400, 'invalid_push_rule', problem);
    return { ok: true, rule: push.setRule(ns, params.key, kinds, channels) };
  }

  function pushRuleGet(params) {
    requirePushNs(params.namespace);
    if (!params.key) fail(400, 'missing_key', 'key is required');
    const rule = push.getRule(params.namespace, params.key);
    if (!rule) fail(404, 'push_rule_not_found', 'no push rule for key ' + params.key);
    return { namespace: params.namespace, rule };
  }

  function pushRuleList(params) {
    requirePushNs(params.namespace);
    return { namespace: params.namespace, rules: push.loadRules(params.namespace) };
  }

  function pushRuleRemove(params) {
    requirePushNs(params.namespace);
    if (!params.key) fail(400, 'missing_key', 'key is required');
    if (!push.removeRule(params.namespace, params.key)) fail(404, 'push_rule_not_found', 'no push rule for key ' + params.key);
    return { ok: true, namespace: params.namespace, key: params.key };
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
    pushRuleSet,
    pushRuleGet,
    pushRuleList,
    pushRuleRemove,
  };
}

module.exports = { createManagement, ManagementError };
