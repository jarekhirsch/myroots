// Brings the photos and scans of a WordPress MyRoots site into the app.
// The JSON backup of the plugin only references attachment ids: we ask the
// site's REST API for each attachment URL and download the file. When the
// browser cannot download a file (CORS), the user can pick the files from
// the site's uploads folder and they are matched by file name.
import { call } from './api.svelte';
import { backend } from './backend';
import { prepareImage } from './media';
import type { WpPending } from './types';

export interface WpProgress {
  done: number;
  total: number;
  ok: number;
  failed: { wp_id: number; file: string; url: string }[];
}

function baseName(url: string): string {
  return decodeURIComponent((url.split('?')[0].split('/').pop() || '').toLowerCase());
}

/** Variants of a WordPress file name: original, "-scaled", size suffixes. */
function keys(name: string): string[] {
  const n = name.toLowerCase();
  const plain = n.replace(/-scaled(\.[a-z0-9]+)$/, '$1').replace(/-\d+x\d+(\.[a-z0-9]+)$/, '$1');
  return [...new Set([n, plain])];
}

async function store(bytes: Uint8Array, fileName: string, mime: string, origin: string, caption: string): Promise<number> {
  if (mime.startsWith('image/') && !/svg/.test(mime)) {
    return backend.addMedia(await prepareImage(new Blob([bytes as BlobPart], { type: mime }), fileName, { origin, caption }));
  }
  // PDFs and other documents: stored as they are, without a thumbnail.
  return backend.addMedia({ original: bytes, thumb: null, fileName, mime: mime || 'application/octet-stream', width: 0, height: 0, origin, caption });
}

export async function downloadWpMedia(onProgress: (p: WpProgress) => void, isCancelled: () => boolean, siteOverride = ''): Promise<WpProgress> {
  const pend = await call<WpPending>('wp.pending');
  const site = (siteOverride || pend.site).replace(/\/+$/, '');
  const prog: WpProgress = { done: 0, total: pend.items.length, ok: 0, failed: [] };
  onProgress({ ...prog, failed: [...prog.failed] });
  for (const it of pend.items) {
    if (isCancelled()) break;
    let url = '';
    let file = '';
    try {
      const meta = await backend.fetchBytes(`${site}/wp-json/wp/v2/media/${it.wp_id}?_fields=source_url,mime_type,caption,alt_text,title`, { Accept: 'application/json' });
      if (meta.status !== 200) throw new Error('meta ' + meta.status);
      const m = JSON.parse(new TextDecoder().decode(meta.bytes));
      url = m.source_url || '';
      file = baseName(url);
      const caption = (m.caption?.rendered || '').replace(/<[^>]+>/g, '').trim() || m.alt_text || '';
      const r = await backend.fetchBytes(url);
      if (r.status !== 200 || !r.bytes.length) throw new Error('file ' + r.status);
      const id = await store(r.bytes, file, (r.type || m.mime_type || '').split(';')[0], url, caption);
      await call('wp.attach', { wp_id: it.wp_id, media: id });
      prog.ok++;
    } catch {
      prog.failed.push({ wp_id: it.wp_id, file, url });
    }
    prog.done++;
    onProgress({ ...prog, failed: [...prog.failed] });
  }
  return prog;
}

/** Attaches local files (from wp-content/uploads) to the failed attachments by file name. */
export async function attachLocalFiles(failed: WpProgress['failed'], files: File[]): Promise<number> {
  const byName = new Map<string, File>();
  for (const f of files) for (const k of keys(f.name)) if (!byName.has(k)) byName.set(k, f);
  let n = 0;
  for (const it of failed) {
    if (!it.file) continue;
    const f = keys(it.file).map((k) => byName.get(k)).find(Boolean);
    if (!f) continue;
    try {
      const id = await store(new Uint8Array(await f.arrayBuffer()), f.name, f.type, it.url, '');
      await call('wp.attach', { wp_id: it.wp_id, media: id });
      n++;
    } catch {
      /* skip */
    }
  }
  return n;
}
