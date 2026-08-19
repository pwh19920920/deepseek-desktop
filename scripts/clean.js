#!/usr/bin/env node
'use strict';

const fs = require('fs');

console.log('=== Cleaning ===');

const dirs = [
  'dsh-app-desktop/src-tauri/target',
  'dsh-app-desktop/src-tauri/gen',
  'dsh-app-desktop/src-tauri/resources',
  'dsh-app-desktop/src-tauri/binaries',
  'dsh-app-desktop/dist',
  'dsh-app-desktop/node_modules',
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