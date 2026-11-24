// Every screen opens without errors, on a computer and on a phone, with a real
// tree (the example with photos from the repository).
import { test, expect, restore, open } from './fixtures';

const ROUTES = [
  '#/', '#/people', '#/tree', '#/map', '#/events', '#/gallery', '#/places', '#/stats', '#/health', '#/graves', '#/clans',
  '#/sources', '#/pets', '#/kinship', '#/lifeline', '#/duplicates', '#/book', '#/poster', '#/website', '#/data', '#/settings',
  '#/suggestions', '#/trees',
];

test('every screen opens', async ({ page }) => {
  await restore(page, 'przyklad-ze-zdjeciami.myroots');
  for (const r of ROUTES) {
    await open(page, r);
    await expect(page.locator('main'), r).not.toBeEmpty();
    await page.waitForTimeout(300);
  }
  // a person, their edit form and history, a source and a pet
  await open(page, '#/people');
  const person = page.locator('main a[href^="#/person/"]:not([href="#/person/new"])').first();
  const href = (await person.getAttribute('href'))!;
  for (const r of [href, `${href}/edit`, `${href}/history`]) {
    await open(page, r);
    await expect(page.locator('main')).not.toBeEmpty();
  }
  await open(page, '#/sources');
  const src = page.locator('main a[href^="#/source/"]').first();
  if (await src.count()) {
    await src.click();
    await expect(page.locator('main h1')).toBeVisible();
  }
});

test('unknown address shows a page, not an error', async ({ page }) => {
  await open(page, '#/no-such-page');
  await expect(page.locator('main')).not.toBeEmpty();
  await open(page, '#/person/999999');
  await expect(page.locator('main')).not.toBeEmpty();
});
