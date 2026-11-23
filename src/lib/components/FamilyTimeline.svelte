<script lang="ts">
  // The whole family on one time axis: a bar from birth to death for every
  // person, grouped by generation. Pointing at a person lights up their
  // partners, parents and children. Generations fold into a thin overview,
  // and the axis zooms (buttons, or Ctrl / pinch on the scroll area).
  import { tick } from 'svelte';
  import { callShared } from '../api.svelte';
  import { useLoad } from '../load.svelte';
  import { t, tx, tn } from '../i18n/index.svelte';
  import { buildTimeline, packGeneration, type Bar, type Generation } from '../generations';
  import { tone } from '../text';
  import Avatar from './Avatar.svelte';
  import Plus from '@lucide/svelte/icons/zoom-in';
  import Minus from '@lucide/svelte/icons/zoom-out';
  import Fit from '@lucide/svelte/icons/move-horizontal';
  import Chevron from '@lucide/svelte/icons/chevron-down';
  import Fold from '@lucide/svelte/icons/fold-vertical';
  import Unfold from '@lucide/svelte/icons/unfold-vertical';
  import type { TreeNode } from '../types';

  let { only = null }: { only?: Set<string> | null } = $props();
  let width = $state(0);
  let scroller: HTMLDivElement | undefined = $state();
  // shared with the other timeline (start page, people list): it never changes the nodes
  const data = useLoad(() => callShared<{ nodes: TreeNode[] }>('tree.data'));
  const full = $derived(data.data ? buildTimeline(data.data.nodes) : null);
  // Filters of the people list keep the generations, but show only matching people.
  const tl = $derived.by(() => {
    if (!full || !only) return full;
    return {
      ...full,
      generations: full.generations.map((g) => ({ ...g, bars: g.bars.filter((b) => only.has(b.id)) })).filter((g) => g.bars.length),
      undated: full.undated.filter((p) => only.has(p.id)),
    };
  });

  // ---- remembered view: zoom and folded generations
  const KEY = 'myroots-timeline';
  type Saved = { zoom: number; folded: number[] | null };
  function readSaved(): Saved {
    try {
      const v = JSON.parse(localStorage.getItem(KEY) || 'null');
      if (v && typeof v.zoom === 'number') return { zoom: v.zoom, folded: Array.isArray(v.folded) ? v.folded : null };
    } catch {
      /* private mode or broken value */
    }
    return { zoom: 1, folded: null };
  }
  const saved = readSaved();
  let zoom = $state(saved.zoom);
  let folded = $state<number[] | null>(saved.folded);
  function save() {
    try {
      localStorage.setItem(KEY, JSON.stringify({ zoom, folded }));
    } catch {
      /* ignore */
    }
  }
  // Large families start with the most crowded generations folded.
  const foldedSet = $derived.by(() => {
    if (folded) return new Set(folded);
    const gs = full?.generations || [];
    const total = gs.reduce((n, g) => n + g.bars.length, 0);
    return new Set(total > 80 ? gs.filter((g) => g.bars.length > 24).map((g) => g.index) : []);
  });
  function toggle(i: number) {
    const s = new Set(foldedSet);
    if (s.has(i)) s.delete(i);
    else s.add(i);
    folded = [...s];
    save();
  }
  const allFolded = $derived(!!tl && tl.generations.every((g) => foldedSet.has(g.index)));
  function foldAll() {
    folded = allFolded ? [] : (tl?.generations || []).map((g) => g.index);
    save();
  }

  // ---- scale
  const narrow = $derived(!!width && width < 640);
  const LABEL = $derived(narrow ? 0 : 132);
  const from = $derived(tl ? Math.floor((tl.min - 6) / 10) * 10 : 1900);
  const to = $derived(tl ? Math.ceil(tl.max + 3) : 2030);
  const span = $derived(Math.max(30, to - from));
  const fitPpy = $derived(Math.max(0.5, (width - LABEL - 8) / span));
  const basePpy = $derived(Math.max(narrow ? 5.5 : 4.5, fitPpy));
  const MIN_PPY = 0.8;
  const MAX_PPY = 40;
  const ppy = $derived(Math.min(MAX_PPY, Math.max(MIN_PPY, basePpy * zoom)));
  const track = $derived(Math.ceil(span * ppy));
  // Below this scale the names no longer fit next to each other.
  const compact = $derived(ppy < 3.2);

  async function setZoom(z: number, anchorPx?: number) {
    const el = scroller;
    const view = el ? el.clientWidth : width;
    const at = anchorPx ?? view / 2;
    const year = el ? (el.scrollLeft + at - LABEL) / ppy + from : 0;
    zoom = Math.min(MAX_PPY / basePpy, Math.max(MIN_PPY / basePpy, z));
    save();
    await tick();
    if (el) el.scrollLeft = (year - from) * ppy + LABEL - at;
  }
  const fit = () => setZoom(fitPpy / basePpy);

  // Ctrl + wheel and trackpad pinch zoom around the pointer.
  $effect(() => {
    const el = scroller;
    if (!el) return;
    const wheel = (e: WheelEvent) => {
      if (!e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      setZoom(zoom * Math.exp(-e.deltaY * 0.004), e.clientX - el.getBoundingClientRect().left);
    };
    el.addEventListener('wheel', wheel, { passive: false });
    return () => el.removeEventListener('wheel', wheel);
  });

  // ---- bars
  const SIZES = { full: [40, 8], compact: [26, 6], mini: [12, 4] } as const;
  type Mode = keyof typeof SIZES;
  let ctx: CanvasRenderingContext2D | null = null;
  function textPx(s: string, font: string) {
    if (!ctx) ctx = document.createElement('canvas').getContext('2d');
    if (!ctx) return s.length * 7.5;
    ctx.font = font;
    return ctx.measureText(s).width;
  }
  const labelPx = (b: Bar) => 6 + 30 + 9 + textPx(b.name, "620 14.5px 'Onest Variable', system-ui") + 8 + textPx(b.years, "13px 'Onest Variable', system-ui") + 12;
  const gens = $derived.by((): (Generation & { mode: Mode })[] => {
    if (!tl || !width) return [];
    return tl.generations.map((g) => {
      const mode: Mode = foldedSet.has(g.index) ? 'mini' : compact ? 'compact' : 'full';
      // folded: one dot per person at the year of birth
      const reach =
        mode === 'full' ? (b: Bar) => Math.max(b.end - b.start, labelPx(b) / ppy) + 14 / ppy : mode === 'compact' ? (b: Bar) => b.end - b.start + 4 / ppy : () => 13 / ppy;
      return { ...packGeneration(g, reach, mode === 'mini' ? 5 : Infinity), mode };
    });
  });

  const x = (y: number) => (y - from) * ppy;
  const now = new Date();
  const nowY = now.getFullYear() + now.getMonth() / 12;
  const step = $derived([10, 25, 50, 100, 200].find((s) => s * ppy >= 150) || 200);
  const minor = $derived(10 * ppy >= 12 ? 10 : step);
  const marks = $derived.by(() => {
    const out: { y: number; big: boolean }[] = [];
    for (let y = Math.ceil(from / minor) * minor; y <= to; y += minor) out.push({ y, big: y % step === 0 });
    return out;
  });

  let hover = $state<string | null>(null);
  const lit = $derived(hover && tl ? tl.related.get(hover) : null);
</script>

{#if tl && (tl.generations.length || tl.undated.length)}
  <div class="ft" bind:clientWidth={width}>
    <div class="bar-tools no-print">
      {#if tl.generations.length > 1}
        <button class="btn ghost sm" onclick={foldAll}>
          {#if allFolded}<Unfold size={16} /> {tx('timeline', 'Expand all')}{:else}<Fold size={16} /> {tx('timeline', 'Collapse all')}{/if}
        </button>
      {/if}
      <span class="zoom" role="group" aria-label={t('Zoom in')}>
        <button class="btn ghost icon sm" onclick={() => setZoom(zoom / 1.5)} disabled={ppy <= MIN_PPY} aria-label={t('Zoom out')} title={t('Zoom out')}><Minus size={17} /></button>
        <button class="btn ghost icon sm" onclick={fit} aria-label={t('Fit to screen')} title={t('Fit to screen')}><Fit size={17} /></button>
        <button class="btn ghost icon sm" onclick={() => setZoom(zoom * 1.5)} disabled={ppy >= MAX_PPY} aria-label={t('Zoom in')} title={t('Zoom in')}><Plus size={17} /></button>
      </span>
    </div>
    <div class="scroll" dir="ltr" bind:this={scroller}>
      <div class="inner" style="width: {LABEL + track}px; --label: {LABEL}px">
        <div class="axis" aria-hidden="true">
          {#each marks as m (m.y)}
            {#if m.big && Math.abs(m.y - nowY) * ppy > 64}<span class="big" style="left: {LABEL + x(m.y)}px">{m.y}</span>{/if}
          {/each}
          <span class="now-tag" style="left: {LABEL + x(nowY)}px">{t('today')}</span>
        </div>
        <div class="grid" aria-hidden="true">
          {#each marks as m (m.y)}
            <span class="line" class:strong={m.big} style="left: {LABEL + x(m.y)}px"></span>
          {/each}
          <span class="now" style="left: {LABEL + x(nowY)}px"></span>
        </div>

        {#each gens as g (g.index)}
          {@const [H, G] = SIZES[g.mode]}
          <section class="gen" class:narrow class:folded={g.mode === 'mini'} aria-label={t('Generation %d', g.index + 1)}>
            <button class="gl" dir="auto" onclick={() => toggle(g.index)} aria-expanded={g.mode !== 'mini'}>
              <span class="gt"><Chevron size={15} class="chev" /><strong>{t('Generation %d', g.index + 1)}</strong></span>
              {#if g.from}<span>{t('born %s', g.from === g.to ? `\u2068${g.from}\u2069` : `\u2068${g.from}–${g.to}\u2069`)}</span>{/if}
              <span>{tn('%d person', '%d people', g.bars.length)}</span>
            </button>
            <!-- one listener per generation instead of four per person (thousands of bars in big trees) -->
            <div
              class="lanes {g.mode}"
              style="height: {g.lanes * (H + G) - G}px"
              role="presentation"
              onpointerover={(e) => (hover = (e.target as HTMLElement).closest?.('[data-id]')?.getAttribute('data-id') ?? null)}
              onpointerleave={() => (hover = null)}
              onfocusin={(e) => (hover = (e.target as HTMLElement).closest?.('[data-id]')?.getAttribute('data-id') ?? null)}
              onfocusout={() => (hover = null)}
            >
              {#each g.bars as b (b.id)}
                {@const n = tone(b.name || '?')}
                <a
                  href="#/person/{b.id}"
                  class="bar"
                  class:living={b.open === 'living'}
                  class:unknown={b.open === 'unknown'}
                  class:approx={b.approxStart}
                  data-id={b.id}
                  class:dim={g.mode !== 'mini' && lit && !lit.has(b.id)}
                  class:me={g.mode !== 'mini' && hover === b.id}
                  style="left: {x(b.start)}px; top: {b.lane * (H + G)}px; width: {g.mode === 'mini' ? 12 : Math.max(8, (b.end - b.start) * ppy)}px; --bg: var(--p{n}); --fg: var(--p{n}t)"
                  title={g.mode === 'full' ? undefined : `${b.name} ${b.years}`}
                  aria-label={g.mode === 'full' ? undefined : `${b.name} ${b.years}`}
                >
                  {#if g.mode === 'full'}
                    {#each b.marriages as m (m.partner + m.year)}
                      <span class="ring" style="left: {(m.year - b.start) * ppy - 5}px" aria-hidden="true"></span>
                    {/each}
                    <span class="lbl">
                      <Avatar photo={b.photo} name={b.name} gender={b.gender} size={30} />
                      <bdi class="nm">{b.name}</bdi>
                      <span class="yr">{b.years}</span>
                    </span>
                  {:else if g.mode === 'compact'}
                    <span class="lbl small-lbl" aria-hidden="true">
                      <Avatar photo={b.photo} name={b.name} gender={b.gender} size={20} />
                      <bdi class="nm">{b.name}</bdi>
                    </span>
                  {/if}
                </a>
              {/each}
            </div>
          </section>
        {/each}
      </div>
    </div>
    {#if tl.undated.length}
      <div class="undated">
        <span class="muted small">{t('Without dates')}</span>
        {#each tl.undated as p (p.id)}
          <a class="chip" href="#/person/{p.id}"><Avatar photo={p.photo} name={p.name} gender={p.gender} size={24} /><bdi>{p.name}</bdi></a>
        {/each}
      </div>
    {/if}
  </div>
{:else if data.data}
  <p class="muted">{tx('timeline', 'Add dates of birth to see the family on a timeline.')}</p>
{/if}

<style>
  .ft {
    min-width: 0;
    position: relative;
  }
  .bar-tools {
    display: flex;
    gap: 6px;
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
    margin-bottom: 2px;
  }
  .zoom {
    display: inline-flex;
    gap: 2px;
    padding: 2px;
    border-radius: 999px;
    background: var(--surface-2);
  }
  .zoom .btn {
    min-height: 32px;
    min-width: 32px;
  }
  .scroll {
    overflow-x: auto;
    overscroll-behavior-x: contain;
    padding-bottom: 6px;
  }
  .inner {
    position: relative;
    padding-bottom: 4px;
  }
  .axis {
    position: relative;
    height: 64px;
  }
  .big {
    position: absolute;
    top: 0;
    transform: translateX(-50%);
    font-size: 3.3rem;
    line-height: 1;
    font-weight: 220;
    letter-spacing: -0.04em;
    color: var(--faint);
    font-variant-numeric: lining-nums;
  }
  .now-tag {
    position: absolute;
    top: 30px;
    transform: translateX(-50%);
    font-size: 0.78rem;
    font-weight: 650;
    color: var(--ink);
    background: var(--ink-soft);
    padding: 1px 9px;
    border-radius: 999px;
    white-space: nowrap;
  }
  .grid {
    position: absolute;
    inset: 64px 0 0 0;
    pointer-events: none;
  }
  .line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: color-mix(in srgb, var(--line) 60%, transparent);
  }
  .line.strong {
    background: var(--line-strong);
  }
  .now {
    position: absolute;
    top: -10px;
    bottom: 0;
    width: 2px;
    margin-left: -1px;
    background: var(--ink);
    opacity: 0.7;
  }
  .gen {
    display: grid;
    grid-template-columns: var(--label) minmax(0, 1fr);
    padding: 14px 0;
    position: relative;
  }
  .gen + .gen {
    border-top: 1px dashed var(--line);
  }
  .gl {
    position: sticky;
    left: 0;
    z-index: 3;
    align-self: stretch;
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    align-items: flex-start;
    padding: 4px 12px 6px 2px;
    background: var(--surface);
    line-height: 1.3;
    border: 0;
    font: inherit;
    color: inherit;
    text-align: start;
    cursor: pointer;
    border-radius: 0;
  }
  .gt {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-inline-start: -2px;
  }
  .gl :global(.chev) {
    color: var(--muted);
    transition: transform 0.15s;
  }
  .folded .gl :global(.chev) {
    transform: rotate(-90deg);
  }
  .gl:hover strong {
    text-decoration: underline;
    text-decoration-color: var(--line-strong);
    text-underline-offset: 3px;
  }
  .gen.narrow {
    grid-template-columns: minmax(0, 1fr);
    gap: 8px;
    padding-top: 10px;
  }
  .narrow .gl {
    align-self: start;
    flex-direction: row;
    gap: 8px;
    align-items: baseline;
    width: max-content;
    padding: 0 8px 0 0;
    background: none;
  }
  .gl strong {
    font-weight: 650;
    font-size: 0.95rem;
  }
  .gen.folded {
    padding-block: 10px;
  }
  .mini .bar {
    border-radius: 50%;
    background: var(--fg);
    opacity: 0.6;
    margin-left: -6px;
  }
  .mini .bar.living,
  .mini .bar.unknown,
  .mini .bar.approx {
    border-radius: 50%;
    background: var(--fg);
    outline: 0;
  }
  .mini .bar:hover,
  .mini .bar.me {
    opacity: 1;
    box-shadow: none;
  }
  .mini .bar.dim {
    opacity: 0.15;
  }
  .compact .bar {
    border-radius: 9px;
  }
  .compact .bar {
    overflow: hidden;
  }
  .compact .lbl {
    margin: 3px 0 0 3px;
    gap: 6px;
  }
  .compact .nm {
    font-size: 0.8rem;
  }
  .gl span {
    font-size: 0.82rem;
    color: var(--muted);
  }
  .lanes {
    position: relative;
  }
  .bar {
    position: absolute;
    height: 40px;
  }
  .compact .bar {
    height: 26px;
  }
  .mini .bar {
    height: 12px;
  }
  .bar {
    border-radius: 13px;
    background: var(--bg);
    color: var(--fg);
    text-decoration: none;
    transition: opacity 0.15s, box-shadow 0.15s;
    z-index: 1;
  }
  .bar:hover,
  .bar:focus-visible,
  .bar.me {
    box-shadow: 0 0 0 2px var(--fg);
    z-index: 2;
  }
  .bar.dim {
    opacity: 0.28;
  }
  .bar.living {
    border-start-end-radius: 3px;
    border-end-end-radius: 3px;
  }
  .bar.unknown {
    background: linear-gradient(to right, var(--bg) 55%, color-mix(in srgb, var(--bg) 0%, transparent));
  }
  .bar.approx {
    outline: 1.5px dashed color-mix(in srgb, var(--fg) 45%, transparent);
    outline-offset: -1.5px;
  }
  /* the label stays in view while the start of a long life is scrolled away */
  .lbl {
    position: sticky;
    left: calc(var(--label) + 6px);
    margin: 5px 0 0 5px;
    display: inline-flex;
    align-items: center;
    gap: 9px;
    white-space: nowrap;
    pointer-events: none;
  }
  .lbl :global(.av) {
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--surface) 70%, transparent);
  }
  .nm {
    font-weight: 620;
    font-size: 0.94rem;
    color: var(--text);
  }
  .yr {
    font-size: 0.84rem;
  }
  .ring {
    position: absolute;
    top: -5px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--surface);
    box-shadow: 0 0 0 2px var(--heirloom);
  }
  .undated {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    margin-top: 14px;
    padding-top: 14px;
    border-top: 1px solid var(--line);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 3px 12px 3px 3px;
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--text);
    text-decoration: none;
    font-size: 0.88rem;
  }
  @media (max-width: 640px) {
    .big {
      font-size: 2.4rem;
    }
  }
</style>
