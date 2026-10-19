// GPS position from the EXIF data of a JPEG photo (taken with a phone at the grave).

/** [latitude, longitude] or null. */
export async function gpsFromFile(file: Blob): Promise<[number, number] | null> {
  try {
    const buf = new DataView(await file.slice(0, 256 * 1024).arrayBuffer());
    if (buf.getUint16(0) !== 0xffd8) return null;
    let off = 2;
    while (off + 4 < buf.byteLength) {
      const marker = buf.getUint16(off);
      const len = buf.getUint16(off + 2);
      if (marker === 0xffe1 && buf.getUint32(off + 4) === 0x45786966) return readTiff(buf, off + 10);
      if ((marker & 0xff00) !== 0xff00 || marker === 0xffda) break;
      off += 2 + len;
    }
  } catch {
    /* not readable */
  }
  return null;
}

function readTiff(v: DataView, t: number): [number, number] | null {
  const le = v.getUint16(t) === 0x4949;
  const u16 = (o: number) => v.getUint16(t + o, le);
  const u32 = (o: number) => v.getUint32(t + o, le);
  const tag = (ifd: number, id: number): number | null => {
    const n = u16(ifd);
    for (let i = 0; i < n; i++) {
      const e = ifd + 2 + i * 12;
      if (u16(e) === id) return e;
    }
    return null;
  };
  const ifd0 = u32(4);
  const gp = tag(ifd0, 0x8825);
  if (gp == null) return null;
  const gps = u32(gp + 8);
  const ref = (id: number) => {
    const e = tag(gps, id);
    return e == null ? '' : String.fromCharCode(v.getUint8(t + e + 8));
  };
  const dms = (id: number): number | null => {
    const e = tag(gps, id);
    if (e == null || u32(e + 4) < 3) return null;
    const at = u32(e + 8);
    const r = (k: number) => {
      const d = u32(at + k * 8 + 4);
      return d ? u32(at + k * 8) / d : 0;
    };
    return r(0) + r(1) / 60 + r(2) / 3600;
  };
  let lat = dms(2);
  let lng = dms(4);
  if (lat == null || lng == null || (lat === 0 && lng === 0)) return null;
  if (ref(1) === 'S') lat = -lat;
  if (ref(3) === 'W') lng = -lng;
  return Math.abs(lat) <= 90 && Math.abs(lng) <= 180 ? [lat, lng] : null;
}
