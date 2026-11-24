<script lang="ts">
  // Data panel beside the tree: everything important about the selected
  // person without leaving the tree. Relatives are clickable and move the
  // selection in the tree; editors can add relatives right here.
  import { app, call } from '../api.svelte';
  import { useLoad } from '../load.svelte';
  import { t, tx, tn } from '../i18n/index.svelte';
  import { fmtDate, genderLabel, relTag, speciesLabel } from '../format';
  import { stripHtml } from '../text';
  import { openLightbox } from '../ui.svelte';
  import Avatar from './Avatar.svelte';
  import Img from './Img.svelte';
  import Html from './Html.svelte';
  import AddRelative from './AddRelative.svelte';
  import X from '../icons/X.svelte';
  import User from '../icons/User.svelte';
  import Pencil from '../icons/PencilSimple.svelte';
  import Crosshair from '../icons/Crosshair.svelte';
  import Plus from '../icons/Plus.svelte';
  import Up from '../icons/CaretUp.svelte';
  import Down from '../icons/CaretDown.svelte';
  import type { Profile, Summary } from '../types';

  let {
    id,
    onclose = null,
    oncenter = null,
    onselect = null,
  }: { id: number; onclose?: (() => void) | null; oncenter?: (() => void) | null; onselect?: ((id: number) => void) | null } = $props();

  const viewer = $derived(!!app.info?.viewer);
  const prof = useLoad(() => call<Profile>('person.profile', { id }));
  const P = $derived(prof.data && prof.data.person.id === id ? prof.data : null);
  const p = $derived(P?.person);
  let adding = $state<null | 'father' | 'mother' | 'spouse' | 'child' | 'sibling'>(null);
  let bioOpen = $state(false);
  $effect(() => {
    void id;
    bioOpen = false;
  });
  // On phones the panel is a sheet folded to one line, so the tree stays visible.
  const phone = typeof matchMedia === 'function' && matchMedia('(max-width: 640px)').matches;
  let open = $state(!phone);

  const facts = $derived.by(() => {
    if (!P || !p) return [] as { label: string; value: string }[];
    const rows: { label: string; value: string }[] = [];
    const add = (label: string, value: string | null | undefined) => value && rows.push({ label, value });
    add(t('Date of birth'), [fmtDate(p.birth_date), p.birth_place].filter(Boolean).join(', '));
    add(t('Date of death'), [p.death_date ? fmtDate(p.death_date) : p.deceased ? t('date unknown') : '', p.death_place].filter(Boolean).join(', '));
    if (P.age != null && !P.presumed_dead) add(P.living ? t('Age') : t('Age at death'), tn('%d year', '%d years', P.age));
    if (P.presumed_dead && !p.death_date) add(t('Death'), t('presumed (no date of death)'));
    add(t('Cause of death'), p.death_cause);
    add(t('Gender'), genderLabel(p.gender));
    add(t('Occupation'), p.occupation);
    add(t('Education'), p.education);
    add(t('Religion'), p.faith);
    return rows;
  });
  const bioText = $derived(p?.biography ? p.biography : '');
  const bioLong = $derived(stripHtml(bioText).length > 320);
  const gallery = $derived((p?.gallery || []).map((g) => ({ media: g.media, caption: g.caption })));
  const family = $derived.by(() => {
    if (!P) return [] as { title: string; people: { s: Summary; note?: string }[] }[];
    const g: { title: string; people: { s: Summary; note?: string }[] }[] = [];
    const parents = [P.father ? { s: P.father, note: relTag(P.person.father_rel) } : null, P.mother ? { s: P.mother, note: relTag(P.person.mother_rel) } : null].filter((x) => !!x) as { s: Summary; note?: string }[];
    if (parents.length) g.push({ title: t('Parents'), people: parents });
    const partners = P.unions.filter((u) => u.partner).map((u) => ({ s: u.partner!, note: u.union.date ? `${u.union.informal ? t('Relationship since') : t('Marriage')} ${fmtDate(u.union.date)}` : '' }));
    if (partners.length) g.push({ title: partners.length > 1 ? t('Spouses / partners') : P.unions[0]?.union.informal ? t('Partner') : t('Spouse'), people: partners });
    if (P.children.length) g.push({ title: t('Children'), people: P.children.map((s) => ({ s })) });
    if (P.siblings.length) g.push({ title: t('Siblings'), people: P.siblings.map((s) => ({ s })) });
    return g;
  });
  function pick(pid: number) {
    if (onselect) onselect(pid);
  }
