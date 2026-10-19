// Finds the photos on a scanned album page: everything that differs from the
// page's background colour is grouped into regions, and each rectangular
// region becomes a (possibly rotated) box. Runs on the device.

/** A box: centre (fractions of the page width and height), size (fractions of the page width) and rotation. */
export interface Box {
  cx: number;
  cy: number;
  w: number;
  h: number;
  /** radians */
  angle: number;
}

type Pt = [number, number];

function hull(pts: Pt[]): Pt[] {
  pts.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  const cross = (o: Pt, a: Pt, b: Pt) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
  const lower: Pt[] = [];
  for (const p of pts) {
    while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], p) <= 0) lower.pop();
    lower.push(p);
  }
  const upper: Pt[] = [];
  for (let i = pts.length - 1; i >= 0; i--) {
    const p = pts[i];
    while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], p) <= 0) upper.pop();
    upper.push(p);
  }
  return lower.slice(0, -1).concat(upper.slice(0, -1));
}

/** Smallest rectangle around the points (rotating calipers over the hull edges). */
function minRect(h: Pt[]): { cx: number; cy: number; w: number; h: number; angle: number; area: number } {
  let best = { cx: 0, cy: 0, w: 0, h: 0, angle: 0, area: Infinity };
  for (let i = 0; i < h.length; i++) {
    const a = h[i];
    const b = h[(i + 1) % h.length];
    const th = Math.atan2(b[1] - a[1], b[0] - a[0]);
    const c = Math.cos(-th);
    const s = Math.sin(-th);
    let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity;
    for (const p of h) {
      const x = p[0] * c - p[1] * s;
      const y = p[0] * s + p[1] * c;
      if (x < x0) x0 = x;
      if (x > x1) x1 = x;
      if (y < y0) y0 = y;
      if (y > y1) y1 = y;
    }
    const area = (x1 - x0) * (y1 - y0);
    if (area < best.area) {
      const mx = (x0 + x1) / 2;
      const my = (y0 + y1) / 2;
      // back to page coordinates
      const ci = Math.cos(th);
      const si = Math.sin(th);
      best = { cx: mx * ci - my * si, cy: mx * si + my * ci, w: x1 - x0, h: y1 - y0, angle: th, area };
    }
  }
  // keep the angle small: a box tilted by 80° is the same as one tilted by -10°
  let { angle, w, h: hh } = best;
  while (angle > Math.PI / 4) {
    angle -= Math.PI / 2;
    [w, hh] = [hh, w];
  }
  while (angle < -Math.PI / 4) {
    angle += Math.PI / 2;
    [w, hh] = [hh, w];
  }
  return { ...best, angle, w, h: hh };
}

