// Screenshots of the app for myroots.cc. Needs `npx vite build && npx vite preview --port 4174`
// and the demo backup (DEMO=path to rodzina-zawadzkich-demo.myroots).
//   node strona/zrodla/zrzuty/zrzuty.mjs /tmp/zrzuty-pl pl-PL
//   python3 strona/zrodla/zrzuty/ksiega.py /tmp/zrzuty-pl     (cover + Józef's page side by side)
// then convert to webp (1600/800, the phone 780/800) into strona/static/img/ (and img/en/ for en-US).
import { chromium } from '@playwright/test';
const out = process.argv[2], loc = process.argv[3] || 'pl-PL';
const B = 'http://localhost:4174/';
const b = await chromium.launch({ executablePath: process.env.PW_CHROME || undefined });
const ctx = await b.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1.25, locale: loc, reducedMotion: 'reduce' });
const p = await ctx.newPage();
const errs = []; p.on('pageerror', (e) => errs.push(e.message));
await p.goto(B + '#/data?tab=archive'); await p.waitForSelector('main'); await p.waitForTimeout(800);
const btn = p.getByRole('button', { name: /Otwórz plik .myroots|Open .myroots file/ });
if (await btn.isVisible().catch(() => false)) {
  const ch = p.waitForEvent('filechooser'); await btn.click();
  await (await ch).setFiles(process.env.DEMO || 'rodzina-zawadzkich-demo.myroots');
  const c = p.getByRole('dialog').getByRole('button', { name: /Przywróć|Restore/ });
  if (await c.isVisible({ timeout: 3000 }).catch(() => false)) await c.click();
  await p.getByText(/Kopia przywrócona|Backup restored/).waitFor({ timeout: 120000 });
}
await p.goto(B + '#/people'); await p.waitForTimeout(1500);
const ids = Object.fromEntries(await p.locator('main a.tile').evaluateAll((l) => l.map((a) => [a.querySelector('.nm')?.textContent?.trim(), a.getAttribute('href').split('/')[2]])));
const id = (n) => ids[n];
const go = async (hash, wait = 1800) => { await p.goto(B + hash); await p.waitForTimeout(wait); };
const shot = async (name) => { await p.mouse.move(1, 1); await p.screenshot({ path: `${out}/${name}.png` }); };
await go('#/tree/' + id('Michał Zawadzki'), 2500);
await p.locator('label.dep').nth(0).locator('select').selectOption('3');
await p.locator('label.dep').nth(1).locator('select').selectOption('1');
await p.waitForTimeout(2500); await shot('drzewo-genealogiczne-online-myroots');
await go('#/', 2500); await shot('drzewo-genealogiczne-z-datami-os-czasu');
await go('#/person/' + id('Tadeusz Zawadzki')); await shot('profil-osoby-ze-zdjeciami');
await go('#/gallery'); await shot('album-rodzinny-galeria');
await go('#/pets'); await shot('zwierzeta-w-drzewie-genealogicznym');
await go('#/poster/' + id('Zofia Zawadzka'), 2500); await shot('plakat-wachlarz-przodkow');
await go('#/person/' + id('Tadeusz Zawadzki') + '/years', 2500); await shot('animacja-przez-lata');
await go('#/book', 2000);
await p.locator('main textarea').first().fill(loc.startsWith('pl') ? 'Pamięci naszych dziadków, Józefa i Heleny' : 'In memory of our grandparents, Józef and Helena');
await p.getByRole('button', { name: /Generuj księgę|Generate book/ }).click();
await p.waitForTimeout(6000); await p.setViewportSize({ width: 1280, height: 1500 });
await p.locator('.sheet.cover').scrollIntoViewIfNeeded(); await p.waitForTimeout(800);
await p.locator('.sheet.cover').screenshot({ path: `${out}/_cover.png` });
const jp = p.locator('.sheet.person').filter({ has: p.getByRole('heading', { name: 'Józef Zawadzki', exact: true }) }).first();
await jp.evaluate((e) => e.scrollIntoView({ block: 'start' })); await p.waitForTimeout(2500);
const bx = await jp.boundingBox();
await p.screenshot({ path: `${out}/_person.png`, clip: { x: bx.x, y: bx.y, width: bx.width, height: Math.min(bx.height, 1500 - bx.y) } });
await p.setViewportSize({ width: 1280, height: 800 });
// marking people on a family photo
await go('#/gallery');
await p.locator('main figure').filter({ hasText: 'Michał Zawadzki' }).nth(1).locator('button').click(); await p.waitForTimeout(1200);
const mark = p.getByRole('button', { name: /Oznacz osoby|Mark people/ });
if (await mark.isVisible().catch(() => false)) { await mark.click(); await p.waitForTimeout(7000); }
await shot('oznaczanie-osob-na-zdjeciach');
await p.keyboard.press('Escape');
// phone
await p.setViewportSize({ width: 390, height: 844 });
const ph = await (await b.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, locale: loc, reducedMotion: 'reduce', storageState: await ctx.storageState({ indexedDB: true }) })).newPage();
await ph.goto(B + '#/person/' + id('Barbara Zawadzka')); await ph.waitForTimeout(3000);
await ph.screenshot({ path: `${out}/drzewo-genealogiczne-aplikacja-telefon.png` });
console.log('ids', Object.keys(ids).length, 'errors', errs);
await b.close();
