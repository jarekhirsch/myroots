/** Lowercase, Latin accents folded (search and sorting), same idea as `model::norm` in Rust. */
export function norm(s: string): string {
  return (s || '')
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/ł/g, 'l')
    .replace(/ß/g, 'ss')
    .replace(/æ/g, 'ae')
    .replace(/ø/g, 'o')
    .replace(/œ/g, 'oe')
    .replace(/[^\p{L}\p{N}\s]/gu, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

/** Collator for names in the current language. */
export function collator(locale: string) {
  return new Intl.Collator(locale, { sensitivity: 'base', numeric: true });
}

/** Plain text from the stored HTML subset. */
export function stripHtml(html: string): string {
  // DOMParser builds an inert document: no images load, no handlers run.
  const d = new DOMParser().parseFromString(html || '', 'text/html');
  return (d.body.textContent || '').trim();
}

const PAIRS: [string, string][] = [['ski', 'ska'], ['cki', 'cka'], ['dzki', 'dzka'], ['ský', 'ská'], ['cký', 'cká']];

/** Polish / Czech surnames inflect by gender: Kowalska → Kowalski. Others unchanged. */
export function masculineSurname(s: string): string {
  return (s || '')
    .split('-')
    .map((w) => {
      for (const [m, f] of PAIRS) if (w.toLowerCase().endsWith(f) && w.length > f.length + 1) return w.slice(0, -f.length) + m;
      return w;
    })
    .join('-');
}

/** Kowalski → Kowalska. Others unchanged. */
export function feminineSurname(s: string): string {
  return (s || '')
    .split('-')
    .map((w) => {
      for (const [m, f] of PAIRS) if (w.toLowerCase().endsWith(m) && w.length > m.length + 1) return w.slice(0, -m.length) + f;
      return w;
    })
    .join('-');
}

/** Stable colour index 1–6 for a name (portraits without a photo). */
export function tone(s: string): number {
  let h = 0;
  for (const c of s || '?') h = (h * 31 + c.charCodeAt(0)) >>> 0;
  return (h % 6) + 1;
}

// Normalized search texts, computed once per loaded list. Lists come from the
// engine as plain arrays that are replaced (never changed) when data changes,
// so the array itself is the cache key. Typing in a search box of a tree with
// tens of thousands of people no longer normalizes every name on every key.
const hayCache = new WeakMap<object, Map<string, string[]>>();

/** `norm(text(item))` for every item of `list`, cached per list and `key`. */
export function haystack<T>(list: readonly T[], key: string, text: (x: T) => string): string[] {
  let m = hayCache.get(list);
  if (!m) {
    m = new Map();
    hayCache.set(list, m);
  }
  let h = m.get(key);
  if (!h) {
    h = list.map((x) => norm(text(x)));
    m.set(key, h);
  }
  return h;
}

/** Search words of a query (normalized). */
export function words(q: string): string[] {
  return norm(q).split(/\s+/).filter(Boolean);
}

/**
 * Items of `list` whose search text contains every word, at most `limit`
 * (stops early, so short result lists from huge trees are instant).
 */
export function search<T>(list: readonly T[], key: string, text: (x: T) => string, q: string | string[], limit = Infinity, keep?: (x: T) => boolean): T[] {
  const w = Array.isArray(q) ? q : words(q);
  const out: T[] = [];
  if (!w.length) {
    for (const x of list) {
      if (keep && !keep(x)) continue;
      out.push(x);
      if (out.length >= limit) break;
    }
    return out;
  }
  const hay = haystack(list, key, text);
  for (let i = 0; i < list.length; i++) {
    const x = list[i];
    if (keep && !keep(x)) continue;
    const h = hay[i];
    if (w.every((s) => h.includes(s))) {
      out.push(x);
      if (out.length >= limit) break;
    }
  }
  return out;
}
