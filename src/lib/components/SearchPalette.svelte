<script lang="ts">
  // Quick search (Ctrl K / Cmd K, "Search" at the top of the menu): people first, then
  // pages of the app and, for the editor, "Add a person". Arrows choose, Enter opens, Esc closes.
  import { summaries, app } from '../api.svelte';
  import { go } from '../router.svelte';
  import { t } from '../i18n/index.svelte';
  import { NAV, visible } from '../nav';
  import { search, words } from '../text';
  import Avatar from './Avatar.svelte';
  import Search from '../icons/MagnifyingGlass.svelte';
  import UserPlus from '../icons/UserPlus.svelte';
  import CaretRight from '../icons/CaretRight.svelte';
  import type { Summary } from '../types';

  let { onclose }: { onclose: () => void } = $props();
  let all: Summary[] = $state.raw([]);
  let q = $state('');
  let active = $state(0);
  let input: HTMLInputElement | undefined = $state();
  const viewer = $derived(!!app.info?.viewer);

  $effect(() => {
    summaries().then((s) => (all = s)).catch(() => {});
    const prev = document.activeElement as HTMLElement | null;
    input?.focus();
    document.body.style.overflow = 'hidden';
    return () => {
      document.body.style.overflow = '';
      prev?.focus?.();
    };
  });

  type Hit = { kind: 'person'; p: Summary } | { kind: 'page'; path: string; label: string; icon: any } | { kind: 'add'; label: string };
  const hits = $derived.by((): Hit[] => {
    const w = words(q);
    if (!w.length) return [];
    const people: Hit[] = search(all, 'quick', (p) => `${p.name} ${p.maiden_name} ${p.nickname || ''} ${p.years} ${p.birth_place}`, w, 8, (p) => !p.anon).map((p) => ({ kind: 'person', p }));
    const pages: Hit[] = NAV.flatMap((g) => g.items)
      .filter((i) => visible(i, viewer, app.info?.sections))
      .filter((i) => w.every((x) => i.label().toLowerCase().includes(x)))
      .slice(0, 4)
      .map((i) => ({ kind: 'page', path: i.path, label: i.label(), icon: i.icon }));
    const add: Hit[] = viewer ? [] : [{ kind: 'add', label: q.trim() }];
    return [...people, ...pages, ...add];
  });
  $effect(() => {
    void hits;
    active = 0;
  });

  function open(h: Hit) {
    onclose();
    if (h.kind === 'person') go(`/person/${h.p.id}`);
    else if (h.kind === 'page') go(h.path);
    else go('/person/new');
  }
  function key(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      active = Math.min(active + 1, hits.length - 1);
      e.preventDefault();
    } else if (e.key === 'ArrowUp') {
      active = Math.max(active - 1, 0);
      e.preventDefault();
    } else if (e.key === 'Enter' && hits[active]) {
      e.preventDefault();
      open(hits[active]);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onclose();
    }
  }
  const place = (p: Summary) => p.birth_place || p.residence || p.death_place || '';
</script>

<div class="ov no-print" role="presentation" onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="pal" role="dialog" aria-modal="true" aria-label={t('Search')}>
    <div class="bar">
      <Search size={22} />
      <input
        bind:this={input}
        bind:value={q}
        onkeydown={key}
        type="search"
        placeholder={t('Search for a person…')}
        aria-label={t('Search for a person…')}
        role="combobox"
        aria-expanded={hits.length > 0}
        aria-controls="pal-list"
        aria-activedescendant={hits.length ? `pal-${active}` : undefined}
        autocomplete="off"
      />
      <kbd>Esc</kbd>
    </div>
    {#if q.trim()}
      <ul id="pal-list" role="listbox">
        {#each hits as h, i (i)}
          {#if i === 0 || hits[i - 1].kind !== h.kind}
            <li class="grp" role="presentation">{h.kind === 'person' ? t('People') : h.kind === 'page' ? t('Pages') : t('Actions')}</li>
          {/if}
          <li id="pal-{i}" role="option" aria-selected={i === active}>
            <button type="button" class:on={i === active} onmousemove={() => (active = i)} onclick={() => open(h)}>
              {#if h.kind === 'person'}
                <Avatar photo={h.p.photo} name={h.p.name} gender={h.p.gender} size={40} />
                <span class="txt"><span class="nm name">{h.p.name}</span><span class="sub">{[h.p.years, place(h.p)].filter(Boolean).join(' · ')}</span></span>
              {:else if h.kind === 'page'}
                {@const Icon = h.icon}
                <span class="ic"><Icon size={20} /></span>
                <span class="txt"><span class="nm">{h.label}</span></span>
              {:else}
                <span class="ic"><UserPlus size={20} /></span>
                <span class="txt"><span class="nm">{t('Add a person')}</span></span>
              {/if}
              <CaretRight size={16} />
            </button>
          </li>
        {:else}
          <li class="none">{t('No results')}</li>
        {/each}
      </ul>
    {:else}
      <p class="tip">{t('Type a name, a place or a year.')}</p>
    {/if}
  </div>
</div>

<style>
  .ov {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(18, 22, 19, 0.32);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 12vh 16px 16px;
    animation: fade 0.3s ease both;
  }
  .pal {
    width: min(660px, 100%);
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-radius: 28px;
    box-shadow: var(--shadow-pop);
    overflow: hidden;
    animation: pop 0.4s cubic-bezier(0.2, 0.8, 0.2, 1) both;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 18px 14px 22px;
    border-bottom: 1px solid var(--line);
    color: var(--muted);
  }
  .bar input {
    flex: 1;
    border: 0 !important;
    background: none !important;
    box-shadow: none !important;
    font-size: 1.12rem !important;
    padding: 8px 0 !important;
    min-height: 44px;
  }
  kbd {
    font: inherit;
    font-size: 0.75rem;
    color: var(--muted);
    background: var(--surface-2);
    border-radius: 8px;
    padding: 3px 8px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 8px 10px 12px;
    overflow-y: auto;
  }
  .grp {
    font-size: 0.8rem;
    color: var(--muted);
    padding: 10px 12px 4px;
  }
  li button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border: 0;
    border-radius: 16px;
    background: none;
    font: inherit;
    color: var(--text);
    text-align: start;
    cursor: pointer;
  }
  li button.on {
    background: var(--surface-2);
  }
  li button > :global(svg:last-child) {
    color: var(--faint);
  }
  .ic {
    width: 40px;
    height: 40px;
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--ink);
    display: grid;
    place-items: center;
    flex: none;
  }
  .txt {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .nm {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub {
    font-size: 0.85rem;
    color: var(--muted);
  }
  .none,
  .tip {
    padding: 18px 22px;
    margin: 0;
    color: var(--muted);
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(-12px) scale(0.97);
    }
  }
</style>
