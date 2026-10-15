<script lang="ts">
  // The whole family on one board: every person and every connection.
  // Drag to move, scroll or pinch to zoom. Pointing at a person lights up
  // their partners, parents and children; a click selects them.
  import { select } from 'd3-selection';
  import { zoom as d3zoom, zoomIdentity, type ZoomBehavior } from 'd3-zoom';
  import 'd3-transition';
  import { backend } from '../backend';
  import { t, tx } from '../i18n/index.svelte';
  import { initials } from '../format';
  import { tone } from '../text';
  import { layoutFamily, type GNode } from '../familygraph';
  import type { TreeNode } from '../types';

  let {
    nodes,
    orient = 'vertical',
    selected = $bindable(null),
  }: { nodes: TreeNode[]; orient?: 'vertical' | 'horizontal'; selected?: string | null } = $props();
  const START_MIN = 0.35; // below this the whole board is unreadable: start at a person instead

  const graph = $derived(layoutFamily(nodes, orient, t('Private')));
  const related = $derived.by(() => {
    const m = new Map<string, Set<string>>();
    for (const n of nodes) m.set(n.id, new Set([n.id, ...n.rels.parents, ...n.rels.spouses, ...n.rels.children]));
    return m;
  });

  let svg: SVGSVGElement | undefined = $state();
  let transform = $state('translate(0,0) scale(1)');
  let z: ZoomBehavior<SVGSVGElement, unknown> | null = null;
  let hover = $state<string | null>(null);
  const lit = $derived(hover ? related.get(hover) : null);

  $effect(() => {
    if (!svg) return;
    z = d3zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.04, 2.5])
      .on('zoom', (e) => {
        const k = e.transform;
        // never keep a broken position (it would freeze the board)
        if (!Number.isFinite(k.x) || !Number.isFinite(k.y) || !Number.isFinite(k.k)) {
          if (svg) select(svg).call(z!.transform as any, zoomIdentity);
          return;
        }
        transform = `translate(${k.x},${k.y}) scale(${k.k})`;
      });
    select(svg).call(z).on('dblclick.zoom', null);
  });

  // fit the first time and after turning the board; edits keep the view
  let fittedFor = '';
  $effect(() => {
    void graph;
    if (svg && graph.nodes.length && fittedFor !== orient) {
      fittedFor = orient;
      queueMicrotask(() => {
        if (fitScale() >= START_MIN || !selected) fit(false);
        else focus(selected, false);
      });
    }
  });
  function fitScale() {
    const w = svg?.clientWidth || 800;
    const h = svg?.clientHeight || 600;
    return Math.min(w / (graph.width + 80), h / (graph.height + 80));
  }

  export function fit(animate = true) {
    if (!svg || !z || !graph.nodes.length) return;
    const w = svg.clientWidth || 800;
    const h = svg.clientHeight || 600;
    const k = Math.max(0.04, Math.min(1.1, Math.min(w / (graph.width + 80), h / (graph.height + 80))));
    const tr = zoomIdentity.translate((w - graph.width * k) / 2, (h - graph.height * k) / 2).scale(k);
    const s = select(svg);
    if (animate) s.transition().duration(450).call(z.transform as any, tr);
    else s.call(z.transform as any, tr);
  }
  export function zoomBy(f: number) {
    if (svg && z) select(svg).transition().duration(250).call(z.scaleBy as any, f);
  }
  export function focus(id: string, animate = true) {
    const n = graph.nodes.find((x) => x.id === id);
    if (!n || !svg || !z) return;
    const w = svg.clientWidth || 800;
    const h = svg.clientHeight || 600;
    const k = 0.9;
    const tr = zoomIdentity.translate(w / 2 - (n.x + graph.cardW / 2) * k, h / 2 - (n.y + graph.cardH / 2) * k).scale(k);
    const s = select(svg);
    if (animate) s.transition().duration(500).call(z.transform as any, tr);
    else s.call(z.transform as any, tr);
    selected = id;
  }

  /** A standalone SVG of the whole board (colours resolved, initials instead of photos). */
  export function exportSvg(): string {
    if (!svg) return '';
    const css = getComputedStyle(svg);
    const clone = svg.cloneNode(true) as SVGSVGElement;
    clone.querySelector('g.board')?.setAttribute('transform', 'translate(40,40)');
    clone.querySelectorAll('image').forEach((i) => i.remove());
    clone.querySelectorAll('.ini').forEach((i) => i.removeAttribute('display'));
    clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
    clone.setAttribute('width', String(graph.width + 80));
    clone.setAttribute('height', String(graph.height + 80));
    clone.setAttribute('viewBox', `0 0 ${graph.width + 80} ${graph.height + 80}`);
    let s = new XMLSerializer().serializeToString(clone);
    s = s.replace(/var\((--[a-z0-9-]+)\)/gi, (_, v) => css.getPropertyValue(v).trim() || '#000');
    const font = css.getPropertyValue('--font-ui').trim();
    return `<?xml version="1.0" encoding="UTF-8"?>\n${s.replace('<svg', `<svg style="font-family:${font.replace(/"/g, "'")};background:#fff"`)}`;
  }

  // names cut to the width of the card
  let ctx: CanvasRenderingContext2D | null = null;
  function short(s: string, max = graph.cardW - 76) {
    if (!ctx) ctx = document.createElement('canvas').getContext('2d');
    if (!ctx) return s.length > 18 ? s.slice(0, 17) + '…' : s;
    ctx.font = "620 14.5px 'Onest Variable', system-ui";
    if (ctx.measureText(s).width <= max) return s;
    let n = s.length;
    while (n > 1 && ctx.measureText(s.slice(0, n) + '…').width > max) n--;
    return s.slice(0, n).trimEnd() + '…';
  }
  function pick(n: GNode) {
    if (!n.anon) selected = n.id;
  }
