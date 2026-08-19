#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

console.log('=== Installing dependencies ===');

// Change to dsh-app-desktop directory for pnpm install
const desktopDir = path.join(__dirname, '..', 'dsh-app-desktop');
const originalCwd = process.cwd();

try {
  process.chdir(desktopDir);
  execSync('pnpm install --ignore-scripts', { stdio: 'inherit' });
} catch (e) {
  console.log('\n[warn] pnpm install had issues, continuing...');
} finally {
  process.chdir(originalCwd);
}

// Force reinstall dshmarket so version changes take effect.
// pnpm install skips the update when the locked version already satisfies
// the new range (e.g. ^1.12.0 is satisfied by 1.14.1).  We extract the
// pinned version from the range spec and use pnpm add to force the lockfile
// update.
console.log('\n=== Reinstalling dshmarket ===');
const dshmarketVersion = readDshmarketVersion(desktopDir);
if (dshmarketVersion) {
  try {
    process.chdir(desktopDir);
    // Strip range prefix (^, ~, >=, etc.) to get the pinned version
    const pinned = dshmarketVersion.replace(/^[~^>=<]*/, '');
    execSync(`pnpm add dshmarket@${pinned} --ignore-scripts`, { stdio: 'inherit' });
  } catch (e) {
    console.log('[warn] failed to reinstall dshmarket, continuing...');
  } finally {
    process.chdir(originalCwd);
  }
}

console.log('\n=== Configuring profile with dshmarket ===');

const dshHome = process.env.DSH_HOME || path.join(os.homedir(), '.dsh');
const profileDir = path.join(dshHome, 'profiles', 'web');
const pkgPath = path.join(profileDir, 'package.json');

// Find bundled dshmarket
const bundledDshmarket = findDshmarket(desktopDir);
if (!bundledDshmarket) {
  console.log('[warn] dshmarket not found in node_modules, skipping profile setup');
  process.exit(0);
}

// Ensure profile directory exists
fs.mkdirSync(profileDir, { recursive: true });

// Read or create package.json
let pkg = {};
if (fs.existsSync(pkgPath)) {
  try {
    pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
  } catch (e) {
    console.log('[warn] failed to parse profile package.json, recreating');
    pkg = {};
  }
}

// Ensure name
if (!pkg.name) {
  pkg.name = 'dsh-profile-web';
  pkg.private = true;
}

// Ensure dependencies
if (!pkg.dependencies) {
  pkg.dependencies = {};
}
if (!pkg.dependencies.dshmarket) {
  pkg.dependencies.dshmarket = '*';
}

// Ensure dsh.profile.bundles
if (!pkg.dsh) {
  pkg.dsh = { profile: { bundles: [] } };
}
if (!pkg.dsh.profile) {
  pkg.dsh.profile = { bundles: [] };
}
if (!pkg.dsh.profile.bundles) {
  pkg.dsh.profile.bundles = [];
}
if (!pkg.dsh.profile.bundles.includes('dshmarket')) {
  // Ensure base bundles are present
  for (const bundle of ['@deepseek-ai/dsh-base', '@deepseek-ai/dsh-web-app']) {
    if (!pkg.dsh.profile.bundles.includes(bundle)) {
      pkg.dsh.profile.bundles.push(bundle);
    }
  }
  pkg.dsh.profile.bundles.push('dshmarket');
}

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
console.log('  profile package.json updated');

// Ensure cordis.patch.yml exists
const patchPath = path.join(profileDir, 'cordis.patch.yml');
if (!fs.existsSync(patchPath)) {
  fs.writeFileSync(patchPath, [
    '# Your patch layer for this dsh profile, applied after every bundle layer:',
    '# a top-level YAML array of loader patch entries (id-targeted config',
    '# overrides, disables, and insert lists; `!!js` expressions allowed).',
    '[]',
    ''
  ].join('\n'));
}

// Ensure pnpm-workspace.yaml exists
const workspacePath = path.join(profileDir, 'pnpm-workspace.yaml');
if (!fs.existsSync(workspacePath)) {
  fs.writeFileSync(workspacePath, [
    'packages:',
    '  - .',
    '',
    'nodeLinker: hoisted',
    'autoInstallPeers: false',
    ''
  ].join('\n'));
}

// Create/update symlink
const linkDir = path.join(profileDir, 'node_modules');
fs.mkdirSync(linkDir, { recursive: true });
const linkPath = path.join(linkDir, 'dshmarket');

// Remove old symlink or directory
try {
  if (fs.existsSync(linkPath)) {
    const stat = fs.lstatSync(linkPath);
    if (stat.isSymbolicLink()) {
      fs.unlinkSync(linkPath);
    } else {
      fs.rmSync(linkPath, { recursive: true, force: true });
    }
  }
} catch (e) {
  // ignore
}

// Create symlink (use junction on Windows)
const target = fs.realpathSync(bundledDshmarket);
if (process.platform === 'win32') {
  fs.symlinkSync(target, linkPath, 'junction');
} else {
  fs.symlinkSync(target, linkPath);
}
console.log(`  symlinked dshmarket → ${target}`);

console.log('\n=== Install complete ===');

/**
 * Find the bundled dshmarket directory.
 */
function findDshmarket(desktopDir) {
  const candidates = [
    path.join(desktopDir, 'node_modules', 'dshmarket'),
    path.join(__dirname, '..', 'node_modules', 'dshmarket'),
  ];
  for (const dir of candidates) {
    const pkg = path.join(dir, 'package.json');
    if (fs.existsSync(pkg)) {
      return dir;
    }
  }
  return null;
}

/**
 * Read the dshmarket version from the desktop app's package.json.
 */
function readDshmarketVersion(desktopDir) {
  const pkgPath = path.join(desktopDir, 'package.json');
  if (!fs.existsSync(pkgPath)) return null;
  try {
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
    return pkg.dependencies?.dshmarket || null;
  } catch {
    return null;
  }
}
