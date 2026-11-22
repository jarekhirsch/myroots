// Password protection of the family website.
//
// The editor never stores the password, only a key derived from it (PBKDF2,
// SHA-256). When the website is made (ZIP export or publishing in WordPress),
// the protected part is encrypted with that key (AES-GCM): the whole website,
// or chosen people and their branches, with their photos. Visitors type the
// password; the browser derives the same key and decrypts. Without the
// password the protected data cannot be read, not even from the files.

import type { SiteLock } from './types';

export const LOCK_ITERATIONS = 250_000;
const PREFIX = 'window.MYROOTS_SITE=';

type Rec = { kind: string; id: number; data: any };

/** What data.js carries when part of the website is protected. */
export interface LockHeader {
  v: 1;
  mode: 'all' | 'people';
  salt: string;
  iterations: number;
  hint: string;
  /** Title shown on the password screen (the whole website protected). */
  title: string;
  subtitle: string;
  /** Encrypted JSON: { records: Rec[] } (base64 of IV + ciphertext). */
  data: string;
  /** Media files stored encrypted (media/<id>.bin, media/<id>_t.bin). */
  media: number[];
  /** Number of protected people (shown on the password screen). */
  people: number;
}

/* ------------------------------------------------------------------ */
/*  Crypto                                                             */
/* ------------------------------------------------------------------ */

export function b64(bytes: Uint8Array): string {
  let s = '';
  for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(s);
}

export function unb64(s: string): Uint8Array {
  const bin = atob(s);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

/** The raw key (base64) for a password; a new salt when none is given. */
export async function deriveKey(password: string, salt?: string, iterations = LOCK_ITERATIONS): Promise<{ salt: string; key: string; iterations: number }> {
  const s = salt ? unb64(salt) : crypto.getRandomValues(new Uint8Array(16));
  const base = await crypto.subtle.importKey('raw', new TextEncoder().encode(password.normalize('NFC')), 'PBKDF2', false, ['deriveBits']);
  const bits = await crypto.subtle.deriveBits({ name: 'PBKDF2', hash: 'SHA-256', salt: s as BufferSource, iterations }, base, 256);
  return { salt: b64(s), key: b64(new Uint8Array(bits)), iterations };
}

async function aesKey(raw: string): Promise<CryptoKey> {
  return crypto.subtle.importKey('raw', unb64(raw) as BufferSource, 'AES-GCM', false, ['encrypt', 'decrypt']);
}

export async function encryptBytes(key: CryptoKey, plain: Uint8Array): Promise<Uint8Array> {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const c = new Uint8Array(await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, plain as BufferSource));
  const out = new Uint8Array(12 + c.length);
  out.set(iv);
  out.set(c, 12);
  return out;
}

export async function decryptBytes(key: CryptoKey, data: Uint8Array): Promise<Uint8Array> {
  return new Uint8Array(await crypto.subtle.decrypt({ name: 'AES-GCM', iv: data.subarray(0, 12) as BufferSource }, key, data.subarray(12) as BufferSource));
}

/* ------------------------------------------------------------------ */
/*  Which records are protected                                        */
/* ------------------------------------------------------------------ */

/** People covered by the chosen people and branches. */
export function lockedPeople(records: Rec[], lock: SiteLock): Set<number> {
  const persons = new Map<number, any>();
  const children = new Map<number, number[]>();
  const partners = new Map<number, number[]>();
  for (const r of records) {
    if (r.kind === 'person' && r.data) {
      persons.set(r.id, r.data);
      for (const p of [r.data.mother, r.data.father]) if (p) children.set(p, [...(children.get(p) || []), r.id]);
    } else if (r.kind === 'union' && r.data && r.data.b) {
      partners.set(r.data.a, [...(partners.get(r.data.a) || []), r.data.b]);
      partners.set(r.data.b, [...(partners.get(r.data.b) || []), r.data.a]);
    }
  }
  const out = new Set<number>();
  for (const b of lock.people || []) {
    if (!persons.has(b.person)) continue;
    out.add(b.person);
    if (b.scope === 'descendants') {
      const stack = [b.person];
      while (stack.length) {
        const id = stack.pop()!;
        for (const p of partners.get(id) || []) out.add(p);
        for (const c of children.get(id) || []) {
          if (!out.has(c)) {
            out.add(c);
            stack.push(c);
          }
        }
      }
    } else if (b.scope === 'ancestors') {
      const stack = [b.person];
      while (stack.length) {
        const p = persons.get(stack.pop()!);
        for (const a of [p?.mother, p?.father]) {
          if (a && persons.has(a) && !out.has(a)) {
            out.add(a);
            stack.push(a);
          }
        }
      }
    }
  }
  return out;
}

