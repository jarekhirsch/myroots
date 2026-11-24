<script lang="ts">
  import { app, call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { speciesLabel } from '../lib/format';
  import { norm } from '../lib/text';
  import Avatar from '../lib/components/Avatar.svelte';
  import Plus from '../lib/icons/Plus.svelte';
  import type { PetView } from '../lib/types';

  const viewer = $derived(!!app.info?.viewer);
  const pets = useLoad(() => call<PetView[]>('pets.list'));
  let q = $state('');
  let species = $state('');
  const kinds = $derived([...new Set((pets.data || []).map((p) => p.pet.species).filter(Boolean))]);
  const shown = $derived.by(() => {
    const w = norm(q).split(' ').filter(Boolean);
    return (pets.data || [])
      .filter((p) => (!species || p.pet.species === species) && (!w.length || w.every((x) => norm(`${p.pet.name} ${p.pet.breed} ${p.owner?.name || ''}`).includes(x))))
      .sort((a, b) => a.pet.name.localeCompare(b.pet.name));
  });
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Pets')}</h1>
      {#if pets.data}<p class="lead">{tn('%d pet', '%d pets', shown.length)}</p>{/if}
    </div>
    {#if !viewer}<a class="btn primary" href="#/pet/new"><Plus size={17} /> {t('Add pet')}</a>{/if}
  </div>
  <div class="filters">
    <input type="search" placeholder={t('Search for a pet…')} bind:value={q} />
    {#if kinds.length > 1}
      <select bind:value={species}>
        <option value="">{t('All species')}</option>
        {#each kinds as k}<option value={k}>{speciesLabel(k)}</option>{/each}
      </select>
    {/if}
  </div>
  {#if pets.data && !pets.data.length}
    <div class="panel empty">
      <h2>{t('No pets.')}</h2>
      {#if !viewer}<p>{t('Dogs, cats and horses are part of family history too.')}</p><a class="btn" href="#/pet/new">{t('Add pet')}</a>{/if}
    </div>
  {:else}
    <div class="cards">
      {#each shown as p (p.pet.id)}
        <a class="card" href="#/pet/{p.pet.id}">
          <Avatar photo={p.pet.photo} name={p.pet.name} size={88} />
          <span class="nm name">{p.pet.name}</span>
          <span class="small muted">{[speciesLabel(p.pet.species), p.pet.breed].filter(Boolean).join(' · ')}</span>
          {#if p.years}<span class="small faint">{p.years}</span>{/if}
          {#if p.owner}<span class="small owner">{p.owner.name}</span>{/if}
        </a>
      {/each}
    </div>
  {/if}
</div>

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
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 12px;
  }
  .card {
    background: var(--surface);
    box-shadow: var(--shadow);
    border-radius: var(--radius);
    padding: 18px 10px 14px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    text-align: center;
    text-decoration: none;
    color: var(--text);
  }
  .card:hover {
    border-color: var(--heirloom);
  }
  .nm {
    font-weight: 620;
    margin-top: 6px;
  }
  .owner {
    color: var(--ink-2);
  }
</style>
