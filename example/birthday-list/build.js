#!/usr/bin/env node
'use strict';

// Simple build script: copies index.html to build/
// No external dependencies. Uses only Node.js standard library.

const fs = require('fs');
const path = require('path');

const buildDir = path.join(__dirname, 'build');

fs.mkdirSync(buildDir, { recursive: true });
fs.copyFileSync(path.join(__dirname, 'index.html'), path.join(buildDir, 'index.html'));

console.log('birthday-list: build complete');
