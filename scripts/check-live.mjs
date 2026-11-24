// Checks the live websites after a publication (and every day, monitor.yml):
//   node scripts/check-live.mjs [version]
// myroots.cc: pages in two languages, the style sheet, every download, the
// update file and the files it points to; demo.myroots.cc: the family website,
// its app files, the editor sign-in. Exits with 1 and a list when anything fails.
const version = process.argv[2] || '';
const SITE = process.env.MYROOTS_SITE || 'https://myroots.cc';
const DEMO = process.env.MYROOTS_DEMO || 'https://demo.myroots.cc';
const bad = [];
const ok = [];
const bust = () => `cb=${Date.now()}${Math.random().toString(36).slice(2, 6)}`;
const withBust = (u) => u + (u.includes('?') ? '&' : '?') + bust();

async function get(url, opt = {}) {
  try {
    return await fetch(withBust(url), { redirect: opt.redirect || 'follow', method: opt.method || 'GET', headers: { 'user-agent': 'myroots-check' } });
  } catch (e) {
    return { status: 0, headers: new Headers(), text: async () => String(e), url };
  }
}
function check(cond, what) {
  (cond ? ok : bad).push(what);
}

// myroots.cc pages and their style sheet
for (const page of ['/', '/en/']) {
  const r = await get(SITE + page);
  const html = r.status === 200 ? await r.text() : '';
  check(r.status === 200, `${SITE}${page}: ${r.status}`);
  const css = html.match(/href="([^"]*style\.css[^"]*)"/)?.[1];
  if (css) {
    const c = await get(new URL(css, SITE + page).href);
    check(c.status === 200 && (c.headers.get('content-type') || '').includes('css'), `style sheet of ${page}: ${c.status} ${c.headers.get('content-type')}`);
  } else check(false, `${page}: no style sheet link`);
  const imgs = [...html.matchAll(/src="([^"]+\.webp[^"]*)"/g)].slice(0, 3).map((m) => m[1]);
  for (const i of imgs) {
    const ri = await get(new URL(i, SITE + page).href, { method: 'HEAD' });
    check(ri.status === 200, `image ${i}: ${ri.status}`);
  }
}

// downloads: present, not an error page, not tiny
const DL = [
  ['MyRoots-mac.dmg', 5e6],
  ['MyRoots-windows-setup.exe', 5e6],
  ['MyRoots-linux.AppImage', 5e6],
  ['MyRoots-linux.deb', 5e6],
  ['MyRoots-linux.rpm', 5e6],
  ['myroots.zip', 1e6],
  ['myroots-server.zip', 1e6],
];
for (const [f, min] of DL) {
  const r = await get(`${SITE}/downloads/${f}`, { method: 'HEAD' });
  const len = Number(r.headers.get('content-length') || 0);
  const type = r.headers.get('content-type') || '';
  check(r.status === 200 && !type.includes('text/html') && (len === 0 || len >= min), `download ${f}: ${r.status} ${type} ${len} B`);
}

// automatic updates
{
  const r = await get(`${SITE}/updates/latest.json`);
  let j = null;
  try {
    j = r.status === 200 ? await r.json() : null;
  } catch {}
  check(!!j, `updates/latest.json: ${r.status}`);
  if (j) {
    if (version) check(j.version === version, `latest.json version ${j.version}, expected ${version}`);
    const plats = Object.entries(j.platforms || {});
    check(plats.length >= 3, `latest.json platforms: ${plats.map((p) => p[0]).join(', ')}`);
    for (const [k, p] of plats) {
      check(typeof p.signature === 'string' && p.signature.length > 50, `${k}: signature`);
      const f = await get(p.url, { method: 'HEAD' });
      check(f.status === 200, `${k}: ${p.url} ${f.status}`);
    }
  }
}

// demo: the family website, its app files, the editor
{
  const r = await get(DEMO + '/');
  const html = r.status === 200 ? await r.text() : '';
  check(r.status === 200 && html.includes('<div id="app">'), `${DEMO}/: ${r.status}`);
  for (const m of html.matchAll(/(?:src|href)="(\/app\/assets\/[^"]+\.(?:js|css))"/g)) {
    const a = await get(DEMO + m[1], { method: 'HEAD' });
    check(a.status === 200, `demo ${m[1]}: ${a.status}`);
  }
  const d = await get(DEMO + '/data.js');
  check(d.status === 200, `demo data.js: ${d.status}`);
  const adm = await get(DEMO + '/admin', { redirect: 'manual' });
  check([301, 302, 303].includes(adm.status) && /login/.test(adm.headers.get('location') || ''), `demo /admin: ${adm.status} -> ${adm.headers.get('location')}`);
  const lg = await get(DEMO + '/login');
  check(lg.status === 200, `demo /login: ${lg.status}`);
}

console.log(`OK (${ok.length}):\n  ` + ok.join('\n  '));
if (bad.length) {
  console.log(`\nFAILED (${bad.length}):\n  ` + bad.join('\n  '));
  process.exit(1);
}
console.log('\nAll checks passed.');