</script>

<svg class="fg" bind:this={svg} role="application" aria-label={tx('tree mode', 'Whole family')}>
  <defs>
    <clipPath id="fg-av" clipPathUnits="objectBoundingBox"><rect width="1" height="1" rx="0.3" ry="0.3" /></clipPath>
  </defs>
  <g class="board" {transform}>
    <g class="edges">
      {#each graph.edges as e, i (i)}
        <path d={e.d} class={e.kind} class:odd={e.odd} class:dim={lit && !e.ids.some((x) => x === hover)} class:on={lit && e.ids.includes(hover!)} />
      {/each}
    </g>
    {#each graph.nodes as n (n.id)}
      {@const tn = tone(n.name)}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <g
        class="card"
        class:anon={n.anon}
        class:sel={selected === n.id}
        class:dim={lit && !lit.has(n.id)}
        transform="translate({n.x},{n.y})"
        role="button"
        tabindex={n.anon ? -1 : 0}
        aria-label="{n.name} {n.years}"
        onclick={() => pick(n)}
        onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), pick(n))}
        onpointerenter={() => (hover = n.id)}
        onpointerleave={() => (hover = null)}
        onfocus={() => (hover = n.id)}
        onblur={() => (hover = null)}
      >
        <title>{n.name} {n.years}</title>
        <rect class="bg" width={graph.cardW} height={graph.cardH} rx="16" />
        <rect x="10" y="10" width="44" height="44" rx="14" fill="var(--p{tn})" />
        <text class="ini" x="32" y="37.5" text-anchor="middle" fill="var(--p{tn}t)" display={n.photo && !n.anon ? 'none' : undefined}>{n.anon ? '?' : initials(n.name)}</text>
        {#if n.photo && !n.anon}
          <image href={backend.mediaUrl(n.photo, true)} x="10" y="10" width="44" height="44" preserveAspectRatio="xMidYMid slice" clip-path="url(#fg-av)" />
        {/if}
        <text class="nm" x="66" y={n.years ? 29 : 37}>{short(n.name)}</text>
        {#if n.years}<text class="yr" x="66" y="47">{n.years}</text>{/if}
      </g>
    {/each}
  </g>
</svg>

<style>
  .fg {
    width: 100%;
    height: 100%;
    display: block;
    cursor: grab;
    touch-action: none;
    user-select: none;
  }
  .fg:active {
    cursor: grabbing;
  }
  path {
    fill: none;
    stroke: var(--line-strong);
    stroke-width: 2;
    stroke-linejoin: round;
    transition: opacity 0.15s;
  }
  path.couple {
    stroke: var(--heirloom);
    stroke-width: 3;
  }
  path.odd {
    stroke-dasharray: 6 5;
  }
  path.on {
    stroke: var(--ink);
    stroke-width: 3;
  }
  path.dim {
    opacity: 0.25;
  }
  .card {
    cursor: pointer;
    transition: opacity 0.15s;
    outline: none;
  }
  .card .bg {
    fill: var(--surface);
    stroke: var(--line);
    stroke-width: 1;
  }
  .card:hover .bg,
  .card:focus-visible .bg {
    stroke: var(--line-strong);
    stroke-width: 2;
  }
  .card.sel .bg {
    stroke: var(--ink);
    stroke-width: 3;
  }
  .card.dim {
    opacity: 0.3;
  }
  .card.anon {
    cursor: default;
  }
  .ini {
    font-size: 15px;
    font-weight: 650;
  }
  .nm {
    font-size: 14.5px;
    font-weight: 620;
    fill: var(--text);
  }
  .yr {
    font-size: 12.5px;
    fill: var(--muted);
  }
</style>
