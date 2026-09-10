#!/usr/bin/env node
'use strict';

// Skrynia admin CLI.
//
// All filesystem operations use Node fs APIs or execFileSync with argument
// arrays. No shell-string command construction (no execSync with template
// literals). This prevents injection via namespace names, paths, or URLs.

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { execFileSync } = require('child_process');
const { normalizeBasePath } = require('./base-path.js');
const { NS_RE, validNs, ensureDir, DEFAULT_QUOTA_BYTES, DEFAULT_MAX_OBJECTS, createShared } = require('./shared.js');
const shared = createShared();

const BUILDS_DIR = path.join(shared.dataDir, 'builds');
const BUILDER_IMAGE = process.env.SKRYNIA_BUILDER_IMAGE || 'skrynia-builder:0.1.0';

// Configurable URL base path for app serving (informational only in admin CLI).
// There is no canonical prefix; the deployer sets this to match the
// reverse proxy or web server configuration.
const APP_BASE_PATH = normalizeBasePath(process.env.SKRYNIA_APP_BASE_PATH || '/apps');

// Optional separate filesystem directory where active app symlinks are exposed.
// When set, admin deploy creates APP_DIR/{ns} -> release dir symlinks.
const APP_DIR = process.env.SKRYNIA_APP_DIR || '';

// --- Utilities ---

function rmrfDir(dir) {
  if (!fs.existsSync(dir)) return;
  fs.rmSync(dir, { recursive: true, force: true });
}

function die(msg) { throw new Error('skrynia: ' + msg); }
function info(msg) { process.stderr.write('skrynia: ' + msg + '\n'); }

// --- Unified active link ---
// When APP_DIR is configured, the active link lives there (for an external web
// server to serve directly).  Otherwise it is the internal RELEASES_DIR/.../current.
// Deploy and rollback always atomically swap this one symlink.

function activeLink(ns) {
  return APP_DIR ? path.join(APP_DIR, ns) : shared.nsCurrentLink(ns);
}

function activateRelease(ns, releaseDir) {
  if (APP_DIR) ensureDir(APP_DIR);
  else ensureDir(path.join(shared.RELEASES_DIR, ns));
  const link = activeLink(ns);
  const tmpLink = link + '.tmp.' + process.pid;
  try { fs.unlinkSync(tmpLink); } catch {}
  safeSymlinkSync(releaseDir, tmpLink);
  safeRenameSync(tmpLink, link);
}

function deactivateRelease(ns) {
  const link = activeLink(ns);
  if (fs.existsSync(link)) {
    try { fs.unlinkSync(link); } catch {}
  }
}

function currentReleaseId(ns) {
  const link = activeLink(ns);
  if (!fs.existsSync(link)) return null;
  try { return fs.basename(fs.readlinkSync(link)); } catch { return null; }
}

// --- Commit hash validator: must be 40 or 64 hex chars ---
const HEX40 = /^[0-9a-f]{40}$/;
const HEX64 = /^[0-9a-f]{64}$/;
function validCommit(h) {
  return typeof h === 'string' && (HEX40.test(h) || HEX64.test(h));
}

// --- Flag parser (--key value pairs, returns flags map + remaining args) ---

function parseFlags(args) {
  const flags = {};
  const rest = [];
  let i = 0;
  while (i < args.length) {
    if (args[i].startsWith('--') && i + 1 < args.length && !args[i + 1].startsWith('--')) {
      flags[args[i].slice(2)] = args[i + 1];
      i += 2;
    } else {
      rest.push(args[i]);
      i++;
    }
  }
  return { flags, rest };
}

function loadConfig(ns) {
  const p = shared.nsConfigPath(ns);
  if (!fs.existsSync(p)) return null;
  return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function saveConfig(ns, cfg) {
  ensureDir(path.dirname(shared.nsConfigPath(ns)));
  fs.writeFileSync(shared.nsConfigPath(ns), JSON.stringify(cfg, null, 2));
}

// --- Build output validation ---

function validateBuildOutput(buildDir) {
  const issues = [];
  function walk(dir, rel) {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const full = path.join(dir, entry.name);
      const r = rel ? rel + '/' + entry.name : entry.name;
      if (entry.isSymbolicLink()) {
        issues.push('symlink not allowed: ' + r);
      } else if (entry.isBlockDevice() || entry.isCharacterDevice() || entry.isFIFO() || entry.isSocket()) {
        issues.push('special file not allowed: ' + r);
      } else if (entry.isDirectory()) {
        walk(full, r);
      }
    }
  }
  walk(buildDir, '');
  return issues;
}

