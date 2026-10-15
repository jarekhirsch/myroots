<script lang="ts">
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tn, tx, locale } from '../lib/i18n/index.svelte';
  import { toast } from '../lib/ui.svelte';
  import Html from '../lib/components/Html.svelte';
  import QuickSearch from '../lib/components/QuickSearch.svelte';
  import FamilyTimeline from '../lib/components/FamilyTimeline.svelte';
  import UserPlus from '@lucide/svelte/icons/user-plus';
  import FileUp from '@lucide/svelte/icons/file-up';
  import Archive from '@lucide/svelte/icons/archive-restore';
  import Press from '@lucide/svelte/icons/newspaper';
  import Globe from '@lucide/svelte/icons/globe';
  import Shield from '@lucide/svelte/icons/shield-check';
  import type { Overview, Anniversary } from '../lib/types';

  const viewer = $derived(!!app.info?.viewer);
  const sec = $derived(app.info?.sections || {});
  const ov = useLoad(() => call<Overview>('overview'));
  const ann = useLoad(() => call<Anniversary[]>('anniversaries', { days: 45, limit: 3 }));
  const issues = useLoad(() => (app.info?.viewer ? Promise.resolve([]) : call<any[]>('health.issues')));
  let treeName = $state('');
  async function saveName() {
    if (!app.settings) return;
    await call('settings.save', { settings: { ...app.settings, tree_name: treeName.trim() } });
    await refreshInfo();
    toast(t('Saved.'));
  }

  function anLabel(a: Anniversary): string {
    if (a.kind === 'birth') return a.living ? t('%d. birthday', a.years) : t('%d. birth anniversary', a.years);
    if (a.kind === 'death') return t('%d. death anniversary', a.years);
    return t('%d. wedding anniversary', a.years);
  }
  function when(a: Anniversary): string {
    if (a.days_left === 0) return t('today');
    if (a.days_left === 1) return t('tomorrow');
    return tn('in %d day', 'in %d days', a.days_left);
  }
  function dayOf(d: string) {
    return Number(d.slice(8, 10));
  }
  function monthOf(d: string) {
    return new Intl.DateTimeFormat(locale(), { month: 'short' }).format(new Date(Number(d.slice(0, 4)), Number(d.slice(5, 7)) - 1, 1));
  }

  const summary = $derived.by(() => {
    const o = ov.data;
    if (!o) return '';
    const parts = [tn('%d person', '%d people', o.persons)];
    if (o.unions) parts.push(tn('%d marriage or relationship', '%d marriages and relationships', o.unions));
    if (o.media) parts.push(tn('%d photo', '%d photos', o.media));
    return parts.join(', ');
  });
  const title = $derived(viewer ? app.info?.site?.title || app.info?.tree_name || t('Family tree') : ov.data?.tree_name || t('Your family'));
</script>

