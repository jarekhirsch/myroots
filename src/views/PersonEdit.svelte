<script lang="ts">
  import { app, call } from '../lib/api.svelte';
  import { go, back } from '../lib/router.svelte';
  import { t, tx } from '../lib/i18n/index.svelte';
  import { relLabel } from '../lib/format';
  import { toast, confirmDialog } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import DateInput from '../lib/components/DateInput.svelte';
  import RichText from '../lib/components/RichText.svelte';
  import PhotoField from '../lib/components/PhotoField.svelte';
  import GalleryEditor from '../lib/components/GalleryEditor.svelte';
  import RecordingsEditor from '../lib/components/RecordingsEditor.svelte';
  import PlaceInput from '../lib/components/PlaceInput.svelte';
  import ClanInput from '../lib/components/ClanInput.svelte';
  import Plus from '@lucide/svelte/icons/plus';
  import X from '@lucide/svelte/icons/x';
  import Up from '@lucide/svelte/icons/arrow-up';
  import Down from '@lucide/svelte/icons/arrow-down';
  import type { Person, Union, EditModel, SourceView, Summary } from '../lib/types';

  let { id }: { id: number | null } = $props();

  const CITABLE = ['birth_date', 'birth_place', 'death_date', 'death_place', 'occupation', 'faith', 'education', 'biography'];

  function blank(): Person {
    return {
      id: 0, status: 'published', first_name: '', last_name: '', maiden_name: '', gender: '',
      birth_date: '', birth_place: '', death_date: '', death_place: '', death_cause: '', deceased: false, biography: '', notes: '',
      occupation: '', education: '', faith: '', height: '', eye_color: '', blood_type: '',
      mother: null, mother_rel: 'bio', father: null, father_rel: 'bio', events: [], clans: [], photo: null, gallery: [], recordings: [], aging: [], aging_audio: null,
      custom: {}, citations: {}, created: 0, updated: 0,
    };
  }

  interface URow extends Union { external: boolean }

  let person = $state<Person>(blank());
  let unions = $state<URow[]>([]);
  let children = $state<(number | null)[]>([]);
  let sources = $state<number[]>([]);
  let allSources = $state<SourceView[]>([]);
  let loaded = $state(false);
  let tab = $state('basic');
  let busy = $state(false);
  let dirty = $state(false);
  let initial = '';

  $effect(() => {
    const cur = id;
    loaded = false;
    (async () => {
      allSources = await call<SourceView[]>('sources.list');
      if (cur) {
        const m = await call<EditModel>('person.edit', { id: cur });
        person = { ...blank(), ...m.person };
        unions = m.unions.map((u) => {
          const partner = u.union.a === cur ? u.union.b : u.union.a;
          return { ...u.union, a: cur, b: partner, external: !partner };
        });
        children = [...m.children];
        sources = [...m.sources];
      } else {
        person = blank();
        unions = [];
        children = [];
        sources = [];
      }
      initial = snapshot();
      dirty = false;
      loaded = true;
    })().catch((e) => toast(errorText(e), 'error'));
  });

  function snapshot() {
    return JSON.stringify([person, unions, children, sources]);
  }
  $effect(() => {
    if (loaded) dirty = snapshot() !== initial;
  });

  const self = $derived(id ? [id] : []);
  const schema = $derived(app.settings?.custom_schema || []);

  function addUnion() {
    unions = [...unions, { id: 0, a: id || 0, b: null, partner_first_name: '', partner_last_name: '', date: '', place: '', end_cause: 'ongoing', end_date: '', informal: false, external: false }];
  }
  function addEvent() {
    person.events = [...person.events, { name: '', date: '', place: '', description: '', image: null }];
  }
  function moveEvent(i: number, d: number) {
    const j = i + d;
    if (j < 0 || j >= person.events.length) return;
    const e = [...person.events];
    [e[i], e[j]] = [e[j], e[i]];
    person.events = e;
  }

  function custom(g: string, f: string): string {
    return person.custom?.[g]?.[f] ?? '';
  }
  function setCustom(g: string, f: string, v: string) {
    const c = { ...(person.custom || {}) };
    c[g] = { ...(c[g] || {}), [f]: v };
    person.custom = c;
  }

  function cited(src: number, field: string): boolean {
    return (person.citations?.[field] || []).includes(src);
  }
  function toggleCite(src: number, field: string) {
    const c = { ...(person.citations || {}) };
    const list = c[field] || [];
    c[field] = list.includes(src) ? list.filter((x) => x !== src) : [...list, src];
    if (!c[field].length) delete c[field];
    person.citations = c;
  }
  function removeSource(src: number) {
    sources = sources.filter((s) => s !== src);
    const c = { ...(person.citations || {}) };
    for (const k of Object.keys(c)) {
      c[k] = c[k].filter((x) => x !== src);
      if (!c[k].length) delete c[k];
    }
    person.citations = c;
  }
  let addSrc = $state<number | ''>('');
  $effect(() => {
    if (addSrc !== '') {
      if (!sources.includes(addSrc)) sources = [...sources, addSrc];
      addSrc = '';
    }
  });

  async function save() {
    if (!person.first_name.trim() && !person.last_name.trim()) {
      tab = 'basic';
      toast(t('Enter at least a first name or a surname.'), 'error');
      return;
    }
    busy = true;
    try {
      const u: Union[] = unions.map(({ external, ...x }) => ({
        ...x,
        a: id || 0,
        b: external ? null : x.b,
        partner_first_name: external ? x.partner_first_name : '',
        partner_last_name: external ? x.partner_last_name : '',
      }));
      const relations = { unions: u, children: children.filter((c): c is number => !!c) };
      let pid = id;
      if (id) {
        await call('person.update', { person: { ...person, id }, relations });
      } else {
        pid = (await call<{ id: number }>('person.create', { person, relations })).id;
      }
      await call('person.set_sources', { id: pid, sources: [...new Set([...sources, ...Object.values(person.citations || {}).flat()])] });
      dirty = false;
      toast(t('Saved.'));
      go(`/person/${pid}`, !id);
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }

  async function cancel() {
    if (dirty && !(await confirmDialog(t('Discard changes?'), t('The changes you made will be lost.'), t('Discard'), true))) return;
    dirty = false;
    if (id) go(`/person/${id}`);
    else back('/people');
  }

  $effect(() => {
    const h = (e: BeforeUnloadEvent) => {
      if (dirty) e.preventDefault();
    };
    window.addEventListener('beforeunload', h);
    return () => window.removeEventListener('beforeunload', h);
  });

  const tabs = $derived([
    ['basic', t('Basic')],
    ['family', t('Family')],
    ['life', t('Biography and events')],
    ['photos', t('Photos')],
    ['sources', t('Sources')],
    ['more', t('More')],
  ]);

  const srcTitle = (sid: number) => allSources.find((s) => s.source.id === sid)?.source.title || `#${sid}`;
  const fieldNames: Record<string, () => string> = {
    birth_date: () => t('Date of birth'),
    birth_place: () => t('Place of birth'),
    death_date: () => t('Date of death'),
    death_place: () => t('Place of death'),
    occupation: () => t('Occupation'),
    faith: () => t('Religion'),
    education: () => t('Education'),
    biography: () => t('Biography'),
  };
</script>

<div class="page">
  <div class="page-head">
    <h1>{id ? t('Edit person') : t('Add person')}</h1>
  </div>

  {#if loaded}
    <div class="tabs" role="tablist">
      {#each tabs as [k, label] (k)}
        <button role="tab" aria-selected={tab === k} onclick={() => (tab = k)}>{label}</button>
      {/each}
    </div>

    <form id="pform" onsubmit={(e) => { e.preventDefault(); save(); }}>
      {#if tab === 'basic'}
        <div class="panel">
          <div class="form-grid">
            <label class="field"><span class="lbl">{t('First name')}</span><input type="text" bind:value={person.first_name} autocomplete="off" /></label>
            <label class="field"><span class="lbl">{t('Surname')}</span><input type="text" bind:value={person.last_name} autocomplete="off" /></label>
            <label class="field"><span class="lbl">{t('Maiden name')}</span><input type="text" bind:value={person.maiden_name} autocomplete="off" /></label>
            <label class="field">
              <span class="lbl">{t('Gender')}</span>
              <select bind:value={person.gender}>
                <option value="">—</option>
                <option value="M">{t('Male')}</option>
                <option value="F">{t('Female')}</option>
                <option value="U">{tx('gender', 'Unknown')}</option>
              </select>
            </label>
            <label class="field"><span class="lbl">{t('Date of birth')}</span><DateInput bind:value={person.birth_date} /></label>
            <label class="field"><span class="lbl">{t('Place of birth')}</span><PlaceInput bind:value={person.birth_place} /></label>
            <label class="field"><span class="lbl">{t('Date of death')}</span><DateInput bind:value={person.death_date} /></label>
            <label class="field"><span class="lbl">{t('Place of death')}</span><PlaceInput bind:value={person.death_place} /></label>
            <label class="field full"><span class="lbl">{t('Cause of death')}</span><input type="text" bind:value={person.death_cause} /></label>
            {#if !person.death_date}
              <label class="check full"><input type="checkbox" bind:checked={person.deceased} /> {t('Deceased, the date of death is unknown')}</label>
            {/if}
            <label class="field"><span class="lbl">{t('Occupation')}</span><input type="text" bind:value={person.occupation} /></label>
            <label class="field"><span class="lbl">{t('Education')}</span><input type="text" bind:value={person.education} /></label>
            <div class="field full">
              <span class="lbl">{t('Clans')}</span>
              <ClanInput bind:value={person.clans} />
            </div>
          </div>
        </div>
        <div class="panel">
          <span class="lbl-s">{t('Profile photo')}</span>
          <PhotoField bind:value={person.photo} round size={110} />
        </div>
      {:else if tab === 'family'}
        <div class="panel">
          <h2>{t('Parents')}</h2>
          <div class="form-grid">
            <div class="field">
              <span class="lbl">{t('Father')}</span>
              <PersonPicker bind:value={person.father} exclude={self} gender="M" create={{ gender: 'M', last_name: person.last_name }} />
              {#if person.father}
                <select bind:value={person.father_rel} aria-label={t('Relation')}>
                  {#each ['bio', 'adopted', 'foster', 'step'] as r}<option value={r}>{relLabel(r)}</option>{/each}
                </select>
              {/if}
            </div>
            <div class="field">
              <span class="lbl">{t('Mother')}</span>
              <PersonPicker bind:value={person.mother} exclude={self} gender="F" create={{ gender: 'F' }} />
              {#if person.mother}
                <select bind:value={person.mother_rel} aria-label={t('Relation')}>
                  {#each ['bio', 'adopted', 'foster', 'step'] as r}<option value={r}>{relLabel(r)}</option>{/each}
                </select>
              {/if}
            </div>
          </div>
        </div>

        <div class="panel">
          <h2>{t('Spouses / partners')}</h2>
          {#each unions as u, i (i)}
            <div class="union">
              <div class="urow">
                <div class="field grow">
                  <span class="lbl">{u.informal ? t('Partner') : t('Spouse')}</span>
                  {#if u.external}
                    <div class="ext">
                      <input type="text" placeholder={t('First name')} bind:value={u.partner_first_name} />
                      <input type="text" placeholder={t('Surname')} bind:value={u.partner_last_name} />
                    </div>
                  {:else}
                    <PersonPicker bind:value={u.b} exclude={self} create={{ gender: person.gender === 'M' ? 'F' : person.gender === 'F' ? 'M' : '' }} />
                  {/if}
                  <label class="check small"><input type="checkbox" bind:checked={u.external} /> {t('Not in the tree (name only)')}</label>
                </div>
                <button type="button" class="btn ghost icon" onclick={() => (unions = unions.filter((_, k) => k !== i))} aria-label={t('Remove')}><X size={16} /></button>
              </div>
              <div class="form-grid">
                <label class="field"><span class="lbl">{u.informal ? t('Relationship since') : t('Date of marriage')}</span><DateInput bind:value={u.date} /></label>
                <label class="field"><span class="lbl">{t('Place of marriage')}</span><PlaceInput bind:value={u.place} /></label>
                <label class="field">
                  <span class="lbl">{t('End of relationship')}</span>
                  <select bind:value={u.end_cause}>
                    <option value="ongoing">{t('Ongoing')}</option>
                    <option value="widowed">{t('Death of spouse')}</option>
                    <option value="divorced">{t('Divorce')}</option>
                    <option value="other">{t('Other/Unknown')}</option>
                    {#if !['ongoing', 'widowed', 'divorced', 'other', ''].includes(u.end_cause)}<option value={u.end_cause}>{u.end_cause}</option>{/if}
                  </select>
                </label>
                {#if u.end_cause && u.end_cause !== 'ongoing'}
                  <label class="field"><span class="lbl">{t('End date')}</span><DateInput bind:value={u.end_date} /></label>
                {/if}
                <label class="check full"><input type="checkbox" bind:checked={u.informal} /> {t('Informal union (cohabitation)')}</label>
              </div>
            </div>
          {/each}
          <button type="button" class="btn sm" onclick={addUnion}><Plus size={15} /> {t('Add spouse or partner')}</button>
        </div>

        <div class="panel">
          <h2>{t('Children')}</h2>
          <p class="hint">{t('The other parent is set on the child\'s own page.')}</p>
          {#each children as c, i (i)}
            <div class="urow">
              <div class="grow"><PersonPicker bind:value={children[i]} exclude={[...self, ...children.filter((x, k) => x && k !== i) as number[]]} create={{ last_name: person.gender === 'M' ? person.last_name : '' }} /></div>
              <button type="button" class="btn ghost icon" onclick={() => (children = children.filter((_, k) => k !== i))} aria-label={t('Remove')}><X size={16} /></button>
            </div>
          {/each}
          <button type="button" class="btn sm" onclick={() => (children = [...children, null])}><Plus size={15} /> {t('Add child')}</button>
        </div>
      {:else if tab === 'life'}
        <div class="panel">
          <span class="lbl-s">{t('Biography')}</span>
          <RichText bind:value={person.biography} label={t('Biography')} minHeight={200} />
        </div>
        <div class="panel">
          <h2>{t('Events')}</h2>
          <p class="hint">{t('Graduation, moving house, military service… Events appear on the lifeline.')}</p>
          {#each person.events as ev, i (i)}
            <div class="event">
              <div class="form-grid">
                <label class="field"><span class="lbl">{t('Event name')}</span><input type="text" bind:value={ev.name} /></label>
                <label class="field"><span class="lbl">{t('Date')}</span><DateInput bind:value={ev.date} /></label>
                <label class="field full"><span class="lbl">{t('Place')}</span><PlaceInput bind:value={ev.place} /></label>
                <div class="field full"><span class="lbl">{t('Description')}</span><RichText bind:value={ev.description} minHeight={80} label={t('Description')} /></div>
                <div class="field full"><span class="lbl">{t('Image')}</span><PhotoField bind:value={ev.image} size={80} /></div>
              </div>
              <div class="ectl">
                <button type="button" class="btn ghost icon sm" onclick={() => moveEvent(i, -1)} disabled={i === 0} aria-label={t('Move up')}><Up size={15} /></button>
                <button type="button" class="btn ghost icon sm" onclick={() => moveEvent(i, 1)} disabled={i === person.events.length - 1} aria-label={t('Move down')}><Down size={15} /></button>
                <button type="button" class="btn ghost sm danger" onclick={() => (person.events = person.events.filter((_, k) => k !== i))}>{t('Remove event')}</button>
              </div>
            </div>
          {/each}
          <button type="button" class="btn sm" onclick={addEvent}><Plus size={15} /> {t('Add event')}</button>
        </div>
        <div class="panel">
          <span class="lbl-s">{t('Notes')}</span>
          <RichText bind:value={person.notes} label={t('Notes')} />
        </div>
      {:else if tab === 'photos'}
        <div class="panel">
          <span class="lbl-s">{t('Profile photo')}</span>
          <PhotoField bind:value={person.photo} round size={110} />
        </div>
        <div class="panel">
          <h2>{t('Gallery')}</h2>
          <GalleryEditor bind:items={person.gallery} />
        </div>
        <div class="panel">
          <h2>{t('Voice recordings')}</h2>
          <p class="hint">{t('Their voice telling a story, singing or just saying hello. Record now or add a file (MP3, M4A, WAV…).')}</p>
          <RecordingsEditor bind:items={person.recordings} name={[person.first_name, person.last_name].filter(Boolean).join(' ')} />
        </div>
      {:else if tab === 'sources'}
        <div class="panel">
          <h2>{t('Sources and documents')}</h2>
          <p class="hint">{t('Link sources to this person and tick the facts each source confirms.')}</p>
          {#each sources as sid (sid)}
            <div class="src">
              <div class="urow">
                <strong class="grow">{srcTitle(sid)}</strong>
                <button type="button" class="btn ghost icon" onclick={() => removeSource(sid)} aria-label={t('Remove')}><X size={16} /></button>
              </div>
              <div class="cites">
                {#each CITABLE as f}
                  <label class="check small"><input type="checkbox" checked={cited(sid, f)} onchange={() => toggleCite(sid, f)} /> {fieldNames[f]()}</label>
                {/each}
              </div>
            </div>
          {/each}
          <div class="urow addsrc">
            <select bind:value={addSrc} aria-label={t('Add source')}>
              <option value="">{t('Add source')}</option>
              {#each allSources.filter((s) => !sources.includes(s.source.id)) as s (s.source.id)}
                <option value={s.source.id}>{s.source.title}</option>
              {/each}
            </select>
            <a class="btn sm ghost" href="#/source/new">{t('New source')}</a>
          </div>
        </div>
      {:else if tab === 'more'}
        <div class="panel">
          <div class="form-grid">
            <label class="field"><span class="lbl">{t('Religion')}</span><input type="text" bind:value={person.faith} /></label>
            <label class="field"><span class="lbl">{t('Height')}</span><input type="text" bind:value={person.height} /></label>
            <label class="field"><span class="lbl">{t('Eye color')}</span><input type="text" bind:value={person.eye_color} /></label>
            <label class="field"><span class="lbl">{t('Blood type')}</span><input type="text" bind:value={person.blood_type} /></label>
            <label class="field">
              <span class="lbl">{t('Status')}</span>
              <select bind:value={person.status}>
                <option value="published">{t('Published')}</option>
                <option value="draft">{t('Draft')}</option>
                <option value="private">{t('Private')}</option>
              </select>
              <span class="hint">{t('Only published people appear on the family website.')}</span>
            </label>
          </div>
        </div>
        {#each schema as g (g.id)}
          <div class="panel">
            <h2>{g.name}</h2>
            <div class="form-grid">
              {#each g.fields as f (f.id)}
                <label class="field" class:full={f.type === 'textarea'}>
                  <span class="lbl">{f.name}</span>
                  {#if f.type === 'textarea'}
                    <textarea bind:value={() => custom(g.id, f.id), (v) => setCustom(g.id, f.id, v)}></textarea>
                  {:else if f.type === 'select'}
                    <select bind:value={() => custom(g.id, f.id), (v) => setCustom(g.id, f.id, v)}>
                      <option value="">—</option>
                      {#each f.options as o}<option value={o}>{o}</option>{/each}
                    </select>
                  {:else if f.type === 'date'}
                    <DateInput bind:value={() => custom(g.id, f.id), (v) => setCustom(g.id, f.id, v)} />
                  {:else if f.type === 'number'}
                    <input type="text" inputmode="decimal" bind:value={() => custom(g.id, f.id), (v) => setCustom(g.id, f.id, v)} />
                  {:else if f.type === 'url'}
                    <input type="url" bind:value={() => custom(g.id, f.id), (v) => setCustom(g.id, f.id, v)} />
                  {:else if f.type === 'email'}
                    <input type="email" bind:value={() => custom(g.id, f.id), (v) => setCustom(g.id, f.id, v)} />
                  {:else}
                    <input type="text" bind:value={() => custom(g.id, f.id), (v) => setCustom(g.id, f.id, v)} />
                  {/if}
                </label>
              {/each}
            </div>
          </div>
        {/each}
        {#if !schema.length}
          <p class="hint">{t('You can define your own fields (e.g. military service) in Settings → Custom fields.')}</p>
        {/if}
      {/if}
    </form>

    <div class="savebar no-print">
      {#if dirty}<span class="small muted">{t('Unsaved changes')}</span>{/if}
      <button class="btn" type="button" onclick={cancel}>{t('Cancel')}</button>
      <button class="btn primary" type="submit" form="pform" disabled={busy}>{t('Save')}</button>
    </div>
  {/if}
</div>

<style>
  .panel {
    margin-bottom: 14px;
  }
  .panel h2 {
    font-size: 1rem;
    margin-bottom: 12px;
  }
  .lbl-s {
    display: block;
    font-size: 0.86rem;
    color: var(--muted);
    font-weight: 500;
    margin-bottom: 8px;
  }
  .field select {
    margin-top: 6px;
  }
  .union,
  .event,
  .src {
    box-shadow: var(--shadow);
    border-radius: 14px;
    padding: 12px 14px;
    margin-bottom: 12px;
    background: var(--surface-2);
  }
  .urow {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    margin-bottom: 10px;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  .ext {
    display: flex;
    gap: 8px;
  }
  .union .check {
    margin-top: 6px;
  }
  .ectl {
    display: flex;
    gap: 4px;
    margin-top: 10px;
    justify-content: flex-end;
  }
  .cites {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
  }
  .addsrc select {
    max-width: 360px;
  }
  .savebar {
    position: sticky;
    bottom: 0;
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    align-items: center;
    padding: 12px 0;
    background: linear-gradient(transparent, var(--paper) 30%);
    margin-top: 10px;
  }
  @media (max-width: 900px) {
    .savebar {
      bottom: calc(var(--tabbar) + env(safe-area-inset-bottom));
    }
  }
</style>
