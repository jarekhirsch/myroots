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
