<script lang="ts">
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { route } from '../lib/router.svelte';
  import { t, tx, LANGUAGES, setLanguage, detectLanguage } from '../lib/i18n/index.svelte';
  import { toast } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import { applyAppearance, FONT_KEYS } from '../lib/theme';
  import { slug } from '../lib/backend/util';
  import { TILES } from '../lib/geocode';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import Plus from '@lucide/svelte/icons/plus';
  import X from '@lucide/svelte/icons/x';
  import type { Settings, CustomGroup } from '../lib/types';

  let tab = $state(['general', 'appearance', 'fields', 'notifications'].includes(route.query.tab) ? route.query.tab : 'general');
  let s = $state<Settings>(JSON.parse(JSON.stringify(app.settings)));
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
    ['primary', () => t('Main color'), '#1d5a44'],
    ['accent', () => t('Accent color'), '#c49a6c'],
    ['bg', () => t('Background'), '#f6f8f6'],
    ['text', () => t('Text'), '#17201b'],
    ['border', () => t('Borders'), '#e3eae5'],
  ];
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
  import Info from '@lucide/svelte/icons/info';
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
            <span class="sw" style="background:{p.ap.primary || '#1d5a44'}"></span><span class="sw" style="background:{p.ap.accent || '#c49a6c'}"></span>
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
            {#each Object.keys(TILES) as k}<option value={k}>{({ osm: 'OpenStreetMap', light: t('Light'), dark: t('Dark'), topo: t('Terrain') } as Record<string, string>)[k]}</option>{/each}
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
</style>
