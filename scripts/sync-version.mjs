#!/usr/bin/env node
/**
 * Syncs the Rust crate version in src-tauri/Cargo.toml with the version in
 * package.json (the single source of truth).
 *
 * package.json is authoritative:
 *   - tauri.conf.json reads its version from "../package.json" directly, and
 *   - the frontend imports the version from package.json,
 * so the only place that can drift is Cargo.toml (plain crate metadata that
 * nothing in the running app reads). This script keeps it honest.
 *
 * Run manually:   node scripts/sync-version.mjs
 * Runs automatically as the npm "version" lifecycle hook, so `npm version
 * x.y.z` (or patch/minor/major) bumps package.json AND Cargo.toml in one shot,
 * with no version argument to pass around.
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const pkgPath = resolve(root, 'package.json');
const cargoPath = resolve(root, 'src-tauri/Cargo.toml');

const version = JSON.parse(readFileSync(pkgPath, 'utf8')).version;
if (!version) {
  console.error('sync-version: no "version" field in package.json');
  process.exit(1);
}

const cargo = readFileSync(cargoPath, 'utf8');

// Only touch the version line inside the [package] section, never dependency
// versions further down the file.
const pkgSection = /(\[package\][^[]*?\nversion\s*=\s*")([^"]*)(")/;
if (!pkgSection.test(cargo)) {
  console.error('sync-version: could not find [package] version in Cargo.toml');
  process.exit(1);
}

const updated = cargo.replace(pkgSection, `$1${version}$3`);

if (updated === cargo) {
  console.log(`sync-version: Cargo.toml already at ${version}`);
} else {
  writeFileSync(cargoPath, updated);
  console.log(`sync-version: Cargo.toml -> ${version}`);
}
