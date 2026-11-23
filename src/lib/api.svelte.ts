import { backend } from './backend';
import { asEngineError } from './backend/types';
import type { AppInfo, Settings, Summary } from './types';

/** Shared reactive state. `version` changes after every write → views reload. */
export const app = $state({
  version: 0,
  info: null as AppInfo | null,
  settings: null as Settings | null,
  busy: 0,
});

// Methods that only read: every other call bumps app.version (views reload).
// A new read-only engine method must be added here, or opening a view that
// uses it would reload all views in a loop.
const READ = /^(app\.info|overview|settings\.get|people\.summaries|person\.(profile|edit|history)|pets\.list|pet\.get|sources\.(list|brief)|source\.get|media\.(list|unused|get|faces)|graves\.(list|cemeteries)|clans\.list|places\.(list|missing|weather)|tree\.data|lifeline|map\.data|stats|anniversaries|events\.list|gallery|kinship|health\.(issues|completeness|count)|merge\.(suggestions|compare)|book\.build|suggestions\.list|suggestion\.encode|gedcom\.export|wp\.pending)$/;

export async function call<T = any>(method: string, params: unknown = {}): Promise<T> {
  try {
    const r = await backend.call<T>(method, params);
    const isRead = READ.test(method) && !(method === 'gedcom.import' && (params as any)?.options?.dry_run);
    if (!isRead && !(method === 'gedcom.import' && (params as any)?.options?.dry_run)) {
      app.version++;
      if (method === 'settings.save') app.settings = r as any;
    }
    return r;
  } catch (e) {
    throw asEngineError(e);
  }
}

export async function refreshInfo() {
  app.info = await call<AppInfo>('app.info');
  app.settings = await call<Settings>('settings.get');
}

// Summaries of all people, cached per data version. Kept outside the reactive
// state on purpose: a deep proxy around tens of thousands of records made
// lists and searches slow in big trees.
let cached: { version: number; list: Summary[] } | null = null;
let pending: { version: number; promise: Promise<Summary[]> } | null = null;

/** Summaries of all people (cached per data version; parallel callers share one request). */
export async function summaries(): Promise<Summary[]> {
  const v = app.version;
  if (cached && cached.version === v) return cached.list;
  if (pending && pending.version === v) return pending.promise;
  const promise = call<Summary[]>('people.summaries').then(
    (list) => {
      if (app.version === v) cached = { version: v, list };
      if (pending?.promise === promise) pending = null;
      return list;
    },
    (e) => {
      if (pending?.promise === promise) pending = null;
      throw e;
    },
  );
  pending = { version: v, promise };
  return promise;
}

/** Summaries already loaded for the current data version (no request). */
export function cachedSummaries(): Summary[] | null {
  return cached && cached.version === app.version ? cached.list : null;
}

export function bump() {
  app.version++;
}

// Read-only results shared by several views (e.g. the family timeline on the
// start page and in the people list), cached per data version. Only for
// callers that do not change the result.
const shared = new Map<string, { version: number; promise: Promise<any> }>();

/** Like `call`, but one request per data version for the same method and parameters. */
export function callShared<T = any>(method: string, params: unknown = {}): Promise<T> {
  const key = method + JSON.stringify(params ?? {});
  const v = app.version;
  const hit = shared.get(key);
  if (hit && hit.version === v) return hit.promise;
  const promise = call<T>(method, params);
  shared.set(key, { version: v, promise });
  promise.catch(() => shared.get(key)?.promise === promise && shared.delete(key));
  return promise;
}
