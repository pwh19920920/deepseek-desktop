#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const ROOT = path.resolve(__dirname, '..');
const DESKTOP_DIR = path.join(ROOT, 'dsh-app-desktop');
const DSH_PKG = path.join(DESKTOP_DIR, 'node_modules', '@deepseek-ai', 'dsh');
const DIST_DIR = path.join(DESKTOP_DIR, 'dist');
const BINARIES_DIR = path.join(DESKTOP_DIR, 'src-tauri', 'binaries');
const NODE_BINARY = path.join(BINARIES_DIR, 'node-aarch64-apple-darwin');

console.log('=== Starting dev server ===');

// 1. Check pnpm install
if (!fs.existsSync(DSH_PKG)) {
  console.error('\n❌ @deepseek-ai/dsh not found — did you run `pnpm install` first?');
  console.error('   Run: pnpm install\n');
  process.exit(1);
}

// 2. Check / download Node.js binary
if (!fs.existsSync(NODE_BINARY)) {
  console.log('⚠️  Node.js binary not found, downloading...');
  try {
    execSync(`node "${path.join(ROOT, 'scripts', 'fetch-node.js')}" aarch64-apple-darwin`, {
      cwd: ROOT,
      stdio: 'inherit',
    });
  } catch (e) {
    console.error('\n❌ Failed to download Node.js binary.');
    console.error('   You can manually run: node scripts/fetch-node.js aarch64-apple-darwin\n');
    process.exit(1);
  }
}

// 3. Check frontend build
if (!fs.existsSync(path.join(DIST_DIR, 'index.html'))) {
  console.log('⚠️  Frontend dist not found, building...');
  try {
    execSync('npx vite build', { cwd: DESKTOP_DIR, stdio: 'inherit' });
  } catch (e) {
    console.error('\n❌ Frontend build failed.');
    console.error('   You can manually run: cd dsh-app-desktop && npx vite build\n');
    process.exit(1);
  }
}

// 4. Start dev
process.chdir(DESKTOP_DIR);
execSync('pnpm dev', { stdio: 'inherit' });