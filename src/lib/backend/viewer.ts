// Read-only family website: the engine runs on the exported public data.
import { WasmEngine } from '../wasm/myroots_wasm.js';
import { initWasm } from './wasm';
import type { Backend } from './types';
import { extForMime, todayLocal } from './util';
import { savedKey, openSecret, mergeRecords, decryptBytes, forgetKey, type LockHeader } from '../sitelock';

declare const __APP_VERSION__: string;

export class ViewerBackend implements Backend {
  platform = 'viewer' as const;
  readonly = true;
  private engine!: WasmEngine;
  private mimes = new Map<number, string>();
  /** Password protection: the header from data.js, and the key once unlocked. */
  lock: LockHeader | null = null;
  unlocked = false;
  private key: CryptoKey | null = null;
  private encrypted = new Set<number>();
  private blobs = new Map<string, string>();

  static async create(): Promise<ViewerBackend> {
    await initWasm();
    const site = (window as any).MYROOTS_SITE;
    if (!site) throw new Error('data.js is missing');
    const b = new ViewerBackend();
    let records = site.records || [];
    if (site.lock) {
      b.lock = site.lock as LockHeader;
      b.encrypted = new Set(b.lock.media || []);
      const raw = savedKey(b.lock);
      if (raw) {
        try {
          const sec = await openSecret(b.lock, raw);
          records = mergeRecords(records, sec.records);
          b.key = sec.key;
          b.unlocked = true;
        } catch {
          forgetKey(b.lock); // the password was changed
        }
      }
      if (!b.unlocked && b.lock.mode === 'all') throw new Error('locked');
    }
    for (const r of records) if (r.kind === 'media' && r.data) b.mimes.set(r.id, r.data.mime);
    b.engine = new WasmEngine(JSON.stringify(records), __APP_VERSION__, true, Date.now());
    return b;
  }

  private base(): string {
    return (window as any).MYROOTS_WP_VIEW?.media || 'media/';
  }

  /** An encrypted photo: fetched, decrypted, shown from a blob URL. */
  private async decrypted(id: number, thumb: boolean): Promise<string> {
    const k = thumb ? `${id}_t` : `${id}`;
    const have = this.blobs.get(k);
    if (have) return have;
    if (!this.key) return '';
    try {
      const r = await fetch(`${this.base()}${k}.bin`);
      if (!r.ok) return thumb ? this.decrypted(id, false) : '';
      const plain = await decryptBytes(this.key, new Uint8Array(await r.arrayBuffer()));
      const url = URL.createObjectURL(new Blob([plain as BlobPart], { type: thumb ? 'image/jpeg' : this.mimes.get(id) || 'application/octet-stream' }));
      this.blobs.set(k, url);
      return url;
    } catch {
      return '';
    }
  }

  async ensureMedia(ids: number[]): Promise<void> {
    await Promise.all(ids.filter((id) => this.encrypted.has(id)).map((id) => this.decrypted(id, true)));
  }

  async mediaUrlAsync(id: number, thumb = true): Promise<string> {
    if (!id) return '';
    if (this.encrypted.has(id)) return this.decrypted(id, thumb);
    return this.mediaUrl(id, thumb);
  }

  async call<T>(method: string, params: unknown = {}): Promise<T> {
    const r = JSON.parse(this.engine.call(method, JSON.stringify(params ?? {}), Date.now(), todayLocal()));
    // Published by WordPress: suggestions go to the site (and wait there for the editor).
    const wp = (window as any).MYROOTS_WP_VIEW;
    if (method === 'app.info' && wp?.suggest && r?.suggestions?.enabled) r.suggestions = { ...r.suggestions, endpoint: wp.suggest, email: '' };
    return r as T;
  }

  async addMedia(): Promise<number> {
    throw new Error('read-only');
  }

  mediaUrl(id: number, thumb = true): string {
    if (!id) return '';
    if (this.encrypted.has(id)) return this.blobs.get(thumb ? `${id}_t` : `${id}`) || (thumb ? '' : this.blobs.get(`${id}_t`) || '');
    const base = this.base();
    return thumb ? `${base}${id}_t.jpg` : `${base}${id}.${extForMime(this.mimes.get(id) || '')}`;
  }

  async mediaBytes(): Promise<Uint8Array | null> {
    return null;
  }
  async exportArchive() {
    return null;
  }
  async importArchive() {}
  async exportSite() {
    return null;
  }
  async saveFile(name: string, data: Uint8Array | string, mime: string) {
    const { downloadBlob } = await import('./util');
    downloadBlob(name, data, mime);
    return name;
  }
  async fetchBytes(url: string) {
    const r = await fetch(url);
    return { status: r.status, bytes: new Uint8Array(await r.arrayBuffer()), type: r.headers.get('content-type') || '' };
  }
}
