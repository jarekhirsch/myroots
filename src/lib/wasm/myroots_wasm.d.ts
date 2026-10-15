/* tslint:disable */
/* eslint-disable */

export class WasmEngine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Every stored record (full dump) as JSON.
     */
    all_records(): string;
    /**
     * Calls an engine method. Returns the JSON result; throws a JSON error `{code, message}`.
     */
    call(method: string, params_json: string, now: number, today: string): string;
    /**
     * GEDCOM bytes (any encoding) → UTF-8 text.
     */
    static decode_gedcom(bytes: Uint8Array): string;
    /**
     * Builds a `.myroots` archive. `media(id, thumb)` must return the bytes synchronously.
     */
    export_archive(media: Function, now: number): Uint8Array;
    /**
     * Builds the family website ZIP. `viewer_paths` / `viewer_files` are the
     * prebuilt read-only app files.
     */
    export_site(viewer_paths: string[], viewer_files: Uint8Array[], media: Function, now: number, today: string): Uint8Array;
    /**
     * Replaces the database with the archive content. Media are passed to
     * `sink(id, thumb, bytes)`. Returns the new records as JSON (to persist).
     */
    import_archive(bytes: Uint8Array, sink: Function): string;
    /**
     * `records_json`: JSON array of stored records ("" or "[]" = new, empty tree).
     */
    constructor(records_json: string, app_version: string, viewer: boolean, now: number);
    /**
     * Media ids needed by the family website (to preload them).
     */
    site_media(now: number, today: string): Float64Array;
    /**
     * Pending changes as JSON `{records: [...], deleted_media: [...]}`.
     */
    take_changes(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmengine_free: (a: number, b: number) => void;
    readonly wasmengine_all_records: (a: number, b: number) => void;
    readonly wasmengine_call: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly wasmengine_decode_gedcom: (a: number, b: number, c: number) => void;
    readonly wasmengine_export_archive: (a: number, b: number, c: number, d: number) => void;
    readonly wasmengine_export_site: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly wasmengine_import_archive: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmengine_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wasmengine_site_media: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmengine_take_changes: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
