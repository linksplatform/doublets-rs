#!/usr/bin/env node

/**
 * Bump version in Cargo.toml
 * Usage: node scripts/bump-version.mjs --bump-type <major|minor|patch> [--dry-run]
 *
 * Uses link-foundation libraries:
 * - use-m: Dynamic package loading without package.json dependencies
 * - lino-arguments: Unified configuration from CLI args, env vars, and .lenv files
 */

import { readFileSync, writeFileSync } from 'fs';

// Load use-m dynamically
const { use } = eval(
  await (await fetch('https://unpkg.com/use-m/use.js')).text()
);

// Import lino-arguments for CLI argument parsing
const { makeConfig } = await use('lino-arguments');

// Parse CLI arguments
const config = makeConfig({
  yargs: ({ yargs, getenv }) =>
    yargs
      .option('bump-type', {
        type: 'string',
        default: getenv('BUMP_TYPE', ''),
        describe: 'Version bump type: major, minor, or patch',
        choices: ['major', 'minor', 'patch'],
      })
      .option('dry-run', {
        type: 'boolean',
        default: false,
        describe: 'Show what would be done without making changes',
      }),
});

const { bumpType, dryRun } = config;

if (!bumpType || !['major', 'minor', 'patch'].includes(bumpType)) {
  console.error(
    'Usage: node scripts/bump-version.mjs --bump-type <major|minor|patch> [--dry-run]'
  );
  process.exit(1);
}

/**
 * Get current version from Cargo.toml
 * @returns {{major: number, minor: number, patch: number}}
 */
function getCurrentVersion() {
  const cargoToml = readFileSync('Cargo.toml', 'utf-8');
  const match = cargoToml.match(/^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"/m);

  if (!match) {
    console.error('Error: Could not parse version from Cargo.toml');
    process.exit(1);
  }

  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
  };
}

/**
 * Calculate new version based on bump type
 * @param {{major: number, minor: number, patch: number}} current
 * @param {string} bumpType
 * @returns {string}
 */
function calculateNewVersion(current, bumpType) {
  const { major, minor, patch } = current;

  switch (bumpType) {
    case 'major':
      return `${major + 1}.0.0`;
    case 'minor':
      return `${major}.${minor + 1}.0`;
    case 'patch':
      return `${major}.${minor}.${patch + 1}`;
    default:
      throw new Error(`Invalid bump type: ${bumpType}`);
  }
}

/**
 * Update version in Cargo.toml
 * @param {string} newVersion
 */
function updateCargoToml(newVersion) {
  let cargoToml = readFileSync('Cargo.toml', 'utf-8');
  cargoToml = cargoToml.replace(
    /^(version\s*=\s*")[^"]+(")/m,
    `$1${newVersion}$2`
  );
  writeFileSync('Cargo.toml', cargoToml, 'utf-8');
}

try {
  const current = getCurrentVersion();
  const currentStr = `${current.major}.${current.minor}.${current.patch}`;
  const newVersion = calculateNewVersion(current, bumpType);

  console.log(`Current version: ${currentStr}`);
  console.log(`New version: ${newVersion}`);

  if (dryRun) {
    console.log('Dry run - no changes made');
  } else {
    updateCargoToml(newVersion);
    console.log('Updated Cargo.toml');
  }
} catch (error) {
  console.error('Error:', error.message);
  process.exit(1);
}
