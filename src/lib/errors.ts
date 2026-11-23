import { t } from './i18n/index.svelte';
import type { EngineError } from './backend/types';

/** Human message for an engine error code. */
/** The browser has no more room for the data of this site. */
export function isStorageFull(e: unknown): boolean {
  const x = e as { name?: string; message?: string; inner?: { name?: string } };
  return x?.name === 'QuotaExceededError' || x?.inner?.name === 'QuotaExceededError' || /quota/i.test(x?.message || '');
}

export function errorText(e: unknown): string {
  if (isStorageFull(e)) return t('There is no more room for MyRoots data on this device. Make a backup, then free up space or use the desktop app.');
  const err = e as EngineError;
  switch (err?.code) {
    case 'cycle':
      return t('This relation would make the person their own ancestor.');
    case 'relation':
      return t('A parent and a child, or an ancestor and a descendant, cannot be partners.');
    case 'not_found':
      return t('Not found');
    case 'private':
      return t('Private content.');
    case 'read_only':
      return t('The family website is read-only.');
    case 'gedcom_no_people':
      return t('The file does not contain any people (INDI).');
    case 'gedcom_zip':
      return t('The archive cannot be read.');
    case 'wp_invalid':
      return t('This is not a valid MyRoots backup file.');
    case 'archive_invalid':
      return t('This is not a MyRoots archive (.myroots).');
    case 'archive_newer':
      return t('This archive was made by a newer version of MyRoots. Update the app first.');
    default:
      return err?.message || t('An error occurred. Please try again.');
  }
}
