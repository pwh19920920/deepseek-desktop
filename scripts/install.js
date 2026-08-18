#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');
const path = require('path');

console.log('=== Installing dependencies ===');

// Change to apps/desktop directory for pnpm install
const desktopDir = path.join(__dirname, '..', 'apps', 'desktop');
const originalCwd = process.cwd();

try {
  process.chdir(desktopDir);
  execSync('pnpm install --ignore-scripts', { stdio: 'inherit' });
} catch (e) {
  console.log('\n[warn] pnpm install had issues, continuing...');
} finally {
  process.chdir(originalCwd);
}

console.log('\n=== Install complete ===');
