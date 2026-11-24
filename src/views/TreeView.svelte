<script lang="ts">
  import { tick } from 'svelte';
  import BackButton from '../lib/components/BackButton.svelte';
  // Interactive family tree (family-chart + d3). Click a card to center it,
  // click the centered card again to open the profile.
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { go, route } from '../lib/router.svelte';
  import { backend } from '../lib/backend';
  import { t, tx, tn } from '../lib/i18n/index.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  import X from '../lib/icons/X.svelte';
  import { toast } from '../lib/ui.svelte';
  import { initials } from '../lib/format';
  import { tone } from '../lib/text';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import FamilyGraph from '../lib/components/FamilyGraph.svelte';
  import TreeDetails from '../lib/components/TreeDetails.svelte';
  import PanelIcon from '../lib/icons/SidebarSimple.svelte';
  import ZoomIn from '../lib/icons/MagnifyingGlassPlus.svelte';
  import ZoomOut from '../lib/icons/MagnifyingGlassMinus.svelte';
  import Download from '../lib/icons/DownloadSimple.svelte';
  import UserPlus from '../lib/icons/UserPlus.svelte';
  import NotFound from './NotFound.svelte';
  import Rotate from '../lib/icons/ArrowClockwise.svelte';
  import Fit from '../lib/icons/GpsFix.svelte';
  import Full from '../lib/icons/ArrowsOut.svelte';
  import Pin from '../lib/icons/PushPin.svelte';
  import 'family-chart/styles/family-chart.css';
  import type { TreeNode } from '../lib/types';

  let { id = null }: { id?: number | null } = $props();
  const viewer = $derived(!!app.info?.viewer);

  let cont: HTMLDivElement | undefined = $state();
  let wrap: HTMLDivElement | undefined = $state();
  let chart: any = null;
  // plain data (no deep proxy): big trees have tens of thousands of nodes
  let nodes = $state.raw<TreeNode[]>([]);
  let main = $state<number | null>(null);
  let pick = $state<number | null>(null);
  let error = $state<unknown>(null);
  let empty = $state(false);
  let orient = $state<'vertical' | 'horizontal'>(stored('myroots-tree-orient', 'vertical') as any);
  let anc = $state(Number(stored('myroots-tree-anc', '0')) || 0);
  let prog = $state(Number(stored('myroots-tree-prog', '0')) || 0);
  // never chosen by the user: big trees start with a few generations
  // (all descendants of an old ancestor can be thousands of cards)
  const depthChosen = stored('myroots-tree-anc', '') !== '' || stored('myroots-tree-prog', '') !== '';
  const BIG_TREE = 1500;
  // "person": the tree around one person; "all": the whole family on one board
  let mode = $state<'person' | 'all'>(stored('myroots-tree-mode', 'person') === 'all' ? 'all' : 'person');
  // the whole family reads best with generations as columns (like a poster)
  let gorient = $state<'vertical' | 'horizontal'>(stored('myroots-graph-orient', 'horizontal') === 'vertical' ? 'vertical' : 'horizontal');
  let graph: FamilyGraph | undefined = $state();
  // "direct": the direct line of the person stays big, side lines get smaller;
  // "colors": the four lines of the grandparents in their own colors (both: "both")
  type Hl = '' | 'direct' | 'colors' | 'both';
  let hl = $state<Hl>((['', 'direct', 'colors', 'both'].includes(stored('myroots-tree-hl', '')) ? stored('myroots-tree-hl', '') : '') as Hl);
  // a clicked clan tag: only the people of this clan are lit and listed
  let clanSel = $state<string | null>(route.query.clan || null);
  const clanPeople = $derived(
    clanSel
      ? nodes
          .filter((n) => !n.data.anon && n.data.clans?.includes(clanSel!))
          .map((n) => ({ id: Number(n.id), name: `${n.data['first name'] || ''} ${n.data['last name'] || ''}`.trim() || '—', years: n.data.years, avatar: n.data.avatar, gender: n.data.gender }))
          .sort((a, b) => a.name.localeCompare(b.name))
      : [],
  );
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
    const svgText = await graph?.exportSvg();
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
    const clans = (n.clans || []).slice(0, 1).map((c: string) => `<span class="mr-clan" data-clan="${esc(c)}" role="button" title="${esc(t('Show only the clan %s', c))}">${esc(c)}</span>`).join('');
    // pets: small photos hanging from the card; a click opens the pet
    const list = (n.pets || []) as { id: number; name: string; photo: number | null }[];
    const pets = list.length
      ? `<div class="mr-pets">${list
          .slice(0, 3)
          .map((p) => {
            const ps = p.photo ? backend.mediaUrl(p.photo, true) : '';
            return `<span class="mr-petav" data-pet="${p.id}" title="${esc(p.name)}" role="link">${ps ? `<img src="${esc(ps)}" alt="">` : `<span>${esc(initials(p.name))}</span>`}</span>`;
          })
          .join('')}${list.length > 3 ? `<span class="mr-petav more" data-pet="${list[3].id}" title="${esc(list.slice(3).map((p) => p.name).join(', '))}">+${list.length - 3}</span>` : ''}</div>`
      : '';
    return `<div class="mr-card"><div class="mr-av">${av}</div><div class="mr-txt"><div class="mr-nm">${esc(name)}</div>${maiden}<div class="mr-yr">${esc(n.years || '')}</div><div class="mr-tags">${clans}</div></div>${pets}</div>`;
  }

  /** Direct line of `mid` (ancestors, descendants, partners of the person)
   *  and the line (0–3: the four grandparents) of every ancestor. */
  const nodeById = $derived(new Map(nodes.map((n) => [n.id, n])));
  function lines(mid: string) {
    const byId = nodeById;
    const direct = new Set<string>([mid]);
    const branch = new Map<string, number[]>();
    const me = byId.get(mid);
    if (!me) return { direct, branch };
    for (const s of me.rels.spouses || []) direct.add(s);
    const sorted = (ids: string[]) => [...ids].sort((a, b) => (byId.get(a)?.data.gender === 'F' ? 1 : 0) - (byId.get(b)?.data.gender === 'F' ? 1 : 0));
    const mark = (id: string, b: number) => {
      const q = [id];
      while (q.length) {
        const x = q.pop()!;
        const l = branch.get(x) || [];
        if (l.includes(b)) continue;
        branch.set(x, [...l, b]);
        direct.add(x);
        q.push(...(byId.get(x)?.rels.parents || []));
      }
    };
    sorted(me.rels.parents || []).slice(0, 2).forEach((pid, pi) => {
      direct.add(pid);
      const gps = sorted(byId.get(pid)?.rels.parents || []).slice(0, 2);
      if (!gps.length) branch.set(pid, [pi * 2, pi * 2 + 1]);
      gps.forEach((g, gi) => mark(g, pi * 2 + gi));
      if (gps.length) branch.set(pid, gps.map((_, gi) => pi * 2 + gi));
    });
    const q = [...(me.rels.children || [])];
    while (q.length) {
      const x = q.pop()!;
      if (direct.has(x)) continue;
      direct.add(x);
      q.push(...(byId.get(x)?.rels.children || []));
    }
    return { direct, branch };
  }
  let deco: ReturnType<typeof lines> | null = null;
  const dId = (d: any): string => String(d?.data?.id ?? '');
  function decoCard(el: Element, d: any) {
    const card = el.querySelector('.card');
    if (!card) return;
    const id = dId(d);
    const side = !!deco && (hl === 'direct' || hl === 'both') && !deco.direct.has(id);
    const br = deco && (hl === 'colors' || hl === 'both') ? deco.branch.get(id) : undefined;
    card.classList.toggle('mr-side', side);
    card.classList.toggle('mr-br', !!br);
    if (br) {
      (card as HTMLElement).style.setProperty('--b1', `var(--br${br[0] + 1})`);
      (card as HTMLElement).style.setProperty('--b2', `var(--br${(br[1] ?? br[0]) + 1})`);
    }
    const inClan = !clanSel || !!d?.data?.data?.clans?.includes(clanSel);
    card.classList.toggle('mr-dim', !inClan);
    card.classList.toggle('mr-lit', !!clanSel && inClan);
  }
  function decoLink(el: Element, l: any) {
    const ends = (x: any) => (Array.isArray(x) ? x : [x]).filter(Boolean).map(dId);
    const from = ends(l.source);
    const to = ends(l.target);
    let direct = false;
    let color: number | null = null;
    if (deco) {
      direct = to.every((x) => deco!.direct.has(x)) && from.some((x) => deco!.direct.has(x));
      if (l.is_ancestry && !l.spouse) {
        const bs = new Set(to.flatMap((x) => deco!.branch.get(x) || []));
        if (bs.size === 1) color = [...bs][0];
      }
    }
    const on = hl === 'direct' || hl === 'both';
    el.classList.toggle('mr-ldirect', on && direct);
    el.classList.toggle('mr-lside', on && !direct);
    const c = hl === 'colors' || hl === 'both' ? color : null;
    el.classList.toggle('mr-lbr', c != null);
    if (c != null) (el as HTMLElement).style.setProperty('--lb', `var(--br${c + 1})`);
    if (clanSel) {
      const all = [...from, ...to].map((x) => nodeById.get(x));
      el.classList.toggle('mr-dim', !all.some((n) => n?.data.clans?.includes(clanSel!)));
    } else el.classList.remove('mr-dim');
  }
  function decorate() {
    if (!cont || mode !== 'person') return;
    deco = hl && main ? lines(String(main)) : null;
    cont.querySelectorAll('div.card_cont').forEach((el) => decoCard(el, (el as any).__data__));
    cont.querySelectorAll('path.link').forEach((el) => decoLink(el, (el as any).__data__));
  }
  function setHl(v: Hl) {
    hl = v;
    store('myroots-tree-hl', v);
    decorate();
  }
  function update(props: Record<string, unknown>) {
    chart?.updateTree(props);
    decorate();
  }
  $effect(() => {
    void clanSel;
    decorate();
  });

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
      .setOnCardUpdate(function (this: Element, d: any) {
        decoCard(this, d);
      })
      .setOnCardClick((e: MouseEvent, d: any) => {
        const clanEl = (e.target as HTMLElement | null)?.closest?.('[data-clan]');
        if (clanEl) {
          const c = clanEl.getAttribute('data-clan');
          clanSel = clanSel === c ? null : c;
          return;
        }
        const petEl = (e.target as HTMLElement | null)?.closest?.('[data-pet]');
        if (petEl) {
          go(`/pet/${petEl.getAttribute('data-pet')}`);
          return;
        }
        if (d.data.data.anon) return;
        const pid = Number(d.data.id);
        if (pid === main) go(`/person/${pid}`);
        else setMain(pid);
      });
    if (main) chart.updateMainId(String(main));
    update({ initial: true });
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
      update({ tree_position: 'main_to_middle' });
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
        await backend.ensureMedia?.(
          r.nodes.flatMap((n) => [n.data.avatar, ...(n.data.pets || []).map((p) => p.photo)]).filter((x): x is number => !!x),
        );
        nodes = r.nodes;
        if (!depthChosen && r.nodes.length > BIG_TREE) {
          if (!anc) anc = 6;
          if (!prog) prog = 3;
        }
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
      tick().then(() => chart && update({ tree_position: 'main_to_middle' }));
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
    else if (chart) update({ tree_position: 'fit' });
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
          <label class="small muted dep">
            {t('Lines')}
            <select value={hl} onchange={(e) => setHl(e.currentTarget.value as Hl)}>
              <option value="">{t('Normal')}</option>
              <option value="direct">{t('Highlight the direct line')}</option>
              <option value="colors">{t('Color the lines of the grandparents')}</option>
              <option value="both">{t('Both')}</option>
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
      {#if clanSel && mode === 'person'}
        <div class="side no-print">
          <div class="panel clanp">
            <div class="ch">
              <div>
                <span class="tag heirloom">{clanSel}</span>
                <span class="small muted">{tn('%d person', '%d people', clanPeople.length)}</span>
              </div>
              <button class="btn ghost icon sm" onclick={() => (clanSel = null)} aria-label={t('Close')}><X size={16} /></button>
            </div>
            <p class="small faint">{t('People of this clan are highlighted in the tree. Click a person to center the tree on them.')}</p>
            <ul>
              {#each clanPeople as c (c.id)}
                <li><button class="row" class:on={c.id === main} onclick={() => setMain(c.id)}><Avatar photo={c.avatar} name={c.name} gender={c.gender as any} size={34} /><span class="grow"><span class="nm">{c.name}</span><span class="small muted">{c.years}</span></span></button></li>
              {/each}
            </ul>
          </div>
        </div>
      {:else if panelId && panelOpen}
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
    background: radial-gradient(circle at 1px 1px, color-mix(in srgb, var(--ink) 14%, transparent) 1px, transparent 0) 0 0 / 22px 22px, var(--tree-bg, var(--paper));
    --br1: var(--tree-br1, #3f7cc4);
    --br2: var(--tree-br2, #2f9a6b);
    --br3: var(--tree-br3, #c9693c);
    --br4: var(--tree-br4, #9b59b6);
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
    --background-color: var(--tree-bg, var(--paper));
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
    stroke: var(--tree-line, var(--line-strong)) !important;
    stroke-width: 2px;
  }
  .chart :global(.link.f3-path-to-main) {
    stroke: var(--tree-line-main, var(--heirloom)) !important;
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
  .chart :global(.mr-card) {
    position: relative;
  }
  .chart :global(.mr-pets) {
    position: absolute;
    inset-inline-end: 8px;
    bottom: -13px;
    display: flex;
    gap: 3px;
  }
  .chart :global(.mr-petav) {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    overflow: hidden;
    border: 2px solid var(--surface);
    background: var(--heirloom-soft);
    box-shadow: var(--shadow-raised);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.62rem;
    font-weight: 650;
    color: var(--heirloom);
    cursor: pointer;
    transition: transform 0.12s;
  }
  .chart :global(.mr-petav:hover) {
    transform: scale(1.25);
  }
  .chart :global(.mr-petav img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .chart :global(.anon .mr-nm) {
    color: var(--faint);
    font-family: var(--font-ui);
  }
  /* direct line: side lines smaller and paler, the line itself stronger */
  .chart :global(.card.mr-side .mr-card) {
    transform: scale(0.8);
    transition: transform 0.2s;
  }
  .chart :global(.card.mr-side .mr-card > *) {
    opacity: 0.6;
  }
  .chart :global(.card.mr-side:hover .mr-card > *) {
    opacity: 1;
  }
  .chart :global(.link.mr-ldirect) {
    stroke: var(--tree-line-main, var(--ink)) !important;
    stroke-width: 3.5px;
  }
  .chart :global(.link.mr-lside) {
    stroke-width: 1.2px;
    opacity: 0.55 !important;
  }
  .chart :global(.link.mr-lbr) {
    stroke: var(--lb) !important;
  }
  /* the lines of the four grandparents */
  .chart :global(.card.mr-br .mr-card) {
    padding-inline-start: 16px;
  }
  .chart :global(.card.mr-br .mr-card::before) {
    content: '';
    position: absolute;
    inset-block: 10px;
    inset-inline-start: 5px;
    width: 5px;
    border-radius: 3px;
    background: linear-gradient(var(--b1) 50%, var(--b2) 50%);
  }
  /* a clicked clan */
  /* the card stays opaque (lines behind it stay hidden), only its content fades */
  .chart :global(.card.mr-dim .mr-card) {
    filter: grayscale(1);
    box-shadow: none;
  }
  .chart :global(.card.mr-dim .mr-card > *),
  .chart :global(.card.mr-dim .mr-card::before),
  .chart :global(.card.mr-dim .mr-card::after) {
    opacity: 0.3;
  }
  .chart :global(.link.mr-dim) {
    opacity: 0.2 !important;
  }
  .chart :global(.card.mr-lit .mr-card) {
    box-shadow: 0 0 0 3px var(--heirloom), var(--shadow-raised);
  }
  .chart :global(.mr-clan[data-clan]) {
    cursor: pointer;
  }
  .chart :global(.mr-clan[data-clan]:hover) {
    background: var(--heirloom);
    color: var(--surface);
  }
  .clanp {
    max-height: 100%;
    display: flex;
    flex-direction: column;
    gap: 6px;
    box-sizing: border-box;
    overflow: hidden;
  }
  .clanp .ch {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }
  .clanp .ch > div {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .clanp p {
    margin: 0;
  }
  .clanp ul {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow: auto;
  }
  .clanp .row {
    display: flex;
    gap: 10px;
    align-items: center;
    width: 100%;
    border: 0;
    background: none;
    font: inherit;
    color: inherit;
    text-align: start;
    padding: 6px;
    border-radius: 12px;
    cursor: pointer;
  }
  .clanp .row:hover,
  .clanp .row.on {
    background: var(--surface-2);
  }
  .clanp .grow {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .clanp .nm {
    font-weight: 600;
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
