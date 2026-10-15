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

/** Preloads the models (e.g. when the editor opens). */
export function warmUp() {
  api().catch(() => {});
}
