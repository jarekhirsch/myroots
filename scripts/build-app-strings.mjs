// Builds src/lib/i18n/extra/{lang}.json from the hand-written translation
// tables in src/lib/i18n/app-strings/*.tsv.
// Line: `<index | "key"> ‖ pl ‖ de ‖ fr ‖ es ‖ it ‖ pt ‖ nl ‖ sv ‖ fi ‖ da ‖ he`
// Plural forms are separated by ` ¦ `. `index` points into keys.json
// ([key, pluralOrNull] pairs). Lines starting with # are comments.
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const dir = path.join(root, 'src/lib/i18n/app-strings');
const outDir = path.join(root, 'src/lib/i18n/extra');
const LANGS = ['pl', 'de', 'fr', 'es', 'it', 'pt', 'nl', 'sv', 'fi', 'da', 'he'];
const keys = JSON.parse(fs.readFileSync(path.join(dir, 'keys.json'), 'utf8'));

const ph = (s) => (s.match(/%(?:\d+\$)?[sd]/g) || []).map((p) => p.replace(/^%\d+\$/, '%')).sort().join(',');
const out = Object.fromEntries(LANGS.map((l) => [l, {}]));
let errors = 0, count = 0;

for (const f of fs.readdirSync(dir).filter((f) => f.endsWith('.tsv')).sort()) {
  const lines = fs.readFileSync(path.join(dir, f), 'utf8').split(/\r?\n/);
  lines.forEach((line, n) => {
    if (!line.trim() || line.startsWith('#')) return;
    const cols = line.split(' ‖ ').map((c) => c.trim());
    const where = `${f}:${n + 1}`;
    if (cols.length !== LANGS.length + 1) { console.error(`${where}: expected ${LANGS.length + 1} columns, got ${cols.length}`); errors++; return; }
    let key, plural = null;
    const id = cols[0];
    if (/^\d+$/.test(id)) {
      const k = keys[Number(id)];
      if (!k) { console.error(`${where}: unknown index ${id}`); errors++; return; }
      [key, plural] = k;
    } else if (id.startsWith('"')) key = JSON.parse(id);
    else { console.error(`${where}: bad id ${id}`); errors++; return; }
    const msg = key.includes('\u0004') ? key.split('\u0004')[1] : key;
    LANGS.forEach((lang, i) => {
      const forms = cols[i + 1].split(' ¦ ').map((s) => s.trim());
      const want = plural ? (lang === 'pl' ? 3 : 2) : 1;
      if (forms.length !== want) { console.error(`${where} [${lang}]: expected ${want} form(s), got ${forms.length}`); errors++; }
      for (const s of forms) {
        // A plural form may drop %d (e.g. "one day"), otherwise the set must match.
        const a = ph(s), b = ph(msg);
        if (a !== b && !(plural && b.replace(/%d,?/, '') === a.replace(/%d,?/, ''))) {
          console.error(`${where} [${lang}]: placeholders "${a}" vs "${b}" in: ${s}`); errors++;
        }
      }
      out[lang][key] = forms.length === 1 && !plural ? forms[0] : forms;
    });
    count++;
  });
}
fs.mkdirSync(outDir, { recursive: true });
for (const lang of LANGS) fs.writeFileSync(path.join(outDir, `${lang}.json`), JSON.stringify(out[lang], null, 1) + '\n');
console.log(`app strings: ${count} entries x ${LANGS.length} languages${errors ? `, ${errors} problem(s)` : ''}`);
if (errors) process.exit(1);
