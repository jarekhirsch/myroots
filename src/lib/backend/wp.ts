// WordPress backend: the same Rust engine (WebAssembly) as the browser app,
// but the records live in the WordPress database and the photos in its
// uploads folder, both reached through the REST API of the MyRoots plugin.
// The editor runs in wp-admin; visitors get the read-only family website,
// published by the plugin at its own address.
import { buildBackup, openBackup } from '../archivejs';
import { unzipSync } from 'fflate';
import { WasmEngine } from '../wasm/myroots_wasm.js';
import { initWasm } from './wasm';
import type { Backend, MediaInput, TreesApi } from './types';
import { wpConfig, type WpConfig } from './assets';
import { downloadBlob, stamp, slug, todayLocal } from './util';
import { t } from '../i18n/index.svelte';
import { progress } from '../ui.svelte';

declare const __APP_VERSION__: string;

type Rec = { kind: string; id: number; data: any };
type Changes = { records: Rec[]; deleted_media: number[] };

/** Requests stay well under the usual PHP limits (post_max_size, upload_max_filesize). */
const CHUNK = 1024 * 1024;
const BATCH = 1_500_000;

export interface PublishProgress {
  stage: 'prepare' | 'upload' | 'done';
  done: number;
  total: number;
}

export interface SiteState {
  url: string;
  published: number;
  auto: boolean;
  persons: number;
  media: number;
}

export class WpError extends Error {
  code: string;
  constructor(code: string, message: string) {
    super(message);
    this.code = code;
  }
}

export class WpBackend implements Backend {
  platform = 'wp' as const;
  readonly = false;
  cfg!: WpConfig;
  private engine!: WasmEngine;
  private queue: Promise<void> = Promise.resolve();
  /** Changes not yet confirmed by the server (kept for a retry). */
  private pending: Changes = { records: [], deleted_media: [] };
  private failing = false;
  private retryTimer: any = 0;
  private publishTimer: any = 0;
  private publishing: Promise<void> | null = null;
  /** Several trees on the site: every request names the open one. */
  trees: TreesApi = {
    list: () => this.api('GET', 'trees'),
    create: (name) => this.api('POST', 'trees', { name }),
    rename: async (id, name) => void (await this.api('POST', `trees/${id}`, { name })),
    remove: async (id) => void (await this.api('POST', `trees/${id}/delete`)),
    open: async (id) => {
      await this.api('POST', `trees/${id}/open`);
      // the editor page reads the tree from the address first
      const u = new URL(location.href);
      u.searchParams.set('tree', id);
      history.replaceState(null, '', u.toString());
    },
  };
  /** Listeners of the save state (the status line in the menu). */
  listeners = new Set<(s: { saving: boolean; error: string }) => void>();

  static async create(): Promise<WpBackend> {
    await initWasm();
    const b = new WpBackend();
    b.cfg = wpConfig()!;
    const records = await b.api<Rec[]>('GET', 'records');
    b.engine = new WasmEngine(JSON.stringify(records), __APP_VERSION__, false, Date.now());
    if (!records.length) b.persist();
    await b.collectSuggestions().catch(() => {});
    // No outside services without consent (WordPress.org guideline 7): historical
    // weather starts switched off; turning it on in the settings is the consent.
    if (!b.cfg.weather_decided) await b.weatherOff().then(() => b.api('POST', 'site/options', { weather_decided: true })).catch(() => {});
    // REST nonces expire after a day: keep a fresh one while the editor is open
    setInterval(() => b.refreshNonce().catch(() => {}), 30 * 60_000);
    window.addEventListener('beforeunload', (e) => {
      if (b.pending.records.length || b.pending.deleted_media.length || b.busy) {
        e.preventDefault();
        e.returnValue = '';
      }
    });
    return b;
  }

  /** Publishing or restoring in progress: closing the page would leave it half done. */
  private busy = 0;

  private async whileBusy<T>(fn: () => Promise<T>): Promise<T> {
    this.busy++;
    try {
      return await fn();
    } finally {
      this.busy--;
    }
  }

  /** Runs the jobs a few at a time: photos reach the server several times faster than one by one. */
  private async pool<T>(items: T[], fn: (item: T) => Promise<void>, size = 4): Promise<void> {
    let i = 0;
    const worker = async () => {
      while (i < items.length) await fn(items[i++]);
    };
    await Promise.all(Array.from({ length: Math.min(size, items.length) }, worker));
  }

  /* ------------------------------------------------------------------ */
  /*  REST                                                               */
  /* ------------------------------------------------------------------ */

