// Photos and scans named in a GEDCOM file (OBJE … FILE): read from the
// .gdz / .zip archive the file came in, or from a folder the user picks,
// added to the tree and linked to their people and sources.
import { zipBlob, type ZipEntry } from './zipindex';
import { backend } from './backend';
import { call } from './api.svelte';
import { prepareImage } from './media';
import { isStorageFull } from './errors';
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

/** A photo to add: read only when it is needed, so a big archive never sits in memory. */
export type MediaFiles = Map<string, () => Promise<Blob>>;

/** Image files of a zip archive, by path (taken out of the archive one at a time). */
export function imagesFromZip(file: Blob, entries: ZipEntry[]): MediaFiles {
  const out: MediaFiles = new Map();
  for (const e of entries) {
    if (!IMG.test(e.name) || e.size >= 60_000_000) continue;
    const ext = e.name.slice(e.name.lastIndexOf('.') + 1).toLowerCase();
    out.set(e.name, () => zipBlob(file, e, MIME[ext] || 'image/jpeg'));
  }
  return out;
}

/** Files of a folder chosen with a file input (webkitRelativePath kept). */
export function imagesFromFiles(files: File[]): MediaFiles {
  const out: MediaFiles = new Map();
  for (const f of files) if (IMG.test(f.name)) out.set((f as any).webkitRelativePath || f.name, () => Promise.resolve(f));
  return out;
}

/** Finds the file of every reference: the same path, a path ending the same way, or the same file name. */
function matcher(files: MediaFiles) {
  type Hit = [string, () => Promise<Blob>];
  const byKey = new Map<string, Hit>();
  const byBase = new Map<string, Hit | null>();
  for (const [name, open] of files) {
    const k = pathKey(name);
    byKey.set(k, [k, open]);
    const b = baseName(k);
    byBase.set(b, byBase.has(b) ? null : [k, open]); // null: the name is not unique
  }
  return (ref: string): Hit | null => {
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
  /** Files found but not readable as a photo (damaged, unknown format). */
  failed: string[];
}

/** Adds the found files (once each, also across imports) and links them. */
export async function attachGedcomMedia(refs: GedcomMedia[], files: MediaFiles, report: (done: number, total: number) => void): Promise<MediaImport> {
  const find = matcher(files);
  const known = new Map<string, number>();
  for (const m of await call<Media[]>('media.list')) if (m.origin?.startsWith('gedcom:')) known.set(m.origin.slice(7), m.id);
  const links: (GedcomMedia & { media: number })[] = [];
  const missing: string[] = [];
  const failed: string[] = [];
  // ask the browser not to clear the photos when space runs low (ignored where unsupported)
  try {
    await navigator.storage?.persist?.();
  } catch {
    /* not available */
  }
  let done = 0;
  for (const r of refs) {
    report(done++, refs.length);
    const hit = find(r.file);
    if (!hit) {
      missing.push(r.file);
      continue;
    }
    const [key, open] = hit;
    let id = known.get(key);
    if (!id) {
      try {
        id = await backend.addMedia(await prepareImage(await open(), baseName(key), { origin: `gedcom:${key}`, caption: r.title || '' }));
      } catch (e) {
        if (isStorageFull(e)) throw e;
        failed.push(r.file);
        continue;
      }
      known.set(key, id);
    }
    links.push({ ...r, media: id });
  }
  report(refs.length, refs.length);
  const { linked } = links.length ? await call<{ linked: number }>('media.link_imported', { links }) : { linked: 0 };
  return { linked, missing, failed };
}
