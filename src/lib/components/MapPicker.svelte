<script lang="ts">
  import { addBaseMap } from '../basemap';
  // Pick a point on the map (e.g. a grave): search a place, then click the exact spot.
  import Modal from './Modal.svelte';
  import { app, call, refreshInfo } from '../api.svelte';
  import { t } from '../i18n/index.svelte';
  import { searchPlaces } from '../geocode';
  import { toast } from '../ui.svelte';
  import SearchIcon from '@lucide/svelte/icons/search';
  import 'leaflet/dist/leaflet.css';

  let {
    title,
    lat = null,
    lng = null,
    query = '',
    onpick,
    onclose,
  }: { title: string; lat?: number | null; lng?: number | null; query?: string; onpick: (lat: number, lng: number) => void; onclose: () => void } = $props();

  const enabled = $derived(!!app.info?.map_external);
  let el: HTMLDivElement | undefined = $state();
  let map: any = null;
  let marker: any = null;
  let L: any = null;
  // svelte-ignore state_referenced_locally
  let point = $state<[number, number] | null>(lat != null && lng != null ? [lat, lng] : null);
  // svelte-ignore state_referenced_locally
  let q = $state(query);
  let results = $state<{ label: string; lat: number; lng: number }[]>([]);
  let busy = $state(false);

  function place(p: [number, number]) {
    point = p;
    if (!map) return;
    if (marker) marker.setLatLng(p);
    else marker = L.marker(p, { icon: L.divIcon({ className: 'mr-pin', html: '<span style="--c:var(--heirloom)"></span>', iconSize: [22, 22], iconAnchor: [11, 11] }) }).addTo(map);
  }

  $effect(() => {
    if (!el || !enabled) return;
    let cancelled = false;
    (async () => {
      L = (await import('leaflet')).default;
      if (cancelled || !el) return;
      map = L.map(el, { zoomControl: true });
      addBaseMap(L, map, app.settings?.appearance?.map_style).catch(() => {});
      if (point) {
        map.setView(point, 17);
        place(point);
      } else map.setView([52.1, 19.4], 6);
      map.on('click', (e: any) => place([e.latlng.lat, e.latlng.lng]));
      if (!point && q.trim()) search();
    })();
    return () => {
      cancelled = true;
      map?.remove();
      map = null;
      marker = null;
    };
  });

  async function search() {
    if (!q.trim()) return;
    busy = true;
    try {
      results = await searchPlaces(q.trim(), 5);
      if (results[0] && map) map.setView([results[0].lat, results[0].lng], 16);
      if (!results.length) toast(t('Nothing found. Try another spelling or point the place on the map.'), 'info');
    } catch {
      toast(t('The map search service did not answer. Try again in a moment.'), 'error');
    } finally {
      busy = false;
    }
  }

  async function enable() {
    if (!app.settings) return;
    await call('settings.save', { settings: { ...app.settings, map_external: true } });
    await refreshInfo();
  }
</script>

<Modal {title} {onclose} wide>
  {#if !enabled}
    <p>{t('The map uses OpenStreetMap: map tiles are loaded from its servers and place names are sent to its search service (Nominatim) to find coordinates. Nothing else leaves your device.')}</p>
    <button type="button" class="btn primary" onclick={enable}>{t('Turn on the map')}</button>
  {:else}
    <form class="srch" onsubmit={(e) => (e.preventDefault(), search())}>
      <input type="search" bind:value={q} placeholder={t('Search, e.g. the name of the cemetery and the town…')} aria-label={t('Search')} />
      <button class="btn" type="submit" disabled={busy}><SearchIcon size={15} /> {t('Search')}</button>
    </form>
    {#if results.length > 1}
      <ul class="res">
        {#each results as r (r.label)}<li><button type="button" onclick={() => map?.setView([r.lat, r.lng], 16)}>{r.label}</button></li>{/each}
      </ul>
    {/if}
    <div class="mp" bind:this={el}></div>
    <p class="hint">{point ? `${point[0].toFixed(6)}, ${point[1].toFixed(6)}` : t('Click the exact spot on the map.')}</p>
  {/if}
  {#snippet footer()}
    <button type="button" class="btn" onclick={onclose}>{t('Cancel')}</button>
    <button type="button" class="btn primary" disabled={!point} onclick={() => point && onpick(point[0], point[1])}>{t('Save the position')}</button>
  {/snippet}
</Modal>

<style>
  .srch {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
  }
  .srch input {
    flex: 1;
    min-width: 0;
  }
  .res {
    list-style: none;
    margin: 0 0 8px;
    padding: 0;
    font-size: 0.85rem;
  }
  .res button {
    background: none;
    border: 0;
    padding: 3px 0;
    color: var(--link, var(--ink));
    cursor: pointer;
    text-align: start;
  }
  .mp {
    height: min(52vh, 440px);
    border-radius: var(--radius-lg);
    overflow: hidden;
    z-index: 0;
    cursor: crosshair;
  }
  .hint {
    margin: 8px 0 0;
    font-variant-numeric: tabular-nums;
  }
</style>