  url(path: string, query: Record<string, string | number> = {}): string {
    const u = this.cfg.rest + path;
    const q: Record<string, string | number> = this.cfg.tree ? { tree: this.cfg.tree, ...query } : query;
    const qs = new URLSearchParams(Object.entries(q).map(([k, v]) => [k, String(v)])).toString();
    return qs ? u + (u.includes('?') ? '&' : '?') + qs : u;
  }

  async api<T = any>(method: string, path: string, body?: unknown, query: Record<string, string | number> = {}): Promise<T> {
    const raw = body instanceof Uint8Array || body instanceof Blob;
    const r = await fetch(this.url(path, query), {
      method,
      credentials: 'same-origin',
      headers: {
        'X-WP-Nonce': this.cfg.nonce,
        ...(body === undefined ? {} : { 'Content-Type': raw ? 'application/octet-stream' : 'application/json' }),
      },
      body: body === undefined ? undefined : raw ? (body as BodyInit) : JSON.stringify(body),
    });
    const text = await r.text();
    let v: any = null;
    try {
      v = text ? JSON.parse(text) : null;
    } catch {
      /* an HTML error page from the server */
    }
    if (!r.ok) {
      if (r.status === 403 && v?.code === 'rest_cookie_invalid_nonce')
        throw new WpError('wp_session', this.cfg.host === 'server' ? t('The session has expired. Reload the page and log in again.') : t('The WordPress session has expired. Reload the page and log in again.'));
      throw new WpError(v?.code || 'wp_http_' + r.status, v?.message || `WordPress: HTTP ${r.status}`);
    }
    return v as T;
  }

  /** Uploads bytes in pieces (raw body, appended by the server). */
  /** Sends a file in parts; `onSent(bytes)` after every part (progress of big files). */
  private async upload(path: string, bytes: Uint8Array, query: Record<string, string | number>, onSent?: (bytes: number) => void) {
    if (!bytes.length) {
      await this.api('POST', path, new Uint8Array(0), { ...query, offset: 0, final: 1 });
      return;
    }
    for (let off = 0; off < bytes.length; off += CHUNK) {
      const end = Math.min(off + CHUNK, bytes.length);
      const q = { ...query, offset: off, final: end >= bytes.length ? 1 : 0 };
      // a busy shared hosting sometimes drops one request: try the piece again
      for (let attempt = 0; ; attempt++) {
        try {
          await this.api('POST', path, bytes.subarray(off, end), q);
          break;
        } catch (e) {
          const code = (e as WpError).code || '';
          const retry = attempt < 3 && (!(e instanceof WpError) || /^wp_http_(5\d\d|429|408)$/.test(code));
          if (!retry) throw e;
          await new Promise((r) => setTimeout(r, 1000 * 2 ** attempt));
        }
      }
      onSent?.(end);
    }
  }

  private async weatherOff() {
    const s = await this.call<any>('settings.get');
    if (!s.weather_off) await this.call('settings.save', { settings: { ...s, weather_off: true } });
  }

  /** A new REST nonce through admin-ajax (authenticated by the login cookie). */
  async refreshNonce(): Promise<boolean> {
    if (!this.cfg.ajax) return false;
    const r = await fetch(this.cfg.ajax + '?action=myroots_nonce', { credentials: 'same-origin' });
    if (!r.ok) return false;
    const v = await r.json().catch(() => null);
    if (!v?.nonce) return false;
    this.cfg.nonce = v.nonce;
    return true;
  }

  /* ------------------------------------------------------------------ */
  /*  Saving                                                             */
  /* ------------------------------------------------------------------ */

  private emit(saving: boolean, error = '') {
    for (const l of this.listeners) l({ saving, error });
  }

  private persist() {
    const ch = JSON.parse(this.engine.take_changes()) as Changes;
    if (!ch.records.length && !ch.deleted_media.length) return;
    // newer changes of the same record replace the waiting ones
    const byKey = new Map(this.pending.records.map((r) => [`${r.kind}:${r.id}`, r]));
    for (const r of ch.records) byKey.set(`${r.kind}:${r.id}`, r);
    this.pending.records = [...byKey.values()];
    this.pending.deleted_media.push(...ch.deleted_media);
    this.schedule();
  }

  private schedule() {
    this.queue = this.queue.then(() => this.send());
  }

