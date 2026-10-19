// Historical weather of a day at a place: Open-Meteo, ERA5 reanalysis
// (daily values from 1940, about a week behind today). Opt-in in the settings;
// only the coordinates of the place and the date are sent. A fetched day is
// kept with the place, so it is asked for once and reaches the family website.

import { call } from './api.svelte';
import { backend } from './backend';
import { t } from './i18n/index.svelte';
import type { DayWeather, PlaceUse } from './types';

export const WEATHER_SOURCE = 'https://open-meteo.com/';
export const FIRST_DAY = '1940-01-01';

let index: Promise<Map<string, PlaceUse>> | null = null;

/** Forget the known places (after coordinates were changed). */
export function refreshPlaces() {
  index = null;
}

function places(): Promise<Map<string, PlaceUse>> {
  if (!index) {
    index = call<PlaceUse[]>('places.list')
      .then((l) => new Map(l.map((p) => [p.name.trim().toLowerCase(), p])))
      .catch(() => {
        index = null;
        return new Map();
      });
  }
  return index;
}

/** "YYYY-MM-DD" when the date is an exact day, otherwise null. */
export function exactDay(v: string | null | undefined): string | null {
  const s = (v || '').trim();
  return /^\d{4}-\d{2}-\d{2}$/.test(s) ? s : null;
}

/** Days the archive covers. */
export function inArchive(day: string): boolean {
  const last = new Date(Date.now() - 7 * 864e5).toISOString().slice(0, 10);
  return day >= FIRST_DAY && day <= last;
}

// a few requests at a time
let active = 0;
const waiting: (() => void)[] = [];
async function slot<T>(f: () => Promise<T>): Promise<T> {
  if (active >= 3) await new Promise<void>((r) => waiting.push(r));
  active++;
  try {
    return await f();
  } finally {
    active--;
    waiting.shift()?.();
  }
}

const inflight = new Map<string, Promise<DayWeather | null>>();

async function download(lat: number, lng: number, day: string): Promise<DayWeather | null> {
  const url =
    'https://archive-api.open-meteo.com/v1/archive' +
    `?latitude=${lat.toFixed(3)}&longitude=${lng.toFixed(3)}&start_date=${day}&end_date=${day}` +
    '&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_sum,snowfall_sum&timezone=auto';
  const r = await backend.fetchBytes(url, { Accept: 'application/json' });
  if (r.status !== 200) return null;
  const d = JSON.parse(new TextDecoder().decode(r.bytes))?.daily;
  const at = (k: string): number | null => {
    const v = d?.[k]?.[0];
    return typeof v === 'number' && Number.isFinite(v) ? v : null;
  };
  const code = at('weather_code');
  if (code == null) return null;
  return { code, tmax: at('temperature_2m_max'), tmin: at('temperature_2m_min'), rain: at('precipitation_sum'), snow: at('snowfall_sum') };
}

/**
 * Weather of the day of an event at its place: the stored values, or (when
 * `fetchMissing`) downloaded and stored. Null when unknown.
 */
export async function weatherFor(place: string, date: string, fetchMissing: boolean): Promise<DayWeather | null> {
  const day = exactDay(date);
  const name = place.trim();
  if (!day || !name) return null;
  const pl = (await places()).get(name.toLowerCase());
  if (!pl) return null;
  const stored = pl.weather?.[day];
  if (stored) return stored;
  if (!fetchMissing || pl.lat == null || pl.lng == null || !inArchive(day)) return null;
  const key = `${pl.name.toLowerCase()}|${day}`;
  let job = inflight.get(key);
  if (!job) {
    job = slot(async () => {
      const w = await download(pl.lat!, pl.lng!, day);
      if (w) {
        await call('places.weather', { name: pl.name, date: day, weather: w }).catch(() => {});
        pl.weather = { ...(pl.weather || {}), [day]: w };
      }
      return w;
    }).catch(() => null);
    inflight.set(key, job);
    job.finally(() => inflight.delete(key));
  }
  return job;
}

export type WeatherKind = 'clear' | 'fair' | 'cloudy' | 'fog' | 'drizzle' | 'rain' | 'snow' | 'storm';

/** Group of a WMO weather code. */
export function weatherKind(code: number): WeatherKind {
  if (code === 0) return 'clear';
  if (code <= 2) return 'fair';
  if (code === 3) return 'cloudy';
  if (code === 45 || code === 48) return 'fog';
  if (code >= 51 && code <= 57) return 'drizzle';
  if ((code >= 61 && code <= 67) || (code >= 80 && code <= 82)) return 'rain';
  if ((code >= 71 && code <= 77) || code === 85 || code === 86) return 'snow';
  if (code >= 95) return 'storm';
  return 'cloudy';
}

export function weatherLabel(code: number): string {
  switch (code) {
    case 0:
      return t('Clear sky');
    case 1:
      return t('Mainly clear');
    case 2:
      return t('Partly cloudy');
    case 3:
      return t('Overcast');
    case 45:
    case 48:
      return t('Fog');
    case 56:
    case 57:
      return t('Freezing drizzle');
    case 66:
    case 67:
      return t('Freezing rain');
    case 80:
    case 81:
    case 82:
      return t('Rain showers');
    case 85:
    case 86:
      return t('Snow showers');
    case 96:
    case 99:
      return t('Thunderstorm with hail');
  }
  switch (weatherKind(code)) {
    case 'drizzle':
      return t('Drizzle');
    case 'rain':
      return t('Rain');
    case 'snow':
      return t('Snow');
    case 'storm':
      return t('Thunderstorm');
    default:
      return t('Overcast');
  }
}
