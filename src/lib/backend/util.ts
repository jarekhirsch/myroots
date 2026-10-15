export function todayLocal(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

export function downloadBlob(name: string, data: Uint8Array | string, mime: string) {
  const blob = new Blob([typeof data === 'string' ? data : (data as BlobPart)], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 30_000);
}

export function stamp(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}-${p(d.getHours())}${p(d.getMinutes())}`;
}

export function slug(s: string): string {
  return (
    s
      .normalize('NFKD')
      .replace(/[̀-ͯ]/g, '')
      .replace(/ł/g, 'l')
      .replace(/Ł/g, 'L')
      .replace(/[^A-Za-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
      .toLowerCase() || 'myroots'
  );
}

export function extForMime(mime: string): string {
  return (
    {
      'image/png': 'png',
      'image/webp': 'webp',
      'image/gif': 'gif',
      'image/avif': 'avif',
      'image/svg+xml': 'svg',
      'application/pdf': 'pdf',
      'audio/mpeg': 'mp3',
      'audio/mp3': 'mp3',
      'audio/mp4': 'm4a',
      'audio/x-m4a': 'm4a',
      'audio/m4a': 'm4a',
      'audio/aac': 'm4a',
      'audio/webm': 'webm',
      'audio/ogg': 'ogg',
      'audio/opus': 'ogg',
      'audio/wav': 'wav',
      'audio/x-wav': 'wav',
      'audio/wave': 'wav',
      'audio/flac': 'flac',
      'audio/x-flac': 'flac',
    } as Record<string, string>
  )[mime] || 'jpg';
}
