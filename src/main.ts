// MyRoots © 2026 Jarek Hirsch, https://hirsch.lt
// SPDX-License-Identifier: GPL-3.0-or-later (with additional terms, see NOTICE.md)
import './app.css';
import { mount, unmount } from 'svelte';
import App from './App.svelte';
import Boot from './Boot.svelte';
import Unlock from './lib/components/Unlock.svelte';
import { initBackend } from './lib/backend';
import { isViewerMode } from './lib/backend/assets';
import { refreshInfo, app, call } from './lib/api.svelte';
import { initRouter } from './lib/router.svelte';
import { setLanguage, detectLanguage, t, tn } from './lib/i18n/index.svelte';
import { applyAppearance } from './lib/theme';
import { installErrorNet } from './lib/errorlog';
import type { Anniversary } from './lib/types';

declare const __VIEWER__: boolean;

async function notifyAnniversaries() {
  const s = app.settings;
  if (!s?.notifications?.enabled || app.info?.viewer) return;
  const key = 'myroots-notified';
  const today = new Date().toISOString().slice(0, 10);
  try {
    if (localStorage.getItem(key) === today) return;
  } catch {
    /* storage unavailable */
  }
  const list = await call<Anniversary[]>('anniversaries', { days: s.notifications.days_before || 14, limit: 0 });
  const soon = list.filter((a) => a.days_left <= 1);
  if (!soon.length) return;
  const { backend } = await import('./lib/backend');
  const body = soon
    .slice(0, 5)
    .map((a) => `${a.name}${a.partner_name ? ' & ' + a.partner_name : ''}: ${a.days_left === 0 ? t('today') : t('tomorrow')}`)
    .join('\n');
  await backend.notify?.(tn('%d anniversary', '%d anniversaries', soon.length), body);
  try {
    localStorage.setItem(key, today);
  } catch {
    /* ignore */
  }
}

async function start() {
  installErrorNet(t);
  const target = document.getElementById('app')!;
  const boot = mount(Boot, { target, props: { error: '' } });
  try {
    await setLanguage(detectLanguage());
    await initBackend();
    await refreshInfo();
    const site = app.info?.viewer ? app.info.site?.language : '';
    const wanted = (isViewerMode() ? site : app.settings?.language) || (window as any).MYROOTS_WP?.lang || '';
    await setLanguage(detectLanguage(wanted || undefined));
    applyAppearance(app.settings?.appearance);
    matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => applyAppearance(app.settings?.appearance));
    if (app.info?.viewer) {
      const title = app.info.site?.title || app.info.tree_name;
      if (title) document.title = title;
    }
    initRouter();
    unmount(boot);
    mount(App, { target });
    notifyAnniversaries().catch(() => {});
    // Installable, offline-capable browser app (not inside Tauri, not the family website).
    if (!isViewerMode() && !(window as any).__TAURI_INTERNALS__ && !(window as any).MYROOTS_WP && 'serviceWorker' in navigator && location.protocol.startsWith('http') && !import.meta.env.DEV) {
      navigator.serviceWorker.register('./sw.js').catch(() => {});
    }
  } catch (e) {
    unmount(boot);
    // the whole family website is protected: the password first
    const lock = (window as any).MYROOTS_SITE?.lock;
    if (isViewerMode() && lock && String((e as any)?.message) === 'locked') {
      if (lock.title) document.title = lock.title;
      mount(Unlock, { target, props: { header: lock, page: true } });
      return;
    }
    console.error(e);
    mount(Boot, { target, props: { error: String((e as any)?.message || e), code: String((e as any)?.code || '') } });
  }
}

start();
