// Gettext-compatible translations. The UI uses the same English source strings
// as the WordPress plugin, so its 11 translations work out of the box.

type Catalog = { locale: string; plural: string; messages: Record<string, string[]> };

export const LANGUAGES: { code: string; name: string; locale: string }[] = [
  { code: 'en', name: 'English', locale: 'en-GB' },
  { code: 'pl', name: 'Polski', locale: 'pl-PL' },
  { code: 'de', name: 'Deutsch', locale: 'de-DE' },
  { code: 'fr', name: 'Français', locale: 'fr-FR' },
  { code: 'es', name: 'Español', locale: 'es-ES' },
  { code: 'it', name: 'Italiano', locale: 'it-IT' },
  { code: 'pt', name: 'Português', locale: 'pt-PT' },
  { code: 'nl', name: 'Nederlands', locale: 'nl-NL' },
  { code: 'sv', name: 'Svenska', locale: 'sv-SE' },
  { code: 'fi', name: 'Suomi', locale: 'fi-FI' },
  { code: 'da', name: 'Dansk', locale: 'da-DK' },
  { code: 'he', name: 'עברית', locale: 'he-IL' },
];

const loaders = import.meta.glob('./locales/*.json');

let state = $state({ lang: 'en', catalog: null as Catalog | null, pluralFn: ((n: number) => (n === 1 ? 0 : 1)) as (n: number) => number });

export function lang(): string {
  return state.lang;
}

export function locale(): string {
  return LANGUAGES.find((l) => l.code === state.lang)?.locale || 'en-GB';
}

export function isRtl(): boolean {
  return state.lang === 'he';
}

export function detectLanguage(preferred?: string): string {
  const cands = [preferred, ...(navigator.languages || [navigator.language])].filter(Boolean) as string[];
  for (const c of cands) {
    const code = c.toLowerCase().split(/[-_]/)[0];
    if (code === 'iw') return 'he';
    if (LANGUAGES.some((l) => l.code === code)) return code;
  }
  return 'en';
}

export async function setLanguage(code: string) {
  if (!LANGUAGES.some((l) => l.code === code)) code = 'en';
  let catalog: Catalog | null = null;
  if (code !== 'en') {
    const loader = loaders[`./locales/${code}.json`];
    if (loader) catalog = ((await loader()) as any).default as Catalog;
  }
  const pluralFn = pluralRule(catalog?.plural || 'n != 1');
  state.lang = code;
  state.catalog = catalog;
  state.pluralFn = pluralFn;
  document.documentElement.lang = code;
  document.documentElement.dir = code === 'he' ? 'rtl' : 'ltr';
}

/** Plural-Forms of the bundled catalogs, without evaluating code (works under a strict CSP). */
function pluralRule(expr: string): (n: number) => number {
  const e = expr.replace(/\s+/g, '');
  if (e === '(n>1)' || e === 'n>1') return (n) => (n > 1 ? 1 : 0);
  if (e.includes('n%10>=2')) return (n) => (n === 1 ? 0 : n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14) ? 1 : 2);
  return (n) => (n !== 1 ? 1 : 0);
}

export function sprintf(fmt: string, ...args: unknown[]): string {
  let i = 0;
  return fmt.replace(/%(?:(\d+)\$)?([sd%])/g, (m, pos, type) => {
    if (type === '%') return '%';
    const v = pos ? args[Number(pos) - 1] : args[i++];
    if (type === 'd') return String(Math.trunc(Number(v) || 0));
    return v == null ? '' : String(v);
  });
}

function lookup(key: string): string[] | undefined {
  return state.catalog?.messages[key];
}

/** __( 'text' ) */
export function t(msgid: string, ...args: unknown[]): string {
  const s = lookup(msgid)?.[0] || msgid;
  return args.length ? sprintf(s, ...args) : s;
}

/** _x( 'text', 'context' ) — note the argument order (context first). */
export function tx(context: string, msgid: string, ...args: unknown[]): string {
  const s = lookup(context + '\u0004' + msgid)?.[0] || msgid;
  return args.length ? sprintf(s, ...args) : s;
}

/** _n( 'single', 'plural', n ) — `n` is also the first sprintf argument unless args are given. */
export function tn(single: string, plural: string, n: number, ...args: unknown[]): string {
  const forms = lookup(single);
  let s: string;
  if (forms && forms.length) s = forms[Math.min(state.pluralFn(n), forms.length - 1)] || (n === 1 ? single : plural);
  else s = n === 1 ? single : plural;
  return sprintf(s, ...(args.length ? args : [n]));
}
