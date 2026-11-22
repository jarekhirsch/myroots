import { t, tx, locale } from './i18n/index.svelte';
import type { Gender } from './types';

function parse(v: string): { q: 'exact' | 'abt' | 'bef' | 'aft' | 'bet'; d1: string; d2: string } {
  v = (v || '').trim();
  if (!v) return { q: 'exact', d1: '', d2: '' };
  if (v.includes('..')) {
    const [a, b] = v.split('..');
    return { q: 'bet', d1: a.trim(), d2: (b || '').trim() };
  }
  const c = v[0];
  if (c === '~') return { q: 'abt', d1: v.slice(1).trim(), d2: '' };
  if (c === '<') return { q: 'bef', d1: v.slice(1).trim(), d2: '' };
  if (c === '>') return { q: 'aft', d1: v.slice(1).trim(), d2: '' };
  return { q: 'exact', d1: v, d2: '' };
}

export { parse as parseDate };

function bare(v: string): string {
  if (/^\d{4}$/.test(v)) return String(Number(v));
  if (/^\d{4}-\d{2}$/.test(v)) {
    const d = new Date(Number(v.slice(0, 4)), Number(v.slice(5, 7)) - 1, 1);
    d.setFullYear(Number(v.slice(0, 4)));
    return new Intl.DateTimeFormat(locale(), { month: 'long', year: 'numeric' }).format(d);
  }
  if (/^\d{4}-\d{2}-\d{2}$/.test(v)) {
    const d = new Date(2000, Number(v.slice(5, 7)) - 1, Number(v.slice(8, 10)));
    d.setFullYear(Number(v.slice(0, 4)));
    return new Intl.DateTimeFormat(locale(), { day: 'numeric', month: 'long', year: 'numeric' }).format(d);
  }
  return v;
}

/** "approx. 1920", "before May 1920", "between 1900 and 1910", "15 May 1920". */
export function fmtDate(v: string | null | undefined): string {
  const p = parse(v || '');
  if (!p.d1) return '';
  if (p.q === 'bet') return t('between %1$s and %2$s', bare(p.d1), bare(p.d2));
  const core = bare(p.d1);
  switch (p.q) {
    case 'abt':
      return t('approx. %s', core);
    case 'bef':
      return t('before %s', core);
    case 'aft':
      return t('after %s', core);
    default:
      return core;
  }
}

/** Short numeric form (lists): 15.05.1920 / 05.1920 / 1920 with qualifier symbol. */
export function fmtDateShort(v: string): string {
  const p = parse(v || '');
  const b = (x: string) => {
    if (/^\d{4}-\d{2}-\d{2}$/.test(x)) return new Intl.DateTimeFormat(locale(), { day: '2-digit', month: '2-digit', year: 'numeric' }).format(new Date(Number(x.slice(0, 4)), Number(x.slice(5, 7)) - 1, Number(x.slice(8, 10))));
    if (/^\d{4}-\d{2}$/.test(x)) return `${x.slice(5, 7)}.${x.slice(0, 4)}`;
    return x;
  };
  if (!p.d1) return '';
  if (p.q === 'bet') return `${b(p.d1)}–${b(p.d2)}`;
  return ({ abt: '~', bef: '<', aft: '>', exact: '' } as const)[p.q] + b(p.d1);
}

export function year(v: string): number | null {
  const m = (parse(v).d1 || '').match(/^(\d{4})/);
  return m ? Number(m[1]) : null;
}

export function genderLabel(g: Gender): string {
  return g === 'M' ? t('Male') : g === 'F' ? t('Female') : g === 'U' ? tx('gender', 'Unknown') : '';
}

/** Option labels (select). */
export function relLabel(r: string): string {
  return ({ bio: t('Biological'), adopted: t('Adoption'), foster: t('Foster parent'), step: t('Step-parent (stepfather/stepmother)') } as Record<string, string>)[r] || r;
}

/** Small tag next to a parent on a profile ('' for biological). */
export function relTag(r: string): string {
  return ({ adopted: t('adoption'), foster: t('foster parent'), step: t('step-parent') } as Record<string, string>)[r] || '';
}

export function endCauseLabel(v: string): string {
  return ({ '': t('Ongoing'), ongoing: t('Ongoing'), widowed: t('Death of spouse'), divorced: t('Divorce'), other: t('Other/Unknown') } as Record<string, string>)[v] ?? v;
}

export function speciesLabel(k: string): string {
  return ({ dog: t('Dog'), cat: t('Cat'), bird: t('Bird'), horse: t('Horse'), other: t('Other') } as Record<string, string>)[k] || k;
}

export const BREEDS: Record<string, string[]> = {
  dog: ['Mixed breed', 'German Shepherd', 'Labrador Retriever', 'Golden Retriever', 'Dachshund', 'Border Collie', 'French Bulldog', 'Yorkshire Terrier'],
  cat: ['Domestic cat', 'British Shorthair', 'Maine Coon', 'Persian', 'Siamese', 'Ragdoll'],
  bird: ['Budgerigar', 'Canary', 'Cockatiel'],
  horse: [],
  other: [],
};

export const FIELD_LABELS: Record<string, () => string> = {
  first_name: () => t('First name'),
  last_name: () => t('Surname'),
  maiden_name: () => t('Maiden name'),
  gender: () => t('Gender'),
  birth_date: () => t('Date of birth'),
  birth_place: () => t('Place of birth'),
  death_date: () => t('Date of death'),
  death_place: () => t('Place of death'),
  death_cause: () => t('Cause of death'),
  occupation: () => t('Occupation'),
  education: () => t('Education'),
  faith: () => t('Religion'),
  height: () => t('Height'),
  eye_color: () => t('Eye color'),
  blood_type: () => t('Blood type'),
  residence: () => t('Address'),
  biography: () => t('Biography'),
  notes: () => t('Notes'),
  research: () => t('Unconfirmed information'),
  mother: () => t('Mother'),
  father: () => t('Father'),
  mother_rel: () => t('Mother'),
  father_rel: () => t('Father'),
  spouses: () => t('Spouses'),
  events: () => t('Events'),
  clans: () => t('Clans'),
  photo: () => t('Profile photo'),
  gallery: () => t('Gallery'),
  custom: () => t('Custom fields'),
  citations: () => t('Source citations'),
  name: () => t('Name'),
  burial: () => t('Grave'),
  status: () => t('Status'),
};

export function fieldLabel(k: string): string {
  // facts of events and marriages cited by a source: "event:<name>", "marriage:<partner>"
  if (k.startsWith('event:')) return k.slice(6) || t('Event');
  if (k.startsWith('marriage:')) return t('Marriage to %s', k.slice(9) || '?');
  return FIELD_LABELS[k]?.() ?? k;
}

export function fmtNumber(n: number | null | undefined, digits = 1): string {
  if (n == null) return '—';
  return new Intl.NumberFormat(locale(), { maximumFractionDigits: digits }).format(n);
}

export function fmtDateTime(ms: number): string {
  return new Intl.DateTimeFormat(locale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(ms));
}

export function initials(name: string): string {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]!.toUpperCase())
    .join('');
}

/** A link that is safe to open (http, https, mailto); anything else gives "". */
export function safeHref(u: string | null | undefined): string {
  const s = String(u ?? '').trim();
  if (/^(https?:\/\/|mailto:)/i.test(s)) return s;
  if (s && !s.includes(':') && /^[^\s/.][^\s]*\.[^\s]+$/.test(s)) return 'https://' + s;
  return '';
}
