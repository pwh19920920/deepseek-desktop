#!/usr/bin/env node
'use strict';

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Platform detection
const platform = process.platform; // 'darwin', 'linux', 'win32'
const arch = process.arch; // 'arm64', 'x64'

// Node.js version to download
const NODE_VERSION = 'v24.9.0';
const NODE_DIST_URL = `https://nodejs.org/dist/${NODE_VERSION}`;

// Determine archive name and type
let archiveName, extractCmd;
if (platform === 'win32') {
  archiveName = `node-${NODE_VERSION}-win-${arch}.zip`;
  extractCmd = 'unzip';
} else if (platform === 'darwin') {
  archiveName = `node-${NODE_VERSION}-darwin-${arch}.tar.gz`;
  extractCmd = 'tar';
} else if (platform === 'linux') {
  archiveName = `node-${NODE_VERSION}-linux-${arch}.tar.gz`;
  extractCmd = 'tar';
} else {
  console.error(`Unsupported platform: ${platform}`);
  process.exit(1);
}

// Resolve paths
const scriptDir = __dirname;
const projectRoot = path.resolve(scriptDir, '..');
const binariesDir = path.join(projectRoot, 'apps', 'desktop', 'src-tauri', 'binaries');
const downloadDir = path.join(projectRoot, '.tmp', 'node-download');

console.log(`[fetch-node] target: ${NODE_VERSION} for ${platform}-${arch}`);
console.log(`[fetch-node] archive: ${archiveName}`);

// Get Tauri target triple from rustc
let targetTriple = '';
try {
  const rustcOutput = execSync('rustc -vV', { encoding: 'utf8' });
  const match = rustcOutput.match(/^host:\s*(.+)$/m);
  if (match) {
    targetTriple = match[1];
  }
} catch (e) {
  console.warn('[fetch-node] warning: could not determine target triple, using default');
}

const binName = platform === 'win32' 
  ? `node-${targetTriple || 'x64'}.exe` 
  : `node-${targetTriple || `${platform}-${arch}`}`;

// Check if already exists
const targetPath = path.join(binariesDir, binName);
if (fs.existsSync(targetPath)) {
  console.log(`[fetch-node] ${binName} already present, skipping.`);
  process.exit(0);
}

// Prepare directories
fs.mkdirSync(downloadDir, { recursive: true });
fs.mkdirSync(binariesDir, { recursive: true });

const archivePath = path.join(downloadDir, archiveName);
const extractDir = path.join(downloadDir, 'extracted');
fs.mkdirSync(extractDir, { recursive: true });

// Download function
function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (response) => {
      if (response.statusCode !== 302 && response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }
      if (response.statusCode === 302) {
        download(response.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      response.pipe(file);
      file.on('finish', () => {
        file.close();
        resolve();
      });
    }).on('error', (err) => {
      fs.unlinkSync(dest);
      reject(err);
    });
  });
}

// Main execution
async function main() {
  const artifactUrl = `${NODE_DIST_URL}/${archiveName}`;
  console.log(`[fetch-node] downloading ${artifactUrl}`);
  
  await download(artifactUrl, archivePath);
  console.log(`[fetch-node] downloading complete`);
  
  // Extract
  console.log(`[fetch-node] extracting...`);
  if (platform === 'win32') {
    execSync(`unzip -q "${archivePath}" -d "${extractDir}"`);
    const srcPath = path.join(extractDir, `node-${NODE_VERSION}-win-${arch}`, 'node.exe');
    fs.copyFileSync(srcPath, targetPath);
  } else {
    execSync(`tar xzf "${archivePath}" -C "${extractDir}"`);
    const srcPath = path.join(extractDir, `node-${NODE_VERSION}-${platform}-${arch}`, 'bin', 'node');
    fs.copyFileSync(srcPath, targetPath);
    fs.chmodSync(targetPath, 0o755);
  }
  
  // Cleanup
  fs.rmSync(downloadDir, { recursive: true, force: true });
  
  console.log(`[fetch-node] done: ${targetPath}`);
}

main().catch((err) => {
  console.error('[fetch-node] error:', err.message);
  process.exit(1);
});
