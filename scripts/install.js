#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

console.log('=== Installing dependencies ===');

// Change to apps/desktop directory for pnpm install
const desktopDir = path.join(__dirname, '..', 'apps', 'desktop');
const originalCwd = process.cwd();

try {
  process.chdir(desktopDir);
  execSync('pnpm install', { stdio: 'inherit' });
} catch (e) {
  console.log('\n[warn] pnpm install had issues, continuing...');
} finally {
  process.chdir(originalCwd);
}

console.log('\n=== Fetching Node.js ===');
execSync('node ' + path.join(__dirname, 'fetch-node.js'), { stdio: 'inherit' });

console.log('\n=== Install complete ===');
