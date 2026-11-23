<script lang="ts">
  import { addBaseMap } from '../lib/basemap';
  // Map of family places (births, marriages, deaths, events). OpenStreetMap is opt-in.
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { backend } from '../lib/backend';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { fmtDate, initials } from '../lib/format';
  import { geocodeMissing, searchPlaces, type GeoProgress } from '../lib/geocode';
  import MapPinned from '@lucide/svelte/icons/map-pin-plus';
  import SearchIcon from '@lucide/svelte/icons/search';
  import X from '@lucide/svelte/icons/x';
  import Waypoints from '@lucide/svelte/icons/waypoints';
  import Play from '@lucide/svelte/icons/play';
  import Pause from '@lucide/svelte/icons/pause';
  import { buildMigrations, homePlaces, arc, type Move, type Birth } from '../lib/migrations';
  import { osmCemeteries, metres, type OsmCemetery } from '../lib/cemeteries';
  import Cross from '@lucide/svelte/icons/cross';
  import type { MapCemetery } from '../lib/types';
  import type { PlaceUse, TreeNode } from '../lib/types';
  import { toast } from '../lib/ui.svelte';
  import NotFound from './NotFound.svelte';
  import BackButton from '../lib/components/BackButton.svelte';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import { go, route } from '../lib/router.svelte';
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
  let data = $state.raw<MapData | null>(null);
  let error = $state<unknown>(null);
  let kinds = $state<Record<string, boolean>>({ birth: true, marriage: true, death: true, event: true, burial: true });
  let clan = $state(route.query.clan || '');
  // people buried at one cemetery: their graves and (with the other kinds) where they came from
  let cemetery = $state('');
  let clans = $state<ClanInfo[]>([]);
  let geo = $state<GeoProgress | null>(null);
  let cancelled = false;
  let personName = $state('');

  const KIND_LABEL: Record<string, () => string> = {
    birth: () => t('Births'),
    marriage: () => t('Marriages'),
    death: () => t('Deaths'),
    event: () => t('Events'),
    burial: () => t('Graves'),
  };
  const KIND_COLOR: Record<string, string> = { birth: 'var(--ev-birth)', marriage: 'var(--ev-marriage)', death: 'var(--ev-death)', event: 'var(--ev-event)', burial: 'var(--ev-burial)' };
  const cems = $derived(data?.cemeteries || []);
  const cem = $derived(cemetery ? cems.find((c) => c.name === cemetery) || null : null);
  const buried = $derived(cem ? new Set(cem.persons) : null);

  async function load() {
    try {
      migSrc = null;
      if (mig) await loadMig();
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
    const what =
      e.kind === 'birth' ? t('Birth') : e.kind === 'death' ? t('Death') : e.kind === 'marriage' ? t('Marriage to %s', e.label || '?') : e.kind === 'burial' ? [t('Grave'), e.label].filter(Boolean).join(': ') : e.label || t('Event');
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
    return p.events.filter((e) => kinds[e.kind] !== false && (!clan || e.clans.includes(clan)) && (!buried || buried.has(e.person)));
  }
  function pickCemetery(name: string) {
    cemetery = name;
    // choosing a cemetery shows its graves even when graves were hidden
    if (name) kinds.burial = true;
  }

  /* ---------- migrations: moves of the family over the years ---------- */
  let mig = $state(route.query.mig === '1' || !!route.query.from);
  // migrations from one place (a family house, a village): its people and their descendants
  let home = $state(route.query.from || '');
  let homes = $state.raw<{ name: string; count: number }[]>([]);
  let migSrc: { nodes: TreeNode[]; pl: PlaceUse[] } | null = null;
  let moves = $state<Move[]>([]);
  let births = $state<Birth[]>([]);
  let yearNow = $state(0);
  let playing = $state(false);
  let migLayer: any = null;
  let migItems: { year: number; obj: any; line: boolean }[] = [];
  const years = $derived.by(() => {
    const all = [...moves.map((m) => m.year), ...births.map((b) => b.year)];
    return all.length ? [Math.min(...all), Math.max(...all)] : [0, 0];
  });
  async function loadMig() {
    if (!migSrc) {
      const [tree, pl] = await Promise.all([call<{ nodes: TreeNode[] }>('tree.data'), call<PlaceUse[]>('places.list')]);
      migSrc = { nodes: tree.nodes, pl };
      homes = homePlaces(tree.nodes, pl);
    }
    const r = buildMigrations(migSrc.nodes, migSrc.pl, person, home);
    moves = r.moves;
    births = r.births;
    const all = [...moves.map((m) => m.year), ...births.map((b) => b.year)];
    yearNow = all.length ? Math.max(...all) : 0;
  }
  function drawMig() {
    migLayer?.remove();
    migItems = [];
    if (!map || !L || !mig) return;
    migLayer = L.layerGroup();
    for (const b of births) {
      const c = L.circleMarker(b.at, { radius: 4, className: 'mig-dot', weight: 1 }).bindTooltip(`${esc(b.name)}, ${b.year}: ${esc(b.place)}`);
      migItems.push({ year: b.year, obj: c, line: false });
    }
    const bounds: [number, number][] = [];
    for (const m of moves) {
      const pl = L.polyline(arc(m.from, m.to), { className: `mig mig-${m.kind}`, weight: 2.5, opacity: 0.9 });
      pl.bindTooltip(`<b>${esc(m.name)}</b>, ${m.year}<br>${esc(m.fromName)} → ${esc(m.toName)}`, { sticky: true });
      const head = L.circleMarker(m.to, { radius: 3.5, className: `mig-head mig-${m.kind}`, weight: 0, fillOpacity: 1 });
      migItems.push({ year: m.year, obj: pl, line: true }, { year: m.year, obj: head, line: false });
      bounds.push(m.from, m.to);
    }
    for (const b of births) bounds.push(b.at);
    migLayer.addTo(map);
    applyYear();
    if (bounds.length) map.fitBounds(bounds, { padding: [60, 60], maxZoom: 8 });
  }
  function applyYear() {
    if (!migLayer) return;
    for (const it of migItems) {
      const on = it.year <= yearNow;
      if (on && !migLayer.hasLayer(it.obj)) migLayer.addLayer(it.obj);
      else if (!on && migLayer.hasLayer(it.obj)) migLayer.removeLayer(it.obj);
      if (on && it.line) it.obj.getElement()?.classList.toggle('fresh', yearNow - it.year < 4);
    }
  }
  $effect(() => {
    void yearNow;
    applyYear();
  });
  let timer = 0;
  function play() {
    if (playing) {
      playing = false;
      clearInterval(timer);
      return;
    }
    if (yearNow >= years[1]) yearNow = years[0];
    playing = true;
    timer = window.setInterval(() => {
      if (yearNow >= years[1]) {
        playing = false;
        clearInterval(timer);
      } else yearNow++;
    }, 90);
  }
  async function toggleMig() {
    mig = !mig;
    playing = false;
    clearInterval(timer);
    if (mig) await loadMig();
    render();
  }
  async function setHome(v: string) {
    home = v;
    playing = false;
    clearInterval(timer);
    await loadMig();
    render();
  }
  const recent = $derived(moves.filter((m) => m.year <= yearNow).slice(-3).reverse());

  /* ---------- cemeteries: the family's, saved by the user, and OpenStreetMap ---------- */
  let cemOn = $state(route.query.cem === '1');
  let cemLayer: any = null;
  let osmLayer: any = null;
  let osm = $state.raw<OsmCemetery[]>([]);
  let osmState = $state<'' | 'zoom' | 'busy' | 'error'>('');
  // the form of a cemetery being added or edited
  let draft = $state<{ id: number | null; name: string; address: string; lat: number; lng: number; origin: string } | null>(null);
  const cemIcon = (own: boolean) =>
    L.divIcon({ className: `mr-cem${own ? '' : ' osm'}`, html: '<span><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round"><path d="M12 3v18M7 8h10"/></svg></span>', iconSize: [26, 26], iconAnchor: [13, 13] });
  function btn(label: string, cls: string, fn: () => void) {
    const b = document.createElement('button');
    b.className = `btn sm ${cls}`;
    b.textContent = label;
    b.onclick = (e) => {
      e.stopPropagation();
      map?.closePopup();
      fn();
    };
    return b;
  }
  function cemPopup(c: MapCemetery): HTMLElement {
    const d = document.createElement('div');
    d.className = 'mp';
    d.innerHTML = `<div class="mp-h">${esc(c.name || t('Cemetery not given'))}</div>${c.address ? `<div class="mp-a">${esc(c.address)}</div>` : ''}<div class="mp-a">${esc(t('Buried here: %d', c.persons.length))}</div>`;
    const row = document.createElement('div');
    row.className = 'mp-b';
    if (c.persons.length) row.append(btn(t('Show the graves'), '', () => pickCemetery(c.name)));
    if (!viewer && c.lat != null && c.lng != null) {
      if (c.place_id) row.append(btn(t('Edit'), 'ghost', () => (draft = { id: c.place_id!, name: c.name, address: c.address || '', lat: c.lat!, lng: c.lng!, origin: '' })));
      else row.append(btn(t('Save on the map'), 'ghost', () => (draft = { id: null, name: c.name, address: '', lat: c.lat!, lng: c.lng!, origin: 'manual' })));
    }
    d.append(row);
    return d;
  }
  function osmPopup(o: OsmCemetery): HTMLElement {
    const d = document.createElement('div');
    d.className = 'mp';
    d.innerHTML = `<div class="mp-h">${esc(o.name || t('Cemetery'))}</div>${o.address ? `<div class="mp-a">${esc(o.address)}</div>` : ''}<div class="mp-a small">OpenStreetMap</div>`;
    if (!viewer) {
      const row = document.createElement('div');
      row.className = 'mp-b';
      row.append(btn(t('Add to my cemeteries'), '', () => (draft = { id: null, name: o.name || '', address: o.address, lat: o.lat, lng: o.lng, origin: 'osm' })));
      d.append(row);
    }
    return d;
  }
  function drawCems() {
    cemLayer?.remove();
    cemLayer = null;
    osmLayer?.remove();
    osmLayer = null;
    if (!map || !L || !cemOn || mig) return;
    cemLayer = L.layerGroup();
    const mine: [number, number][] = [];
    for (const c of cems) {
      if (c.lat == null || c.lng == null) continue;
      mine.push([c.lat, c.lng]);
      L.marker([c.lat, c.lng], { icon: cemIcon(true), title: c.name, zIndexOffset: 500 }).bindPopup(() => cemPopup(c), { maxWidth: 280 }).addTo(cemLayer);
    }
    cemLayer.addTo(map);
    osmLayer = L.layerGroup();
    for (const o of osm) {
      if (mine.some((m) => metres(m, [o.lat, o.lng]) < 250)) continue;
      L.marker([o.lat, o.lng], { icon: cemIcon(false), title: o.name }).bindPopup(() => osmPopup(o), { maxWidth: 280 }).addTo(osmLayer);
    }
    osmLayer.addTo(map);
  }
  let osmTimer = 0;
  function loadOsm() {
    clearTimeout(osmTimer);
    if (!map || !cemOn || mig) return;
    if (map.getZoom() < 11) {
      osmState = 'zoom';
      return;
    }
    osmTimer = window.setTimeout(async () => {
      const b = map.getBounds();
      osmState = 'busy';
      try {
        osm = await osmCemeteries(b.getSouth(), b.getWest(), b.getNorth(), b.getEast());
        osmState = '';
      } catch {
        osmState = 'error';
      }
      drawCems();
    }, 500);
  }
  function toggleCems() {
    cemOn = !cemOn;
    drawCems();
    loadOsm();
  }
  function newCemetery() {
    const c = map?.getCenter();
    draft = { id: null, name: '', address: '', lat: c?.lat ?? 52, lng: c?.lng ?? 19, origin: 'manual' };
  }
  // the position of the cemetery in the form: a marker that can be dragged
  let draftMarker: any = null;
  $effect(() => {
    const d = draft;
    if (!map || !L) return;
    if (!d) {
      draftMarker?.remove();
      draftMarker = null;
      return;
    }
    const at: [number, number] = [d.lat, d.lng];
    if (!draftMarker) {
      draftMarker = L.marker(at, { icon: cemIcon(true), draggable: true, zIndexOffset: 1000 }).addTo(map);
      draftMarker.on('dragend', () => {
        const p = draftMarker.getLatLng();
        if (draft) (draft.lat = p.lat), (draft.lng = p.lng);
      });
    } else draftMarker.setLatLng(at);
  });
  let finding = $state(false);
  async function findDraft() {
    if (!draft) return;
    const q = [draft.address, draft.name].map((x) => x.trim()).filter(Boolean).join(', ');
    if (!q) return;
    finding = true;
    try {
      const r = await searchPlaces(q);
      if (r.length && draft) {
        draft.lat = r[0].lat;
        draft.lng = r[0].lng;
        map?.setView([r[0].lat, r[0].lng], Math.max(map.getZoom(), 15));
      } else toast(t('Not found on the map. Click the place or drag the marker.'));
    } catch {
      toast(t('The map search service did not answer. Try again in a moment.'), 'error');
    } finally {
      finding = false;
    }
  }
  async function saveDraft() {
    if (!draft || !draft.name.trim()) return;
    try {
      await call('cemeteries.save', { id: draft.id, name: draft.name.trim(), address: draft.address.trim(), lat: draft.lat, lng: draft.lng, origin: draft.origin });
      toast(t('Saved: %s', draft.name.trim()));
      draft = null;
      await load();
    } catch (e) {
      toast((e as any)?.message || String(e), 'error');
    }
  }
  async function removeDraft() {
    if (!draft?.id) return;
    await call('cemeteries.remove', { id: draft.id });
    draft = null;
    await load();
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
      addBaseMap(L, map, app.settings?.appearance?.map_style).catch(() => {});
      // placing a missing place by hand: click on the map
      map.on('click', (e: any) => {
        if (picking) saveCoords(picking, e.latlng.lat, e.latlng.lng, 'manual');
        else if (draft) {
          // with the form open a click moves the cemetery
          draft.lat = e.latlng.lat;
          draft.lng = e.latlng.lng;
        }
      });
      map.on('moveend', loadOsm);
    }
    if (cluster) cluster.remove();
    if (mig) {
      drawMig();
      setTimeout(() => map?.invalidateSize(), 50);
      return;
    }
    migLayer?.remove();
    migLayer = null;
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
    drawCems();
    if (focusOn) {
      map.setView(focusOn, Math.max(map.getZoom(), 9));
      focusOn = null;
    } else if (bounds.length) map.fitBounds(bounds, { padding: [40, 40], maxZoom: person ? 9 : 7 });
    setTimeout(() => map?.invalidateSize(), 50);
  }

  $effect(() => {
    void data;
    void clan;
    void buried;
    void kinds.birth, kinds.death, kinds.marriage, kinds.event, kinds.burial;
    void enabled;
    render();
  });

  $effect(() => {
    return () => {
      cancelled = true;
      clearInterval(timer);
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
  // the name typed in the search replaces the (misspelled) name in the tree
  let renames = $state<Record<string, boolean>>({});
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
    let target = name;
    const typed = (queries[name] ?? '').trim();
    if (renames[name] && typed && typed !== name) {
      await call('places.rename', { from: name, to: typed });
      target = typed;
    }
    await call('places.set', { name: target, lat, lng, origin });
    picking = null;
    delete found[name];
    delete renames[name];
    focusOn = [lat, lng];
    toast(t('Saved: %s', target));
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
      {#if !mig}
        <div class="kinds" role="group" aria-label={t('Show')}>
          {#each Object.keys(KIND_LABEL) as k (k)}
            <button aria-pressed={kinds[k]} style="--c:{KIND_COLOR[k]}" onclick={() => (kinds[k] = !kinds[k])}><i aria-hidden="true"></i>{KIND_LABEL[k]()}</button>
          {/each}
        </div>
      {/if}
      <button class="migbtn" aria-pressed={mig} onclick={toggleMig}><Waypoints size={16} /> {t('Migrations')}</button>
      {#if !mig}<button class="migbtn" aria-pressed={cemOn} onclick={toggleCems}><Cross size={16} /> {t('Cemeteries')}</button>{/if}
      {#if mig && !person && homes.length}
        <select value={home} onchange={(e) => setHome(e.currentTarget.value)} aria-label={t('From one place')}>
          <option value="">{t('The whole family')}</option>
          <optgroup label={t('Who from this place went further')}>
            {#each homes as h (h.name)}<option value={h.name}>{h.name} ({h.count})</option>{/each}
          </optgroup>
        </select>
      {/if}
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
      {#if cems.length && !person && !mig}
        <select value={cemetery} onchange={(e) => pickCemetery(e.currentTarget.value)} aria-label={t('Cemetery')}>
          <option value="">{t('All cemeteries')}</option>
          {#each cems as c (c.name)}<option value={c.name}>{c.name || t('Cemetery not given')} ({c.persons.length})</option>{/each}
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
        {#if cem && !cem.located}<span class="count">{t('The graves at this cemetery have no position yet: set it in the grave form of a person.')}</span>{/if}
        {#if cemOn && !mig}
          {#if osmState === 'zoom'}<span class="count">{t('Zoom in to see the cemeteries of OpenStreetMap.')}</span>
          {:else if osmState === 'busy'}<span class="count">{t('Looking for cemeteries…')}</span>
          {:else if osmState === 'error'}<span class="count">{t('OpenStreetMap did not answer. Try again in a moment.')}</span>{/if}
          {#if !viewer && !draft}<button class="btn sm" onclick={newCemetery}>{t('Add a cemetery')}</button>{/if}
        {/if}
      </div>
    </div>
    <div class="map" class:picking={picking || !!draft} bind:this={el}></div>
    {#if draft}
      <form class="panel cemform no-print" onsubmit={(e) => (e.preventDefault(), saveDraft())}>
        <h2>{draft.id ? t('Edit the cemetery') : t('New cemetery')}</h2>
        <label class="field"><span class="lbl">{t('Name')}</span><input type="text" bind:value={draft.name} placeholder={t('e.g. Parish cemetery in Wieliczka')} required /></label>
        <label class="field"><span class="lbl">{t('Address')}</span><input type="text" bind:value={draft.address} placeholder={t('Street, town, country')} /></label>
        <button class="btn sm" type="button" onclick={findDraft} disabled={finding || !(draft.address.trim() || draft.name.trim())}><SearchIcon size={15} /> {t('Find the address on the map')}</button>
        <p class="hint">{t('You can also click the place on the map or drag the marker.')} {t('Graves with this cemetery name are shown here. You can choose it in the grave form of a person.')}</p>
        <div class="actions">
          <button class="btn primary" type="submit">{t('Save')}</button>
          <button class="btn ghost" type="button" onclick={() => (draft = null)}>{t('Cancel')}</button>
          {#if draft.id}<button class="btn ghost danger" type="button" onclick={removeDraft}>{t('Remove from the map')}</button>{/if}
        </div>
      </form>
    {/if}

    {#if mig}
      <div class="timeline no-print">
        {#if moves.length || births.length}
          <button class="btn icon primary" onclick={play} aria-label={playing ? t('Pause') : t('Play')}>{#if playing}<Pause size={18} />{:else}<Play size={18} />{/if}</button>
          <div class="yr" aria-live="off">{yearNow}</div>
          <input type="range" min={years[0]} max={years[1]} bind:value={yearNow} aria-label={t('Year')} />
          <div class="legend small">
            <span><i class="lg gen"></i>{t('a new generation elsewhere')}</span>
            <span><i class="lg life"></i>{t('moved during their life')}</span>
          </div>
          <ul class="recent small">
            {#each recent as m (m.person + m.kind + m.year)}
              <li><b>{m.year}</b> <a href="#/person/{m.person}">{m.name}</a>: {m.fromName} → {m.toName}</li>
            {:else}
              <li class="muted">{tn('%d move in the family', '%d moves in the family', moves.length)}</li>
            {/each}
          </ul>
        {:else}
          <p class="small muted">{t('No moves to show yet: places of birth and death need coordinates.')}</p>
        {/if}
      </div>
    {/if}

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
              {#if (queries[p.place] ?? p.place).trim() && (queries[p.place] ?? p.place).trim() !== p.place}
                <label class="check small ren"><input type="checkbox" bind:checked={renames[p.place]} /> {t('Also change the name in the tree to “%s”', (queries[p.place] ?? '').trim())}</label>
              {/if}
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
  .ren {
    display: flex;
    gap: 6px;
    align-items: flex-start;
    margin: 6px 0 2px;
  }
  .map {
    flex: 1;
    min-height: 300px;
    z-index: 0;
  }
  .migbtn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 0;
    border-radius: 999px;
    padding: 8px 14px;
    background: var(--surface);
    box-shadow: var(--shadow);
    color: var(--ink);
    font: inherit;
    font-size: 0.88rem;
    font-weight: 600;
    cursor: pointer;
  }
  .migbtn[aria-pressed='true'] {
    background: var(--ink);
    color: var(--surface);
  }
  .timeline {
    position: absolute;
    inset: auto 16px 22px 16px;
    margin: 0 auto;
    max-width: 760px;
    z-index: 1000;
    background: var(--surface);
    box-shadow: var(--shadow-raised, var(--shadow));
    border-radius: 18px;
    padding: 12px 16px;
    display: grid;
    grid-template-columns: auto auto 1fr;
    gap: 6px 14px;
    align-items: center;
  }
  .yr {
    font-size: 1.7rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
    min-width: 3.2ch;
  }
  .timeline input[type='range'] {
    width: 100%;
  }
  .legend,
  .recent {
    grid-column: 1 / -1;
  }
  .legend {
    display: flex;
    gap: 16px;
    color: var(--muted);
  }
  .lg {
    display: inline-block;
    width: 22px;
    height: 0;
    border-top: 3px solid var(--heirloom);
    vertical-align: middle;
    margin-inline-end: 6px;
  }
  .lg.life {
    border-top: 3px dashed var(--ev-death);
  }
  .recent {
    list-style: none;
    margin: 0;
    padding: 0;
    color: var(--muted);
  }
  .recent b {
    color: var(--ink);
    font-variant-numeric: tabular-nums;
  }
  .map :global(path.mig-gen) {
    stroke: var(--heirloom);
  }
  .map :global(path.mig-life) {
    stroke: var(--ev-death);
    stroke-dasharray: 6 5;
  }
  .map :global(path.mig.fresh) {
    stroke-width: 5;
  }
  .map :global(path.mig-head.mig-gen) {
    fill: var(--heirloom);
  }
  .map :global(path.mig-head.mig-life) {
    fill: var(--ev-death);
  }
  .map :global(path.mig-dot) {
    fill: var(--surface);
    stroke: var(--ink);
    fill-opacity: 1;
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
  .map :global(.mr-cem span) {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 8px;
    background: var(--surface);
    color: var(--ink);
    border: 2px solid var(--ink);
    box-shadow: var(--shadow-raised);
  }
  .map :global(.mr-cem.osm span) {
    border-style: dashed;
    color: var(--ink-3);
    border-color: var(--ink-3);
    opacity: 0.85;
  }
  .map :global(.mp-a) {
    color: var(--ink-3);
    font-size: 0.85rem;
    margin-top: 2px;
  }
  .map :global(.mp-b) {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: 8px;
  }
  .cemform {
    position: absolute;
    z-index: 1000;
    right: 16px;
    bottom: 16px;
    width: min(360px, calc(100% - 32px));
    display: flex;
    flex-direction: column;
    gap: 10px;
    box-shadow: var(--shadow-raised);
  }
  .cemform h2 {
    margin: 0;
    font-size: 1.05rem;
  }
  .cemform .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
