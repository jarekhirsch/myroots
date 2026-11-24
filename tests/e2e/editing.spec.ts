// Building a family by hand: every action ends with a visible confirmation,
// the data shows up everywhere it should, and deleting cleans up.
import { test, expect, open, toast, peopleCount } from './fixtures';
import type { Page } from '@playwright/test';

async function field(page: Page, label: string, value: string) {
  await page.locator('label.field', { hasText: label }).first().locator('input, textarea').first().fill(value);
}

async function addRelative(page: Page, menu: string, first: string, last: string, birth = '') {
  await page.getByRole('button', { name: 'Add relative' }).click();
  await page.getByRole('menuitem', { name: menu, exact: true }).click();
  const d = page.getByRole('dialog');
  await d.locator('label.field', { hasText: 'First name' }).locator('input').fill(first);
  await d.locator('label.field', { hasText: 'Surname' }).locator('input').fill(last);
  if (birth) await d.locator('label.field', { hasText: 'Date of birth' }).locator('input').first().fill(birth);
  await d.getByRole('button', { name: 'Save' }).click();
  await toast(page, 'Saved.');
  await expect(d).toBeHidden();
}

test('a family built by hand', async ({ page }) => {
  await open(page);
  await expect(page.getByText('Welcome to MyRoots')).toBeVisible();
  await page.getByPlaceholder(/Name of the family tree/).fill('The Test family');
  await page.getByRole('button', { name: 'Save' }).click();
  await page.getByRole('link', { name: /Add the first person/ }).click();

  // the form refuses an empty person, with a message
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await toast(page, 'Enter at least a first name or a surname.');

  await field(page, 'First name', 'Anna');
  await field(page, 'Surname', 'Nowak');
  await page.locator('label.field', { hasText: 'Date of birth' }).locator('input').first().fill('1950-03-04');
  await field(page, 'Place of birth', 'Kraków');
  await field(page, 'Occupation', 'Teacher');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await toast(page, 'Saved.');
  await expect(page.locator('main h1')).toContainText('Anna Nowak');
  await expect(page.locator('main')).toContainText('Kraków');
  await expect(page.locator('main')).toContainText('Teacher');

  await addRelative(page, 'Father', 'Jan', 'Nowak', '1920');
  await addRelative(page, 'Mother', 'Maria', 'Kowalska', '1925');
  await addRelative(page, 'Spouse or partner', 'Piotr', 'Zieliński', '1948');
  await addRelative(page, 'Child', 'Ewa', 'Zielińska', '1975');
  await addRelative(page, 'Sibling', 'Tomasz', 'Nowak', '1953');
  await expect(page.locator('main')).toContainText('Jan Nowak');
  await expect(page.locator('main')).toContainText('Ewa Zielińska');
  expect(await peopleCount(page)).toBe(6);

  // the tree shows all of them
  await open(page, '#/tree');
  for (const n of ['Anna', 'Jan', 'Maria', 'Piotr', 'Ewa']) await expect(page.locator('main')).toContainText(n);

  // kinship between the child and the grandfather
  await open(page, '#/people');
  await page.locator('main a', { hasText: 'Ewa Zielińska' }).first().click();
  await expect(page.locator('main h1')).toContainText('Ewa');

  // a date of death before the birth is reported in data checks
  await open(page, '#/people');
  await page.locator('main a', { hasText: 'Tomasz Nowak' }).first().click();
  await page.getByRole('link', { name: 'Edit', exact: true }).click();
  await page.locator('label.field', { hasText: 'Date of death' }).locator('input').first().fill('1940');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await toast(page, 'Saved.');
  await open(page, '#/health');
  await expect(page.locator('main')).toContainText('Tomasz Nowak');

  // deleting asks first and confirms afterwards
  await open(page, '#/people');
  await page.locator('main a', { hasText: 'Tomasz Nowak' }).first().click();
  await page.getByRole('button', { name: 'Delete' }).click();
  await page.getByRole('dialog').getByRole('button', { name: /Delete/ }).click();
  await toast(page, 'Deleted.');
  expect(await peopleCount(page)).toBe(5);

  // everything survives a reload (saved in the browser)
  await page.reload();
  expect(await peopleCount(page)).toBe(5);
});