// --- Safe filesystem operations ---

function safeRenameSync(oldPath, newPath) { fs.renameSync(oldPath, newPath); }
function safeSymlinkSync(target, linkPath) { fs.symlinkSync(target, linkPath); }

// --- Namespace creation / ensure ---

function ensureNamespace(ns, quotaBytes) {
  ensureDir(path.join(shared.STORAGE_DIR, ns));
  ensureDir(path.join(shared.STATE_DIR, ns));
  const p = shared.nsQuotaPath(ns);
  if (!fs.existsSync(p)) {
    const q = { bytes: 0, count: 0, quotaBytes: quotaBytes || DEFAULT_QUOTA_BYTES, maxObjects: DEFAULT_MAX_OBJECTS };
    fs.writeFileSync(p, JSON.stringify(q, null, 2));
    info('created namespace ' + ns + ' (quota: ' + q.quotaBytes + ' bytes)');
  }
}

// --- Release listing (excludes hidden staging dirs) ---

function listReleases(ns) {
  const relBase = path.join(shared.RELEASES_DIR, ns);
  if (!fs.existsSync(relBase)) return [];
  return fs.readdirSync(relBase)
    .filter(d => !d.startsWith('.') && d !== 'current' && fs.statSync(path.join(relBase, d)).isDirectory())
    .sort();
}

// --- Commands ---

