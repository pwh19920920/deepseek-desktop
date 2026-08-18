#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');
const path = require('path');

const args = process.argv.slice(2);
const command = args[0];
const platformArg = args[1]; // 'macos', 'windows', 'linux', or 'all'

// Platform mapping
const PLATFORM_MAP = {
  'macos': '--target aarch64-apple-darwin',
  'windows': '--target x86_64-pc-windows-msvc',
  'linux': '--target x86_64-unknown-linux-gnu',
  'all': ''
};

const commands = {
  install: () => {
    console.log('=== Installing dependencies ===');
    try {
      execSync('pnpm install', { stdio: 'inherit' });
    } catch (e) {
      console.log('\n[warn] pnpm install had issues, continuing...');
    }
    console.log('\n=== Fetching Node.js for current platform ===');
    execSync('node scripts/fetch-node.js', { stdio: 'inherit' });
    console.log('\n=== Install complete ===');
  },
  build: () => {
    console.log('=== Building ===');
    
    // Fetch Node.js for current platform
    execSync('node scripts/fetch-node.js', { stdio: 'silent' });
    
    // Determine target platforms
    let targets = [];
    if (platformArg && platformArg !== 'all') {
      const target = PLATFORM_MAP[platformArg];
      if (!target) {
        console.error(`Unknown platform: ${platformArg}`);
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
  },
  dev: () => {
    console.log('=== Starting dev server ===');
    process.chdir('apps/desktop');
    execSync('pnpm dev', { stdio: 'inherit' });
  },
  clean: () => {
    console.log('=== Cleaning ===');
    const fs = require('fs');
    const dirs = [
      'apps/desktop/src-tauri/target',
      'apps/desktop/src-tauri/gen',
      '.tmp'
    ];
    for (const dir of dirs) {
      if (fs.existsSync(dir)) {
        fs.rmSync(dir, { recursive: true, force: true });
        console.log(`  ✅ ${dir}/ cleaned`);
      }
    }
    console.log('\n=== Clean complete ===');
  }
};

if (!command || !commands[command]) {
  console.log('Usage: pnpm <command> [platform]');
  console.log('');
  console.log('Commands:');
  console.log('  install          - Install dependencies and fetch Node.js');
  console.log('  build [platform] - Build for production');
  console.log('                     platforms: macos, windows, linux, all');
  console.log('  dev              - Start development server');
  console.log('  clean            - Clean build artifacts');
  console.log('');
  console.log('Examples:');
  console.log('  pnpm build           # Build for current platform');
  console.log('  pnpm build macos     # Build for macOS only');
  console.log('  pnpm build all       # Build for all platforms');
  process.exit(1);
}

commands[command]();
