// .myroots backups built and read piece by piece, so a tree with thousands of
// photos never has to fit in memory at once (phones, browsers). The format is
// the one of the engine (crates/myroots-core/src/archive.rs):
//   myroots.json   the records (written by the engine)
//   media/<id>     original file
//   media/<id>_t   thumbnail
// The engine makes the small archive with the records only; the photos are
// appended here one at a time, as slices of a Blob the browser may keep on disk.
import { zipEntries, zipBlob, type ZipEntry } from './zipindex';

/* ---------------- CRC-32 ---------------- */

const TABLE = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

export function crc32(b: Uint8Array): number {
  let c = 0xffffffff;
  for (let i = 0; i < b.length; i++) c = TABLE[(c ^ b[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

/* ---------------- writer ---------------- */

const MAX32 = 0xffffffff;

function u64(v: DataView, o: number, n: number) {
  v.setUint32(o, n % 2 ** 32, true);
  v.setUint32(o + 4, Math.floor(n / 2 ** 32), true);
}

/** A zip archive assembled from parts (stored entries, or entries copied as they are). */
export class ZipBuilder {
  private parts: BlobPart[] = [];
  private central: Uint8Array[] = [];
  private offset = 0;
  private count = 0;
  private enc = new TextEncoder();

  private push(p: Uint8Array | Blob) {
    this.parts.push(p as BlobPart);
    this.offset += p instanceof Blob ? p.size : p.length;
  }

  /** Adds an entry: `data` already compressed with `method` (0 stored, 8 deflate). */
  addRaw(name: string, data: Uint8Array | Blob, e: { method: number; crc: number; csize: number; size: number }) {
    const nm = this.enc.encode(name);
    const big = e.size >= MAX32 || e.csize >= MAX32 || this.offset >= MAX32;
    // local header (ZIP64 sizes in the extra field when needed)
    const lx = big ? 20 : 0;
    const lh = new DataView(new ArrayBuffer(30 + nm.length + lx));
    lh.setUint32(0, 0x04034b50, true);
    lh.setUint16(4, big ? 45 : 20, true);
    lh.setUint16(6, 0x800, true); // names in UTF-8
    lh.setUint16(8, e.method, true);
    lh.setUint32(10, 0x00210000, true); // 1980-01-01 00:00
    lh.setUint32(14, e.crc, true);
    lh.setUint32(18, big ? MAX32 : e.csize, true);
    lh.setUint32(22, big ? MAX32 : e.size, true);
    lh.setUint16(26, nm.length, true);
    lh.setUint16(28, lx, true);
    new Uint8Array(lh.buffer).set(nm, 30);
    if (big) {
      lh.setUint16(30 + nm.length, 0x0001, true);
      lh.setUint16(32 + nm.length, 16, true);
      u64(lh, 34 + nm.length, e.size);
      u64(lh, 42 + nm.length, e.csize);
    }
    const at = this.offset;
    this.push(new Uint8Array(lh.buffer));
    this.push(data);
    // central directory record
    const cx = big ? 28 : 0;
    const ch = new DataView(new ArrayBuffer(46 + nm.length + cx));
    ch.setUint32(0, 0x02014b50, true);
    ch.setUint16(4, big ? 45 : 20, true);
    ch.setUint16(6, big ? 45 : 20, true);
    ch.setUint16(8, 0x800, true);
    ch.setUint16(10, e.method, true);
    ch.setUint32(12, 0x00210000, true);
    ch.setUint32(16, e.crc, true);
    ch.setUint32(20, big ? MAX32 : e.csize, true);
    ch.setUint32(24, big ? MAX32 : e.size, true);
    ch.setUint16(28, nm.length, true);
    ch.setUint16(30, cx, true);
    ch.setUint32(42, big ? MAX32 : at, true);
    new Uint8Array(ch.buffer).set(nm, 46);
    if (big) {
      ch.setUint16(46 + nm.length, 0x0001, true);
      ch.setUint16(48 + nm.length, 24, true);
      u64(ch, 50 + nm.length, e.size);
      u64(ch, 58 + nm.length, e.csize);
      u64(ch, 66 + nm.length, at);
    }
    this.central.push(new Uint8Array(ch.buffer));
    this.count++;
  }

  /** Adds a file without compression (photos are compressed already). */
  add(name: string, data: Uint8Array) {
    this.addRaw(name, new Blob([data as BlobPart]), { method: 0, crc: crc32(data), csize: data.length, size: data.length });
  }

  finish(type = 'application/zip'): Blob {
    const cdAt = this.offset;
    for (const c of this.central) this.push(c);
    const cdSize = this.offset - cdAt;
    const z64 = this.count >= 0xffff || cdAt >= MAX32 || cdSize >= MAX32;
    if (z64) {
      const r = new DataView(new ArrayBuffer(56 + 20));
      r.setUint32(0, 0x06064b50, true);
      u64(r, 4, 44);
      r.setUint16(12, 45, true);
      r.setUint16(14, 45, true);
      u64(r, 24, this.count);
      u64(r, 32, this.count);
      u64(r, 40, cdSize);
      u64(r, 48, cdAt);
      // locator
      r.setUint32(56, 0x07064b50, true);
      u64(r, 64, this.offset);
      r.setUint32(72, 1, true);
      this.push(new Uint8Array(r.buffer));
    }
    const e = new DataView(new ArrayBuffer(22));
    e.setUint32(0, 0x06054b50, true);
    e.setUint16(8, z64 ? 0xffff : this.count, true);
    e.setUint16(10, z64 ? 0xffff : this.count, true);
    e.setUint32(12, z64 ? MAX32 : cdSize, true);
    e.setUint32(16, z64 ? MAX32 : cdAt, true);
    this.push(new Uint8Array(e.buffer));
    return new Blob(this.parts, { type });
  }
}

/* ---------------- .myroots ---------------- */

/**
 * The backup: `records` is the archive the engine wrote without photos;
 * `photo(id, thumb)` returns one stored file at a time.
 */
export async function buildBackup(
  records: Uint8Array,
  ids: number[],
  photo: (id: number, thumb: boolean) => Promise<Uint8Array | null | undefined>,
  onProgress?: (done: number, total: number) => void,
): Promise<Blob> {
  const small = new Blob([records as BlobPart]);
  const z = new ZipBuilder();
  for (const e of (await zipEntries(small)) || []) z.addRaw(e.name, await rawData(small, e), e);
  // a few photos are fetched ahead (a server answers faster in parallel); written in order
  const pair = (id: number) => Promise.all([photo(id, false), photo(id, true)]);
  const ahead: Promise<[Uint8Array | null | undefined, Uint8Array | null | undefined]>[] = [];
  let next = 0;
  for (let done = 0; done < ids.length; done++) {
    while (next < ids.length && ahead.length < 4) ahead.push(pair(ids[next++]));
    onProgress?.(done, ids.length);
    const [o, t] = await ahead.shift()!;
    if (o) z.add(`media/${ids[done]}`, o);
    if (t) z.add(`media/${ids[done]}_t`, t);
  }
  onProgress?.(ids.length, ids.length);
  return z.finish();
}

/** The compressed bytes of an entry, as they are in the archive. */
async function rawData(file: Blob, e: ZipEntry): Promise<Blob> {
  const head = new DataView(await file.slice(e.offset, e.offset + 30).arrayBuffer());
  const start = e.offset + 30 + head.getUint16(26, true) + head.getUint16(28, true);
  return file.slice(start, start + e.csize);
}

export interface BackupPhoto {
  id: number;
  thumb: boolean;
  size: number;
  read: () => Promise<Uint8Array>;
}

/** Largest file taken from a backup (as in the engine: protection against "zip bombs"). */
const LIMIT = 2 ** 30;

/**
 * Opens a backup without reading it whole: `records` is a small archive with
 * only myroots.json (for the engine), `photos` are read one at a time.
 * Returns null when the file is not a readable zip (the engine then reports why).
 */
export async function openBackup(file: Blob): Promise<{ records: Uint8Array; photos: BackupPhoto[] } | null> {
  const entries = await zipEntries(file);
  const json = entries?.find((e) => e.name === 'myroots.json');
  if (!entries || !json) return null;
  const z = new ZipBuilder();
  z.addRaw(json.name, await rawData(file, json), json);
  const records = new Uint8Array(await z.finish().arrayBuffer());
  const photos: BackupPhoto[] = [];
  for (const e of entries) {
    const m = /^media\/(\d+)(_t)?$/.exec(e.name);
    if (!m) continue;
    if (e.size > LIMIT) throw Object.assign(new Error('a file in the archive is too large'), { code: 'archive_invalid' });
    photos.push({ id: Number(m[1]), thumb: !!m[2], size: e.size, read: async () => new Uint8Array(await (await zipBlob(file, e)).arrayBuffer()) });
  }
  return { records, photos };
}
