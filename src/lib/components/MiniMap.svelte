<script lang="ts">
  import { addBaseMap } from '../basemap';
  // Small map of one person's places in the profile (only when map services
  // are turned on and the places already have coordinates).
  import { app, call } from '../api.svelte';
  import { t } from '../i18n/index.svelte';
  import { fmtDate } from '../format';
    import 'leaflet/dist/leaflet.css';
  import type { MapData } from '../types';

  let { person }: { person: number } = $props();
  const enabled = $derived(!!app.info?.map_external);
  let el: HTMLDivElement | undefined = $state();
  let data = $state<MapData | null>(null);
  let map: any = null;

  $effect(() => {
    const id = person;
    if (!enabled) return;
    call<MapData>('map.data', { person: id })
      .then((d) => (data = d))
      .catch(() => (data = null));
  });

  const points = $derived((data?.geocoded || []).filter((p) => p.lat != null && p.lng != null));

  function esc(s: string) {
    return String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!);
  }
  const COLOR: Record<string, string> = { birth: 'var(--ev-birth)', marriage: 'var(--ev-marriage)', death: 'var(--ev-death)', event: 'var(--ev-event)', burial: 'var(--ev-burial)' };
  function label(k: string, l: string) {
    return k === 'birth' ? t('Birth') : k === 'death' ? t('Death') : k === 'marriage' ? t('Marriage to %s', l || '?') : k === 'burial' ? [t('Grave'), l].filter(Boolean).join(': ') : l || t('Event');
  }

  $effect(() => {
    if (!el || !points.length) return;
    let cancelled = false;
    (async () => {
      const L = (await import('leaflet')).default;
      if (cancelled || !el) return;
      map?.remove();
      map = L.map(el, { zoomControl: false, attributionControl: true, scrollWheelZoom: false, dragging: !L.Browser.mobile });
      addBaseMap(L, map, app.settings?.appearance?.map_style).catch(() => {});
      const bounds: [number, number][] = [];
      for (const p of points) {
        const evs = p.events.filter((e) => e.person === person);
        const kind = evs[0]?.kind || 'event';
        const icon = L.divIcon({ className: 'mr-pin', html: `<span style="--c:${COLOR[kind] || 'var(--ink)'}"></span>`, iconSize: [22, 22], iconAnchor: [11, 11] });
        const rows = evs.map((e) => `<div>${esc(label(e.kind, e.label))}${e.date ? ', ' + esc(fmtDate(e.date)) : ''}</div>`).join('');
        L.marker([p.lat!, p.lng!], { icon, title: p.place }).bindPopup(`<b>${esc(p.place)}</b>${rows}`).addTo(map);
        bounds.push([p.lat!, p.lng!]);
      }
      if (bounds.length === 1) map.setView(bounds[0], 9);
      else map.fitBounds(bounds, { padding: [28, 28], maxZoom: 9 });
    })();
    return () => {
      cancelled = true;
      map?.remove();
      map = null;
    };
  });
</script>

{#if enabled && points.length}
  <section class="block minimap">
    <h2>{t('Map')}</h2>
    <div class="mm" bind:this={el}></div>
    <a class="small" href="#/map/{person}">{t('Open the map')}</a>
  </section>
{/if}

<style>
  .mm {
    height: 220px;
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: var(--shadow);
    z-index: 0;
    margin-bottom: 6px;
  }
  .mm :global(.mr-pin span) {
    display: block;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--c);
    border: 3px solid #fff;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
  }
</style>
