// MyRoots Server (PHP): installation, logging in, the editor with photos, saving,
// publishing and the family website. Needs PHP 8 with pdo_sqlite and the server
// folder: MYROOTS_SERVER_DIR=/path/to/myroots-server npm run test:e2e
// The app in the server's app/ folder is replaced with the current build (dist/).
import { test, expect, file } from './fixtures';
import { spawn, execSync, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const SRC = process.env.MYROOTS_SERVER_DIR;
test.skip(!SRC, 'MYROOTS_SERVER_DIR not set');
test.describe.configure({ mode: 'serial' });

let php: ChildProcess;
let url = '';
let dir = '';

test.beforeAll(async () => {
  dir = fs.mkdtempSync(path.join(os.tmpdir(), 'myroots-server-'));
  fs.cpSync(SRC!, dir, { recursive: true, filter: (p) => !/[/\\]data([/\\]|$)/.test(p.slice(SRC!.length)) });
  fs.rmSync(path.join(dir, 'app'), { recursive: true, force: true });
  fs.cpSync(path.join(file('dist')), path.join(dir, 'app'), { recursive: true, filter: (p) => !/[/\\](viewer|sw\.js|manifest\.webmanifest)$/.test(p) });
  const port = 8900 + Math.floor(Math.random() * 90);
  url = `http://127.0.0.1:${port}/`;
  // limits of a cheap shared hosting
  php = spawn('php', ['-d', 'memory_limit=128M', '-d', 'post_max_size=8M', '-d', 'upload_max_filesize=8M', '-S', `127.0.0.1:${port}`, 'router.php'], {
    cwd: dir,
    env: { ...process.env, PHP_CLI_SERVER_WORKERS: '6' },
    stdio: 'ignore',
  });
  for (let i = 0; i < 50; i++) {
    try {
      execSync(`curl -s -o /dev/null --noproxy '*' ${url}setup`);
      break;
    } catch {
      await new Promise((r) => setTimeout(r, 200));
    }
  }
});

test.afterAll(() => {
  php?.kill();
  if (dir) fs.rmSync(dir, { recursive: true, force: true });
});

test('install, log in, restore, publish', async ({ page }) => {
  // installation
  await page.goto(url + 'setup');
  await page.fill('input[name=title]', 'Test family');
  await page.fill('input[name=login]', 'admin');
  await page.fill('input[name=pass]', 'secret-pass-1');
  await page.fill('input[name=pass2]', 'secret-pass-1');
  await page.locator('form button').last().click();
  await expect(page.locator('main')).toBeVisible();

  // a backup with photos
  await page.goto(url + 'admin#/data?tab=archive');
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Open .myroots file' }).click();
  await (await chooser).setFiles(file('przyklad-ze-zdjeciami.myroots'));
  await expect(page.getByText('Backup restored.')).toBeVisible({ timeout: 90_000 });
  await expect(page.locator('.wp-state')).toHaveText('Saved on the server', { timeout: 60_000 });

  // after a reload everything comes back from the server, photos included
  await page.goto(url + 'admin#/gallery');
  await page.reload();
  await expect.poll(() => page.locator('main img').evaluateAll((l) => l.filter((i) => (i as HTMLImageElement).naturalWidth > 0).length), { timeout: 30_000 }).toBeGreaterThan(0);

  // one edit is saved
  await page.goto(url + 'admin#/people');
  await page.locator('main a[href^="#/person/"]:not([href="#/person/new"])').first().click();
  await page.getByRole('link', { name: 'Edit', exact: true }).click();
  await page.locator('label.field', { hasText: 'Occupation' }).locator('input').fill('Baker');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.locator('.toast').filter({ hasText: 'Saved.' })).toBeVisible();
  await expect(page.locator('.wp-state')).toHaveText('Saved on the server', { timeout: 30_000 });

  // publishing
  await page.goto(url + 'admin#/website');
  await page.getByRole('button', { name: /Publish/ }).first().click();
  await expect(page.locator('.toast').filter({ hasText: 'The family website is published.' })).toBeVisible({ timeout: 90_000 });

  // the family website for a visitor: own browser, no session
  const visitor = await page.context().browser()!.newContext({ locale: 'en-US' });
  const v = await visitor.newPage();
  await v.goto(url);
  await expect(v.locator('main')).toBeVisible();
  await v.goto(url + '#/gallery');
  await expect.poll(() => v.locator('main img').evaluateAll((l) => l.filter((i) => (i as HTMLImageElement).naturalWidth > 0).length), { timeout: 30_000 }).toBeGreaterThan(0);
  // the editor is closed for a visitor
  const r = await v.request.get(url + 'api/records');
  expect(r.status()).toBe(403);
  await visitor.close();
});

test('a wrong password is refused with a message', async ({ browser }) => {
  const ctx = await browser.newContext({ locale: 'en-US' });
  const p = await ctx.newPage();
  await p.goto(url + 'login');
  await p.fill('input[name=login]', 'admin');
  await p.fill('input[name=pass]', 'wrong-password');
  await p.locator('form button').last().click();
  await expect(p.getByText('Wrong login or password.')).toBeVisible();
  await ctx.close();
});