function cmdDeploy(args) {
  const { flags, rest } = parseFlags(args);

  const repoUrl = flags['repo'];
  const commit = flags['commit'];
  const ns = flags['namespace'];
  const subdir = flags['subdir'];

  const missing = [];
  if (!repoUrl) missing.push('--repo');
  if (!commit) missing.push('--commit');
  if (!ns) missing.push('--namespace');
  if (!subdir) missing.push('--subdir');
  if (missing.length > 0) {
    die('missing required flags: ' + missing.join(', ') + '\nusage: skrynia deploy --repo <url> --commit <sha> --subdir <path> --namespace <name> [--builder IMAGE]');
  }
  if (rest.length > 0) {
    die('unexpected positional arguments: ' + rest.join(' '));
  }

  if (!validNs(ns)) die('invalid namespace: ' + ns + ' (must match ' + NS_RE + ')');
  if (!validCommit(commit)) die('--commit must be a full 40 or 64 hex char git object id');

  let builderImage = BUILDER_IMAGE;
  if (flags['builder']) builderImage = flags['builder'];

  info('deploying namespace=' + ns);
  info('  repo=' + repoUrl);
  info('  commit=' + commit);
  info('  subdir=' + subdir);
  info('  builder=' + builderImage);

  // 0. Early subdir validation
  if (path.isAbsolute(subdir)) die('subdirectory must be relative, got: ' + subdir);
  if (subdir.includes('..')) die('subdirectory must not contain ..: ' + subdir);
  if (subdir.includes('\0')) die('subdirectory must not contain null bytes');
  if (subdir === '') die('subdirectory must not be empty');

  // 1. Clone to temp workspace
  ensureDir(BUILDS_DIR);
  const workDir = fs.mkdtempSync(path.join(BUILDS_DIR, 'build-'));
  const stageDir = shared.nsStagingDir(ns);
  try {
    info('cloning repository...');
    execFileSync('git', ['clone', '--quiet', repoUrl, path.join(workDir, 'repo')], { stdio: 'inherit' });
    execFileSync('git', ['-C', path.join(workDir, 'repo'), 'checkout', '--quiet', commit], { stdio: 'inherit' });

    // Verify HEAD matches requested commit
    const head = execFileSync('git', ['-C', path.join(workDir, 'repo'), 'rev-parse', 'HEAD'], { stdio: 'pipe' }).toString().trim();
    if (head !== commit) die('checked-out HEAD ' + head + ' does not match requested commit ' + commit);

    // 2. Validate subdirectory stays inside clone
    const repoDir = path.join(workDir, 'repo');
    const appDir = path.resolve(repoDir, subdir);
    const repoReal = fs.realpathSync(repoDir);
    const appReal = fs.realpathSync(appDir);
    if (!appReal.startsWith(repoReal + path.sep) && appReal !== repoReal) {
      die('subdirectory escapes repository: ' + subdir);
    }
    if (!fs.existsSync(appDir)) die('subdirectory not found: ' + subdir);

    // 3. Build in container (repo RW, root FS ro, drop caps)
    ensureDir(stageDir);
    const absSubdir = path.relative(repoDir, appDir);
    const owner = fs.statSync(repoDir);

    info('building with container...');
    const dockerArgs = [
      'run', '--rm',
      '--user', `${owner.uid}:${owner.gid}`,
      '--read-only',
      '--tmpfs', '/tmp:size=256m',
      '--network', 'none',
      '--security-opt', 'no-new-privileges',
      '--cap-drop', 'ALL',
      '-v', repoDir + ':/repo',
      '-w', '/repo/' + absSubdir,
      builderImage,
      'make', 'build',
    ];
    try {
      execFileSync('docker', dockerArgs, { stdio: 'inherit' });
    } catch (e) {
      die('build failed (exit ' + e.status + ')');
    }

    // 4. Validate build output
    const buildOutput = path.join(appDir, 'build');
    if (!fs.existsSync(buildOutput)) die('build output directory not found: build/');

    fs.cpSync(buildOutput, stageDir, { recursive: true });

    const issues = validateBuildOutput(stageDir);
    if (issues.length > 0) die('invalid build output:\n  ' + issues.join('\n  '));

    // 5. Auto-create namespace ONLY after successful build validation
    ensureNamespace(ns);

    // 6. Atomic rename: staging dir -> release dir (same filesystem)
    const now = Date.now();
    const ts = new Date(now).toISOString().replace(/[^0-9]/g, '').slice(0, 17);
    const rand = crypto.randomBytes(3).toString('hex');
    const releaseId = ts + '-' + rand;
    const relDir = path.join(shared.RELEASES_DIR, ns, releaseId);
    ensureDir(path.dirname(relDir));
    safeRenameSync(stageDir, relDir);

    // 7. Atomic activation
    activateRelease(ns, relDir);

    // 8. Save deployment config
    const cfg = loadConfig(ns) || {};
    Object.assign(cfg, {
      namespace: ns, repo: repoUrl, commit, subdir,
      builder: builderImage, currentReleaseId: releaseId,
      deployedAt: new Date().toISOString(),
    });
    saveConfig(ns, cfg);

    info('activated release ' + releaseId + ' for ' + APP_BASE_PATH + '/' + ns + '/');

    // 9. Prune old releases (keep last 3, skip hidden staging dirs)
    const releases = listReleases(ns);
    if (releases.length > 3) {
      for (const old of releases.slice(0, releases.length - 3)) {
        rmrfDir(path.join(shared.RELEASES_DIR, ns, old));
        info('pruned old release ' + old);
      }
    }
  } finally {
    rmrfDir(workDir);
    // Clean staging dir on failure
    try { if (fs.existsSync(stageDir) && fs.lstatSync(stageDir).isDirectory()) rmrfDir(stageDir); } catch {}
  }
}

function cmdUndeploy(args) {
  const { flags, rest } = parseFlags(args);
  const ns = flags['namespace'];
  if (!ns) die('missing required flag: --namespace\nusage: skrynia undeploy --namespace <name>');
  if (rest.length > 0) die('unexpected positional arguments: ' + rest.join(' '));
  if (!validNs(ns)) die('invalid namespace: ' + ns);

  info('undeploying namespace=' + ns);

  deactivateRelease(ns);

  const relDir = path.join(shared.RELEASES_DIR, ns);
  if (fs.existsSync(relDir)) { rmrfDir(relDir); info('removed releases'); }

  const storageNs = path.join(shared.STORAGE_DIR, ns);
  if (fs.existsSync(storageNs)) { rmrfDir(storageNs); info('removed stored data'); }

  const stateNs = path.join(shared.STATE_DIR, ns);
  if (fs.existsSync(stateNs)) { rmrfDir(stateNs); info('removed namespace state'); }

  info('undeploy complete for ' + APP_BASE_PATH + '/' + ns + '/');
}

