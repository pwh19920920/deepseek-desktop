#!/usr/bin/env node
'use strict';

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

/**
 * Fetch portable Node.js for a given target platform and arch.
 *
 * Usage:
 *   node scripts/fetch-node.js                    # current platform
 *   node scripts/fetch-node.js aarch64-apple-darwin
 *   node scripts/fetch-node.js x86_64-pc-windows-msvc
 *   node scripts/fetch-node.js x86_64-unknown-linux-gnu
 *
 * Downloads to: dsh-app-desktop/src-tauri/binaries/node-{target}
 */

const NODE_VERSION = 'v24.9.0';

const PLATFORM_MAP = {
  darwin: {
    'arm64':  { distro: 'darwin-arm64',   ext: '.tar.gz' },
    'x64':    { distro: 'darwin-x64',     ext: '.tar.gz' },
  },
  win32: {
    'x64':    { distro: 'win-x64',        ext: '.zip' },
    'arm64':  { distro: 'win-arm64',      ext: '.zip' },
  },
  linux: {
    'x64':    { distro: 'linux-x64',      ext: '.tar.xz' },
    'arm64':  { distro: 'linux-arm64',    ext: '.tar.xz' },
  },
};

// Map Rust target triples to Node.js distro names
const TRIPLE_TO_DISTRO = {
  'aarch64-apple-darwin':  { distro: 'darwin-arm64',  ext: '.tar.gz' },
  'x86_64-apple-darwin':   { distro: 'darwin-x64',    ext: '.tar.gz' },
  'x86_64-pc-windows-msvc':{ distro: 'win-x64',       ext: '.zip' },
  'armv7-unknown-linux-gnueabihf': { distro: 'linux-armv7l', ext: '.tar.xz' },
  'aarch64-unknown-linux-gnu':        { distro: 'linux-arm64', ext: '.tar.xz' },
  'x86_64-unknown-linux-gnu':         { distro: 'linux-x64',   ext: '.tar.xz' },
};

function getTargetTriple() {
  const args = process.argv.slice(2);
  if (args[0]) return args[0];

  // Auto-detect from current platform
  const plat = process.platform;
  const arch = process.arch;
  const map = PLATFORM_MAP[plat]?.[arch];
  if (!map) {
    console.error(`Unsupported platform: ${plat}-${arch}`);
    process.exit(1);
  }
  return map.distro;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (res) => {
      if (res.statusCode !== 200) {
        reject(new Error(`HTTP ${res.statusCode}: ${url}`));
        return;
      }
      res.pipe(file);
      file.on('finish', () => { file.close(); resolve(); });
    }).on('error', reject);
  });
}

function extractTarGz(tarFile, destDir) {
  execSync(`tar -xzf "${tarFile}" -C "${destDir}"`, { stdio: 'inherit' });
}

function extractZip(zipFile, destDir) {
  execSync(`unzip -qo "${zipFile}" -d "${destDir}"`, { stdio: 'inherit' });
}

function extractTarXz(tarFile, destDir) {
  execSync(`tar -xJf "${tarFile}" -C "${destDir}"`, { stdio: 'inherit' });
}

async function main() {
  const input = getTargetTriple();
  const isTriple = !['darwin-arm64', 'darwin-x64', 'win-x64', 'win-arm64', 'linux-x64', 'linux-arm64'].includes(input);

  let distro, ext;
  if (isTriple) {
    const info = TRIPLE_TO_DISTRO[input];
    if (!info) {
      console.error(`Unknown target triple: ${input}`);
      console.log('Supported: aarch64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu');
      process.exit(1);
    }
    distro = info.distro;
    ext = info.ext;
    console.log(`Fetching Node.js ${NODE_VERSION} for target: ${input} (${distro}${ext})`);
  } else {
    distro = input;
    ext = input.split('-')[1] === 'arm64' ? (input.startsWith('win') ? '.zip' : '.tar.gz') : '.tar.gz';
    console.log(`Fetching Node.js ${NODE_VERSION} for platform: ${distro}${ext}`);
  }

  const baseUrl = `https://nodejs.org/dist/${NODE_VERSION}/node-${NODE_VERSION}-${distro}${ext}`;
  const binariesDir = path.join(__dirname, '..', 'dsh-app-desktop', 'src-tauri', 'binaries');
  const tarFile = path.join(binariesDir, `node-${NODE_VERSION}-${distro}${ext}`);
  const outputName = `node-${input}`;
  const outputFile = path.join(binariesDir, outputName);

  fs.mkdirSync(binariesDir, { recursive: true });

  // Check if already exists
  if (fs.existsSync(outputFile)) {
    console.log(`✓ ${outputName} already exists, skipping download.`);
    return;
  }

  console.log(`Downloading ${baseUrl} ...`);

  if (!fs.existsSync(tarFile)) {
    await download(baseUrl, tarFile);
  }

  console.log('Extracting ...');

  // Extract and get the node binary
  const tmpDir = path.join(binariesDir, `.tmp-${input.replace(/[-/]/g, '_')}`);
  fs.mkdirSync(tmpDir, { recursive: true });

  if (ext === '.zip') {
    await new Promise((resolve, reject) => {
      execSync(`unzip -qo "${tarFile}" -d "${tmpDir}"`, { stdio: 'pipe' });
      resolve();
    });
  } else if (ext === '.tar.gz') {
    await new Promise((resolve, reject) => {
      execSync(`tar -xzf "${tarFile}" -C "${tmpDir}"`, { stdio: 'pipe' });
      resolve();
    });
  } else if (ext === '.tar.xz') {
    await new Promise((resolve, reject) => {
      execSync(`tar -xJf "${tarFile}" -C "${tmpDir}"`, { stdio: 'pipe' });
      resolve();
    });
  }

  // Find the node binary in extracted tree
  let nodeBinary = null;
  function findNode(dir) {
    for (const entry of fs.readdirSync(dir)) {
      const full = path.join(dir, entry);
      if (fs.statSync(full).isDirectory()) {
        const found = findNode(full);
        if (found) return found;
      } else if (entry === 'node' || entry === 'node.exe') {
        return full;
      }
    }
    return null;
  }
  nodeBinary = findNode(tmpDir);

  if (!nodeBinary) {
    // List what we got for debugging
    console.error('Extracted contents:');
    console.log(execSync(`find "${tmpDir}" -type f`, { encoding: 'utf-8' }));
    throw new Error('Could not find node binary in archive');
  }

  fs.copyFileSync(nodeBinary, outputFile);
  fs.chmodSync(outputFile, '755');

  // Cleanup
  fs.rmSync(tmpDir, { recursive: true, force: true });
  fs.unlinkSync(tarFile);

  console.log(`✓ Saved to ${outputFile}`);
}

main().catch((err) => {
  console.error('Error:', err.message);
  process.exit(1);
});
