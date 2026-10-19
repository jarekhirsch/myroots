import { backend } from './backend';
import { asEngineError } from './backend/types';
import type { AppInfo, Settings, Summary } from './types';

/** Shared reactive state. `version` changes after every write → views reload. */
export const app = $state({
  version: 0,
  info: null as AppInfo | null,
  settings: null as Settings | null,
  summaries: [] as Summary[],
  summariesVersion: -1,
  busy: 0,
});

const READ = /^(app\.info|overview|settings\.get|people\.summaries|person\.(profile|edit|history)|pets\.list|pet\.get|sources\.list|source\.get|media\.(list|unused|get)|clans\.list|places\.(list|missing|weather)|tree\.data|lifeline|map\.data|stats|anniversaries|events\.list|gallery|kinship|health\.(issues|completeness)|merge\.(suggestions|compare)|book\.build|suggestions\.list|suggestion\.encode|gedcom\.export|wp\.pending)$/;

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

/** Summaries of all people (cached per data version). */
export async function summaries(): Promise<Summary[]> {
  if (app.summariesVersion !== app.version) {
    app.summaries = await call<Summary[]>('people.summaries');
    app.summariesVersion = app.version;
  }
  return app.summaries;
}

export function bump() {
  app.version++;
}
