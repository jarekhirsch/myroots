<script lang="ts">
  // Printable family book: cover, table of contents, one page per person, index of surnames.
  import { app, call } from '../lib/api.svelte';
  import { backend } from '../lib/backend';
  import { t, tn, locale } from '../lib/i18n/index.svelte';
  import { fmtDate, relTag, endCauseLabel, genderLabel } from '../lib/format';
  import { errorText } from '../lib/errors';
  import { toast } from '../lib/ui.svelte';
  import { collator } from '../lib/text';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import Img from '../lib/components/Img.svelte';
  import Html from '../lib/components/Html.svelte';
  import LifelineList from '../lib/components/LifelineList.svelte';
  import NotFound from './NotFound.svelte';
  import Printer from '@lucide/svelte/icons/printer';
  import type { Book, Chapter, Profile, Lifeline, ClanInfo } from '../lib/types';

  const blocked = $derived(app.info?.viewer && app.info?.sections?.list === false);

  let title = $state(app.info?.viewer ? app.info?.site?.title || t('Genealogy book') : app.info?.tree_name || t('Genealogy book'));
  let dedication = $state('');
  let structure = $state('generations');
  let mode = $state('clans');
  let person = $state<number | null>(null);
  let maxGen = $state(0);
  let yearFrom = $state('');
  let yearTo = $state('');
  let selClans = $state<string[]>([]);
  let opts = $state({ photos: true, lifeline: true, sources: false, bio: true, index: true });
  let clans = $state<ClanInfo[]>([]);
  let busy = $state(false);
  let book = $state<null | { chapters: Chapter[]; profiles: Map<number, Profile>; lifelines: Map<number, Lifeline> }>(null);

  $effect(() => {
    call<ClanInfo[]>('clans.list').then((c) => (clans = c)).catch(() => {});
  });

  async function generate() {
    if ((mode === 'ancestors' || mode === 'descendants') && !person) {
      toast(t('Select a starting person.'), 'error');
      return;
    }
    busy = true;
    book = null;
    try {
      const b = await call<Book>('book.build', {
        mode,
        structure,
        person,
        max_gen: Number(maxGen) || 0,
        year_from: Number(yearFrom) || null,
        year_to: Number(yearTo) || null,
        clans: mode === 'clans' ? selClans : [],
      });
      const ids = [...new Set(b.chapters.flatMap((c) => c.persons))];
      const profiles = new Map<number, Profile>();
      const lifelines = new Map<number, Lifeline>();
      for (const id of ids) {
        try {
          profiles.set(id, await call<Profile>('person.profile', { id }));
          if (opts.lifeline) lifelines.set(id, await call<Lifeline>('lifeline', { id }));
        } catch {
          /* hidden person */
        }
      }
      const media = [...profiles.values()].flatMap((p) => [p.person.photo, ...(opts.photos ? p.person.gallery.map((g) => g.media) : [])]).filter((x): x is number => !!x);
      await backend.ensureMedia?.(media);
      book = { chapters: b.chapters.map((c) => ({ ...c, persons: c.persons.filter((p) => profiles.has(p)) })).filter((c) => c.persons.length), profiles, lifelines };
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }

  function chapterTitle(c: Chapter): string {
    switch (c.kind) {
      case 'generation':
        return t('Generation %s', c.title);
      case 'clan':
        return t('Clan %s', c.title);
      case 'no_clan':
        return t('Without an assigned clan');
      case 'family':
        return t('Family: %s', c.names.join(' ' + t('and') + ' '));
      default:
        return t('Other people');
    }
  }

  const index = $derived.by(() => {
    if (!book) return [] as [string, { id: number; name: string; ch: number }[]][];
    const m = new Map<string, { id: number; name: string; ch: number }[]>();
    book.chapters.forEach((c, ci) => {
      for (const id of c.persons) {
        const p = book!.profiles.get(id)!;
        for (const s of new Set([p.person.last_name, p.person.maiden_name].filter(Boolean))) {
          const list = m.get(s) || [];
          list.push({ id, name: p.summary.name, ch: ci + 1 });
          m.set(s, list);
        }
      }
    });
    const coll = collator(locale());
    return [...m.entries()].sort((a, b) => coll.compare(a[0], b[0]));
  });

  const today = new Intl.DateTimeFormat(locale(), { dateStyle: 'long' }).format(new Date());
</script>

{#if blocked}
  <NotFound private />
{:else}
  <div class="page book-page">
    <div class="no-print">
      <div class="page-head">
        <div>
          <h1>{t('Genealogy book')}</h1>
          <p class="lead">{t('A printable book: every person on a separate page. Save it as PDF from the print dialog.')}</p>
        </div>
      </div>
      <div class="panel cfg">
        <div class="form-grid">
          <label class="field"><span class="lbl">{t('Book title')}</span><input type="text" bind:value={title} /></label>
          <label class="field">
            <span class="lbl">{t('Chapters')}</span>
            <select bind:value={structure}>
              <option value="generations">{t('By generation (chronologically)')}</option>
              <option value="clans">{t('By clan')}</option>
              <option value="families">{t('By family branch (family = chapter)')}</option>
            </select>
          </label>
          <label class="field full"><span class="lbl">{t('Dedication (optional)')}</span><textarea rows="2" bind:value={dedication} placeholder={t('e.g. In memory of our ancestors…')}></textarea></label>
          <div class="field full">
            <span class="lbl">{t('Scope')}</span>
            <div class="row-opts">
              <label class="check"><input type="radio" bind:group={mode} value="clans" /> {t('Clans')}</label>
              <label class="check"><input type="radio" bind:group={mode} value="ancestors" /> {t('Ancestors of a person')}</label>
              <label class="check"><input type="radio" bind:group={mode} value="descendants" /> {t('Descendants of a person')}</label>
            </div>
            {#if mode === 'clans'}
              {#if clans.length}
                <div class="row-opts">
                  {#each clans as c (c.name)}<label class="check chip"><input type="checkbox" bind:group={selClans} value={c.name} /> {c.name}</label>{/each}
                </div>
              {/if}
              <span class="hint">{t('If you select nothing, all people will be included.')}</span>
            {:else}
              <div class="form-grid sub">
                <PersonPicker bind:value={person} placeholder={t('Starting person…')} />
                <label class="field"><span class="lbl">{t('Max. generations (0 = no limit):')}</span><input type="number" min="0" max="30" bind:value={maxGen} /></label>
              </div>
            {/if}
          </div>
          <label class="field"><span class="lbl">{t('Born from year:')}</span><input type="text" inputmode="numeric" bind:value={yearFrom} /></label>
          <label class="field"><span class="lbl">{t('to year:')}</span><input type="text" inputmode="numeric" bind:value={yearTo} /></label>
          <div class="field full">
            <span class="lbl">{t('Elements')}</span>
            <div class="row-opts">
              <label class="check chip"><input type="checkbox" bind:checked={opts.photos} /> {t('Photos')}</label>
              <label class="check chip"><input type="checkbox" bind:checked={opts.lifeline} /> {t('Lifeline')}</label>
              <label class="check chip"><input type="checkbox" bind:checked={opts.bio} /> {t('Biographies')}</label>
              <label class="check chip"><input type="checkbox" bind:checked={opts.sources} /> {t('Sources')}</label>
              <label class="check chip"><input type="checkbox" bind:checked={opts.index} /> {t('Index of surnames')}</label>
            </div>
          </div>
        </div>
        <div class="actions go">
          <button class="btn primary" onclick={generate} disabled={busy}>{busy ? t('Generating the book…') : t('Generate book')}</button>
          {#if book}<button class="btn" onclick={() => print()}><Printer size={16} /> {t('Print / save as PDF')}</button>{/if}
        </div>
      </div>
    </div>

    {#if book}
      {#if !book.chapters.length}
        <div class="panel empty">{t('No results')}</div>
      {:else}
        <article class="book">
          <section class="sheet cover">
            <div class="cv">
              <div class="orn" aria-hidden="true">❦</div>
              <h1 class="name">{title}</h1>
              {#if dedication}<p class="ded">{dedication}</p>{/if}
              <p class="muted small">{tn('%d person', '%d people', book.profiles.size)} · {today}</p>
            </div>
          </section>

          <section class="sheet toc">
            <h2>{t('Table of contents')}</h2>
            <ol>
              {#each book.chapters as c, i (i)}
                <li><a href="#ch-{i + 1}">{t('Chapter')} {i + 1}. {chapterTitle(c)}</a> <span class="muted">({tn('%d person', '%d people', c.persons.length)})</span></li>
              {/each}
              {#if opts.index}<li><a href="#index">{t('Index of surnames')}</a></li>{/if}
            </ol>
          </section>

          {#each book.chapters as c, ci (ci)}
            <section class="sheet chapter" id="ch-{ci + 1}">
              <div class="chead">
                <span class="cno">{t('Chapter')} {ci + 1}</span>
                <h2 class="name">{chapterTitle(c)}</h2>
                <p class="muted">{c.persons.map((id) => book!.profiles.get(id)!.summary.name).join(' · ')}</p>
              </div>
            </section>
            {#each c.persons as id (id)}
              {@const P = book.profiles.get(id)!}
              {@const p = P.person}
              <section class="sheet person">
                <header>
                  {#if opts.photos && p.photo}<div class="ph"><Img id={p.photo} thumb={false} alt="" /></div>{/if}
                  <div>
                    <h3 class="name">{P.summary.name}</h3>
                    {#if p.maiden_name}<div class="muted">{t('née %s', p.maiden_name)}</div>{/if}
                    <div class="muted">{P.summary.years}</div>
                  </div>
                </header>
                <dl class="facts">
                  {#if p.gender}<dt>{t('Gender')}</dt><dd>{genderLabel(p.gender)}</dd>{/if}
                  {#if p.birth_date || p.birth_place}<dt>{t('Birth')}</dt><dd>{[fmtDate(p.birth_date), p.birth_place].filter(Boolean).join(', ')}</dd>{/if}
                  {#if p.death_date || p.death_place}<dt>{t('Death')}</dt><dd>{[fmtDate(p.death_date), p.death_place, p.death_cause].filter(Boolean).join(', ')}</dd>{/if}
                  {#if p.occupation}<dt>{t('Occupation')}</dt><dd>{p.occupation}</dd>{/if}
                  {#if p.education}<dt>{t('Education')}</dt><dd>{p.education}</dd>{/if}
                  {#if p.faith}<dt>{t('Religion')}</dt><dd>{p.faith}</dd>{/if}
                  {#if P.father || P.mother}<dt>{t('Parents')}</dt><dd>{[P.father, P.mother].filter(Boolean).map((x) => x!.name).join(', ')}{#if relTag(p.father_rel) || relTag(p.mother_rel)} ({[relTag(p.father_rel), relTag(p.mother_rel)].filter(Boolean).join(', ')}){/if}</dd>{/if}
                  {#each P.unions as u (u.union.id)}
                    <dt>{u.union.informal ? t('Partner') : t('Spouse')}</dt>
                    <dd>{u.partner?.name || u.partner_name}{#if u.union.date || u.union.place} · {[fmtDate(u.union.date), u.union.place].filter(Boolean).join(', ')}{/if}{#if u.union.end_cause && u.union.end_cause !== 'ongoing'} · {endCauseLabel(u.union.end_cause)}{/if}</dd>
                  {/each}
                  {#if P.children.length}<dt>{t('Children')}</dt><dd>{P.children.map((x) => x.name).join(', ')}</dd>{/if}
                </dl>
                {#if opts.bio && p.biography}<div class="bio"><Html html={p.biography} /></div>{/if}
                {#if opts.lifeline && book.lifelines.get(id)?.events.length}
                  <h4>{t('Lifeline')}</h4>
                  <LifelineList ruler={false} data={book.lifelines.get(id)!} />
                {/if}
                {#if opts.photos && p.gallery.length}
                  <div class="gal">
                    {#each p.gallery.slice(0, 6) as g (g.media)}<figure><Img id={g.media} alt={g.caption} />{#if g.caption}<figcaption>{g.caption}</figcaption>{/if}</figure>{/each}
                  </div>
                {/if}
                {#if opts.sources && P.sources.length}
                  <h4>{t('Sources')}</h4>
                  <ol class="srcs">{#each P.sources as s (s.id)}<li>{s.title}{s.date ? ', ' + fmtDate(s.date) : ''}</li>{/each}</ol>
                {/if}
              </section>
            {/each}
          {/each}

          {#if opts.index}
            <section class="sheet idx" id="index">
              <h2>{t('Index of surnames')}</h2>
              <div class="cols">
                {#each index as [surname, list] (surname)}
                  <div class="ie"><strong class="name">{surname}</strong> <span class="muted">{list.map((x) => `${x.name} (${t('Chapter')} ${x.ch})`).join('; ')}</span></div>
                {/each}
              </div>
            </section>
          {/if}
        </article>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .cfg {
    margin-bottom: 26px;
  }
  .row-opts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    margin: 4px 0;
  }
  .chip {
    font-size: 0.9rem;
  }
  .sub {
    margin-top: 8px;
    align-items: end;
  }
  .go {
    margin-top: 18px;
  }
  .book {
    display: flex;
    flex-direction: column;
    gap: 22px;
    align-items: center;
  }
  .sheet {
    background: #fff;
    color: #1d2833;
    width: min(210mm, 100%);
    min-height: 120mm;
    padding: 18mm 18mm;
    box-shadow: var(--shadow-pop);
    border-radius: 3px;
    font-family: var(--font-name);
    --text: #1d2833;
    --muted: #5f6d7a;
    --line: #dde3e8;
    --surface: #fff;
    --ink-soft: rgba(33, 57, 81, 0.08);
  }
  .cover {
    min-height: 260mm;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
  }
  .cover h1 {
    font-size: 2.6rem;
    margin: 10px 0 18px;
  }
  .orn {
    font-size: 2.4rem;
    color: #caa288;
  }
  .ded {
    font-style: italic;
    max-width: 120mm;
    margin: 0 auto 30px;
    white-space: pre-line;
  }
  .toc ol {
    padding-inline-start: 18px;
    line-height: 2;
  }
  .toc a,
  .toc a:visited {
    color: inherit;
    text-decoration: none;
  }
  .chapter {
    min-height: 0;
    display: flex;
    align-items: center;
  }
  .cno {
    font-family: var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    font-size: 0.8rem;
    color: #caa288;
  }
  .chead h2 {
    font-size: 2rem;
    margin: 6px 0 12px;
  }
  .person header {
    display: flex;
    gap: 18px;
    align-items: center;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid #dde3e8;
  }
  .person h3 {
    font-size: 1.7rem;
    margin: 0;
  }
  .ph :global(img) {
    width: 34mm;
    height: 34mm;
    object-fit: cover;
    border-radius: 50%;
  }
  .facts {
    display: grid;
    grid-template-columns: 34mm minmax(0, 1fr);
    gap: 4px 12px;
    font-family: var(--font-ui);
    font-size: 0.92rem;
  }
  .facts dt {
    color: #5f6d7a;
  }
  .facts dd {
    margin: 0;
  }
  .bio {
    margin-top: 14px;
  }
  h4 {
    font-family: var(--font-ui);
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #8c99a5;
    margin: 18px 0 8px;
  }
  .gal {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    margin-top: 14px;
  }
  .gal figure {
    margin: 0;
  }
  .gal :global(img) {
    width: 100%;
    aspect-ratio: 4 / 3;
    object-fit: cover;
    border-radius: 4px;
  }
  .gal figcaption {
    font-size: 0.75rem;
    font-family: var(--font-ui);
    color: #5f6d7a;
  }
  .srcs {
    font-family: var(--font-ui);
    font-size: 0.86rem;
  }
  .idx .cols {
    columns: 2;
    column-gap: 10mm;
    font-size: 0.86rem;
  }
  .ie {
    break-inside: avoid;
    margin-bottom: 6px;
  }
  @media (max-width: 640px) {
    .sheet {
      padding: 22px 18px;
    }
    .cover {
      min-height: 60vh;
    }
    .idx .cols {
      columns: 1;
    }
  }
  @media print {
    .book-page {
      padding: 0 !important;
      max-width: none !important;
    }
    .book {
      display: block;
    }
    .sheet {
      box-shadow: none;
      width: auto;
      padding: 0;
      min-height: 0;
      break-after: page;
    }
    .cover {
      min-height: 240mm;
    }
    .chapter {
      min-height: 200mm;
    }
  }
</style>
