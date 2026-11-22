<script lang="ts">
  // The grave of a person: cemetery, place on it, position, tombstone photo.
  import PhotoField from './PhotoField.svelte';
  import MapPicker from './MapPicker.svelte';
  import { call } from '../api.svelte';
  import { gpsFromFile } from '../exif';
  import { toast } from '../ui.svelte';
  import { t } from '../i18n/index.svelte';
  import Crosshair from '@lucide/svelte/icons/locate-fixed';
  import MapPin from '@lucide/svelte/icons/map-pin';
  import X from '@lucide/svelte/icons/x';
  import type { Grave } from '../types';

  let { grave = $bindable(), name = '' }: { grave: Grave; name?: string } = $props();
  let picking = $state(false);
  let locating = $state(false);
  let cemeteries = $state<string[]>([]);
  const listId = 'cem-' + Math.random().toString(36).slice(2);

  $effect(() => {
    call<{ name: string; count: number }[]>('graves.cemeteries')
      .then((l) => (cemeteries = l.map((c) => c.name)))
      .catch(() => {});
  });

  function setPos(lat: number, lng: number) {
    grave.lat = Math.round(lat * 1e6) / 1e6;
    grave.lng = Math.round(lng * 1e6) / 1e6;
  }

  async function fromPhoto(f: File) {
    const p = await gpsFromFile(f);
    if (p) {
      setPos(p[0], p[1]);
      toast(t('The position of the grave was read from the photo.'));
    }
  }

  function here() {
    if (!navigator.geolocation) return toast(t('This device cannot share its position.'), 'error');
    locating = true;
    navigator.geolocation.getCurrentPosition(
      (p) => {
        locating = false;
        setPos(p.coords.latitude, p.coords.longitude);
        toast(t('Saved the current position (accuracy about %d m).', Math.round(p.coords.accuracy)));
      },
      () => {
        locating = false;
        toast(t('The position is not available. Allow location access or point the grave on the map.'), 'error');
      },
      { enableHighAccuracy: true, timeout: 20000 },
    );
  }
</script>

<div class="ge">
  <div class="form-grid">
    <label class="field">
      <span class="lbl">{t('Cemetery')}</span>
      <input type="text" bind:value={grave.cemetery} list={listId} placeholder={t('e.g. Rakowicki Cemetery, Kraków')} />
      <datalist id={listId}>{#each cemeteries as c (c)}<option value={c}></option>{/each}</datalist>
    </label>
    <label class="field">
      <span class="lbl">{t('Place in the cemetery')}</span>
      <input type="text" bind:value={grave.location} placeholder={t('section, row, number')} />
    </label>
    <div class="field full">
      <span class="lbl">{t('Position')}</span>
      <div class="pos">
        {#if grave.lat != null && grave.lng != null}
          <span class="coord"><MapPin size={14} /> {grave.lat.toFixed(5)}, {grave.lng.toFixed(5)}</span>
          <button type="button" class="btn ghost icon sm" onclick={() => ((grave.lat = null), (grave.lng = null))} aria-label={t('Remove')}><X size={14} /></button>
        {:else}
          <span class="muted small">{t('not set')}</span>
        {/if}
        <button type="button" class="btn sm" onclick={() => (picking = true)}><MapPin size={14} /> {t('Point on the map')}</button>
        <button type="button" class="btn sm ghost" onclick={here} disabled={locating}><Crosshair size={14} /> {t('I am at the grave')}</button>
      </div>
      <span class="hint">{t('A photo of the tombstone taken with a phone usually carries the position: it is filled in automatically.')}</span>
    </div>
    <div class="field">
      <span class="lbl">{t('Photo of the tombstone')}</span>
      <PhotoField bind:value={grave.photo} size={96} onfile={fromPhoto} />
    </div>
    <div class="field">
      <label class="field"><span class="lbl">{t('Link to an online grave register')}</span><input type="url" bind:value={grave.url} placeholder="https://" /></label>
      <label class="field"><span class="lbl">{t('Notes')}</span><textarea rows="2" bind:value={grave.notes}></textarea></label>
    </div>
  </div>
</div>

{#if picking}
  <MapPicker
    title={t('Where is the grave of %s?', name || '…')}
    lat={grave.lat}
    lng={grave.lng}
    query={grave.cemetery}
    onpick={(a, b) => {
      setPos(a, b);
      picking = false;
    }}
    onclose={() => (picking = false)}
  />
{/if}

<style>
  .pos {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
  .coord {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-variant-numeric: tabular-nums;
  }
  .ge .field > .field {
    margin-bottom: 8px;
  }
</style>