  private async send() {
    if (!this.pending.records.length && !this.pending.deleted_media.length) return;
    const work = this.pending;
    this.pending = { records: [], deleted_media: [] };
    this.emit(true);
    try {
      // batches small enough for any PHP configuration
      let batch: Rec[] = [];
      let size = 0;
      const flushBatch = async (media: number[]) => {
        // each record as its exact JSON text (PHP would turn {} into [])
        await this.api('POST', 'records', { records: batch.map((r) => ({ kind: r.kind, id: r.id, json: r.data == null ? null : JSON.stringify(r) })), deleted_media: media });
        batch = [];
        size = 0;
      };
      for (const r of work.records) {
        const n = JSON.stringify(r).length;
        if (batch.length && size + n > BATCH) await flushBatch([]);
        batch.push(r);
        size += n;
      }
      await flushBatch(work.deleted_media);
      this.failing = false;
      this.emit(false);
      this.autoPublish();
    } catch (e) {
      // keep everything for the next attempt (newer edits win)
      const keep = new Map(work.records.map((r) => [`${r.kind}:${r.id}`, r]));
      for (const r of this.pending.records) keep.set(`${r.kind}:${r.id}`, r);
      this.pending = { records: [...keep.values()], deleted_media: [...work.deleted_media, ...this.pending.deleted_media] };
      const msg = String((e as any)?.message || e);
      if (!this.failing) {
        this.failing = true;
        const { toast } = await import('../ui.svelte');
        toast(msg, 'error');
      }
      this.emit(false, msg);
      clearTimeout(this.retryTimer);
      const session = (e as WpError)?.code === 'wp_session';
      if (session && (await this.refreshNonce().catch(() => false))) {
        this.retryTimer = setTimeout(() => this.schedule(), 500);
        return;
      }
      this.retryTimer = setTimeout(() => this.schedule(), session ? 60_000 : 8_000);
    }
  }

  async call<T>(method: string, params: unknown = {}): Promise<T> {
    const out = this.engine.call(method, JSON.stringify(params ?? {}), Date.now(), todayLocal());
    this.persist();
    return JSON.parse(out) as T;
  }

  async flush() {
    await this.queue;
  }

  /* ------------------------------------------------------------------ */
  /*  Media                                                              */
  /* ------------------------------------------------------------------ */

  async addMedia(m: MediaInput): Promise<number> {
    const { id } = await this.call<{ id: number }>('media.reserve');
    await this.flush();
    // a big file (scan, recording) moves the progress bar while it is sent
    const big = m.original.length > CHUNK && !progress.active;
    if (big) progress.active++;
    try {
      await this.upload(`media/${id}`, m.original, { thumb: 0, mime: m.mime }, big ? (n) => ((progress.done = n), (progress.total = m.original.length), (progress.label ||= m.fileName)) : undefined);
    } finally {
      if (big) progress.active = Math.max(0, progress.active - 1);
    }
    if (m.thumb) await this.upload(`media/${id}`, m.thumb, { thumb: 1, mime: 'image/jpeg' });
    await this.call('media.register', {
      media: { id, file_name: m.fileName, mime: m.mime, size: m.original.length, width: m.width, height: m.height, has_thumb: !!m.thumb, caption: m.caption || '', origin: m.origin || '' },
    });
    return id;
  }

  mediaUrl(id: number, thumb = true): string {
    if (!id) return '';
    return this.url(`media/${id}`, { t: thumb ? 1 : 0, g: this.cfg.gen, _wpnonce: this.cfg.nonce });
  }

  async mediaBytes(id: number, thumb = false): Promise<Uint8Array | null> {
    const r = await fetch(this.url(`media/${id}`, { t: thumb ? 1 : 0, strict: 1, g: this.cfg.gen, _wpnonce: this.cfg.nonce }), { credentials: 'same-origin' });
    if (!r.ok) return null;
    const b = new Uint8Array(await r.arrayBuffer());
    return b.length ? b : null;
  }

  private async loadBlobs(ids: number[], onEach?: () => void, original: (id: number) => boolean = () => true): Promise<Map<string, Uint8Array>> {
    const map = new Map<string, Uint8Array>();
    let i = 0;
    const worker = async () => {
      while (i < ids.length) {
        const id = ids[i++];
        const [o, t] = await Promise.all([original(id) ? this.mediaBytes(id, false) : null, this.mediaBytes(id, true)]);
        if (o) map.set(`${id}`, o);
        if (t) map.set(`${id}_t`, t);
        onEach?.();
      }
    };
    await Promise.all([worker(), worker(), worker(), worker()]);
    return map;
  }

  /* ------------------------------------------------------------------ */
  /*  Backup                                                             */
  /* ------------------------------------------------------------------ */

