#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');
const path = require('path');

const args = process.argv.slice(2);
const command = args[0];

const commands = {
  install: () => {
    console.log('=== Installing dependencies ===');
    execSync('pnpm install', { stdio: 'inherit' });
    console.log('\n=== Fetching Node.js ===');
    execSync('node scripts/fetch-node.js', { stdio: 'inherit' });
    console.log('\n=== Install complete ===');
  },
  build: () => {
    console.log('=== Building ===');
    execSync('node scripts/fetch-node.js', { stdio: 'silent' });
    process.chdir('apps/desktop');
    execSync('pnpm tauri build', { stdio: 'inherit' });
    console.log('=== Build complete ===');
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
  console.log('Usage: pnpm <command>');
  console.log('');
  console.log('Commands:');
  console.log('  install  - Install dependencies and fetch Node.js');
  console.log('  build    - Build for production');
  console.log('  dev      - Start development server');
  console.log('  clean    - Clean build artifacts');
  process.exit(1);
}

commands[command]();
