<script lang="ts">
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { route } from '../lib/router.svelte';
  import { t, tx, LANGUAGES, setLanguage, detectLanguage } from '../lib/i18n/index.svelte';
  import { canUpdate, checkUpdates, upd } from '../lib/updates.svelte';
  import { toast } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import { applyAppearance, FONT_KEYS } from '../lib/theme';
  import { slug } from '../lib/backend/util';
  import { MAP_STYLES } from '../lib/basemap';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import DateInput from '../lib/components/DateInput.svelte';
  import { POLAND } from '../lib/history';
  import { REGIONS } from '../lib/regions';
  import PlaceInput from '../lib/components/PlaceInput.svelte';
  import { fmtDate } from '../lib/format';
  import Plus from '../lib/icons/Plus.svelte';
  import X from '../lib/icons/X.svelte';
  import type { Settings, CustomGroup } from '../lib/types';

  let tab = $state(['general', 'appearance', 'fields', 'notifications', 'history'].includes(route.query.tab) ? route.query.tab : 'general');
  let s = $state<Settings>(JSON.parse(JSON.stringify(app.settings)));
  if (!s.history) s.history = { off: [], own: [] };
  function toggleBuiltIn(id: string, on: boolean) {
    const off = new Set(s.history!.off);
    if (on) off.delete(id);
    else off.add(id);
    s.history!.off = [...off];
  }
  function addOwn() {
    s.history!.own = [...s.history!.own, { id: 'own-' + Date.now().toString(36), date: '', title: '', place: '' }];
  }
  let busy = $state(false);
  const dirty = $derived(JSON.stringify(s) !== JSON.stringify(app.settings));

  $effect(() => {
    // live preview of the appearance
    applyAppearance(s.appearance);
  });
  $effect(() => {
    return () => applyAppearance(app.settings?.appearance);
  });

  async function save() {
    busy = true;
    try {
      // drop empty custom fields
      s.custom_schema = s.custom_schema
        .map((g) => ({ ...g, name: g.name.trim(), fields: g.fields.filter((f) => f.name.trim()).map((f) => ({ ...f, name: f.name.trim(), options: f.type === 'select' ? f.options.map((o) => o.trim()).filter(Boolean) : [] })) }))
        .filter((g) => g.name);
      const langChanged = s.language !== app.settings?.language;
      await call('settings.save', { settings: s });
      await refreshInfo();
      s = JSON.parse(JSON.stringify(app.settings));
      if (langChanged) await setLanguage(detectLanguage(s.language || undefined));
      toast(t('Settings saved.'));
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }

  function uid(base: string, taken: string[]) {
    let id = slug(base).replace(/-/g, '_') || 'field';
    let k = id;
    let n = 2;
    while (taken.includes(k)) k = `${id}_${n++}`;
    return k;
  }
  function addGroup() {
    s.custom_schema = [...s.custom_schema, { id: '', name: '', fields: [{ id: '', name: '', type: 'text', options: [] }] }];
  }
  function fixIds(g: CustomGroup) {
    if (!g.id && g.name.trim()) g.id = uid(g.name, s.custom_schema.map((x) => x.id));
    for (const f of g.fields) if (!f.id && f.name.trim()) f.id = uid(f.name, g.fields.map((x) => x.id));
  }

  const COLORS: [keyof Settings['appearance'], () => string, string][] = [
    ['primary', () => t('Main color'), '#1f4a37'],
    ['accent', () => t('Accent color'), '#c49a6c'],
    ['bg', () => t('Background'), '#ffffff'],
    ['text', () => t('Text'), '#121613'],
    ['border', () => t('Borders'), '#e6e9e4'],
  ];
  const TREE_COLORS: [keyof Settings['appearance'], () => string, string][] = [
    ['tree_bg', () => t('Tree background'), '#f6f8f6'],
    ['tree_line', () => t('Lines between people'), '#b9c6bd'],
    ['tree_line_main', () => t('Line to the chosen person'), '#c49a6c'],
  ];
  const BRANCHES: [() => string, string][] = [
    [() => t('Father’s father'), '#3f7cc4'],
    [() => t('Father’s mother'), '#2f9a6b'],
    [() => t('Mother’s father'), '#c9693c'],
    [() => t('Mother’s mother'), '#9b59b6'],
  ];
  function setBranch(i: number, v: string) {
    const b = [...(s.appearance.tree_branches || [])];
    while (b.length < 4) b.push('');
    b[i] = v.trim();
    s.appearance.tree_branches = b;
  }
  const FONTS: Record<string, () => string> = {
    system: () => t('System'),
    serif: () => t('Serif (book)'),
    humanist: () => t('Humanist'),
    rounded: () => t('Rounded'),
    mono: () => t('Monospace'),
  };
  const PRESETS: { name: () => string; ap: Partial<Settings['appearance']> }[] = [
    { name: () => t('Forest (default)'), ap: { primary: '', accent: '', bg: '', text: '', border: '' } },
    { name: () => t('Heirloom'), ap: { primary: '#213951', accent: '#caa288', bg: '#f3f5f7', text: '', border: '' } },
    { name: () => t('Burgundy'), ap: { primary: '#6b2737', accent: '#c8a165', bg: '#f7f3f1', text: '', border: '' } },
    { name: () => t('Slate'), ap: { primary: '#334155', accent: '#0ea5a4', bg: '#f4f6f8', text: '', border: '' } },
  ];
  import AboutDialog from '../lib/components/AboutDialog.svelte';
  import Info from '../lib/icons/Info.svelte';
  let about = $state(false);
</script>

<div class="page">
  <div class="page-head">
    <h1>{t('Settings')}</h1>
    <button type="button" class="btn ghost" onclick={() => (about = true)}><Info size={16} /> {t('About MyRoots')}</button>
  </div>
  <div class="tabs" role="tablist">
    <button role="tab" aria-selected={tab === 'general'} onclick={() => (tab = 'general')}>{t('General')}</button>
    <button role="tab" aria-selected={tab === 'appearance'} onclick={() => (tab = 'appearance')}>{t('Appearance')}</button>
    <button role="tab" aria-selected={tab === 'fields'} onclick={() => (tab = 'fields')}>{t('Custom fields')}</button>
    <button role="tab" aria-selected={tab === 'notifications'} onclick={() => (tab = 'notifications')}>{t('Notifications')}</button>
    <button role="tab" aria-selected={tab === 'history'} onclick={() => (tab = 'history')}>{t('Historical background')}</button>
  </div>

  {#if tab === 'general'}
    <div class="panel">
      <div class="form-grid">
        <label class="field"><span class="lbl">{t('Name of the family tree')}</span><input type="text" bind:value={s.tree_name} /></label>
        <label class="field">
          <span class="lbl">{t('Language')}</span>
          <select bind:value={s.language}>
            <option value="">{t('Same as the system')}</option>
            {#each LANGUAGES as l (l.code)}<option value={l.code}>{l.name}</option>{/each}
          </select>
        </label>
        <label class="field">
          <span class="lbl">{t('Presumed death after age')}</span>
          <input type="number" min="0" max="150" bind:value={s.max_age} />
          <span class="hint">{t('A person older than this without a date of death is treated as deceased (0 = off).')}</span>
        </label>
        <label class="field">
          <span class="lbl">{t('Default order of the people list')}</span>
          <select bind:value={s.sort_order}>
            <option value="name_asc">{t('Surname A–Z')}</option>
            <option value="name_desc">{t('Surname Z–A')}</option>
            <option value="birth_asc">{t('Oldest first')}</option>
            <option value="birth_desc">{t('Youngest first')}</option>
            <option value="newest">{t('Recently edited')}</option>
          </select>
        </label>
        <label class="field"><span class="lbl">{t('People per page')}</span><input type="number" min="10" max="500" bind:value={s.people_per_page} /></label>
        <label class="field">
          <span class="lbl">{t('The tree opens with')}</span>
          <select bind:value={s.tree_start_mode}>
            <option value="person">{t('A chosen person')}</option>
            <option value="oldest">{t('The oldest ancestor')}</option>
            <option value="youngest">{t('The youngest person')}</option>
            <option value="last">{t('The most recently edited person')}</option>
          </select>
        </label>
        {#if s.tree_start_mode === 'person'}
          <div class="field full"><span class="lbl">{t('Starting person')}</span><PersonPicker bind:value={s.tree_start_person} /></div>
        {/if}
      </div>
    </div>
    <div class="panel">
      <h2>{t('Sources')}</h2>
      <div class="radios">
        <label class="check"><input type="radio" name="srcmode" value="facts" checked={(s.sources_mode || 'facts') !== 'general'} onchange={() => (s.sources_mode = 'facts')} /> {t('Next to every piece of information')}</label>
        <p class="hint">{t('In the edit form each fact (birth, death, occupation, events, marriages) has its own Source button with the page or record number, reliability and the text of the record, as in MyHeritage or Gramps.')}</p>
        <label class="check"><input type="radio" name="srcmode" value="general" checked={s.sources_mode === 'general'} onchange={() => (s.sources_mode = 'general')} /> {t('In one list per person')}</label>
        <p class="hint">{t('Sources are linked on the Sources tab of the form, with ticks of the facts each source confirms. Quicker for a few sources per person.')}</p>
      </div>
      <p class="hint">{t('Both ways keep the same data: you can switch at any time.')}</p>
    </div>
    {#if canUpdate()}
      <div class="panel">
        <h2>{t('Updates')}</h2>
        <label class="check"><input type="checkbox" checked={!s.updates_off} onchange={(e) => (s.updates_off = !e.currentTarget.checked)} /> {t('Check for a new version once a day (myroots.cc)')}</label>
        <p class="hint">{t('Only the version number is read from myroots.cc; nothing about your tree is sent. Updates are signed by the author and installed after you agree.')}</p>
        <p class="small muted">{t('Installed version: %s', app.info?.version || '')}</p>
        <button type="button" class="btn sm" onclick={() => checkUpdates(true)} disabled={upd.checking}>{upd.checking ? t('Checking…') : t('Check now')}</button>
      </div>
    {/if}
    <div class="panel">
      <h2>{t('Weather')}</h2>
      <label class="check"><input type="checkbox" checked={!s.weather_off} onchange={(e) => (s.weather_off = !e.currentTarget.checked)} /> {t('Show the weather on the day of events (from Open-Meteo.com)')}</label>
      <p class="hint">{t('For events with an exact date from 1940 on, at places with coordinates. Only the coordinates and the date are sent; the result is kept with the place.')}</p>
    </div>
  {:else if tab === 'appearance'}
    <div class="panel">
      <h2>{t('Theme')}</h2>
      <div class="row-opts">
        <label class="check"><input type="radio" bind:group={s.appearance.theme} value="" /> {t('Same as the system')}</label>
        <label class="check"><input type="radio" bind:group={s.appearance.theme} value="light" /> {t('Light')}</label>
        <label class="check"><input type="radio" bind:group={s.appearance.theme} value="dark" /> {t('Dark')}</label>
      </div>
    </div>
    <div class="panel">
      <h2>{t('Colors')}</h2>
      <div class="presets">
        {#each PRESETS as p}
          <button class="btn sm" onclick={() => (s.appearance = { ...s.appearance, ...p.ap })}>
            <span class="sw" style="background:{p.ap.primary || '#1f4a37'}"></span><span class="sw" style="background:{p.ap.accent || '#c49a6c'}"></span>
            {p.name()}
          </button>
        {/each}
      </div>
      <div class="colors">
        {#each COLORS as [k, label, def] (k)}
          <label class="field">
            <span class="lbl">{label()}</span>
            <span class="cin">
              <input type="color" value={(s.appearance[k] as string) || def} oninput={(e) => ((s.appearance as any)[k] = e.currentTarget.value)} />
              <input type="text" bind:value={s.appearance[k]} placeholder={def} />
            </span>
          </label>
        {/each}
      </div>
      <p class="hint">{t('In the dark theme only the accent color is used, so text stays readable.')}</p>
    </div>
    <div class="panel">
      <h2>{t('Family tree colors')}</h2>
      <div class="colors">
        {#each TREE_COLORS as [k, label, def] (k)}
          <label class="field">
            <span class="lbl">{label()}</span>
            <span class="cin">
              <input type="color" value={(s.appearance[k] as string) || def} oninput={(e) => ((s.appearance as any)[k] = e.currentTarget.value)} />
              <input type="text" bind:value={s.appearance[k]} placeholder={t('From the theme')} />
            </span>
          </label>
        {/each}
      </div>
      <h3 class="sub">{t('Colors of the lines of the grandparents')}</h3>
      <div class="colors">
        {#each BRANCHES as [label, def], i (i)}
          <label class="field">
            <span class="lbl">{label()}</span>
            <span class="cin">
              <input type="color" value={s.appearance.tree_branches?.[i] || def} oninput={(e) => setBranch(i, e.currentTarget.value)} />
              <input type="text" value={s.appearance.tree_branches?.[i] || ''} placeholder={def} onchange={(e) => setBranch(i, e.currentTarget.value)} />
            </span>
          </label>
        {/each}
      </div>
      <p class="hint">{t('The lines of the grandparents are colored in the tree when you choose “Color the lines of the grandparents” above the tree.')}</p>
      <button class="btn sm ghost" onclick={() => (s.appearance = { ...s.appearance, tree_bg: '', tree_line: '', tree_line_main: '', tree_branches: [] })}>{t('Restore the default tree colors')}</button>
    </div>
    <div class="panel">
      <div class="form-grid">
        <label class="field">
          <span class="lbl">{t('Font')}</span>
          <select bind:value={s.appearance.font}>
            <option value="">{t('Default')}</option>
            {#each FONT_KEYS as f}<option value={f}>{FONTS[f]?.() || f}</option>{/each}
          </select>
        </label>
        <label class="field">
          <span class="lbl">{t('Text size')}</span>
          <select bind:value={s.appearance.font_size}>
            <option value="">{t('Default')}</option>
            <option value="small">{t('Small')}</option>
            <option value="medium">{t('Medium')}</option>
            <option value="large">{t('Large')}</option>
            <option value="xlarge">{t('Very large')}</option>
          </select>
        </label>
        <label class="field">
          <span class="lbl">{t('Corners')}</span>
          <select bind:value={s.appearance.radius}>
            <option value="">{t('Default')}</option>
            <option value="none">{t('Square')}</option>
            <option value="small">{t('Slightly rounded')}</option>
            <option value="medium">{t('Rounded')}</option>
            <option value="large">{t('Very rounded')}</option>
          </select>
        </label>
        <label class="field">
          <span class="lbl">{t('Shadows')}</span>
          <select bind:value={s.appearance.shadow}>
            <option value="">{t('Default')}</option>
            <option value="none">{t('None')}</option>
            <option value="soft">{t('Soft')}</option>
            <option value="strong">{t('Strong')}</option>
          </select>
        </label>
        <label class="field">
          <span class="lbl">{t('Map style')}</span>
          <select bind:value={s.appearance.map_style}>
            {#each MAP_STYLES as k}<option value={k}>{({ osm: t('Standard'), light: t('Pale'), dark: t('Night'), topo: t('Terrain') } as Record<string, string>)[k]}</option>{/each}
          </select>
        </label>
      </div>
      <p class="hint">{t('The appearance also applies to the exported family website.')}</p>
    </div>
  {:else if tab === 'fields'}
    <p class="hint top">{t('Add your own fields to the person form, grouped in sections, e.g. “Military service” with “Rank” and “Unit”.')}</p>
    {#each s.custom_schema as g, gi (gi)}
      <div class="panel grp">
        <div class="ghead">
          <label class="field grow"><span class="lbl">{t('Section name')}</span><input type="text" bind:value={g.name} onblur={() => fixIds(g)} /></label>
          <button class="btn ghost icon" onclick={() => (s.custom_schema = s.custom_schema.filter((_, k) => k !== gi))} aria-label={t('Remove')}><X size={16} /></button>
        </div>
        {#each g.fields as f, fi (fi)}
          <div class="frow">
            <input type="text" placeholder={t('Field name')} bind:value={f.name} onblur={() => fixIds(g)} />
            <select bind:value={f.type}>
              <option value="text">{tx('custom field type', 'Text')}</option>
              <option value="textarea">{tx('custom field type', 'Text area')}</option>
              <option value="number">{t('Number')}</option>
              <option value="date">{tx('custom field type', 'Date')}</option>
              <option value="url">{t('Link')}</option>
              <option value="email">{t('E-mail')}</option>
              <option value="select">{tx('custom field type', 'Select list')}</option>
            </select>
            <button class="btn ghost icon" onclick={() => (g.fields = g.fields.filter((_, k) => k !== fi))} aria-label={t('Remove')}><X size={16} /></button>
            {#if f.type === 'select'}
              <input class="opts" type="text" placeholder={t('Options separated by commas')} value={f.options.join(', ')} onchange={(e) => (f.options = e.currentTarget.value.split(','))} />
            {/if}
          </div>
        {/each}
        <button class="btn sm" onclick={() => (g.fields = [...g.fields, { id: '', name: '', type: 'text', options: [] }])}><Plus size={14} /> {t('Add field')}</button>
      </div>
    {/each}
    <button class="btn" onclick={addGroup}><Plus size={16} /> {t('Add section')}</button>
  {:else if tab === 'history' && s.history}
    <div class="panel">
      <h2>{t('Your own events')}</h2>
      <p class="hint">{t('Events of your town, region or family history (a fire, a flood, the resettlement of a village…). With a place, an event is shown only to people connected with that place (or living within 50 km); without one, to everyone alive at the time. Also on the family website.')}</p>
      {#each s.history.own as e, i (e.id)}
        <div class="hrow">
          <div class="hdate"><DateInput bind:value={e.date} /></div>
          <input type="text" class="htitle" bind:value={e.title} placeholder={t('What happened')} aria-label={t('What happened')} />
          <div class="hplace"><PlaceInput bind:value={e.place} /></div>
          <button class="btn ghost icon" onclick={() => (s.history!.own = s.history!.own.filter((_, k) => k !== i))} aria-label={t('Remove')} title={t('Remove')}><X size={16} /></button>
        </div>
      {/each}
      <button class="btn sm" onclick={addOwn}><Plus size={15} /> {t('Add an event')}</button>
    </div>
    <div class="panel">
      <h2>{t('History of Poland')} <span class="muted small">({POLAND.length - s.history.off.length} / {POLAND.length})</span></h2>
      <p class="hint">{t('Built-in events. Untick the ones you do not want to see; to word one differently, untick it and add your own.')} {t('Local events (marked “only”) appear for people whose places on the map lie in that region.')}</p>
      <ul class="builtin">
        {#each POLAND as e (e.date)}
          <li>
            <label class="check">
              <input type="checkbox" checked={!s.history.off.includes(e.date)} onchange={(ev) => toggleBuiltIn(e.date, ev.currentTarget.checked)} />
              <span class="bd">{fmtDate(e.date)}</span>
              <span>{e.title()}{#if e.region && REGIONS[e.region]}<span class="reg"> · {t('only: %s', REGIONS[e.region].label())}</span>{/if}</span>
            </label>
          </li>
        {/each}
      </ul>
      {#if s.history.off.length}<button class="btn sm ghost" onclick={() => (s.history!.off = [])}>{t('Turn all on')}</button>{/if}
    </div>
  {:else if tab === 'notifications'}
    <div class="panel">
      <label class="check"><input type="checkbox" bind:checked={s.notifications.enabled} /> {t('Remind me about birthdays and anniversaries')}</label>
      <label class="field days">
        <span class="lbl">{t('Show upcoming anniversaries for the next')}</span>
        <span class="inl"><input type="number" min="1" max="90" bind:value={s.notifications.days_before} /> {t('days')}</span>
      </label>
      <p class="hint">{t('When the app is opened, it shows a system notification on the day before and on the day of an anniversary.')}</p>
    </div>
  {/if}

  <div class="savebar">
    {#if dirty}<span class="small muted">{t('Unsaved changes')}</span>{/if}
    <button class="btn" disabled={!dirty} onclick={() => (s = JSON.parse(JSON.stringify(app.settings)))}>{t('Undo changes')}</button>
    <button class="btn primary" disabled={!dirty || busy} onclick={save}>{t('Save changes')}</button>
  </div>
</div>

{#if about}<AboutDialog onclose={() => (about = false)} />{/if}

<style>
  .reg {
    color: var(--muted);
    font-size: 0.85rem;
  }
  .hrow {
    display: grid;
    grid-template-columns: minmax(250px, auto) minmax(0, 1.3fr) minmax(0, 1fr) auto;
    gap: 8px;
    align-items: center;
    margin-bottom: 8px;
  }
  .builtin {
    list-style: none;
    margin: 0 0 10px;
    padding: 0;
    columns: 2 420px;
    column-gap: 28px;
  }
  .builtin li {
    break-inside: avoid;
    padding: 3px 0;
  }
  .builtin .check {
    align-items: baseline;
  }
  .bd {
    color: var(--muted);
    font-size: 0.85rem;
    min-width: 9.5em;
    font-variant-numeric: tabular-nums;
  }
  @media (max-width: 700px) {
    .hrow {
      grid-template-columns: minmax(0, 1fr) auto;
    }
    .hdate {
      grid-column: 1 / -1;
    }
  }
  .panel {
    margin-bottom: 14px;
  }
  .panel h2 {
    font-size: 1rem;
  }
  .row-opts {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 20px;
  }
  .presets {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 14px;
  }
  .sw {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    display: inline-block;
    margin-inline-end: -4px;
    box-shadow: 0 0 0 1.5px var(--surface);
  }
  .colors {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: 12px;
  }
  .cin {
    display: flex;
    gap: 6px;
  }
  .cin input[type='color'] {
    width: 42px;
    height: 38px;
    padding: 2px;
    border: 1px solid var(--line-strong);
    border-radius: 12px;
    background: var(--surface);
    flex-shrink: 0;
  }
  .top {
    margin-bottom: 14px;
  }
  .ghead {
    display: flex;
    gap: 8px;
    align-items: flex-end;
    margin-bottom: 12px;
  }
  .grow {
    flex: 1;
  }
  .frow {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 170px auto;
    gap: 8px;
    margin-bottom: 8px;
  }
  .frow .opts {
    grid-column: 1 / -1;
  }
  .days {
    margin-top: 14px;
    max-width: 360px;
  }
  .inl {
    display: flex;
    gap: 8px;
    align-items: center;
    color: var(--text);
  }
  .inl input {
    width: 90px;
  }
  .savebar {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    align-items: center;
    position: sticky;
    bottom: 0;
    padding: 12px 0;
    background: linear-gradient(transparent, var(--paper) 30%);
  }
  @media (max-width: 900px) {
    .savebar {
      bottom: calc(var(--tabbar) + env(safe-area-inset-bottom));
    }
  }
  h3.sub {
    font-size: 1rem;
    margin: 18px 0 8px;
  }
</style>
