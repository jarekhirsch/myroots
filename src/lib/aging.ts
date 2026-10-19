// "Through the years": photos of one person aligned on the eyes and played
// one after another with cross-fades, the year and the age in the corner.
import { backend } from './backend';
import type { AgingFrame } from './types';
import { year as yearOf } from './format';

import { AUTHOR } from './credits';
const CREDIT = `© MyRoots · ${AUTHOR}`;

export const W = 720;
export const H = 900;
/** Where the eyes land on the canvas (fractions). */
const EYE_Y = 0.42;
const EYE_DIST = 0.21;

export interface Loaded {
  frame: AgingFrame;
  img: HTMLImageElement;
}

const cache = new Map<number, Promise<HTMLImageElement>>();
const blobUrls = new Map<number, string>();

/**
 * The original photo, same-origin (blob URL) so the canvas can be recorded to a
 * video; the family website uses its own files.
 */
export function loadPhoto(id: number): Promise<HTMLImageElement> {
  let p = cache.get(id);
  if (!p) {
    p = (async () => {
      let src = '';
      if (backend.platform === 'viewer') src = backend.mediaUrl(id, false);
      else {
        const bytes = (await backend.mediaBytes(id, false)) || (await backend.mediaBytes(id, true));
        if (!bytes) throw new Error('missing photo');
        src = URL.createObjectURL(new Blob([bytes as BlobPart]));
        blobUrls.set(id, src);
      }
      const img = new Image();
      img.decoding = 'async';
      img.src = src;
      await img.decode();
      return img;
    })();
    p.catch(() => cache.delete(id));
    cache.set(id, p);
  }
  return p;
}

/** A blob/file URL of the original (for the eye editor). */
export async function photoUrl(id: number): Promise<string> {
  await loadPhoto(id);
  return backend.platform === 'viewer' ? backend.mediaUrl(id, false) : blobUrls.get(id) || '';
}

interface Xf {
  s: number;
  ang: number;
  mx: number;
  my: number;
}

/** Scale / rotation / eye midpoint that put the eyes on the fixed line. */
export function transformOf(eyes: AgingFrame['eyes'], iw: number, ih: number, cw = W, ch = H): Xf {
  if (!eyes) {
    // no eyes: the photo covers the canvas, centred a bit high
    const s = Math.max(cw / iw, ch / ih);
    return { s, ang: 0, mx: iw / 2, my: ih * 0.42 + (EYE_Y * ch - ch / 2) / s };
  }
  const [lx, ly, rx, ry] = [eyes[0] * iw, eyes[1] * ih, eyes[2] * iw, eyes[3] * ih];
  const d = Math.hypot(rx - lx, ry - ly) || 1;
  return { s: (EYE_DIST * cw) / d, ang: Math.atan2(ry - ly, rx - lx), mx: (lx + rx) / 2, my: (ly + ry) / 2 };
}

const canFilter = (() => {
  try {
    const c = document.createElement('canvas').getContext('2d') as any;
    return c && 'filter' in c;
  } catch {
    return false;
  }
})();

export interface DrawOpts {
  gray: boolean;
  /** 0..1 gentle zoom during the life of a frame */
  zoom: number;
  alpha: number;
}

/** One photo, aligned, with a blurred copy of itself filling the rest of the canvas. */
export function drawPhoto(ctx: CanvasRenderingContext2D, l: Loaded, o: DrawOpts, cw = W, ch = H) {
  const { img, frame } = l;
  const xf = transformOf(frame.eyes, img.naturalWidth, img.naturalHeight, cw, ch);
  const z = 1 + 0.045 * o.zoom;
  ctx.save();
  ctx.globalAlpha = o.alpha;
  // background: the same photo, covering, blurred and darkened
  if (canFilter) {
    ctx.filter = `${o.gray ? 'grayscale(1) ' : ''}blur(28px) brightness(0.55)`;
    const s = Math.max(cw / img.naturalWidth, ch / img.naturalHeight) * 1.15;
    ctx.drawImage(img, (cw - img.naturalWidth * s) / 2, (ch - img.naturalHeight * s) / 2, img.naturalWidth * s, img.naturalHeight * s);
  }
  ctx.filter = o.gray && canFilter ? 'grayscale(1)' : 'none';
  ctx.translate(cw / 2, EYE_Y * ch);
  ctx.rotate(-xf.ang);
  ctx.scale(xf.s * z, xf.s * z);
  ctx.translate(-xf.mx, -xf.my);
  ctx.drawImage(img, 0, 0);
  ctx.restore();
}

