// Platform abstraction: the same UI runs on Tauri (desktop / mobile),
// in the browser (WASM + IndexedDB) and as the read-only family website.

export type Platform = 'tauri' | 'web' | 'viewer' | 'wp';

export interface MediaInput {
  /** Original file (already downscaled when very large). */
  original: Uint8Array;
  /** Small JPEG thumbnail (~480 px). */
  thumb: Uint8Array | null;
  fileName: string;
  mime: string;
  width: number;
  height: number;
  caption?: string;
  origin?: string;
}

export interface SiteExportResult {
  /** Where it went (desktop: path) or a download name (browser). */
  location: string;
}

/** One family tree in the list of trees (each has its own people and photos). */
export interface TreeInfo {
  id: string;
  name: string;
  created?: number;
}

export interface TreeList {
  current: string;
  trees: TreeInfo[];
}

/** Several separate family trees on one device / site. */
export interface TreesApi {
  list(): Promise<TreeList>;
  /** A new empty tree (not opened). Returns its id. */
  create(name: string): Promise<string>;
  rename(id: string, name: string): Promise<void>;
  /** Deletes a tree with its photos (not the open one). */
  remove(id: string): Promise<void>;
  /** Switches to another tree. The page reloads afterwards. */
  open(id: string): Promise<void>;
}

export interface Backend {
  platform: Platform;
  /** Read-only family website. */
  readonly: boolean;
  /** Calls an engine method (see crates/myroots-core/src/engine.rs). */
  call<T = any>(method: string, params?: unknown): Promise<T>;
  /** Stores media bytes and registers the metadata; returns the media id. */
  addMedia(m: MediaInput): Promise<number>;
  /** URL usable in <img src>. */
  mediaUrl(id: number, thumb?: boolean): string;
  /** Resolves when the URL from mediaUrl() is ready (browser: blob URLs). */
  ensureMedia?(ids: number[]): Promise<void>;
  /** URL for <img src> that may need work first (encrypted photos of the family website). */
  mediaUrlAsync?(id: number, thumb?: boolean): Promise<string>;
  /** Raw bytes of a media item. */
  mediaBytes(id: number, thumb?: boolean): Promise<Uint8Array | null>;
  /** Full backup (.myroots archive). Returns a save location / name. */
  exportArchive(onProgress?: (done: number, total: number) => void): Promise<string | null>;
  /** Restore from a .myroots archive (replaces everything); `onProgress(done, total)` for the progress bar. */
  importArchive(file: Blob, onProgress?: (done: number, total: number) => void): Promise<void>;
  /** Family website (ZIP). */
  exportSite(): Promise<string | null>;
  /** Saves a text / binary file chosen by the user. */
  saveFile(name: string, data: Uint8Array | string, mime: string): Promise<string | null>;
  /** The system print dialog (the desktop app cannot use window.print()). */
  print?(): Promise<void>;
  /** HTTP GET without CORS limits where possible (Tauri). */
  fetchBytes(url: string, headers?: Record<string, string>): Promise<{ status: number; bytes: Uint8Array; type: string }>;
  /** System notification (anniversaries). */
  notify?(title: string, body: string): Promise<void>;
  /** Several family trees (absent in the read-only website). */
  trees?: TreesApi;
  /** Persist pending writes (browser). */
  flush?(): Promise<void>;
}

export interface EngineError {
  code: string;
  message: string;
}

export function asEngineError(e: unknown): EngineError {
  if (e && typeof e === 'object' && 'code' in (e as any)) return e as EngineError;
  if (typeof e === 'string') {
    try {
      const v = JSON.parse(e);
      if (v && v.code) return v;
    } catch {
      /* not JSON */
    }
    return { code: 'error', message: e };
  }
  return { code: 'error', message: String((e as any)?.message ?? e) };
}
