<script lang="ts">
  import { summaries } from '../api.svelte';
  import { t } from '../i18n/index.svelte';
  import Avatar from './Avatar.svelte';
  import X from '@lucide/svelte/icons/x';
  import type { Summary } from '../types';

  let {
    value = $bindable(null),
    exclude = [],
    placeholder = '',
    gender = '',
    onchange,
    oncreate,
    id = undefined,
  }: {
    value?: number | null;
    exclude?: number[];
    placeholder?: string;
    gender?: string;
    onchange?: (id: number | null) => void;
    oncreate?: (name: string) => void;
    id?: string;
  } = $props();

  const listId = 'pp-' + Math.random().toString(36).slice(2);
  let all: Summary[] = $state([]);
  let q = $state('');
  let open = $state(false);
  let active = $state(0);
  let input: HTMLInputElement | undefined = $state();

  $effect(() => {
    summaries().then((s) => (all = s));
  });

  const selected = $derived(all.find((p) => p.id === value) || null);

  function norm(s: string) {
    return s
      .toLowerCase()
      .normalize('NFKD')
      .replace(/[̀-ͯ]/g, '')
      .replace(/ł/g, 'l');
  }

  const results = $derived.by(() => {
    const words = norm(q).split(/\s+/).filter(Boolean);
    let list = all.filter((p) => !exclude.includes(p.id));
    if (words.length) {
      list = list.filter((p) => {
        const hay = norm(`${p.name} ${p.maiden_name} ${p.years} ${p.birth_place}`);
        return words.every((w) => hay.includes(w));
      });
    }
    if (gender) list = [...list.filter((p) => p.gender === gender), ...list.filter((p) => p.gender !== gender)];
    return list.slice(0, 40);
  });

  function pick(p: Summary | null) {
    value = p ? p.id : null;
    q = '';
    open = false;
    onchange?.(value);
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
      {#if oncreate && q.trim()}
        <li>
          <button type="button" class="create" onmousedown={(e) => e.preventDefault()} onclick={() => { const n = q.trim(); open = false; q = ''; oncreate?.(n); }}>
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