</script>

{#if P && p}
  <aside class="td" class:folded={!open} aria-label={P.summary.name}>
    <header>
      <button class="ph" onclick={() => p.photo && openLightbox([{ media: p.photo, caption: P.summary.name }, ...gallery.filter((x) => x.media !== p.photo)])} disabled={!p.photo} aria-label={t('Enlarge')}>
        <Avatar photo={p.photo} name={P.summary.name} gender={p.gender} size={open ? 64 : 44} anon={P.summary.anon} />
      </button>
      <div class="who">
        <a class="name" href="#/person/{id}">{P.summary.name}</a>
        {#if p.maiden_name}<span class="muted small">{t('née %s', p.maiden_name)}</span>{/if}
        <span class="yrs">
          {#if P.summary.years}<bdi>{P.summary.years}</bdi>{/if}
          {#if P.living && !viewer}<span class="tag ok">{t('Living')}</span>{/if}
        </span>
      </div>
      {#if phone}
        <button class="btn ghost icon sm" onclick={() => (open = !open)} aria-expanded={open} aria-label={open ? t('Hide') : t('Show')}>{#if open}<Down size={18} />{:else}<Up size={18} />{/if}</button>
      {/if}
      {#if onclose}<button class="btn ghost icon sm" onclick={onclose} aria-label={t('Close')}><X size={18} /></button>{/if}
    </header>

    {#if open}
      <div class="body">
        <div class="acts">
          <a class="btn sm" href="#/person/{id}"><User size={15} /> {t('Open profile')}</a>
          {#if !viewer}<a class="btn sm" href="#/person/{id}/edit"><Pencil size={15} /> {t('Edit')}</a>{/if}
          {#if oncenter}<button class="btn sm" onclick={oncenter}><Crosshair size={15} /> {tx('tree', 'Around this person')}</button>{/if}
        </div>

        {#if p.clans.length}
          <div class="clans">{#each p.clans as c}<span class="tag heirloom">{c}</span>{/each}</div>
        {/if}

        {#if facts.length}
          <section>
            <h3>{t('Basic information')}</h3>
            <dl>
              {#each facts as f (f.label)}<dt>{f.label}</dt><dd>{f.value}</dd>{/each}
            </dl>
          </section>
        {/if}

        <section>
          <h3>{t('Family')}</h3>
          {#each family as g (g.title)}
            <div class="fg">
              <span class="small muted">{g.title}</span>
              {#each g.people as r (r.s.id)}
                <button class="rel" onclick={() => pick(r.s.id)} disabled={r.s.anon}>
                  <Avatar photo={r.s.photo} name={r.s.name} gender={r.s.gender} size={34} anon={r.s.anon} />
                  <span class="rt">
                    <span class="rn">{r.s.anon ? t('Private') : r.s.name}</span>
                    <span class="small muted">{[r.s.years, r.note].filter(Boolean).join(' · ')}</span>
                  </span>
                </button>
              {/each}
            </div>
          {:else}
            <p class="small muted">{t('No relatives recorded yet.')}</p>
          {/each}
          {#if !viewer && !P.summary.anon}
            <div class="add">
              <span class="small muted"><Plus size={14} /> {t('Add relative')}</span>
              <div class="chips">
                {#if !P.father}<button onclick={() => (adding = 'father')}>{t('Father')}</button>{/if}
                {#if !P.mother}<button onclick={() => (adding = 'mother')}>{t('Mother')}</button>{/if}
                <button onclick={() => (adding = 'spouse')}>{t('Spouse or partner')}</button>
                <button onclick={() => (adding = 'child')}>{t('Child')}</button>
                {#if P.father || P.mother}<button onclick={() => (adding = 'sibling')}>{t('Sibling')}</button>{/if}
              </div>
            </div>
          {/if}
        </section>

        {#if P.pets.length}
          <section>
            <h3>{t('Pets')}</h3>
            {#each P.pets as pet (pet.id)}
              <a class="rel" href="#/pet/{pet.id}">
                <Avatar photo={pet.photo} name={pet.name} size={34} />
                <span class="rt">
                  <span class="rn">{pet.name}</span>
                  <span class="small muted">{[speciesLabel(pet.species), pet.breed, pet.years].filter(Boolean).join(' · ')}</span>
                </span>
              </a>
            {/each}
          </section>
        {/if}

        {#if bioText}
          <section>
            <h3>{t('Biography')}</h3>
            <div class="bio" class:clamp={bioLong && !bioOpen}><Html html={bioText} /></div>
            {#if bioLong}<button class="btn ghost sm more" onclick={() => (bioOpen = !bioOpen)}>{bioOpen ? t('Hide') : t('Show')}</button>{/if}
          </section>
        {/if}

        {#if gallery.length}
          <section>
            <h3>{t('Photos')} <span class="muted small">({gallery.length})</span></h3>
            <div class="gal">
              {#each gallery.slice(0, 6) as g, i (g.media)}
                <button onclick={() => openLightbox(gallery, i)} aria-label={g.caption || t('Enlarge')}><Img id={g.media} alt={g.caption} /></button>
              {/each}
            </div>
          </section>
        {/if}
      </div>
    {/if}
  </aside>
  {#if adding}
    <AddRelative profile={P} relation={adding} onclose={() => (adding = null)} />
  {/if}
{/if}

<style>
  .td {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-radius: 24px;
    box-shadow: var(--shadow-raised);
    overflow: hidden;
  }
  header {
    display: flex;
    gap: 12px;
    align-items: center;
    padding: 16px 14px 12px 16px;
  }
  .ph {
    border: 0;
    padding: 0;
    background: none;
    cursor: zoom-in;
    border-radius: 20px;
  }
  .ph:disabled {
    cursor: default;
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.3;
  }
  .name {
    font-weight: 680;
    font-size: 1.12rem;
    letter-spacing: -0.015em;
    color: var(--text);
    text-decoration: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .name:hover {
    text-decoration: underline;
  }
  .yrs {
    display: flex;
    gap: 8px;
    align-items: center;
    color: var(--muted);
    font-size: 0.95rem;
  }
  .body {
    flex: 1;
    overflow-y: auto;
    padding: 0 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .acts {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .clans {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    margin-top: -6px;
  }
  h3 {
    font-size: 0.95rem;
    margin: 0 0 8px;
  }
  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 6px 14px;
    margin: 0;
    font-size: 0.92rem;
  }
  dt {
    color: var(--muted);
  }
  dd {
    margin: 0;
    font-weight: 560;
  }
  .fg {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-bottom: 8px;
  }
  .rel {
    display: flex;
    align-items: center;
    gap: 10px;
    border: 0;
    background: none;
    font: inherit;
    color: inherit;
    text-align: start;
    padding: 5px 6px;
    margin-inline: -6px;
    border-radius: 12px;
    cursor: pointer;
  }
  .rel:hover:not(:disabled) {
    background: var(--surface-2);
  }
  .rel:disabled {
    cursor: default;
  }
  .rt {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.25;
  }
  .rn {
    font-weight: 600;
    font-size: 0.93rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .add {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 4px;
  }
  .add > span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chips button {
    border: 0;
    border-radius: 999px;
    padding: 6px 12px;
    background: var(--ink-soft);
    color: var(--ink);
    font: inherit;
    font-size: 0.86rem;
    font-weight: 600;
    cursor: pointer;
  }
  .chips button:hover {
    background: var(--ink);
    color: var(--surface);
  }
  .bio {
    font-size: 0.93rem;
    line-height: 1.55;
  }
  .bio.clamp {
    display: -webkit-box;
    -webkit-line-clamp: 6;
    line-clamp: 6;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .more {
    padding-inline: 0;
    margin-top: 2px;
    color: var(--ink-2);
  }
  .gal {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
  }
  .gal button {
    border: 0;
    padding: 0;
    background: var(--surface-2);
    border-radius: 12px;
    overflow: hidden;
    aspect-ratio: 1;
    cursor: zoom-in;
  }
  .gal :global(img),
  .gal :global(.ph) {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .td.folded {
    height: auto;
  }
  @media (max-width: 640px) {
    .td {
      height: auto;
      max-height: 62vh;
    }
  }
  .td.folded header {
    padding: 10px 10px 10px 12px;
  }
</style>
