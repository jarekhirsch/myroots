// Map background for every map in the app (map view, graves, place picker,
// mini maps). Vector maps from OpenFreeMap: free, no key, no request limits,
// drawn with MapLibre inside Leaflet so markers and clusters stay the same.
// Labels follow the app language (Wiedeń, Lwów for a Polish user) and fall
// back to the local name. Without WebGL the CARTO raster map is used instead.
import type * as Leaflet from 'leaflet';
import { lang } from './i18n/index.svelte';

const OFM = 'https://tiles.openfreemap.org/styles/';
const OFM_ATTR =
  '<a href="https://openfreemap.org" target="_blank" rel="noopener">OpenFreeMap</a> &copy; <a href="https://www.openmaptiles.org/" target="_blank" rel="noopener">OpenMapTiles</a> &copy; <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a>';
const OSM_ATTR = '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>';

type Raster = { url: string; attribution: string; sub?: string; max?: number };

// Settings keep the old keys: osm = colourful, light, dark, topo.
const VECTOR: Record<string, string> = { osm: 'liberty', light: 'positron', dark: 'dark' };
const RASTER: Record<string, Raster> = {
  topo: { url: 'https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png', attribution: `${OSM_ATTR}, <a href="https://opentopomap.org">OpenTopoMap</a> (CC-BY-SA)`, sub: 'abc', max: 17 },
  light: { url: 'https://{s}.basemaps.cartocdn.com/rastertiles/voyager/{z}/{x}/{y}{r}.png', attribution: `${OSM_ATTR} &copy; <a href="https://carto.com/attributions">CARTO</a>`, sub: 'abcd', max: 19 },
  dark: { url: 'https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png', attribution: `${OSM_ATTR} &copy; <a href="https://carto.com/attributions">CARTO</a>`, sub: 'abcd', max: 19 },
};

export const MAP_STYLES = ['osm', 'light', 'dark', 'topo'];

let webgl: boolean | null = null;
function hasWebGL(): boolean {
  if (webgl === null) {
    try {
      const c = document.createElement('canvas');
      webgl = !!(c.getContext('webgl2') || c.getContext('webgl'));
    } catch {
      webgl = false;
    }
  }
  return webgl;
}

const styles = new Map<string, Promise<any>>();

/** Style JSON with labels in the app language. */
function localStyle(name: string, code: string): Promise<any> {
  const key = `${name}:${code}`;
  if (!styles.has(key)) {
    const p = fetch(OFM + name)
      .then((r) => {
        if (!r.ok) throw new Error(String(r.status));
        return r.json();
      })
      .then((st) => {
        // Every name label: the name in the app language, else the Latin
        // spelling, else the local name (no second line in another script).
        const named = (v: any): boolean => Array.isArray(v) && (v[1] === 'name_en' || v.some(named));
        const label = ['coalesce', ['get', `name:${code}`], ['get', 'name:latin'], ['get', 'name']];
        for (const l of st.layers || []) if (named(l.layout?.['text-field'])) l.layout['text-field'] = label;
        return st;
      });
    p.catch(() => styles.delete(key));
    styles.set(key, p);
  }
  return styles.get(key)!;
}

function raster(L: typeof Leaflet, map: Leaflet.Map, r: Raster): Leaflet.Layer {
  return L.tileLayer(r.url, { attribution: r.attribution, subdomains: r.sub || 'abc', maxZoom: r.max || 19 }).addTo(map);
}

/** Puts the chosen background under a Leaflet map. Returns the layer. */
export async function addBaseMap(L: typeof Leaflet, map: Leaflet.Map, style: string | undefined): Promise<Leaflet.Layer> {
  const key = style && (VECTOR[style] || RASTER[style]) ? style : 'osm';
  if (map.getMaxZoom() === Infinity) map.setMaxZoom(19); // clusters need a limit
  const name = VECTOR[key];
  if (!name || !hasWebGL()) return raster(L, map, RASTER[key] || RASTER.light);
  try {
    const [{ maplibreGL }, st] = await Promise.all([
      import('./maplibre'),
      localStyle(name, (lang() || 'en').slice(0, 2)),
    ]);
    const layer = (maplibreGL as any)({ style: st, attributionControl: false }) as Leaflet.Layer;
    layer.getAttribution = () => OFM_ATTR;
    layer.addTo(map);
    return layer;
  } catch {
    return raster(L, map, RASTER[key] || RASTER.light);
  }
}
