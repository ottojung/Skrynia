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

const DATA_DIR = process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
const RELEASES_DIR = path.join(DATA_DIR, 'releases');
const STORAGE_DIR = path.join(DATA_DIR, 'storage');
const STATE_DIR = path.join(DATA_DIR, 'state');
const BUILDER_IMAGE = process.env.SKRYNIA_BUILDER_IMAGE || 'ghcr.io/ottojung/skrynia-builder:0.1.0';

// --- Utilities ---

function ensureDir(dir) { fs.mkdirSync(dir, { recursive: true }); }

function rmrfDir(dir) {
  if (!fs.existsSync(dir)) return;
  fs.rmSync(dir, { recursive: true, force: true });
}

function die(msg) { throw new Error('skrynia: ' + msg); }
function info(msg) { process.stderr.write('skrynia: ' + msg + '\n'); }

function currentLink(ns) { return path.join(RELEASES_DIR, ns, 'current'); }
function configPath(ns) { return path.join(STATE_DIR, ns, 'config.json'); }
function quotaPath(ns) { return path.join(STATE_DIR, ns, 'quota.json'); }

// --- Strict namespace validator ---
// Same rule as server.js: one canonical validator for all namespace-derived paths.
const NS_RE = /^[a-z0-9][a-z0-9_-]{0,63}$/;
function validNs(ns) {
  return typeof ns === 'string' && NS_RE.test(ns);
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
  const p = configPath(ns);
  if (!fs.existsSync(p)) return null;
  return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function saveConfig(ns, cfg) {
  ensureDir(path.dirname(configPath(ns)));
  fs.writeFileSync(configPath(ns), JSON.stringify(cfg, null, 2));
}

function loadQuota(ns) {
  const p = quotaPath(ns);
  if (!fs.existsSync(p)) return { bytes: 0, count: 0 };
  return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function recalcQuota(ns) {
  const dir = path.join(STORAGE_DIR, ns);
  let bytes = 0, count = 0;
  if (fs.existsSync(dir)) {
    for (const e of fs.readdirSync(dir)) {
      if (e.endsWith('.dat')) { bytes += fs.statSync(path.join(dir, e)).size; count++; }
    }
  }
  const q = loadQuota(ns);
  q.bytes = bytes; q.count = count;
  fs.writeFileSync(quotaPath(ns), JSON.stringify(q, null, 2));
  return q;
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

// --- Safe filesystem operations (no shell strings) ---

function safeRenameSync(oldPath, newPath) {
  fs.renameSync(oldPath, newPath);
}

function safeSymlinkSync(target, linkPath) {
  fs.symlinkSync(target, linkPath);
}

// --- Namespace creation / ensure ---

function ensureNamespace(ns, quotaBytes) {
  ensureDir(path.join(STORAGE_DIR, ns));
  ensureDir(path.join(STATE_DIR, ns));
  const p = quotaPath(ns);
  if (!fs.existsSync(p)) {
    const q = { bytes: 0, count: 0, quotaBytes: quotaBytes || 10485760, maxObjects: 10000 };
    fs.writeFileSync(p, JSON.stringify(q, null, 2));
    info('created namespace ' + ns + ' (quota: ' + q.quotaBytes + ' bytes)');
  }
}

// --- Commands ---

function cmdDeploy(args) {
  const { flags, rest } = parseFlags(args);

  // All four flags are required; positional args rejected.
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
    die('unexpected positional arguments: ' + rest.join(' ') + '\nusage: skrynia deploy --repo <url> --commit <sha> --subdir <path> --namespace <name> [--builder IMAGE]');
  }

  if (!validNs(ns)) die('invalid namespace: ' + ns + ' (must match ' + NS_RE + ')');

  // Ensure namespace exists (auto-create with default quota on first deploy)
  ensureNamespace(ns);

  let builderImage = BUILDER_IMAGE;
  if (flags['builder']) {
    builderImage = flags['builder'];
  }

  info('deploying namespace=' + ns);
  info('  repo=' + repoUrl);
  info('  commit=' + commit);
  info('  subdir=' + subdir);
  info('  builder=' + builderImage);

  // 0. Early subdir validation (before any clone/network work)
  if (path.isAbsolute(subdir)) die('subdirectory must be relative, got: ' + subdir);
  if (subdir.includes('..')) die('subdirectory must not contain ..: ' + subdir);
  if (subdir.includes('\0')) die('subdirectory must not contain null bytes');
  if (subdir === '') die('subdirectory must not be empty');

  // 1. Clone to temp workspace
  const workDir = fs.mkdtempSync('/tmp/skrynia-build-');
  let buildSucceeded = false;
  try {
    info('cloning repository...');
    execFileSync('git', ['clone', '--quiet', repoUrl, path.join(workDir, 'repo')], { stdio: 'inherit' });
    execFileSync('git', ['-C', path.join(workDir, 'repo'), 'checkout', '--quiet', commit], { stdio: 'inherit' });

    // 2. Validate subdirectory stays inside clone
    const repoDir = path.join(workDir, 'repo');
    const appDir = path.resolve(repoDir, subdir);
    const repoReal = fs.realpathSync(repoDir);
    const appReal = fs.realpathSync(appDir);
    if (!appReal.startsWith(repoReal + path.sep) && appReal !== repoReal) {
      die('subdirectory escapes repository: ' + subdir);
    }
    if (!fs.existsSync(appDir)) die('subdirectory not found: ' + subdir);

    // 3. Build into a staging area (not the final release dir)
    const stageDir = path.join(workDir, 'stage');
    ensureDir(stageDir);

    // Mount the whole repo so monorepo parent paths (../shared etc) work
    // but set working directory to the app subdirectory.
    // Repo is mounted RW so make build can write build/ inside it.
    const repoMount = repoDir;
    const absSubdir = path.relative(repoDir, appDir);

    info('building with container...');
    const dockerArgs = [
      'run', '--rm',
      '--network', 'none',
      '--tmpfs', '/tmp:size=256m',
      '-v', repoMount + ':/repo',
      '-v', stageDir + ':/stage',
      '-w', '/repo/' + absSubdir,
      builderImage,
      'make', 'build',
    ];
    try {
      execFileSync('docker', dockerArgs, { stdio: 'inherit' });
    } catch (e) {
      die('build failed (exit ' + e.status + ')');
    }

    // 4. Validate build output (no symlinks, no special files)
    const buildOutput = path.join(appDir, 'build');
    if (!fs.existsSync(buildOutput)) {
      die('build output directory not found: build/');
    }

    // Copy validated output from build/ into stage
    fs.cpSync(buildOutput, stageDir, { recursive: true });

    const issues = validateBuildOutput(stageDir);
    if (issues.length > 0) {
      die('invalid build output:\n  ' + issues.join('\n  '));
    }

    buildSucceeded = true;

    // 5. Create release directory only after successful build + validation
    // Unique release ID with millisecond + random suffix for sub-second uniqueness.
    const now = Date.now();
    const ts = new Date(now).toISOString().replace(/[^0-9]/g, '').slice(0, 14);
    const rand = crypto.randomBytes(3).toString('hex');
    const releaseId = ts + '-' + rand;
    const relDir = path.join(RELEASES_DIR, ns, releaseId);
    ensureDir(path.dirname(relDir));

    // Atomic: stage -> release dir (rename is atomic on same filesystem)
    safeRenameSync(stageDir, relDir);

    // 6. Atomic activation
    const link = currentLink(ns);
    ensureDir(path.dirname(link));
    const tmpLink = link + '.tmp.' + process.pid;
    try { fs.unlinkSync(tmpLink); } catch {}
    safeSymlinkSync(relDir, tmpLink);
    safeRenameSync(tmpLink, link);

    // 7. Save deployment config with currentReleaseId
    const cfg = loadConfig(ns) || {};
    Object.assign(cfg, {
      namespace: ns,
      repo: repoUrl,
      commit,
      subdir,
      builder: builderImage,
      currentReleaseId: releaseId,
      deployedAt: new Date().toISOString(),
    });
    saveConfig(ns, cfg);

    info('activated release ' + releaseId + ' for /a/' + ns + '/');

    // 8. Prune old releases (keep last 3)
    const relsDir = path.join(RELEASES_DIR, ns);
    const releases = fs.readdirSync(relsDir)
      .filter(d => d !== 'current' && fs.statSync(path.join(relsDir, d)).isDirectory())
      .sort();
    if (releases.length > 3) {
      for (const old of releases.slice(0, releases.length - 3)) {
        rmrfDir(path.join(relsDir, old));
        info('pruned old release ' + old);
      }
    }
  } finally {
    // Always clean workspace; if build failed, stage dir is removed by rmrfDir
    rmrfDir(workDir);
  }
}

function cmdUndeploy(args) {
  if (args.length < 1) {
    die('usage: skrynia undeploy <namespace>');
  }
  const ns = args[0];
  if (!validNs(ns)) die('invalid namespace: ' + ns);

  info('undeploying namespace=' + ns);

  // Remove releases
  const relDir = path.join(RELEASES_DIR, ns);
  if (fs.existsSync(relDir)) {
    rmrfDir(relDir);
    info('removed releases');
  }

  // Always remove stored data
  const storageNs = path.join(STORAGE_DIR, ns);
  if (fs.existsSync(storageNs)) {
    rmrfDir(storageNs);
    info('removed stored data');
  }

  // Always remove namespace state
  const stateNs = path.join(STATE_DIR, ns);
  if (fs.existsSync(stateNs)) {
    rmrfDir(stateNs);
    info('removed namespace state');
  }

  info('undeploy complete for /a/' + ns + '/');
}

function cmdRollback(args) {
  if (args.length < 1) {
    die('usage: skrynia rollback <namespace> [release-id]');
  }
  const ns = args[0];
  if (!validNs(ns)) die('invalid namespace: ' + ns);
  const target = args[1];

  const relBase = path.join(RELEASES_DIR, ns);
  if (!fs.existsSync(relBase)) die('no releases for namespace ' + ns);

  const releases = fs.readdirSync(relBase)
    .filter(d => d !== 'current' && fs.statSync(path.join(relBase, d)).isDirectory())
    .sort();

  if (releases.length === 0) die('no releases for namespace ' + ns);

  let rollbackTo;
  if (target) {
    if (!releases.includes(target)) die('release ' + target + ' not found');
    rollbackTo = target;
  } else {
    if (releases.length < 2) die('only one release exists, nothing to rollback to');
    rollbackTo = releases[releases.length - 2];
  }

  const link = currentLink(ns);
  const tmpLink = link + '.tmp.' + process.pid;
  const targetDir = path.join(relBase, rollbackTo);
  try { fs.unlinkSync(tmpLink); } catch {}
  safeSymlinkSync(targetDir, tmpLink);
  safeRenameSync(tmpLink, link);

  // Update config with rollback target
  const cfg = loadConfig(ns);
  if (cfg) {
    cfg.currentReleaseId = rollbackTo;
    cfg.lastRollbackAt = new Date().toISOString();
    saveConfig(ns, cfg);
  }

  info('rolled back /a/' + ns + '/ to release ' + rollbackTo);
}

function cmdReleases(args) {
  if (args.length < 1) die('usage: skrynia releases <namespace>');
  const ns = args[0];
  if (!validNs(ns)) die('invalid namespace: ' + ns);
  const relBase = path.join(RELEASES_DIR, ns);
  if (!fs.existsSync(relBase)) die('no releases for namespace ' + ns);

  const current = fs.existsSync(currentLink(ns))
    ? fs.basename(fs.readlinkSync(currentLink(ns)))
    : '(none)';

  const releases = fs.readdirSync(relBase)
    .filter(d => d !== 'current' && fs.statSync(path.join(relBase, d)).isDirectory())
    .sort();

  console.log('Namespace: ' + ns);
  console.log('Current:   ' + current);
  console.log('Releases:');
  for (const r of releases) {
    const marker = r === current ? ' <-- current' : '';
    console.log('  ' + r + marker);
  }
}

function cmdInspect(args) {
  if (args.length < 1) die('usage: skrynia inspect <namespace>');
  const ns = args[0];
  if (!validNs(ns)) die('invalid namespace: ' + ns);
  const cfg = loadConfig(ns);
  if (!cfg) die('namespace ' + ns + ' not found');
  console.log(JSON.stringify(cfg, null, 2));
}

function cmdNsCreate(args) {
  if (args.length < 1) die('usage: skrynia ns create <namespace> [--quota BYTES]');
  const ns = args[0];
  if (!validNs(ns)) die('invalid namespace: ' + ns + ' (must match ' + NS_RE + ')');
  ensureDir(path.join(STORAGE_DIR, ns));
  ensureDir(path.join(STATE_DIR, ns));

  const qidx = args.indexOf('--quota');
  const quotaBytes = qidx !== -1 ? parseInt(args[qidx + 1], 10) : 10485760;
  const p = quotaPath(ns);
  if (!fs.existsSync(p)) {
    const q = { bytes: 0, count: 0, quotaBytes, maxObjects: 10000 };
    fs.writeFileSync(p, JSON.stringify(q, null, 2));
    info('created namespace ' + ns + ' (quota: ' + quotaBytes + ' bytes)');
  } else {
    info('namespace ' + ns + ' already exists (quota preserved)');
  }
}

function cmdNsRemove(args) {
  if (args.length < 1) die('usage: skrynia ns remove <namespace>');
  const ns = args[0];
  if (!validNs(ns)) die('invalid namespace: ' + ns);
  rmrfDir(path.join(STORAGE_DIR, ns));
  rmrfDir(path.join(STATE_DIR, ns));
  info('removed namespace ' + ns);
}

function cmdNsInspect(args) {
  if (args.length < 1) die('usage: skrynia ns inspect <namespace>');
  const ns = args[0];
  if (!validNs(ns)) die('invalid namespace: ' + ns);
  const q = recalcQuota(ns);
  const cfg = loadConfig(ns);
  console.log(JSON.stringify({ namespace: ns, quota: q, deployment: cfg || null }, null, 2));
}

function cmdNsList() {
  if (!fs.existsSync(STORAGE_DIR)) { console.log('No namespaces.'); return; }
  const entries = fs.readdirSync(STORAGE_DIR).filter(e => {
    try { return fs.statSync(path.join(STORAGE_DIR, e)).isDirectory(); }
    catch { return false; }
  });
  if (entries.length === 0) { console.log('No namespaces.'); return; }
  for (const ns of entries.sort()) {
    const q = recalcQuota(ns);
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
    '  skrynia undeploy <ns>\n' +
    '                        Remove app, releases, namespace data and state\n' +
    '  skrynia rollback <ns> [release-id]\n' +
    '                        Rollback to previous or specified release\n' +
    '  skrynia releases <ns> List releases for a namespace\n' +
    '  skrynia inspect <ns>  Show deployment config for a namespace\n' +
    '\n  skrynia ns create <ns> [--quota BYTES]\n' +
    '                        Create a namespace with quota\n' +
    '  skrynia ns remove <ns>\n' +
    '                        Delete namespace and all its data\n' +
    '  skrynia ns inspect <ns>\n' +
    '                        Show namespace usage and config\n' +
    '  skrynia ns list       List all namespaces with usage\n' +
    '\n  skrynia help          Show this help\n' +
    '\nExamples:\n' +
    '  skrynia deploy --repo git@github.com:myorg/myapp.git --commit abc123def --subdir . --namespace myapp\n' +
    '  skrynia deploy --repo git@github.com:myorg/mono.git --commit def456ghi --subdir frontend --namespace myapp\n' +
    '  skrynia undeploy myapp\n' +
    '  skrynia rollback myapp\n' +
    '  skrynia releases myapp\n' +
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