  async exportArchive(onProgress?: (done: number, total: number) => void): Promise<string | null> {
    await this.flush();
    // the records from the engine, the photos fetched from the server one at a time
    const records = this.engine.export_archive(() => undefined, Date.now());
    const ids = (await this.call<{ id: number }[]>('media.list')).map((m) => m.id);
    const blob = await buildBackup(records, ids, (id, thumb) => this.mediaBytes(id, thumb), onProgress);
    const info = await this.call<any>('app.info');
    const name = `${slug(info.tree_name || 'myroots')}-${stamp()}.myroots`;
    downloadBlob(name, blob, 'application/zip');
    return name;
  }

  async importArchive(file: Blob, onProgress?: (done: number, total: number) => void): Promise<void> {
    await this.flush();
    // the backup is not read whole: the photos are taken out one at a time while sending
    const opened = await openBackup(file);
    const recordsJson = this.engine.import_archive(opened ? opened.records : new Uint8Array(await file.arrayBuffer()), () => {});
    const media = opened?.photos || [];
    const recs = JSON.parse(recordsJson) as Rec[];
    const { gen } = await this.api<{ gen: number }>('POST', 'reset');
    this.cfg.gen = gen;
    // the photos: the longest part of a restore, sent a few at a time
    const total = media.reduce((a, m) => a + m.size, 0) || 1;
    let sent = 0;
    const part = new Map<number, number>();
    const report = () => onProgress?.(sent + [...part.values()].reduce((a, n) => a + n, 0), total);
    let k = 0;
    await this.whileBusy(() =>
      this.pool(media, async (m) => {
        const key = k++;
        const [id, thumb, b] = [m.id, m.thumb, await m.read()];
        await this.upload(`media/${id}`, b, { thumb: thumb ? 1 : 0 }, (n) => (part.set(key, n), report()));
        part.delete(key);
        sent += b.length;
        report();
      }),
    );
    this.pending = { records: recs, deleted_media: [] };
    // a backup from another device may have the weather on: ask again (off)
    await this.weatherOff();
    this.schedule();
    await this.flush();
    if (this.pending.records.length) throw new WpError('wp_save', t('Some data could not be saved in WordPress.'));
  }

  /* ------------------------------------------------------------------ */
  /*  Family website                                                     */
  /* ------------------------------------------------------------------ */

  /** The prebuilt read-only app, packed into the ZIP like on other platforms. */
  async exportSite(): Promise<string | null> {
    await this.flush();
    const base = this.cfg.assets + 'viewer/';
    const list: string[] = await fetch(base + 'files.json').then((r) => r.json());
    const files: Uint8Array[] = [];
    for (const f of list) files.push(new Uint8Array((await fetch(base + f).then((r) => r.arrayBuffer())) as ArrayBuffer));
    const ids = Array.from(this.engine.site_media(Date.now(), todayLocal()));
    const blobs = await this.loadBlobs(ids);
    const { lockZip } = await import('../sitelock');
    const zip = await lockZip(this.engine.export_site(list, files, (id: number, thumb: boolean) => blobs.get(thumb ? `${id}_t` : `${id}`), Date.now(), todayLocal()), await this.call('settings.get'));
    const info = await this.call<any>('app.info');
    const name = `${slug(info.site?.title || info.tree_name || 'family')}-website.zip`;
    downloadBlob(name, zip, 'application/zip');
    return name;
  }

  async siteState(): Promise<SiteState> {
    return this.api<SiteState>('GET', 'site');
  }

  async setAutoPublish(auto: boolean): Promise<void> {
    await this.api('POST', 'site/options', { auto });
    this.cfg.auto_publish = auto;
  }

  private autoPublish() {
    if (!this.cfg.auto_publish || !this.cfg.published) return;
    clearTimeout(this.publishTimer);
    this.publishTimer = setTimeout(() => this.publish().catch((e) => console.warn('MyRoots: publishing failed', e)), 15_000);
  }

  /**
   * Publishes the family website on this WordPress site. Only the public data
   * (privacy applied by the engine) and the photos it needs leave the editor;
   * photos already on the server are not sent again.
   */
  publish(onProgress?: (p: PublishProgress) => void): Promise<void> {
    if (this.publishing) return this.publishing;
    this.publishing = this.whileBusy(() => this.doPublish(onProgress)).finally(() => (this.publishing = null));
    return this.publishing;
  }

