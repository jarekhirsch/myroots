<script lang="ts">
  import { summaries, call } from '../api.svelte';
  import { toast } from '../ui.svelte';
  import { errorText } from '../errors';
  import { t } from '../i18n/index.svelte';
  import Avatar from './Avatar.svelte';
  import X from '@lucide/svelte/icons/x';
  import { search } from '../text';
  import type { Summary } from '../types';

  let {
    value = $bindable(null),
    exclude = [],
    placeholder = '',
    gender = '',
    onchange,
    oncreate,
    create = undefined,
    id = undefined,
  }: {
    value?: number | null;
    exclude?: number[];
    placeholder?: string;
    gender?: string;
    onchange?: (id: number | null) => void;
    oncreate?: (name: string) => void;
    /** Offer "Add new person": creates the person right away and picks them. */
    create?: { gender?: string; last_name?: string };
    id?: string;
  } = $props();

  const listId = 'pp-' + Math.random().toString(36).slice(2);
  let all: Summary[] = $state.raw([]);
  let q = $state('');
  let open = $state(false);
  let active = $state(0);
  let input: HTMLInputElement | undefined = $state();

  $effect(() => {
    summaries().then((s) => (all = s));
  });

  const selected = $derived(all.find((p) => p.id === value) || null);

  const text = (p: Summary) => `${p.name} ${p.maiden_name} ${p.years} ${p.birth_place}`;
  const results = $derived.by(() => {
    const ex = new Set(exclude);
    const keep = (p: Summary) => !ex.has(p.id);
    if (!gender) return search(all, 'picker', text, q, 40, keep);
    // people of the wanted gender first
    const first = search(all, 'picker', text, q, 40, (p) => keep(p) && p.gender === gender);
    return first.length >= 40 ? first : [...first, ...search(all, 'picker', text, q, 40 - first.length, (p) => keep(p) && p.gender !== gender)];
  });

  function pick(p: Summary | null) {
    value = p ? p.id : null;
    q = '';
    open = false;
    onchange?.(value);
  }

  let creating = $state(false);
  async function createPerson(name: string) {
    if (oncreate) return oncreate(name);
    const parts = name.trim().split(/\s+/);
    const first = parts.length > 1 ? parts.slice(0, -1).join(' ') : parts[0];
    const last = parts.length > 1 ? parts[parts.length - 1] : create?.last_name || '';
    creating = true;
    try {
      const r = await call<{ id: number }>('person.create', { person: { first_name: first, last_name: last, gender: create?.gender || '', status: 'published' } });
      all = await summaries();
      pick(all.find((p) => p.id === r.id) || null);
      toast(t('Added: %s', [first, last].filter(Boolean).join(' ')));
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      creating = false;
    }
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      open = true;
      active = Math.min(active + 1, results.length - 1);
      e.preventDefault();
    } else if (e.key === 'ArrowUp') {
      active = Math.max(active - 1, 0);
      e.preventDefault();
    } else if (e.key === 'Enter') {
      if (open && results[active]) {
        pick(results[active]);
        e.preventDefault();
      }
    } else if (e.key === 'Escape') {
      open = false;
    }
  }
</script>

<div class="pp">
  {#if selected && !open}
    <div class="sel">
      <Avatar photo={selected.photo} name={selected.name} gender={selected.gender} size={26} />
      <button type="button" class="nm" onclick={() => { open = true; setTimeout(() => input?.focus()); }}>
        <span class="name">{selected.name}</span> <span class="muted small">{selected.years}</span>
      </button>
      <button type="button" class="btn ghost icon sm" aria-label={t('Remove')} onclick={() => pick(null)}><X size={16} /></button>
    </div>
  {:else}
    <input
      {id}
      bind:this={input}
      type="search"
      autocomplete="off"
      placeholder={placeholder || t('Search…')}
      bind:value={q}
      onfocus={() => (open = true)}
      oninput={() => { open = true; active = 0; }}
      onblur={() => setTimeout(() => (open = false), 160)}
      onkeydown={key}
      role="combobox"
      aria-expanded={open}
      aria-controls={listId}
      aria-autocomplete="list"
    />
  {/if}
  {#if open}
    <ul class="dd" role="listbox" id={listId}>
      {#each results as p, i (p.id)}
        <li role="option" aria-selected={i === active}>
          <button type="button" class:on={i === active} onmousedown={(e) => e.preventDefault()} onclick={() => pick(p)}>
            <Avatar photo={p.photo} name={p.name} gender={p.gender} size={26} />
            <span class="name">{p.name}</span>
            <span class="muted small">{p.years}</span>
          </button>
        </li>
      {:else}
        <li class="none muted small">{t('No results')}</li>
      {/each}
      {#if (oncreate || create) && q.trim()}
        <li>
          <button type="button" class="create" disabled={creating} onmousedown={(e) => e.preventDefault()} onclick={() => { const n = q.trim(); open = false; q = ''; createPerson(n); }}>
            + {t('Add new person: %s', q.trim())}
          </button>
        </li>
      {/if}
    </ul>
  {/if}
</div>

<style>
  .pp {
    position: relative;
    min-width: 0;
  }
  .sel {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 1.5px solid transparent;
    border-radius: 12px;
    padding: 4px 4px 4px 6px;
    min-height: 42px;
    background: var(--surface-2);
  }
  .sel .nm {
    flex: 1;
    min-width: 0;
    text-align: start;
    background: none;
    border: 0;
    font: inherit;
    color: inherit;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dd {
    position: absolute;
    z-index: 50;
    top: calc(100% + 4px);
    inset-inline: 0;
    margin: 0;
    padding: 4px;
    list-style: none;
    background: var(--surface);
    border-radius: 16px;
    box-shadow: var(--shadow-raised);
    max-height: 300px;
    overflow: auto;
  }
  .dd button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    border: 0;
    background: none;
    font: inherit;
    color: inherit;
    padding: 7px 9px;
    border-radius: 12px;
    cursor: pointer;
    text-align: start;
  }
  .dd button.on,
  .dd button:hover {
    background: var(--ink-soft);
  }
  .dd .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .none {
    padding: 8px;
  }
  .create {
    color: var(--ink-2);
    font-weight: 550;
  }
</style>
