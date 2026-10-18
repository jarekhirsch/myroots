<script lang="ts">
  // Map of family places (births, marriages, deaths, events). OpenStreetMap is opt-in.
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { backend } from '../lib/backend';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { fmtDate, initials } from '../lib/format';
  import { TILES, geocodeMissing, searchPlaces, type GeoProgress } from '../lib/geocode';
  import MapPinned from '@lucide/svelte/icons/map-pin-plus';
  import SearchIcon from '@lucide/svelte/icons/search';
  import X from '@lucide/svelte/icons/x';
  import type { PlaceUse } from '../lib/types';
  import { toast } from '../lib/ui.svelte';
  import NotFound from './NotFound.svelte';
  import BackButton from '../lib/components/BackButton.svelte';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import { go } from '../lib/router.svelte';
  import 'leaflet/dist/leaflet.css';
  import 'leaflet.markercluster/dist/MarkerCluster.css';
  import type { MapData, MapPlace, ClanInfo, Summary } from '../lib/types';
  import { untrack } from 'svelte';

  let { person = null }: { person?: number | null } = $props();
  const viewer = $derived(!!app.info?.viewer);
  const enabled = $derived(!!app.info?.map_external);

  let el: HTMLDivElement | undefined = $state();
  let map: any = null;
  let L: any = null;
  let cluster: any = null;
  let data = $state<MapData | null>(null);
  let error = $state<unknown>(null);
  let kinds = $state<Record<string, boolean>>({ birth: true, marriage: true, death: true, event: true });
  let clan = $state('');
  let clans = $state<ClanInfo[]>([]);
  let geo = $state<GeoProgress | null>(null);
  let cancelled = false;
  let personName = $state('');

  const KIND_LABEL: Record<string, () => string> = {
    birth: () => t('Births'),
    marriage: () => t('Marriages'),
    death: () => t('Deaths'),
    event: () => t('Events'),
  };
  const KIND_COLOR: Record<string, string> = { birth: 'var(--ev-birth)', marriage: 'var(--ev-marriage)', death: 'var(--ev-death)', event: 'var(--ev-event)' };

  async function load() {
    try {
      data = await call<MapData>('map.data', person ? { person } : {});
      if (!viewer) clans = await call<ClanInfo[]>('clans.list');
      if (person) personName = (await call<Summary[]>('people.summaries')).find((p) => p.id === person)?.name || '';
    } catch (e) {
      error = e;
    }
  }

  $effect(() => {
    void person;
    untrack(() => load());
  });

  function esc(s: string) {
    return String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!);
  }

  function evText(e: MapPlace['events'][number]): string {
    const what = e.kind === 'birth' ? t('Birth') : e.kind === 'death' ? t('Death') : e.kind === 'marriage' ? t('Marriage to %s', e.label || '?') : e.label || t('Event');
    return `${what}${e.date ? ' · ' + fmtDate(e.date) : ''}`;
  }

  function popup(p: MapPlace, evs: MapPlace['events']): string {
    const rows = evs
      .slice(0, 40)
      .map((e) => {
        const src = e.photo ? backend.mediaUrl(e.photo, true) : '';
        const av = src ? `<img src="${esc(src)}" alt="">` : `<span>${esc(initials(e.name))}</span>`;
        return `<a class="mp-row" href="#/person/${e.person}"><span class="mp-av">${av}</span><span><b>${esc(e.name)}</b><br><small>${esc(evText(e))}</small></span></a>`;
      })
      .join('');
    const more = evs.length > 40 ? `<div class="mp-more">+${evs.length - 40}</div>` : '';
    return `<div class="mp"><div class="mp-h">${esc(p.place)}</div>${rows}${more}</div>`;
  }

  function filtered(p: MapPlace) {
    return p.events.filter((e) => kinds[e.kind] !== false && (!clan || e.clans.includes(clan)));
  }

  async function render() {
    if (!el || !data || !enabled) return;
    if (!L) {
      L = (await import('leaflet')).default;
      (window as any).L = L;
      await import('leaflet.markercluster');
    }
    if (!map) {
      map = L.map(el, { zoomControl: false, worldCopyJump: true }).setView([52, 19], 4);
      L.control.zoom({ position: 'bottomright', zoomInTitle: t('Zoom in'), zoomOutTitle: t('Zoom out') }).addTo(map);
      const style = TILES[app.settings?.appearance?.map_style || 'osm'] || TILES.osm;
      L.tileLayer(style.url, { attribution: style.attribution, subdomains: style.sub || 'abc', maxZoom: style.max || 19 }).addTo(map);
      // placing a missing place by hand: click on the map
      map.on('click', (e: any) => {
        if (picking) saveCoords(picking, e.latlng.lat, e.latlng.lng, 'manual');
      });
    }
    if (cluster) cluster.remove();
    cluster = (L as any).markerClusterGroup({
      showCoverageOnHover: false,
      maxClusterRadius: 44,
      // the same round pins as single places, a little bigger, with the number of places inside
      iconCreateFunction: (c: any) => {
        const n = c.getChildCount();
        const size = n < 10 ? 32 : n < 100 ? 38 : 44;
        return L.divIcon({ className: 'mr-pin mr-cluster', html: `<span style="--c:var(--ink);--s:${size}px">${n}</span>`, iconSize: [size, size], iconAnchor: [size / 2, size / 2] });
      },
    });
    const ids = new Set<number>();
    const bounds: [number, number][] = [];
    for (const p of data.geocoded) {
      const evs = filtered(p);
      if (!evs.length || p.lat == null || p.lng == null) continue;
      evs.forEach((e) => e.photo && ids.add(e.photo));
      const main = evs[0].kind;
      const icon = L.divIcon({ className: 'mr-pin', html: `<span style="--c:${KIND_COLOR[main] || 'var(--ink)'}">${evs.length > 1 ? evs.length : ''}</span>`, iconSize: [26, 26], iconAnchor: [13, 13] });
      const m = L.marker([p.lat, p.lng], { icon, title: p.place });
      m.bindPopup(() => popup(p, evs), { maxWidth: 300, maxHeight: 320 });
      cluster.addLayer(m);
      bounds.push([p.lat, p.lng]);
    }
    await backend.ensureMedia?.([...ids]);
    map.addLayer(cluster);
    if (focusOn) {
      map.setView(focusOn, Math.max(map.getZoom(), 9));
      focusOn = null;
    } else if (bounds.length) map.fitBounds(bounds, { padding: [40, 40], maxZoom: person ? 9 : 7 });
    setTimeout(() => map?.invalidateSize(), 50);
  }

  $effect(() => {
    void data;
    void clan;
    void kinds.birth, kinds.death, kinds.marriage, kinds.event;
    void enabled;
    render();
  });

  $effect(() => {
    return () => {
      cancelled = true;
      map?.remove();
      map = null;
    };
  });

  async function enable() {
    if (!app.settings) return;
    await call('settings.save', { settings: { ...app.settings, map_external: true } });
    await refreshInfo();
  }

  async function geocode() {
    cancelled = false;
    geo = { done: 0, total: 0, found: 0, failed: 0 };
    const r = await geocodeMissing((p) => (geo = p), () => cancelled);
    toast(t('Geocoded: %1$d, not recognized: %2$d, remaining: %3$d.', r.found, r.failed, r.total - r.done));
    geo = null;
    await load();
  }

  const pending = $derived(data ? data.uncached.length : 0);

  /* ---------- places without coordinates: search by hand or point on the map ---------- */
  let fixOpen = $state(false);
  let picking = $state<string | null>(null);
  let focusOn: [number, number] | null = null;
  let origins = $state<Record<string, string>>({});
  let queries = $state<Record<string, string>>({});
  let found = $state<Record<string, { label: string; lat: number; lng: number }[] | 'none' | 'busy'>>({});
  const untried = $derived(data ? data.uncached.filter((p) => origins[p.place] !== 'failed').length : 0);

  $effect(() => {
    if (!fixOpen || viewer) return;
    call<PlaceUse[]>('places.list').then((v) => (origins = Object.fromEntries(v.map((p) => [p.name, p.origin]))));
  });

  async function search(name: string) {
    const q = (queries[name] ?? name).trim();
    if (!q) return;
    found[name] = 'busy';
    try {
      const r = await searchPlaces(q);
      found[name] = r.length ? r : 'none';
    } catch {
      found[name] = 'none';
      toast(t('The map search service did not answer. Try again in a moment.'), 'error');
    }
  }

  async function saveCoords(name: string, lat: number, lng: number, origin: string) {
    await call('places.set', { name, lat, lng, origin });
    picking = null;
    delete found[name];
    focusOn = [lat, lng];
    toast(t('Saved: %s', name));
    await load();
  }

  function who(p: MapPlace): string {
    const names = [...new Set(p.events.map((e) => e.name))];
    return names.slice(0, 3).join(', ') + (names.length > 3 ? ` +${names.length - 3}` : '');
  }
  const shownCount = $derived(data ? data.geocoded.filter((p) => filtered(p).length).length : 0);
