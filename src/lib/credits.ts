// MyRoots © 2026 Jarek Hirsch, https://hirsch.fi
// SPDX-License-Identifier: GPL-3.0-or-later (with additional terms, see NOTICE.md)
// Author, licence and the components MyRoots is built with (shown in "About").
export const AUTHOR = 'Jarek Hirsch';
/** Copyright year: always the current one. */
export const YEAR = new Date().getFullYear();
export const LICENSE_NAME = 'GNU GPL 3.0 or later';
export const LICENSE_URL = 'https://www.gnu.org/licenses/gpl-3.0.html';
export const SOURCE_NOTE = 'GPL-3.0-or-later';
/** Where the source code is published (section 6 of the GPL). */
export const SOURCE_URL = 'https://github.com/jarekhirsz/myroots';

/** The author's website, tagged so visits from the app and the family websites can be told apart. */
export function authorUrl(medium: 'app' | 'family-website', content = 'credits'): string {
  const q = new URLSearchParams({ utm_source: 'myroots', utm_medium: medium, utm_campaign: 'myroots', utm_content: content });
  return `https://hirsch.fi/?${q}`;
}

export const THIRD_PARTY: { name: string; license: string; url: string }[] = [
  { name: 'Svelte', license: 'MIT', url: 'https://svelte.dev' },
  { name: 'Tauri', license: 'Apache-2.0 / MIT', url: 'https://tauri.app' },
  { name: 'serde, chrono, zip, base64, rusqlite (Rust)', license: 'MIT / Apache-2.0', url: 'https://crates.io' },
  { name: 'SQLite', license: 'Public domain', url: 'https://sqlite.org' },
  { name: 'family-chart', license: 'ISC', url: 'https://github.com/donatso/family-chart' },
  { name: 'D3', license: 'ISC', url: 'https://d3js.org' },
  { name: 'Leaflet, Leaflet.markercluster', license: 'BSD-2-Clause, MIT', url: 'https://leafletjs.com' },
  { name: 'Map data © OpenStreetMap contributors', license: 'ODbL', url: 'https://www.openstreetmap.org/copyright' },
  { name: 'face-api (face detection, TensorFlow.js)', license: 'MIT, Apache-2.0', url: 'https://github.com/vladmandic/face-api' },
  { name: 'DOMPurify', license: 'MPL-2.0 / Apache-2.0', url: 'https://github.com/cure53/DOMPurify' },
  { name: 'fflate', license: 'MIT', url: 'https://github.com/101arrowz/fflate' },
  { name: 'Lucide icons', license: 'ISC', url: 'https://lucide.dev' },
  { name: 'Literata, Onest fonts', license: 'SIL OFL 1.1', url: 'https://fontsource.org' },
];
