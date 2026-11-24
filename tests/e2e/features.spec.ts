// The other parts of the app: sources, photos, pets, duplicates, settings in
// every language, the problem report and the safety net for unexpected errors.
import { test, expect, open, toast, peopleCount, restore } from './fixtures';
import path from 'node:path';
import type { Page } from '@playwright/test';

const files = (n: string) => path.join(import.meta.dirname, 'files', n);

async function newPerson(page: Page, first: string, last: string, birth = '') {
  await open(page, '#/person/new');
  await page.locator('label.field', { hasText: 'First name' }).first().locator('input').fill(first);
  await page.locator('label.field', { hasText: 'Surname' }).first().locator('input').fill(last);
  if (birth) await page.locator('label.field', { hasText: 'Date of birth' }).locator('input').first().fill(birth);
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await toast(page, 'Saved.');
}

test('sources, photos and pets', async ({ page }) => {
  await newPerson(page, 'Jan', 'Kowalski', '1901-02-03');

  // a profile photo: a real photo is added, a broken file is refused with a message
  await page.getByRole('link', { name: 'Edit', exact: true }).click();
  let chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Upload', exact: true }).first().click();
  await (await chooser).setFiles(files('broken.jpg'));
  await expect(page.locator('.toast.error').filter({ hasText: 'could not be added' })).toBeVisible();
  chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Upload', exact: true }).first().click();
  await (await chooser).setFiles(files('photo.jpg'));
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await toast(page, 'Saved.');
  await open(page, '#/gallery');
  await expect.poll(() => page.locator('main img').evaluateAll((l) => l.filter((i) => (i as HTMLImageElement).naturalWidth > 0).length)).toBeGreaterThan(0);

  // a source: a title is required, then it is saved
  await open(page, '#/source/new');
  const title = page.locator('label.field', { hasText: 'Title' }).first().locator('input');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  // the browser itself points at the empty required field
  expect(await title.evaluate((i) => (i as HTMLInputElement).validity.valueMissing)).toBe(true);
  await expect(page).toHaveURL(/#\/source\/new/);
  await title.fill('Birth certificate of Jan');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await toast(page, 'Saved.');
  await open(page, '#/sources');
  await expect(page.locator('main')).toContainText('Birth certificate of Jan');

  // a pet
  await open(page, '#/pet/new');
  const name = page.locator('label.field', { hasText: 'Name' }).first().locator('input');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  expect(await name.evaluate((i) => (i as HTMLInputElement).validity.valueMissing)).toBe(true);
  await name.fill('Burek');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await toast(page, 'Saved.');
  await open(page, '#/pets');
  await expect(page.locator('main')).toContainText('Burek');
});

test('duplicates are found and merged', async ({ page }) => {
  await newPerson(page, 'Zofia', 'Wiśniewska', '1930-05-06');
  await newPerson(page, 'Zofia', 'Wiśniewska', '1930-05-06');
  expect(await peopleCount(page)).toBe(2);
  await open(page, '#/duplicates');
  await expect(page.locator('main')).toContainText('Zofia');
  await page.locator('main button:not([disabled])', { hasText: 'Compare' }).first().click();
  await page.getByRole('button', { name: 'Merge', exact: true }).click();
  // asks first: merging cannot be undone
  await page.locator('button.danger', { hasText: 'Merge' }).click();
  await toast(page, 'People merged.');
  expect(await peopleCount(page)).toBe(1);
});

test('the app works in every language', async ({ page }) => {
  await restore(page, 'przyklad-ze-zdjeciami.myroots');
  const langs = ['pl', 'de', 'fr', 'es', 'it', 'pt', 'nl', 'sv', 'fi', 'da', 'he', 'en'];
  for (const l of langs) {
    await open(page, '#/settings');
    await page.locator('select').filter({ has: page.locator(`option[value="${l}"]`) }).first().selectOption(l);
    await page.locator('main button.primary').last().click(); // "Save changes", in the language shown at the moment
    await expect(page.locator('html')).toHaveAttribute('lang', new RegExp(`^${l}`));
    await expect(page.locator('html')).toHaveAttribute('dir', l === 'he' ? 'rtl' : 'ltr');
    for (const r of ['#/', '#/people', '#/tree', '#/stats', '#/data']) {
      await open(page, r);
      await expect(page.locator('main')).not.toBeEmpty();
    }
  }
});

test('report a problem and the safety net', async ({ page, errors }) => {
  await newPerson(page, 'Adam', 'Nowy');
  // an unexpected error in the app gives a message instead of silence
  await page.evaluate(() => {
    setTimeout(() => Promise.reject(new TypeError('test failure')), 0);
  });
  await expect(page.locator('.toast.error').filter({ hasText: 'Something went wrong' })).toBeVisible();
  errors.length = 0; // the error above was made on purpose

  await page.locator('button.dk.more').click();
  // a discreet link to support the project (Ko-fi outside Polish)
  await expect(page.getByRole('link', { name: 'Support the project' })).toHaveAttribute('href', 'https://ko-fi.com/jarekhirsch');
  await page.getByRole('button', { name: 'Report a problem' }).click();
  const d = page.getByRole('dialog');
  await expect(d.getByRole('link', { name: /Send by e-mail/ })).toHaveClass(/disabled/);
  await d.locator('textarea').first().fill('The tree does not show my grandmother.');
  const mail = await d.getByRole('link', { name: /Send by e-mail/ }).getAttribute('href');
  expect(mail).toMatch(/^mailto:/);
  expect(decodeURIComponent(mail!)).toContain('The tree does not show my grandmother.');
  expect(decodeURIComponent(mail!)).toContain('test failure'); // the last errors are attached
  expect(decodeURIComponent(mail!)).not.toContain('Adam'); // nothing from the tree
  const gh = await d.getByRole('link', { name: 'GitHub' }).getAttribute('href');
  expect(gh).toContain('/issues/new?');
});
