// Layout of the whole family on one board (the plugin's "whole graph"):
// generations are rows (or columns), couples stand side by side, children
// stand together under their parents, connected by orthogonal lines.
import { generationsOf } from './generations';
import type { TreeNode } from './types';

export interface GNode {
  id: string;
  name: string;
  years: string;
  gender: string;
  photo: number | null;
  anon: boolean;
  gen: number;
  /** top-left corner */
  x: number;
  y: number;
}
export interface GEdge {
  d: string;
  kind: 'couple' | 'child';
  /** the line skips a generation or joins partners from different rows */
  odd: boolean;
  ids: string[];
}
export interface Graph {
  nodes: GNode[];
  edges: GEdge[];
  width: number;
  height: number;
  cardW: number;
  cardH: number;
}

export const CARD_W = 210;
export const CARD_H = 64;
const COUPLE_GAP = 26; // between partners (room for the marriage line)
const UNIT_GAP = 22; // between siblings
const FAMILY_GAP = 64; // between groups of siblings
const ROW_GAP = 96; // between generations (vertical layout)
const COL_GAP = 120; // between generations (horizontal layout)

type Unit = { members: string[]; key: number; family: string };

export function layoutFamily(all: TreeNode[], orient: 'vertical' | 'horizontal', privateLabel = '?'): Graph {
  const nodes = new Map<string, TreeNode>();
  for (const n of all) nodes.set(n.id, n);
  const has = (id: string) => nodes.has(id);
  const { gen, birth } = generationsOf(nodes);
  const parents = (id: string) => nodes.get(id)!.rels.parents.filter(has);
  const spouses = (id: string) => nodes.get(id)!.rels.spouses.filter(has);
  const children = (id: string) => nodes.get(id)!.rels.children.filter(has);
  const famKey = (id: string) => [...parents(id)].sort().join(',');

  const vertical = orient === 'vertical';
  const along = vertical ? CARD_W : CARD_H; // size of a card along its row
  const across = vertical ? CARD_H : CARD_W; // size across the row
  const coupleGap = vertical ? COUPLE_GAP : 18;
  const unitGap = vertical ? UNIT_GAP : 10;
  const familyGap = vertical ? FAMILY_GAP : 34;
  const rowGap = vertical ? ROW_GAP : COL_GAP;

  // rows
  const rows = new Map<number, string[]>();
  for (const id of nodes.keys()) {
    const g = gen.get(id) ?? 0;
    rows.set(g, [...(rows.get(g) || []), id]);
  }
  const rowIdx = [...rows.keys()].sort((a, b) => a - b);

  // units: partners standing next to each other in the same row
  const unitsOf = (row: string[]): string[][] => {
    const inRow = new Set(row);
    const seen = new Set<string>();
    const out: string[][] = [];
    for (const id of row) {
      if (seen.has(id)) continue;
      const group: string[] = [];
      const stack = [id];
      seen.add(id);
      while (stack.length) {
        const x = stack.pop()!;
        group.push(x);
        for (const s of spouses(x)) {
          if (inRow.has(s) && !seen.has(s)) {
            seen.add(s);
            stack.push(s);
          }
        }
      }
      out.push(order(group));
    }
    return out;
  };
  // inside a unit: people of the family in the middle, partners who married in beside them
  function order(group: string[]): string[] {
    if (group.length === 1) return group;
    const core = group.filter((id) => parents(id).length).sort((a, b) => (birth.get(a) ?? 0) - (birth.get(b) ?? 0));
    const rest = new Set(group.filter((id) => !parents(id).length));
    const seq: string[] = [];
    const base = core.length ? core : [group.sort((a, b) => (nodes.get(a)!.data.gender === 'M' ? -1 : 1) - (nodes.get(b)!.data.gender === 'M' ? -1 : 1))[0]];
    if (!core.length) rest.delete(base[0]);
    for (const c of base) {
      const mine = spouses(c).filter((s) => rest.has(s));
      mine.forEach((s) => rest.delete(s));
      const male = nodes.get(c)!.data.gender === 'M';
      // partner on the right of a man and on the left of a woman, the next ones on the other side
      const right = mine.filter((_, i) => (i % 2 === 0) === male);
      const left = mine.filter((_, i) => (i % 2 === 0) !== male);
      seq.push(...left, c, ...right);
    }
    seq.push(...rest);
    return seq;
  }

  const pos = new Map<string, number>(); // centre along the row
  const prevOrder = new Map<string, number>();
  let rowUnits = new Map<number, Unit[]>();
  for (const r of rowIdx) {
    const us = unitsOf(rows.get(r)!).map((m) => ({ members: m, key: NaN, family: famKey(m.find((x) => parents(x).length) || m[0]) }));
    us.sort((a, b) => (birth.get(a.members[0]) ?? 9999) - (birth.get(b.members[0]) ?? 9999));
    us.forEach((u, i) => u.members.forEach((m) => prevOrder.set(m, i)));
    rowUnits.set(r, us);
  }
  const mean = (xs: number[]) => (xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : NaN);
  const unitWidth = (u: Unit) => u.members.length * along + (u.members.length - 1) * coupleGap;

  function place(r: number, by: 'parents' | 'children') {
    const us = rowUnits.get(r)!;
    for (const u of us) {
      const fromParents = mean(u.members.flatMap((m) => parents(m).map((p) => pos.get(p)).filter((v): v is number => v != null)));
      const fromChildren = mean(u.members.flatMap((m) => children(m).map((c) => pos.get(c)).filter((v): v is number => v != null)));
      u.key = by === 'parents' ? (Number.isNaN(fromParents) ? fromChildren : fromParents) : Number.isNaN(fromChildren) ? fromParents : fromChildren;
    }
    us.sort((a, b) => {
      const ka = Number.isNaN(a.key) ? Infinity : a.key;
      const kb = Number.isNaN(b.key) ? Infinity : b.key;
      return ka - kb || a.family.localeCompare(b.family) || (prevOrder.get(a.members[0]) ?? 0) - (prevOrder.get(b.members[0]) ?? 0);
    });
    let cursor = -Infinity;
    let lastFamily: string | null = null;
    for (const u of us) {
      const w = unitWidth(u);
      const gap = cursor === -Infinity ? 0 : u.family && u.family === lastFamily ? unitGap : familyGap;
      const want = Number.isNaN(u.key) ? -Infinity : u.key - w / 2;
      const left = Math.max(cursor + gap, want === -Infinity ? cursor + gap : want);
      const start = Number.isFinite(left) ? left : 0;
      u.members.forEach((m, i) => pos.set(m, start + i * (along + coupleGap) + along / 2));
      cursor = start + w;
      lastFamily = u.family;
    }
    us.forEach((u, i) => u.members.forEach((m) => prevOrder.set(m, i)));
  }
  // down, up, down: children under parents, roots above their descendants
  for (const r of rowIdx) place(r, 'parents');
  for (const r of [...rowIdx].reverse()) place(r, 'children');
  for (const r of rowIdx) place(r, 'parents');

  // final positions
  const minAlong = pos.size ? Math.min(...[...pos.values()].map((v) => v - along / 2)) : 0;
  const rowPos = new Map<number, number>();
  rowIdx.forEach((r, i) => rowPos.set(r, i * (across + rowGap)));
  const xy = (u: number, v: number): [number, number] => (vertical ? [u, v] : [v, u]);
  const out: GNode[] = [];
  for (const [id, n] of nodes) {
    const u = pos.get(id)! - along / 2 - minAlong;
    const v = rowPos.get(gen.get(id) ?? 0)!;
    const [x, y] = xy(u, v);
    const name = n.data.anon ? privateLabel : [n.data['first name'], n.data['last name']].filter(Boolean).join(' ').trim() || '—';
    out.push({ id, name, years: n.data.years || '', gender: n.data.gender, photo: n.data.avatar, anon: !!n.data.anon, gen: gen.get(id) ?? 0, x, y });
  }
  const at = new Map(out.map((g) => [g.id, g]));
  const U = (id: string) => pos.get(id)! - minAlong; // centre along
  const V = (id: string) => rowPos.get(gen.get(id) ?? 0)!; // row start across
  const P = (u: number, v: number) => xy(u, v).map((n) => Math.round(n * 10) / 10).join(' ');

  // couples
  const edges: GEdge[] = [];
  const done = new Set<string>();
  for (const id of nodes.keys()) {
    for (const s of spouses(id)) {
      const k = [id, s].sort().join('|');
      if (done.has(k)) continue;
      done.add(k);
      const [a, b] = U(id) <= U(s) ? [id, s] : [s, id];
      if (V(a) === V(b)) {
        const mid = V(a) + across / 2;
        edges.push({ kind: 'couple', odd: false, ids: [a, b], d: `M ${P(U(a) + along / 2, mid)} L ${P(U(b) - along / 2, mid)}` });
      } else {
        const ua = U(a), ub = U(b);
        const va = V(a) + across, vb = V(b) + across;
        const low = Math.max(va, vb) + rowGap / 3;
        edges.push({ kind: 'couple', odd: true, ids: [a, b], d: `M ${P(ua, va)} L ${P(ua, low)} L ${P(ub, low)} L ${P(ub, vb)}` });
      }
    }
  }
  // children, one bundle per set of parents
  const fams = new Map<string, string[]>();
  for (const id of nodes.keys()) {
    const ps = parents(id);
    if (!ps.length) continue;
    const k = [...ps].sort().join(',');
    fams.set(k, [...(fams.get(k) || []), id]);
  }
  let fi = 0;
  for (const [k, kids] of fams) {
    const ps = k.split(',');
    const couple = ps.length === 2 && V(ps[0]) === V(ps[1]);
    const startU = mean(ps.map(U));
    const startV = couple ? V(ps[0]) + across / 2 : Math.max(...ps.map((p) => V(p) + across));
    const kidRow = Math.min(...kids.map(V));
    const bus = kidRow - rowGap / 2 + ((fi++ % 3) - 1) * 7;
    const us = kids.map(U);
    const lo = Math.min(startU, ...us);
    const hi = Math.max(startU, ...us);
    let d = `M ${P(startU, startV)} L ${P(startU, bus)} M ${P(lo, bus)} L ${P(hi, bus)}`;
    for (const c of kids) d += ` M ${P(U(c), bus)} L ${P(U(c), V(c))}`;
    const parentRow = Math.max(...ps.map((p) => gen.get(p) ?? 0));
    const odd = kids.some((c) => (gen.get(c) ?? 0) - parentRow !== 1);
    edges.push({ kind: 'child', odd, ids: [...ps, ...kids], d });
  }

  const width = out.length ? Math.max(...out.map((n) => n.x + CARD_W)) + 2 : 0;
  const height = out.length ? Math.max(...out.map((n) => n.y + CARD_H)) + 2 : 0;
  void at;
  return { nodes: out, edges, width, height, cardW: CARD_W, cardH: CARD_H };
}
