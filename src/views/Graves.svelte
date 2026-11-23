<script lang="ts">
  import { addBaseMap } from '../lib/basemap';
  import { printPage } from '../lib/print';
  // Graves of the family: by cemetery, on a map, and a route for a visit
  // (All Saints' Day) opened in Google Maps.
  import { app, call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
    import { openLightbox, toast } from '../lib/ui.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  import Img from '../lib/components/Img.svelte';
  import Route from '@lucide/svelte/icons/route';
  import Printer from '@lucide/svelte/icons/printer';
  import Flower from '@lucide/svelte/icons/flower-2';
  import 'leaflet/dist/leaflet.css';
  import 'leaflet.markercluster/dist/MarkerCluster.css';
  import { haystack, norm, words } from '../lib/text';
  import type { GraveEntry } from '../lib/types';

  const list = useLoad(() => call<GraveEntry[]>('graves.list'));
  const viewer = $derived(!!app.info?.viewer);
  const mapOn = $derived(!!app.info?.map_external);
  let q = $state('');

  // who is skipped on the next visit (kept on this device)
  const KEY = 'myroots.graves.skip';
  let skip = $state<Set<number>>(
    (() => {
      try {
        return new Set<number>(JSON.parse(localStorage.getItem(KEY) || '[]'));
      } catch {
        return new Set<number>();
      }
    })(),
  );
  function toggle(id: number) {
    const s = new Set(skip);
    if (s.has(id)) s.delete(id);
    else s.add(id);
    skip = s;
    try {
      localStorage.setItem(KEY, JSON.stringify([...s]));
    } catch {
      /* this session only */
    }
  }

  const shown = $derived.by(() => {
    const all = list.data || [];
    const w = words(q);
    if (!w.length) return all;
    const hay = haystack(all, 'graves', (g) => `${g.person.name} ${g.grave.cemetery} ${g.grave.location}`);
    return all.filter((_, i) => w.every((x) => hay[i].includes(x)));
  });
  // Large families: cemeteries start folded (thousands of rows at once were slow).
  const BIG = 300;
  const big = $derived(shown.length > BIG && !q.trim());
  let unfolded = $state<Set<string>>(new Set());
  let allOpen = $state(false);
  function fold(name: string) {
    const s = new Set(unfolded);
    if (s.has(name)) s.delete(name);
    else s.add(name);
    unfolded = s;
  }
  const isOpen = (name: string) => !big || allOpen || unfolded.has(name);

  type Group = { name: string; items: GraveEntry[]; lat: number | null; lng: number | null };
  const groups = $derived.by(() => {
    const m = new Map<string, Group>();
    for (const g of shown) {
      const name = g.grave.cemetery.trim();
      const k = norm(name);
      if (!m.has(k)) m.set(k, { name, items: [], lat: null, lng: null });
      m.get(k)!.items.push(g);
    }
    for (const gr of m.values()) {
      const pts = gr.items.filter((g) => g.grave.lat != null && g.grave.lng != null);
      if (pts.length) {
        gr.lat = pts.reduce((a, g) => a + g.grave.lat!, 0) / pts.length;
        gr.lng = pts.reduce((a, g) => a + g.grave.lng!, 0) / pts.length;
      }
    }
    // known cemeteries first, the ones without a name last
    return [...m.values()].sort((a, b) => (!a.name ? 1 : !b.name ? -1 : a.name.localeCompare(b.name)));
  });

  // stops of the visit: one per cemetery with a position and at least one grave to visit
  const stops = $derived(groups.filter((g) => g.lat != null && g.items.some((i) => !skip.has(i.person.id))));
  const noPos = $derived(groups.filter((g) => g.lat == null).reduce((a, g) => a + g.items.length, 0));

  const dist = (a: [number, number], b: [number, number]) => {
    const k = Math.cos(((a[0] + b[0]) / 2) * (Math.PI / 180));
    return Math.hypot(a[0] - b[0], (a[1] - b[1]) * k);
  };
  function order(from: [number, number] | null, pts: Group[]): Group[] {
    const left = [...pts];
    const out: Group[] = [];
    let cur = from ?? ([left[0].lat!, left[0].lng!] as [number, number]);
    while (left.length) {
      let best = 0;
      for (let i = 1; i < left.length; i++) if (dist(cur, [left[i].lat!, left[i].lng!]) < dist(cur, [left[best].lat!, left[best].lng!])) best = i;
      const g = left.splice(best, 1)[0];
      out.push(g);
      cur = [g.lat!, g.lng!];
    }
    return out;
  }
  const here = (): Promise<[number, number] | null> =>
    new Promise((resolve) => {
      if (!navigator.geolocation) return resolve(null);
      navigator.geolocation.getCurrentPosition((p) => resolve([p.coords.latitude, p.coords.longitude]), () => resolve(null), { timeout: 6000, maximumAge: 600000 });
    });

  let planning = $state(false);
  async function route() {
    if (!stops.length) return;
    planning = true;
    const start = await here();
    planning = false;
    let seq = order(start, stops);
    if (seq.length > 10) {
      toast(t('Google Maps takes up to 10 stops: the route covers the 10 nearest cemeteries.'), 'info');
      seq = seq.slice(0, 10);
    }
    const ll = (g: Group) => `${g.lat!.toFixed(6)},${g.lng!.toFixed(6)}`;
    const dest = seq[seq.length - 1];
    const way = seq.slice(0, -1).map(ll).join('|');
    const url = `https://www.google.com/maps/dir/?api=1&destination=${ll(dest)}${way ? `&waypoints=${encodeURIComponent(way)}` : ''}&travelmode=driving`;
    // a link click: the desktop app opens it in the system browser (window.open would not)
    const a = document.createElement('a');
    a.href = url;
    a.target = '_blank';
    a.rel = 'noopener noreferrer';
    document.body.appendChild(a);
    a.click();
    a.remove();
  }

  /* ---------- map ---------- */
  let el: HTMLDivElement | undefined = $state();
  let map: any = null;
  let layer: any = null;
  let Lf: any = null;
  const located = $derived(shown.filter((g) => g.grave.lat != null && g.grave.lng != null));
  function esc(s: string) {
    return String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!);
  }
  $effect(() => {
    if (!el || !mapOn) return;
    let cancelled = false;
    (async () => {
      Lf = (await import('leaflet')).default;
      (window as any).L = Lf;
      await import('leaflet.markercluster');
      if (cancelled || !el) return;
      map = Lf.map(el, { zoomControl: true, scrollWheelZoom: false });
      addBaseMap(Lf, map, app.settings?.appearance?.map_style).catch(() => {});
      draw();
    })();
    return () => {
      cancelled = true;
      map?.remove();
      map = null;
      layer = null;
    };
  });
  function draw() {
    if (!map || !Lf) return;
    layer?.remove();
    // many graves: group nearby pins (the same plugin as the main map)
    layer = (located.length > 300 && (Lf as any).markerClusterGroup ? (Lf as any).markerClusterGroup({ showCoverageOnHover: false, maxClusterRadius: 40 }) : Lf.layerGroup()).addTo(map);
    const bounds: [number, number][] = [];
    for (const g of located) {
      const icon = Lf.divIcon({ className: 'mr-pin', html: `<span style="--c:${skip.has(g.person.id) ? 'var(--faint)' : 'var(--ev-death)'}"></span>`, iconSize: [20, 20], iconAnchor: [10, 10] });
      Lf.marker([g.grave.lat!, g.grave.lng!], { icon, title: g.person.name })
        .bindPopup(`<b>${esc(g.person.name)}</b> ${esc(g.person.years)}<br>${esc(g.grave.cemetery)}${g.grave.location ? '<br>' + esc(g.grave.location) : ''}`)
        .addTo(layer);
      bounds.push([g.grave.lat!, g.grave.lng!]);
    }
    if (bounds.length === 1) map.setView(bounds[0], 15);
    else if (bounds.length) map.fitBounds(bounds, { padding: [30, 30], maxZoom: 15 });
    else map.setView([52.1, 19.4], 6);
  }
  $effect(() => {
    void located;
    void skip;
    draw();
  });
  function focus(g: GraveEntry) {
    if (map && g.grave.lat != null) {
      map.setView([g.grave.lat, g.grave.lng], 18);
      el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Graves')}</h1>
      {#if list.data}<p class="muted small sub">{tn('%d grave', '%d graves', list.data.length)}{#if noPos} · {tn('%d without a position', '%d without a position', noPos)}{/if}</p>{/if}
    </div>
    <div class="acts no-print">
      <input type="search" placeholder={t('Search…')} bind:value={q} aria-label={t('Search')} />
      {#if big}<button class="btn" onclick={() => (allOpen = !allOpen)}>{allOpen ? t('Fold all') : t('Unfold all')}</button>{/if}
      <button class="btn" onclick={printPage}><Printer size={16} /> {t('Print the list')}</button>
      <button class="btn primary" onclick={route} disabled={!stops.length || planning}><Route size={16} /> {t('Route of the visit')}</button>
    </div>
  </div>

  {#if list.data && !list.data.length}
    <div class="panel empty">
      <Flower size={28} />
      <p>{t('No graves recorded yet.')}</p>
      {#if !viewer}<p class="muted small">{t('Add a grave in the edit form of a person (tab Basic): cemetery, place, position and a photo of the tombstone.')}</p>{/if}
    </div>
  {:else if list.data}
    {#if mapOn && located.length}<div class="gmap no-print" bind:this={el}></div>{/if}
    {#if stops.length}
      <p class="hint no-print">{tn('The route leads through %d cemetery; untick the graves you will not visit this time.', 'The route leads through %d cemeteries; untick the graves you will not visit this time.', stops.length)}</p>
    {/if}
    {#each groups as g (g.name)}
      <section class="cem">
        <h2>
          {#if big}
            <button class="fold" aria-expanded={isOpen(g.name)} onclick={() => fold(g.name)}>{g.name || t('Cemetery not given')}</button>
          {:else}
            {g.name || t('Cemetery not given')}
          {/if}
          <span class="muted small">({g.items.length})</span>
          {#if g.lat != null}<a class="small no-print" href="https://www.google.com/maps/dir/?api=1&destination={g.lat},{g.lng}" target="_blank" rel="noopener noreferrer">{t('Directions')}</a>{/if}
        </h2>
        {#if isOpen(g.name)}
        <ul class="panel rows">
          {#each g.items as e (e.person.id)}
            <li class:off={skip.has(e.person.id)}>
              <input type="checkbox" class="no-print" checked={!skip.has(e.person.id)} onchange={() => toggle(e.person.id)} aria-label={t('Visit: %s', e.person.name)} />
              <Avatar photo={e.person.photo} name={e.person.name} gender={e.person.gender} size={36} />
              <span class="grow">
                <a class="title" href="#/person/{e.person.id}">{e.person.name}</a>
                <span class="sub">{[e.person.years, e.grave.location].filter(Boolean).join(' · ')}</span>
              </span>
              {#if e.grave.lat != null}<button class="btn ghost sm no-print" onclick={() => focus(e)} disabled={!mapOn}>{t('Show on map')}</button>{/if}
              {#if e.grave.photo}
                <button class="th" onclick={() => openLightbox([{ media: e.grave.photo!, caption: e.person.name }])} aria-label={t('Enlarge')}><Img id={e.grave.photo} alt="" /></button>
              {/if}
            </li>
          {/each}
        </ul>
        {/if}
      </section>
    {/each}
  {/if}
</div>

<style>
  .sub {
    margin: 4px 0 0;
  }
  .fold {
    all: unset;
    cursor: pointer;
    font: inherit;
  }
  .fold::before {
    content: '▸ ';
    color: var(--muted);
  }
  .fold[aria-expanded='true']::before {
    content: '▾ ';
  }
  .fold:focus-visible {
    outline: 2px solid var(--ink);
    outline-offset: 2px;
  }
  .acts {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
  }
  .acts input {
    width: 200px;
  }
  .gmap {
    height: 340px;
    border-radius: var(--radius-lg);
    overflow: hidden;
    margin-bottom: 14px;
    z-index: 0;
  }
  .cem h2 {
    font-size: 1.05rem;
    display: flex;
    gap: 10px;
    align-items: baseline;
    flex-wrap: wrap;
    margin: 22px 0 8px;
  }
  .rows {
    list-style: none;
    margin: 0;
    padding: 4px 0;
  }
  .rows li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 14px;
  }
  .rows li + li {
    border-top: 1px solid var(--line);
  }
  .rows li.off {
    opacity: 0.5;
  }
  .grow {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .title {
    font-weight: 600;
  }
  .sub {
    color: var(--muted);
    font-size: 0.85rem;
  }
  .th {
    width: 40px;
    height: 52px;
    border: 0;
    padding: 0;
    border-radius: 6px;
    overflow: hidden;
    cursor: zoom-in;
    background: var(--ink-soft);
    flex: none;
  }
  .th :global(img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .empty {
    text-align: center;
    padding: 36px 20px;
    color: var(--muted);
  }
  @media (max-width: 640px) {
    .acts input {
      width: 100%;
    }
  }
</style>
