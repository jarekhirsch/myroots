// The WordPress plugin on a real WordPress: logging in, the editor with photos,
// saving, publishing, the family website for a visitor, the [myroots] shortcode.
// Needs a running WordPress with the plugin (target/packages/myroots) active and
// pretty permalinks; GitHub sets one up in Docker (.github/workflows/tests.yml):
//   MYROOTS_WP_URL=http://localhost:8088/ npm run test:e2e -- wordpress
// Optional: MYROOTS_WP_USER, MYROOTS_WP_PASS (an administrator), MYROOTS_WP_PAGE
// (path of a page with the [myroots view="/people"] shortcode).
import { test, expect, file } from './fixtures';
import type { Page } from '@playwright/test';

const WP = process.env.MYROOTS_WP_URL?.replace(/\/?$/, '/');
const USER = process.env.MYROOTS_WP_USER ?? 'admin';
const PASS = process.env.MYROOTS_WP_PASS ?? 'secret-pass-1';
test.skip(!WP, 'MYROOTS_WP_URL not set');
test.describe.configure({ mode: 'serial' });

async function logIn(page: Page) {
  await page.goto(WP + 'wp-login.php');
  await page.fill('#user_login', USER);
  await page.fill('#user_pass', PASS);
  await page.click('#wp-submit');
  await expect(page).toHaveURL(/wp-admin/);
}

test('editor in wp-admin: restore, save, publish; the website for a visitor', async ({ page }) => {
  // (JavaScript errors fail the test: tests/e2e/fixtures.ts)
  await logIn(page);

  // the app opens full screen from the WordPress menu
  await page.goto(WP + 'wp-admin/admin.php?page=myroots#/data?tab=archive');
  await expect(page.locator('main')).toBeVisible({ timeout: 30_000 });

  // a backup with photos, sent to WordPress in small parts
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Open .myroots file' }).click();
  await (await chooser).setFiles(file('przyklad-ze-zdjeciami.myroots'));
  await expect(page.getByText('Backup restored.')).toBeVisible({ timeout: 120_000 });
  await expect(page.locator('.wp-state')).toHaveText('Saved in WordPress', { timeout: 60_000 });

  // after a reload everything comes back from WordPress, photos included
  await page.goto(WP + 'wp-admin/admin.php?page=myroots#/gallery');
  await page.reload();
  await expect
    .poll(() => page.locator('main img').evaluateAll((l) => l.filter((i) => (i as HTMLImageElement).naturalWidth > 0).length), { timeout: 30_000 })
    .toBeGreaterThan(0);

  // one edit is saved
  await page.goto(WP + 'wp-admin/admin.php?page=myroots#/people');
  await page.locator('main a[href^="#/person/"]:not([href="#/person/new"])').first().click();
  await page.getByRole('link', { name: 'Edit', exact: true }).click();
  await page.locator('label.field', { hasText: 'Occupation' }).locator('input').fill('Baker');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.locator('.toast').filter({ hasText: 'Saved.' })).toBeVisible();
  await expect(page.locator('.wp-state')).toHaveText('Saved in WordPress', { timeout: 30_000 });

  // publishing
  await page.goto(WP + 'wp-admin/admin.php?page=myroots#/website');
  await page.getByRole('button', { name: /Publish/ }).first().click();
  await expect(page.locator('.toast').filter({ hasText: 'The family website is published.' })).toBeVisible({ timeout: 120_000 });
  const site: string = await page.evaluate(() => (window as unknown as { MYROOTS_WP: { site_url: string } }).MYROOTS_WP.site_url);

  // the family website for a visitor: own browser, not logged in
  const visitor = await page.context().browser()!.newContext({ locale: 'en-US' });
  const v = await visitor.newPage();
  await v.goto(site);
  await expect(v.locator('main')).toBeVisible({ timeout: 30_000 });
  await v.goto(site + '#/people');
  await expect(v.locator('main a[href^="#/person/"]').first()).toBeVisible({ timeout: 30_000 });
  // the editor and its data are closed for a visitor
  const r = await v.request.get(WP + 'wp-json/myroots/v3/records');
  expect([401, 403]).toContain(r.status());
  const a = await v.request.get(WP + 'wp-admin/admin.php?page=myroots', { maxRedirects: 0 });
  expect(a.status()).toBe(302);

  // the website inside an ordinary WordPress page, through the shortcode
  if (process.env.MYROOTS_WP_PAGE) {
    await v.goto(WP + process.env.MYROOTS_WP_PAGE.replace(/^\//, ''));
    const frame = v.frameLocator('.myroots-embed iframe');
    await expect(frame.locator('main a[href^="#/person/"]').first()).toBeVisible({ timeout: 30_000 });
  }
  await visitor.close();
});

test('the editor is closed to a subscriber', async ({ browser }) => {
  test.skip(!process.env.MYROOTS_WP_SUBSCRIBER, 'MYROOTS_WP_SUBSCRIBER (login:password) not set');
  const [login, pass] = process.env.MYROOTS_WP_SUBSCRIBER!.split(':');
  const ctx = await browser.newContext({ locale: 'en-US' });
  const p = await ctx.newPage();
  await p.goto(WP + 'wp-login.php');
  await p.fill('#user_login', login);
  await p.fill('#user_pass', pass);
  await p.click('#wp-submit');
  await expect(p).toHaveURL(/wp-admin/);
  const r = await p.goto(WP + 'wp-admin/admin.php?page=myroots');
  expect(r?.status()).toBe(403);
  await ctx.close();
});
