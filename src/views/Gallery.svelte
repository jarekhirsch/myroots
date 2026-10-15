<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { backend } from '../lib/backend';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { openLightbox } from '../lib/ui.svelte';
  import { norm } from '../lib/text';
  import Img from '../lib/components/Img.svelte';
  import NotFound from './NotFound.svelte';
  import type { GalleryEntry } from '../lib/types';

  const gal = useLoad(async () => {
    const v = await call<GalleryEntry[]>('gallery');
    await backend.ensureMedia?.(v.map((g) => g.media));
    return v;
  });

  let q = $state('');
  let clan = $state('');
  let kind = $state('');
  let limit = $state(120);

  const clans = $derived([...new Set((gal.data || []).flatMap((g) => g.clans))].sort());
  const shown = $derived.by(() => {
    const words = norm(q).split(' ').filter(Boolean);
    return (gal.data || []).filter(
      (g) =>
        (!clan || g.clans.includes(clan)) &&
        (!kind || g.owner_type === kind) &&
        (!words.length || words.every((w) => norm(`${g.owner_name} ${g.caption}`).includes(w))),
    );
  });

  function open(i: number) {
    openLightbox(
      shown.map((g) => ({ media: g.media, caption: [g.caption, g.owner_name].filter(Boolean).join(' · ') })),
      i,
    );
  }
</script>

{#if gal.error}
  <NotFound private={(gal.error as any)?.code === 'private'} />
{:else}
  <div class="page">
    <div class="page-head">
      <div>
        <h1>{t('Gallery')}</h1>
        {#if gal.data}<p class="lead">{tn('%d photo', '%d photos', shown.length)}</p>{/if}
      </div>
    </div>
    <div class="filters">
      <input type="search" placeholder={t('Search by person…')} bind:value={q} />
      {#if clans.length}
        <select bind:value={clan}>
          <option value="">{t('All clans')}</option>
          {#each clans as c}<option value={c}>{c}</option>{/each}
        </select>
      {/if}
      <select bind:value={kind}>
        <option value="">{t('All photos')}</option>
        <option value="person">{t('People')}</option>
        <option value="pet">{t('Pets')}</option>
        <option value="source">{t('Sources')}</option>
      </select>
    </div>
    {#if gal.data && !gal.data.length}
      <div class="panel empty">{t('There are no photos yet.')}</div>
    {/if}
    <div class="masonry">
      {#each shown.slice(0, limit) as g, i (g.media + ':' + g.owner_id + ':' + i)}
        <figure>
          <button onclick={() => open(i)} aria-label={g.caption || g.owner_name}><Img id={g.media} alt={g.caption} /></button>
          <figcaption>
            <a href="#/{g.owner_type}/{g.owner_id}">{g.owner_name}</a>
            {#if g.caption}<span class="muted"> · {g.caption}</span>{/if}
          </figcaption>
        </figure>
      {/each}
    </div>
    {#if shown.length > limit}
      <div class="more"><button class="btn" onclick={() => (limit += 120)}>{t('Load more')}</button></div>
    {/if}
  </div>
{/if}

<style>
  .filters {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 16px;
  }
  .filters input {
    flex: 1 1 240px;
  }
  .filters select {
    width: auto;
  }
  .masonry {
    columns: 220px;
    column-gap: 12px;
  }
  figure {
    break-inside: avoid;
    margin: 0 0 12px;
    background: var(--surface);
    box-shadow: var(--shadow);
    border-radius: var(--radius);
    overflow: hidden;
  }
  figure button {
    display: block;
    width: 100%;
    border: 0;
    padding: 0;
    background: var(--ink-soft);
    cursor: zoom-in;
  }
  figure button :global(img) {
    width: 100%;
    display: block;
  }
  figure button :global(.ph) {
    width: 100%;
    aspect-ratio: 4 / 3;
    display: block;
  }
  figcaption {
    padding: 7px 10px 8px;
    font-size: 0.83rem;
    line-height: 1.35;
  }
  figcaption a {
    font-weight: 550;
    text-decoration: none;
  }
  .more {
    text-align: center;
    margin-top: 16px;
  }
</style>
