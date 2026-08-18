#!/usr/bin/env node
'use strict';

const { execSync } = require('child_process');

console.log('=== Starting dev server ===');
process.chdir('apps/desktop');
execSync('pnpm dev', { stdio: 'inherit' });
