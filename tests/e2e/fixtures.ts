// Shared helpers: every test fails on a JavaScript error or a console error,
// so problems that do not show on screen are caught too.
import { test as base, expect, type Page } from '@playwright/test';
import path from 'node:path';

export const ROOT = path.resolve(import.meta.dirname, '../..');
export const file = (name: string) => path.join(ROOT, name);

/** Console messages that are not problems of the app. */
const IGNORE = [/Download the Svelte DevTools/i, /favicon/i, /net::ERR_INTERNET_DISCONNECTED/, /tile|openfreemap|cartocdn|opentopomap|overpass|nominatim|photon/i, /Failed to load resource/i];

export const test = base.extend<{ errors: string[] }>({
  errors: [
    async ({ page }, use) => {
      const errors: string[] = [];
      // a blocked request says where it came from
      await page.addInitScript(() =>
        document.addEventListener('securitypolicyviolation', (e) => console.error(`CSP: ${e.violatedDirective} blocked ${e.blockedURI.slice(0, 40)} at ${e.sourceFile}:${e.lineNumber}`)),
      );
      page.on('pageerror', (e) => errors.push(`pageerror: ${e.message}`));
      page.on('console', (m) => {
        if (m.type() === 'error' && !IGNORE.some((r) => r.test(m.text()))) errors.push(`console: ${m.text()}`);
      });
      await use(errors);
      expect(errors, 'JavaScript errors during the test').toEqual([]);
    },
    { auto: true },
  ],
});
export { expect };

/** Waits for the app to start (the loading screen goes away). */
export async function open(page: Page, hash = '#/') {
  await page.goto(hash);
  await expect(page.locator('main')).toBeVisible();
}

/** Restores a .myroots backup from the repository (replaces the tree). */
export async function restore(page: Page, name: string) {
  await open(page, '#/data?tab=archive');
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Open .myroots file' }).click();
  await (await chooser).setFiles(file(name));
  const confirm = page.getByRole('dialog').getByRole('button', { name: 'Restore' });
  if (await confirm.isVisible().catch(() => false)) await confirm.click();
  await expect(page.getByText('Backup restored.')).toBeVisible({ timeout: 60_000 });
}

/** Imports a GEDCOM file (with the preview step). */
export async function importGedcom(page: Page, name: string) {
  await open(page, '#/data?tab=gedcom');
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Choose a .ged file' }).click();
  await (await chooser).setFiles(file(name));
  await page.getByRole('button', { name: 'Import', exact: true }).click();
  await expect(page.getByText('Import finished')).toBeVisible({ timeout: 60_000 });
}

/** A toast with this text appears (every action must say what happened). */
export async function toast(page: Page, text: string | RegExp) {
  await expect(page.locator('.toast').filter({ hasText: text }).first()).toBeVisible();
}

/** Number of people from the overview of the start page. */
export async function peopleCount(page: Page): Promise<number> {
  await open(page, '#/people');
  const t = await page.locator('main .lead').first().innerText();
  return Number(t.replace(/\D/g, ''));
}
