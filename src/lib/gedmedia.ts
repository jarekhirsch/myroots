// Photos and scans named in a GEDCOM file (OBJE … FILE): read from the
// .gdz / .zip archive the file came in, or from a folder the user picks,
// added to the tree and linked to their people and sources.
import { unzipSync } from 'fflate';
import { backend } from './backend';
import { call } from './api.svelte';
import { prepareImage } from './media';
import type { GedcomMedia, Media } from './types';

const IMG = /\.(jpe?g|png|gif|webp|bmp)$/i;
const MIME: Record<string, string> = { jpg: 'image/jpeg', jpeg: 'image/jpeg', png: 'image/png', gif: 'image/gif', webp: 'image/webp', bmp: 'image/bmp' };

/** "C:\Zdjęcia\Anna.JPG", "./media/anna.jpg", "file:///…" → "zdjęcia/anna.jpg" style key. */
export function pathKey(p: string): string {
  let k = (p || '').trim().replace(/\\/g, '/').replace(/^file:\/\/+/i, '');
  try {
    k = decodeURIComponent(k);
  } catch {
    /* keep as written */
  }
  return k.replace(/^[a-z]:\//i, '').replace(/^(\.\/)+/, '').replace(/^\/+/, '').toLowerCase();
}
const baseName = (k: string) => k.slice(k.lastIndexOf('/') + 1);

/** Image files of a zip archive, by path. */
export function imagesFromZip(bytes: Uint8Array): Map<string, Blob> {
  const out = new Map<string, Blob>();
  let files: Record<string, Uint8Array> = {};
  try {
    files = unzipSync(bytes, { filter: (f) => IMG.test(f.name) && f.originalSize < 60_000_000 });
  } catch {
    return out;
  }
  for (const [name, data] of Object.entries(files)) {
    const ext = name.slice(name.lastIndexOf('.') + 1).toLowerCase();
    out.set(name, new Blob([data as BlobPart], { type: MIME[ext] || 'image/jpeg' }));
  }
  return out;
}

/** Files of a folder chosen with a file input (webkitRelativePath kept). */
export function imagesFromFiles(files: File[]): Map<string, Blob> {
  const out = new Map<string, Blob>();
  for (const f of files) if (IMG.test(f.name)) out.set((f as any).webkitRelativePath || f.name, f);
  return out;
}

/** Finds the file of every reference: the same path, a path ending the same way, or the same file name. */
function matcher(files: Map<string, Blob>) {
  const byKey = new Map<string, [string, Blob]>();
  const byBase = new Map<string, [string, Blob] | null>();
  for (const [name, blob] of files) {
    const k = pathKey(name);
    byKey.set(k, [k, blob]);
    const b = baseName(k);
    byBase.set(b, byBase.has(b) ? null : [k, blob]); // null: the name is not unique
  }
  return (ref: string): [string, Blob] | null => {
    const k = pathKey(ref);
    if (byKey.has(k)) return byKey.get(k)!;
    for (const [key, v] of byKey) if (k.endsWith('/' + key) || key.endsWith('/' + k)) return v;
    return byBase.get(baseName(k)) || null;
  };
}

export interface MediaImport {
  linked: number;
  /** Files named in the GEDCOM that were not found (or are not photos). */
  missing: string[];
}

/** Adds the found files (once each, also across imports) and links them. */
export async function attachGedcomMedia(refs: GedcomMedia[], files: Map<string, Blob>, report: (done: number, total: number) => void): Promise<MediaImport> {
  const find = matcher(files);
  const known = new Map<string, number>();
  for (const m of await call<Media[]>('media.list')) if (m.origin?.startsWith('gedcom:')) known.set(m.origin.slice(7), m.id);
  const links: (GedcomMedia & { media: number })[] = [];
  const missing: string[] = [];
  let done = 0;
  for (const r of refs) {
    report(done++, refs.length);
    const hit = find(r.file);
    if (!hit) {
      missing.push(r.file);
      continue;
    }
    const [key, blob] = hit;
    let id = known.get(key);
    if (!id) {
      try {
        id = await backend.addMedia(await prepareImage(blob, baseName(key), { origin: `gedcom:${key}`, caption: r.title || '' }));
      } catch {
        missing.push(r.file);
        continue;
      }
      known.set(key, id);
    }
    links.push({ ...r, media: id });
  }
  report(refs.length, refs.length);
  const { linked } = links.length ? await call<{ linked: number }>('media.link_imported', { links }) : { linked: 0 };
  return { linked, missing };
}
