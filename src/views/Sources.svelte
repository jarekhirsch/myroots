<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { fmtDate } from '../lib/format';
  import { norm, stripHtml } from '../lib/text';
  import Img from '../lib/components/Img.svelte';
  import Plus from '../lib/icons/Plus.svelte';
  import FileText from '../lib/icons/FileText.svelte';
  import type { SourceView } from '../lib/types';

  const src = useLoad(() => call<SourceView[]>('sources.list'));
  let q = $state('');
  const shown = $derived.by(() => {
    const w = norm(q).split(' ').filter(Boolean);
    return (src.data || [])
      .filter((s) => !w.length || w.every((x) => norm(`${s.source.title} ${stripHtml(s.source.notes)} ${s.persons.map((p) => p.name).join(' ')}`).includes(x)))
      .sort((a, b) => a.source.title.localeCompare(b.source.title));
  });
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Sources and documents')}</h1>
      {#if src.data}<p class="lead">{tn('%d source', '%d sources', shown.length)}</p>{/if}
    </div>
    <a class="btn primary" href="#/source/new"><Plus size={17} /> {t('Add source')}</a>
  </div>
  <input type="search" class="q" placeholder={t('Search for a source…')} bind:value={q} />
  {#if src.data && !src.data.length}
    <div class="panel empty">
      <h2>{t('No sources yet')}</h2>
      <p>{t('Birth certificates, letters, recordings: add a source and pin it to the facts it confirms.')}</p>
      <a class="btn" href="#/source/new">{t('Add source')}</a>
    </div>
  {:else}
    <div class="list">
      {#each shown as s (s.source.id)}
        <a class="row" href="#/source/{s.source.id}">
          <span class="th">{#if s.source.photo}<Img id={s.source.photo} alt="" />{:else}<FileText size={20} />{/if}</span>
          <span class="grow">
            <span class="title">{s.source.title}</span>
            <span class="sub">{[fmtDate(s.source.date), tn('%d person', '%d people', s.persons.length), s.source.scans.length ? tn('%d scan', '%d scans', s.source.scans.length) : ''].filter(Boolean).join(' · ')}</span>
          </span>
        </a>
      {/each}
    </div>
  {/if}
</div>

<style>
  .q {
    margin-bottom: 14px;
  }
  .th {
    width: 44px;
    height: 44px;
    border-radius: 10px;
    overflow: hidden;
    background: var(--ink-soft);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--ink-2);
    flex-shrink: 0;
  }
  .th :global(img),
  .th :global(.ph) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .title,
  .sub {
    display: block;
  }
</style>
