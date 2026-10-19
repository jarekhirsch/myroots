<script lang="ts">
  import { app, call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { go } from '../lib/router.svelte';
  import { t, tx, tn } from '../lib/i18n/index.svelte';
  import { fmtDate, genderLabel, relTag, endCauseLabel, speciesLabel, fieldLabel, safeHref } from '../lib/format';
  import { confirmDialog, toast, openLightbox } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import Avatar from '../lib/components/Avatar.svelte';
  import Img from '../lib/components/Img.svelte';
  import Html from '../lib/components/Html.svelte';
  import PersonLink from '../lib/components/PersonLink.svelte';
  import LifelineList from '../lib/components/LifelineList.svelte';
  import MiniMap from '../lib/components/MiniMap.svelte';
  import AudioPlayer from '../lib/components/AudioPlayer.svelte';
  import AgingPlayer from '../lib/components/AgingPlayer.svelte';
  import Film from '@lucide/svelte/icons/film';
  import AddRelative from '../lib/components/AddRelative.svelte';
  import SuggestDialog from '../lib/components/SuggestDialog.svelte';
  import NotFound from './NotFound.svelte';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Plus from '@lucide/svelte/icons/plus';
  import Tree from '@lucide/svelte/icons/network';
  import MapIcon from '@lucide/svelte/icons/map';
  import Link from '@lucide/svelte/icons/link-2';
  import Clock from '@lucide/svelte/icons/rotate-ccw-clock';
  import Trash from '@lucide/svelte/icons/trash';
  import Flag from '@lucide/svelte/icons/message-square-warning';
  import Down from '@lucide/svelte/icons/chevron-down';
  import Printer from '@lucide/svelte/icons/printer';
  import type { Profile, Lifeline, Summary } from '../lib/types';

  let { id }: { id: number } = $props();
  const viewer = $derived(!!app.info?.viewer);
  const prof = useLoad(() => call<Profile>('person.profile', { id }));
  const life = useLoad(() => call<Lifeline>('lifeline', { id }).catch(() => null));

  let adding = $state<null | 'father' | 'mother' | 'spouse' | 'child' | 'sibling'>(null);
  let addMenu = $state(false);
  let suggest = $state(false);
  // Sections of the profile on phones.
  let tab = $state<'life' | 'family' | 'photos' | 'sources'>('life');

  const P = $derived(prof.data);
  const p = $derived(P?.person);
  const schema = $derived(app.settings?.custom_schema || []);

  // Citation numbers: index of each source in the list.
  const srcNo = $derived(new Map((P?.sources || []).map((s, i) => [s.id, i + 1])));
  function cites(field: string): number[] {
    return (p?.citations?.[field] || []).map((s) => srcNo.get(s)).filter((n): n is number => !!n);
  }

  const facts = $derived.by(() => {
    if (!P || !p) return [] as { key: string; label: string; value: string }[];
    const rows: { key: string; label: string; value: string }[] = [];
    const add = (key: string, label: string, value: string | null | undefined) => {
      if (value) rows.push({ key, label, value });
    };
    add('gender', t('Gender'), genderLabel(p.gender));
    add('birth_date', t('Date of birth'), fmtDate(p.birth_date));
    add('birth_place', t('Place of birth'), p.birth_place);
    add('death_date', t('Date of death'), p.death_date ? fmtDate(p.death_date) : p.deceased ? t('date unknown') : '');
    add('death_place', t('Place of death'), p.death_place);
    add('death_cause', t('Cause of death'), p.death_cause);
    if (P.age != null && !P.presumed_dead) add('age', P.living ? t('Age') : t('Age at death'), tn('%d year', '%d years', P.age));
    if (P.presumed_dead && !p.death_date) add('presumed', t('Death'), t('presumed (no date of death)'));
    add('occupation', t('Occupation'), p.occupation);
    add('education', t('Education'), p.education);
    add('faith', t('Religion'), p.faith);
    add('height', t('Height'), p.height);
    add('eye_color', t('Eye color'), p.eye_color);
    add('blood_type', t('Blood type'), p.blood_type);
    return rows;
  });

  const custom = $derived(
    schema
      .map((g) => ({ name: g.name, rows: g.fields.map((f) => ({ label: f.name, value: p?.custom?.[g.id]?.[f.id] || '', type: f.type })).filter((r) => r.value) }))
      .filter((g) => g.rows.length),
  );

  const childrenByUnion = $derived.by(() => {
    if (!P) return { groups: [] as { label: string; kids: Summary[] }[], rest: [] as Summary[] };
    const byId = new Map(P.children.map((c) => [c.id, c]));
    const used = new Set<number>();
    const groups = P.unions
      .filter((u) => u.children.length)
      .map((u) => {
        const kids = u.children.map((c) => byId.get(c)).filter((x): x is Summary => !!x);
        kids.forEach((k) => used.add(k.id));
        return { label: u.partner?.name || u.partner_name || t('Unknown'), kids };
      });
    const rest = P.children.filter((c) => !used.has(c.id));
    return { groups, rest };
  });

  const gallery = $derived((p?.gallery || []).map((g) => ({ media: g.media, caption: g.caption })));
  // group photos of others where this person is marked
  const tagged = $derived(P?.tagged || []);

  function openPhoto() {
    if (!p?.photo) return;
    const items = [{ media: p.photo, caption: P!.summary.name }, ...gallery.filter((g) => g.media !== p!.photo)];
    openLightbox(items, 0);
  }

  async function remove() {
    if (!P) return;
    const ok = await confirmDialog(t('Delete %s?', P.summary.name), t('The person and their relations will be removed. Photos stay in the media library. This cannot be undone.'), t('Delete'), true);
    if (!ok) return;
    try {
      await call('person.delete', { id });
      toast(t('Deleted.'));
      go('/people', true);
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }

  function relLine(u: Profile['unions'][number]): string {
    const parts: string[] = [];
    if (u.union.date) parts.push((u.union.informal ? t('Relationship since') : t('Marriage')) + ' ' + fmtDate(u.union.date));
    if (u.union.place) parts.push(u.union.place);
    if (u.union.end_cause && u.union.end_cause !== 'ongoing') parts.push(endCauseLabel(u.union.end_cause) + (u.union.end_date ? ' ' + fmtDate(u.union.end_date) : ''));
    return parts.join(' · ');
  }

  const sec = $derived(app.info?.sections || {});
</script>

{#if prof.error}
  <NotFound private={(prof.error as any)?.code === 'private'} />
{:else if P && p}
  <div class="page">
    <header class="hero">
      <button class="photo" onclick={openPhoto} disabled={!p.photo} aria-label={t('Enlarge')}>
        <Avatar photo={p.photo} name={P.summary.name} gender={p.gender} size={156} anon={P.summary.anon} />
      </button>
      <div class="id">
        <div class="ident">
        <h1 class="name">{P.summary.name}</h1>
        {#if p.maiden_name}<div class="maiden">{t('née %s', p.maiden_name)}</div>{/if}
        <div class="meta">
          {#if P.summary.years}<bdi class="yrs">{P.summary.years}</bdi>{/if}
          {#if P.age != null && !P.presumed_dead}<span class="age">{tn('%d year', '%d years', P.age)}</span>{/if}
          {#each p.clans as c}<a class="tag heirloom" href="#/people?clan={encodeURIComponent(c)}">{c}</a>{/each}
          {#if !viewer && p.status !== 'published'}<span class="tag warn">{p.status === 'draft' ? t('Draft') : t('Private')}</span>{/if}
          {#if P.censored}<span class="tag">{t('Some details are hidden')}</span>{/if}
        </div>
        </div>
        <div class="actions no-print">
          {#if !viewer}
            <a class="btn primary" href="#/person/{id}/edit"><Pencil size={16} /> {t('Edit')}</a>
            <div class="menu-wrap">
              <button class="btn" onclick={() => (addMenu = !addMenu)} aria-expanded={addMenu}><Plus size={16} /> {t('Add relative')} <Down size={14} /></button>
              {#if addMenu}
                <div class="menu" role="menu">
                  {#if !P.father}<button role="menuitem" onclick={() => ((adding = 'father'), (addMenu = false))}>{t('Father')}</button>{/if}
                  {#if !P.mother}<button role="menuitem" onclick={() => ((adding = 'mother'), (addMenu = false))}>{t('Mother')}</button>{/if}
                  <button role="menuitem" onclick={() => ((adding = 'spouse'), (addMenu = false))}>{t('Spouse or partner')}</button>
                  <button role="menuitem" onclick={() => ((adding = 'child'), (addMenu = false))}>{t('Child')}</button>
                  {#if P.father || P.mother}<button role="menuitem" onclick={() => ((adding = 'sibling'), (addMenu = false))}>{t('Sibling')}</button>{/if}
                </div>
              {/if}
            </div>
          {/if}
          {#if sec.tree !== false}<a class="btn" href="#/tree/{id}"><Tree size={16} /> {t('Family tree')}</a>{/if}
          <span class="icons">
            {#if sec.map}<a class="btn ghost icon" href="#/map/{id}" title={t('Map')} aria-label={t('Map')}><MapIcon size={18} /></a>{/if}
            {#if sec.kinship !== false}<a class="btn ghost icon" href="#/kinship?a={id}" title={t('Kinship')} aria-label={t('Kinship')}><Link size={18} /></a>{/if}
            {#if !viewer || (p.aging?.length || 0) >= 2}<a class="btn ghost icon" href="#/person/{id}/years" title={t('Through the years')} aria-label={t('Through the years')}><Film size={18} /></a>{/if}
            {#if !viewer && P.revisions}<a class="btn ghost icon" href="#/person/{id}/history" title="{t('History')} ({P.revisions})" aria-label={t('History')}><Clock size={18} /></a>{/if}
            <button class="btn ghost icon" onclick={() => print()} aria-label={t('Print / PDF')} title={t('Print / PDF')}><Printer size={18} /></button>
          </span>
          {#if viewer && app.info?.suggestions?.enabled}
            <button class="btn ghost" onclick={() => (suggest = true)}><Flag size={16} /> {t('Suggest a correction')}</button>
          {/if}
          {#if !viewer}<button class="btn ghost danger icon" onclick={remove} aria-label={t('Delete')} title={t('Delete')}><Trash size={16} /></button>{/if}
        </div>
      </div>
    </header>

    <div class="ptabs no-print" role="tablist" aria-label={t('View')}>
      <button role="tab" aria-selected={tab === 'life'} onclick={() => (tab = 'life')}>{tx('profile tab', 'Life')}</button>
      <button role="tab" aria-selected={tab === 'family'} onclick={() => (tab = 'family')}>{t('Family')}</button>
      {#if gallery.length || tagged.length || (p.aging?.length || 0) >= 2}<button role="tab" aria-selected={tab === 'photos'} onclick={() => (tab = 'photos')}>{t('Photos')}</button>{/if}
      {#if P.sources.length}<button role="tab" aria-selected={tab === 'sources'} onclick={() => (tab = 'sources')}>{t('Sources')}</button>{/if}
    </div>

    <div class="cols" data-tab={tab}>
      <div class="main-col">
        {#if p.biography}
          <section class="block tab-life">
            <h2>{t('Biography')}{#each cites('biography') as n}<sup class="cite"><a href="#src-{n}">{n}</a></sup>{/each}</h2>
            <div class="panel"><Html html={p.biography} /></div>
          </section>
        {/if}

        {#if p.recordings?.length}
          <section class="block tab-life voice">
            <h2>{t('Voice')}</h2>
            <div class="panel recs">
              {#each p.recordings as r (r.media)}
                <AudioPlayer media={r.media} title={r.title || t('Recording')} sub={fmtDate(r.date)} />
              {/each}
            </div>
          </section>
        {/if}

        {#if (p.aging?.length || 0) >= 2}
          <section class="block tab-photos years no-print">
            <h2>{t('Through the years')}</h2>
            <AgingPlayer frames={p.aging || []} birth={p.birth_date} name={P.summary.name} audio={p.aging_audio || null} compact />
            <a class="small" href="#/person/{id}/years">{viewer ? t('Open larger') : t('Open and edit')}</a>
          </section>
        {/if}

        {#if life.data && life.data.events.length > 1}
          <section class="block tab-life">
            <h2>{t('Lifeline')}</h2>
            <div class="panel"><LifelineList data={life.data} /></div>
          </section>
        {/if}

        {#if gallery.length}
          <section class="block tab-photos">
            <h2>{t('Gallery')}</h2>
            <div class="gal">
              {#each gallery as g, i (g.media)}
                <button class="g" onclick={() => openLightbox(gallery, i)} aria-label={g.caption || t('Enlarge')}>
                  <Img id={g.media} alt={g.caption} />
                  {#if g.caption}<span class="cap">{g.caption}</span>{/if}
                </button>
              {/each}
            </div>
          </section>
        {/if}

        {#if tagged.length}
          <section class="block tab-photos">
            <h2>{t('Also in photos')} <span class="muted small">({tagged.length})</span></h2>
            <div class="gal">
              {#each tagged as g, i (g.media)}
                <button class="g" onclick={() => openLightbox(tagged, i)} aria-label={g.caption || t('Enlarge')}>
                  <Img id={g.media} alt={g.caption} />
                  {#if g.caption}<span class="cap">{g.caption}</span>{/if}
                </button>
              {/each}
            </div>
          </section>
        {/if}

        {#if p.notes}
          <section class="block tab-life">
            <h2>{t('Notes')}</h2>
            <div class="panel"><Html html={p.notes} /></div>
          </section>
        {/if}

        {#if P.sources.length}
          <section class="block tab-sources">
            <h2>{t('Sources and documents')}</h2>
            <ol class="list srcs">
              {#each P.sources as s, i (s.id)}
                <li class="row" id="src-{i + 1}">
                  <span class="no">{i + 1}</span>
                  {#if s.photo}<Img id={s.photo} class="sthumb" alt="" />{/if}
                  <span class="grow">
                    {#if viewer}<span class="title">{s.title}</span>{:else}<a class="title" href="#/source/{s.id}">{s.title}</a>{/if}
                    <span class="sub">
                      {[fmtDate(s.date), s.fields.map((f) => fieldLabel(f)).join(', ')].filter(Boolean).join(' · ')}
                    </span>
                    {#if safeHref(s.url)}<a class="small" href={safeHref(s.url)} target="_blank" rel="noopener noreferrer">{s.url}</a>{/if}
                  </span>
                  {#if s.scans.length}
                    <button class="btn ghost sm" onclick={() => openLightbox(s.scans.map((m) => ({ media: m, caption: s.title })))}>{tn('%d scan', '%d scans', s.scans.length)}</button>
                  {/if}
                </li>
              {/each}
            </ol>
          </section>
        {/if}
      </div>

      <aside class="side">
        <section class="block tab-life">
          <h2>{t('Basic information')}</h2>
          <dl class="panel facts">
            {#each facts as f (f.key)}
              <dt>{f.label}</dt>
              <dd>{f.value}{#each cites(f.key) as n}<sup class="cite"><a href="#src-{n}">{n}</a></sup>{/each}</dd>
            {:else}
              <dd class="muted">{t('No data.')}</dd>
            {/each}
          </dl>
        </section>

        {#each custom as g (g.name)}
          <section class="block tab-life">
            <h2>{g.name}</h2>
            <dl class="panel facts">
              {#each g.rows as r}
                <dt>{r.label}</dt>
                <dd>
                  {#if r.type === 'url' && safeHref(r.value)}<a href={safeHref(r.value)} target="_blank" rel="noopener noreferrer">{r.value}</a>
                  {:else if r.type === 'date'}{fmtDate(r.value)}
                  {:else}<span class="pre">{r.value}</span>{/if}
                </dd>
              {/each}
            </dl>
          </section>
        {/each}

        {#if sec.map}<div class="tab-life"><MiniMap person={id} /></div>{/if}

        <section class="block tab-family">
          <h2>{t('Family')}</h2>
          <div class="panel fam">
            {#if P.father || P.mother}
              <h3>{t('Parents')}</h3>
              {#if P.father}<PersonLink p={P.father} tag={relTag(p.father_rel)} />{/if}
              {#if P.mother}<PersonLink p={P.mother} tag={relTag(p.mother_rel)} />{/if}
            {/if}

            {#if P.unions.length}
              <h3>{P.unions.length > 1 ? t('Spouses / partners') : P.unions[0].union.informal ? t('Partner') : t('Spouse')}</h3>
              {#each P.unions as u (u.union.id)}
                <div class="union">
                  {#if u.partner}<PersonLink p={u.partner} />{:else}<span class="ext name">{u.partner_name || t('Unknown')}</span>{/if}
                  {#if relLine(u)}<div class="small muted ul">{relLine(u)}</div>{/if}
                </div>
              {/each}
            {/if}

            {#if P.children.length}
              <h3>{t('Children')} <span class="muted small">({P.children.length})</span></h3>
              {#each childrenByUnion.groups as g}
                {#if childrenByUnion.groups.length > 1 || childrenByUnion.rest.length}<div class="with small muted">{t('with %s', g.label)}</div>{/if}
                {#each g.kids as c (c.id)}<PersonLink p={c} />{/each}
              {/each}
              {#each childrenByUnion.rest as c (c.id)}<PersonLink p={c} />{/each}
            {/if}

            {#if P.siblings.length}
              <h3>{t('Siblings')} <span class="muted small">({P.siblings.length})</span></h3>
              {#each P.siblings as s (s.id)}<PersonLink p={s} tag={P.half_siblings.includes(s.id) ? tx('half-sibling tag', 'half') : ''} />{/each}
            {/if}

            {#if P.grandparents.length}
              <h3>{t('Grandparents')}</h3>
              {#each P.grandparents as s (s.id)}<PersonLink p={s} />{/each}
            {/if}

            {#if P.grandchildren.length}
              <h3>{t('Grandchildren')} <span class="muted small">({P.grandchildren.length})</span></h3>
              {#each P.grandchildren as s (s.id)}<PersonLink p={s} />{/each}
            {/if}

            {#if !P.father && !P.mother && !P.unions.length && !P.children.length && !P.siblings.length}
              <p class="muted small">{t('No relatives recorded yet.')}</p>
            {/if}
          </div>
        </section>

        {#if P.pets.length}
          <section class="block tab-family">
            <h2>{t('Pets')}</h2>
            <div class="list">
              {#each P.pets as pet (pet.id)}
                <a class="row" href="#/pet/{pet.id}">
                  <Avatar photo={pet.photo} name={pet.name} size={36} />
                  <span class="grow"><span class="title">{pet.name}</span><span class="sub">{[speciesLabel(pet.species), pet.breed, pet.years].filter(Boolean).join(' · ')}</span></span>
                </a>
              {/each}
            </div>
          </section>
        {/if}
      </aside>
    </div>
  </div>

  {#if adding}
    <AddRelative profile={P} relation={adding} onclose={() => (adding = null)} />
  {/if}
  {#if suggest}
    <SuggestDialog person={id} personName={P.summary.name} onclose={() => (suggest = false)} />
  {/if}
{/if}

<svelte:window onclick={(e) => { if (addMenu && !(e.target as HTMLElement).closest('.menu-wrap')) addMenu = false; }} />

<style>
  .hero {
    display: flex;
    gap: 34px;
    align-items: center;
    margin: 8px 0 36px;
  }
  .photo {
    border: 0;
    padding: 0;
    background: none;
    border-radius: 50px;
    cursor: zoom-in;
    flex-shrink: 0;
    box-shadow: var(--shadow-raised);
  }
  .photo:disabled {
    cursor: default;
    box-shadow: none;
  }
  .id {
    min-width: 0;
    flex: 1;
  }
  .id h1 {
    margin: 0;
    font-size: clamp(2rem, 4vw, 2.9rem);
    letter-spacing: -0.035em;
    line-height: 1.05;
  }
  .maiden {
    color: var(--muted);
    margin-top: 4px;
    font-size: 1.02rem;
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 10px;
    align-items: center;
    color: var(--muted);
    margin: 10px 0 18px;
    font-size: 1.02rem;
  }
  .meta a.tag {
    text-decoration: none;
  }
  .yrs {
    color: var(--text);
    font-size: 1.32rem;
    font-weight: 520;
    letter-spacing: -0.01em;
  }
  .age {
    padding: 2px 10px;
    border-radius: 999px;
    background: var(--surface-2);
    font-size: 0.9rem;
    font-weight: 520;
  }
  .icons {
    display: inline-flex;
    gap: 2px;
  }
  .menu-wrap {
    position: relative;
  }
  .menu {
    position: absolute;
    top: calc(100% + 6px);
    inset-inline-start: 0;
    z-index: 40;
    background: var(--surface);
    border-radius: 16px;
    box-shadow: var(--shadow-raised);
    padding: 6px;
    min-width: 210px;
  }
  .menu button {
    display: block;
    width: 100%;
    text-align: start;
    border: 0;
    background: none;
    font: inherit;
    font-weight: 500;
    padding: 9px 12px;
    border-radius: 10px;
    cursor: pointer;
    color: var(--text);
  }
  .menu button:hover {
    background: var(--surface-2);
  }
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 350px;
    gap: 30px;
    align-items: start;
  }
  .facts {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 0 16px;
    margin: 0;
    font-size: 0.95rem;
    padding-block: 8px;
  }
  .facts dt {
    color: var(--muted);
    padding: 9px 0;
  }
  .facts dd {
    margin: 0;
    min-width: 0;
    overflow-wrap: anywhere;
    padding: 9px 0;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .facts dt:not(:first-of-type),
  .facts dt:not(:first-of-type) + dd {
    border-top: 1px solid var(--line);
  }
  .pre {
    white-space: pre-line;
  }
  .cite {
    font-size: 0.7rem;
    margin-inline-start: 2px;
  }
  .cite a {
    text-decoration: none;
    color: var(--heirloom-text);
    font-weight: 700;
  }
  .fam {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 2px;
    padding: 14px 12px;
  }
  .fam h3 {
    font-size: 0.86rem;
    color: var(--faint);
    margin: 14px 8px 4px;
    font-weight: 550;
  }
  .fam h3:first-child {
    margin-top: 2px;
  }
  .union {
    display: flex;
    flex-direction: column;
    margin-bottom: 4px;
  }
  .ul {
    padding-inline-start: 58px;
    margin-top: -6px;
    margin-bottom: 4px;
  }
  .ext {
    padding: 6px 8px;
  }
  .with {
    margin: 6px 0 0 8px;
  }
  .gal {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 10px;
  }
  .g {
    position: relative;
    border: 0;
    padding: 0;
    background: var(--surface-2);
    border-radius: var(--radius);
    overflow: hidden;
    cursor: zoom-in;
    aspect-ratio: 1;
  }
  .g :global(img),
  .g :global(.ph) {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    transition: transform 0.3s;
  }
  .g:hover :global(img) {
    transform: scale(1.03);
  }
  .cap {
    position: absolute;
    inset-inline: 0;
    bottom: 0;
    padding: 22px 10px 8px;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.6));
    color: #fff;
    font-size: 0.8rem;
    text-align: start;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .srcs {
    margin: 0;
    list-style: none;
  }
  .srcs .no {
    font-weight: 700;
    color: var(--heirloom-text);
    min-width: 18px;
  }
  .srcs .title,
  .srcs .sub {
    display: block;
  }
  .srcs :global(.sthumb) {
    width: 46px;
    height: 46px;
    object-fit: cover;
    border-radius: 10px;
  }
  @media (max-width: 980px) {
    .cols {
      grid-template-columns: minmax(0, 1fr);
    }
    .side {
      order: -1;
    }
  }
  .ptabs {
    display: none;
  }
  @media (max-width: 640px) {
    .hero {
      display: grid;
      grid-template-columns: 84px minmax(0, 1fr);
      gap: 0 16px;
      align-items: center;
      margin-bottom: 18px;
    }
    .id {
      display: contents;
    }
    .id > .actions {
      grid-column: 1 / -1;
      margin-top: 16px;
    }
    .photo,
    .photo :global(.av) {
      width: 84px !important;
      height: 84px !important;
      border-radius: 28px;
    }
    .photo :global(.av) {
      font-size: 1.9rem;
    }
    .id h1 {
      font-size: 1.75rem;
    }
    .yrs {
      font-size: 1.12rem;
    }
    .meta {
      margin: 6px 0 0;
    }
    .side {
      order: 0;
    }
    .ptabs {
      display: flex;
      gap: 2px;
      padding: 4px;
      border-radius: 999px;
      background: var(--surface-2);
      margin: 0 0 20px;
    }
    .ptabs button {
      flex: 1;
      min-height: 38px;
      border: 0;
      border-radius: 999px;
      background: none;
      font: inherit;
      font-size: 0.9rem;
      font-weight: 520;
      color: var(--muted);
      cursor: pointer;
    }
    .ptabs button[aria-selected='true'] {
      background: var(--surface);
      color: var(--text);
      font-weight: 640;
      box-shadow: var(--shadow);
    }
    .cols:not([data-tab='life']) .tab-life,
    .cols:not([data-tab='family']) .tab-family,
    .cols:not([data-tab='photos']) .tab-photos,
    .cols:not([data-tab='sources']) .tab-sources {
      display: none;
    }
  }
  @media print {
    .cols {
      grid-template-columns: minmax(0, 1fr);
    }
    .panel {
      padding: 0;
    }
  }
  .recs {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .years .small {
    display: inline-block;
    margin-top: 8px;
  }
</style>
