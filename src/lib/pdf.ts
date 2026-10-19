// A one-page PDF of an SVG at an exact paper size. The picture is drawn in
// tiles (each rendered from the vector source at full resolution), so large
// formats are not limited by the size of one canvas.

const enc = new TextEncoder();

async function tileJpeg(svg: string, vb: [number, number, number, number], px: [number, number]): Promise<Uint8Array> {
  // the same drawing, cut to this tile by its viewBox
  const doc = new DOMParser().parseFromString(svg, 'image/svg+xml');
  const root = doc.documentElement;
  root.setAttribute('viewBox', vb.join(' '));
  root.setAttribute('width', String(px[0]));
  root.setAttribute('height', String(px[1]));
  root.setAttribute('preserveAspectRatio', 'none');
  const url = URL.createObjectURL(new Blob([new XMLSerializer().serializeToString(root)], { type: 'image/svg+xml' }));
  try {
    const img = new Image();
    await new Promise<void>((ok, bad) => {
      img.onload = () => ok();
      img.onerror = () => bad(new Error('svg'));
      img.src = url;
    });
    const c = document.createElement('canvas');
    c.width = px[0];
    c.height = px[1];
    const g = c.getContext('2d')!;
    g.fillStyle = '#fff';
    g.fillRect(0, 0, px[0], px[1]);
    g.drawImage(img, 0, 0, px[0], px[1]);
    const blob: Blob = await new Promise((ok, bad) => c.toBlob((b) => (b ? ok(b) : bad(new Error('jpeg'))), 'image/jpeg', 0.92));
    return new Uint8Array(await blob.arrayBuffer());
  } finally {
    URL.revokeObjectURL(url);
  }
}

/**
 * @param svg  standalone SVG (fonts embedded), viewBox in millimetres 0 0 W H
 * @param W,H  paper size in millimetres
 * @param dpi  resolution of the picture
 */
export async function svgToPdf(svg: string, W: number, H: number, dpi: number, onProgress?: (done: number, total: number) => void): Promise<Uint8Array> {
  const pxW = Math.round((W / 25.4) * dpi);
  const pxH = Math.round((H / 25.4) * dpi);
  const T = 3000;
  const cols = Math.ceil(pxW / T);
  const rows = Math.ceil(pxH / T);
  const tiles: { x: number; y: number; w: number; h: number; jpeg: Uint8Array }[] = [];
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      const x = c * T;
      const y = r * T;
      const w = Math.min(T, pxW - x);
      const h = Math.min(T, pxH - y);
      const mm = (v: number, total: number, size: number) => (v / total) * size;
      const jpeg = await tileJpeg(svg, [mm(x, pxW, W), mm(y, pxH, H), mm(w, pxW, W), mm(h, pxH, H)], [w, h]);
      tiles.push({ x, y, w, h, jpeg });
      onProgress?.(tiles.length, rows * cols);
    }
  }
  // points
  const pw = (W / 25.4) * 72;
  const ph = (H / 25.4) * 72;
  const k = pw / pxW;
  const parts: Uint8Array[] = [];
  const offsets: number[] = [];
  let size = 0;
  const push = (b: Uint8Array | string) => {
    const u = typeof b === 'string' ? enc.encode(b) : b;
    parts.push(u);
    size += u.length;
  };
  const obj = (n: number, body: () => void) => {
    offsets[n] = size;
    push(`${n} 0 obj\n`);
    body();
    push('\nendobj\n');
  };
  push('%PDF-1.4\n%\xE2\xE3\xCF\xD3\n');
  const first = 5;
  const xobjects = tiles.map((_, i) => `/Im${i} ${first + i} 0 R`).join(' ');
  const content = tiles
    .map((t, i) => `q ${(t.w * k).toFixed(3)} 0 0 ${(t.h * k).toFixed(3)} ${(t.x * k).toFixed(3)} ${(ph - (t.y + t.h) * k).toFixed(3)} cm /Im${i} Do Q`)
    .join('\n');
  obj(1, () => push('<< /Type /Catalog /Pages 2 0 R >>'));
  obj(2, () => push('<< /Type /Pages /Kids [3 0 R] /Count 1 >>'));
  obj(3, () => push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${pw.toFixed(2)} ${ph.toFixed(2)}] /Resources << /XObject << ${xobjects} >> >> /Contents 4 0 R >>`));
  obj(4, () => push(`<< /Length ${enc.encode(content).length} >>\nstream\n${content}\nendstream`));
  tiles.forEach((t, i) =>
    obj(first + i, () => {
      push(`<< /Type /XObject /Subtype /Image /Width ${t.w} /Height ${t.h} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length ${t.jpeg.length} >>\nstream\n`);
      push(t.jpeg);
      push('\nendstream');
    }),
  );
  const n = first + tiles.length;
  const xref = size;
  push(`xref\n0 ${n}\n0000000000 65535 f \n`);
  for (let i = 1; i < n; i++) push(`${String(offsets[i]).padStart(10, '0')} 00000 n \n`);
  push(`trailer\n<< /Size ${n} /Root 1 0 R /Info << /Producer (MyRoots) >> >>\nstartxref\n${xref}\n%%EOF\n`);
  const out = new Uint8Array(size);
  let o = 0;
  for (const p of parts) {
    out.set(p, o);
    o += p.length;
  }
  return out;
}