<div class="page home">
  {#if ov.data?.empty && !viewer}
    <section class="welcome">
      <img src="./icon.svg" alt="" width="68" height="68" />
      <h1>{t('Welcome to MyRoots')}</h1>
      <p class="lead">{t('Your family history, on your own device. Start with one person, or bring in a tree you already have.')}</p>
      <form class="name-form" onsubmit={(e) => { e.preventDefault(); saveName(); }}>
        <input type="text" bind:value={treeName} placeholder={t('Name of the family tree, e.g. The Nowak family')} aria-label={t('Name of the family tree')} />
        <button class="btn" type="submit" disabled={!treeName.trim()}>{t('Save')}</button>
      </form>
      <div class="starts">
        <a class="start main" href="#/person/new">
          <span class="ic"><UserPlus size={24} strokeWidth={1.8} /></span>
          <strong>{t('Add the first person')}</strong>
          <span>{t('Usually yourself. Then add parents, partners and children from the profile.')}</span>
        </a>
        <a class="start" href="#/data?tab=gedcom">
          <span class="ic"><FileUp size={22} strokeWidth={1.8} /></span>
          <strong>{t('Import a GEDCOM file')}</strong>
          <span>{t('From MyHeritage, Ancestry, FamilySearch, Gramps or any other program.')}</span>
        </a>
        <a class="start" href="#/data?tab=wp">
          <span class="ic"><Press size={22} strokeWidth={1.8} /></span>
          <strong>{t('Move from the WordPress plugin')}</strong>
          <span>{t('Import the JSON backup of the MyRoots plugin, with photos and scans.')}</span>
        </a>
        <a class="start" href="#/data?tab=archive">
          <span class="ic"><Archive size={22} strokeWidth={1.8} /></span>
          <strong>{t('Restore a backup')}</strong>
          <span>{t('Open a .myroots file made on another device.')}</span>
        </a>
      </div>
    </section>
  {:else}
    <header class="head">
      <div>
        {#if summary}<p class="sum">{summary}</p>{/if}
        <h1>{title}</h1>
        {#if viewer && app.info?.site?.subtitle}<p class="sub">{app.info.site.subtitle}</p>{/if}
      </div>
      <div class="tools">
        {#if !viewer || sec.list !== false}<div class="search"><QuickSearch placeholder={t('Who are you looking for?')} /></div>{/if}
        {#if !viewer}<a class="btn primary" href="#/person/new"><UserPlus size={17} /> {t('Add person')}</a>{/if}
      </div>
    </header>

    {#if viewer && app.info?.site?.intro}
      <section class="intro"><Html html={app.info.site.intro} /></section>
    {/if}

    {#if !viewer || sec.list !== false || sec.tree !== false}
      <section class="panel timeline" aria-label={tx('timeline', 'Family on a timeline')}>
        <FamilyTimeline />
      </section>
    {/if}

    <div class="tints">
      <section class="tint sand">
        <h2>{t('Coming up')}</h2>
        {#if ann.data?.length}
          <ul>
            {#each ann.data as a (a.kind + a.person + a.date)}
              <li>
                <a href="#/person/{a.person}">
                  <span class="day"><b>{dayOf(a.date)}</b> {monthOf(a.date)}</span>
                  <span class="txt"><bdi>{a.name}{#if a.partner_name} {t('and')} {a.partner_name}{/if}</bdi>, {anLabel(a)}</span>
                  <span class="when">{when(a)}</span>
                </a>
              </li>
            {/each}
          </ul>
          <a class="more" href="#/events?tab=anniversaries">{t('All anniversaries')}</a>
        {:else if ann.data}
          <p>{t('No birthdays or anniversaries in the next weeks.')}</p>
        {/if}
      </section>
      {#if !viewer}
        {#if issues.data?.length}
          <a class="tint sky" href="#/health">
            <span class="ic"><Shield size={20} /></span>
            <strong>{tn('%d thing to check', '%d things to check', issues.data.length)}</strong>
            <span>{t('Dates or relations that look wrong. Open the data check.')}</span>
          </a>
        {/if}
        <a class="tint sage" href="#/website">
          <span class="ic"><Globe size={20} /></span>
          <strong>{t('Share the tree with your family')}</strong>
          <span>{t('Export a read-only family website and put it on any hosting. Living people stay protected.')}</span>
        </a>
        <a class="tint lilac" href="#/data">
          <span class="ic"><Archive size={20} /></span>
          <strong>{t('Make a backup')}</strong>
          <span>{t('One file with everything, photos included. Keep it somewhere safe.')}</span>
        </a>
      {/if}
    </div>
  {/if}
</div>

<style>
  /* ---------- welcome ---------- */
  .welcome {
    max-width: 720px;
    margin: 5vh auto 0;
    text-align: center;
  }
  .welcome img {
    border-radius: 22px;
  }
  .welcome h1 {
    margin: 18px 0 8px;
    font-size: 2.5rem;
  }
  .lead {
    color: var(--muted);
    font-size: 1.08rem;
    max-width: 52ch;
    margin: 0 auto;
  }
  .name-form {
    display: flex;
    gap: 8px;
    max-width: 460px;
    margin: 26px auto 30px;
  }
  .name-form input {
    border-radius: 999px;
    background: var(--surface);
    box-shadow: var(--shadow);
    padding-inline: 18px;
  }
  .starts {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
    text-align: start;
  }
  .start {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 20px;
    background: var(--surface);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow);
    color: var(--text);
    text-decoration: none;
    transition: transform 0.15s, box-shadow 0.15s;
  }
  .start:hover {
    box-shadow: var(--shadow-raised);
  }
  .start .ic {
    width: 44px;
    height: 44px;
    border-radius: 14px;
    background: var(--sage);
    color: var(--ink);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 6px;
  }
  .start.main {
    background: var(--ink);
    color: var(--surface);
  }
  .start.main .ic {
    background: rgba(255, 255, 255, 0.16);
    color: var(--surface);
  }
  .start.main span {
    color: color-mix(in srgb, var(--surface) 82%, transparent);
  }
  .start span {
    color: var(--muted);
    font-size: 0.92rem;
  }

  /* ---------- family ---------- */
  .head {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 18px 24px;
    flex-wrap: wrap;
    margin: 6px 0 22px;
  }
  .head h1 {
    font-size: clamp(2rem, 3.6vw, 2.9rem);
    letter-spacing: -0.04em;
    line-height: 1.02;
    margin: 0;
  }
  .sum,
  .sub {
    color: var(--muted);
    margin: 0 0 6px;
  }
  .sub {
    margin: 8px 0 0;
    font-size: 1.08rem;
  }
  .tools {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
  }
  .search {
    width: 260px;
  }
  .tools .btn {
    min-height: 44px;
  }
  .intro {
    max-width: 70ch;
    margin: 0 0 22px;
  }
  .timeline {
    border-radius: 28px;
    padding: 18px 22px 20px;
  }
  .tints {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 260px), 1fr));
    gap: 14px;
    margin-top: 14px;
  }
  .tint {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 20px 22px;
    border-radius: 24px;
    background: var(--tb);
    color: var(--tt);
    text-decoration: none;
    line-height: 1.45;
  }
  a.tint:hover {
    box-shadow: inset 0 0 0 2px color-mix(in srgb, var(--tt) 22%, transparent);
  }
  .tint strong,
  .tint h2 {
    font-weight: 650;
    font-size: 1.04rem;
    letter-spacing: -0.01em;
    margin: 0;
    color: var(--tt);
  }
  .tint > span:not(.ic) {
    color: color-mix(in srgb, var(--tt) 82%, transparent);
    font-size: 0.94rem;
  }
  .tint .ic {
    display: inline-flex;
    margin-bottom: 4px;
  }
  .sand {
    --tb: var(--p2);
    --tt: var(--p2t);
  }
  .sky {
    --tb: var(--p3);
    --tt: var(--p3t);
  }
  .sage {
    --tb: var(--p1);
    --tt: var(--p1t);
  }
  .lilac {
    --tb: var(--p5);
    --tt: var(--p5t);
  }
  .tint ul {
    list-style: none;
    margin: 4px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .tint li a {
    display: grid;
    grid-template-columns: 4.6em minmax(0, 1fr) auto;
    gap: 10px;
    align-items: baseline;
    padding: 5px 0;
    color: var(--tt);
    text-decoration: none;
    font-size: 0.94rem;
  }
  .tint li a:hover .txt {
    text-decoration: underline;
  }
  .day b {
    font-size: 1.15rem;
    font-weight: 650;
  }
  .when {
    font-size: 0.85rem;
    opacity: 0.8;
    white-space: nowrap;
  }
  .tint .more,
  .tint p {
    color: var(--tt);
    font-size: 0.92rem;
    margin: 4px 0 0;
  }
  @media (max-width: 900px) {
    .tools .search {
      display: none;
    }
  }
  @media (max-width: 640px) {
    .starts {
      grid-template-columns: minmax(0, 1fr);
    }
    .welcome h1 {
      font-size: 2rem;
    }
    .search {
      width: auto;
      flex: 1 1 200px;
    }
    .timeline {
      padding: 14px 12px;
      border-radius: 22px;
    }
  }
</style>
