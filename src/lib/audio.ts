// Sound recordings: storing files and recordings from the microphone, and
// playable URLs on every platform (the bytes are turned into a blob URL with
// the right type, which also works in the desktop app's web view).
import { backend } from './backend';
import type { Media } from './types';
import { call } from './api.svelte';
import { withProgress } from './ui.svelte';
import { t } from './i18n/index.svelte';

/** Larger files are refused (a long interview in MP3 is ~1 MB a minute). */
export const MAX_AUDIO_BYTES = 80 * 1024 * 1024;

const urls = new Map<number, string>();
let mimes: Map<number, string> | null = null;

async function mimeOf(id: number): Promise<string> {
  if (!mimes) {
    mimes = new Map();
    try {
      for (const m of await call<Media[]>('media.list')) mimes.set(m.id, m.mime);
    } catch {
      /* the family website: the browser guesses from the file */
    }
  }
  return mimes.get(id) || 'audio/mpeg';
}

/** A URL for <audio src>. */
export async function audioUrl(id: number): Promise<string> {
  if (backend.platform === 'viewer') return backend.mediaUrlAsync ? backend.mediaUrlAsync(id, false) : backend.mediaUrl(id, false);
  const cached = urls.get(id);
  if (cached) return cached;
  const bytes = await backend.mediaBytes(id, false);
  if (!bytes) return '';
  const url = URL.createObjectURL(new Blob([bytes as BlobPart], { type: await mimeOf(id) }));
  urls.set(id, url);
  return url;
}

function guessMime(f: File): string {
  if (f.type) return f.type;
  const ext = f.name.split('.').pop()?.toLowerCase() || '';
  return { mp3: 'audio/mpeg', m4a: 'audio/mp4', aac: 'audio/aac', wav: 'audio/wav', ogg: 'audio/ogg', oga: 'audio/ogg', opus: 'audio/ogg', webm: 'audio/webm', flac: 'audio/flac' }[ext] || 'audio/mpeg';
}

/** Stores an audio file (or a recording) and returns the media id. */
export async function storeAudio(blob: Blob, fileName: string, mime?: string): Promise<number> {
  if (blob.size > MAX_AUDIO_BYTES) throw Object.assign(new Error('too large'), { code: 'audio_too_large' });
  const type = (mime || blob.type || 'audio/mpeg').split(';')[0];
  const id = await withProgress(t('Adding the recording…'), async () => backend.addMedia({ original: new Uint8Array(await blob.arrayBuffer()), thumb: null, fileName, mime: type, width: 0, height: 0 }));
  mimes?.set(id, type);
  return id;
}

/** Opens a file picker for sound files; resolves with the new media ids and their names. */
export function pickAudio(): Promise<{ id: number; name: string }[]> {
  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'audio/*,.mp3,.m4a,.wav,.ogg,.opus,.webm,.flac,.aac';
    input.multiple = true;
    input.onchange = async () => {
      const out: { id: number; name: string }[] = [];
      try {
        for (const f of Array.from(input.files || [])) out.push({ id: await storeAudio(f, f.name, guessMime(f)), name: f.name.replace(/\.[^.]+$/, '') });
        resolve(out);
      } catch (e) {
        reject(e);
      }
    };
    input.oncancel = () => resolve([]);
    input.click();
  });
}

/** The best container this browser can record into. */
export function recorderMime(): string {
  if (typeof MediaRecorder === 'undefined') return '';
  for (const m of ['audio/webm;codecs=opus', 'audio/mp4', 'audio/ogg;codecs=opus', 'audio/webm']) if (MediaRecorder.isTypeSupported(m)) return m;
  return '';
}

export function canRecord(): boolean {
  return !!navigator.mediaDevices?.getUserMedia && !!recorderMime();
}

export function fmtTime(s: number): string {
  if (!Number.isFinite(s) || s < 0) s = 0;
  const m = Math.floor(s / 60);
  const sec = Math.floor(s % 60);
  return `${m}:${String(sec).padStart(2, '0')}`;
}
