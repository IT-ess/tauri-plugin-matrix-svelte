#!/usr/bin/env node
// Bumps the shared version of the plugin, the JS package and the example app.
// Usage: node scripts/bump-version.mjs <patch|minor|major>  → prints the new version.
// Run `cargo update -w` afterwards to refresh Cargo.lock.
import { readFileSync, writeFileSync } from 'node:fs';

const level = process.argv[2];
const current = JSON.parse(readFileSync('package.json', 'utf8')).version;
const [major, minor, patch] = current.split('.').map(Number);
const next = {
	major: `${major + 1}.0.0`,
	minor: `${major}.${minor + 1}.0`,
	patch: `${major}.${minor}.${patch + 1}`
}[level];
if (!next || !/^\d+\.\d+\.\d+$/.test(current)) {
	console.error(`usage: bump-version.mjs <patch|minor|major> (current: ${current})`);
	process.exit(1);
}

const apple = 'example/matrix-svelte-client/src-tauri/gen/apple';
const plist = /(<key>CFBundle(?:ShortVersionString|Version)<\/key>\s*<string>)[^<]+/g;
// [file, pattern, expected match count] — a count mismatch aborts before anything is written.
const edits = [
	['Cargo.toml', /(\[workspace\.package\]\nversion = ")[^"]+/g, 1],
	['package.json', /^(\t"version": ")[^"]+/gm, 1],
	['example/matrix-svelte-client/package.json', /^(\t"version": ")[^"]+/gm, 1],
	['example/matrix-svelte-client/src-tauri/tauri.conf.json', /^(\t"version": ")[^"]+/gm, 1],
	[`${apple}/project.yml`, /^( +CFBundle(?:ShortVersionString|Version): "?)[^"\n]+/gm, 4],
	[`${apple}/matrix-svelte-client_iOS/Info.plist`, plist, 2],
	[`${apple}/NotificationService/Info.plist`, plist, 2]
];

const out = edits.map(([file, re, count]) => {
	const text = readFileSync(file, 'utf8');
	const found = text.match(re)?.length ?? 0;
	if (found !== count) throw new Error(`${file}: expected ${count} version field(s), found ${found}`);
	return [file, text.replace(re, `$1${next}`)];
});
for (const [file, text] of out) writeFileSync(file, text);
console.log(next);