</script>

{#if error}
  <NotFound private={(error as any)?.code === 'private'} />
{:else if !enabled}
  <div class="page">
    <BackButton />
    <div class="page-head"><h1>{t('Map')}</h1></div>
    <div class="panel off">
      <h2>{t('The map is turned off')}</h2>
      {#if viewer}
        <p>{t('This family website does not use online maps.')}</p>
      {:else}
        <p>{t('The map uses OpenStreetMap: map tiles are loaded from its servers and place names are sent to its search service (Nominatim) to find coordinates. Nothing else leaves your device.')}</p>
        <button class="btn primary" onclick={enable}>{t('Turn on the map')}</button>
        <p class="hint">{t('You can also enter coordinates by hand in Places.')}</p>
      {/if}
    </div>
  </div>
{:else}
  <div class="map-page">
    <!-- floating toolbar in the style of the tree -->
    <div class="bar no-print">
      <BackButton pill />
      <div class="kinds" role="group" aria-label={t('Show')}>
        {#each Object.keys(KIND_LABEL) as k (k)}
          <button aria-pressed={kinds[k]} style="--c:{KIND_COLOR[k]}" onclick={() => (kinds[k] = !kinds[k])}><i aria-hidden="true"></i>{KIND_LABEL[k]()}</button>
        {/each}
      </div>
      {#if !viewer || app.info?.sections?.list !== false}
        <div class="pick">
          <PersonPicker value={person} placeholder={t('Places of one person…')} onchange={(v) => go(v ? `/map/${v}` : '/map')} />
        </div>
      {/if}
      {#if clans.length && !person}
        <select bind:value={clan} aria-label={t('Clan')}>
          <option value="">{t('All clans')}</option>
          {#each clans as c (c.name)}<option value={c.name}>{c.name}</option>{/each}
        </select>
      {/if}
      <div class="tools">
        {#if !viewer && pending}
          <div class="geo">
            {#if geo}
              <span>{t('Finding places on the map… %1$d of %2$d', geo.done, geo.total)}</span>
              <button class="btn sm ghost" onclick={() => (cancelled = true)}>{t('Stop')}</button>
            {:else}
              <span>{tn('%d place has no coordinates yet.', '%d places have no coordinates yet.', pending)}</span>
              <button class="btn sm" onclick={() => (fixOpen = !fixOpen)} aria-expanded={fixOpen}>{t('Correct')}</button>
              {#if untried}<button class="btn sm primary" onclick={geocode}>{t('Find on the map')}</button>{/if}
            {/if}
          </div>
        {/if}
        {#if data}<span class="count">{tn('%d place', '%d places', shownCount)}</span>{/if}
      </div>
    </div>
    <div class="map" class:picking bind:this={el}></div>

    {#if picking}
      <div class="pick-banner no-print" role="status">
        <MapPinned size={17} />
        <span>{t('Click on the map where %s is.', picking)}</span>
        <button class="btn sm ghost" onclick={() => (picking = null)}>{t('Cancel')}</button>
      </div>
    {/if}

    {#if fixOpen && !viewer && data}
      <aside class="fix no-print" aria-label={t('Places without coordinates')}>
        <header>
          <h2>{t('Places without coordinates')} <span class="muted">{data.uncached.length}</span></h2>
          <button class="btn ghost icon sm" onclick={() => (fixOpen = false)} aria-label={t('Close')}><X size={16} /></button>
        </header>
        <p class="small muted">{t('Search with a different name (e.g. with the region or the old name) or point the place on the map.')}</p>
        {#each data.uncached as p (p.place)}
          {@const f = found[p.place]}
          <div class="fx">
            <div class="fx-h">
              <b>{p.place}</b>
              {#if origins[p.place] === 'failed'}<span class="tag">{t('not found')}</span>{/if}
            </div>
            <div class="small muted">{who(p)}</div>
            <form class="fx-q" onsubmit={(e) => { e.preventDefault(); search(p.place); }}>
              <input type="search" value={queries[p.place] ?? p.place} oninput={(e) => (queries[p.place] = (e.target as HTMLInputElement).value)} aria-label={t('Search on the map')} />
              <button class="btn sm" type="submit" disabled={f === 'busy'} aria-label={t('Search on the map')}><SearchIcon size={15} /></button>
              <button class="btn sm ghost" type="button" onclick={() => (picking = p.place)} title={t('Point on the map')}><MapPinned size={15} /> {t('Point on the map')}</button>
            </form>
            {#if f === 'none'}
              <p class="small muted">{t('Nothing found. Try another spelling or point the place on the map.')}</p>
            {:else if Array.isArray(f)}
              <ul class="res">
                {#each f as r, i (i)}
                  <li><button type="button" onclick={() => saveCoords(p.place, r.lat, r.lng, 'nominatim')}>{r.label}</button></li>
                {/each}
              </ul>
            {/if}
          </div>
        {:else}
          <p class="small muted">{t('All places are on the map.')}</p>
        {/each}
        <a class="small" href="#/places">{t('All places')}</a>
      </aside>
    {/if}
  </div>
{/if}

<style>
  .map-page {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  /* the toolbar floats over the map, like the one of the tree */
  .bar {
    position: absolute;
    inset: 0 0 auto 0;
    z-index: 1000;
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    padding: 14px 20px;
    pointer-events: none;
  }
  .bar > :global(*) {
    pointer-events: auto;
  }
  .kinds {
    display: flex;
    gap: 2px;
    padding: 3px;
    border-radius: 999px;
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  .kinds button {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    border: 0;
    background: none;
    font: inherit;
    font-size: 0.9rem;
    font-weight: 540;
    color: var(--muted);
    padding: 7px 14px 7px 11px;
    border-radius: 999px;
    cursor: pointer;
    white-space: nowrap;
  }
  .kinds button i {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    box-shadow: inset 0 0 0 2px var(--c);
  }
  .kinds button[aria-pressed='true'] {
    color: var(--ink);
    font-weight: 640;
    background: color-mix(in srgb, var(--c) 16%, var(--surface));
  }
  .kinds button[aria-pressed='true'] i {
    background: var(--c);
  }
  .kinds button:focus-visible {
    outline: 2px solid var(--ink);
    outline-offset: 1px;
  }
  .pick {
    flex: 1 1 220px;
    max-width: 340px;
  }
  .bar :global(input),
  .bar select {
    background-color: var(--surface);
    box-shadow: var(--shadow);
    border-radius: 999px;
  }
  .bar select {
    width: auto;
    min-height: 38px;
    padding: 6px 30px 6px 14px;
  }
  .tools {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    margin-inline-start: auto;
  }
  .count,
  .geo {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    background: var(--surface);
    box-shadow: var(--shadow);
    border-radius: 999px;
    font-size: 0.85rem;
    color: var(--muted);
    padding: 8px 14px;
  }
  .geo {
    padding: 4px 4px 4px 14px;
    color: var(--ink);
  }
  .off {
    max-width: 640px;
  }
  .map {
    flex: 1;
    min-height: 300px;
    z-index: 0;
  }
  .map :global(.mr-pin span) {
    display: flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    width: var(--s, 26px);
    height: var(--s, 26px);
    border-radius: 50%;
    background: var(--c);
    border: 3px solid #fff;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.35);
    color: #fff;
    font-size: 0.72rem;
    font-weight: 700;
  }
  .map :global(.mr-cluster span) {
    font-size: 0.8rem;
    box-shadow:
      0 0 0 4px color-mix(in srgb, var(--ink) 22%, transparent),
      0 1px 4px rgba(0, 0, 0, 0.35);
  }
  .map :global(.mp-h) {
    font-weight: 600;
    margin-bottom: 6px;
    font-size: 0.95rem;
  }
  .map :global(.mp-row) {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 4px 0;
    text-decoration: none;
    color: #1d2833;
    border-top: 1px solid #eef1f4;
  }
  .map :global(.mp-av) {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    overflow: hidden;
    background: #e8edf2;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.7rem;
    font-weight: 600;
  }
  .map :global(.mp-av img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  @media (max-width: 900px) {
    .map-page {
      height: calc(100vh - var(--topbar) - var(--tabbar) - env(safe-area-inset-bottom) - env(safe-area-inset-top));
    }
    .bar {
      padding: 8px 10px;
    }
    .pick {
      flex-basis: 100%;
      max-width: none;
      order: 3;
    }
    .count {
      display: none;
    }
  }
  .map.picking {
    cursor: crosshair;
  }
  .map.picking :global(.leaflet-grab) {
    cursor: crosshair;
  }
  .pick-banner {
    position: absolute;
    left: 50%;
    bottom: 28px;
    transform: translateX(-50%);
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--ink);
    color: var(--surface);
    border-radius: 999px;
    padding: 6px 6px 6px 16px;
    box-shadow: var(--shadow);
    font-size: 0.9rem;
  }
  .pick-banner .btn {
    color: var(--surface);
  }
  .fix {
    position: absolute;
    top: 70px;
    right: 18px;
    bottom: 18px;
    width: 360px;
    z-index: 1000;
    background: var(--surface);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow);
    padding: 14px 16px;
    overflow: auto;
  }
  .fix header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .fix h2 {
    font-size: 1rem;
    margin: 0;
  }
  .fx {
    border-top: 1px solid var(--line);
    padding: 10px 0;
  }
  .fx-h {
    display: flex;
    gap: 8px;
    align-items: baseline;
    flex-wrap: wrap;
  }
  .fx-q {
    display: flex;
    gap: 6px;
    margin-top: 6px;
    flex-wrap: wrap;
  }
  .fx-q input {
    flex: 1 1 150px;
    min-width: 0;
  }
  .res {
    list-style: none;
    margin: 6px 0 0;
    padding: 0;
  }
  .res button {
    display: block;
    width: 100%;
    text-align: start;
    border: 0;
    background: none;
    font: inherit;
    font-size: 0.85rem;
    padding: 6px 8px;
    border-radius: 8px;
    cursor: pointer;
    color: var(--ink);
  }
  .res button:hover,
  .res button:focus-visible {
    background: color-mix(in srgb, var(--ink) 8%, transparent);
  }
  @media (max-width: 640px) {
    .fix {
      left: 10px;
      right: 10px;
      width: auto;
      top: auto;
      max-height: 60%;
    }
  }
</style>
