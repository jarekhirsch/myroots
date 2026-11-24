<script lang="ts">
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { backend } from '../lib/backend';
  import { t, tn, LANGUAGES } from '../lib/i18n/index.svelte';
  import { toast } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import RichText from '../lib/components/RichText.svelte';
  import Globe from '@lucide/svelte/icons/globe';
  import Upload from '@lucide/svelte/icons/upload-cloud';
  import External from '@lucide/svelte/icons/external-link';
  import { locale } from '../lib/i18n/index.svelte';
  import type { WpBackend, SiteState, PublishProgress } from '../lib/backend/wp';
  import type { Settings, SiteLock } from '../lib/types';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import LockIcon from '@lucide/svelte/icons/lock-keyhole';
  import X from '@lucide/svelte/icons/x';
  import { deriveKey } from '../lib/sitelock';
  import { summaries } from '../lib/api.svelte';

  const NO_LOCK: SiteLock = { mode: '', salt: '', key: '', iterations: 0, people: [], hint: '' };
  function withLock(v: Settings): Settings {
    const c = JSON.parse(JSON.stringify(v));
    c.site.lock = { ...NO_LOCK, ...(c.site.lock || {}) };
    return c;
  }
  let s = $state<Settings>(withLock(app.settings!));
  let pw1 = $state('');
  let pw2 = $state('');

  let busy = $state(false);
  let saving = $state(false);
  const dirty = $derived(JSON.stringify(s) !== JSON.stringify(withLock(app.settings!)) || !!pw1);

  /* ---------- password ---------- */
  let pick = $state<number | null>(null);
  let names = $state(new Map<number, string>());
  $effect(() => {
    summaries().then((v) => (names = new Map(v.map((p) => [p.id, p.name]))));
  });
  const lock = $derived(s.site.lock!);
  const needsPassword = $derived(!!lock.mode && !lock.key);
  function addPerson(id: number | null) {
    if (id && !lock.people.some((p) => p.person === id)) lock.people = [...lock.people, { person: id, scope: 'descendants' }];
    pick = null;
  }

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
    if (pw1 || pw2) {
      if (pw1 !== pw2) return toast(t('The passwords are not the same.'), 'error');
      if (pw1.length < 6) return toast(t('The password must have at least 6 characters.'), 'error');
    }
    if (needsPassword && !pw1) return toast(t('Set a password for the protected part.'), 'error');
    saving = true;
    try {
      if (pw1) {
        const k = await deriveKey(pw1);
        s.site.lock = { ...lock, salt: k.salt, key: k.key, iterations: k.iterations };
      }
      if (!s.site.lock!.mode) s.site.lock = { ...NO_LOCK };
      await call('settings.save', { settings: s });
      await refreshInfo();
      s = withLock(app.settings!);
      pw1 = pw2 = '';
      toast(t('Settings saved.'));
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      saving = false;
    }
  }

  /* ---------- published by WordPress ---------- */
  const wp = backend.platform === 'wp' ? (backend as unknown as WpBackend) : null;
  let site = $state<SiteState | null>(null);
  let prog = $state<PublishProgress | null>(null);
  $effect(() => {
    if (wp) wp.siteState().then((v) => (site = v)).catch((e) => toast(errorText(e), 'error'));
  });
  async function publish() {
    if (!wp) return;
    if (dirty) await save();
    busy = true;
    try {
      await wp.publish((p) => (prog = p));
      site = await wp.siteState();
      toast(t('The family website is published.'));
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
      prog = null;
    }
  }
  async function setAuto(v: boolean) {
    if (!wp || !site) return;
    try {
      await wp.setAutoPublish(v);
      site = { ...site, auto: v };
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }
  function when(ms: number) {
    return new Intl.DateTimeFormat(locale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(ms));
  }
  async function copyCode(code: string) {
    try {
      await navigator.clipboard.writeText(code);
      toast(t('Copied.'));
    } catch {
      toast(t('Could not copy. Select the text and copy it by hand.'), 'error');
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
  const server = (window as any).MYROOTS_WP?.host === 'server';
  const wpShortcode = $derived.by(() => {
    const ts = (window as any).MYROOTS_WP?.tree_slug || '';
    return ts ? `[myroots tree="${ts}"]` : '[myroots]';
  });
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Family website')}</h1>
      {#if wp}
        <p class="lead">{server ? t('A read-only copy of the tree for your family, published on this server. Living people are protected by the privacy settings below.') : t('A read-only copy of the tree for your family, published on this WordPress site. Living people are protected by the privacy settings below.')}</p>
      {:else}
        <p class="lead">{t('A read-only copy of the tree for your family: a folder of files that works on any web hosting, or opened straight from a disk or USB stick.')}</p>
      {/if}
    </div>
    {#if wp}
      <div class="head-btns">
        <button class="btn primary" onclick={publish} disabled={busy}><Upload size={16} /> {busy ? t('Preparing…') : t('Publish on the website')}</button>
      </div>
    {:else}
      <button class="btn primary" onclick={exportSite} disabled={busy}><Globe size={16} /> {busy ? t('Preparing…') : t('Export website (.zip)')}</button>
    {/if}
  </div>

  {#if wp && site}
    <section class="panel wpsite">
      <div class="wp-row">
        <div>
          <h2>{server ? t('On this server') : t('On this WordPress site')}</h2>
          <p class="url"><a href={site.url} target="_blank" rel="noopener"><bdi>{new URL(site.url, location.href).href}</bdi> <External size={14} /></a></p>
          <p class="hint">
            {#if prog}
              {prog.stage === 'prepare' ? t('Preparing photos: %d of %d', prog.done, prog.total) : t('Sending: %d of %d', prog.done, prog.total)}
            {:else if site.published}
              {t('Last published: %s', when(site.published))} · {tn('%d person', '%d people', site.persons)} · {tn('%d photo', '%d photos', site.media)}
            {:else}
              {t('Not published yet. Click “Publish on the website”.')}
            {/if}
          </p>
          {#if prog && prog.total}<div class="bar"><span style="width:{Math.round((prog.done / prog.total) * 100)}%"></span></div>{/if}
        </div>
      </div>
      <label class="check"><input type="checkbox" checked={site.auto} onchange={(e) => setAuto((e.currentTarget as HTMLInputElement).checked)} /> {t('Publish automatically after every change')}</label>
      {#if server}
        <p class="hint">{t('Share this address with the family. On a phone it can be added to the home screen like an app.')}</p>
      {:else}
        <p class="hint">{t('To show the tree inside a page or post, add the MyRoots block or this shortcode:')} <button type="button" class="code" onclick={() => copyCode(wpShortcode)}>{wpShortcode}</button></p>
      {/if}
    </section>
  {/if}

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
            <label class="check"><input type="radio" bind:group={s.privacy.level} value="censor_living" /> <span><strong>{t('Censor data of living people')}</strong><br /><span class="hint">{t('People are visible, but selected fields of living people are hidden.')}</span></span></label>
            <label class="check"><input type="radio" bind:group={s.privacy.level} value="hide_living" /> <span><strong>{t('Hide living people')}</strong><br /><span class="hint">{t('Sections are visible, but living people are omitted entirely.')}</span></span></label>
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

      <section class="panel lockp">
        <h2><LockIcon size={17} strokeWidth={1.9} /> {t('Password')}</h2>
        <div class="lv">
          <label class="check"><input type="radio" bind:group={lock.mode} value="" /> <span>{t('No password')}</span></label>
          <label class="check"><input type="radio" bind:group={lock.mode} value="all" /> <span><strong>{t('The whole website')}</strong><br /><span class="hint">{t('Visitors type the password before they see anything.')}</span></span></label>
          <label class="check"><input type="radio" bind:group={lock.mode} value="people" /> <span><strong>{t('Chosen people and branches')}</strong><br /><span class="hint">{t('The rest of the tree stays open, the chosen people appear as protected.')}</span></span></label>
        </div>
        {#if lock.mode === 'people'}
          <div class="locked">
            {#each lock.people as b, i (b.person)}
              <div class="lrow">
                <span class="nm">{names.get(b.person) || '#' + b.person}</span>
                <select bind:value={b.scope} aria-label={t('What is protected')}>
                  <option value="person">{t('Only this person')}</option>
                  <option value="descendants">{t('With descendants and their partners')}</option>
                  <option value="ancestors">{t('With ancestors')}</option>
                </select>
                <button type="button" class="btn ghost icon" aria-label={t('Remove')} onclick={() => (lock.people = lock.people.filter((_, j) => j !== i))}><X size={15} /></button>
              </div>
            {/each}
            <PersonPicker bind:value={pick} onchange={addPerson} exclude={lock.people.map((p) => p.person)} placeholder={t('Add a person or a branch…')} />
          </div>
        {/if}
        {#if lock.mode}
          <div class="form-grid mt">
            <label class="field"><span class="lbl">{lock.key ? t('New password') : t('Password')}</span><input type="password" bind:value={pw1} autocomplete="new-password" placeholder={lock.key ? t('(unchanged)') : ''} /></label>
            <label class="field"><span class="lbl">{t('Repeat the password')}</span><input type="password" bind:value={pw2} autocomplete="new-password" /></label>
            <label class="field full"><span class="lbl">{t('Hint (optional)')}</span><input type="text" bind:value={lock.hint} maxlength="120" placeholder={t('e.g. the name of grandma’s dog')} /></label>
          </div>
          <p class="hint">{t('The protected part is encrypted, so without the password it cannot be read even from the files of the website. Give the password only to the family. MyRoots does not keep it: if you forget it, simply set a new one.')}</p>
          {#if !wp}<p class="hint">{t('Protected photos are shown when the website is opened from a web address (hosting), not straight from a disk.')}</p>{/if}
        {/if}
      </section>

      <section class="panel">
        <h2>{t('Map on the website')}</h2>
        <label class="check"><input type="checkbox" bind:checked={s.map_external} /> {t('Use the online map (OpenStreetMap data; the map background is loaded from OpenFreeMap)')}</label>
      </section>

      {#if !wp}
      <section class="panel how">
        <h2>{t('How to publish')}</h2>
        <ol>
          <li>{t('Export the website and unpack the ZIP file.')}</li>
          <li>{t('Upload the folder to your hosting (FTP), or drag it onto a free static host such as Netlify Drop, Cloudflare Pages or GitHub Pages.')}</li>
          <li>{t('Or simply give the folder to the family: index.html opens in any browser, even without internet.')}</li>
        </ol>
        <p class="hint">{t('After changes in the tree, export again and replace the files.')}</p>
      </section>
      {/if}
    </div>
  </div>

  {#if dirty}
    <div class="savebar">
      <button class="btn" onclick={() => ((s = withLock(app.settings!)), (pw1 = pw2 = ''))}>{t('Undo changes')}</button>
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
  .head-btns {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .wpsite .url {
    margin: 4px 0 6px;
    font-weight: 600;
    word-break: break-all;
  }
  .wpsite .url a {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .wpsite .check {
    margin: 10px 0 4px;
  }
  .bar {
    height: 6px;
    border-radius: 99px;
    background: var(--line);
    overflow: hidden;
    margin: 8px 0 4px;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--heirloom);
    transition: width 0.2s;
  }
  .code {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 0.85rem;
    border: 1px solid var(--line);
    background: var(--surface-2);
    border-radius: 6px;
    padding: 1px 7px;
    cursor: copy;
    color: var(--text);
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
  .lockp h2 {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .locked {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 10px;
  }
  .lrow {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .lrow .nm {
    flex: 1 1 140px;
    min-width: 0;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .lrow select {
    flex: 1 1 200px;
    min-width: 0;
  }
</style>
