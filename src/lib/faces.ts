// Finding the eyes on a photo, on the device (no network): a small face
// detector and a 68-point landmark model (~550 KB, loaded on first use).
// Only the editor uses it; the family website just plays the stored positions.

declare const __VIEWER__: boolean;

export type Eyes = [number, number, number, number];

let ready: Promise<any> | null = null;

async function api(): Promise<any> {
  if (__VIEWER__) return null;
  if (!ready) {
    ready = (async () => {
      const faceapi: any = await import('@vladmandic/face-api');
      const base = new URL('./models/face/', document.baseURI).href;
      // the graphics card when possible, otherwise the processor (no extra downloads)
      if (!(await faceapi.tf.setBackend('webgl').catch(() => false))) await faceapi.tf.setBackend('cpu');
      await faceapi.tf.ready();
      await faceapi.nets.tinyFaceDetector.loadFromUri(base);
      await faceapi.nets.faceLandmark68Net.loadFromUri(base);
      return faceapi;
    })().catch((e) => {
      ready = null;
      throw e;
    });
  }
  return ready;
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error('image'));
    img.src = src;
  });
}

/**
 * Centres of the eyes as fractions of the image size (left / right as seen on
 * the photo), of the largest face; null when no face is found.
 */
export async function detectEyes(src: string): Promise<Eyes | null> {
  const faceapi = await api();
  if (!faceapi) return null;
  const img = await loadImage(src);
  // detection on a copy limited to ~1000 px: faster, and small faces in big scans still work
  const scale = Math.min(1, 1000 / Math.max(img.naturalWidth, img.naturalHeight));
  const c = document.createElement('canvas');
  c.width = Math.round(img.naturalWidth * scale);
  c.height = Math.round(img.naturalHeight * scale);
  c.getContext('2d')!.drawImage(img, 0, 0, c.width, c.height);
  let found: any[] = [];
  for (const [inputSize, scoreThreshold] of [
    [512, 0.4],
    [416, 0.25],
    [608, 0.2],
  ] as const) {
    found = await faceapi.detectAllFaces(c, new faceapi.TinyFaceDetectorOptions({ inputSize, scoreThreshold })).withFaceLandmarks();
    if (found.length) break;
  }
  if (!found.length) return null;
  const best = found.sort((a: any, b: any) => b.detection.box.area - a.detection.box.area)[0];
  const pts: { x: number; y: number }[] = best.landmarks.positions;
  const centre = (from: number, to: number) => {
    const s = pts.slice(from, to + 1);
    return { x: s.reduce((a, p) => a + p.x, 0) / s.length, y: s.reduce((a, p) => a + p.y, 0) / s.length };
  };
  let a = centre(36, 41);
  let b = centre(42, 47);
  if (a.x > b.x) [a, b] = [b, a];
  return [a.x / c.width, a.y / c.height, b.x / c.width, b.y / c.height];
}

/**
 * Boxes around the faces on a photo (x, y, width, height as fractions of the
 * image, a little larger than the face), largest first. Group photos have
 * small faces, so the photo is also searched in overlapping tiles.
 */
