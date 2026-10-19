// Moves of the family between places: from a parent's place of birth to the
// child's (a new generation somewhere else), and from a person's place of
// birth to the place of death (a move during one life).
import { year } from './format';
import type { PlaceUse, TreeNode } from './types';

export type LatLng = [number, number];
export interface Move {
  /** gen: parent's birthplace → child's birthplace; life: birthplace → place of death */
  kind: 'gen' | 'life';
  from: LatLng;
  to: LatLng;
  fromName: string;
  toName: string;
  year: number;
  person: number;
  name: string;
}
export interface Birth {
  at: LatLng;
  year: number;
  person: number;
  name: string;
  place: string;
}

const name = (n: TreeNode) => [n.data['first name'], n.data['last name']].filter(Boolean).join(' ') || '?';

/** ~2 km: closer places count as the same place */
const far = (a: LatLng, b: LatLng) => Math.hypot(a[0] - b[0], (a[1] - b[1]) * Math.cos((a[0] * Math.PI) / 180)) > 0.02;

export function buildMigrations(nodes: TreeNode[], places: PlaceUse[], only: number | null): { moves: Move[]; births: Birth[] } {
  const at = new Map<string, LatLng>();
  for (const p of places) if (p.lat != null && p.lng != null) at.set(p.name.trim().toLowerCase(), [p.lat, p.lng]);
  const byId = new Map(nodes.map((n) => [n.id, n]));
  // only the ancestors of one person (and the person)
  let keep: Set<string> | null = null;
  if (only != null) {
    keep = new Set([String(only)]);
    const q = [String(only)];
    while (q.length) {
      for (const p of byId.get(q.pop()!)?.rels.parents || []) {
        if (!keep.has(p)) {
          keep.add(p);
          q.push(p);
        }
      }
    }
  }
  const pos = (place: string) => at.get((place || '').trim().toLowerCase());
  const moves: Move[] = [];
  const births: Birth[] = [];
  for (const n of nodes) {
    if (keep && !keep.has(n.id)) continue;
    const b = pos(n.data.birth_place);
    const by = year(n.data.birthday);
    if (b && by != null) births.push({ at: b, year: by, person: Number(n.id), name: name(n), place: n.data.birth_place });
    if (b && by != null) {
      for (const pid of n.rels.parents) {
        const p = byId.get(pid);
        if (!p || (keep && !keep.has(pid))) continue;
        const pb = pos(p.data.birth_place);
        if (pb && far(pb, b)) moves.push({ kind: 'gen', from: pb, to: b, fromName: p.data.birth_place, toName: n.data.birth_place, year: by, person: Number(n.id), name: name(n) });
      }
    }
    const d = pos(n.data.death_place);
    const dy = year(n.data.death_date);
    if (b && d && dy != null && far(b, d)) moves.push({ kind: 'life', from: b, to: d, fromName: n.data.birth_place, toName: n.data.death_place, year: dy, person: Number(n.id), name: name(n) });
  }
  // a child of two parents born in the same place: one line is enough
  const seen = new Set<string>();
  const uniq = moves.filter((m) => {
    const k = `${m.kind}|${m.person}|${m.from.join()}|${m.to.join()}`;
    if (seen.has(k)) return false;
    seen.add(k);
    return true;
  });
  uniq.sort((a, b) => a.year - b.year);
  births.sort((a, b) => a.year - b.year);
  return { moves: uniq, births };
}

/** Points of a gentle arc between two places (so that lines do not overlap). */
export function arc(a: LatLng, b: LatLng, steps = 24): LatLng[] {
  const mx = (a[0] + b[0]) / 2;
  const my = (a[1] + b[1]) / 2;
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const cx = mx - dy * 0.18;
  const cy = my + dx * 0.18;
  const out: LatLng[] = [];
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    out.push([(1 - t) * (1 - t) * a[0] + 2 * (1 - t) * t * cx + t * t * b[0], (1 - t) * (1 - t) * a[1] + 2 * (1 - t) * t * cy + t * t * b[1]]);
  }
  return out;
}
