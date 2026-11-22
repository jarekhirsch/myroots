// Updates of the desktop app (macOS, Windows, Linux AppImage) from myroots.cc.
// The app reads https://myroots.cc/updates/latest.json (see tauri.conf.json,
// plugins.updater) at most once a day, unless Settings → Updates turns it
// off. Every update file is signed with the key of the author; the updater
// refuses files with a wrong signature. Phones update through their stores,
// .deb / .rpm through the system, the browser version with the page.
import { app } from './api.svelte';
import { t } from './i18n/index.svelte';
import { toast } from './ui.svelte';

export const upd = $state({
  checking: false,
  available: null as null | { version: string; notes: string },
  installing: false,
  progress: 0,
  dismissed: false,
});

let pending: any = null;
const KEY = 'myroots-update-check';
const DAY = 24 * 3600 * 1000;

/** The desktop app built with Tauri (not the phone apps, not the browser). */
export function canUpdate(): boolean {
  return !!(window as any).__TAURI_INTERNALS__ && !/Android|iPhone|iPad/i.test(navigator.userAgent);
}

function lastCheck(): number {
  try {
    return Number(localStorage.getItem(KEY)) || 0;
  } catch {
    return 0;
  }
}

/** `manual`: the button in Settings (always checks and reports the result). */
export async function checkUpdates(manual = false): Promise<void> {
  if (!canUpdate() || upd.checking || upd.installing) return;
  if (!manual && (app.settings?.updates_off || Date.now() - lastCheck() < DAY)) return;
  upd.checking = true;
  try {
    const { check } = await import('@tauri-apps/plugin-updater');
    const u = await check({ timeout: 20000 });
    try {
      localStorage.setItem(KEY, String(Date.now()));
    } catch {
      /* ignore */
    }
    if (u) {
      pending = u;
      upd.available = { version: u.version, notes: u.body || '' };
      upd.dismissed = false;
    } else if (manual) {
      toast(t('You have the latest version of MyRoots.'));
    }
  } catch {
    if (manual) toast(t('Could not check for a new version. Try again later.'), 'error');
  } finally {
    upd.checking = false;
  }
}

/** Downloads, verifies and installs the update, then restarts the app. */
export async function installUpdate(): Promise<void> {
  if (!pending || upd.installing) return;
  upd.installing = true;
  upd.progress = 0;
  let total = 0;
  let done = 0;
  try {
    await pending.downloadAndInstall((ev: any) => {
      if (ev.event === 'Started') total = ev.data?.contentLength || 0;
      else if (ev.event === 'Progress') {
        done += ev.data?.chunkLength || 0;
        upd.progress = total ? Math.min(1, done / total) : 0;
      }
    });
    const { relaunch } = await import('@tauri-apps/plugin-process');
    await relaunch();
  } catch {
    toast(t('The update could not be installed. Download the new version from myroots.cc.'), 'error');
  } finally {
    upd.installing = false;
  }
}

/** The "What's new" page of myroots.cc in the language of the app. */
export function newsUrl(lang: string): string {
  const slug: Record<string, string> = { pl: 'zmiany/', de: 'de/neuigkeiten/', fr: 'fr/nouveautes/', es: 'es/novedades/', it: 'it/novita/', pt: 'pt/novidades/', nl: 'nl/nieuw/', sv: 'sv/nyheter/', fi: 'fi/uutta/', da: 'da/nyheder/', he: 'he/whats-new/' };
  return `https://myroots.cc/${slug[lang] || 'en/whats-new/'}`;
}
