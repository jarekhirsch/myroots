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

export interface DNode {
  x: number;
  y: number;
  person: Who;
  partners: string[];
  depth: number;
}
export interface Desc {
  W: number;
  H: number;
  nodes: DNode[];
  links: { x1: number; y1: number; x2: number; y2: number }[];
  box: [number, number];
  scale: number;
  titleY: number;
  /** the paper is landscape */
  landscape: boolean;
  count: number;
}

export function descendants(nodes: TreeNode[], rootId: string, gens: number, paper: [number, number]): Desc | null {
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const start = byId.get(rootId);
  if (!start) return null;
  type T = { n: TreeNode; children: T[] };
  const seen = new Set<string>();
  const build = (n: TreeNode, d: number): T => {
    seen.add(n.id);
    const kids = d + 1 < gens ? n.rels.children.map((c) => byId.get(c)).filter((x): x is TreeNode => !!x && !seen.has(x.id)) : [];
    kids.sort((a, b) => (a.data.birthday || '').localeCompare(b.data.birthday || ''));
    return { n, children: kids.map((k) => build(k, d + 1)) };
  };
  const root = hierarchy<T>(build(start, 0));
  // design units: a box of 52 × 17
  const bw = 52;
  const bh = 17;
  d3tree<T>().nodeSize([bw + 6, bh + 16])(root);
  const all = root.descendants();
  const xs = all.map((d) => d.x!);
  const ys = all.map((d) => d.y!);
  const minX = Math.min(...xs) - bw / 2;
  const maxX = Math.max(...xs) + bw / 2;
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys) + bh;
  const tw = maxX - minX;
  const th = maxY - minY;
  // the orientation that gives bigger boxes
  const fit = (W: number, H: number) => {
    const m = Math.min(W, H) * 0.05;
    return Math.min((W - 2 * m) / tw, (H - 2 * m - H * 0.09) / th);
  };
  const [a, b] = paper;
  const land = fit(Math.max(a, b), Math.min(a, b)) > fit(Math.min(a, b), Math.max(a, b));
  const W = land ? Math.max(a, b) : Math.min(a, b);
  const H = land ? Math.min(a, b) : Math.max(a, b);
  const s = fit(W, H);
  const m = Math.min(W, H) * 0.05;
  const ox = (W - tw * s) / 2 - minX * s;
  const oy = m + H * 0.09 - minY * s;
  const name = (id: string) => {
    const n = byId.get(id);
    return n ? [n.data['first name'], n.data['last name']].filter(Boolean).join(' ') : '';
  };
  const out: DNode[] = all.map((d) => ({
    x: ox + d.x! * s,
    y: oy + d.y! * s,
    person: who(d.data.n)!,
    partners: d.data.n.rels.spouses.map(name).filter(Boolean),
    depth: d.depth,
  }));
  const links = root.links().map((l) => ({ x1: ox + l.source.x! * s, y1: oy + (l.source.y! + bh) * s, x2: ox + l.target.x! * s, y2: oy + l.target.y! * s }));
  return { W, H, nodes: out, links, box: [bw * s, bh * s], scale: s, titleY: m + H * 0.05, landscape: land, count: all.length };
}
