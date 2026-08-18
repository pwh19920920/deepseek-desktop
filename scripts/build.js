#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');

const args = process.argv.slice(2);
const platform = args[0]; // 'macos', 'windows', 'linux', or 'all'

const PLATFORM_MAP = {
  'macos': '--target aarch64-apple-darwin',
  'windows': '--target x86_64-pc-windows-msvc',
  'linux': '--target x86_64-unknown-linux-gnu',
  'all': ''
};

console.log('=== Building ===');

// Fetch Node.js for current platform
execSync('node scripts/fetch-node.js', { stdio: 'silent' });

// Determine targets
let targets = [];
if (platform && platform !== 'all') {
  const target = PLATFORM_MAP[platform];
  if (!target) {
    console.error(`Unknown platform: ${platform}`);
    console.log('Supported platforms: macos, windows, linux, all');
    process.exit(1);
  }
  targets = [target];
} else {
  // Default: build for all platforms
  targets = Object.values(PLATFORM_MAP).filter(t => t);
}

// Build for each target
process.chdir('apps/desktop');
for (const target of targets) {
  console.log(`\n=== Building for ${target || 'current platform'} ===`);
  const cmd = target ? `pnpm tauri build ${target}` : 'pnpm tauri build';
  execSync(cmd, { stdio: 'inherit' });
}

console.log('\n=== Build complete ===');
