<script lang="ts">
  import { app, call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { toast, promptDialog } from '../lib/ui.svelte';
  import { geocodeMissing, geocodeOne, type GeoProgress } from '../lib/geocode';
  import { norm } from '../lib/text';
  import Modal from '../lib/components/Modal.svelte';
  import type { PlaceUse } from '../lib/types';

  const places = useLoad(() => call<PlaceUse[]>('places.list'));
  let q = $state('');
  let only = $state('');
  let edit = $state<PlaceUse | null>(null);
  let lat = $state('');
  let lng = $state('');
  let geo = $state<GeoProgress | null>(null);
  let stop = false;
  const ext = $derived(!!app.settings?.map_external);

  const shown = $derived.by(() => {
    const w = norm(q).split(' ').filter(Boolean);
    return (places.data || []).filter((p) => {
      if (only === 'missing' && p.lat != null) return false;
      if (only === 'failed' && p.origin !== 'failed') return false;
      return !w.length || w.every((x) => norm(p.name).includes(x));
    });
  });
  const missing = $derived((places.data || []).filter((p) => p.lat == null).length);

  function open(p: PlaceUse) {
    edit = p;
    lat = p.lat != null ? String(p.lat) : '';
    lng = p.lng != null ? String(p.lng) : '';
  }
  function parseCoords() {
    // Accept "52.23, 21.01" pasted into the first field.
    const m = lat.match(/^\s*(-?\d+(?:\.\d+)?)\s*[,;\s]\s*(-?\d+(?:\.\d+)?)\s*$/);
    if (m) {
      lat = m[1];
      lng = m[2];
    }
  }
  async function saveCoords() {
    if (!edit) return;
    parseCoords();
    const la = lat.trim() ? Number(lat.replace(',', '.')) : null;
    const lo = lng.trim() ? Number(lng.replace(',', '.')) : null;
    if ((la != null && (isNaN(la) || la < -90 || la > 90)) || (lo != null && (isNaN(lo) || lo < -180 || lo > 180))) {
      toast(t('Invalid coordinates.'), 'error');
      return;
    }
    await call('places.set', { name: edit.name, lat: la, lng: lo, origin: 'manual' });
    edit = null;
    toast(t('Saved.'));
  }
  async function findOne() {
    if (!edit) return;
    const r = await geocodeOne(edit.name).catch(() => null);
    if (r) {
      lat = String(r.lat);
      lng = String(r.lng);
    } else toast(t('Place not found.'), 'error');
  }
  async function rename(p: PlaceUse) {
    const to = (await promptDialog(t('Rename place'), t('The new name is saved in every person who uses it.'), p.name, t('Rename')))?.trim();
    if (!to || to === p.name) return;
    const r = await call<{ count: number }>('places.rename', { from: p.name, to });
    toast(tn('Changed in %d record.', 'Changed in %d records.', r.count));
  }
  async function geocodeAll(retry = false) {
    stop = false;
    geo = { done: 0, total: 0, found: 0, failed: 0 };
    const r = await geocodeMissing((p) => (geo = p), () => stop, retry);
    toast(t('Geocoded: %1$d, not recognized: %2$d, remaining: %3$d.', r.found, r.failed, r.total - r.done));
    geo = null;
  }
  function originLabel(o: string) {
    return ({ nominatim: 'OpenStreetMap', manual: t('manual'), gedcom: 'GEDCOM', failed: t('not found') } as Record<string, string>)[o] || '';
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Places')}</h1>
      <p class="lead">{t('All places used in the tree. Fix spelling in one go and set coordinates for the map.')}</p>
    </div>
    {#if ext}
      <div class="actions">
        {#if geo}
          <span class="small muted">{t('Finding places on the map… %1$d of %2$d', geo.done, geo.total)}</span>
          <button class="btn sm" onclick={() => (stop = true)}>{t('Stop')}</button>
        {:else if missing}
          <button class="btn" onclick={() => geocodeAll(false)}>{t('Find missing on the map')}</button>
          <button class="btn ghost" onclick={() => geocodeAll(true)}>{t('Retry not found')}</button>
        {/if}
      </div>
    {/if}
  </div>
  {#if !ext}
    <div class="notice">{t('Automatic search is off because the online map is off. You can still enter coordinates by hand.')} <a href="#/map">{t('Map')}</a></div>
  {/if}
  <div class="filters">
    <input type="search" placeholder={t('Search…')} bind:value={q} />
    <select bind:value={only}>
      <option value="">{tn('All (%d)', 'All (%d)', places.data?.length || 0)}</option>
      <option value="missing">{t('Without coordinates (%d)', missing)}</option>
      <option value="failed">{t('Not found')}</option>
    </select>
  </div>
  {#if places.data && !places.data.length}
    <div class="panel empty">{t('No places yet. Places appear here when you enter them for people and events.')}</div>
  {:else}
    <table class="data panel tbl">
      <thead><tr><th>{t('Place')}</th><th>{t('Uses')}</th><th>{t('Coordinates')}</th><th></th></tr></thead>
      <tbody>
        {#each shown as p (p.name)}
          <tr>
            <td>{p.name}</td>
            <td>{p.count}</td>
            <td class="small">{#if p.lat != null}{p.lat.toFixed(4)}, {p.lng?.toFixed(4)}{:else}<span class="faint">—</span>{/if} <span class="faint">{originLabel(p.origin)}</span></td>
            <td class="act">
              <button class="btn sm ghost" onclick={() => open(p)}>{t('Coordinates')}</button>
              <button class="btn sm ghost" onclick={() => rename(p)}>{t('Rename')}</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

{#if edit}
  <Modal title={edit.name} onclose={() => (edit = null)}>
    <div class="form-grid">
      <label class="field"><span class="lbl">{t('Latitude')}</span><input type="text" inputmode="decimal" bind:value={lat} onblur={parseCoords} placeholder="52.2297" /></label>
      <label class="field"><span class="lbl">{t('Longitude')}</span><input type="text" inputmode="decimal" bind:value={lng} placeholder="21.0122" /></label>
    </div>
    <p class="hint">{t('Tip: in any online map, right-click a spot to copy its coordinates and paste them into the first field.')}</p>
    {#if ext}<button class="btn sm" onclick={findOne}>{t('Find on the map')}</button>{/if}
    {#snippet footer()}
      <button class="btn" onclick={() => (edit = null)}>{t('Cancel')}</button>
      <button class="btn primary" onclick={saveCoords}>{t('Save')}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .filters {
    display: flex;
    gap: 8px;
    margin-bottom: 14px;
    flex-wrap: wrap;
  }
  .filters input {
    flex: 1 1 240px;
  }
  .filters select {
    width: auto;
  }
  .tbl {
    padding: 0;
  }
  .act {
    text-align: end;
    white-space: nowrap;
  }
</style>