function mediaOfPerson(p: any): number[] {
  return [p.photo, ...(p.gallery || []).map((g: any) => g.media), ...(p.events || []).map((e: any) => e.image), ...(p.recordings || []).map((r: any) => r.media), ...(p.aging || []).map((f: any) => f.media), p.grave?.photo].filter((x) => !!x);
}
function mediaOfPet(p: any): number[] {
  return [p.photo, ...(p.gallery || []).map((g: any) => g.media)].filter((x) => !!x);
}
function mediaOfSource(s: any): number[] {
  return [s.photo, ...(s.scans || [])].filter((x) => !!x);
}

/**
 * Splits the public records into what stays readable and what is encrypted.
 * Protected people stay in the tree as anonymous placeholders, so the
 * structure of the family is kept; everything about them is encrypted.
 */
export function splitRecords(records: Rec[], lock: SiteLock): { open: Rec[]; secret: Rec[]; media: Set<number>; people: number } {
  if (lock.mode === 'all') {
    const media = new Set(records.filter((r) => r.kind === 'media').map((r) => r.id));
    return { open: [], secret: records, media, people: records.filter((r) => r.kind === 'person').length };
  }
  const locked = lockedPeople(records, lock);
  const open: Rec[] = [];
  const secret: Rec[] = [];
  const hiddenMedia = new Set<number>();
  const usedOpen = new Set<number>();
  const mediaRecs: Rec[] = [];
  for (const r of records) {
    const d = r.data;
    if (r.kind === 'person' && d && locked.has(r.id)) {
      open.push({ kind: r.kind, id: r.id, data: { id: d.id, mother: d.mother ?? null, father: d.father ?? null, mother_rel: d.mother_rel, father_rel: d.father_rel } });
      secret.push(r);
      mediaOfPerson(d).forEach((m) => hiddenMedia.add(m));
    } else if (r.kind === 'union' && d && (locked.has(d.a) || (d.b && locked.has(d.b)))) {
      open.push({ kind: r.kind, id: r.id, data: { id: d.id, a: d.a, b: d.b ?? null } });
      secret.push(r);
    } else if (r.kind === 'pet' && d && d.owner && locked.has(d.owner)) {
      secret.push(r);
      mediaOfPet(d).forEach((m) => hiddenMedia.add(m));
    } else if (r.kind === 'source' && d && (d.persons || []).length && d.persons.every((p: number) => locked.has(p))) {
      secret.push(r);
      mediaOfSource(d).forEach((m) => hiddenMedia.add(m));
    } else if (r.kind === 'media') {
      mediaRecs.push(r);
    } else if (r.kind === 'settings' && d && d.tree_start_person && locked.has(d.tree_start_person)) {
      open.push({ ...r, data: { ...d, tree_start_person: null } });
      secret.push(r);
    } else {
      open.push(r);
      if (r.kind === 'person' && d) mediaOfPerson(d).forEach((m) => usedOpen.add(m));
      if (r.kind === 'pet' && d) mediaOfPet(d).forEach((m) => usedOpen.add(m));
      if (r.kind === 'source' && d) mediaOfSource(d).forEach((m) => usedOpen.add(m));
    }
  }
  const media = new Set<number>();
  for (const r of mediaRecs) {
    const tags: any[] = r.data?.tags || [];
    const tagsLocked = tags.some((t) => t.person && locked.has(t.person));
    if (hiddenMedia.has(r.id) && !usedOpen.has(r.id)) {
      // only the protected part uses it: the whole photo is encrypted
      secret.push(r);
      media.add(r.id);
    } else if (tagsLocked) {
      // a shared photo: visible, but the names of protected people on it are not
      open.push({ ...r, data: { ...r.data, tags: tags.filter((t) => !(t.person && locked.has(t.person))) } });
      secret.push(r);
    } else {
      open.push(r);
    }
  }
  return { open, secret, media, people: locked.size };
}

/* ------------------------------------------------------------------ */
/*  The website files                                                  */
/* ------------------------------------------------------------------ */

export function parseDataJs(bytes: Uint8Array): any {
  const s = new TextDecoder().decode(bytes).trim();
  if (!s.startsWith(PREFIX)) throw new Error('data.js has an unknown format');
  return JSON.parse(s.slice(PREFIX.length).replace(/;\s*$/, ''));
}

