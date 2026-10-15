<script lang="ts">
  // Picks images already stored in the app (shared between people, pets and sources).
  import Modal from './Modal.svelte';
  import Img from './Img.svelte';
  import { call } from '../api.svelte';
  import { t } from '../i18n/index.svelte';
  import { backend } from '../backend';
  import type { Media } from '../types';

  let { multiple = true, onpick, onclose }: { multiple?: boolean; onpick: (ids: number[]) => void; onclose: () => void } = $props();
  let items = $state<Media[]>([]);
  let sel = $state<number[]>([]);
  let q = $state('');

  $effect(() => {
    call<Media[]>('media.list').then(async (v) => {
      v = v.filter((m) => m.mime.startsWith('image/')).sort((a, b) => b.created - a.created);
      await backend.ensureMedia?.(v.map((m) => m.id));
      items = v;
    });
  });

  const shown = $derived(q ? items.filter((m) => `${m.caption} ${m.file_name}`.toLowerCase().includes(q.toLowerCase())) : items);

  function toggle(id: number) {
    if (!multiple) {
      onpick([id]);
      onclose();
      return;
    }
    sel = sel.includes(id) ? sel.filter((x) => x !== id) : [...sel, id];
  }
</script>

<Modal title={t('Media library')} {onclose} wide>
  <input type="search" placeholder={t('Search…')} bind:value={q} class="q" />
  {#if !items.length}
    <p class="muted">{t('There are no photos yet.')}</p>
  {/if}
  <div class="grid">
    {#each shown as m (m.id)}
      <button type="button" class:on={sel.includes(m.id)} onclick={() => toggle(m.id)} title={m.caption || m.file_name}>
        <Img id={m.id} alt={m.caption} />
        {#if sel.includes(m.id)}<span class="n">{sel.indexOf(m.id) + 1}</span>{/if}
      </button>
    {/each}
  </div>
  {#snippet footer()}
    <button class="btn" onclick={onclose}>{t('Cancel')}</button>
    {#if multiple}
      <button class="btn primary" disabled={!sel.length} onclick={() => { onpick(sel); onclose(); }}>{t('Add selected (%d)', sel.length)}</button>
    {/if}
  {/snippet}
</Modal>

<style>
  .q {
    margin-bottom: 12px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
    gap: 8px;
  }
  button {
    position: relative;
    aspect-ratio: 1;
    border: 2px solid transparent;
    border-radius: 12px;
    padding: 0;
    overflow: hidden;
    background: var(--ink-soft);
    cursor: pointer;
  }
  button.on {
    border-color: var(--heirloom);
  }
  button :global(img),
  button :global(.ph) {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .n {
    position: absolute;
    top: 5px;
    inset-inline-end: 5px;
    background: var(--heirloom);
    color: #fff;
    border-radius: 999px;
    font-size: 0.75rem;
    font-weight: 700;
    min-width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
