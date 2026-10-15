<script lang="ts">
  import { app, call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { fmtDateTime } from '../lib/format';
  import { toast, confirmDialog } from '../lib/ui.svelte';
  import { pickFile } from '../lib/media';
  import Modal from '../lib/components/Modal.svelte';
  import Trash from '@lucide/svelte/icons/trash';
  import Paste from '@lucide/svelte/icons/clipboard-paste';
  import type { Suggestion } from '../lib/types';

  const list = useLoad(() => call<Suggestion[]>('suggestions.list'));
  let filter = $state('new');
  let pasting = $state(false);
  let text = $state('');
  const shown = $derived((list.data || []).filter((s) => !filter || s.status === filter));
  const counts = $derived({
    new: (list.data || []).filter((s) => s.status === 'new').length,
    done: (list.data || []).filter((s) => s.status === 'done').length,
    rejected: (list.data || []).filter((s) => s.status === 'rejected').length,
  });

  async function importText(txt: string) {
    const r = await call<{ added: number; duplicates: number }>('suggestions.import', { text: txt });
    if (!r.added && !r.duplicates) toast(t('No suggestion code was found in the text.'), 'error');
    else toast(tn('Imported %d suggestion.', 'Imported %d suggestions.', r.added) + (r.duplicates ? ' ' + tn('%d was already imported.', '%d were already imported.', r.duplicates) : ''));
    pasting = false;
    text = '';
  }
  async function fromFile() {
    const f = await pickFile('.eml,.txt,.msg,text/plain,message/rfc822');
    if (f) importText(await f.text());
  }
  async function setStatus(s: Suggestion, status: string) {
    await call('suggestions.set_status', { id: s.id, status });
  }
  async function del(s: Suggestion) {
    if (await confirmDialog(t('Delete this suggestion?'), '', t('Delete'), true)) await call('suggestions.delete', { id: s.id });
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Suggestions')}</h1>
      <p class="lead">{t('Corrections sent by family members from the family website.')}</p>
    </div>
    <div class="actions">
      <button class="btn primary" onclick={() => (pasting = true)}><Paste size={16} /> {t('Paste from e-mail')}</button>
      <button class="btn" onclick={fromFile}>{t('Open a file')}</button>
    </div>
  </div>

  {#if !app.settings?.suggestions?.enabled}
    <div class="notice">{t('The suggestion form on the family website is currently disabled. You can turn it on in Family website.')} <a href="#/website">{t('Family website')}</a></div>
  {/if}

  <div class="tabs" role="tablist">
    <button role="tab" aria-selected={filter === 'new'} onclick={() => (filter = 'new')}>{t('New')} ({counts.new})</button>
    <button role="tab" aria-selected={filter === 'done'} onclick={() => (filter = 'done')}>{t('Done')} ({counts.done})</button>
    <button role="tab" aria-selected={filter === 'rejected'} onclick={() => (filter = 'rejected')}>{t('Rejected')} ({counts.rejected})</button>
    <button role="tab" aria-selected={filter === ''} onclick={() => (filter = '')}>{t('All')}</button>
  </div>

  {#if list.data && !shown.length}
    <div class="panel empty">{t('No suggestions.')}</div>
  {/if}
  {#each shown as s (s.id)}
    <article class="panel sg">
      <header>
        <div>
          {#if s.person}<a href="#/person/{s.person}" class="name who">{s.person_name}</a>{:else}<span class="name who">{s.person_name || t('General')}</span>{/if}
          <div class="small muted">{fmtDateTime(s.created)}{#if s.author} · {s.author}{/if}{#if s.email} · <a href="mailto:{s.email}">{s.email}</a>{/if}</div>
        </div>
        <span class="tag" class:ok={s.status === 'done'} class:danger={s.status === 'rejected'} class:heirloom={s.status === 'new'}>
          {s.status === 'done' ? t('Done') : s.status === 'rejected' ? t('Rejected') : t('New')}
        </span>
      </header>
      <p class="msg">{s.message}</p>
      <div class="actions">
        {#if s.person}<a class="btn sm" href="#/person/{s.person}/edit">{t('Edit person')}</a>{/if}
        {#if s.status !== 'done'}<button class="btn sm" onclick={() => setStatus(s, 'done')}>{t('Mark as done')}</button>{/if}
        {#if s.status !== 'rejected'}<button class="btn sm ghost" onclick={() => setStatus(s, 'rejected')}>{t('Reject')}</button>{/if}
        {#if s.status !== 'new'}<button class="btn sm ghost" onclick={() => setStatus(s, 'new')}>{t('Mark as new')}</button>{/if}
        <button class="btn sm ghost icon danger" onclick={() => del(s)} aria-label={t('Delete')}><Trash size={15} /></button>
      </div>
    </article>
  {/each}
</div>

{#if pasting}
  <Modal title={t('Paste from e-mail')} onclose={() => (pasting = false)}>
    <p class="small muted">{t('Paste the whole e-mail (or several). MyRoots finds the suggestion codes in the text.')}</p>
    <textarea rows="10" bind:value={text}></textarea>
    {#snippet footer()}
      <button class="btn" onclick={() => (pasting = false)}>{t('Cancel')}</button>
      <button class="btn primary" disabled={!text.trim()} onclick={() => importText(text)}>{t('Import')}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .sg {
    margin-bottom: 12px;
  }
  .sg header {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: flex-start;
  }
  .who {
    font-weight: 620;
    font-size: 1.05rem;
  }
  .msg {
    white-space: pre-line;
    margin: 10px 0 12px;
  }
</style>
