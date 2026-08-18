#!/usr/bin/env node
'use strict';

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('=== DeepSeek Harness Desktop - Clean ===');

// Clean Rust target
const targetDir = path.join('apps', 'desktop', 'src-tauri', 'target');
if (fs.existsSync(targetDir)) {
  console.log('[1/4] Cleaning Rust target...');
  fs.rmSync(targetDir, { recursive: true, force: true });
  console.log('  ✅ target/ cleaned');
}

// Clean gen
const genDir = path.join('apps', 'desktop', 'src-tauri', 'gen');
if (fs.existsSync(genDir)) {
  console.log('[2/4] Cleaning Tauri gen...');
  fs.rmSync(genDir, { recursive: true, force: true });
  console.log('  ✅ gen/ cleaned');
}

// Clean tmp
const tmpDir = path.join('.tmp');
if (fs.existsSync(tmpDir)) {
  console.log('[3/4] Cleaning temp files...');
  fs.rmSync(tmpDir, { recursive: true, force: true });
  console.log('  ✅ .tmp/ cleaned');
}

// Clean Node modules (optional)
console.log('[4/4] Optional: Clean Node modules');
console.log('  To clean node_modules, run:');
console.log('    rm -rf apps/desktop/node_modules apps/desktop/pnpm-lock.yaml');
console.log('  Or use: pnpm prune --prod');

console.log('\n=== Clean complete ===');