export interface Timing {
  hold: number;
  fade: number;
}

export function totalTime(n: number, t: Timing): number {
  return n ? n * t.hold + Math.max(0, n - 1) * t.fade : 0;
}

/** Which frames are visible at time t: index of the current one and the mix towards the next (0..1). */
export function at(n: number, tm: Timing, t: number): { i: number; mix: number; life: number } {
  const step = tm.hold + tm.fade;
  if (n <= 1) return { i: 0, mix: 0, life: Math.min(1, t / Math.max(0.001, tm.hold)) };
  const i = Math.min(n - 1, Math.floor(t / step));
  const into = t - i * step;
  const mix = i >= n - 1 ? 0 : Math.max(0, (into - tm.hold) / tm.fade);
  return { i, mix: ease(Math.min(1, mix)), life: Math.min(1, into / step) };
}

function ease(x: number) {
  return x < 0.5 ? 2 * x * x : 1 - Math.pow(-2 * x + 2, 2) / 2;
}

export function frameYear(f: AgingFrame): number | null {
  return yearOf(f.date);
}

export interface Captions {
  name: string;
  year: (f: AgingFrame) => string;
  age: (f: AgingFrame) => string;
}

/** Year and age at the bottom, the name at the top. */
export function drawCaptions(ctx: CanvasRenderingContext2D, a: AgingFrame, b: AgingFrame | null, mix: number, c: Captions, cw = W, ch = H) {
  const g = ctx.createLinearGradient(0, ch * 0.72, 0, ch);
  g.addColorStop(0, 'rgba(0,0,0,0)');
  g.addColorStop(1, 'rgba(0,0,0,0.6)');
  ctx.fillStyle = g;
  ctx.fillRect(0, ch * 0.7, cw, ch * 0.3);
  const top = ctx.createLinearGradient(0, 0, 0, ch * 0.14);
  top.addColorStop(0, 'rgba(0,0,0,0.45)');
  top.addColorStop(1, 'rgba(0,0,0,0)');
  ctx.fillStyle = top;
  ctx.fillRect(0, 0, cw, ch * 0.14);
  const pad = cw * 0.06;
  ctx.fillStyle = '#fff';
  ctx.textBaseline = 'alphabetic';
  ctx.font = `600 ${Math.round(cw * 0.034)}px Onest, system-ui, sans-serif`;
  ctx.globalAlpha = 0.92;
  ctx.fillText(c.name, pad, pad + cw * 0.03);
  const one = (f: AgingFrame, alpha: number) => {
    if (alpha <= 0.001) return;
    ctx.globalAlpha = alpha;
    ctx.font = `500 ${Math.round(cw * 0.12)}px Literata, Georgia, serif`;
    const y = c.year(f);
    ctx.fillText(y, pad, ch - pad - cw * 0.06);
    ctx.font = `500 ${Math.round(cw * 0.04)}px Onest, system-ui, sans-serif`;
    ctx.fillText(c.age(f), pad + 2, ch - pad);
  };
  one(a, 1 - mix);
  if (b) one(b, mix);
  // a quiet credit in the corner (also in the saved video)
  ctx.globalAlpha = 0.5;
  ctx.font = `400 ${Math.round(cw * 0.018)}px Onest, system-ui, sans-serif`;
  ctx.textAlign = 'right';
  ctx.fillText(CREDIT, cw - pad * 0.6, ch - pad * 0.45);
  ctx.textAlign = 'left';
  ctx.globalAlpha = 1;
}