export async function detectFaces(src: string): Promise<[number, number, number, number][]> {
  const faceapi = await api();
  if (!faceapi) return [];
  const img = await loadImage(src);
  const scale = Math.min(1, 1800 / Math.max(img.naturalWidth, img.naturalHeight));
  const W = Math.round(img.naturalWidth * scale);
  const H = Math.round(img.naturalHeight * scale);
  const full = document.createElement('canvas');
  full.width = W;
  full.height = H;
  full.getContext('2d')!.drawImage(img, 0, 0, W, H);

  type Hit = { x: number; y: number; w: number; h: number; score: number };
  const hits: Hit[] = [];
  const run = async (sx: number, sy: number, sw: number, sh: number, inputSize: number, threshold: number) => {
    const c = document.createElement('canvas');
    c.width = sw;
    c.height = sh;
    c.getContext('2d')!.drawImage(full, sx, sy, sw, sh, 0, 0, sw, sh);
    const found = await faceapi.detectAllFaces(c, new faceapi.TinyFaceDetectorOptions({ inputSize, scoreThreshold: threshold }));
    for (const d of found) hits.push({ x: sx + d.box.x, y: sy + d.box.y, w: d.box.width, h: d.box.height, score: d.score });
  };
  // the whole photo (large faces) …
  await run(0, 0, W, H, 608, 0.4);
  // … and square tiles with an overlap (small faces), at most about 12
  let side = Math.max(416, Math.round(Math.min(W, H) * 0.6));
  const count = (s: number) => (n: number) => (n <= s ? 1 : Math.ceil((n - s) / (s * 0.75)) + 1);
  while (count(side)(W) * count(side)(H) > 12) side = Math.round(side * 1.15);
  if (Math.max(W, H) > side * 1.2) {
    const steps = (n: number) => {
      const k = count(side)(n);
      return k === 1 ? [0] : Array.from({ length: k }, (_, i) => Math.round((i * (n - side)) / (k - 1)));
    };
    const input = Math.min(608, Math.round(side / 32) * 32);
    for (const y of steps(H)) for (const x of steps(W)) await run(x, y, Math.min(side, W - x), Math.min(side, H - y), input, 0.45);
  }
  // one box per face: the more certain of overlapping ones
  const iou = (a: Hit, b: Hit) => {
    const ix = Math.max(0, Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x));
    const iy = Math.max(0, Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y));
    const i = ix * iy;
    return i / Math.min(a.w * a.h, b.w * b.h);
  };
  const kept: Hit[] = [];
  for (const h of hits.sort((a, b) => b.score - a.score)) if (!kept.some((k) => iou(k, h) > 0.5)) kept.push(h);
  return kept
    .sort((a, b) => b.w * b.h - a.w * a.h)
    .slice(0, 80)
    .map((b): [number, number, number, number] => {
      const pad = 0.18;
      const x = Math.max(0, b.x - b.w * pad);
      const y = Math.max(0, b.y - b.h * pad * 1.3);
      const w = Math.min(W - x, b.w * (1 + 2 * pad));
      const h = Math.min(H - y, b.h * (1 + 2 * pad));
      return [x / W, y / H, w / W, h / H];
    });
}

const extra: Record<string, Promise<void> | undefined> = {};
/** Loads an additional model once ("ageGenderNet" ~0.4 MB, "faceRecognitionNet" ~6 MB). */
async function need(net: 'ageGenderNet' | 'faceRecognitionNet'): Promise<any> {
  const faceapi = await api();
  if (!faceapi) return null;
  if (!extra[net]) {
    extra[net] = faceapi.nets[net].loadFromUri(new URL('./models/face/', document.baseURI).href).catch((e: unknown) => {
      extra[net] = undefined;
      throw e;
    });
  }
  await extra[net];
  return faceapi;
}

/** A square crop around a box (fractions of the image), larger than the face, as a canvas. */
function cropFace(img: HTMLImageElement, r: [number, number, number, number], size = 256): HTMLCanvasElement {
  const W = img.naturalWidth;
  const H = img.naturalHeight;
  const cx = (r[0] + r[2] / 2) * W;
  const cy = (r[1] + r[3] / 2) * H;
  const side = Math.max(r[2] * W, r[3] * H) * 1.7;
  const c = document.createElement('canvas');
  c.width = size;
  c.height = size;
  const g = c.getContext('2d')!;
  g.fillStyle = '#808080';
  g.fillRect(0, 0, size, size);
  g.drawImage(img, cx - side / 2, cy - side / 2, side, side, 0, 0, size, size);
  return c;
}

/** Estimated ages of the faces in the boxes (null where no face is found). */
export async function estimateAges(src: string, rects: [number, number, number, number][]): Promise<(number | null)[]> {
  const faceapi = await need('ageGenderNet');
  if (!faceapi) return rects.map(() => null);
  const img = await loadImage(src);
  const out: (number | null)[] = [];
  for (const r of rects) {
    const c = cropFace(img, r);
    const d = await faceapi.detectSingleFace(c, new faceapi.TinyFaceDetectorOptions({ inputSize: 256, scoreThreshold: 0.2 })).withFaceLandmarks().withAgeAndGender();
    out.push(d ? Math.max(0, d.age) : null);
  }
  return out;
}

/** 128-number descriptors of the faces in the boxes, to tell people apart (null where no face is found). */
export async function describeFaces(src: string, rects: [number, number, number, number][]): Promise<(number[] | null)[]> {
  const faceapi = await need('faceRecognitionNet');
  if (!faceapi) return rects.map(() => null);
  const img = await loadImage(src);
  const out: (number[] | null)[] = [];
  for (const r of rects) {
    const c = cropFace(img, r);
    const d = await faceapi.detectSingleFace(c, new faceapi.TinyFaceDetectorOptions({ inputSize: 256, scoreThreshold: 0.2 })).withFaceLandmarks().withFaceDescriptor();
    out.push(d ? Array.from(d.descriptor as Float32Array, (v) => Math.round(v * 1e4) / 1e4) : null);
  }
  return out;
}

/** Preloads the models (e.g. when the editor opens). */
export function warmUp() {
  api().catch(() => {});
}
