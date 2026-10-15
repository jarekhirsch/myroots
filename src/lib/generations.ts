// Family on a time axis: generations from the relations, a life span for
// everyone who can be placed in time, and lanes so bars and names never overlap.
import { year } from './format';
import type { TreeNode } from './types';

export interface Bar {
  id: string;
  name: string;
  gender: string;
  photo: number | null;
  years: string;
  start: number;
  end: number;
  /** Birth year was estimated from relatives. */
  approxStart: boolean;
  /** No death date: alive, or the end is unknown. */
  open: 'living' | 'unknown' | null;
  marriages: { year: number; partner: string }[];
  lane: number;
}

export interface Generation {
  index: number;
  from: number | null;
  to: number | null;
  lanes: number;
  bars: Bar[];
}

export interface Timeline {
  generations: Generation[];
  undated: { id: string; name: string; gender: string; photo: number | null }[];
  hidden: number;
  min: number;
  max: number;
  related: Map<string, Set<string>>;
}

const GEN_GAP = 28; // typical age of a parent at a child's birth
const UNKNOWN_SPAN = 70;

function nameOf(n: TreeNode) {
  return [n.data['first name'], n.data['last name']].filter(Boolean).join(' ').trim();
}

/**
 * Generation levels inside each connected group. A child is one level below
 * its deepest parent; people without known parents take the level of their
 * partner (married into the family) or sit one level above their children.
 */
function levels(nodes: Map<string, TreeNode>): Map<string, { comp: number; level: number }> {
  const has = (id: string) => nodes.has(id);
  const parents = (id: string) => nodes.get(id)!.rels.parents.filter(has);
  const level = new Map<string, number>();
  const root = new Map<string, number>(); // chosen level of people without parents

  for (let pass = 0; pass < 12; pass++) {
    level.clear();
    const visiting = new Set<string>();
    const lv = (id: string): number => {
      const known = level.get(id);
      if (known != null) return known;
      const ps = parents(id);
      if (!ps.length || visiting.has(id)) return root.get(id) ?? 0;
      visiting.add(id);
      const v = Math.max(...ps.map(lv)) + 1;
      visiting.delete(id);
      level.set(id, v);
      return v;
    };
    for (const id of nodes.keys()) lv(id);
    let changed = false;
    for (const [id, n] of nodes) {
      if (parents(id).length) continue;
      const sp = n.rels.spouses.filter((s) => has(s) && parents(s).length).map((s) => level.get(s)!);
      const ch = n.rels.children.filter(has).map((c) => level.get(c)!);
      const want = sp.length ? Math.max(...sp) : ch.length ? Math.min(...ch) - 1 : 0;
      if ((root.get(id) ?? 0) !== want) {
        root.set(id, want);
        changed = true;
      }
    }
    if (!changed) break;
  }

  // connected groups
  const comp = new Map<string, number>();
  let c = 0;
  for (const start of nodes.keys()) {
    if (comp.has(start)) continue;
    const stack = [start];
    comp.set(start, c);
    while (stack.length) {
      const n = nodes.get(stack.pop()!)!;
      for (const o of [...n.rels.spouses, ...n.rels.parents, ...n.rels.children]) {
        if (has(o) && !comp.has(o)) {
          comp.set(o, c);
          stack.push(o);
        }
      }
    }
    c++;
  }
  const out = new Map<string, { comp: number; level: number }>();
  for (const id of nodes.keys()) out.set(id, { comp: comp.get(id)!, level: level.get(id) ?? root.get(id) ?? 0 });
  return out;
}

/**
 * Generation of everyone (0 = oldest), plus birth years (known or estimated
 * from relatives). Separate family groups are aligned in time with the
 * largest one; groups without any dates start at generation 0.
 */
