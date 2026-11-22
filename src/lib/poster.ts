// Geometry of the printable posters: a fan of ancestors and a tree of
// descendants. Units are millimetres of the paper.
import { hierarchy, tree as d3tree } from 'd3-hierarchy';
import type { TreeNode } from './types';

export const PAPERS: Record<string, [number, number]> = {
  A4: [210, 297],
  A3: [297, 420],
  A2: [420, 594],
  A1: [594, 841],
  A0: [841, 1189],
  '2A0': [1189, 1682],
  '4A0': [1682, 2378],
};

export interface Who {
  id: string;
  first: string;
  last: string;
  years: string;
  gender: string;
}
const who = (n: TreeNode | undefined): Who | null =>
  n ? { id: n.id, first: n.data['first name'] || '', last: n.data['last name'] || '', years: n.data.years || '', gender: n.data.gender || '' } : null;

/* ---------------- fan of ancestors ---------------- */

export interface FanCell {
  gen: number;
  /** angles (radians, SVG: 0 = right, growing clockwise) */
  a0: number;
  a1: number;
  r0: number;
  r1: number;
  person: Who | null;
  /** 0 = father's side, 1 = mother's side */
  side: number;
}
export interface Fan {
  W: number;
  H: number;
  cx: number;
  cy: number;
  R: number;
  r0: number;
  root: Who | null;
  cells: FanCell[];
  titleY: number;
}

/** A fan opened 230° (a semicircle and a little more), the person in the centre. */
export function fan(nodes: TreeNode[], rootId: string, gens: number, paper: [number, number]): Fan {
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const [W, H] = [Math.max(...paper), Math.min(...paper)];
  const margin = W * 0.035;
  const title = H * 0.1;
  const spread = (25 * Math.PI) / 180;
  // width 2R, height R (1 + sin spread) + title
  const R = Math.min((W - 2 * margin) / 2, (H - 2 * margin - title) / (1 + Math.sin(spread)));
  const cx = W / 2;
  const cy = margin + R;
  const r0 = R / (gens + 0.6);
  const ring = (R - r0) / (gens - 1);
  // Ahnentafel: 1 = the person, 2n = father, 2n+1 = mother
  const anc = new Map<number, TreeNode>();
  const start = byId.get(rootId);
  if (start) anc.set(1, start);
  for (let k = 1; k < 2 ** gens; k++) {
    const n = anc.get(k);
    if (!n) continue;
    const parents = n.rels.parents.map((p) => byId.get(p)).filter((x): x is TreeNode => !!x);
    let father = parents.find((p) => p.data.gender === 'M');
    let mother = parents.find((p) => p.data.gender === 'F');
    const rest = parents.filter((p) => p !== father && p !== mother);
    if (!father) father = rest.shift();
    if (!mother) mother = rest.shift();
    if (father) anc.set(2 * k, father);
    if (mother) anc.set(2 * k + 1, mother);
  }
  const from = Math.PI - spread;
  const span = Math.PI + 2 * spread;
  const cells: FanCell[] = [];
  for (let g = 1; g < gens; g++) {
    const n = 2 ** g;
    for (let i = 0; i < n; i++) {
      const k = n + i;
      cells.push({
        gen: g,
        a0: from + (span * i) / n,
        a1: from + (span * (i + 1)) / n,
        r0: r0 + ring * (g - 1),
        r1: r0 + ring * g,
        person: who(anc.get(k)),
        side: i < n / 2 ? 0 : 1,
      });
    }
  }
  return { W, H, cx, cy, R, r0, root: who(start), cells, titleY: cy + R * Math.sin(spread) + title * 0.55 };
}

/** SVG path of an annular sector. */
export function sector(cx: number, cy: number, c: FanCell): string {
  const p = (r: number, a: number) => `${(cx + r * Math.cos(a)).toFixed(2)} ${(cy + r * Math.sin(a)).toFixed(2)}`;
  const large = c.a1 - c.a0 > Math.PI ? 1 : 0;
  return `M${p(c.r0, c.a0)} L${p(c.r1, c.a0)} A${c.r1} ${c.r1} 0 ${large} 1 ${p(c.r1, c.a1)} L${p(c.r0, c.a1)} A${c.r0} ${c.r0} 0 ${large} 0 ${p(c.r0, c.a0)}Z`;
}

/** Arc along the middle of a sector, for text written along the ring. */
export function midArc(cx: number, cy: number, c: FanCell, r: number): string {
  const p = (a: number) => `${(cx + r * Math.cos(a)).toFixed(2)} ${(cy + r * Math.sin(a)).toFixed(2)}`;
  return `M${p(c.a0)} A${r} ${r} 0 0 1 ${p(c.a1)}`;
}

