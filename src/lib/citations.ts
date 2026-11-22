// Citations of sources for single facts (see `Person.citations` and
// `Person.citation_info` in crates/myroots-core/src/model.rs; events and
// unions keep their own `citations` lists).
import { t } from './i18n/index.svelte';
import type { Citation, Person, Settings } from './types';

/** Reliability of a citation (GEDCOM QUAY), best first. */
export const QUALITY: [number, () => string][] = [
  [3, () => t('Direct evidence (original record)')],
  [2, () => t('Secondary evidence')],
  [1, () => t('Questionable')],
  [0, () => t('Unreliable')],
];

export function qualityLabel(q: number | null | undefined): string {
  return q == null ? '' : QUALITY.find(([k]) => k === q)?.[1]() || '';
}

/** Sources next to every fact (default) or one list per person. */
export function perFact(s: Settings | null | undefined): boolean {
  return (s?.sources_mode || 'facts') !== 'general';
}

const key = (field: string, source: number) => `${field}#${source}`;

/**
 * Citations of a group of person fields (e.g. birth = birth_date + birth_place),
 * merged: a source cited for any of the fields appears once.
 */
export function personCites(p: Person, fields: string[]): Citation[] {
  const out: Citation[] = [];
  for (const f of fields) {
    for (const s of p.citations?.[f] || []) {
      const info = p.citation_info?.[key(f, s)];
      const have = out.find((c) => c.source === s);
      if (have) {
        if (info && !have.page && !have.note && have.quality == null) Object.assign(have, info);
        continue;
      }
      out.push({ source: s, page: info?.page || '', quality: info?.quality ?? null, note: info?.note || '' });
    }
  }
  return out;
}

/** Sets the same citations on every field of the group (returns new maps). */
export function withPersonCites(p: Person, fields: string[], list: Citation[]): Pick<Person, 'citations' | 'citation_info'> {
  const citations = { ...(p.citations || {}) };
  const info = { ...(p.citation_info || {}) };
  for (const f of fields) {
    for (const k of Object.keys(info)) if (k.startsWith(`${f}#`)) delete info[k];
    if (list.length) citations[f] = list.map((c) => c.source);
    else delete citations[f];
    for (const c of list) {
      if (c.page || c.note || c.quality != null) info[key(f, c.source)] = { page: c.page, quality: c.quality, note: c.note };
    }
  }
  return { citations, citation_info: info };
}

/** Facts of the edit form that take sources, with the person fields they cover. */
export const FACTS: Record<string, string[]> = {
  name: ['name'],
  birth: ['birth_date', 'birth_place'],
  death: ['death_date', 'death_place', 'death_cause'],
  burial: ['burial'],
  occupation: ['occupation'],
  education: ['education'],
  faith: ['faith'],
  residence: ['residence'],
  biography: ['biography'],
};
