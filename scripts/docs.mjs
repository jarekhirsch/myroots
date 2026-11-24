// Keeps the generated parts of docs/dev/*.md in step with the code.
//   node scripts/docs.mjs          rewrites the blocks between
//                                  <!-- auto:NAME --> and <!-- /auto:NAME -->
//   node scripts/docs.mjs --check  exits with 1 when a block is out of date
//                                  (run by .github/workflows/tests.yml)
// Blocks: engine-methods, files, deps.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const read = (p) => fs.readFileSync(path.join(root, p), 'utf8');
const check = process.argv.includes('--check');

/* ---------- engine methods ---------- */
function engineMethods() {
  const src = read('crates/myroots-core/src/engine.rs');
  const methods = [...src.matchAll(/^\s+"([a-z_]+(?:\.[a-z_]+)?)"\s*=>/gm)].map((m) => m[1]);
  const viewer = new Set([...(src.match(/const READ_ONLY: &\[&str\] = &\[([\s\S]*?)\];/)?.[1] || '').matchAll(/"([^"]+)"/g)].map((m) => m[1]));
  const api = read('src/lib/api.svelte.ts');
  const re = new RegExp(api.match(/const READ = \/(.+)\/;/)[1]);
  const rows = methods.map((m) => `| \`${m}\` | ${re.test(m) ? 'odczyt' : '**zapis**'} | ${viewer.has(m) ? 'tak' : ''} |`);
  const unknown = [...viewer].filter((m) => !methods.includes(m));
  return [
    `Metod: ${methods.length}. „odczyt” = pasuje do \`READ\` w \`src/lib/api.svelte.ts\` (nie odświeża widoków). „strona” = dozwolona na stronie rodzinnej (\`READ_ONLY\` w \`engine.rs\`).`,
    '',
    '| Metoda | Rodzaj | Strona |',
    '|---|---|---|',
    ...rows,
    ...(unknown.length ? ['', `Uwaga: w READ_ONLY są metody, których nie ma w engine.rs: ${unknown.join(', ')}`] : []),
  ].join('\n');
}

/* ---------- files with their first comment ---------- */
function firstComment(file) {
  const t = fs.readFileSync(file, 'utf8');
  const lines = t.split('\n').slice(0, 12);
  for (const l of lines) {
    const m = l.match(/^\s*(?:\/\/!?|\/\*\*?|\*|<!--)\s*(.+?)\s*(?:\*\/|-->)?$/);
    if (m && !/^(eslint|svelte-ignore|@ts-|SPDX|MyRoots ©)/.test(m[1]) && m[1].length > 3) return m[1].replace(/\|/g, '/');
  }
  return '';
}
function files() {
  const groups = [
    ['crates/myroots-core/src', /\.rs$/],
    ['crates/myroots-store/src', /\.rs$/],
    ['crates/myroots-wasm/src', /\.rs$/],
    ['src-tauri/src', /\.rs$/],
    ['src/views', /\.svelte$/],
    ['src/lib/components', /\.svelte$/],
    ['src/lib', /\.ts$/],
    ['src/lib/backend', /\.ts$/],
    ['scripts', /\.mjs$/],
  ];
  const out = [];
  for (const [dir, re] of groups) {
    const abs = path.join(root, dir);
    if (!fs.existsSync(abs)) continue;
    out.push('', `### ${dir}`, '', '| Plik | Opis (pierwszy komentarz w pliku) |', '|---|---|');
    for (const f of fs.readdirSync(abs).filter((x) => re.test(x)).sort()) out.push(`| \`${f}\` | ${firstComment(path.join(abs, f))} |`);
  }
  return out.join('\n').trim();
}

/* ---------- dependencies ---------- */
function deps() {
  const pkg = JSON.parse(read('package.json'));
  const out = ['### npm (package.json)', '', '| Pakiet | Wersja | Rodzaj |', '|---|---|---|'];
  for (const [k, v] of Object.entries(pkg.dependencies || {})) out.push(`| \`${k}\` | ${v} | aplikacja |`);
  for (const [k, v] of Object.entries(pkg.devDependencies || {})) out.push(`| \`${k}\` | ${v} | budowanie |`);
  out.push('', '### Rust (Cargo.toml)', '', '| Crate | Zależność | Wersja |', '|---|---|---|');
  for (const c of ['crates/myroots-core', 'crates/myroots-store', 'crates/myroots-wasm', 'src-tauri']) {
    const t = read(`${c}/Cargo.toml`);
    for (const sec of t.split(/^\[/m)) {
      if (!/^(dependencies|build-dependencies|target\..*dependencies)\]/.test(sec)) continue;
      for (const line of sec.split('\n').slice(1)) {
        const m = line.match(/^([a-zA-Z0-9_-]+)\s*=\s*(.+)$/);
        if (!m) continue;
        let v = m[2].trim();
        const ver = v.match(/version\s*=\s*"([^"]+)"/)?.[1] || v.match(/^"([^"]+)"$/)?.[1] || (/workspace\s*=\s*true/.test(v) ? 'workspace' : /path\s*=/.test(v) ? 'lokalny' : v);
        out.push(`| ${path.basename(c)} | \`${m[1]}\` | ${ver} |`);
      }
    }
  }
  return out.join('\n');
}

const BLOCKS = { 'engine-methods': engineMethods, files, deps };
let stale = [];
for (const f of fs.readdirSync(path.join(root, 'docs/dev')).filter((x) => x.endsWith('.md'))) {
  const p = path.join(root, 'docs/dev', f);
  const before = fs.readFileSync(p, 'utf8');
  const after = before.replace(/(<!-- auto:([a-z-]+) -->)[\s\S]*?(<!-- \/auto:\2 -->)/g, (all, open, name, close) => {
    if (!BLOCKS[name]) return all;
    return `${open}\n${BLOCKS[name]()}\n${close}`;
  });
  if (after !== before) {
    stale.push(f);
    if (!check) fs.writeFileSync(p, after);
  }
}
if (check && stale.length) {
  console.error(`docs/dev out of date: ${stale.join(', ')}. Run: npm run docs`);
  process.exit(1);
}
console.log(stale.length ? `updated: ${stale.join(', ')}` : 'docs/dev up to date');
