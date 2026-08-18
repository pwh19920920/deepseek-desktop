#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');
const path = require('path');

console.log('=== DeepSeek Harness Desktop - Build ===');

try {
  // Step 1: Fetch Node.js
  console.log('[1/3] Fetching Node.js...');
  execSync('node scripts/fetch-node.js', { stdio: 'inherit' });
  
  // Step 2: Bundle dsh
  console.log('[2/3] Bundling dsh dependencies...');
  execSync('cargo build --manifest-path apps/desktop/src-tauri/Cargo.toml', { stdio: 'inherit' });
  
  // Step 3: Build frontend
  console.log('[3/3] Building Tauri app...');
  process.chdir('apps/desktop');
  execSync('pnpm tauri build', { stdio: 'inherit' });
  
  console.log('=== Build complete ===');
} catch (error) {
  console.error('Build failed:', error.message);
  process.exit(1);
}
