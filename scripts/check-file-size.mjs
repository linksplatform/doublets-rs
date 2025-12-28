#!/usr/bin/env node

/**
 * Check for files exceeding the maximum allowed line count
 * Exits with error code 1 if any files exceed the limit
 *
 * Uses link-foundation libraries:
 * - use-m: Dynamic package loading without package.json dependencies
 */

import { readFileSync, readdirSync, statSync } from 'fs';
import { join, relative, extname } from 'path';

const MAX_LINES = 1000;
const FILE_EXTENSIONS = ['.rs'];
const EXCLUDE_PATTERNS = ['target', '.git', 'node_modules'];

/**
 * Check if a path should be excluded
 * @param {string} path
 * @returns {boolean}
 */
function shouldExclude(path) {
  return EXCLUDE_PATTERNS.some((pattern) => path.includes(pattern));
}

/**
 * Recursively find all Rust files in a directory
 * @param {string} directory
 * @returns {string[]}
 */
function findRustFiles(directory) {
  const files = [];

  function walkDir(dir) {
    const entries = readdirSync(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = join(dir, entry.name);

      if (shouldExclude(fullPath)) {
        continue;
      }

      if (entry.isDirectory()) {
        walkDir(fullPath);
      } else if (entry.isFile() && FILE_EXTENSIONS.includes(extname(entry.name))) {
        files.push(fullPath);
      }
    }
  }

  walkDir(directory);
  return files;
}

/**
 * Count lines in a file
 * @param {string} filePath
 * @returns {number}
 */
function countLines(filePath) {
  const content = readFileSync(filePath, 'utf-8');
  return content.split('\n').length;
}

try {
  const cwd = process.cwd();
  console.log(`\nChecking Rust files for maximum ${MAX_LINES} lines...\n`);

  const files = findRustFiles(cwd);
  const violations = [];

  for (const file of files) {
    const lineCount = countLines(file);
    if (lineCount > MAX_LINES) {
      violations.push({
        file: relative(cwd, file),
        lines: lineCount,
      });
    }
  }

  if (violations.length === 0) {
    console.log('All files are within the line limit\n');
    process.exit(0);
  } else {
    console.log('Found files exceeding the line limit:\n');
    for (const violation of violations) {
      console.log(
        `  ${violation.file}: ${violation.lines} lines (exceeds ${MAX_LINES})`
      );
    }
    console.log(`\nPlease refactor these files to be under ${MAX_LINES} lines\n`);
    process.exit(1);
  }
} catch (error) {
  console.error('Error:', error.message);
  process.exit(1);
}
