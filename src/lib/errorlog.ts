// Safety net for errors no view handled: the person always gets a message, and
// the last errors are kept (in memory only) for the "Report a problem" form.
import { toast } from './ui.svelte';
import { errorText } from './errors';

export interface LoggedError {
  at: string;
  message: string;
  stack: string;
}

const MAX = 20;
const log: LoggedError[] = [];

/** Remembers an error (also the handled ones worth reporting). */
export function remember(e: unknown) {
  const err = e as { message?: string; stack?: string; code?: string };
  const message = String(err?.code ? `${err.code}: ${err.message || ''}` : err?.message || e).slice(0, 500);
  log.push({ at: new Date().toISOString(), message, stack: String(err?.stack || '').split('\n').slice(0, 6).join('\n').slice(0, 1500) });
  if (log.length > MAX) log.shift();
}

export function recentErrors(): LoggedError[] {
  return [...log];
}

/** Noise of browsers and extensions that is not a problem of the app. */
function harmless(e: unknown): boolean {
  const m = String((e as { message?: string })?.message || e || '');
  const name = (e as { name?: string })?.name || '';
  return name === 'AbortError' || /ResizeObserver loop|Script error\.?$|NotAllowedError: play\(\)|The play\(\) request was interrupted/i.test(m);
}

/** A new version of the app replaced the files: a reload helps. */
function staleChunk(e: unknown): boolean {
  return /Failed to fetch dynamically imported module|Importing a module script failed|error loading dynamically imported module/i.test(String((e as { message?: string })?.message || e));
}

/** Program errors (a bug, not something the person did) get a plain message instead of their technical text. */
function friendly(e: unknown, t: (s: string) => string): string {
  if (e instanceof TypeError && /fetch|network|load failed/i.test(e.message)) return t('No connection. Check the internet and try again.');
  if (e instanceof TypeError || e instanceof ReferenceError || e instanceof RangeError || e instanceof SyntaxError || typeof e === 'string')
    return t('Something went wrong. Try again; if it happens again, report the problem (menu: Report a problem).');
  return errorText(e);
}

export function installErrorNet(t: (s: string) => string) {
  const handle = (e: unknown) => {
    if (harmless(e)) return;
    remember(e);
    console.error(e);
    toast(staleChunk(e) ? t('The app was updated. Reload the page.') : friendly(e, t), 'error');
  };
  window.addEventListener('unhandledrejection', (ev) => {
    ev.preventDefault();
    handle(ev.reason);
  });
  window.addEventListener('error', (ev) => {
    // errors of images and other resources have no `error` and are handled where they are shown
    if (!ev.error && !ev.message) return;
    handle(ev.error || ev.message);
  });
}
