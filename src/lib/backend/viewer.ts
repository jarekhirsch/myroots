// Read-only family website: the engine runs on the exported public data.
import { WasmEngine } from '../wasm/myroots_wasm.js';
import { initWasm } from './wasm';
import type { Backend } from './types';
import { extForMime, todayLocal } from './util';

declare const __APP_VERSION__: string;

export class ViewerBackend implements Backend {
  platform = 'viewer' as const;
  readonly = true;
  private engine!: WasmEngine;
  private mimes = new Map<number, string>();

  static async create(): Promise<ViewerBackend> {
    await initWasm();
    const site = (window as any).MYROOTS_SITE;
    if (!site) throw new Error('data.js is missing');
    const b = new ViewerBackend();
    for (const r of site.records) if (r.kind === 'media' && r.data) b.mimes.set(r.id, r.data.mime);
    b.engine = new WasmEngine(JSON.stringify(site.records), __APP_VERSION__, true, Date.now());
    return b;
  }

  async call<T>(method: string, params: unknown = {}): Promise<T> {
    return JSON.parse(this.engine.call(method, JSON.stringify(params ?? {}), Date.now(), todayLocal())) as T;
  }

  async addMedia(): Promise<number> {
    throw new Error('read-only');
  }

  mediaUrl(id: number, thumb = true): string {
    if (!id) return '';
    return thumb ? `media/${id}_t.jpg` : `media/${id}.${extForMime(this.mimes.get(id) || '')}`;
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
