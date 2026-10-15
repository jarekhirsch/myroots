<script lang="ts">
  import { tick } from 'svelte';
  import BackButton from '../lib/components/BackButton.svelte';
  // Interactive family tree (family-chart + d3). Click a card to center it,
  // click the centered card again to open the profile.
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { go } from '../lib/router.svelte';
  import { backend } from '../lib/backend';
  import { t, tx } from '../lib/i18n/index.svelte';
  import { toast } from '../lib/ui.svelte';
  import { initials } from '../lib/format';
  import { tone } from '../lib/text';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import FamilyGraph from '../lib/components/FamilyGraph.svelte';
  import TreeDetails from '../lib/components/TreeDetails.svelte';
  import PanelIcon from '@lucide/svelte/icons/panel-right';
  import ZoomIn from '@lucide/svelte/icons/zoom-in';
  import ZoomOut from '@lucide/svelte/icons/zoom-out';
  import Download from '@lucide/svelte/icons/download';
  import UserPlus from '@lucide/svelte/icons/user-plus';
  import NotFound from './NotFound.svelte';
  import Rotate from '@lucide/svelte/icons/rotate-cw-square';
  import Fit from '@lucide/svelte/icons/locate-fixed';
  import Full from '@lucide/svelte/icons/maximize';
  import Pin from '@lucide/svelte/icons/pin';
  import 'family-chart/styles/family-chart.css';
  import type { TreeNode } from '../lib/types';

  let { id = null }: { id?: number | null } = $props();
  const viewer = $derived(!!app.info?.viewer);

  let cont: HTMLDivElement | undefined = $state();
  let wrap: HTMLDivElement | undefined = $state();
  let chart: any = null;
  let nodes = $state<TreeNode[]>([]);
  let main = $state<number | null>(null);
  let pick = $state<number | null>(null);
  let error = $state<unknown>(null);
  let empty = $state(false);
  let orient = $state<'vertical' | 'horizontal'>(stored('myroots-tree-orient', 'vertical') as any);
  let anc = $state(Number(stored('myroots-tree-anc', '0')) || 0);
  let prog = $state(Number(stored('myroots-tree-prog', '0')) || 0);
  // "person": the tree around one person; "all": the whole family on one board
  let mode = $state<'person' | 'all'>(stored('myroots-tree-mode', 'person') === 'all' ? 'all' : 'person');
  // the whole family reads best with generations as columns (like a poster)
  let gorient = $state<'vertical' | 'horizontal'>(stored('myroots-graph-orient', 'horizontal') === 'vertical' ? 'vertical' : 'horizontal');
  let graph: FamilyGraph | undefined = $state();
  let selected = $state<string | null>(null);
  let panelOpen = $state(true);

  async function setMode(m: 'person' | 'all') {
    if (m === mode) return;
    mode = m;
    store('myroots-tree-mode', m);
    if (m === 'person') {
      if (selected) main = Number(selected);
      await tick();
      await build();
    } else {
      selected = main ? String(main) : null;
    }
  }
  function centerOn(pid: number) {
    setMode('person').then(() => setMain(pid));
  }
  async function saveSvg() {
    const svgText = graph?.exportSvg();
    if (!svgText) return;
    const name = (app.info?.tree_name || 'MyRoots').replace(/[\\/:*?"<>|]+/g, ' ').trim() || 'MyRoots';
    const saved = await backend.saveFile(`${name}.svg`, svgText, 'image/svg+xml');
    if (saved) toast(t('Saved.'));
  }

  function stored(k: string, d: string) {
    try {
      return localStorage.getItem(k) || d;
    } catch {
      return d;
    }
  }
  function store(k: string, v: string) {
    try {
      localStorage.setItem(k, v);
    } catch {
      /* ignore */
    }
  }

  function esc(s: string) {
    return String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!);
  }

  function cardHtml(d: any): string {
    const n = d.data.data;
    if (n.anon) {
      return `<div class="mr-card anon"><div class="mr-av"><span>?</span></div><div class="mr-txt"><div class="mr-nm">${esc(t('Private'))}</div></div></div>`;
    }
    const name = `${n['first name'] || ''} ${n['last name'] || ''}`.trim() || '—';
    const src = n.avatar ? backend.mediaUrl(n.avatar, true) : '';
    const n2 = tone(name);
    const av = src ? `<img src="${esc(src)}" alt="" loading="lazy">` : `<span style="background:var(--p${n2});color:var(--p${n2}t)">${esc(initials(name))}</span>`;
    const maiden = n.maiden_name ? `<div class="mr-mn">${esc(t('née %s', n.maiden_name))}</div>` : '';
    const clans = (n.clans || []).slice(0, 1).map((c: string) => `<span class="mr-clan">${esc(c)}</span>`).join('');
    const pets = (n.pets || []).length ? `<span class="mr-pet" title="${esc(n.pets.map((p: any) => p.name).join(', '))}">🐾</span>` : '';
    return `<div class="mr-card"><div class="mr-av">${av}</div><div class="mr-txt"><div class="mr-nm">${esc(name)}</div>${maiden}<div class="mr-yr">${esc(n.years || '')}</div><div class="mr-tags">${clans}${pets}</div></div></div>`;
  }

  async function build() {
    if (!cont || mode !== 'person') return;
    const f3 = await import('family-chart');
    const data = nodes.map((n) => ({ ...n, data: { ...n.data } }));
    cont.innerHTML = '';
    chart = f3.createChart(cont, data as any);
    chart.setTransitionTime(600).setCardXSpacing(244).setCardYSpacing(orient === 'vertical' ? 150 : 270).setShowSiblingsOfMain(true).setSingleParentEmptyCard(false);
    if (orient === 'horizontal') chart.setOrientationHorizontal();
    else chart.setOrientationVertical();
    if (anc) chart.setAncestryDepth(anc);
    if (prog) chart.setProgenyDepth(prog);
    chart
      .setCardHtml()
      .setCardDim({ w: 216, h: 84 })
      .setCardInnerHtmlCreator(cardHtml)
      .setOnCardClick((e: MouseEvent, d: any) => {
        if (d.data.data.anon) return;
        const pid = Number(d.data.id);
        if (pid === main) go(`/person/${pid}`);
        else setMain(pid);
      });
    if (main) chart.updateMainId(String(main));
    chart.updateTree({ initial: true });
  }

  function setMain(pid: number) {
    main = pid;
    pick = pid;
    panelOpen = true;
    if (mode === 'all') {
      graph?.focus(String(pid));
      return;
    }
    if (chart) {
      chart.updateMainId(String(pid));
      chart.updateTree({ tree_position: 'main_to_middle' });
    }
    history.replaceState(null, '', `#/tree/${pid}`);
  }

  $effect(() => {
    void app.version;
    const want = id;
    (async () => {
      try {
        const r = await call<{ nodes: TreeNode[]; start: number | null }>('tree.data');
        empty = !r.nodes.length;
        if (empty) return;
        await backend.ensureMedia?.(r.nodes.map((n) => n.data.avatar).filter((x): x is number => !!x));
        nodes = r.nodes;
        const ids = new Set(r.nodes.filter((n) => !n.data.anon).map((n) => Number(n.id)));
        main = want && ids.has(want) ? want : main && ids.has(main) ? main : r.start ?? Number(r.nodes[0].id);
        pick = main;
        if (mode === 'all' && !selected && main) selected = String(main);
        await build();
      } catch (e) {
        error = e;
      }
    })();
  });

  function rotate() {
    if (mode === 'all') {
      gorient = gorient === 'vertical' ? 'horizontal' : 'vertical';
      store('myroots-graph-orient', gorient);
      return;
    }
    orient = orient === 'vertical' ? 'horizontal' : 'vertical';
    store('myroots-tree-orient', orient);
    build();
  }
  const panelId = $derived(mode === 'all' ? (selected ? Number(selected) : null) : main);
  $effect(() => {
    void selected;
    panelOpen = true;
  });
  // the tree area changes width when the panel opens or closes: keep the person in the middle
  let lastPanel: boolean | null = null;
  $effect(() => {
    const shown = !!(panelId && panelOpen);
    if (lastPanel !== null && lastPanel !== shown && mode === 'person' && chart) {
      tick().then(() => chart?.updateTree({ tree_position: 'main_to_middle' }));
    }
    lastPanel = shown;
  });
  function depth() {
    store('myroots-tree-anc', String(anc));
    store('myroots-tree-prog', String(prog));
    build();
  }
  function fit() {
    if (mode === 'all') graph?.fit();
    else chart?.updateTree({ tree_position: 'fit' });
  }
  function full() {
    if (document.fullscreenElement) document.exitFullscreen();
    else wrap?.requestFullscreen?.().catch(() => {});
  }
  async function setStart() {
    if (!app.settings || !main) return;
    await call('settings.save', { settings: { ...app.settings, tree_start_mode: 'person', tree_start_person: main } });
    await refreshInfo();
    toast(t('This person will open first in the tree.'));
  }

</script>

{#if error}
  <NotFound private={(error as any)?.code === 'private'} />
{:else if empty}
  <div class="page">
    <div class="panel empty">
      <h2>{t('The tree is empty')}</h2>
      {#if !viewer}<a class="btn primary" href="#/person/new">{t('Add the first person')}</a>{/if}
    </div>
  </div>
{:else}
  <div class="tree-wrap" bind:this={wrap}>
    <div class="bar no-print">
      <BackButton pill />
      <div class="modes" role="group" aria-label={t('View')}>
        <button aria-pressed={mode === 'person'} onclick={() => setMode('person')}>{tx('tree mode', 'Around a person')}</button>
        <button aria-pressed={mode === 'all'} onclick={() => setMode('all')}>{tx('tree mode', 'Whole family')}</button>
      </div>
      <div class="pick"><PersonPicker bind:value={pick} placeholder={t('Search for a person in the tree…')} onchange={(v) => v && setMain(v)} /></div>
      <div class="tools">
        {#if mode === 'person'}
          <label class="small muted dep">
            {t('Ancestors')}
            <select bind:value={anc} onchange={depth}>
              <option value={0}>{t('All')}</option>
              {#each [1, 2, 3, 4, 5, 6] as n}<option value={n}>{n}</option>{/each}
            </select>
          </label>
          <label class="small muted dep">
            {t('Descendants')}
            <select bind:value={prog} onchange={depth}>
              <option value={0}>{t('All')}</option>
              {#each [1, 2, 3, 4, 5, 6] as n}<option value={n}>{n}</option>{/each}
            </select>
          </label>
        {:else}
          <button class="btn icon ghost" onclick={() => graph?.zoomBy(1 / 1.4)} title={t('Zoom out')} aria-label={t('Zoom out')}><ZoomOut size={18} /></button>
          <button class="btn icon ghost" onclick={() => graph?.zoomBy(1.4)} title={t('Zoom in')} aria-label={t('Zoom in')}><ZoomIn size={18} /></button>
        {/if}
        <button class="btn icon ghost" onclick={rotate} title={(mode === 'all' ? gorient : orient) === 'vertical' ? t('Horizontal') : t('Vertical')} aria-label={t('Change orientation')}><Rotate size={18} /></button>
        <button class="btn icon ghost" onclick={fit} title={t('Fit to screen')} aria-label={t('Fit to screen')}><Fit size={18} /></button>
        <button class="btn icon ghost" onclick={full} title={t('Full screen')} aria-label={t('Full screen')}><Full size={18} /></button>
        {#if mode === 'all'}
          <button class="btn icon ghost" onclick={saveSvg} title={tx('tree', 'Save as an image (SVG)')} aria-label={tx('tree', 'Save as an image (SVG)')}><Download size={18} /></button>
        {/if}
        {#if mode === 'person' && !viewer && main && app.settings?.tree_start_person !== main}
          <button class="btn icon ghost" onclick={setStart} title={t('Open the tree with this person')} aria-label={t('Open the tree with this person')}><Pin size={18} /></button>
        {/if}
        {#if panelId && !panelOpen}
          <button class="btn icon ghost" onclick={() => (panelOpen = true)} title={t('Details')} aria-label={t('Details')}><PanelIcon size={18} /></button>
        {/if}
        {#if !viewer}<a class="btn sm primary add" href="#/person/new" title={t('Add person')}><UserPlus size={15} /> <span>{t('Add person')}</span></a>{/if}
      </div>
    </div>
    <div class="body">
      {#if mode === 'person'}
        <div class="f3 chart" bind:this={cont}></div>
      {:else}
        <div class="chart graph">{#if nodes.length}<FamilyGraph bind:this={graph} {nodes} orient={gorient} bind:selected />{/if}</div>
      {/if}
      {#if panelId && panelOpen}
        <div class="side no-print">
          <TreeDetails
            id={panelId}
            onclose={() => (panelOpen = false)}
            oncenter={mode === 'all' ? () => centerOn(panelId!) : null}
            onselect={(pid) => setMain(pid)}
          />
        </div>
      {/if}
    </div>
    {#if mode === 'person'}<p class="help small faint no-print" class:beside={panelId && panelOpen}>{t('Click a person to center the tree on them. Click again to open the profile. Drag to move, scroll or pinch to zoom.')}</p>{/if}
  </div>
{/if}

<style>
  .tree-wrap {
    position: relative;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: radial-gradient(circle at 1px 1px, color-mix(in srgb, var(--ink) 14%, transparent) 1px, transparent 0) 0 0 / 22px 22px, var(--paper);
  }
  .bar {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
    padding: 14px 20px;
  }
  .pick {
    flex: 1 1 220px;
    max-width: 340px;
  }
  .modes {
    display: flex;
    gap: 2px;
    padding: 3px;
    border-radius: 999px;
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  .modes button {
    border: 0;
    background: none;
    font: inherit;
    font-size: 0.9rem;
    font-weight: 540;
    color: var(--muted);
    padding: 7px 14px;
    border-radius: 999px;
    cursor: pointer;
    white-space: nowrap;
  }
  .modes button[aria-pressed='true'] {
    background: var(--ink);
    color: var(--surface);
    font-weight: 640;
  }
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    position: relative;
  }
  .side {
    width: 370px;
    flex-shrink: 0;
    padding: 0 18px 18px 0;
    box-sizing: border-box;
    z-index: 20;
  }
  :global([dir='rtl']) .side {
    padding: 0 0 18px 18px;
  }
  .graph {
    cursor: grab;
  }
  .bar :global(input),
  .bar :global(.sel),
  .bar select {
    background-color: var(--surface);
    box-shadow: var(--shadow);
    border-radius: 999px;
  }
  .tools .btn.icon {
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  .tools {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
  }
  .dep {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .dep select {
    width: auto;
    min-height: 32px;
    padding: 4px 8px;
  }
  .chart {
    flex: 1;
    min-width: 0;
    min-height: 0;
    position: relative;
    overflow: hidden;
    --female-color: var(--female);
    --male-color: var(--male);
    --genderless-color: var(--faint);
    --background-color: var(--paper);
    --text-color: var(--text);
    font-family: var(--font-ui);
    color: var(--text);
  }
  .help {
    position: absolute;
    bottom: 8px;
    inset-inline-start: 18px;
    margin: 0;
    pointer-events: none;
  }
  .chart :global(.link) {
    stroke: var(--line-strong) !important;
    stroke-width: 2px;
  }
  .chart :global(.link.f3-path-to-main) {
    stroke: var(--heirloom) !important;
  }
  /* the board can be dragged: a hand everywhere except on the people */
  .chart.f3 {
    cursor: grab;
  }
  .chart.f3:active {
    cursor: grabbing;
  }
  .chart :global(div.card) {
    cursor: pointer;
  }
  .chart :global(.mr-card) {
    width: 216px;
    min-height: 76px;
    display: flex;
    gap: 12px;
    align-items: center;
    padding: 10px 14px 10px 10px;
    background: var(--surface);
    border-radius: 20px;
    box-shadow: 0 1px 2px rgba(23, 32, 27, 0.05), 0 6px 18px rgba(23, 32, 27, 0.07);
    text-align: start;
    color: var(--text);
    transition: box-shadow 0.15s, transform 0.15s;
    position: relative;
  }
  .chart :global(.mr-card::after) {
    content: '';
    position: absolute;
    top: 12px;
    inset-inline-end: 12px;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--faint);
    opacity: 0.5;
  }
  .chart :global(.card-male .mr-card::after) {
    background: var(--male);
    opacity: 0.9;
  }
  .chart :global(.card-female .mr-card::after) {
    background: var(--female);
    opacity: 0.9;
  }
  .chart :global(.card-main .mr-card) {
    background: var(--ink);
    color: var(--surface);
    box-shadow: 0 12px 30px rgba(29, 90, 68, 0.3);
  }
  .chart :global(.card-main .mr-yr),
  .chart :global(.card-main .mr-mn) {
    color: color-mix(in srgb, var(--surface) 78%, transparent);
  }
  .chart :global(.mr-card:hover) {
    box-shadow: var(--shadow-raised);
  }
  .chart :global(.mr-av) {
    width: 54px;
    height: 54px;
    border-radius: 17px;
    overflow: hidden;
    flex-shrink: 0;
    background: var(--surface-2);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 650;
    font-size: 1.05rem;
  }
  .chart :global(.mr-av span) {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .chart :global(.mr-av img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .chart :global(.mr-txt) {
    min-width: 0;
    line-height: 1.25;
  }
  .chart :global(.mr-nm) {
    font-weight: 650;
    font-size: 0.95rem;
    letter-spacing: -0.01em;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
  }
  .chart :global(.mr-mn) {
    font-size: 0.76rem;
    color: var(--muted);
  }
  .chart :global(.mr-yr) {
    font-size: 0.78rem;
    color: var(--muted);
  }
  .chart :global(.mr-tags) {
    display: flex;
    gap: 4px;
    align-items: center;
    margin-top: 2px;
  }
  .chart :global(.mr-clan) {
    font-size: 0.68rem;
    background: var(--heirloom-soft);
    border-radius: 999px;
    padding: 0 6px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 110px;
  }
  .chart :global(.mr-pet) {
    font-size: 0.75rem;
  }
  .chart :global(.anon .mr-nm) {
    color: var(--faint);
    font-family: var(--font-ui);
  }
  .help.beside {
    inset-inline-end: 400px;
  }
  .tree-wrap:fullscreen .help {
    display: none;
  }
  @media (max-width: 900px) {
    .tree-wrap {
      height: calc(100vh - var(--topbar) - var(--tabbar) - env(safe-area-inset-bottom) - env(safe-area-inset-top));
    }
    .help {
      display: none;
    }
    .bar {
      padding: 8px 10px;
    }
    .dep {
      display: none;
    }
    .side {
      width: 320px;
      padding: 0 10px 10px 0;
    }
    .modes button {
      padding: 6px 11px;
    }
    .add span {
      display: none;
    }
    .pick {
      flex-basis: 100%;
      max-width: none;
      order: 3;
    }
  }
  /* phones: the panel is a sheet over the bottom of the tree */
  @media (max-width: 640px) {
    .side {
      position: absolute;
      left: 10px;
      right: 10px;
      bottom: 10px;
      width: auto;
      padding: 0;
      display: flex;
    }
  }
  @media print {
    .tree-wrap {
      height: auto;
      min-height: 90vh;
    }
  }
</style>
