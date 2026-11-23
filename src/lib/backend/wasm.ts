// Browser backend: the Rust engine compiled to WebAssembly + IndexedDB storage.
import { buildBackup, openBackup } from '../archivejs';
import init, { WasmEngine } from '../wasm/myroots_wasm.js';
import wasmUrl from '../wasm/myroots_wasm_bg.wasm?url';
import type { Backend, MediaInput, TreeList, TreesApi } from './types';
import * as idb from './idb';
import { assetUrl } from './assets';
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

// The list of trees in this browser (the trees themselves are IndexedDB databases).
const REG_KEY = 'myroots.trees';

function readRegistry(): TreeList {
  let r: TreeList | null = null;
  try {
    r = JSON.parse(localStorage.getItem(REG_KEY) || 'null');
  } catch {
    r = null;
  }
  if (!r || !Array.isArray(r.trees)) r = { current: 'default', trees: [] };
  if (!r.trees.length) r.trees.push({ id: 'default', name: '', created: 0 });
  if (!r.trees.some((t) => t.id === r!.current)) r.current = r.trees[0].id;
  return r;
}

function writeRegistry(r: TreeList) {
  try {
    localStorage.setItem(REG_KEY, JSON.stringify(r));
  } catch {
    /* private mode: the list lives only until the tab closes */
  }
}

const webTrees: TreesApi = {
  async list() {
    return readRegistry();
  },
  async create(name) {
    const r = readRegistry();
    let id = `t${Date.now()}`;
    while (r.trees.some((t) => t.id === id)) id += 'x';
    r.trees.push({ id, name: name.trim(), created: Date.now() });
    writeRegistry(r);
    return id;
  },
  async rename(id, name) {
    const r = readRegistry();
    const t = r.trees.find((t) => t.id === id);
    if (t) t.name = name.trim();
    writeRegistry(r);
  },
  async remove(id) {
    const r = readRegistry();
    if (r.current === id) throw new Error('open another tree first');
    await idb.deleteDb(id);
    r.trees = r.trees.filter((t) => t.id !== id);
    writeRegistry(r);
  },
  async open(id) {
    const r = readRegistry();
    if (!r.trees.some((t) => t.id === id)) throw new Error('no such tree');
    r.current = id;
    writeRegistry(r);
  },
};

export class WasmBackend implements Backend {
  platform = 'web' as const;
  readonly = false;
  trees = webTrees;
  private engine!: WasmEngine;
  private queue: Promise<void> = Promise.resolve();
  private urls = new Map<string, string>();

  static async create(): Promise<WasmBackend> {
    await initWasm();
    idb.useTree(readRegistry().current);
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
    this.queue = this.queue.then(() => idb.write('records', recOps)).then(() => idb.write('blobs', blobOps)).catch((e) => {
        console.error('IndexedDB write failed', e);
        // the change stays in memory only: tell the person (App shows it)
        window.dispatchEvent(new CustomEvent('myroots-save-failed', { detail: e }));
      });
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

  async exportArchive(onProgress?: (done: number, total: number) => void): Promise<string | null> {
    await this.flush();
    // the engine writes the records; the photos are added one at a time (never all in memory)
    const records = this.engine.export_archive(() => undefined, Date.now());
    const ids = (await this.call<{ id: number }[]>('media.list')).map((m) => m.id);
    const blob = await buildBackup(records, ids, async (id, thumb) => {
      const v = await idb.get('blobs', thumb ? `${id}_t` : `${id}`);
      return v ? new Uint8Array(v) : null;
    }, onProgress);
    const info = await this.call<any>('app.info');
    const name = `${slug(info.tree_name || 'myroots')}-${stamp()}.myroots`;
    downloadBlob(name, blob, 'application/zip');
    return name;
  }

  async importArchive(file: Blob, onProgress?: (done: number, total: number) => void): Promise<void> {
    await this.flush();
    // records checked by the engine first: a wrong file leaves the tree untouched
    const opened = await openBackup(file);
    const recordsJson = this.engine.import_archive(opened ? opened.records : new Uint8Array(await file.arrayBuffer()), () => {});
    for (const u of this.urls.values()) URL.revokeObjectURL(u);
    this.urls.clear();
    await idb.clear('records');
    await idb.clear('blobs');
    const recs = JSON.parse(recordsJson) as { kind: string; id: number }[];
    await idb.write('records', recs.map((r) => [`${r.kind}:${r.id}`, r]));
    const photos = opened?.photos || [];
    for (let i = 0; i < photos.length; i += 20) {
      onProgress?.(i, photos.length);
      const batch = photos.slice(i, i + 20);
      await idb.write('blobs', await Promise.all(batch.map(async (p) => [p.thumb ? `${p.id}_t` : `${p.id}`, await p.read()] as [string, any])));
    }
    onProgress?.(photos.length, photos.length);
  }

  async exportSite(): Promise<string | null> {
    await this.flush();
    const list: string[] = await fetch('./viewer/files.json').then((r) => r.json());
    const files: Uint8Array[] = [];
    for (const f of list) files.push(new Uint8Array((await fetch(`./viewer/${f}`).then((r) => r.arrayBuffer())) as ArrayBuffer));
    const ids = Array.from(this.engine.site_media(Date.now(), todayLocal()));
    const blobs = await this.loadAllBlobs(ids);
    const { lockZip } = await import('../sitelock');
    const zip = await lockZip(this.engine.export_site(list, files, (id: number, thumb: boolean) => blobs.get(thumb ? `${id}_t` : `${id}`), Date.now(), todayLocal()), await this.call('settings.get'));
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
    if (Notification.permission === 'granted') new Notification(title, { body, icon: assetUrl('icon.svg') });
  }
}
