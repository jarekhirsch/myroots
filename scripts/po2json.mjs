// Converts the WordPress plugin translations (.po) + extra strings of the app
// into compact JSON catalogs used by the UI: { plural, messages: { key: [forms] } }
// key = msgid, or msgctxt + "\u0004" + msgid.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const poDir = path.join(root, 'src/lib/i18n/po');
const extraDir = path.join(root, 'src/lib/i18n/extra');
const outDir = path.join(root, 'src/lib/i18n/locales');

function unquote(s) {
  s = s.trim();
  if (!s.startsWith('"')) return '';
  return JSON.parse(s.replace(/\\(?!["\\/bfnrtu])/g, '\\\\'));
}

function parsePo(text) {
  const entries = [];
  let cur = null, field = null;
  const flush = () => { if (cur && cur.msgid !== undefined) entries.push(cur); cur = null; field = null; };
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line === '' ) { flush(); continue; }
    if (line.startsWith('#')) { if (cur && cur.msgid !== undefined && field) flush(); continue; }
    let m;
    if ((m = line.match(/^(msgctxt|msgid|msgid_plural)\s+(".*")$/))) {
      if (m[1] === 'msgctxt' || (m[1] === 'msgid' && cur && cur.msgid !== undefined && !('msgid_plural' in cur) && cur.msgstr)) flush();
      cur = cur || { msgstr: [] };
      field = m[1]; cur[field] = unquote(m[2]); continue;
    }
    if ((m = line.match(/^msgstr(?:\[(\d+)\])?\s+(".*")$/))) {
      cur = cur || { msgstr: [] };
      field = 'msgstr' + (m[1] ?? '0'); cur.msgstr[Number(m[1] ?? 0)] = unquote(m[2]); continue;
    }
    if (line.startsWith('"') && cur && field) {
      if (field.startsWith('msgstr')) { const i = Number(field.slice(6)); cur.msgstr[i] += unquote(line); }
      else cur[field] += unquote(line);
    }
  }
  flush();
  return entries;
}

fs.mkdirSync(outDir, { recursive: true });
const langs = {};
for (const f of fs.readdirSync(poDir).filter((f) => f.endsWith('.po'))) {
  const locale = f.replace(/^myroots-/, '').replace(/\.po$/, '');
  const lang = locale.split('_')[0];
  const entries = parsePo(fs.readFileSync(path.join(poDir, f), 'utf8'));
  const header = entries.find((e) => e.msgid === '');
  const pf = (header?.msgstr[0] || '').match(/plural=([^;]+);/);
  const messages = {};
  for (const e of entries) {
    if (!e.msgid) continue;
    if (!e.msgstr.some((s) => s)) continue;
    const key = (e.msgctxt ? e.msgctxt + '\u0004' : '') + e.msgid;
    messages[key] = e.msgstr;
  }
  langs[lang] = { locale, plural: pf ? pf[1].trim() : 'n != 1', messages };
}
// Extra strings of the app (not present in the plugin), maintained by hand.
if (fs.existsSync(extraDir)) {
  for (const f of fs.readdirSync(extraDir).filter((f) => f.endsWith('.json'))) {
    const lang = f.replace(/\.json$/, '');
    const extra = JSON.parse(fs.readFileSync(path.join(extraDir, f), 'utf8'));
    langs[lang] = langs[lang] || { locale: lang, plural: 'n != 1', messages: {} };
    for (const [k, v] of Object.entries(extra)) {
      if (k.startsWith('//')) continue;
      langs[lang].messages[k] = Array.isArray(v) ? v : [v];
    }
  }
}
for (const [lang, cat] of Object.entries(langs)) {
  fs.writeFileSync(path.join(outDir, `${lang}.json`), JSON.stringify(cat));
  console.log(lang, Object.keys(cat.messages).length, 'messages');
}