/* ---------------- tree of descendants ---------------- */

/** Cards as in the family tree of the app (design units, like the screen). */
export const CARD_W = 210;
export const CARD_H = 64;
const COUPLE_GAP = 26;
const UNIT_GAP = 22;
const FAMILY_GAP = 64;
const ROW_GAP = 96;

export interface DUnit {
  /** left edge and top of the person's card (design units) */
  x: number;
  y: number;
  person: Who;
  spouse: Who | null;
  /** further partners, not drawn */
  more: number;
}
export interface Desc {
  W: number;
  H: number;
  units: DUnit[];
  /** connectors in design units: couples and parents → children */
  couples: string[];
  links: string[];
  /** design units → millimetres */
  scale: number;
  ox: number;
  oy: number;
  titleY: number;
  landscape: boolean;
  count: number;
}

export function descendants(nodes: TreeNode[], rootId: string, gens: number, paper: [number, number]): Desc | null {
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const start = byId.get(rootId);
  if (!start) return null;
  type T = { n: TreeNode; spouse: TreeNode | null; more: number; w: number; children: T[] };
  const seen = new Set<string>();
  const build = (n: TreeNode, d: number): T => {
    seen.add(n.id);
    const sp = n.rels.spouses.map((x) => byId.get(x)).filter((x): x is TreeNode => !!x);
    const spouse = sp.find((x) => !seen.has(x.id)) || null;
    if (spouse) seen.add(spouse.id);
    const kids = d + 1 < gens ? n.rels.children.map((c) => byId.get(c)).filter((x): x is TreeNode => !!x && !seen.has(x.id)) : [];
    kids.sort((a, b) => (a.data.birthday || '').localeCompare(b.data.birthday || ''));
    return { n, spouse, more: Math.max(0, sp.length - 1), w: spouse ? 2 * CARD_W + COUPLE_GAP : CARD_W, children: kids.map((k) => build(k, d + 1)) };
  };
  const root = hierarchy<T>(build(start, 0));
  d3tree<T>()
    .nodeSize([1, CARD_H + ROW_GAP])
    .separation((a, b) => (a.data.w + b.data.w) / 2 + (a.parent === b.parent ? UNIT_GAP : FAMILY_GAP))(root);
  const all = root.descendants();
  const left = (d: (typeof all)[number]) => d.x! - d.data.w / 2;
  const minX = Math.min(...all.map(left));
  const maxX = Math.max(...all.map((d) => d.x! + d.data.w / 2));
  const minY = Math.min(...all.map((d) => d.y!));
  const maxY = Math.max(...all.map((d) => d.y!)) + CARD_H;
  const tw = maxX - minX;
  const th = maxY - minY;
  const fit = (W: number, H: number) => {
    const m = Math.min(W, H) * 0.05;
    return Math.min((W - 2 * m) / tw, (H - 2 * m - H * 0.1) / th);
  };
  const [a, b] = paper;
  const land = fit(Math.max(a, b), Math.min(a, b)) > fit(Math.min(a, b), Math.max(a, b));
  const W = land ? Math.max(a, b) : Math.min(a, b);
  const H = land ? Math.min(a, b) : Math.max(a, b);
  const s = fit(W, H);
  const m = Math.min(W, H) * 0.05;
  const units: DUnit[] = all.map((d) => ({ x: left(d) - minX, y: d.y! - minY, person: who(d.data.n)!, spouse: who(d.data.spouse ?? undefined), more: d.data.more }));
  const couples: string[] = [];
  const links: string[] = [];
  for (const d of all) {
    const x0 = left(d) - minX;
    const y0 = d.y! - minY;
    let fromX = x0 + CARD_W / 2;
    let fromY = y0 + CARD_H;
    if (d.data.spouse) {
      couples.push(`M ${x0 + CARD_W} ${y0 + CARD_H / 2} L ${x0 + CARD_W + COUPLE_GAP} ${y0 + CARD_H / 2}`);
      fromX = x0 + CARD_W + COUPLE_GAP / 2;
      fromY = y0 + CARD_H / 2;
    }
    for (const c of d.children || []) {
      const cx = left(c) - minX + CARD_W / 2;
      const cy = c.y! - minY;
      const mid = cy - ROW_GAP / 2;
      links.push(`M ${fromX} ${fromY} L ${fromX} ${mid} L ${cx} ${mid} L ${cx} ${cy}`);
    }
  }
  return { W, H, units, couples, links, scale: s, ox: (W - tw * s) / 2, oy: m + H * 0.1, titleY: m + H * 0.05, landscape: land, count: all.length + all.filter((d) => d.data.spouse).length };
}
