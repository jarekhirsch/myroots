<script lang="ts">
  import { app, call, refreshInfo, bump } from '../lib/api.svelte';
  import { backend } from '../lib/backend';
  import { route, go } from '../lib/router.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { toast, confirmDialog, promptDialog, withProgress } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import { pickFile } from '../lib/media';
  import { slug, stamp } from '../lib/backend/util';
  import { downloadWpMedia, attachLocalFiles, type WpProgress } from '../lib/wpmedia';
  import FileUp from '@lucide/svelte/icons/file-up';
  import FileDown from '@lucide/svelte/icons/file-down';
  import type { GedcomReport, WpReport, WpPending } from '../lib/types';

  let people = $state(0);
  $effect(() => {
    call<{ persons: number }>('overview').then((o) => (people = o.persons));
  });
  let tab = $state(['gedcom', 'wp', 'archive', 'reset'].includes(route.query.tab) ? route.query.tab : 'archive');
  let busy = $state(false);

  /* ---------- backup ---------- */
  async function backup() {
    busy = true;
    try {
      const where = await backend.exportArchive();
      if (where) toast(t('Backup saved: %s', where));
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
  async function restore() {
    const f = await pickFile('.myroots,application/zip');
    if (!f) return;
    if (!app.info?.empty && !(await confirmDialog(t('Restore a backup?'), t('Everything currently in the app will be replaced by the content of the backup.'), t('Restore'), true))) return;
    busy = true;
    try {
      await withProgress(t('Restoring the backup…'), async (report) => backend.importArchive(new Uint8Array(await f.arrayBuffer()), report));
      await refreshInfo();
      bump();
      toast(t('Backup restored.'));
      go('/');
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }

  /* ---------- GEDCOM ---------- */
  let gedFile = $state<File | null>(null);
  let gedB64 = '';
  let gedMode = $state('update');
  let preview = $state<GedcomReport | null>(null);
  let result = $state<GedcomReport | null>(null);
  let exp = $state({ notes: true, attrs: true, sources: true, places: true, pets: true, published_only: false });

  const EVENT_NAMES = () => ({
    BAPM: t('Baptism'), CHR: t('Christening'), CONF: t('Confirmation'), FCOM: t('First communion'), BURI: t('Burial'), CREM: t('Cremation'),
    ADOP: t('Adoption'), BARM: t('Bar mitzvah'), BASM: t('Bat mitzvah'), GRAD: t('Graduation'), RETI: t('Retirement'), EMIG: t('Emigration'),
    IMMI: t('Immigration'), NATU: t('Naturalization'), CENS: t('Census'), RESI: t('Residence'), ORDN: t('Ordination'), PROB: t('Probate'),
    WILL: t('Will'), _MILT: t('Military service'), _MILI: t('Military service'),
    CHRA: t('Adult christening'), BLES: t('Blessing'), TITL: t('Title'), NATI: t('Nationality'), DSCR: t('Physical description'),
    PROP: t('Property'), ENGA: t('Engagement'), MARB: t('Marriage banns'), MARC: t('Marriage contract'), MARL: t('Marriage license'),
    MARS: t('Marriage settlement'), DIVF: t('Divorce filed'),
  });

  function b64(bytes: Uint8Array): string {
    let s = '';
    for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
    return btoa(s);
  }

  async function chooseGed() {
    const f = await pickFile('.ged,.gedcom,.gdz,.zip,text/plain,application/zip');
    if (!f) return;
    gedFile = f;
    gedB64 = await withProgress(t('Reading the file…'), async () => b64(new Uint8Array(await f.arrayBuffer())));
    result = null;
    await runGed(true);
  }
  async function runGed(dry: boolean) {
    busy = true;
    try {
      const r = await withProgress(dry ? t('Checking the file…') : t('Importing…'), () =>
        call<GedcomReport>('gedcom.import', { base64: gedB64, options: { mode: gedMode, dry_run: dry, event_names: EVENT_NAMES() } }),
      );
      if (dry) preview = r;
      else {
        result = r;
        preview = null;
        gedFile = null;
        await refreshInfo();
      }
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
  async function exportGed() {
    busy = true;
    try {
      const r = await call<{ text: string }>('gedcom.export', { options: { ...exp, media: false, event_names: EVENT_NAMES() } });
      const where = await backend.saveFile(`${slug(app.info?.tree_name || 'myroots')}-${stamp()}.ged`, r.text, 'text/plain');
      if (where) toast(t('Saved: %s', where));
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
  function warnText(w: [string, string]): string {
    const m: Record<string, () => string> = {
      indi_no_name: () => t('Person without a name: %s', w[1]),
      pet_no_name: () => t('Pet without a name'),
      child_points_elsewhere: () => t('Child points to a different parent') + ': ' + w[1],
      ancestor_cycle: () => t('Ancestor loop') + ': ' + w[1],
    };
    return m[w[0]]?.() || `${w[0]} ${w[1]}`;
  }

  /* ---------- WordPress ---------- */
  let wpReport = $state<WpReport | null>(null);
  let pending = $state<WpPending | null>(null);
  let wpProg = $state<WpProgress | null>(null);
  let stop = false;
  $effect(() => {
    if (tab === 'wp') call<WpPending>('wp.pending').then((p) => (pending = p));
  });

  // Inside WordPress: the 2.x plugin data of this very site, without a file.
  const wpb = backend.platform === 'wp' ? (backend as any) : null;
  const legacy = wpb ? Number(wpb.cfg?.legacy || 0) : 0;
  async function importHere() {
    if (!app.info?.empty && !(await confirmDialog(t('Import into a tree that is not empty?'), t('The people from the backup will be added next to the existing ones. You can merge duplicates later.'), t('Import')))) return;
    busy = true;
    try {
      const json = await wpb.legacyExport();
      wpReport = await call<WpReport>('wp.import', { json });
      // the tree gets the name of the site (not its address)
      const st = await call<any>('settings.get');
      if (wpb.cfg.site_name && (!st.tree_name || st.tree_name === new URL(wpb.cfg.home).host)) await call('settings.save', { settings: { ...st, tree_name: wpb.cfg.site_name } });
      pending = await call<WpPending>('wp.pending');
      await refreshInfo();
    } catch (e) {
      toast(errorText(e), 'error');
      busy = false;
      return;
    }
    busy = false;
    if (pending?.items.length) await fetchMedia(wpb.cfg.home);
  }

  async function importWp() {
    const f = await pickFile('.json,application/json');
    if (!f) return;
    if (!app.info?.empty && !(await confirmDialog(t('Import into a tree that is not empty?'), t('The people from the backup will be added next to the existing ones. You can merge duplicates later.'), t('Import')))) return;
    busy = true;
    try {
      wpReport = await call<WpReport>('wp.import', { json: await f.text() });
      pending = await call<WpPending>('wp.pending');
      await refreshInfo();
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
  async function fetchMedia(here = '') {
    let site = here || pending?.site || '';
    if (!site) {
      site = ((await promptDialog(t('Address of the WordPress site'), '', 'https://')) || '').trim();
      if (!/^https?:\/\/./.test(site)) return;
    }
    stop = false;
    busy = true;
    try {
      wpProg = await downloadWpMedia((p) => (wpProg = p), () => stop, site);
      pending = await call<WpPending>('wp.pending');
    } finally {
      busy = false;
    }
  }
  function chooseUploads() {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true;
    (input as any).webkitdirectory = true;
    input.onchange = async () => {
      if (!wpProg) return;
      busy = true;
      const n = await attachLocalFiles(wpProg.failed, Array.from(input.files || []));
      pending = await call<WpPending>('wp.pending');
      wpProg = { ...wpProg, ok: wpProg.ok + n, failed: wpProg.failed.filter((f) => pending!.items.some((i) => i.wp_id === f.wp_id)) };
      busy = false;
      toast(tn('Attached %d file.', 'Attached %d files.', n));
    };
    input.click();
  }

  /* ---------- reset ---------- */
  async function reset() {
    const word = await promptDialog(t('Delete all data'), t('Type DELETE to confirm. Make a backup first!'), '', t('Delete everything'));
    if (word !== 'DELETE') return;
    await call('data.reset', { confirm: 'DELETE', keep_settings: true });
    await refreshInfo();
    toast(t('All data has been deleted.'));
    go('/');
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Import and backup')}</h1>
      <p class="lead">{wpb && (window as any).MYROOTS_WP?.host === 'server' ? t('Your data is stored in the database on this server. Make a backup regularly and keep it somewhere safe.') : wpb ? t('Your data is stored in the database of this WordPress site. Make a backup regularly and keep it somewhere safe.') : t('Your data lives only on this device. Make a backup regularly and keep it somewhere safe.')}</p>
    </div>
  </div>
  <div class="tabs" role="tablist">
    <button role="tab" aria-selected={tab === 'archive'} onclick={() => (tab = 'archive')}>{t('Backup')}</button>
    <button role="tab" aria-selected={tab === 'gedcom'} onclick={() => (tab = 'gedcom')}>GEDCOM</button>
    <button role="tab" aria-selected={tab === 'wp'} onclick={() => (tab = 'wp')}>{t('From WordPress')}</button>
    <button role="tab" aria-selected={tab === 'reset'} onclick={() => (tab = 'reset')}>{t('Delete data')}</button>
  </div>

  {#if tab === 'archive'}
    <div class="grid-2">
      <section class="panel">
        <h2>{t('Make a backup')}</h2>
        <p>{t('One .myroots file with everything: people, relations, sources, settings and all photos and scans. Use it to move the tree to another device.')}</p>
        <button class="btn primary" onclick={backup} disabled={busy}><FileDown size={16} /> {t('Save backup')}</button>
      </section>
      <section class="panel">
        <h2>{t('Restore a backup')}</h2>
        <p>{t('Open a .myroots file. The current content will be replaced.')}</p>
        <button class="btn" onclick={restore} disabled={busy}><FileUp size={16} /> {t('Open .myroots file')}</button>
      </section>
    </div>
  {:else if tab === 'gedcom'}
    <div class="grid-2">
      <section class="panel">
        <h2>{t('Import a GEDCOM file')}</h2>
        <p class="small muted">{t('GEDCOM is the common format of genealogy programs (MyHeritage, Ancestry, FamilySearch, Gramps…). You will see a preview before anything is saved.')}</p>
        <label class="field">
          <span class="lbl">{t('People who already exist')}</span>
          <select bind:value={gedMode} onchange={() => gedFile && runGed(true)}>
            <option value="update">{t('Update them with data from the file')}</option>
            <option value="skip">{t('Skip them')}</option>
            <option value="new">{t('Always add as new people')}</option>
          </select>
        </label>
        <div class="actions mt">
          <button class="btn" onclick={chooseGed} disabled={busy}><FileUp size={16} /> {gedFile ? gedFile.name : t('Choose a .ged file')}</button>
        </div>
        {#if preview}
          <div class="report">
            <strong>{t('Import preview (dry run):')}</strong>
            <ul>
              <li>{t('People added: %d', preview.created)}</li>
              <li>{t('People updated: %d', preview.updated)}</li>
              <li>{t('People skipped (duplicates): %d', preview.skipped)}</li>
              <li>{tn('%d family', '%d families', preview.families)}</li>
              {#if preview.pets_created}<li>{t('Pets added: %d', preview.pets_created)}</li>{/if}
              {#if preview.sources}<li>{tn('%d source', '%d sources', preview.sources)}</li>{/if}
            </ul>
            {#each preview.warnings.slice(0, 10) as w}<div class="small warn-t">{warnText(w)}</div>{/each}
            <button class="btn primary" onclick={() => runGed(false)} disabled={busy}>{t('Import')}</button>
          </div>
        {/if}
        {#if result}
          <div class="notice ok">
            <strong>{t('Import finished:')}</strong>
            {t('People added: %d', result.created)}, {t('People updated: %d', result.updated)}.
            <a href="#/people">{t('People')}</a> · <a href="#/duplicates">{t('Possible duplicates')}</a>
          </div>
        {/if}
      </section>
      <section class="panel">
        <h2>{t('Export to GEDCOM')}</h2>
        <p class="small muted">{t('Export the whole database (%d people) to a GEDCOM 5.5.1 file.', people)}</p>
        <div class="opts">
          <label class="check"><input type="checkbox" bind:checked={exp.notes} /> {t('Biographies and notes')}</label>
          <label class="check"><input type="checkbox" bind:checked={exp.attrs} /> {t('Occupation, education, religion')}</label>
          <label class="check"><input type="checkbox" bind:checked={exp.sources} /> {t('Sources')}</label>
          <label class="check"><input type="checkbox" bind:checked={exp.places} /> {t('Place coordinates')}</label>
          <label class="check"><input type="checkbox" bind:checked={exp.pets} /> {t('Pets')}</label>
          <label class="check"><input type="checkbox" bind:checked={exp.published_only} /> {t('Only published people')}</label>
        </div>
        <button class="btn primary" onclick={exportGed} disabled={busy}><FileDown size={16} /> {t('Export GEDCOM')}</button>
      </section>
    </div>
  {:else if tab === 'wp'}
    {#if legacy}
      <section class="panel here">
        <h2>{t('MyRoots 2.x data on this site')}</h2>
        <p>{tn('This WordPress site has %d person from the previous version of the plugin.', 'This WordPress site has %d people from the previous version of the plugin.', legacy, legacy)}</p>
        <p class="hint">{t('They are copied into the new version together with relations, pets, sources, photos and scans. The old data stays untouched.')}</p>
        <button class="btn primary" onclick={importHere} disabled={busy}><FileUp size={16} /> {t('Import from this site')}</button>
      </section>
    {/if}
    <section class="panel" class:mt={!!legacy}>
      <h2>{t('Move from the MyRoots WordPress plugin')}</h2>
      <ol class="steps">
        <li>{t('In WordPress open MyRoots → Backup and click “Download backup (JSON)”.')}</li>
        <li>{t('Open the downloaded file here. People, relations, pets and sources are imported.')}</li>
        <li>{t('Then download the photos and scans from your site. The site must still be online.')}</li>
      </ol>
      <button class="btn primary" onclick={importWp} disabled={busy}><FileUp size={16} /> {t('Open the JSON backup')}</button>
      {#if wpReport}
        <div class="notice ok mt">
          {t('Import finished: %1$d people, %2$d pets, %3$d sources.', wpReport.persons, wpReport.pets, wpReport.sources)}
          {#each wpReport.warnings.slice(0, 8) as w}<div class="small">{warnText(w)}</div>{/each}
        </div>
      {/if}
    </section>
    {#if pending?.items.length || wpProg}
      <section class="panel mt">
        <h2>{t('Photos and scans from the site')}</h2>
        {#if pending?.items.length}
          <p>{tn('%1$d file is waiting to be downloaded from %2$s.', '%1$d files are waiting to be downloaded from %2$s.', pending.items.length, pending.items.length, pending.site)}</p>
        {/if}
        {#if wpProg && busy}
          <p>{t('Downloading… %1$d of %2$d', wpProg.done, wpProg.total)}</p>
          <progress max={wpProg.total} value={wpProg.done}></progress>
          <button class="btn sm" onclick={() => (stop = true)}>{t('Stop')}</button>
        {:else}
          <div class="actions">
            {#if pending?.items.length}<button class="btn primary" onclick={() => fetchMedia(wpb ? wpb.cfg.home : '')}>{t('Download photos')}</button>{/if}
            {#if pending?.items.length}<button class="btn ghost" onclick={() => call('wp.clear').then(() => (pending = { site: '', items: [] }))}>{t('Skip the photos')}</button>{/if}
          </div>
        {/if}
        {#if wpProg && !busy}
          <p class="mt">{t('Downloaded: %1$d, failed: %2$d.', wpProg.ok, wpProg.failed.length)}</p>
          {#if wpProg.failed.length}
            <p class="small muted">{t('Some files could not be downloaded (in a web browser this is usually blocked by the site). Copy the folder wp-content/uploads from your server and choose it here: files are matched by name.')}</p>
            <button class="btn" onclick={chooseUploads}>{t('Choose the uploads folder')}</button>
          {/if}
        {/if}
      </section>
    {/if}
  {:else if tab === 'reset'}
    <section class="panel danger-zone">
      <h2>{t('Delete all data')}</h2>
      <p>{t('Removes all people, relations, pets, sources and photos from this device. Settings are kept. This cannot be undone.')}</p>
      <button class="btn danger" onclick={reset}>{t('Delete all data…')}</button>
    </section>
  {/if}
</div>

<style>
  .mt {
    margin-top: 14px;
  }
  .grid-2 .panel {
    margin: 0;
  }
  .report {
    margin-top: 16px;
    padding: 12px 14px;
    background: var(--surface-2);
    border-radius: 12px;
    box-shadow: var(--shadow);
  }
  .report ul {
    margin: 6px 0 12px;
    padding-inline-start: 18px;
  }
  .warn-t {
    color: var(--warn);
  }
  .opts {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 10px 0 16px;
  }
  .steps {
    padding-inline-start: 20px;
    line-height: 1.7;
  }
  progress {
    width: 100%;
    margin-bottom: 8px;
  }
  .danger-zone {
    border-color: color-mix(in srgb, var(--danger) 40%, transparent);
  }
</style>
