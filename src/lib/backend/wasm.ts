// Browser backend: the Rust engine compiled to WebAssembly + IndexedDB storage.
import init, { WasmEngine } from '../wasm/myroots_wasm.js';
import wasmUrl from '../wasm/myroots_wasm_bg.wasm?url';
import type { Backend, MediaInput } from './types';
import * as idb from './idb';
import { downloadBlob, stamp, slug, todayLocal } from './util';

declare const __APP_VERSION__: string;

let ready: Promise<unknown> | null = null;
export function initWasm() {
  if (!ready) ready = init({ module_or_path: wasmUrl });
  return ready;
}

export function decodeGedcom(bytes: Uint8Array): string {
  return WasmEngine.decode_gedcom(bytes);
}

export class WasmBackend implements Backend {
  platform = 'web' as const;
  readonly = false;
  private engine!: WasmEngine;
  private queue: Promise<void> = Promise.resolve();
  private urls = new Map<string, string>();

  static async create(): Promise<WasmBackend> {
    await initWasm();
    const b = new WasmBackend();
    const records = await idb.getAll('records');
    b.engine = new WasmEngine(JSON.stringify(records), __APP_VERSION__, false, Date.now());
    if (!records.length) {
      // brand-new tree: persist the initial meta / settings records
      b.persist();
    }
    if (navigator.storage?.persist) navigator.storage.persist().catch(() => {});
    return b;
  }

  private persist() {
    const changes = JSON.parse(this.engine.take_changes()) as { records: { kind: string; id: number; data: any }[]; deleted_media: number[] };
    if (!changes.records.length && !changes.deleted_media.length) return;
    const recOps: [string, any][] = changes.records.map((r) => [`${r.kind}:${r.id}`, r.data == null ? undefined : r]);
    const blobOps: [string, any][] = [];
    for (const id of changes.deleted_media) {
      blobOps.push([`${id}`, undefined], [`${id}_t`, undefined]);
      for (const k of [`${id}`, `${id}_t`]) {
        const u = this.urls.get(k);
        if (u) URL.revokeObjectURL(u);
        this.urls.delete(k);
      }
    }
    this.queue = this.queue.then(() => idb.write('records', recOps)).then(() => idb.write('blobs', blobOps)).catch((e) => console.error('IndexedDB write failed', e));
  }

  async call<T>(method: string, params: unknown = {}): Promise<T> {
    const out = this.engine.call(method, JSON.stringify(params ?? {}), Date.now(), todayLocal());
    this.persist();
    return JSON.parse(out) as T;
  }

  async flush() {
    await this.queue;
  }

  async addMedia(m: MediaInput): Promise<number> {
    const { id } = await this.call<{ id: number }>('media.reserve');
    const ops: [string, any][] = [[`${id}`, m.original]];
    if (m.thumb) ops.push([`${id}_t`, m.thumb]);
    await idb.write('blobs', ops);
    await this.call('media.register', {
      media: { id, file_name: m.fileName, mime: m.mime, size: m.original.length, width: m.width, height: m.height, has_thumb: !!m.thumb, caption: m.caption || '', origin: m.origin || '' },
    });
    return id;
  }

  mediaUrl(id: number, thumb = true): string {
    return this.urls.get(thumb ? `${id}_t` : `${id}`) || this.urls.get(`${id}`) || '';
  }

  async ensureMedia(ids: number[]): Promise<void> {
    const need = ids.filter((id) => id && !this.urls.has(`${id}_t`) && !this.urls.has(`${id}`));
    await Promise.all(
      need.map(async (id) => {
        const t = await idb.get('blobs', `${id}_t`);
        if (t) this.urls.set(`${id}_t`, URL.createObjectURL(new Blob([t])));
        const o = t ? null : await idb.get('blobs', `${id}`);
        if (o) this.urls.set(`${id}`, URL.createObjectURL(new Blob([o])));
      }),
    );
  }

  async originalUrl(id: number): Promise<string> {
    const k = `${id}`;
    if (!this.urls.has(k)) {
      const o = await idb.get('blobs', k);
      if (o) this.urls.set(k, URL.createObjectURL(new Blob([o])));
    }
    return this.urls.get(k) || '';
  }

  async mediaBytes(id: number, thumb = false): Promise<Uint8Array | null> {
    const v = await idb.get('blobs', thumb ? `${id}_t` : `${id}`);
    return v ? new Uint8Array(v) : null;
  }

  private async loadAllBlobs(ids?: number[]): Promise<Map<string, Uint8Array>> {
    const map = new Map<string, Uint8Array>();
    const keys = ids ? ids.flatMap((i) => [`${i}`, `${i}_t`]) : await idb.keys('blobs');
    for (const k of keys) {
      const v = await idb.get('blobs', k);
      if (v) map.set(k, new Uint8Array(v));
    }
    return map;
  }

  async exportArchive(): Promise<string | null> {
    await this.flush();
    const blobs = await this.loadAllBlobs();
    const bytes = this.engine.export_archive((id: number, thumb: boolean) => blobs.get(thumb ? `${id}_t` : `${id}`), Date.now());
    const info = await this.call<any>('app.info');
    const name = `${slug(info.tree_name || 'myroots')}-${stamp()}.myroots`;
    downloadBlob(name, bytes, 'application/zip');
    return name;
  }

  async importArchive(bytes: Uint8Array): Promise<void> {
    await this.flush();
    const media: [string, any][] = [];
    const recordsJson = this.engine.import_archive(bytes, (id: number, thumb: boolean, b: Uint8Array) => {
      media.push([thumb ? `${id}_t` : `${id}`, b.slice()]);
    });
    for (const u of this.urls.values()) URL.revokeObjectURL(u);
    this.urls.clear();
    await idb.clear('records');
    await idb.clear('blobs');
    const recs = JSON.parse(recordsJson) as { kind: string; id: number }[];
    await idb.write('records', recs.map((r) => [`${r.kind}:${r.id}`, r]));
    for (let i = 0; i < media.length; i += 50) await idb.write('blobs', media.slice(i, i + 50));
  }

  async exportSite(): Promise<string | null> {
    await this.flush();
    const list: string[] = await fetch('./viewer/files.json').then((r) => r.json());
    const files: Uint8Array[] = [];
    for (const f of list) files.push(new Uint8Array((await fetch(`./viewer/${f}`).then((r) => r.arrayBuffer())) as ArrayBuffer));
    const ids = Array.from(this.engine.site_media(Date.now(), todayLocal()));
    const blobs = await this.loadAllBlobs(ids);
    const zip = this.engine.export_site(list, files, (id: number, thumb: boolean) => blobs.get(thumb ? `${id}_t` : `${id}`), Date.now(), todayLocal());
    const info = await this.call<any>('app.info');
    const name = `${slug(info.site?.title || info.tree_name || 'family')}-website.zip`;
    downloadBlob(name, zip, 'application/zip');
    return name;
  }

  async saveFile(name: string, data: Uint8Array | string, mime: string): Promise<string | null> {
    downloadBlob(name, data, mime);
    return name;
  }

  async fetchBytes(url: string, headers?: Record<string, string>) {
    const r = await fetch(url, { headers });
    return { status: r.status, bytes: new Uint8Array(await r.arrayBuffer()), type: r.headers.get('content-type') || '' };
  }

  async notify(title: string, body: string) {
    if (!('Notification' in window)) return;
    if (Notification.permission === 'default') await Notification.requestPermission();
    if (Notification.permission === 'granted') new Notification(title, { body, icon: './icon.svg' });
  }
}
