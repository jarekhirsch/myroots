// Geocoding of place names with OpenStreetMap Nominatim (opt-in, 1 request per second).
import { call } from './api.svelte';
import { backend } from './backend';
import { lang } from './i18n/index.svelte';
import type { PlaceUse } from './types';

export const TILES: Record<string, { url: string; attribution: string; sub?: string; max?: number }> = {
  osm: { url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png', attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>', max: 19 },
  light: { url: 'https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png', attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> &copy; <a href="https://carto.com/attributions">CARTO</a>', sub: 'abcd', max: 19 },
  dark: { url: 'https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png', attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> &copy; <a href="https://carto.com/attributions">CARTO</a>', sub: 'abcd', max: 19 },
  topo: { url: 'https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png', attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>, <a href="https://opentopomap.org">OpenTopoMap</a> (CC-BY-SA)', sub: 'abc', max: 17 },
};

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

type Hit = { label: string; lat: number; lng: number };

async function nominatim(q: string, limit: number): Promise<Hit[]> {
  const url = `https://nominatim.openstreetmap.org/search?format=jsonv2&limit=${limit}&accept-language=${encodeURIComponent(lang())}&q=${encodeURIComponent(q)}`;
  const r = await backend.fetchBytes(url, { Accept: 'application/json' });
  if (r.status !== 200) throw new Error(r.status === 429 ? 'rate' : 'http');
  const v = JSON.parse(new TextDecoder().decode(r.bytes));
  if (!Array.isArray(v)) return [];
  return v
    .map((x: any) => ({ label: String(x.display_name || ''), lat: Number(x.lat), lng: Number(x.lon) }))
    .filter((x) => x.label && Number.isFinite(x.lat) && Number.isFinite(x.lng));
}

// Photon (komoot): the same OpenStreetMap data, more forgiving with spelling
// and small villages; asked when Nominatim finds nothing or does not answer.
async function photon(q: string, limit: number): Promise<Hit[]> {
  const l = ['de', 'en', 'fr', 'it'].includes(lang()) ? `&lang=${lang()}` : '';
  const r = await backend.fetchBytes(`https://photon.komoot.io/api/?limit=${limit}${l}&q=${encodeURIComponent(q)}`, { Accept: 'application/json' });
  if (r.status !== 200) throw new Error(r.status === 429 ? 'rate' : 'http');
  const v = JSON.parse(new TextDecoder().decode(r.bytes));
  return (Array.isArray(v?.features) ? v.features : [])
    .map((f: any) => {
      const p = f.properties || {};
      const label = [p.name, p.city !== p.name ? p.city : '', p.county, p.state, p.country].filter(Boolean).join(', ');
      return { label, lat: Number(f.geometry?.coordinates?.[1]), lng: Number(f.geometry?.coordinates?.[0]) };
    })
    .filter((x: Hit) => x.label && Number.isFinite(x.lat) && Number.isFinite(x.lng));
}

/** Matches for a place name: OpenStreetMap Nominatim, then Photon. */
async function lookup(q: string, limit: number): Promise<Hit[]> {
  let first: unknown = null;
  try {
    const r = await nominatim(q, limit);
    if (r.length) return r;
  } catch (e) {
    first = e;
  }
  try {
    return await photon(q, limit);
  } catch (e) {
    throw first || e;
  }
}

export async function geocodeOne(name: string): Promise<{ lat: number; lng: number } | null> {
  const r = await lookup(name, 1).catch((e) => {
    if ((e as Error)?.message === 'rate') throw e;
    return [];
  });
  return r[0] ? { lat: r[0].lat, lng: r[0].lng } : null;
}

/** Up to `limit` matches for a search text (for correcting a place by hand). */
export async function searchPlaces(q: string, limit = 5): Promise<Hit[]> {
  return lookup(q, limit);
}

export interface GeoProgress {
  done: number;
  total: number;
  found: number;
  failed: number;
}

/** Geocodes places that have no coordinates and were never tried. */
export async function geocodeMissing(onProgress: (p: GeoProgress) => void, isCancelled: () => boolean, retryFailed = false): Promise<GeoProgress> {
  const places = await call<PlaceUse[]>('places.list');
  const todo = places.filter((p) => (p.lat == null || p.lng == null) && (retryFailed || p.origin !== 'failed')).map((p) => p.name);
  const prog: GeoProgress = { done: 0, total: todo.length, found: 0, failed: 0 };
  onProgress({ ...prog });
  for (const name of todo) {
    if (isCancelled()) break;
    try {
      const r = await geocodeOne(name);
      if (r) {
        await call('places.set', { name, lat: r.lat, lng: r.lng, origin: 'nominatim' });
        prog.found++;
      } else {
        await call('places.set', { name, lat: null, lng: null, origin: 'failed' });
        prog.failed++;
      }
    } catch (e) {
      if ((e as Error)?.message === 'rate') {
        await sleep(5000);
        continue;
      }
      prog.failed++;
    }
    prog.done++;
    onProgress({ ...prog });
    await sleep(1100);
  }
  return prog;
}
