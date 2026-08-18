#!/usr/bin/env node
'use strict';

const fs = require('fs');

console.log('=== Cleaning ===');

const dirs = [
  'apps/desktop/src-tauri/target',
  'apps/desktop/src-tauri/gen',
  'apps/desktop/src-tauri/resources',
  'apps/desktop/src-tauri/binaries',
  'apps/desktop/dist',
  'apps/desktop/node_modules',
  'resources/dsh',
  'node_modules',
  '.tmp',
];

for (const dir of dirs) {
  if (fs.existsSync(dir)) {
    fs.rmSync(dir, { recursive: true, force: true });
    console.log(`  ✅ ${dir}/ cleaned`);
  }
}

console.log('\n=== Clean complete ===');