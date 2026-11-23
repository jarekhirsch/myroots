// Cemeteries of OpenStreetMap around the visible part of the map (Overpass
// API). Open data (ODbL): the map already credits "© OpenStreetMap contributors".
import { backend } from './backend';

export interface OsmCemetery {
  key: string;
  name: string;
  address: string;
  lat: number;
  lng: number;
}

const cache = new Map<string, OsmCemetery[]>();
const SERVERS = ['https://overpass-api.de/api/interpreter', 'https://overpass.private.coffee/api/interpreter', 'https://overpass.kumi.systems/api/interpreter'];

/** Cemeteries inside the box (south, west, north, east); at most 300. */
export async function osmCemeteries(s: number, w: number, n: number, e: number): Promise<OsmCemetery[]> {
  // a coarse grid, so moving the map a little reuses the answer
  const r = (v: number) => Math.round(v * 20) / 20;
  const box = [r(s) - 0.05, r(w) - 0.05, r(n) + 0.05, r(e) + 0.05].map((v) => v.toFixed(2)).join(',');
  if (cache.has(box)) return cache.get(box)!;
  const q = `[out:json][timeout:20];(way["landuse"="cemetery"](${box});relation["landuse"="cemetery"](${box});way["amenity"="grave_yard"](${box});node["amenity"="grave_yard"](${box}););out center tags 300;`;
  let json: any = null;
  // the main Overpass server is often busy: its public mirrors take over
  for (const host of SERVERS) {
    try {
      const res = await Promise.race([
        backend.fetchBytes(`${host}?data=${encodeURIComponent(q)}`, { Accept: 'application/json' }),
        new Promise<never>((_, no) => setTimeout(() => no(new Error('timeout')), 25_000)),
      ]);
      if (res.status !== 200) continue;
      json = JSON.parse(new TextDecoder().decode(res.bytes));
      break;
    } catch {
      /* the next server */
    }
  }
  if (!json) throw new Error('Overpass: no answer');
  const out: OsmCemetery[] = [];
  for (const el of json.elements || []) {
    const lat = el.lat ?? el.center?.lat;
    const lng = el.lon ?? el.center?.lon;
    if (lat == null || lng == null) continue;
    const tg = el.tags || {};
    const street = [tg['addr:street'] || tg['addr:place'], tg['addr:housenumber']].filter(Boolean).join(' ');
    const city = [tg['addr:postcode'], tg['addr:city']].filter(Boolean).join(' ');
    out.push({ key: `${el.type}/${el.id}`, name: tg.name || tg['name:pl'] || tg.official_name || '', address: [street, city].filter(Boolean).join(', '), lat, lng });
  }
  cache.set(box, out);
  return out;
}

/** Distance in metres (short distances). */
export function metres(a: [number, number], b: [number, number]): number {
  const k = 111_320;
  const dx = (a[1] - b[1]) * k * Math.cos(((a[0] + b[0]) / 2) * (Math.PI / 180));
  const dy = (a[0] - b[0]) * k;
  return Math.hypot(dx, dy);
}
