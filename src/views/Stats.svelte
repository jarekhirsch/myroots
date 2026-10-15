<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tx, tn, locale } from '../lib/i18n/index.svelte';
  import { fmtNumber } from '../lib/format';
  import BarList from '../lib/components/BarList.svelte';
  import NotFound from './NotFound.svelte';
  import type { Stats, ClanInfo } from '../lib/types';

  let clan = $state('');
  const st = useLoad(() => call<Stats>('stats', { clan }));
  const clans = useLoad(() => call<ClanInfo[]>('clans.list').catch(() => [] as ClanInfo[]));
  const S = $derived(st.data);

  const months = $derived.by(() => {
    const f = new Intl.DateTimeFormat(locale(), { month: 'short' });
    return (S?.months_of_birth || []).map((n, i) => [f.format(new Date(2000, i, 1)), n] as [string, number]);
  });

  function years(v: number | null | undefined) {
    return v == null ? '—' : fmtNumber(v, 1);
  }
</script>

{#if st.error}
  <NotFound private={(st.error as any)?.code === 'private'} />
{:else}
  <div class="page">
    <div class="page-head">
      <h1>{t('Family statistics')}</h1>
      {#if clans.data?.length}
        <select bind:value={clan} aria-label={t('Clan')} class="clan">
          <option value="">{t('Whole family')}</option>
          {#each clans.data as c (c.name)}<option value={c.name}>{t('Clan %s', c.name)}</option>{/each}
        </select>
      {/if}
    </div>

    {#if S}
      {#if !S.total}
        <div class="panel empty">{t('No data.')}</div>
      {:else}
        <div class="tiles">
          <div class="tile"><b>{fmtNumber(S.total, 0)}</b><span>{t('People in the database')}</span></div>
          <div class="tile"><b>{S.females} / {S.males}</b><span>{t('Women / men')}</span></div>
          <div class="tile"><b>{years(S.avg_age)} <small>{S.avg_age != null ? t('years') : ''}</small></b><span>{t('Average age')}</span></div>
          <div class="tile"><b>{fmtNumber(S.marriages, 0)}</b><span>{tx('statistics count', 'Marriages')}</span></div>
          <div class="tile"><b>{S.avg_children == null ? '—' : fmtNumber(S.avg_children, 2)}</b><span>{t('Avg. number of children')}</span></div>
          <div class="tile"><b>{years(S.avg_marriage_age_m)}</b><span>{t('Avg. age at marriage (M)')}</span></div>
          <div class="tile"><b>{years(S.avg_marriage_age_f)}</b><span>{t('Avg. age at marriage (F)')}</span></div>
          <div class="tile"><b>{years(S.avg_gen_gap)} <small>{S.avg_gen_gap != null ? t('years') : ''}</small></b><span>{t('Avg. generation gap')}</span></div>
          {#if S.generations}<div class="tile"><b>{S.generations}</b><span>{t('Generations')}</span></div>{/if}
          {#if S.pets_total}<div class="tile"><b>{S.pets_alive} / {S.pets_dead}</b><span>{t('Pets (living / deceased)')}</span></div>{/if}
          {#if S.oldest}<a class="tile wide" href="#/person/{S.oldest.id}"><b class="nm">{S.oldest.name}</b><span>{t('Oldest person')} · {tn('%d year', '%d years', S.oldest.age)}</span></a>{/if}
          {#if S.youngest}<a class="tile wide" href="#/person/{S.youngest.id}"><b class="nm">{S.youngest.name}</b><span>{t('Youngest person')} · {tn('%d year', '%d years', S.youngest.age)}</span></a>{/if}
        </div>

        <div class="grid-2">
          <section class="panel"><h2>{t('Most popular first names')}</h2><BarList data={S.names_top10} empty={t('No first name data.')} /></section>
          <section class="panel"><h2>{t('Most popular surnames')}</h2><BarList data={S.surnames_top10} empty={t('No surname data.')} hrefFor={(n) => `#/people?q=${encodeURIComponent(n)}`} /></section>
          <section class="panel"><h2>{t('Birth decades')}</h2><BarList columns data={S.birth_decades} empty={t('No birth date data.')} /></section>
          <section class="panel"><h2>{t('Death decades')}</h2><BarList columns data={S.death_decades} empty={t('No death date data.')} /></section>
          <section class="panel"><h2>{t('Age distribution')}</h2><BarList columns data={S.age_groups} empty={t('No age data.')} /></section>
          <section class="panel"><h2>{t('Number of children per person')}</h2><BarList columns data={S.children_dist} empty={t('No data on children.')} /></section>
          <section class="panel"><h2>{t('Most common places of birth')}</h2><BarList data={S.birth_places_top10} empty={t('No place of birth data.')} /></section>
          <section class="panel"><h2>{t('Months of birth')}</h2><BarList columns data={months} empty={t('No birth date data.')} /></section>
          {#if S.occupations_top10.length}
            <section class="panel"><h2>{t('Most common occupations')}</h2><BarList data={S.occupations_top10} /></section>
          {/if}
        </div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .clan {
    width: auto;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 12px;
    margin-bottom: 22px;
  }
  .tile {
    background: var(--surface);
    box-shadow: var(--shadow);
    border-radius: var(--radius);
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 0.86rem;
    color: var(--muted);
    text-decoration: none;
  }
  .tile b {
    font-size: 1.55rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text);
    line-height: 1.15;
  }
  .tile b small {
    font-family: var(--font-ui);
    font-size: 0.8rem;
    color: var(--muted);
    font-weight: 400;
  }
  .tile.wide {
    grid-column: span 2;
  }
  .tile .nm {
    font-size: 1.2rem;
  }
  .grid-2 .panel {
    margin: 0;
  }
  .panel h2 {
    font-size: 0.98rem;
    margin-bottom: 12px;
  }
  @media (max-width: 560px) {
    .tile.wide {
      grid-column: span 1;
    }
  }
</style>
