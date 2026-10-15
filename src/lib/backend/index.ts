import type { Backend } from './types';

declare const __VIEWER__: boolean;

export let backend: Backend;

export function isTauri(): boolean {
  return typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;
}

export async function initBackend(): Promise<Backend> {
  if (__VIEWER__) {
    const { ViewerBackend } = await import('./viewer');
    backend = await ViewerBackend.create();
  } else if (isTauri()) {
    const { TauriBackend } = await import('./tauri');
    backend = await TauriBackend.create();
  } else {
    const { WasmBackend } = await import('./wasm');
    backend = await WasmBackend.create();
  }
  return backend;
}
