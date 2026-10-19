// Approximate historical regions, to show local events of history only to
// people who lived there. Borders are simplified (good enough to tell
// Pomerania from Galicia, not for maps). Points are [latitude, longitude].
import { t } from './i18n/index.svelte';

type Pt = [number, number];
type Shape = { poly: Pt[] } | { box: [number, number, number, number] } | { near: Pt; km: number };

// German Empire east of the Oder (Prussian partition, Silesia, Pomerania, East Prussia), 1815–1918
const PRUSSIAN: Pt[] = [
  [55.9, 21.2], [55.3, 22.95], [54.35, 22.8], [54.15, 22.55], [53.7, 22.35], [53.45, 21.85], [53.2, 20.6], [53.15, 19.6],
  [53.05, 19.15], [52.85, 18.75], [52.55, 18.45], [52.35, 17.95], [51.75, 17.95], [51.3, 18.2], [51.05, 18.5], [50.9, 18.9],
  [50.4, 19.05], [50.3, 19.08], [50.24, 19.17], [50.15, 19.2], [50.0, 18.6], [49.95, 18.3], [50.05, 17.75], [50.3, 17.35], [50.15, 16.85], [50.45, 16.2],
  [50.85, 15.0], [51.0, 14.0], [54.9, 14.0], [55.3, 19.5],
];
// Galicia (Austrian partition), 1772–1918
const GALICIA: Pt[] = [
  [49.8, 18.95], [50.0, 19.12], [50.2, 19.2], [50.27, 19.35], [50.22, 19.7], [50.12, 19.95], [50.25, 20.8], [50.62, 21.6], [50.75, 22.0], [50.45, 22.6], [50.4, 23.5], [50.45, 24.2],
  [50.2, 25.1], [49.8, 25.8], [49.3, 26.1], [48.6, 26.0], [48.25, 25.3], [48.15, 24.7], [48.6, 23.6], [49.05, 22.7], [49.2, 21.4],
  [49.35, 20.3], [49.45, 19.6], [49.6, 19.1],
];

function inPoly(p: Pt, poly: Pt[]): boolean {
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [yi, xi] = poly[i];
    const [yj, xj] = poly[j];
    if (yi > p[0] !== yj > p[0] && p[1] < ((xj - xi) * (p[0] - yi)) / (yj - yi) + xi) inside = !inside;
  }
  return inside;
}
export function km(a: Pt, b: Pt): number {
  const r = Math.PI / 180;
  const dLat = (b[0] - a[0]) * r;
  const dLng = (b[1] - a[1]) * r;
  const h = Math.sin(dLat / 2) ** 2 + Math.cos(a[0] * r) * Math.cos(b[0] * r) * Math.sin(dLng / 2) ** 2;
  return 12742 * Math.asin(Math.sqrt(h));
}
function inShape(p: Pt, s: Shape): boolean {
  if ('poly' in s) return inPoly(p, s.poly);
  if ('box' in s) return p[0] >= s.box[0] && p[0] <= s.box[1] && p[1] >= s.box[2] && p[1] <= s.box[3];
  return km(p, s.near) <= s.km;
}

// the lands of the old Commonwealth and today's Poland, roughly
const LANDS: Shape = { box: [47.5, 57.5, 13.8, 33.5] };
const AUSTRIAN_SILESIA: Shape = { box: [49.45, 50.05, 17.4, 19.0] };

export interface Region {
  label: () => string;
  test: (p: Pt) => boolean;
}
const shape = (label: () => string, ...shapes: Shape[]): Region => ({ label, test: (p) => shapes.some((s) => inShape(p, s)) });
const prussian = (p: Pt) => inPoly(p, PRUSSIAN);
const austrian = (p: Pt) => inPoly(p, GALICIA) || (inShape(p, AUSTRIAN_SILESIA) && !prussian(p));
const russian = (p: Pt) => inShape(p, LANDS) && !prussian(p) && !austrian(p);

export const REGIONS: Record<string, Region> = {
  prussian: { label: () => t('the Prussian partition'), test: prussian },
  austrian: { label: () => t('Galicia (the Austrian partition)'), test: austrian },
  russian: { label: () => t('the Russian partition'), test: russian },
  // the Kingdom of Poland (Congress Poland)
  congress: {
    label: () => t('the Kingdom of Poland (Congress Poland)'),
    // without the Białystok region (part of the Russian Empire itself)
    test: (p) => russian(p) && inShape(p, { box: [50.2, 54.5, 17.8, 24.2] }) && !(p[0] > 52.6 && p[0] < 53.9 && p[1] > 22.85),
  },
  // the Duchy of Warsaw: Congress Poland and Greater Poland with Bydgoszcz
  duchy: { label: () => t('the Duchy of Warsaw'), test: (p) => REGIONS.congress.test(p) || inShape(p, { box: [51.4, 53.3, 15.9, 18.9] }) },
  greaterpoland: shape(() => t('Greater Poland'), { box: [51.4, 53.2, 15.8, 18.6] }),
  silesia: shape(() => t('Upper Silesia'), { box: [49.9, 50.95, 17.8, 19.45] }),
  pomerania: shape(() => t('Pomerania'), { box: [53.3, 54.9, 16.3, 19.7] }),
  southeast: shape(() => t('Podolia and Red Ruthenia'), { box: [48.0, 51.0, 22.5, 30.0] }),
  volhynia: shape(() => t('Volhynia and Eastern Galicia'), { box: [48.6, 51.95, 23.4, 28.0] }),
  sepoland: shape(() => t('south-eastern Poland'), { box: [49.0, 51.0, 21.0, 24.2] }),
  zamosc: shape(() => t('the Zamość region'), { box: [50.3, 51.0, 22.6, 23.9] }),
  cop: shape(() => t('the Central Industrial Region'), { box: [49.8, 51.4, 20.6, 22.9] }),
  zaolzie: shape(() => t('Cieszyn Silesia'), { box: [49.55, 49.95, 18.3, 19.0] }),
  southwest: shape(() => t('south-western Poland'), { box: [49.9, 52.2, 14.5, 18.8] }),
  warsaw: shape(() => t('Warsaw and around'), { near: [52.23, 21.01], km: 45 }),
  wroclaw: shape(() => t('Wrocław and around'), { near: [51.11, 17.03], km: 45 }),
};

/** Is the person (any of their places) in the region? */
export function within(region: string, points: Pt[]): boolean {
  const r = REGIONS[region];
  return !!r && points.some((p) => r.test(p));
}
