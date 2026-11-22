// Toasts and confirmation dialogs.
export const ui = $state({
  toasts: [] as { id: number; text: string; kind: 'ok' | 'error' | 'info' }[],
  confirm: null as null | { title: string; body: string; ok: string; danger: boolean; resolve: (v: boolean) => void },
  prompt: null as null | { title: string; label: string; value: string; ok: string; resolve: (v: string | null) => void },
  lightbox: null as null | { items: { media: number; caption: string }[]; index: number },
  menuOpen: false,
});

let n = 0;
export function toast(text: string, kind: 'ok' | 'error' | 'info' = 'ok') {
  const id = ++n;
  ui.toasts.push({ id, text, kind });
  setTimeout(() => {
    const i = ui.toasts.findIndex((x) => x.id === id);
    if (i >= 0) ui.toasts.splice(i, 1);
  }, kind === 'error' ? 7000 : 3500);
}

export function confirmDialog(title: string, body = '', ok = '', danger = false): Promise<boolean> {
  return new Promise((resolve) => {
    ui.confirm = { title, body, ok, danger, resolve };
  });
}

export function promptDialog(title: string, label = '', value = '', ok = ''): Promise<string | null> {
  return new Promise((resolve) => {
    ui.prompt = { title, label, value, ok, resolve };
  });
}

export function openLightbox(items: { media: number; caption: string }[], index = 0) {
  ui.lightbox = { items, index };
}

// Progress of long file work (adding photos, restoring a backup, importing
// GEDCOM, sending files to WordPress / MyRoots Server). Shown as a bar at the
// top of the window by ProgressBar.svelte. total = 0: unknown length.
export const progress = $state({ active: 0, label: '', done: 0, total: 0 });

/** Runs `fn` with the progress bar; `report(done, total)` moves it. */
export async function withProgress<T>(label: string, fn: (report: (done: number, total: number) => void) => Promise<T>): Promise<T> {
  progress.active++;
  progress.label = label;
  progress.done = 0;
  progress.total = 0;
  try {
    return await fn((done, total) => {
      progress.done = done;
      progress.total = total;
    });
  } finally {
    progress.active = Math.max(0, progress.active - 1);
    if (!progress.active) {
      progress.label = '';
      progress.done = progress.total = 0;
    }
  }
}
