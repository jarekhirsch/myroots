import type { Backend } from './types';
import { isViewerMode } from './assets';

declare const __VIEWER__: boolean;

export let backend: Backend;

export function isTauri(): boolean {
  return typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;
}

/** The editor inside WordPress (the MyRoots plugin prints its configuration). */
export function isWordPress(): boolean {
  return typeof window !== 'undefined' && !!(window as any).MYROOTS_WP;
}

export async function initBackend(): Promise<Backend> {
  if (isViewerMode()) {
    const { ViewerBackend } = await import('./viewer');
    backend = await ViewerBackend.create();
  } else if (isWordPress()) {
    const { WpBackend } = await import('./wp');
    backend = await WpBackend.create();
  } else if (isTauri()) {
    const { TauriBackend } = await import('./tauri');
    backend = await TauriBackend.create();
  } else {
    const { WasmBackend } = await import('./wasm');
    backend = await WasmBackend.create();
  }
  return backend;
}
