<script lang="ts">
  // Clans (lineages) of a person: chips + input with suggestions.
  import { call } from '../api.svelte';
  import { t } from '../i18n/index.svelte';
  import X from '@lucide/svelte/icons/x';
  import type { ClanInfo } from '../types';

  let { value = $bindable([]) }: { value?: string[] } = $props();
  let known = $state<string[]>([]);
  let q = $state('');
  const listId = 'clans-' + Math.random().toString(36).slice(2);

  $effect(() => {
    call<ClanInfo[]>('clans.list').then((c) => (known = c.map((x) => x.name)));
  });

  function add(name: string) {
    name = name.trim();
    if (name && !value.includes(name)) value = [...value, name];
    q = '';
  }
  function key(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      add(q);
    } else if (e.key === 'Backspace' && !q && value.length) {
      value = value.slice(0, -1);
    }
  }
</script>

<div class="ci">
  {#each value as c (c)}
    <span class="tag heirloom">{c}<button type="button" onclick={() => (value = value.filter((x) => x !== c))} aria-label={t('Remove')}><X size={12} /></button></span>
  {/each}
  <input type="text" bind:value={q} list={listId} onkeydown={key} onchange={() => known.includes(q.trim()) && add(q)} onblur={() => q.trim() && add(q)} placeholder={value.length ? '' : t('Type a clan name and press Enter')} />
  <datalist id={listId}>
    {#each known.filter((k) => !value.includes(k)) as k}<option value={k}></option>{/each}
  </datalist>
</div>

<style>
  .ci {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    border: 1.5px solid transparent;
    border-radius: 12px;
    padding: 6px 8px;
    background: var(--surface-2);
    min-height: 38px;
  }
  .ci:focus-within {
    background: var(--surface);
    border-color: var(--ink);
    box-shadow: 0 0 0 3px var(--ink-soft);
  }
  .ci input {
    border: 0 !important;
    background: transparent !important;
    box-shadow: none !important;
    flex: 1;
    min-width: 140px;
    min-height: 26px !important;
    padding: 2px 4px !important;
  }
  .tag button {
    border: 0;
    background: none;
    padding: 0;
    margin-inline-start: 2px;
    display: inline-flex;
    cursor: pointer;
    color: inherit;
  }
</style>