function dataJs(payload: any): Uint8Array {
  return new TextEncoder().encode(PREFIX + JSON.stringify(payload).replace(/<\//g, '<\\/') + ';\n');
}

export function lockActive(lock: SiteLock | undefined | null): lock is SiteLock {
  return !!lock && (lock.mode === 'all' || (lock.mode === 'people' && lock.people?.length > 0)) && !!lock.key && !!lock.salt;
}

/** A short fingerprint of the key (new password = all protected files again). */
export function lockSig(lock: SiteLock | undefined | null): string {
  return lockActive(lock) ? `${lock.mode}-${lock.salt.slice(0, 10)}` : '';
}

/**
 * Applies the protection to the files of the website (data.js and media/…):
 * returns the files to write. Encrypted photos get the extension .bin.
 */
export async function lockFiles(files: Record<string, Uint8Array>, lock: SiteLock | undefined | null, extra: { title: string; subtitle: string }): Promise<Record<string, Uint8Array>> {
  if (!lockActive(lock) || !files['data.js']) return files;
  const payload = parseDataJs(files['data.js']);
  const { open, secret, media, people } = splitRecords(payload.records || [], lock);
  const key = await aesKey(lock.key);
  const enc = await encryptBytes(key, new TextEncoder().encode(JSON.stringify({ records: secret })));
  const header: LockHeader = { v: 1, mode: lock.mode as 'all' | 'people', salt: lock.salt, iterations: lock.iterations || LOCK_ITERATIONS, hint: lock.hint || '', title: extra.title, subtitle: extra.subtitle, data: b64(enc), media: [...media], people };
  const out: Record<string, Uint8Array> = {};
  for (const [name, bytes] of Object.entries(files)) {
    const m = /^media\/(\d+)(_t)?\.[a-z0-9]+$/.exec(name);
    if (m && media.has(Number(m[1]))) {
      out[`media/${m[1]}${m[2] || ''}.bin`] = await encryptBytes(key, bytes);
    } else if (name !== 'data.js') {
      out[name] = bytes;
    }
  }
  out['data.js'] = dataJs({ ...payload, records: open, lock: header });
  return out;
}

/* ------------------------------------------------------------------ */
/*  Visitors: unlocking                                                */
/* ------------------------------------------------------------------ */

const STORE = 'myroots.unlock.';

/** A key remembered in this browser (for this website and this password). */
export function savedKey(h: LockHeader): string | null {
  for (const st of [() => sessionStorage, () => localStorage]) {
    try {
      const v = st().getItem(STORE + h.salt);
      if (v) return v;
    } catch {
      /* storage blocked */
    }
  }
  return null;
}

export function rememberKey(h: LockHeader, key: string, persist: boolean) {
  try {
    (persist ? localStorage : sessionStorage).setItem(STORE + h.salt, key);
  } catch {
    /* storage blocked: the password is asked again next time */
  }
}

export function forgetKey(h: LockHeader) {
  for (const st of [() => sessionStorage, () => localStorage]) {
    try {
      st().removeItem(STORE + h.salt);
    } catch {
      /* storage blocked */
    }
  }
}

/** Decrypts the protected records; throws when the key (password) is wrong. */
export async function openSecret(h: LockHeader, rawKey: string): Promise<{ records: Rec[]; key: CryptoKey }> {
  const key = await aesKey(rawKey);
  const plain = await decryptBytes(key, unb64(h.data));
  return { records: JSON.parse(new TextDecoder().decode(plain)).records, key };
}

/** Tries a password; on success returns the raw key. */
export async function tryPassword(h: LockHeader, password: string): Promise<string | null> {
  const { key } = await deriveKey(password, h.salt, h.iterations);
  try {
    await openSecret(h, key);
    return key;
  } catch {
    return null;
  }
}

/** The public records with the protected ones put back. */
export function mergeRecords(open: Rec[], secret: Rec[]): Rec[] {
  const map = new Map(open.map((r) => [`${r.kind}:${r.id}`, r]));
  for (const r of secret) map.set(`${r.kind}:${r.id}`, r);
  return [...map.values()];
}

/** The website ZIP with the protection applied (the export of the app). */
export async function lockZip(zip: Uint8Array, settings: any): Promise<Uint8Array> {
  const lock = settings?.site?.lock as SiteLock | undefined;
  if (!lockActive(lock)) return zip;
  const { unzipSync, zipSync } = await import('fflate');
  const files = await lockFiles(unzipSync(zip), lock, { title: settings.site?.title || settings.tree_name || '', subtitle: settings.site?.subtitle || '' });
  return zipSync(files, { level: 6 });
}
