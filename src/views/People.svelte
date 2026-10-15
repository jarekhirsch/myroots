<script lang="ts">
  import { app, summaries, call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { route } from '../lib/router.svelte';
  import { t, tx, tn, locale } from '../lib/i18n/index.svelte';
  import { year, fmtDateShort } from '../lib/format';
  import { norm, collator } from '../lib/text';
  import Avatar from '../lib/components/Avatar.svelte';
  import UserPlus from '@lucide/svelte/icons/user-plus';
  import FamilyTimeline from '../lib/components/FamilyTimeline.svelte';
  import type { Summary, ClanInfo } from '../lib/types';

  const viewer = $derived(!!app.info?.viewer);
  const people = useLoad(() => summaries());
  const clans = useLoad(() => call<ClanInfo[]>('clans.list'));

  let q = $state(route.query.q || '');
  let clan = $state(route.query.clan || '');
  let gender = $state('');
  let life = $state('');
  let status = $state('');
  let sort = $state(app.settings?.sort_order || 'name_asc');
  let page = $state(1);
  type View = 'timeline' | 'grid' | 'list';
  let view = $state<View>(load('myroots-people-view', 'grid') as View);
  const per = $derived(Math.max(10, app.settings?.people_per_page || 50));

  function load(k: string, d: string) {
    try {
      return localStorage.getItem(k) || d;
    } catch {
      return d;
    }
  }
  function setView(v: View) {
    view = v;
    try {
      localStorage.setItem('myroots-people-view', v);
    } catch {
      /* ignore */
    }
  }

  const filtered = $derived.by(() => {
    const words = norm(q).split(' ').filter(Boolean);
    const coll = collator(locale());
    let list = (people.data || []).filter((p) => !p.anon);
    if (words.length) list = list.filter((p) => words.every((w) => norm(`${p.name} ${p.maiden_name} ${p.birth_place} ${p.death_place} ${p.occupation} ${p.years}`).includes(w)));
    if (clan) list = list.filter((p) => (clan === '-' ? !p.clans.length : p.clans.includes(clan)));
    if (gender) list = list.filter((p) => (gender === 'U' ? p.gender !== 'M' && p.gender !== 'F' : p.gender === gender));
    if (life) list = list.filter((p) => (life === 'living' ? p.living : !p.living));
    if (status) list = list.filter((p) => p.status === status);
    const by = (p: Summary) => year(p.birth_date) ?? 99999;
    const surname = (a: Summary, b: Summary) => coll.compare(a.last_name || a.name, b.last_name || b.name) || coll.compare(a.first_name, b.first_name);
    const sorters: Record<string, (a: Summary, b: Summary) => number> = {
      name_asc: surname,
      name_desc: (a, b) => surname(b, a),
      first_asc: (a, b) => coll.compare(a.name, b.name),
      newest: (a, b) => b.updated - a.updated,
      birth_asc: (a, b) => by(a) - by(b) || surname(a, b),
      birth_desc: (a, b) => (year(b.birth_date) ?? -1) - (year(a.birth_date) ?? -1) || surname(a, b),
    };
    return [...list].sort(sorters[sort] || surname);
  });

  $effect(() => {
    void q;
    void clan;
    void gender;
    void life;
    void status;
    void sort;
    page = 1;
  });

  const pages = $derived(Math.max(1, Math.ceil(filtered.length / per)));
  const shown = $derived(filtered.slice((page - 1) * per, page * per));
  const active = $derived(!!(q || clan || gender || life || status));

  function clear() {
    q = clan = gender = life = status = '';
  }

  function dates(p: Summary) {
    const b = p.birth_date ? `* ${fmtDateShort(p.birth_date)}` : '';
    const d = p.death_date ? `† ${fmtDateShort(p.death_date)}` : '';
    return [b, d].filter(Boolean).join('  ');
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('People')}</h1>
      {#if people.data}<p class="lead">{tn('%d person', '%d people', filtered.length)}</p>{/if}
    </div>
    <div class="actions">
      <div class="seg" role="group" aria-label={t('View')}>
        <button class="btn ghost" aria-pressed={view === 'timeline'} onclick={() => setView('timeline')}>{tx('view', 'Timeline')}</button>
        <button class="btn ghost" aria-pressed={view === 'grid'} onclick={() => setView('grid')}>{tx('view', 'Album')}</button>
        <button class="btn ghost" aria-pressed={view === 'list'} onclick={() => setView('list')}>{t('List')}</button>
      </div>
      {#if !viewer}<a class="btn primary" href="#/person/new"><UserPlus size={17} /> {t('Add person')}</a>{/if}
    </div>
  </div>

  <div class="filters">
    <input type="search" class="q" placeholder={t('Search…')} bind:value={q} aria-label={t('Search…')} />
    {#if clans.data?.length}
      <select bind:value={clan} aria-label={t('Clan')}>
        <option value="">{t('All clans')}</option>
        {#each clans.data as c (c.name)}<option value={c.name}>{c.name} ({c.count})</option>{/each}
        <option value="-">{t('No clan')}</option>
      </select>
    {/if}
    <select bind:value={gender} aria-label={t('Gender')}>
      <option value="">{t('All')}</option>
      <option value="M">{t('Men')}</option>
      <option value="F">{t('Women')}</option>
      <option value="U">{tx('gender', 'Unknown')}</option>
    </select>
    <select bind:value={life} aria-label={t('Living')}>
      <option value="">{t('Living and deceased')}</option>
      <option value="living">{t('Living')}</option>
      <option value="dead">{t('Deceased')}</option>
    </select>
    {#if !viewer}
      <select bind:value={status} aria-label={t('Status')}>
        <option value="">{t('Any status')}</option>
        <option value="published">{t('Published')}</option>
        <option value="draft">{t('Draft')}</option>
        <option value="private">{t('Private')}</option>
      </select>
    {/if}
    {#if view !== 'timeline'}<select bind:value={sort} aria-label={t('Sort by')}>
      <option value="name_asc">{t('Surname A–Z')}</option>
      <option value="name_desc">{t('Surname Z–A')}</option>
      <option value="first_asc">{t('First name A–Z')}</option>
      <option value="birth_asc">{t('Oldest first')}</option>
      <option value="birth_desc">{t('Youngest first')}</option>
      <option value="newest">{t('Recently edited')}</option>
    </select>{/if}
    {#if active}<button class="btn ghost sm" onclick={clear}>{t('Clear filters')}</button>{/if}
  </div>

  {#if people.data && !people.data.length}
    <div class="panel empty">
      <h2>{t('No people yet')}</h2>
      {#if !viewer}<a class="btn primary" href="#/person/new">{t('Add the first person')}</a>{/if}
    </div>
  {:else if people.data && !filtered.length}
    <div class="panel empty">{t('No results.')}</div>
  {:else if view === 'timeline'}
    <section class="panel tlp"><FamilyTimeline only={active ? new Set(filtered.map((p) => String(p.id))) : null} /></section>
  {:else if view === 'list'}
    <div class="list">
      {#each shown as p (p.id)}
        <a class="row" href="#/person/{p.id}">
          <Avatar photo={p.photo} name={p.name} gender={p.gender} size={42} />
          <span class="grow">
            <span class="title name">
              {p.name}
              {#if p.maiden_name}<span class="muted maiden">({t('née %s', p.maiden_name)})</span>{/if}
            </span>
            <span class="sub"><bdi>{dates(p)}</bdi>{#if p.birth_place}<span class="faint"> · {p.birth_place}</span>{/if}</span>
          </span>
          <span class="tags">
            {#each p.clans.slice(0, 2) as c}<span class="tag heirloom">{c}</span>{/each}
            {#if p.status !== 'published'}<span class="tag warn">{p.status === 'draft' ? t('Draft') : t('Private')}</span>{/if}
            {#if p.living && !viewer}<span class="tag ok">{t('Living')}</span>{/if}
          </span>
        </a>
      {/each}
    </div>
  {:else}
    <div class="album">
      {#each shown as p (p.id)}
        <a class="tile" href="#/person/{p.id}">
          <span class="pic">
            <Avatar photo={p.photo} name={p.name} gender={p.gender} fill />
            {#if p.status !== 'published' && !viewer}<span class="tag warn flag">{p.status === 'draft' ? t('Draft') : t('Private')}</span>{/if}
          </span>
          <span class="nm">{p.name}</span>
          <span class="meta"><bdi>{p.years}</bdi>{#if p.maiden_name}<span class="faint"> · {t('née %s', p.maiden_name)}</span>{/if}</span>
          {#if p.clans.length}<span class="meta faint">{p.clans[0]}</span>{/if}
        </a>
      {/each}
    </div>
  {/if}

  {#if pages > 1 && view !== 'timeline'}
    <div class="pager">
      <button class="btn sm" disabled={page <= 1} onclick={() => (page--, scrollTo({ top: 0 }))}>{t('Previous')}</button>
      <span class="small muted">{t('Page %1$d of %2$d', page, pages)}</span>
      <button class="btn sm" disabled={page >= pages} onclick={() => (page++, scrollTo({ top: 0 }))}>{t('Next')}</button>
    </div>
  {/if}
</div>

<style>
  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 20px;
  }
  .filters select {
    width: auto;
    flex: 0 1 auto;
    background-color: var(--surface);
    border-radius: 999px;
    min-height: 40px;
    padding-block: 7px;
    box-shadow: var(--shadow);
  }
  .filters select:hover {
    box-shadow: var(--shadow-raised);
  }
  .filters .q {
    flex: 1 1 240px;
    background: var(--surface);
    border-radius: 999px;
    padding-inline: 18px;
    box-shadow: var(--shadow);
  }
  .seg {
    display: flex;
    gap: 2px;
    background: var(--surface-2);
    border-radius: 999px;
    padding: 3px;
  }
  .seg .btn {
    min-height: 34px;
    padding: 4px 14px;
    font-size: 0.92rem;
  }
  .tlp {
    border-radius: 28px;
    padding: 18px 22px 20px;
  }
  .seg .btn[aria-pressed='true'] {
    background: var(--surface);
    color: var(--ink);
    box-shadow: 0 1px 3px rgba(23, 32, 27, 0.1);
  }
  .title,
  .sub {
    display: block;
  }
  .maiden {
    font-size: 0.86rem;
    font-weight: 400;
  }
  .tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }
  /* Family album: the portrait leads, the caption sits under it. */
  .album {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 26px 18px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    text-decoration: none;
    color: var(--text);
  }
  .pic {
    position: relative;
    display: block;
    container-type: inline-size;
    margin-bottom: 10px;
    border-radius: var(--radius-lg);
    transition: box-shadow 0.15s;
  }
  .tile:hover .pic {
    box-shadow: var(--shadow-raised);
  }
  .flag {
    position: absolute;
    top: 10px;
    inset-inline-start: 10px;
    background: var(--surface);
  }
  .tile .nm {
    font-weight: 620;
    line-height: 1.25;
    letter-spacing: -0.01em;
    overflow-wrap: anywhere;
  }
  .tile .meta {
    font-size: 0.87rem;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .pager {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: center;
    margin-top: 22px;
  }
  @media (max-width: 640px) {
    .tags {
      display: none;
    }
    .filters select {
      flex: 1 1 45%;
    }
    .album {
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 18px 10px;
    }
    .album :global(.av.fill) {
      aspect-ratio: 1;
      border-radius: var(--radius);
    }
    .pic {
      border-radius: var(--radius);
      margin-bottom: 7px;
    }
    .tile .nm {
      font-size: 0.9rem;
    }
    .tile .meta {
      font-size: 0.8rem;
    }
  }
</style>
