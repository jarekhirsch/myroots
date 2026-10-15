<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { route } from '../lib/router.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { fmtDate, year } from '../lib/format';
  import { norm } from '../lib/text';
  import { openLightbox } from '../lib/ui.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  import Html from '../lib/components/Html.svelte';
  import Img from '../lib/components/Img.svelte';
  import AnniversaryList from '../lib/components/AnniversaryList.svelte';
  import NotFound from './NotFound.svelte';
  import type { FamilyEvent, Anniversary } from '../lib/types';

  let tab = $state(route.query.tab === 'anniversaries' ? 'anniversaries' : 'events');
  let days = $state(30);
  const evs = useLoad(() => call<FamilyEvent[]>('events.list'));
  const ann = useLoad(() => call<Anniversary[]>('anniversaries', { days, limit: 0 }));

  let q = $state('');
  let kind = $state('');
  let from = $state('');
  let to = $state('');
  let limit = $state(100);
  let open = $state<Record<number, boolean>>({});

  const KINDS: Record<string, () => string> = {
    birth: () => t('Birth'),
    death: () => t('Death'),
    marriage: () => t('Marriage'),
    event: () => t('Event'),
  };

  const shown = $derived.by(() => {
    const words = norm(q).split(' ').filter(Boolean);
    const f = Number(from) || -Infinity;
    const tt = Number(to) || Infinity;
    return (evs.data || []).filter((e) => {
      if (kind && e.kind !== kind) return false;
      const y = year(e.date);
      if ((from || to) && (y == null || y < f || y > tt)) return false;
      if (words.length && !words.every((w) => norm(`${e.person_name} ${e.with_name} ${e.name} ${e.place} ${e.description}`).includes(w))) return false;
      return true;
    });
  });

  function title(e: FamilyEvent): string {
    if (e.kind === 'marriage') return t('Marriage to %s', e.with_name || '?');
    if (e.kind === 'event') return e.name || t('Event');
    return KINDS[e.kind]?.() || e.kind;
  }
</script>

{#if evs.error}
  <NotFound private={(evs.error as any)?.code === 'private'} />
{:else}
  <div class="page">
    <div class="page-head"><h1>{t('Events')}</h1></div>
    <div class="tabs" role="tablist">
      <button role="tab" aria-selected={tab === 'events'} onclick={() => (tab = 'events')}>{t('Family events')}</button>
      <button role="tab" aria-selected={tab === 'anniversaries'} onclick={() => (tab = 'anniversaries')}>{t('Upcoming anniversaries')}</button>
    </div>

    {#if tab === 'events'}
      <div class="filters">
        <input type="search" placeholder={t('Search (person, type, description)…')} bind:value={q} />
        <select bind:value={kind}>
          <option value="">{t('All types')}</option>
          {#each Object.keys(KINDS) as k}<option value={k}>{KINDS[k]()}</option>{/each}
        </select>
        <input type="text" inputmode="numeric" class="yr" placeholder={t('From year')} bind:value={from} />
        <input type="text" inputmode="numeric" class="yr" placeholder={t('To year')} bind:value={to} />
      </div>
      <p class="small muted">{tn('%d event', '%d events', shown.length)}</p>
      {#if evs.data && !shown.length}
        <div class="panel empty">{t('No events.')}</div>
      {:else}
        <ol class="tl">
          {#each shown.slice(0, limit) as e, i (i)}
            <li class="ev-{e.kind}">
              <span class="dt">{fmtDate(e.date)}</span>
              <div class="card">
                <a class="who" href="#/person/{e.person}">
                  <Avatar photo={e.photo} name={e.person_name} size={34} />
                  <span>
                    <span class="name pn">{e.person_name}</span>
                    <span class="what">{title(e)}{#if e.place}<span class="muted"> · {e.place}</span>{/if}</span>
                  </span>
                </a>
                {#if e.cause}<div class="small muted">{t('Cause: %s', e.cause)}</div>{/if}
                {#if e.image}
                  <button class="im" onclick={() => openLightbox([{ media: e.image!, caption: title(e) }])}><Img id={e.image} alt="" /></button>
                {/if}
                {#if e.description}
                  {#if open[i]}<Html html={e.description} />{:else}<button class="btn ghost sm lnk" onclick={() => (open[i] = true)}>{t('Show description')}</button>{/if}
                {/if}
              </div>
            </li>
          {/each}
        </ol>
        {#if shown.length > limit}<div class="more"><button class="btn" onclick={() => (limit += 100)}>{t('Load more')}</button></div>{/if}
      {/if}
    {:else}
      <div class="filters">
        <label class="small muted dl">
          {t('Days ahead')}
          <select bind:value={days}>
            {#each [7, 14, 30, 60, 90, 180, 365] as d}<option value={d}>{d}</option>{/each}
          </select>
        </label>
      </div>
      {#if ann.data?.length}
        <AnniversaryList items={ann.data} />
      {:else if ann.data}
        <div class="panel empty">{t('No upcoming anniversaries.')}</div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .filters {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }
  .filters input[type='search'] {
    flex: 1 1 240px;
  }
  .filters select {
    width: auto;
  }
  .yr {
    width: 110px !important;
  }
  .dl {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .tl {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .tl li {
    display: grid;
    grid-template-columns: 130px minmax(0, 1fr);
    gap: 14px;
    padding: 6px 0;
  }
  .dt {
    text-align: end;
    font-size: 0.86rem;
    color: var(--muted);
    padding-top: 10px;
    border-inline-end: 3px solid var(--ev, var(--line));
    padding-inline-end: 12px;
  }
  .card {
    background: var(--surface);
    box-shadow: var(--shadow);
    border-radius: var(--radius);
    padding: 8px 12px;
  }
  .who {
    display: flex;
    gap: 10px;
    align-items: center;
    text-decoration: none;
    color: var(--text);
  }
  .pn,
  .what {
    display: block;
  }
  .pn {
    font-weight: 620;
  }
  .what {
    font-size: 0.88rem;
  }
  .im {
    border: 0;
    padding: 0;
    background: none;
    margin-top: 6px;
    cursor: zoom-in;
  }
  .im :global(img),
  .im :global(.ph) {
    width: 140px;
    height: 96px;
    object-fit: cover;
    border-radius: 10px;
    display: block;
  }
  .lnk {
    padding-inline: 0;
    color: var(--ink-2);
  }
  .more {
    text-align: center;
    margin-top: 14px;
  }
  @media (max-width: 560px) {
    .tl li {
      grid-template-columns: 82px minmax(0, 1fr);
      gap: 10px;
    }
  }
</style>
