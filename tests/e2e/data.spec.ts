// Moving data in and out: GEDCOM both ways, a full backup restored in an empty
// browser, the family website as a ZIP. Nothing may be lost on the way.
import { test, expect, open, toast, importGedcom, restore, peopleCount, file } from './fixtures';
import fs from 'node:fs';
import JSZip from 'jszip';

test('GEDCOM import, export and import again', async ({ page }, info) => {
  await importGedcom(page, 'przyklad-rodzina.ged');
  const n = await peopleCount(page);
  expect(n).toBeGreaterThan(5);

  await open(page, '#/data?tab=gedcom');
  const dl = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export GEDCOM' }).click();
  const out = info.outputPath('export.ged');
  await (await dl).saveAs(out);
  await toast(page, /Saved: /);
  const text = fs.readFileSync(out, 'utf8');
  expect(text.startsWith('0 HEAD')).toBeTruthy();
  expect((text.match(/^0 @[^@]+@ INDI/gm) || []).length).toBe(n);

  // the exported file read back into an empty browser gives the same tree
  const fresh = await page.context().browser()!.newPage({ locale: 'en-US', baseURL: info.project.use.baseURL });
  await open(fresh, '#/data?tab=gedcom');
  const chooser = fresh.waitForEvent('filechooser');
  await fresh.getByRole('button', { name: 'Choose a .ged file' }).click();
  await (await chooser).setFiles(out);
  await fresh.getByRole('button', { name: 'Import', exact: true }).click();
  await expect(fresh.getByText('Import finished')).toBeVisible();
  expect(await peopleCount(fresh)).toBe(n);
  await fresh.close();
});

test('a backup with photos restored in an empty browser', async ({ page, browser }, info) => {
  await restore(page, 'przyklad-ze-zdjeciami.myroots');
  const n = await peopleCount(page);
  await open(page, '#/gallery');
  const photos = await page.locator('main img').count();
  expect(photos).toBeGreaterThan(0);

  await open(page, '#/data?tab=archive');
  const dl = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Save backup' }).click();
  const out = info.outputPath('backup.myroots');
  await (await dl).saveAs(out);
  await toast(page, /Backup saved/);

  // the same files as in the original backup
  const a = await JSZip.loadAsync(fs.readFileSync(file('przyklad-ze-zdjeciami.myroots')));
  const b = await JSZip.loadAsync(fs.readFileSync(out));
  const media = (z: JSZip) => Object.keys(z.files).filter((k) => k.startsWith('media/')).sort();
  expect(media(b)).toEqual(media(a));

  const ctx = await browser.newContext({ locale: 'en-US', baseURL: info.project.use.baseURL });
  const fresh = await ctx.newPage();
  await open(fresh, '#/data?tab=archive');
  const chooser = fresh.waitForEvent('filechooser');
  await fresh.getByRole('button', { name: 'Open .myroots file' }).click();
  await (await chooser).setFiles(out);
  await toast(fresh, 'Backup restored.');
  expect(await peopleCount(fresh)).toBe(n);
  await open(fresh, '#/gallery');
  await expect(fresh.locator('main img').first()).toBeVisible();
  // photos really load (not broken images)
  await expect.poll(() => fresh.locator('main img').evaluateAll((l) => l.filter((i) => (i as HTMLImageElement).naturalWidth > 0).length)).toBeGreaterThan(0);
  await ctx.close();
});

test('a wrong file gives a clear message and keeps the tree', async ({ page }, info) => {
  await restore(page, 'przyklad-ze-zdjeciami.myroots');
  const n = await peopleCount(page);
  const bad = info.outputPath('not-a-backup.myroots');
  fs.writeFileSync(bad, 'this is not a backup');
  await open(page, '#/data?tab=archive');
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Open .myroots file' }).click();
  await (await chooser).setFiles(bad);
  const confirm = page.getByRole('dialog').getByRole('button', { name: 'Restore' });
  if (await confirm.isVisible().catch(() => false)) await confirm.click();
  await expect(page.locator('.toast.error').filter({ hasText: 'not a MyRoots' })).toBeVisible();
  expect(await peopleCount(page)).toBe(n);

  const badGed = info.outputPath('empty.ged');
  fs.writeFileSync(badGed, '0 HEAD\n1 CHAR UTF-8\n0 TRLR\n');
  await open(page, '#/data?tab=gedcom');
  const c2 = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Choose a .ged file' }).click();
  await (await c2).setFiles(badGed);
  await expect(page.locator('.toast.error').filter({ hasText: 'does not contain any people' })).toBeVisible();
});

test('the family website as a ZIP', async ({ page }, info) => {
  await restore(page, 'przyklad-ze-zdjeciami.myroots');
  await open(page, '#/website');
  const dl = page.waitForEvent('download');
  await page.getByRole('button', { name: /Export website/ }).click();
  const out = info.outputPath('site.zip');
  await (await dl).saveAs(out);
  await toast(page, /Website saved/);
  const z = await JSZip.loadAsync(fs.readFileSync(out));
  const names = Object.keys(z.files);
  expect(names.some((n) => /index\.html$/.test(n))).toBeTruthy();
  expect(names.some((n) => /data\.js$/.test(n))).toBeTruthy();
});