function cmdRollback(args) {
  const { flags, rest } = parseFlags(args);
  const ns = flags['namespace'];
  const target = flags['release'];
  if (!ns) die('missing required flag: --namespace\nusage: skrynia rollback --namespace <name> [--release <id>]');
  if (rest.length > 0) die('unexpected positional arguments: ' + rest.join(' '));
  if (!validNs(ns)) die('invalid namespace: ' + ns);

  const relBase = path.join(shared.RELEASES_DIR, ns);
  if (!fs.existsSync(relBase)) die('no releases for namespace ' + ns);

  const releases = listReleases(ns);
  if (releases.length === 0) die('no releases for namespace ' + ns);

  let rollbackTo;
  if (target) {
    if (!releases.includes(target)) die('release ' + target + ' not found');
    rollbackTo = target;
  } else {
    if (releases.length < 2) die('only one release exists, nothing to rollback to');
    rollbackTo = releases[releases.length - 2];
  }

  const targetDir = path.join(relBase, rollbackTo);
  activateRelease(ns, targetDir);

  const cfg = loadConfig(ns);
  if (cfg) {
    cfg.currentReleaseId = rollbackTo;
    cfg.lastRollbackAt = new Date().toISOString();
    saveConfig(ns, cfg);
  }

  info('rolled back ' + APP_BASE_PATH + '/' + ns + '/ to release ' + rollbackTo);
}

function cmdReleases(args) {
  const { flags, rest } = parseFlags(args);
  const ns = flags['namespace'];
  if (!ns) die('missing required flag: --namespace\nusage: skrynia releases --namespace <name>');
  if (rest.length > 0) die('unexpected positional arguments: ' + rest.join(' '));
  if (!validNs(ns)) die('invalid namespace: ' + ns);

  const relBase = path.join(shared.RELEASES_DIR, ns);
  if (!fs.existsSync(relBase)) die('no releases for namespace ' + ns);

  const current = currentReleaseId(ns) || '(none)';

  const releases = listReleases(ns);

  console.log('Namespace: ' + ns);
  console.log('Current:   ' + current);
  console.log('Releases:');
  for (const r of releases) {
    const marker = r === current ? ' <-- current' : '';
    console.log('  ' + r + marker);
  }
}

function cmdInspect(args) {
  const { flags, rest } = parseFlags(args);
  const ns = flags['namespace'];
  if (!ns) die('missing required flag: --namespace\nusage: skrynia inspect --namespace <name>');
  if (rest.length > 0) die('unexpected positional arguments: ' + rest.join(' '));
  if (!validNs(ns)) die('invalid namespace: ' + ns);

  const cfg = loadConfig(ns);
  if (!cfg) die('namespace ' + ns + ' not found');
  console.log(JSON.stringify(cfg, null, 2));
}

function cmdNsCreate(args) {
  const { flags, rest } = parseFlags(args);
  const ns = flags['namespace'];
  const quotaStr = flags['quota'];
  if (!ns) die('missing required flag: --namespace\nusage: skrynia ns create --namespace <name> [--quota BYTES]');
  if (rest.length > 0) die('unexpected positional arguments: ' + rest.join(' '));
  if (!validNs(ns)) die('invalid namespace: ' + ns + ' (must match ' + NS_RE + ')');

  ensureDir(path.join(shared.STORAGE_DIR, ns));
  ensureDir(path.join(shared.STATE_DIR, ns));

  const quotaBytes = quotaStr ? parseInt(quotaStr, 10) : DEFAULT_QUOTA_BYTES;
  const p = shared.nsQuotaPath(ns);
  if (!fs.existsSync(p)) {
    const q = { bytes: 0, count: 0, quotaBytes, maxObjects: DEFAULT_MAX_OBJECTS };
    fs.writeFileSync(p, JSON.stringify(q, null, 2));
    info('created namespace ' + ns + ' (quota: ' + quotaBytes + ' bytes)');
  } else {
    info('namespace ' + ns + ' already exists (quota preserved)');
  }
}

function cmdNsRemove(args) {
  const { flags, rest } = parseFlags(args);
  const ns = flags['namespace'];
  if (!ns) die('missing required flag: --namespace\nusage: skrynia ns remove --namespace <name>');
  if (rest.length > 0) die('unexpected positional arguments: ' + rest.join(' '));
  if (!validNs(ns)) die('invalid namespace: ' + ns);

  rmrfDir(path.join(shared.STORAGE_DIR, ns));
  rmrfDir(path.join(shared.STATE_DIR, ns));
  info('removed namespace ' + ns);
}

