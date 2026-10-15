<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { go, back } from '../lib/router.svelte';
  import { t } from '../lib/i18n/index.svelte';
  import { toast } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import DateInput from '../lib/components/DateInput.svelte';
  import RichText from '../lib/components/RichText.svelte';
  import PhotoField from '../lib/components/PhotoField.svelte';
  import GalleryEditor from '../lib/components/GalleryEditor.svelte';
  import X from '@lucide/svelte/icons/x';
  import Plus from '@lucide/svelte/icons/plus';
  import type { Source, SourceView, GalleryItem } from '../lib/types';

  let { id }: { id: number | null } = $props();
  function blank(): Source {
    return { id: 0, title: '', date: '', url: '', notes: '', media_url: '', photo: null, scans: [], persons: [], created: 0, updated: 0 };
  }
  let src = $state<Source>(blank());
  let scans = $state<GalleryItem[]>([]);
  let persons = $state<(number | null)[]>([]);
  let loaded = $state(false);
  let busy = $state(false);

  $effect(() => {
    const cur = id;
    loaded = false;
    if (cur) {
      call<SourceView>('source.get', { id: cur }).then((v) => {
        src = { ...blank(), ...v.source };
        scans = v.source.scans.map((m) => ({ media: m, caption: '' }));
        persons = [...v.source.persons];
        loaded = true;
      });
    } else {
      src = blank();
      scans = [];
      persons = [];
      loaded = true;
    }
  });

  async function save() {
    if (!src.title.trim()) {
      toast(t('Enter a title.'), 'error');
      return;
    }
    busy = true;
    try {
      const r = await call<{ id: number }>('source.save', { source: { ...src, id: id || 0, scans: scans.map((s) => s.media), persons: persons.filter((p): p is number => !!p) } });
      toast(t('Saved.'));
      go(`/source/${r.id}`, true);
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
</script>

<div class="page">
  <div class="page-head"><h1>{id ? t('Edit source') : t('Add source')}</h1></div>
  {#if loaded}
    <form onsubmit={(e) => { e.preventDefault(); save(); }}>
      <div class="panel">
        <div class="form-grid">
          <label class="field full"><span class="lbl">{t('Title')}</span><input type="text" bind:value={src.title} placeholder={t('e.g. Birth certificate of Jan Kowalski, parish of Wieliczka')} required /></label>
          <label class="field"><span class="lbl">{t('Date')}</span><DateInput bind:value={src.date} /></label>
          <label class="field"><span class="lbl">{t('Link (URL)')}</span><input type="url" bind:value={src.url} placeholder="https://" /></label>
          <label class="field full"><span class="lbl">{t('Audio or video recording (URL)')}</span><input type="url" bind:value={src.media_url} placeholder="https://" /></label>
        </div>
      </div>
      <div class="panel"><span class="lbl-s">{t('Notes')}</span><RichText bind:value={src.notes} label={t('Notes')} /></div>
      <div class="panel"><span class="lbl-s">{t('Cover image')}</span><PhotoField bind:value={src.photo} /></div>
      <div class="panel"><h2>{t('Scans')}</h2><GalleryEditor bind:items={scans} captions={false} /></div>
      <div class="panel">
        <h2>{t('People')}</h2>
        <p class="hint">{t('To pin the source to specific facts (date of birth, place…), open the person\'s form, tab “Sources”.')}</p>
        {#each persons as p, i (i)}
          <div class="pr">
            <div class="grow"><PersonPicker bind:value={persons[i]} /></div>
            <button type="button" class="btn ghost icon" onclick={() => (persons = persons.filter((_, k) => k !== i))} aria-label={t('Remove')}><X size={16} /></button>
          </div>
        {/each}
        <button type="button" class="btn sm" onclick={() => (persons = [...persons, null])}><Plus size={15} /> {t('Add person')}</button>
      </div>
      <div class="savebar">
        <button type="button" class="btn" onclick={() => back(id ? `/source/${id}` : '/sources')}>{t('Cancel')}</button>
        <button class="btn primary" type="submit" disabled={busy}>{t('Save')}</button>
      </div>
    </form>
  {/if}
</div>

<style>
  .panel {
    margin-bottom: 14px;
  }
  .panel h2 {
    font-size: 1rem;
  }
  .lbl-s {
    display: block;
    font-size: 0.86rem;
    color: var(--muted);
    font-weight: 500;
    margin-bottom: 8px;
  }
  .pr {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
  }
  .grow {
    flex: 1;
    min-width: 0;
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
