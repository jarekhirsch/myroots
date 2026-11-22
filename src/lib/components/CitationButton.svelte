<script lang="ts">
  // Sources of one fact (birth, an event, a marriage …): a small button next
  // to the field that opens a dialog with the list of citations. Each citation
  // names a source and optionally where in it the fact is (page, record
  // number), how reliable it is and the text of the record. A new source can
  // be created right here, without leaving the form.
  import Modal from './Modal.svelte';
  import { call } from '../api.svelte';
  import { t, tn } from '../i18n/index.svelte';
  import { toast } from '../ui.svelte';
  import { errorText } from '../errors';
  import BookMarked from '@lucide/svelte/icons/book-marked';
  import Plus from '@lucide/svelte/icons/plus';
  import X from '@lucide/svelte/icons/x';
  import { QUALITY } from '../citations';
  import type { Citation, SourceBrief } from '../types';

  let {
    value = [],
    sources = $bindable([]),
    fact = '',
    onchange,
  }: {
    /** Citations of the fact. */
    value?: Citation[];
    /** All sources of the tree (a source created here is added to it). */
    sources?: SourceBrief[];
    /** Name of the fact, shown in the dialog title. */
    fact?: string;
    onchange: (v: Citation[]) => void;
  } = $props();

  let open = $state(false);
  let draft = $state<Citation[]>([]);
  let newTitle = $state('');
  let creating = $state(false);

  function start() {
    draft = value.map((c) => ({ source: c.source, page: c.page || '', quality: c.quality ?? null, note: c.note || '' }));
    if (!draft.length) draft = [{ source: 0, page: '', quality: null, note: '' }];
    newTitle = '';
    open = true;
  }
  function add() {
    draft = [...draft, { source: 0, page: '', quality: null, note: '' }];
  }
  function remove(i: number) {
    draft = draft.filter((_, k) => k !== i);
  }
  async function createSource(i: number) {
    const title = newTitle.trim();
    if (!title) return;
    creating = true;
    try {
      const { id } = await call<{ id: number }>('source.save', { source: { title } });
      sources = [...sources, { id, title, date: '' }].sort((a, b) => a.title.localeCompare(b.title));
      draft[i].source = id;
      newTitle = '';
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      creating = false;
    }
  }
  function save() {
    const seen = new Set<number>();
    const out = draft.filter((c) => c.source && !seen.has(c.source) && seen.add(c.source)).map((c) => ({ ...c, page: c.page.trim(), note: c.note.trim() }));
    onchange(out);
    open = false;
  }
  const title = (id: number) => sources.find((s) => s.id === id)?.title || `#${id}`;
  const label = $derived(value.length ? tn('%d source', '%d sources', value.length) : t('Source'));
</script>

<button type="button" class="cite-btn" class:has={value.length > 0} onclick={start} title={value.map((c) => title(c.source) + (c.page ? `, ${c.page}` : '')).join('\n') || t('Add a source of this information')} aria-label={`${t('Sources')}: ${fact}`}>
  <BookMarked size={14} />
  <span>{label}</span>
</button>

{#if open}
  <Modal title={fact ? t('Sources: %s', fact) : t('Sources')} onclose={() => (open = false)} wide>
    <p class="hint">{t('Which record confirms this information? Add the source and, if you can, the page or record number.')}</p>
    {#each draft as c, i (i)}
      <div class="cit">
        <div class="row">
          <label class="field grow">
            <span class="lbl">{t('Source')}</span>
            <select bind:value={c.source}>
              <option value={0}>{t('Choose a source…')}</option>
              {#each sources as s (s.id)}<option value={s.id}>{s.title}{s.date ? ` (${s.date})` : ''}</option>{/each}
            </select>
          </label>
          <button type="button" class="btn ghost icon" onclick={() => remove(i)} aria-label={t('Remove')}><X size={16} /></button>
        </div>
        {#if !c.source}
          <div class="row new">
            <input type="text" placeholder={t('Or type the title of a new source, e.g. Parish register of births 1890-1900')} bind:value={newTitle} onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), createSource(i))} />
            <button type="button" class="btn sm" disabled={!newTitle.trim() || creating} onclick={() => createSource(i)}><Plus size={15} /> {t('New source')}</button>
          </div>
        {/if}
        <div class="row">
          <label class="field grow">
            <span class="lbl">{t('Page, record number')}</span>
            <input type="text" bind:value={c.page} placeholder={t('e.g. record 112, p. 34')} />
          </label>
          <label class="field">
            <span class="lbl">{t('Reliability')}</span>
            <select bind:value={c.quality}>
              <option value={null}>—</option>
              {#each QUALITY as [q, l] (q)}<option value={q}>{l()}</option>{/each}
            </select>
          </label>
        </div>
        <label class="field">
          <span class="lbl">{t('Text of the record or a note')}</span>
          <textarea rows="2" bind:value={c.note}></textarea>
        </label>
      </div>
    {/each}
    <button type="button" class="btn sm ghost" onclick={add}><Plus size={15} /> {t('Another source')}</button>
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => (open = false)}>{t('Cancel')}</button>
      <button type="button" class="btn primary" onclick={save}>{t('Done')}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .cite-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border: 1px dashed var(--line);
    background: transparent;
    color: var(--muted);
    border-radius: 999px;
    padding: 1px 8px;
    font: inherit;
    font-size: 0.78rem;
    line-height: 1.6;
    cursor: pointer;
    white-space: nowrap;
  }
  .cite-btn:hover,
  .cite-btn:focus-visible {
    color: var(--ink);
    border-color: var(--ink);
  }
  .cite-btn.has {
    border-style: solid;
    color: var(--ink);
    border-color: var(--line-strong);
    background: var(--ink-soft);
  }
  .cit {
    border: 1px solid var(--line);
    border-radius: var(--radius);
    padding: 10px 12px;
    margin-bottom: 12px;
    display: grid;
    gap: 8px;
  }
  .row {
    display: flex;
    gap: 8px;
    align-items: flex-end;
    flex-wrap: wrap;
  }
  .row.new input {
    flex: 1 1 240px;
  }
  .grow {
    flex: 1 1 220px;
  }
  textarea {
    resize: vertical;
  }
  .hint {
    margin-top: 0;
  }
</style>