function cmdNsInspect(args) {
  const { flags, rest } = parseFlags(args);
  const ns = flags['namespace'];
  if (!ns) die('missing required flag: --namespace\nusage: skrynia ns inspect --namespace <name>');
  if (rest.length > 0) die('unexpected positional arguments: ' + rest.join(' '));
  if (!validNs(ns)) die('invalid namespace: ' + ns);

  const q = shared.recalcQuota(ns);
  const cfg = loadConfig(ns);
  console.log(JSON.stringify({ namespace: ns, quota: q, deployment: cfg || null }, null, 2));
}

function cmdNsList() {
  if (!fs.existsSync(shared.STORAGE_DIR)) { console.log('No namespaces.'); return; }
  const entries = fs.readdirSync(shared.STORAGE_DIR).filter(e => {
    try { return fs.statSync(path.join(shared.STORAGE_DIR, e)).isDirectory(); }
    catch { return false; }
  });
  if (entries.length === 0) { console.log('No namespaces.'); return; }
  for (const ns of entries.sort()) {
    const q = shared.recalcQuota(ns);
    const cfg = loadConfig(ns);
    const deployed = cfg ? ' (repo=' + (cfg.commit || '').slice(0, 8) + '...)' : '';
    console.log(ns + ': ' + q.count + ' objects, ' + q.bytes + '/' + q.quotaBytes + ' bytes' + deployed);
  }
}

function cmdHelp() {
  console.log('skrynia - Skrynia platform admin CLI\n' +
    '\nUsage:\n' +
    '  skrynia deploy --repo <url> --commit <sha> --subdir <path> --namespace <name>\n' +
    '                        [--builder IMAGE]\n' +
    '                        Deploy app from git source\n' +
    '  skrynia undeploy --namespace <name>\n' +
    '                        Remove app, releases, namespace data and state\n' +
    '  skrynia rollback --namespace <name> [--release <id>]\n' +
    '                        Rollback to previous or specified release\n' +
    '  skrynia releases --namespace <name>\n' +
    '                        List releases for a namespace\n' +
    '  skrynia inspect --namespace <name>\n' +
    '                        Show deployment config for a namespace\n' +
    '\n  skrynia ns create --namespace <name> [--quota BYTES]\n' +
    '                        Create a namespace with quota\n' +
    '  skrynia ns remove --namespace <name>\n' +
    '                        Delete namespace and all its data\n' +
    '  skrynia ns inspect --namespace <name>\n' +
    '                        Show namespace usage and config\n' +
    '  skrynia ns list       List all namespaces with usage\n' +
    '\n  skrynia help          Show this help\n' +
    '\nExamples:\n' +
    '  skrynia deploy --repo git@github.com:myorg/myapp.git --commit abc123...def --subdir . --namespace myapp\n' +
    '  skrynia deploy --repo git@github.com:myorg/mono.git --commit def456...ghi --subdir frontend --namespace myapp\n' +
    '  skrynia undeploy --namespace myapp\n' +
    '  skrynia rollback --namespace myapp\n' +
    '  skrynia releases --namespace myapp\n' +
    '  skrynia ns list');
}

// --- Main ---

const args = process.argv.slice(2);
if (args.length === 0 || args[0] === 'help') { cmdHelp(); process.exit(0); }

const cmd = args[0];
const cmdArgs = args.slice(1);

try {
  switch (cmd) {
    case 'deploy': cmdDeploy(cmdArgs); break;
    case 'undeploy': cmdUndeploy(cmdArgs); break;
    case 'rollback': cmdRollback(cmdArgs); break;
    case 'releases': cmdReleases(cmdArgs); break;
    case 'inspect': cmdInspect(cmdArgs); break;
    case 'ns':
      if (cmdArgs[0] === 'create') cmdNsCreate(cmdArgs.slice(1));
      else if (cmdArgs[0] === 'remove') cmdNsRemove(cmdArgs.slice(1));
      else if (cmdArgs[0] === 'inspect') cmdNsInspect(cmdArgs.slice(1));
      else if (cmdArgs[0] === 'list') cmdNsList();
      else die('unknown ns command: use create, remove, inspect, or list');
      break;
    default:
      die('unknown command: ' + cmd + ". Run 'skrynia help' for usage.");
  }
} catch (e) {
  process.stderr.write('skrynia: ' + e.message + '\n');
  process.exit(1);
}
