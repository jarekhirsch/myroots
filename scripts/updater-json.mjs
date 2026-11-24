// Prepares the update files for myroots.cc/updates/ from the assets of a
// GitHub release (used by .github/workflows/strona.yml):
//   node scripts/updater-json.mjs <version> <assets dir> <out dir> [base url]
// Writes <out>/latest.json and copies the signed update files to
// <out>/<version>/. The app (tauri-plugin-updater) reads latest.json and
// verifies every file with the public key in src-tauri/tauri.conf.json.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const [version, assets, out, base = 'https://myroots.cc/updates'] = process.argv.slice(2);
if (!version || !assets || !out) {
  console.error('usage: updater-json.mjs <version> <assets dir> <out dir> [base url]');
  process.exit(1);
}
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const files = fs.readdirSync(assets);
// which file updates which platform (Tauri 2 updater keys)
const RULES = [
  [/\.app\.tar\.gz$/, ['darwin-aarch64', 'darwin-x86_64']],
  [/_x64-setup\.exe$/, ['windows-x86_64']],
  [/_amd64\.AppImage$/, ['linux-x86_64']],
  [/_aarch64\.AppImage$/, ['linux-aarch64']],
];
const platforms = {};
fs.mkdirSync(path.join(out, version), { recursive: true });
for (const [re, keys] of RULES) {
  const f = files.find((x) => re.test(x));
  if (!f || !files.includes(f + '.sig')) continue;
  fs.copyFileSync(path.join(assets, f), path.join(out, version, f));
  const signature = fs.readFileSync(path.join(assets, f + '.sig'), 'utf8').trim();
  for (const k of keys) platforms[k] = { signature, url: `${base}/${version}/${encodeURIComponent(f)}` };
}
if (!Object.keys(platforms).length) {
  console.error('no signed update files in', assets);
  process.exit(1);
}
// notes: the section of this version in CHANGELOG.en.md
let notes = '';
try {
  const log = fs.readFileSync(path.join(root, 'CHANGELOG.en.md'), 'utf8');
  const m = log.split(/^## /m).find((s) => s.startsWith(`${version} `) || s.startsWith(`v${version} `));
  if (m) notes = m.split('\n').slice(1).join('\n').trim();
} catch {
  /* no changelog */
}
const latest = { version, notes, pub_date: new Date().toISOString().replace(/\.\d+Z$/, 'Z'), platforms };
fs.writeFileSync(path.join(out, 'latest.json'), JSON.stringify(latest, null, 2) + '\n');
console.log('latest.json:', version, Object.keys(platforms).join(', '));
