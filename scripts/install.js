#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');

console.log('=== Installing dependencies ===');

try {
  execSync('pnpm install', { stdio: 'inherit' });
} catch (e) {
  console.log('\n[warn] pnpm install had issues, continuing...');
}

console.log('\n=== Fetching Node.js ===');
execSync('node scripts/fetch-node.js', { stdio: 'inherit' });

console.log('\n=== Install complete ===');