export function generationsOf(nodes: Map<string, TreeNode>): { gen: Map<string, number>; birth: Map<string, number>; approx: Set<string> } {
  // Birth years: known, else estimated from spouses, parents or children.
  const birth = new Map<string, number>();
  const approx = new Set<string>();
  for (const [id, n] of nodes) {
    const y = year(n.data.birthday);
    if (y) birth.set(id, y);
  }
  for (let pass = 0; pass < 4; pass++) {
    for (const [id, n] of nodes) {
      if (birth.has(id)) continue;
      const known = (ids: string[], shift: number) => ids.map((i) => birth.get(i)).filter((v): v is number => v != null).map((v) => v + shift);
      const guesses = [...known(n.rels.spouses, 0), ...known(n.rels.parents, GEN_GAP), ...known(n.rels.children, -GEN_GAP)];
      if (guesses.length) {
        birth.set(id, Math.round(guesses.reduce((a, b) => a + b, 0) / guesses.length));
        approx.add(id);
      }
    }
  }

  // Generations: levels per connected group, then groups aligned in time
  // with the largest one through the median birth year of each level.
  const lv = levels(nodes);
  const comps = new Map<number, string[]>();
  for (const [id, v] of lv) comps.set(v.comp, [...(comps.get(v.comp) || []), id]);
  const median = (xs: number[]) => {
    const s = [...xs].sort((a, b) => a - b);
    return s.length ? s[Math.floor(s.length / 2)] : null;
  };
  const levelYears = (ids: string[]) => {
    const m = new Map<number, number[]>();
    for (const id of ids) {
      const b = birth.get(id);
      if (b == null) continue;
      const l = lv.get(id)!.level;
      m.set(l, [...(m.get(l) || []), b]);
    }
    return m;
  };
  const sorted = [...comps.values()].sort((a, b) => b.length - a.length);
  const main = sorted[0] || [];
  const mainLevels = levelYears(main);
  const mainMin = Math.min(0, ...main.map((id) => lv.get(id)!.level));
  // anchor: level → median birth year of the main group, extrapolated by GEN_GAP
  const anchors = [...mainLevels].map(([l, ys]) => [l, median(ys)!] as [number, number]);
  const yearToLevel = (y: number) => {
    if (!anchors.length) return 0;
    let best = anchors[0];
    for (const a of anchors) if (Math.abs(a[1] - y) < Math.abs(best[1] - y)) best = a;
    return best[0] + Math.round((y - best[1]) / GEN_GAP);
  };
  const gen = new Map<string, number>();
  for (const id of main) gen.set(id, lv.get(id)!.level - mainMin);
  for (const ids of sorted.slice(1)) {
    const ly = levelYears(ids);
    if (!ly.size) {
      // no dates at all: keep the group's own levels
      const m0 = Math.min(...ids.map((id) => lv.get(id)!.level));
      for (const id of ids) gen.set(id, lv.get(id)!.level - m0);
      continue;
    }
    const [l0, ys] = [...ly][0];
    const shift = yearToLevel(median(ys)!) - l0;
    for (const id of ids) gen.set(id, lv.get(id)!.level + shift - mainMin);
  }

  const low = gen.size ? Math.min(...gen.values()) : 0;
  for (const [id, g] of gen) gen.set(id, g - low);
  return { gen, birth, approx };
}

export function buildTimeline(all: TreeNode[], now = new Date()): Timeline {
  const nodes = new Map<string, TreeNode>();
  let hidden = 0;
  for (const n of all) {
    if (n.data.anon) hidden++;
    else nodes.set(n.id, n);
  }
  const nowY = now.getFullYear() + (now.getMonth() + now.getDate() / 31) / 12;

  const { birth, approx, gen } = generationsOf(nodes);
  const minGen = 0;

  const undated: Timeline['undated'] = [];
  const byGen = new Map<number, Bar[]>();
  let min = Infinity;
  let max = nowY;
  for (const [id, n] of nodes) {
    const b = birth.get(id);
    const g = gen.get(id);
    if (b == null || g == null) {
      undated.push({ id, name: nameOf(n), gender: n.data.gender, photo: n.data.avatar });
      continue;
    }
    const d = year(n.data.death_date);
    const open = d ? null : n.data.living ? 'living' : 'unknown';
    const end = d ?? (n.data.living ? nowY : Math.min(nowY, b + UNKNOWN_SPAN));
    min = Math.min(min, b);
    max = Math.max(max, end);
    const marriages = Object.entries(n.data.marriages || {})
      .map(([partner, date]) => ({ partner, year: year(date) || 0 }))
      .filter((m) => m.year && m.year >= b && m.year <= end);
    const bar: Bar = {
      id,
      name: nameOf(n),
      gender: n.data.gender,
      photo: n.data.avatar,
      years: n.data.years,
      start: b,
      end: Math.max(end, b + 0.6),
      approxStart: approx.has(id),
      open,
      marriages,
      lane: 0,
    };
    const k = g - minGen;
    byGen.set(k, [...(byGen.get(k) || []), bar]);
  }

  const generations: Generation[] = [...byGen.entries()]
    .sort((a, b) => a[0] - b[0])
    .map(([index, bars]) => {
      bars.sort((a, b) => a.start - b.start || a.name.localeCompare(b.name));
      const known = bars.filter((x) => !x.approxStart).map((x) => x.start);
      return { index, bars, lanes: 0, from: known.length ? Math.min(...known) : null, to: known.length ? Math.max(...known) : null };
    });

  // Partners, parents and children of everyone (for highlighting).
  const related = new Map<string, Set<string>>();
  for (const [id, n] of nodes) related.set(id, new Set([id, ...n.rels.spouses, ...n.rels.parents, ...n.rels.children]));

  return { generations, undated, hidden, min: Number.isFinite(min) ? min : nowY - 100, max, related };
}

/**
 * Puts the bars of one generation into lanes. `reach` says how many years a
 * bar occupies on its lane (its own span, or more when its label is longer).
 * With `maxLanes`, crowded bars overlap instead of adding lanes.
 */
export function packGeneration(g: Generation, reach: (b: Bar) => number, maxLanes = Infinity): Generation {
  const ends: number[] = [];
  const bars = g.bars.map((b) => {
    const until = b.start + reach(b);
    let lane = ends.findIndex((e) => e <= b.start);
    if (lane < 0 && ends.length >= maxLanes) {
      // no free lane left: overlap on the one that frees up first
      lane = ends.indexOf(Math.min(...ends));
      ends[lane] = Math.max(ends[lane], until);
    } else if (lane < 0) {
      lane = ends.length;
      ends.push(until);
    } else ends[lane] = until;
    return { ...b, lane };
  });
  return { ...g, bars, lanes: ends.length };
}