  private async doPublish(onProgress?: (p: PublishProgress) => void) {
    clearTimeout(this.publishTimer);
    await this.flush();
    onProgress?.({ stage: 'prepare', done: 0, total: 0 });
    const now = Date.now();
    const today = todayLocal();
    const state = await this.api<{ sigs: Record<string, string> }>('GET', 'site/files');
    const settings = await this.call<any>('settings.get');
    const full = settings?.site?.full_photos ? 1 : 0;
    const { lockFiles, lockSig, lockActive, parseDataJs, splitRecords } = await import('../sitelock');
    const lsig = lockSig(settings?.site?.lock);
    // photos that will be encrypted (they are sent again when that changes)
    let enc = new Set<number>();
    if (lockActive(settings?.site?.lock)) {
      const z0 = unzipSync(this.engine.export_site(['.keep'], [new Uint8Array(0)], () => undefined, now, today));
      enc = splitRecords(parseDataJs(z0['data.js']).records || [], settings.site.lock).media;
    }
    const meta = new Map((await this.call<{ id: number; size: number; width: number; height: number; has_thumb: boolean; mime: string }[]>('media.list')).map((m) => [m.id, m]));
    const ids = Array.from(this.engine.site_media(now, today)) as number[];
    const sigs: Record<string, string> = {};
    for (const id of ids) {
      const m = meta.get(id);
      sigs[id] = m ? `${m.size}:${m.width}x${m.height}:${m.has_thumb ? 1 : 0}:${full}:${this.cfg.gen}${enc.has(id) ? ':' + lsig : ''}` : '';
    }
    const need = ids.filter((id) => state.sigs?.[id] !== sigs[id]);
    let fetched = 0;
    // without full photos the website gets thumbnails only: no need to fetch the originals
    const original = (id: number) => !!full || !meta.get(id)?.has_thumb || !String(meta.get(id)?.mime || '').startsWith('image/');
    const blobs = await this.loadBlobs(need, () => onProgress?.({ stage: 'prepare', done: ++fetched, total: need.length }), original);
    const zip = this.engine.export_site(['.keep'], [new Uint8Array(0)], (id: number, thumb: boolean) => blobs.get(thumb ? `${id}_t` : `${id}`), now, today);
    blobs.clear();
    const files = await lockFiles(unzipSync(zip), settings?.site?.lock, { title: settings?.site?.title || settings?.tree_name || '', subtitle: settings?.site?.subtitle || '' });
    const names = Object.keys(files).filter((n) => n !== '.keep' && n !== 'data.js');
    const total = names.length + 1;
    let done = 0;
    onProgress?.({ stage: 'upload', done, total });
    const token = (await this.api<{ token: string }>('POST', 'site/begin')).token;
    await this.whileBusy(() =>
      this.pool(names, async (n) => {
        await this.upload('site/file', files[n], { token, path: n });
        onProgress?.({ stage: 'upload', done: ++done, total });
      }),
    );
    // the data last: the website switches to the new version in one step
    await this.upload('site/file', files['data.js'], { token, path: 'data.js' });
    await this.api('POST', 'site/finish', { token, sigs });
    this.cfg.published = Date.now();
    onProgress?.({ stage: 'done', done: total, total });
  }

  /** Suggestions sent from the public website wait in WordPress; bring them in. */
  private async collectSuggestions() {
    const inbox = await this.api<{ codes: string[]; ids: string[] }>('GET', 'suggestions');
    if (!inbox?.codes?.length) return;
    await this.call('suggestions.import', { text: inbox.codes.join('\n') });
    await this.flush();
    if (!this.pending.records.length) await this.api('POST', 'suggestions/done', { ids: inbox.ids });
  }

  /* ------------------------------------------------------------------ */

  async saveFile(name: string, data: Uint8Array | string, mime: string): Promise<string | null> {
    downloadBlob(name, data, mime);
    return name;
  }

  async fetchBytes(url: string, headers?: Record<string, string>) {
    // this site (photos of the 2.x plugin): with the login, so private files work too
    const own = new URL(url, location.href).origin === location.origin;
    const r = await fetch(url, own ? { headers: { 'X-WP-Nonce': this.cfg.nonce, ...headers }, credentials: 'same-origin' } : { headers });
    return { status: r.status, bytes: new Uint8Array(await r.arrayBuffer()), type: r.headers.get('content-type') || '' };
  }

  /** The 2.x plugin data on this site (MyRoots → Backup), as its JSON backup. */
  async legacyExport(): Promise<string> {
    const r = await fetch(this.url('legacy'), { credentials: 'same-origin', headers: { 'X-WP-Nonce': this.cfg.nonce } });
    if (!r.ok) throw new WpError('wp_legacy', `WordPress: HTTP ${r.status}`);
    return r.text();
  }

  async notify(title: string, body: string) {
    if (!('Notification' in window)) return;
    if (Notification.permission === 'default') await Notification.requestPermission();
    if (Notification.permission === 'granted') new Notification(title, { body, icon: this.cfg.assets + 'icon.svg' });
  }
}
