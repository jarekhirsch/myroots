<script lang="ts">
  // Search box in the menu: jumps to a person's profile.
  import { summaries } from '../api.svelte';
  import { go } from '../router.svelte';
  import { t } from '../i18n/index.svelte';
  import Avatar from './Avatar.svelte';
  import Search from '../icons/MagnifyingGlass.svelte';
  import ArrowRight from '../icons/ArrowRight.svelte';
  import { search, words } from '../text';
  import type { Summary } from '../types';

  let { compact = false, big = false, placeholder = '' }: { compact?: boolean; big?: boolean; placeholder?: string } = $props();
  let all: Summary[] = $state.raw([]);
  let q = $state('');
  let open = $state(false);
  let active = $state(0);
  let input: HTMLInputElement | undefined = $state();

  async function load() {
    all = await summaries();
  }

  const results = $derived.by(() => {
    const w = words(q);
    if (!w.length) return [];
    return search(all, 'quick', (p) => `${p.name} ${p.maiden_name} ${p.nickname || ''} ${p.years}`, w, 12, (p) => !p.anon);
  });

  function pick(p: Summary) {
    q = '';
    open = false;
    go(`/person/${p.id}`);
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      active = Math.min(active + 1, results.length - 1);
      e.preventDefault();
    } else if (e.key === 'ArrowUp') {
      active = Math.max(active - 1, 0);
      e.preventDefault();
    } else if (e.key === 'Enter' && results[active]) {
      pick(results[active]);
    } else if (e.key === 'Escape') {
      open = false;
      q = '';
    }
  }
</script>

<div class="qs" class:compact class:big>
  {#if !big}<Search size={16} class="ic" />{/if}
  <input
    type="search"
    placeholder={placeholder || (compact ? t('Search…') : t('Search for a person…'))}
    aria-label={t('Search for a person…')}
    bind:value={q}
    onfocus={() => {
      open = true;
      load();
    }}
    oninput={() => {
      open = true;
      active = 0;
    }}
    onblur={() => setTimeout(() => (open = false), 160)}
    onkeydown={key}
    autocomplete="off"
    bind:this={input}
  />
  {#if big}
    <button
      type="button"
      class="orb"
      aria-label={t('Search for a person…')}
      onmousedown={(e) => e.preventDefault()}
      onclick={() => (results[active] ? pick(results[active]) : input?.focus())}
    >
      {#if q.trim() && results.length}<ArrowRight size={22} />{:else}<Search size={21} />{/if}
    </button>
  {/if}
  {#if open && q.trim()}
    <ul class="dd">
      {#each results as p, i (p.id)}
        <li>
          <button type="button" class:on={i === active} onmousedown={(e) => e.preventDefault()} onclick={() => pick(p)}>
            <Avatar photo={p.photo} name={p.name} gender={p.gender} size={big ? 38 : 30} />
            <span class="nm name">{p.name}</span>
            <span class="muted small">{p.years}</span>
          </button>
        </li>
      {:else}
        <li class="none muted small">{t('No results')}</li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .qs {
    position: relative;
  }
  .qs :global(.ic) {
    position: absolute;
    inset-inline-start: 10px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--faint);
    pointer-events: none;
  }
  input {
    padding-inline-start: 36px !important;
    background: var(--surface) !important;
    border-radius: 999px !important;
    min-height: 40px !important;
    box-shadow: var(--shadow);
  }
  .qs :global(.ic) {
    inset-inline-start: 13px;
  }
  .big input {
    min-height: 64px !important;
    font-size: 1.12rem !important;
    padding-inline-start: 28px !important;
    padding-inline-end: 66px !important;
    box-shadow: var(--shadow-raised);
  }
  .orb {
    position: absolute;
    inset-inline-end: 8px;
    top: 8px;
    width: 48px;
    height: 48px;
    border: 0;
    border-radius: 50%;
    background: var(--ink);
    color: var(--surface);
    display: grid;
    place-items: center;
    cursor: pointer;
    transition: background 0.15s;
  }
  .orb:hover {
    background: var(--ink-2);
  }
  :global([dir='rtl']) .orb :global(svg) {
    transform: scaleX(-1);
  }
  .big .dd {
    width: 100%;
    border-radius: 20px;
    padding: 6px;
  }
  .compact input {
    min-height: 34px !important;
  }
  .dd {
    position: absolute;
    z-index: 80;
    top: calc(100% + 4px);
    inset-inline-start: 0;
    width: max(100%, 280px);
    margin: 0;
    padding: 4px;
    list-style: none;
    background: var(--surface);
    border-radius: 16px;
    box-shadow: var(--shadow-raised);
    max-height: 60vh;
    overflow: auto;
  }
  .compact .dd {
    inset-inline-start: auto;
    inset-inline-end: 0;
  }
  .dd button {
    display: flex;
    align-items: center;
    gap: 9px;
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
  .nm {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .none {
    padding: 8px;
  }
</style>
