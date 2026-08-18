#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');
const path = require('path');

const args = process.argv.slice(2);
const platform = args[0]; // 'macos', 'windows', 'linux', or 'all'

// Auto-detect current platform
const CURRENT_PLATFORM = {
  'darwin-arm64': 'macos',
  'darwin-x64': 'macos',
  'win32-x64': 'windows',
  'linux-x64': 'linux',
}[[process.platform, process.arch].join('-')] || 'macos';

// Maps platform name → Rust target triple
const PLATFORM_TARGET = {
  'macos': 'aarch64-apple-darwin',
  'windows': 'x86_64-pc-windows-msvc',
  'linux': 'x86_64-unknown-linux-gnu',
};

console.log('=== Building ===');

// Determine which Node.js binaries to fetch
const targetsToFetch = platform === 'all'
  ? ['macos', 'windows', 'linux']
  : [platform || CURRENT_PLATFORM];

for (const plat of targetsToFetch) {
  const triple = PLATFORM_TARGET[plat];
  console.log(`\n=== Fetching Node.js for ${plat} (${triple}) ===`);
  execSync('node "' + path.join(__dirname, 'fetch-node.js') + '" ' + triple, { stdio: 'inherit' });
}

// Build frontend with Vite (in apps/desktop context)
console.log('\n=== Building frontend ===');
execSync('pnpm --filter @dsh/desktop exec vite build', { stdio: 'inherit' });

// Build for each target
process.chdir('apps/desktop');
let targets = [];
if (platform === 'all') {
  targets = Object.values(PLATFORM_TARGET).map(t => `--target ${t}`);
} else {
  const triple = PLATFORM_TARGET[platform || CURRENT_PLATFORM];
  if (!triple) {
    console.error(`Unknown platform: ${platform}`);
    console.log('Supported platforms: macos, windows, linux, all');
    process.exit(1);
  }
  targets = [`--target ${triple}`];
}

for (const target of targets) {
  console.log(`\n=== Building for ${target.replace('--target ', '')} ===`);
  execSync(`pnpm tauri build ${target}`, { stdio: 'inherit' });
}

console.log('\n=== Build complete ===');
