<script lang="ts">
  import { app, call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { go } from '../lib/router.svelte';
  import { t } from '../lib/i18n/index.svelte';
  import { fmtDate, speciesLabel, genderLabel } from '../lib/format';
  import { confirmDialog, toast, openLightbox } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import Avatar from '../lib/components/Avatar.svelte';
  import Img from '../lib/components/Img.svelte';
  import Html from '../lib/components/Html.svelte';
  import PersonLink from '../lib/components/PersonLink.svelte';
  import NotFound from './NotFound.svelte';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Trash from '@lucide/svelte/icons/trash';
  import type { PetView } from '../lib/types';

  let { id }: { id: number } = $props();
  const viewer = $derived(!!app.info?.viewer);
  const pv = useLoad(() => call<PetView>('pet.get', { id }));
  const pet = $derived(pv.data?.pet);
  const gallery = $derived((pet?.gallery || []).map((g) => ({ media: g.media, caption: g.caption })));

  async function remove() {
    if (!pet || !(await confirmDialog(t('Delete %s?', pet.name), '', t('Delete'), true))) return;
    try {
      await call('pet.delete', { id });
      toast(t('Deleted.'));
      go('/pets', true);
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }
  function femaleMale(g: string) {
    return g === 'F' ? t('She') : g === 'M' ? t('He') : '';
  }
</script>

{#if pv.error}
  <NotFound />
{:else if pv.data && pet}
  <div class="page">
    <header class="hero">
      <button class="ph" onclick={() => pet.photo && openLightbox([{ media: pet.photo, caption: pet.name }, ...gallery])} disabled={!pet.photo}>
        <Avatar photo={pet.photo} name={pet.name} size={120} />
      </button>
      <div>
        <h1 class="name">{pet.name}</h1>
        <p class="muted">{[speciesLabel(pet.species), pet.breed, pv.data.years].filter(Boolean).join(' · ')}</p>
        {#if !viewer}
          <div class="actions">
            <a class="btn primary" href="#/pet/{id}/edit"><Pencil size={16} /> {t('Edit')}</a>
            <button class="btn ghost danger icon" onclick={remove} aria-label={t('Delete')}><Trash size={16} /></button>
          </div>
        {/if}
      </div>
    </header>
    <div class="grid-2">
      <div>
        {#if pet.biography}<section class="block"><h2>{t('Biography')}</h2><div class="panel"><Html html={pet.biography} /></div></section>{/if}
        {#if gallery.length}
          <section class="block">
            <h2>{t('Gallery')}</h2>
            <div class="gal">
              {#each gallery as g, i (g.media)}
                <button onclick={() => openLightbox(gallery, i)}><Img id={g.media} alt={g.caption} /></button>
              {/each}
            </div>
          </section>
        {/if}
        {#if pet.notes}<section class="block"><h2>{t('Notes')}</h2><div class="panel"><Html html={pet.notes} /></div></section>{/if}
      </div>
      <div>
        <section class="block">
          <h2>{t('Information')}</h2>
          <dl class="panel facts">
            {#if pet.species}<dt>{t('Species')}</dt><dd>{speciesLabel(pet.species)}</dd>{/if}
            {#if pet.breed}<dt>{t('Breed')}</dt><dd>{pet.breed}</dd>{/if}
            {#if pet.gender && pet.gender !== 'U'}<dt>{t('Gender')}</dt><dd>{femaleMale(pet.gender) || genderLabel(pet.gender)}</dd>{/if}
            {#if pet.birth_date}<dt>{t('Date of birth')}</dt><dd>{fmtDate(pet.birth_date)}</dd>{/if}
            {#if pet.death_date}<dt>{t('Date of death')}</dt><dd>{fmtDate(pet.death_date)}</dd>{/if}
          </dl>
        </section>
        {#if pv.data.owner}
          <section class="block"><h2>{t('Owner')}</h2><div class="panel"><PersonLink p={pv.data.owner} /></div></section>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .hero {
    display: flex;
    gap: 24px;
    align-items: center;
    margin-bottom: 24px;
  }
  .ph {
    border: 0;
    padding: 0;
    background: none;
    border-radius: 50%;
    cursor: zoom-in;
  }
  .hero h1 {
    margin: 0;
  }
  .facts {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 8px 14px;
    margin: 0;
  }
  .facts dt {
    color: var(--muted);
  }
  .facts dd {
    margin: 0;
  }
  .gal {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 8px;
  }
  .gal button {
    border: 0;
    padding: 0;
    aspect-ratio: 1;
    border-radius: 12px;
    overflow: hidden;
    cursor: zoom-in;
    background: var(--ink-soft);
  }
  .gal :global(img),
  .gal :global(.ph) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  @media (max-width: 640px) {
    .hero {
      flex-direction: column;
      text-align: center;
    }
  }
</style>