export function findPhotos(img: HTMLImageElement | HTMLCanvasElement): Box[] {
  const sw = 'naturalWidth' in img ? img.naturalWidth : img.width;
  const sh = 'naturalHeight' in img ? img.naturalHeight : img.height;
  const k = Math.min(1, 700 / Math.max(sw, sh));
  const W = Math.max(1, Math.round(sw * k));
  const H = Math.max(1, Math.round(sh * k));
  const c = document.createElement('canvas');
  c.width = W;
  c.height = H;
  const g = c.getContext('2d', { willReadFrequently: true })!;
  g.drawImage(img, 0, 0, W, H);
  const px = g.getImageData(0, 0, W, H).data;

  // background: median colour of the outer frame of the page
  const m = Math.max(2, Math.round(Math.min(W, H) * 0.02));
  const rs: number[] = [], gs: number[] = [], bs: number[] = [];
  const sample = (x: number, y: number) => {
    const i = (y * W + x) * 4;
    rs.push(px[i]);
    gs.push(px[i + 1]);
    bs.push(px[i + 2]);
  };
  for (let x = 0; x < W; x += 2) for (let y = 0; y < m; y++) (sample(x, y), sample(x, H - 1 - y));
  for (let y = 0; y < H; y += 2) for (let x = 0; x < m; x++) (sample(x, y), sample(W - 1 - x, y));
  const med = (a: number[]) => a.sort((p, q) => p - q)[a.length >> 1];
  const bg = [med(rs), med(gs), med(bs)];
  // how much the background itself varies
  const devs = rs.map((r, i) => Math.abs(r - bg[0]) + Math.abs(gs[i] - bg[1]) + Math.abs(bs[i] - bg[2]));
  const thr = Math.max(36, med(devs) * 4 + 18);

  // foreground mask, smoothed: a pixel is "photo" when most of its 5×5 neighbourhood differs
  const diff = new Uint8Array(W * H);
  for (let i = 0, j = 0; i < W * H; i++, j += 4) {
    diff[i] = Math.abs(px[j] - bg[0]) + Math.abs(px[j + 1] - bg[1]) + Math.abs(px[j + 2] - bg[2]) > thr ? 1 : 0;
  }
  const integral = new Int32Array((W + 1) * (H + 1));
  for (let y = 0; y < H; y++) {
    let row = 0;
    for (let x = 0; x < W; x++) {
      row += diff[y * W + x];
      integral[(y + 1) * (W + 1) + x + 1] = integral[y * (W + 1) + x + 1] + row;
    }
  }
  const r = 3;
  const mask = new Uint8Array(W * H);
  for (let y = 0; y < H; y++) {
    const y0 = Math.max(0, y - r), y1 = Math.min(H, y + r + 1);
    for (let x = 0; x < W; x++) {
      const x0 = Math.max(0, x - r), x1 = Math.min(W, x + r + 1);
      const sum = integral[y1 * (W + 1) + x1] - integral[y0 * (W + 1) + x1] - integral[y1 * (W + 1) + x0] + integral[y0 * (W + 1) + x0];
      mask[y * W + x] = sum * 2 > (x1 - x0) * (y1 - y0) ? 1 : 0;
    }
  }

  // regions
  const seen = new Uint8Array(W * H);
  const boxes: Box[] = [];
  const stack: number[] = [];
  for (let s = 0; s < W * H; s++) {
    if (!mask[s] || seen[s]) continue;
    seen[s] = 1;
    stack.push(s);
    let count = 0;
    const edge: Pt[] = [];
    while (stack.length) {
      const p = stack.pop()!;
      count++;
      const x = p % W;
      const y = (p - x) / W;
      let border = false;
      for (const q of [x > 0 ? p - 1 : -1, x < W - 1 ? p + 1 : -1, y > 0 ? p - W : -1, y < H - 1 ? p + W : -1]) {
        if (q < 0) {
          border = true;
          continue;
        }
        if (!mask[q]) border = true;
        else if (!seen[q]) {
          seen[q] = 1;
          stack.push(q);
        }
      }
      if (border) edge.push([x + 0.5, y + 0.5]);
    }
    if (count < W * H * 0.012 || edge.length < 8) continue;
    const rect = minRect(hull(edge));
    const fill = count / Math.max(1, rect.area);
    const ratio = rect.w / Math.max(1, rect.h);
    if (fill < 0.72 || ratio < 0.18 || ratio > 5.5) continue;
    // the whole page is not a photo on the page
    if (rect.w * rect.h > W * H * 0.9) continue;
    boxes.push({ cx: rect.cx / W, cy: rect.cy / H, w: rect.w / W, h: rect.h / W, angle: rect.angle });
  }
  // reading order: top to bottom, left to right
  return boxes.sort((a, b) => (Math.abs(a.cy - b.cy) < 0.08 ? a.cx - b.cx : a.cy - b.cy));
}

/** Corners of a box in page pixels (for drawing). */
export function corners(b: Box, W: number, H: number): Pt[] {
  const c = Math.cos(b.angle);
  const s = Math.sin(b.angle);
  const hw = (b.w * W) / 2;
  const hh = (b.h * W) / 2;
  const cx = b.cx * W;
  const cy = b.cy * H;
  return [
    [-hw, -hh],
    [hw, -hh],
    [hw, hh],
    [-hw, hh],
  ].map(([x, y]) => [cx + x * c - y * s, cy + x * s + y * c] as Pt);
}

/** Cuts a box out of the full-size page, straightened, slightly inside its edge. */
export async function cutOut(img: HTMLImageElement, b: Box): Promise<Blob> {
  const W = img.naturalWidth;
  const H = img.naturalHeight;
  const inset = 0.012;
  const w = Math.max(1, Math.round(b.w * W * (1 - 2 * inset)));
  const h = Math.max(1, Math.round(b.h * W * (1 - 2 * inset)));
  const c = document.createElement('canvas');
  c.width = w;
  c.height = h;
  const g = c.getContext('2d')!;
  g.imageSmoothingQuality = 'high';
  g.translate(w / 2, h / 2);
  g.rotate(-b.angle);
  g.drawImage(img, -b.cx * W, -b.cy * H);
  return new Promise((resolve, reject) => c.toBlob((x) => (x ? resolve(x) : reject(new Error('canvas'))), 'image/jpeg', 0.92));
}
