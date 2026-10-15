<script lang="ts">
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { backend } from '../lib/backend';
  import { t, LANGUAGES } from '../lib/i18n/index.svelte';
  import { toast } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import RichText from '../lib/components/RichText.svelte';
  import Globe from '@lucide/svelte/icons/globe';
  import type { Settings } from '../lib/types';

  let s = $state<Settings>(JSON.parse(JSON.stringify(app.settings)));
  let busy = $state(false);
  let saving = $state(false);
  const dirty = $derived(JSON.stringify(s) !== JSON.stringify(app.settings));

  const SECTIONS: [string, () => string][] = [
    ['profile', () => t('Person profiles')],
    ['list', () => t('People list')],
    ['tree', () => t('Family tree')],
    ['map', () => t('Map')],
    ['stats', () => t('Statistics')],
    ['gallery', () => t('Gallery')],
  ];
  const FIELDS: [string, () => string][] = [
    ['dates', () => t('Exact dates (keep only the year)')],
    ['places', () => t('Places (birth / death / marriage)')],
    ['bio_events', () => t('Biography and events')],
    ['photos_sensitive', () => t('Photos and sensitive data (height, eyes, blood, religion)')],
  ];

  async function save() {
    saving = true;
    try {
      await call('settings.save', { settings: s });
      await refreshInfo();
      s = JSON.parse(JSON.stringify(app.settings));
      toast(t('Settings saved.'));
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      saving = false;
    }
  }

  async function exportSite() {
    if (dirty) await save();
    busy = true;
    try {
      const where = await backend.exportSite();
      if (where) toast(t('Website saved: %s', where));
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Family website')}</h1>
      <p class="lead">{t('A read-only copy of the tree for your family: a folder of files that works on any web hosting, or opened straight from a disk or USB stick.')}</p>
    </div>
    <button class="btn primary" onclick={exportSite} disabled={busy}><Globe size={16} /> {busy ? t('Preparing…') : t('Export website (.zip)')}</button>
  </div>

  <div class="grid-2">
    <div>
      <section class="panel">
        <h2>{t('Content')}</h2>
        <div class="form-grid">
          <label class="field full"><span class="lbl">{t('Title')}</span><input type="text" bind:value={s.site.title} placeholder={s.tree_name} /></label>
          <label class="field full"><span class="lbl">{t('Subtitle')}</span><input type="text" bind:value={s.site.subtitle} /></label>
          <div class="field full"><span class="lbl">{t('Welcome text')}</span><RichText bind:value={s.site.intro} minHeight={100} label={t('Welcome text')} /></div>
          <label class="field">
            <span class="lbl">{t('Language')}</span>
            <select bind:value={s.site.language}>
              <option value="">{t('Visitor’s browser language')}</option>
              {#each LANGUAGES as l (l.code)}<option value={l.code}>{l.name}</option>{/each}
            </select>
          </label>
          <label class="check full"><input type="checkbox" bind:checked={s.site.full_photos} /> {t('Include full-size photos (otherwise only smaller copies; the website is lighter)')}</label>
        </div>
      </section>

      <section class="panel">
        <h2>{t('Suggestions from the family')}</h2>
        <label class="check"><input type="checkbox" bind:checked={s.suggestions.enabled} /> {t('Show “Suggest a correction” on profiles')}</label>
        {#if s.suggestions.enabled}
          <div class="form-grid mt">
            <label class="field full">
              <span class="lbl">{t('Your e-mail address')}</span>
              <input type="email" bind:value={s.suggestions.email} />
              <span class="hint">{t('The visitor’s e-mail program opens with the suggestion and a code. Paste the e-mail into Suggestions to import it.')}</span>
            </label>
            <label class="field full">
              <span class="lbl">{t('Form service address (optional)')}</span>
              <input type="url" bind:value={s.suggestions.endpoint} placeholder="https://" />
              <span class="hint">{t('Any service accepting a JSON POST (e.g. Formspree). Used instead of e-mail when it answers.')}</span>
            </label>
          </div>
        {/if}
      </section>
    </div>

    <div>
      <section class="panel">
        <h2>{t('Privacy')}</h2>
        <label class="check"><input type="checkbox" bind:checked={s.privacy.enabled} /> {t('Protect living people on the website')}</label>
        {#if s.privacy.enabled}
          <div class="lv">
            <label class="check"><input type="radio" bind:group={s.privacy.level} value="censor_living" /> <span><strong>{t('Censor data of living people')}</strong> {t('— people are visible, but selected fields of living people are hidden.')}</span></label>
            <label class="check"><input type="radio" bind:group={s.privacy.level} value="hide_living" /> <span><strong>{t('Hide living people')}</strong> {t('— sections are visible, but living people are omitted entirely.')}</span></label>
            <label class="check"><input type="radio" bind:group={s.privacy.level} value="full" /> <span><strong>{t('Full lock of selected sections')}</strong></span></label>
          </div>
          {#if s.privacy.level === 'censor_living'}
            <h3>{t('Censored fields of living people')}</h3>
            {#each FIELDS as [k, label] (k)}<label class="check"><input type="checkbox" bind:group={s.privacy.fields} value={k} /> {label()}</label>{/each}
          {:else if s.privacy.level === 'full'}
            <h3>{t('Locked sections')}</h3>
            {#each SECTIONS as [k, label] (k)}<label class="check"><input type="checkbox" bind:group={s.privacy.sections} value={k} /> {label()}</label>{/each}
          {/if}
        {/if}
        <p class="hint">{t('Only records with the status “Published” are exported. Drafts, private records and suggestions from the family never leave the app.')}</p>
      </section>

      <section class="panel">
        <h2>{t('Map on the website')}</h2>
        <label class="check"><input type="checkbox" bind:checked={s.map_external} /> {t('Use OpenStreetMap (map tiles are loaded from its servers)')}</label>
      </section>

      <section class="panel how">
        <h2>{t('How to publish')}</h2>
        <ol>
          <li>{t('Export the website and unpack the ZIP file.')}</li>
          <li>{t('Upload the folder to your hosting (FTP), or drag it onto a free static host such as Netlify Drop, Cloudflare Pages or GitHub Pages.')}</li>
          <li>{t('Or simply give the folder to the family: index.html opens in any browser, even without internet.')}</li>
        </ol>
        <p class="hint">{t('After changes in the tree, export again and replace the files.')}</p>
      </section>
    </div>
  </div>

  {#if dirty}
    <div class="savebar">
      <button class="btn" onclick={() => (s = JSON.parse(JSON.stringify(app.settings)))}>{t('Undo changes')}</button>
      <button class="btn primary" onclick={save} disabled={saving}>{t('Save changes')}</button>
    </div>
  {/if}
</div>

<style>
  .panel {
    margin-bottom: 14px;
  }
  .panel h2 {
    font-size: 1rem;
  }
  h3 {
    font-size: 0.88rem;
    margin: 14px 0 8px;
    color: var(--muted);
  }
  .mt {
    margin-top: 12px;
  }
  .lv {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 12px 0;
  }
  .panel .check {
    margin-bottom: 6px;
  }
  .how ol {
    padding-inline-start: 20px;
    line-height: 1.6;
  }
  .savebar {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
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
