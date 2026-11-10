#!/usr/bin/env node
// Skrap's version lives in three files that must agree, and every version needs its notes.
//
//   node scripts/version.mjs check         the files agree, the notes exist (CI runs this; on a
//                                          tag, the tag must match too)
//   node scripts/version.mjs set 0.2.0     writes the version everywhere, then says what is left
//
// The files: Cargo.toml ([workspace.package]), app/package.json, app/src-tauri/tauri.conf.json.
// The notes: CHANGELOG.md (GitHub) and app/src/lib/changelog.ts ("What's new" in the app).
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const path = (p) => join(root, p);
const read = (p) => readFileSync(path(p), 'utf8');

const CARGO = 'Cargo.toml';
const PACKAGE = 'app/package.json';
const TAURI = 'app/src-tauri/tauri.conf.json';
const CHANGELOG = 'CHANGELOG.md';
const NOTES = 'app/src/lib/changelog.ts';

const cargoVersion = /(\[workspace\.package\][^[]*?\nversion = ")([^"]+)(")/;

function versions() {
  return {
    [CARGO]: read(CARGO).match(cargoVersion)?.[2],
    [PACKAGE]: JSON.parse(read(PACKAGE)).version,
    [TAURI]: JSON.parse(read(TAURI)).version,
  };
}

function check() {
  const found = versions();
  const problems = [];
  const distinct = new Set(Object.values(found));
  if (distinct.size !== 1 || distinct.has(undefined)) {
    problems.push(`the versions differ: ${Object.entries(found).map(([f, v]) => `${f} ${v}`).join(', ')}`);
  }
  const version = found[CARGO];
  if (!new RegExp(`^## ${version.replaceAll('.', '\\.')}\\b`, 'm').test(read(CHANGELOG))) {
    problems.push(`${CHANGELOG} has no "## ${version}" section`);
  }
  if (!read(NOTES).includes(`version: '${version}'`)) {
    problems.push(`${NOTES} has no notes for ${version}`);
  }
  const tag = process.env.GITHUB_REF_TYPE === 'tag' ? process.env.GITHUB_REF_NAME : null;
  if (tag && tag !== `v${version}`) {
    problems.push(`the tag ${tag} is not v${version}`);
  }
  if (problems.length) {
    console.error(`Version check failed:\n- ${problems.join('\n- ')}`);
    process.exit(1);
  }
  console.log(`Version ${version}: files, notes${tag ? ' and tag' : ''} agree.`);
}

function set(version) {
  if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.]+)?$/.test(version)) {
    console.error(`Not a version: ${version} (expected like 0.2.0)`);
    process.exit(1);
  }
  writeFileSync(path(CARGO), read(CARGO).replace(cargoVersion, `$1${version}$3`));
  for (const file of [PACKAGE, TAURI]) {
    const json = JSON.parse(read(file));
    json.version = version;
    writeFileSync(path(file), `${JSON.stringify(json, null, 2)}\n`);
  }
  console.log(`Version set to ${version} in ${CARGO}, ${PACKAGE} and ${TAURI}.`);
  console.log(`Now: add "## ${version}" to ${CHANGELOG}, its notes to ${NOTES} (every language),`);
  console.log(`run any cargo command to refresh Cargo.lock, commit, then tag v${version} and push the tag.`);
}

const [command, arg] = process.argv.slice(2);
if (command === 'check') check();
else if (command === 'set' && arg) set(arg);
else {
  console.error('Usage: node scripts/version.mjs check | set <version>');
  process.exit(1);
}
