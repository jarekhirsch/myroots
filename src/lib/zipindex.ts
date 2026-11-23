// Reads a zip archive without loading it into memory: only the list of files
// (the central directory at the end) is read, and a file is taken out when it
// is needed. Photos stored without compression (usual for JPEG) are slices of
// the original file and cost no memory at all. Handles ZIP64 archives (> 4 GB).
import { inflateSync } from 'fflate';

export interface ZipEntry {
  name: string;
  method: number;
  csize: number;
  size: number;
  offset: number;
}

const u16 = (v: DataView, o: number) => v.getUint16(o, true);
const u32 = (v: DataView, o: number) => v.getUint32(o, true);
const u64 = (v: DataView, o: number) => u32(v, o) + u32(v, o + 4) * 2 ** 32;

async function read(file: Blob, from: number, len: number): Promise<DataView> {
  return new DataView(await file.slice(from, from + len).arrayBuffer());
}

/** The files of a zip archive, or null when it is not a (readable) zip. */
export async function zipEntries(file: Blob): Promise<ZipEntry[] | null> {
  const tailLen = Math.min(file.size, 65_557);
  const tail = await read(file, file.size - tailLen, tailLen);
  let eocd = -1;
  for (let i = tailLen - 22; i >= 0; i--) {
    if (u32(tail, i) === 0x06054b50) {
      eocd = i;
      break;
    }
  }
  if (eocd < 0) return null;
  let count = u16(tail, eocd + 10);
  let cdSize = u32(tail, eocd + 12);
  let cdOffset = u32(tail, eocd + 16);
  // ZIP64: the real values are in the zip64 end record
  if (cdOffset === 0xffffffff || count === 0xffff || cdSize === 0xffffffff) {
    const loc = eocd - 20;
    if (loc < 0 || u32(tail, loc) !== 0x07064b50) return null;
    const recAt = u64(tail, loc + 8);
    const rec = await read(file, recAt, 56);
    if (u32(rec, 0) !== 0x06064b50) return null;
    count = u64(rec, 32);
    cdSize = u64(rec, 40);
    cdOffset = u64(rec, 48);
  }
  const cd = await read(file, cdOffset, cdSize);
  const utf8 = new TextDecoder();
  const latin = new TextDecoder('latin1');
  const out: ZipEntry[] = [];
  let p = 0;
  for (let n = 0; n < count && p + 46 <= cd.byteLength; n++) {
    if (u32(cd, p) !== 0x02014b50) return null;
    const flags = u16(cd, p + 8);
    const method = u16(cd, p + 10);
    let csize = u32(cd, p + 20);
    let size = u32(cd, p + 24);
    const nameLen = u16(cd, p + 28);
    const extraLen = u16(cd, p + 30);
    const commentLen = u16(cd, p + 32);
    let offset = u32(cd, p + 42);
    const nameBytes = new Uint8Array(cd.buffer, cd.byteOffset + p + 46, nameLen);
    // names flagged as UTF-8, or valid UTF-8 anyway (most programs); otherwise an old code page
    const asUtf8 = utf8.decode(nameBytes);
    const name = flags & 0x800 || !asUtf8.includes('\ufffd') ? asUtf8 : latin.decode(nameBytes);
    // ZIP64 extra field: the 64-bit values, in this order, for the fields set to 0xffffffff
    let e = p + 46 + nameLen;
    const end = e + extraLen;
    while (e + 4 <= end) {
      const id = u16(cd, e);
      const len = u16(cd, e + 2);
      if (id === 0x0001) {
        let q = e + 4;
        if (size === 0xffffffff) (size = u64(cd, q)), (q += 8);
        if (csize === 0xffffffff) (csize = u64(cd, q)), (q += 8);
        if (offset === 0xffffffff) offset = u64(cd, q);
      }
      e += 4 + len;
    }
    if (!name.endsWith('/')) out.push({ name, method, csize, size, offset });
    p += 46 + nameLen + extraLen + commentLen;
  }
  return out;
}

/** One file of the archive (a slice of the original when not compressed). */
export async function zipBlob(file: Blob, e: ZipEntry, type = ''): Promise<Blob> {
  const head = await read(file, e.offset, 30);
  if (u32(head, 0) !== 0x04034b50) throw new Error('zip: bad local header');
  const start = e.offset + 30 + u16(head, 26) + u16(head, 28);
  const raw = file.slice(start, start + e.csize, type);
  if (e.method === 0) return raw;
  if (e.method !== 8) throw new Error('zip: unsupported compression');
  const data = inflateSync(new Uint8Array(await raw.arrayBuffer()), { out: new Uint8Array(e.size) });
  return new Blob([data as BlobPart], { type });
}

export async function zipBytes(file: Blob, e: ZipEntry): Promise<Uint8Array> {
  return new Uint8Array(await (await zipBlob(file, e)).arrayBuffer());
}
