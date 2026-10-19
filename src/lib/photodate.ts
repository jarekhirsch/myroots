// When was a photo taken? From the people marked on it: everyone was already
// born and still alive (hard limits), and the age each face looks (estimated
// on the device) added to that person's year of birth.
import { year } from './format';
import { estimateAges } from './faces';
import type { TagView } from './types';

export interface DateGuess {
  /** Most likely year, when ages could be estimated. */
  best: number | null;
  from: number;
  to: number;
  /** Who the guess is based on, with the estimated age. */
  basis: { name: string; age: number | null; born: number }[];
}

export async function guessPhotoDate(src: string, tags: TagView[]): Promise<DateGuess | null> {
  const now = new Date().getFullYear();
  const people = tags.filter((t) => t.person && year(t.birth_date) != null);
  if (!people.length) return null;
  let lo = Math.max(...people.map((t) => year(t.birth_date)!));
  const deaths = tags.map((t) => year(t.death_date)).filter((y): y is number => y != null);
  let hi = Math.min(now, ...deaths);
  if (hi < lo) hi = lo;

  let ages: (number | null)[] = people.map(() => null);
  try {
    ages = await estimateAges(src, people.map((t) => t.rect));
  } catch {
    /* only the limits */
  }
  // weighted mean of "born + looks N years old"; older faces are judged less precisely
  let w = 0;
  let sum = 0;
  people.forEach((t, i) => {
    const a = ages[i];
    if (a == null) return;
    const sigma = 3 + 0.12 * a;
    const k = 1 / (sigma * sigma);
    w += k;
    sum += k * (year(t.birth_date)! + a);
  });
  const basis = people.map((t, i) => ({ name: t.label, age: ages[i] == null ? null : Math.round(ages[i]!), born: year(t.birth_date)! }));
  if (!w) return { best: null, from: lo, to: hi, basis };
  const mean = sum / w;
  const spread = 1.5 * (Math.sqrt(1 / w) + 1);
  const best = Math.min(hi, Math.max(lo, Math.round(mean)));
  const from = Math.max(lo, Math.floor(mean - spread));
  const to = Math.min(hi, Math.ceil(mean + spread));
  return { best, from: Math.min(from, best), to: Math.max(to, best), basis };
}

/** The guess as a stored (qualified) date: "1952", "~1952" or "1931..1961". */
export function guessToDate(g: DateGuess): string {
  if (g.best != null) return g.to - g.from <= 2 ? String(g.best) : `~${g.best}`;
  return g.from === g.to ? String(g.from) : `${g.from}..${g.to}`;
}
