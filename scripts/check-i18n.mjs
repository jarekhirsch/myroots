// Lists UI strings that have no translation in the catalogs (plugin .po + extra/*.json).
// Usage: node scripts/check-i18n.mjs [lang] [--json]  → prints missing msgids (keys).
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const lang = process.argv[2] && !process.argv[2].startsWith('--') ? process.argv[2] : 'pl';
const files = [];
const walk = (d) => { for (const f of fs.readdirSync(d)) { const p = path.join(d, f); if (fs.statSync(p).isDirectory()) { if (!['wasm', 'locales', 'po'].includes(f)) walk(p); } else if (/\.(svelte|ts)$/.test(f)) files.push(p); } };
walk(path.join(root, 'src'));
const str = String.raw`'((?:[^'\\]|\\.)*)'|"((?:[^"\\]|\\.)*)"`;
const un = (m, a, b) => (a ?? b).replace(/\\(['"\\])/g, '$1');
const keys = new Map();
const add = (k, f) => { if (!keys.has(k)) keys.set(k, f); };
for (const f of files) {
  const s = fs.readFileSync(f, 'utf8');
  const rel = path.relative(root, f);
  for (const m of s.matchAll(new RegExp(String.raw`\bt\(\s*(?:${str})`, 'g'))) add(un(m, m[1], m[2]), rel);
  for (const m of s.matchAll(new RegExp(String.raw`\btx\(\s*(?:${str})\s*,\s*(?:${str})`, 'g'))) add(un(m, m[1], m[2]) + '\u0004' + un(m, m[3], m[4]), rel);
  for (const m of s.matchAll(new RegExp(String.raw`\btn\(\s*(?:${str})\s*,\s*(?:${str})`, 'g'))) add(un(m, m[1], m[2]), rel + ' [plural:' + un(m, m[3], m[4]) + ']');
}
const cat = JSON.parse(fs.readFileSync(path.join(root, 'src/lib/i18n/locales', lang + '.json'), 'utf8'));
const missing = [...keys].filter(([k]) => !cat.messages[k]);
if (process.argv.includes('--json')) console.log(JSON.stringify(missing.map(([k, f]) => ({ k, f })), null, 1));
else { for (const [k, f] of missing) console.log(JSON.stringify(k), ' ', f); console.error(`${keys.size} strings, ${missing.length} missing in ${lang}`); }
