#!/usr/bin/env node
'use strict';

const fs = require('fs');

console.log('=== Cleaning ===');

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
