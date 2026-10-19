// Image processing before storing: downscale very large photos and make a thumbnail.
import { backend } from './backend';
import { toast } from './ui.svelte';
import { t } from './i18n/index.svelte';
import type { MediaInput } from './backend/types';

const MAX_SIDE = 2400;
const THUMB_SIDE = 480;

function loadImage(blob: Blob): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const url = URL.createObjectURL(blob);
    const img = new Image();
    img.onload = () => {
      URL.revokeObjectURL(url);
      resolve(img);
    };
    img.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error('not an image'));
    };
    img.src = url;
  });
}

function toBlob(canvas: HTMLCanvasElement, type: string, q: number): Promise<Blob> {
  return new Promise((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('encode'))), type, q));
}

async function scaled(img: HTMLImageElement, max: number, type: string, q: number): Promise<Uint8Array> {
  const r = Math.min(1, max / Math.max(img.naturalWidth, img.naturalHeight));
  const w = Math.max(1, Math.round(img.naturalWidth * r));
  const h = Math.max(1, Math.round(img.naturalHeight * r));
  const c = document.createElement('canvas');
  c.width = w;
  c.height = h;
  const ctx = c.getContext('2d')!;
  if (type === 'image/jpeg') {
    ctx.fillStyle = '#fff';
    ctx.fillRect(0, 0, w, h);
  }
  ctx.imageSmoothingQuality = 'high';
  ctx.drawImage(img, 0, 0, w, h);
  return new Uint8Array(await (await toBlob(c, type, q)).arrayBuffer());
}

export async function prepareImage(blob: Blob, fileName: string, extra: Partial<MediaInput> = {}): Promise<MediaInput> {
  const img = await loadImage(blob);
  // SVG can carry scripts: it is always turned into a PNG picture
  const svg = /svg/i.test(blob.type) || /\.svg$/i.test(fileName);
  const big = svg || Math.max(img.naturalWidth, img.naturalHeight) > MAX_SIDE || blob.size > 6_000_000;
  const keepType = blob.type === 'image/png' || blob.type === 'image/webp' || svg ? 'image/png' : 'image/jpeg';
  const original = big ? await scaled(img, MAX_SIDE, keepType === 'image/png' ? 'image/png' : 'image/jpeg', 0.88) : new Uint8Array(await blob.arrayBuffer());
  const mime = big ? (keepType === 'image/png' ? 'image/png' : 'image/jpeg') : blob.type || 'image/jpeg';
  const thumb = await scaled(img, THUMB_SIDE, 'image/jpeg', 0.82);
  const r = big ? Math.min(1, MAX_SIDE / Math.max(img.naturalWidth, img.naturalHeight)) : 1;
  return {
    original,
    thumb,
    fileName,
    mime,
    width: Math.round(img.naturalWidth * r),
    height: Math.round(img.naturalHeight * r),
    ...extra,
  };
}

/** Opens a file picker for images; resolves with the new media ids. */
export function pickImages(multiple = true, onFile?: (file: File, id: number) => void): Promise<number[]> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/*';
    input.multiple = multiple;
    input.onchange = async () => {
      const ids: number[] = [];
      const failed: string[] = [];
      for (const f of Array.from(input.files || [])) {
        try {
          const id = await backend.addMedia(await prepareImage(f, f.name));
          ids.push(id);
          onFile?.(f, id);
        } catch (e) {
          console.warn('skipped', f.name, e);
          failed.push(f.name);
        }
      }
      if (failed.length) {
        // HEIC (iPhone) is not shown by every browser
        const heic = failed.some((n) => /\.(heic|heif)$/i.test(n));
        toast(t('These photos could not be added: %s', failed.join(', ')) + (heic ? ' ' + t('Save HEIC photos as JPEG or use the desktop app.') : ''), 'error');
      }
      resolve(ids);
    };
    input.oncancel = () => resolve([]);
    input.click();
  });
}

/** Opens a file picker; resolves with the chosen file (or null). */
export function pickFile(accept: string): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = accept;
    input.onchange = () => resolve(input.files?.[0] || null);
    input.oncancel = () => resolve(null);
    input.click();
  });
}

/** Image URL (async on the browser backend). */
export async function mediaSrc(id: number | null | undefined, thumb = true): Promise<string> {
  if (!id) return '';
  if (backend.platform === 'web') {
    if (thumb) {
      await backend.ensureMedia?.([id]);
      return backend.mediaUrl(id, true);
    }
    return (backend as any).originalUrl(id);
  }
  return backend.mediaUrl(id, thumb);
}
