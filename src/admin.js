#!/usr/bin/env node
'use strict';

const fs = require('fs');
const path = require('path');
const { execSync, execFileSync } = require('child_process');
const crypto = require('crypto');

const DATA_DIR = process.env.SKRYNIA_DATA_DIR || '/var/lib/skrynia';
const RELEASES_DIR = path.join(DATA_DIR, 'releases');
const STORAGE_DIR = path.join(DATA_DIR, 'storage');
const STATE_DIR = path.join(DATA_DIR, 'state');
const BUILDER_IMAGE = process.env.SKRYNIA_BUILDER_IMAGE || 'ghcr.io/ottojung/skrynia-builder:latest';

// --- Utilities ---

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function die(msg) {
  process.stderr.write(`skrynia: ${msg}\n`);
  process.exit(1);
}

function info(msg) {
  process.stderr.write(`skrynia: ${msg}\n`);
}

function currentLink(ns) {
  return path.join(RELEASES_DIR, ns, 'current');
}

function configPath(ns) {
  return path.join(STATE_DIR, ns, 'config.json');
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

function quotaPath(ns) {
  return path.join(STATE_DIR, ns, 'quota.json');
}

function loadQuota(ns) {
  const p = quotaPath(ns);
  if (!fs.existsSync(p)) return { bytes: 0, count: 0 };
  return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function recalcQuota(ns) {
  const dir = path.join(STORAGE_DIR, ns);
  let bytes = 0;
  let count = 0;
  if (fs.existsSync(dir)) {
    for (const e of fs.readdirSync(dir)) {
      if (e.endsWith('.dat')) {
        bytes += fs.statSync(path.join(dir, e)).size;
        count++;
      }
    }
  }
  const q = loadQuota(ns);
  q.bytes = bytes;
  q.count = count;
  fs.writeFileSync(quotaPath(ns), JSON.stringify(q, null, 2));
  return q;
}

function nsDir(ns) {
  return path.join(STORAGE_DIR, ns);
}

// --- Commands ---

function cmdDeploy(args) {
  if (args.length < 4) {
    die('usage: skrynia deploy <repo-url> <commit> <subdir> <namespace> [--builder IMAGE]');
  }
  const repoUrl = args[0];
  const commit = args[1];
  const subdir = args[2];
  const ns = args[3];

  let builderImage = BUILDER_IMAGE;
  const builderIdx = args.indexOf('--builder');
  if (builderIdx !== -1) {
    builderImage = args[builderIdx + 1];
    if (!builderImage) die('--builder requires a value');
  }

  info(`deploying namespace=${ns}`);
  info(`  repo=${repoUrl}`);
  info(`  commit=${commit}`);
  info(`  subdir=${subdir}`);
  info(`  builder=${builderImage}`);

  // 1. Clone repo to temp workspace
  const workDir = fs.mkdtempSync('/tmp/skrynia-build-');
  try {
    info('cloning repository...');
    execSync(`git clone --quiet ${repoUrl} ${workDir}/repo`, { stdio: 'inherit' });
    execSync(`git -C ${workDir}/repo checkout --quiet ${commit}`, { stdio: 'inherit' });

    const appDir = path.join(workDir, 'repo', subdir);
    if (!fs.existsSync(appDir)) die(`subdirectory not found: ${subdir}`);

    // 2. Build using container
    const releaseId = new Date().toISOString().replace(/[^0-9]/g, '').slice(0, 14);
    const relDir = path.join(RELEASES_DIR, ns, releaseId);
    ensureDir(relDir);

    info('building with container...');
    const buildCmd = [
      'docker', 'run', '--rm',
      '--network', 'none',
      '--read-only',
      '--tmpfs', '/tmp:size=256m',
      '-v', `${appDir}:/workspace`,
      '-w', '/workspace',
      builderImage,
      'make', 'build',
    ];
    try {
      execFileSync(buildCmd[0], buildCmd.slice(1), { stdio: 'inherit' });
    } catch (e) {
      die(`build failed (exit ${e.status})`);
    }

    // 3. Copy build output
    const buildOutput = path.join(appDir, 'build');
    if (!fs.existsSync(buildOutput)) {
      die('build output directory not found: build/');
    }
    execSync(`cp -r ${buildOutput}/. ${relDir}/`, { stdio: 'inherit' });

    // 4. Atomic activation
    const link = currentLink(ns);
    const linkDir = path.dirname(link);
    ensureDir(linkDir);

    const tmpLink = link + '.tmp';
    try { fs.unlinkSync(tmpLink); } catch {}
    fs.symlinkSync(relDir, tmpLink);
    fs.renameSync(tmpLink, link);

    // 5. Save deployment config
    saveConfig(ns, {
      namespace: ns,
      repo: repoUrl,
      commit,
      subdir,
      builder: builderImage,
      releaseId,
      deployedAt: new Date().toISOString(),
    });

    info(`activated release ${releaseId} for /a/${ns}/`);

    // List releases for rollback
    const releases = fs.readdirSync(path.join(RELEASES_DIR, ns))
      .filter(d => d !== 'current' && fs.statSync(path.join(RELEASES_DIR, ns, d)).isDirectory())
      .sort();
    if (releases.length > 3) {
      const toRemove = releases.slice(0, releases.length - 3);
      for (const old of toRemove) {
        const oldDir = path.join(RELEASES_DIR, ns, old);
        execSync(`rm -rf ${oldDir}`);
        info(`pruned old release ${old}`);
      }
    }
  } finally {
    execSync(`rm -rf ${workDir}`, { stdio: 'inherit' });
  }
}

function cmdUndeploy(args) {
  if (args.length < 1) {
    die('usage: skrynia undeploy <namespace>');
  }
  const ns = args[0];
  const preserveData = args.includes('--preserve-data');

  info(`undeploying namespace=${ns}${preserveData ? ' (preserving data)' : ''}`);

  // Stop any running service references
  const relDir = path.join(RELEASES_DIR, ns);
  if (fs.existsSync(relDir)) {
    execSync(`rm -rf ${relDir}`);
    info('removed releases');
  }

  if (!preserveData) {
    const storageNs = path.join(STORAGE_DIR, ns);
    if (fs.existsSync(storageNs)) {
      execSync(`rm -rf ${storageNs}`);
      info('removed stored data');
    }
    const stateNs = path.join(STATE_DIR, ns);
    if (fs.existsSync(stateNs)) {
      execSync(`rm -rf ${stateNs}`);
      info('removed namespace state');
    }
  }

  info(`undeploy complete for /a/${ns}/`);
}

function cmdRollback(args) {
  if (args.length < 1) {
    die('usage: skrynia rollback <namespace> [release-id]');
  }
  const ns = args[0];
  const target = args[1];

  const relBase = path.join(RELEASES_DIR, ns);
  if (!fs.existsSync(relBase)) {
    die(`no releases for namespace ${ns}`);
  }

  let releases = fs.readdirSync(relBase)
    .filter(d => d !== 'current' && fs.statSync(path.join(relBase, d)).isDirectory())
    .sort();

  if (releases.length === 0) {
    die(`no releases for namespace ${ns}`);
  }

  let rollbackTo;
  if (target) {
    if (!releases.includes(target)) die(`release ${target} not found`);
    rollbackTo = target;
  } else {
    // Rollback to previous (second most recent)
    if (releases.length < 2) die('only one release exists, nothing to rollback to');
    rollbackTo = releases[releases.length - 2];
  }

  const link = currentLink(ns);
  const tmpLink = link + '.tmp';
  const targetDir = path.join(relBase, rollbackTo);
  try { fs.unlinkSync(tmpLink); } catch {}
  fs.symlinkSync(targetDir, tmpLink);
  fs.renameSync(tmpLink, link);

  info(`rolled back /a/${ns}/ to release ${rollbackTo}`);
}

function cmdReleases(args) {
  if (args.length < 1) {
    die('usage: skrynia releases <namespace>');
  }
  const ns = args[0];
  const relBase = path.join(RELEASES_DIR, ns);
  if (!fs.existsSync(relBase)) {
    die(`no releases for namespace ${ns}`);
  }

  const current = fs.existsSync(currentLink(ns))
    ? fs.basename(fs.readlinkSync(currentLink(ns)))
    : '(none)';

  const releases = fs.readdirSync(relBase)
    .filter(d => d !== 'current' && fs.statSync(path.join(relBase, d)).isDirectory())
    .sort();

  console.log(`Namespace: ${ns}`);
  console.log(`Current:   ${current}`);
  console.log(`Releases:`);
  for (const r of releases) {
    const marker = r === current ? ' <-- current' : '';
    console.log(`  ${r}${marker}`);
  }
}

function cmdInspect(args) {
  if (args.length < 1) {
    die('usage: skrynia inspect <namespace>');
  }
  const ns = args[0];
  const cfg = loadConfig(ns);
  if (!cfg) {
    die(`namespace ${ns} not found`);
  }
  console.log(JSON.stringify(cfg, null, 2));
}

function cmdNsCreate(args) {
  if (args.length < 1) {
    die('usage: skrynia ns create <namespace> [--quota BYTES]');
  }
  const ns = args[0];
  const nsDirPath = path.join(STORAGE_DIR, ns);
  ensureDir(nsDirPath);
  ensureDir(path.join(STATE_DIR, ns));

  const quotaBytes = parseInt(args[args.indexOf('--quota') + 1] || '10485760', 10);
  const q = { bytes: 0, count: 0, quotaBytes };
  fs.writeFileSync(quotaPath(ns), JSON.stringify(q, null, 2));

  info(`created namespace ${ns} (quota: ${quotaBytes} bytes)`);
}

function cmdNsRemove(args) {
  if (args.length < 1) {
    die('usage: skrynia ns remove <namespace>');
  }
  const ns = args[0];
  const storageNs = path.join(STORAGE_DIR, ns);
  const stateNs = path.join(STATE_DIR, ns);
  if (fs.existsSync(storageNs)) execSync(`rm -rf ${storageNs}`);
  if (fs.existsSync(stateNs)) execSync(`rm -rf ${stateNs}`);
  info(`removed namespace ${ns}`);
}

function cmdNsInspect(args) {
  if (args.length < 1) {
    die('usage: skrynia ns inspect <namespace>');
  }
  const ns = args[0];
  const q = recalcQuota(ns);
  const cfg = loadConfig(ns);
  console.log(JSON.stringify({
    namespace: ns,
    quota: q,
    deployment: cfg || null,
  }, null, 2));
}

function cmdNsList() {
  if (!fs.existsSync(STORAGE_DIR)) {
    console.log('No namespaces.');
    return;
  }
  const entries = fs.readdirSync(STORAGE_DIR).filter(e => {
    try { return fs.statSync(path.join(STORAGE_DIR, e)).isDirectory(); }
    catch { return false; }
  });
  if (entries.length === 0) {
    console.log('No namespaces.');
    return;
  }
  for (const ns of entries.sort()) {
    const q = recalcQuota(ns);
    const cfg = loadConfig(ns);
    const deployed = cfg ? ` (repo=${cfg.commit?.slice(0, 8)}...)` : '';
    console.log(`${ns}: ${q.count} objects, ${q.bytes}/${q.quotaBytes} bytes${deployed}`);
  }
}

function cmdHelp() {
  console.log(`skrynia - Skrynia platform admin CLI

Usage:
  skrynia deploy <repo> <commit> <subdir> <ns> [--builder IMAGE]
                        Deploy app from git source
  skrynia undeploy <ns> [--preserve-data]
                        Remove app and namespace (deletes data by default)
  skrynia rollback <ns> [release-id]
                        Rollback to previous or specified release
  skrynia releases <ns> List releases for a namespace
  skrynia inspect <ns>  Show deployment config for a namespace

  skrynia ns create <ns> [--quota BYTES]
                        Create a namespace with quota
  skrynia ns remove <ns>
                        Delete namespace and all its data
  skrynia ns inspect <ns>
                        Show namespace usage and config
  skrynia ns list       List all namespaces with usage

  skrynia help          Show this help

Examples:
  skrynia deploy git@github.com:myorg/myapp.git abc123def . myapp
  skrynia deploy git@github.com:myorg/mono.git abc123 frontend myapp
  skrynia undeploy myapp
  skrynia rollback myapp
  skrynia releases myapp
  skrynia ns list`);
}

// --- Main ---

const args = process.argv.slice(2);
if (args.length === 0 || args[0] === 'help') {
  cmdHelp();
  process.exit(0);
}

const cmd = args[0];
const cmdArgs = args.slice(1);

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
    die(`unknown command: ${cmd}. Run 'skrynia help' for usage.`);
}
