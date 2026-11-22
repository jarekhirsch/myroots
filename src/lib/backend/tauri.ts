// Desktop / mobile backend: Tauri commands backed by the native Rust engine + SQLite.
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import type { Backend, MediaInput, TreesApi } from './types';
import { slug, stamp } from './util';

function bytes(r: ArrayBuffer | number[] | Uint8Array | null): Uint8Array {
  if (!r) return new Uint8Array();
  if (r instanceof Uint8Array) return r;
  return r instanceof ArrayBuffer ? new Uint8Array(r) : new Uint8Array(r as number[]);
}

/** Links to websites and e-mail open in the system browser / mail app, not in the app window. */
function externalLinks() {
  document.addEventListener(
    'click',
    async (e) => {
      const a = (e.target as HTMLElement)?.closest?.('a') as HTMLAnchorElement | null;
      if (!a) return;
      const href = a.getAttribute('href') || '';
      if (!/^(https?:|mailto:)/i.test(href)) return;
      e.preventDefault();
      const { openUrl } = await import('@tauri-apps/plugin-opener');
      openUrl(href).catch(() => {});
    },
    true,
  );
  // mailto: assigned to location (suggestion form) is handled the same way
}

export class TauriBackend implements Backend {
  platform = 'tauri' as const;
  readonly = false;
  /** Open tree; part of media URLs because the media protocol caches forever. */
  private tree = 'default';
  trees: TreesApi = {
    list: () => invoke('trees_list'),
    create: (name) => invoke('tree_create', { name }),
    rename: (id, name) => invoke('tree_rename', { id, name }),
    remove: (id) => invoke('tree_delete', { id }),
    open: (id) => invoke('tree_open', { id }),
  };

  static async create(): Promise<TauriBackend> {
    const b = new TauriBackend();
    await invoke('engine_open');
    try {
      b.tree = (await b.trees.list()).current;
    } catch {
      /* older core: a single tree */
    }
    externalLinks();
    return b;
  }

  async call<T>(method: string, params: unknown = {}): Promise<T> {
    return (await invoke('engine_call', { method, params: params ?? {} })) as T;
  }

  async addMedia(m: MediaInput): Promise<number> {
    const { id } = await this.call<{ id: number }>('media.reserve');
    await invoke('media_put', m.original, { headers: { 'x-id': String(id), 'x-thumb': '0' } });
    if (m.thumb) await invoke('media_put', m.thumb, { headers: { 'x-id': String(id), 'x-thumb': '1' } });
    await this.call('media.register', {
      media: { id, file_name: m.fileName, mime: m.mime, size: m.original.length, width: m.width, height: m.height, has_thumb: !!m.thumb, caption: m.caption || '', origin: m.origin || '' },
    });
    return id;
  }

  mediaUrl(id: number, thumb = true): string {
    if (!id) return '';
    const u = convertFileSrc(thumb ? `${id}_t` : `${id}`, 'media');
    return this.tree === 'default' ? u : `${u}?t=${encodeURIComponent(this.tree)}`;
  }

  async mediaBytes(id: number, thumb = false): Promise<Uint8Array | null> {
    const b = bytes(await invoke<ArrayBuffer>('media_get', { id, thumb }));
    return b.length ? b : null;
  }

  /** Asks where to save, then writes the bytes there (works with Android/iOS document pickers too). */
  private async saveAs(name: string, ext: string, label: string, data: () => Promise<Uint8Array>): Promise<string | null> {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const path = await save({ defaultPath: name, filters: [{ name: label, extensions: [ext] }] });
    if (!path) return null;
    const { writeFile } = await import('@tauri-apps/plugin-fs');
    await writeFile(path, await data());
    return path;
  }

  async exportArchive(): Promise<string | null> {
    const info = await this.call<any>('app.info');
    return this.saveAs(`${slug(info.tree_name || 'myroots')}-${stamp()}.myroots`, 'myroots', 'MyRoots', async () => bytes(await invoke<ArrayBuffer>('archive_bytes')));
  }

  async importArchive(data: Uint8Array): Promise<void> {
    await invoke('archive_import', data);
  }

  async exportSite(): Promise<string | null> {
    const info = await this.call<any>('app.info');
    return this.saveAs(`${slug(info.site?.title || info.tree_name || 'family')}-website.zip`, 'zip', 'ZIP', async () => {
      const { lockZip } = await import('../sitelock');
      return lockZip(bytes(await invoke<ArrayBuffer>('site_bytes')), await this.call('settings.get'));
    });
  }

  async saveFile(name: string, data: Uint8Array | string, _mime: string): Promise<string | null> {
    const ext = name.split('.').pop() || 'txt';
    return this.saveAs(name, ext, ext.toUpperCase(), async () => (typeof data === 'string' ? new TextEncoder().encode(data) : data));
  }

  async print() {
    try {
      await invoke('print_page');
    } catch {
      window.print();
    }
  }

  async fetchBytes(url: string, headers?: Record<string, string>) {
    const { fetch } = await import('@tauri-apps/plugin-http');
    // map services (OpenStreetMap Nominatim) refuse requests without an identification
    const r = await fetch(url, { headers: { 'User-Agent': 'MyRoots/3 (family tree app; https://hirsch.lt)', ...headers } });
    return { status: r.status, bytes: new Uint8Array(await r.arrayBuffer()), type: r.headers.get('content-type') || '' };
  }

  async notify(title: string, body: string) {
    const n = await import('@tauri-apps/plugin-notification');
    let ok = await n.isPermissionGranted();
    if (!ok) ok = (await n.requestPermission()) === 'granted';
    if (ok) n.sendNotification({ title, body });
  }
}
