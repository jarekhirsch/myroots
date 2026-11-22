<script lang="ts">
  import { printPage } from '../lib/print';
  // A poster to print in a large format: a fan of ancestors or a tree of
  // descendants, as vector graphics (print to PDF, or save SVG / PNG).
  import { app, call } from '../lib/api.svelte';
  import { backend } from '../lib/backend';
  import { go } from '../lib/router.svelte';
  import { t } from '../lib/i18n/index.svelte';
  import { toast } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import { PAPERS, fan, sector, midArc, descendants, CARD_W, CARD_H, type FanCell } from '../lib/poster';
  import { tone } from '../lib/text';
  import { initials } from '../lib/format';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import Printer from '@lucide/svelte/icons/printer';
  import Download from '@lucide/svelte/icons/download';
  import FanIcon from '@lucide/svelte/icons/fan';
  import Tree from '@lucide/svelte/icons/network';
  import fontLatin from '@fontsource-variable/literata/files/literata-latin-wght-normal.woff2?url';
  import fontExt from '@fontsource-variable/literata/files/literata-latin-ext-wght-normal.woff2?url';
  import uiLatin from '@fontsource-variable/onest/files/onest-latin-wght-normal.woff2?url';
  import uiExt from '@fontsource-variable/onest/files/onest-latin-ext-wght-normal.woff2?url';
  import { svgToPdf } from '../lib/pdf';
  import { AUTHOR } from '../lib/credits';
  import FileDown from '@lucide/svelte/icons/file-down';
  import type { TreeNode } from '../lib/types';

  let { id = null }: { id?: number | null } = $props();

  let nodes = $state.raw<TreeNode[]>([]);
  let start = $state<number | null>(null);
  let kind = $state<'fan' | 'desc'>('fan');
  let gensFan = $state(6);
  let gensDesc = $state(4);
  let paper = $state('A2');
  let title = $state('');
  let svgEl: SVGSVGElement | undefined = $state();
  let busy = $state(false);

  $effect(() => {
    call<{ nodes: TreeNode[]; start: number | null }>('tree.data').then((r) => {
      nodes = r.nodes;
      start = r.start ?? (r.nodes[0] ? Number(r.nodes[0].id) : null);
    });
  });
  const root = $derived(id ?? start);
  const rootNode = $derived(nodes.find((n) => n.id === String(root)));
  const rootName = $derived(rootNode ? [rootNode.data['first name'], rootNode.data['last name']].filter(Boolean).join(' ') : '');
  const F = $derived(kind === 'fan' && root != null && nodes.length ? fan(nodes, String(root), gensFan, PAPERS[paper]) : null);
  const D = $derived(kind === 'desc' && root != null && nodes.length ? descendants(nodes, String(root), gensDesc, PAPERS[paper]) : null);
  const W = $derived(F?.W ?? D?.W ?? 594);
  const H = $derived(F?.H ?? D?.H ?? 420);
  const heading = $derived(title.trim() || (kind === 'fan' ? t('Ancestors of %s', rootName) : t('Descendants of %s', rootName)));
  const subtitle = $derived([app.info?.tree_name || app.info?.site?.title || '', String(new Date().getFullYear())].filter(Boolean).join(' · '));
  // the smallest names on the descendants poster (mm): too small to read under ~1.6 mm
  // years are written 12.5 units high: under ~1.6 mm they cannot be read
  // (a fan of 12 generations has 4 094 places; more would be unprintable)
  // on the fan: the smallest name of a person (mm); under ~1.4 mm it cannot be read
  const fanSmall = $derived.by(() => {
    if (!F) return false;
    let min = Infinity;
    for (const c of F.cells) {
      const L = c.person ? cellText(c, F.cx, F.cy) : null;
      if (L && L.fs < min) min = L.fs;
    }
    return min < 1.4;
  });
  const tooSmall = $derived((!!D && D.scale * 12.5 < 1.6) || fanSmall);

  /* ---------- how many generations the tree has around the person ---------- */
  const GENS = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16];
  function depth(dir: 'parents' | 'children'): number {
    const byId = new Map(nodes.map((n) => [n.id, n]));
    let level = root != null ? [String(root)] : [];
    const seen = new Set(level);
    let d = 0;
    while (level.length && d < 40) {
      d++;
      const next: string[] = [];
      for (const id of level) for (const x of byId.get(id)?.rels[dir] || []) if (!seen.has(x)) (seen.add(x), next.push(x));
      level = next;
    }
    return d;
  }
  const ancGens = $derived(nodes.length ? depth('parents') : 0);
  const descGens = $derived(nodes.length ? depth('children') : 0);

  /* ---------- fan text ---------- */
  const SERIF = "'Literata Variable', Literata, Georgia, serif";
  const UI = "'Onest Variable', Onest, system-ui, sans-serif";
  // the light colours of the app (the poster is printed on white paper)
  const C = {
    text: '#17201b',
    muted: '#56645b',
    line: '#e3eae5',
    lineStrong: '#cdd8d1',
    heirloom: '#c49a6c',
    p: [[], ['#dfe9e3', '#24503f'], ['#f1e4d3', '#7a5328'], ['#dde8f0', '#2e5468'], ['#f2dfe3', '#7d3d4d'], ['#e6e2f1', '#4b4170'], ['#e9ecd9', '#4f5826']] as string[][],
  };
  const fullName = (p: { first: string; last: string }) => [p.first, p.last].filter(Boolean).join(' ') || '?';
  // names cut to the card, as in the tree
  let mctx: CanvasRenderingContext2D | null = null;
  function fit(name: string): string {
    if (!mctx) mctx = document.createElement('canvas').getContext('2d');
    if (!mctx) return name;
    mctx.font = "620 14.5px 'Onest Variable', system-ui";
    const max = CARD_W - 76;
    if (mctx.measureText(name).width <= max) return name;
    let n = name.length;
    while (n > 1 && mctx.measureText(name.slice(0, n) + '…').width > max) n--;
    return name.slice(0, n).trimEnd() + '…';
  }
  function fill(c: FanCell, gens: number): string {
    if (!c.person) return '#f2eee6';
    const l = 58 + ((c.gen - 1) / Math.max(1, gens - 2)) * 30;
    return c.side === 0 ? `hsl(30 42% ${l}%)` : `hsl(165 28% ${l}%)`;
  }
  type Lines = { tangential: boolean; lines: string[]; fs: number; deg: number; x: number; y: number; r: number };
  function cellText(c: FanCell, cx: number, cy: number): Lines | null {
    if (!c.person) return null;
    const mid = (c.r0 + c.r1) / 2;
    const ring = c.r1 - c.r0;
    const arcLen = mid * (c.a1 - c.a0);
    const p = c.person;
    const a = (c.a0 + c.a1) / 2;
    if (arcLen > ring * 1.1) {
      const lines = [p.first, p.last, p.years].filter(Boolean);
      const longest = Math.max(4, ...lines.map((l) => l.length));
      const fs = Math.min(ring / (lines.length * 1.42 + 0.4), (arcLen * 0.86) / (longest * 0.56), 9);
      return { tangential: true, lines, fs, deg: 0, x: 0, y: 0, r: mid };
    }
    const name = [p.first, p.last].filter(Boolean).join(' ');
    const lines = arcLen > ring * 0.34 ? [name, p.years].filter(Boolean) : [name];
    const longest = Math.max(4, ...lines.map((l) => l.length));
    const fs = Math.min(arcLen / (lines.length * 1.3 + 0.2), (ring * 0.9) / (longest * 0.55), 7);
    let deg = (a * 180) / Math.PI;
    if (Math.cos(a) < 0) deg += 180;
    return { tangential: false, lines, fs, deg, x: cx + mid * Math.cos(a), y: cy + mid * Math.sin(a), r: mid };
  }
  // text along the ring: one arc per line, from the outside in
  function lineArcs(c: FanCell, L: Lines, cx: number, cy: number): string[] {
    const block = L.lines.length * L.fs * 1.32;
    return L.lines.map((_, j) => midArc(cx, cy, c, L.r + block / 2 - L.fs * (0.95 + 1.32 * j)));
  }

  /* ---------- print and save ---------- */
  $effect(() => {
    const st = document.createElement('style');
    st.textContent = `@page { size: ${W}mm ${H}mm; margin: 0; } @media print { .poster-svg { width: ${W}mm !important; height: ${H}mm !important; } }`;
    document.head.appendChild(st);
    return () => st.remove();
  });

  async function dataUrl(u: string): Promise<string> {
    const b = await (await fetch(u)).blob();
    return await new Promise((resolve) => {
      const r = new FileReader();
      r.onload = () => resolve(String(r.result));
      r.readAsDataURL(b);
    });
  }
  // the SVG on its own, with the font inside (print shops do not have it)
  async function standalone(): Promise<string> {
    const clone = svgEl!.cloneNode(true) as SVGSVGElement;
    clone.removeAttribute('class');
    const [a, b, c, d] = await Promise.all([dataUrl(fontLatin), dataUrl(fontExt), dataUrl(uiLatin), dataUrl(uiExt)]);
    const style = document.createElementNS('http://www.w3.org/2000/svg', 'style');
    const LATIN = 'U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD';
    const EXT = 'U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF';
    const face = (family: string, url: string, range: string) => `@font-face{font-family:'${family}';font-weight:100 900;src:url(${url}) format('woff2');unicode-range:${range}}`;
    style.textContent = face('Literata Variable', a, LATIN) + face('Literata Variable', b, EXT) + face('Onest Variable', c, LATIN) + face('Onest Variable', d, EXT);
    clone.insertBefore(style, clone.firstChild);
    return '<?xml version="1.0" encoding="UTF-8"?>\n' + new XMLSerializer().serializeToString(clone);
  }
  const fileBase = $derived((heading || 'poster').toLowerCase().replace(/[^\p{L}\p{N}]+/gu, '-').replace(/^-|-$/g, '') + `-${paper}`);
  async function saveSvg() {
    busy = true;
    try {
      await backend.saveFile(`${fileBase}.svg`, await standalone(), 'image/svg+xml');
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
  let progress = $state('');
  async function savePdf() {
    busy = true;
    try {
      // print quality for small formats, still sharp on large ones
      const dpi = W * H <= 420 * 594 ? 300 : 200;
      const pdf = await svgToPdf(await standalone(), W, H, dpi, (d, n) => (progress = `${d}/${n}`));
      await backend.saveFile(`${fileBase}.pdf`, pdf, 'application/pdf');
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
      progress = '';
    }
  }
  const credit = `© MyRoots · ${AUTHOR}`;

  async function savePng() {
    busy = true;
    try {
      const svg = await standalone();
      const url = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }));
      const img = new Image();
      await new Promise<void>((ok, bad) => {
        img.onload = () => ok();
        img.onerror = () => bad(new Error('svg'));
        img.src = url;
      });
      // 150 dpi, at most ~16 million pixels (the limit of some browsers)
      const dpi = Math.min(150, Math.sqrt(16e6 / (W * H)) * 25.4);
      const c = document.createElement('canvas');
      c.width = Math.round((W / 25.4) * dpi);
      c.height = Math.round((H / 25.4) * dpi);
      c.getContext('2d')!.drawImage(img, 0, 0, c.width, c.height);
      URL.revokeObjectURL(url);
      const blob: Blob = await new Promise((ok, bad) => c.toBlob((b) => (b ? ok(b) : bad(new Error('png'))), 'image/png'));
      await backend.saveFile(`${fileBase}.png`, new Uint8Array(await blob.arrayBuffer()), 'image/png');
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
</script>

<div class="page poster-page">
  <div class="page-head no-print">
    <h1>{t('Poster')}</h1>
  </div>
  <div class="panel controls no-print">
    <div class="seg" role="group" aria-label={t('Kind of poster')}>
      <button aria-pressed={kind === 'fan'} onclick={() => (kind = 'fan')}><FanIcon size={16} /> {t('Fan of ancestors')}</button>
      <button aria-pressed={kind === 'desc'} onclick={() => (kind = 'desc')}><Tree size={16} /> {t('Tree of descendants')}</button>
    </div>
    <div class="who"><PersonPicker value={root} onchange={(v) => v && go(`/poster/${v}`)} placeholder={t('Search for a person…')} /></div>
    <label class="field sm">
      <span class="lbl">{t('Generations')}</span>
      {#if kind === 'fan'}
        <select bind:value={gensFan}>{#each GENS.filter((g) => g >= 3 && g <= 12) as g (g)}<option value={g}>{g}{g === ancGens ? ' ✓' : ''}</option>{/each}</select>
      {:else}
        <select bind:value={gensDesc}>{#each GENS as g (g)}<option value={g}>{g}{g === descGens ? ' ✓' : ''}</option>{/each}</select>
      {/if}
      <span class="hint small">{t('In the tree: %d', kind === 'fan' ? ancGens : descGens)}</span>
    </label>
    <label class="field sm">
      <span class="lbl">{t('Paper')}</span>
      <select bind:value={paper}>{#each Object.keys(PAPERS) as p (p)}<option value={p}>{p} ({PAPERS[p][0]} × {PAPERS[p][1]} mm)</option>{/each}</select>
    </label>
    <label class="field grow">
      <span class="lbl">{t('Title')}</span>
      <input type="text" bind:value={title} placeholder={heading} />
    </label>
    <div class="acts">
      <button class="btn primary" onclick={savePdf} disabled={busy || (!F && !D)}><FileDown size={16} /> {progress ? `PDF ${progress}` : 'PDF'}</button>
      <button class="btn" onclick={printPage} disabled={!F && !D}><Printer size={16} /> {t('Print')}</button>
      <button class="btn" onclick={saveSvg} disabled={busy || (!F && !D)}><Download size={16} /> SVG</button>
      <button class="btn" onclick={savePng} disabled={busy || (!F && !D)}><Download size={16} /> PNG</button>
    </div>
    {#if tooSmall}<p class="warn small">{t('There are many people for this paper: the names will be very small. Choose a larger paper or fewer generations.')}</p>{/if}
    <p class="hint small">{t('For a print shop, save the PDF (exactly the chosen paper size) or the SVG.')}</p>
  </div>

  <div class="stage">
    {#if F}
      <svg bind:this={svgEl} class="poster-svg" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {F.W} {F.H}" width="{F.W}mm" height="{F.H}mm" font-family={SERIF}>
        <rect width={F.W} height={F.H} fill="#fbf8f2" />
        <defs>
          {#each F.cells as c, i (i)}
            {@const L = cellText(c, F.cx, F.cy)}
            {#if L?.tangential}
              {#each lineArcs(c, L, F.cx, F.cy) as d, j (j)}<path id="fa{i}-{j}" d={d} fill="none" />{/each}
            {/if}
          {/each}
        </defs>
        {#each F.cells as c, i (i)}
          <path d={sector(F.cx, F.cy, c)} fill={fill(c, gensFan)} stroke="#fbf8f2" stroke-width={Math.max(0.3, F.R * 0.004)} />
          {@const L = cellText(c, F.cx, F.cy)}
          {#if L}
            {#if L.tangential}
              {#each L.lines as line, j (j)}
                <text font-size={L.fs} fill="#2a2420" font-weight={j === 0 && L.lines.length > 2 ? 400 : j === L.lines.length - 1 && c.person?.years ? 400 : 600} opacity={j === L.lines.length - 1 && c.person?.years ? 0.75 : 1}>
                  <textPath href="#fa{i}-{j}" startOffset="50%" text-anchor="middle">{line}</textPath>
                </text>
              {/each}
            {:else}
              <g transform="translate({L.x} {L.y}) rotate({L.deg})">
                <text font-size={L.fs} text-anchor="middle" fill="#2a2420">
                  {#each L.lines as line, j (j)}
                    <tspan x="0" y={(j - (L.lines.length - 1) / 2) * L.fs * 1.2 + L.fs * 0.35} font-weight={j === 0 ? 600 : 400} opacity={j === 0 ? 1 : 0.75}>{line}</tspan>
                  {/each}
                </text>
              </g>
            {/if}
          {/if}
        {/each}
        <circle cx={F.cx} cy={F.cy} r={F.r0 * 0.97} fill="#2f4a3c" />
        {#if F.root}
          <text x={F.cx} y={F.cy - F.r0 * 0.22} font-size={F.r0 * 0.2} text-anchor="middle" fill="#fff" font-weight="600">{F.root.first}</text>
          <text x={F.cx} y={F.cy + F.r0 * 0.04} font-size={F.r0 * 0.2} text-anchor="middle" fill="#fff" font-weight="600">{F.root.last}</text>
          <text x={F.cx} y={F.cy + F.r0 * 0.32} font-size={F.r0 * 0.14} text-anchor="middle" fill="#fff" opacity="0.8">{F.root.years}</text>
        {/if}
        <text x={F.W / 2} y={F.titleY} font-size={F.H * 0.042} text-anchor="middle" fill="#2a2420" font-weight="600">{heading}</text>
        <text x={F.W / 2} y={F.titleY + F.H * 0.035} font-size={F.H * 0.017} text-anchor="middle" fill="#6b6259">{subtitle}</text>
        <text x={F.W - F.W * 0.02} y={F.H - F.H * 0.02} font-size={Math.max(1.6, F.H * 0.009)} text-anchor="end" fill="#a39a8f">{credit}</text>
      </svg>
    {:else if D}
      <svg bind:this={svgEl} class="poster-svg" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {D.W} {D.H}" width="{D.W}mm" height="{D.H}mm" font-family={UI}>
        <rect width={D.W} height={D.H} fill="#f6f8f6" />
        <text x={D.W / 2} y={D.titleY} font-size={Math.min(D.W, D.H) * 0.04} text-anchor="middle" fill={C.text} font-weight="650">{heading}</text>
        <text x={D.W / 2} y={D.titleY + Math.min(D.W, D.H) * 0.028} font-size={Math.min(D.W, D.H) * 0.015} text-anchor="middle" fill={C.muted}>{subtitle}</text>
        <!-- the same cards and lines as the family tree in the app -->
        <g transform="translate({D.ox} {D.oy}) scale({D.scale})">
          {#each D.links as d, i (i)}<path {d} fill="none" stroke={C.lineStrong} stroke-width="2" stroke-linejoin="round" />{/each}
          {#each D.couples as d, i (i)}<path {d} fill="none" stroke={C.heirloom} stroke-width="3" />{/each}
          {#each D.units as u, i (i)}
            {#each [u.person, u.spouse] as p, k (k)}
              {#if p}
                {@const tn = tone(fullName(p))}
                <g transform="translate({u.x + k * (CARD_W + 26)} {u.y})">
                  <rect width={CARD_W} height={CARD_H} rx="16" fill="#fff" stroke={C.line} stroke-width="1" />
                  <rect x="10" y="10" width="44" height="44" rx="14" fill={C.p[tn][0]} />
                  <text x="32" y="37.5" text-anchor="middle" font-size="15" font-weight="650" fill={C.p[tn][1]}>{initials(fullName(p))}</text>
                  <text x="66" y={p.years ? 29 : 37} font-size="14.5" font-weight="620" fill={C.text}>{fit(fullName(p))}</text>
                  {#if p.years}<text x="66" y="47" font-size="12.5" fill={C.muted}>{p.years}</text>{/if}
                  {#if k === 1 && u.more}<text x={CARD_W - 12} y="20" font-size="11" text-anchor="end" fill={C.muted}>+{u.more}</text>{/if}
                </g>
              {/if}
            {/each}
          {/each}
        </g>
        <text x={D.W - Math.min(D.W, D.H) * 0.02} y={D.H - Math.min(D.W, D.H) * 0.02} font-size={Math.max(1.6, Math.min(D.W, D.H) * 0.009)} text-anchor="end" fill="#9aa59f">{credit}</text>
      </svg>
    {:else if nodes.length === 0}
      <div class="panel empty">{t('No people yet.')}</div>
    {/if}
  </div>
</div>

<style>
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 12px 16px;
    align-items: flex-end;
    margin-bottom: 18px;
  }
  .seg {
    display: inline-flex;
    background: var(--ink-soft);
    border-radius: 999px;
    padding: 3px;
  }
  .seg button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 0;
    background: none;
    border-radius: 999px;
    padding: 7px 14px;
    font: inherit;
    font-size: 0.9rem;
    color: var(--muted);
    cursor: pointer;
  }
  .seg button[aria-pressed='true'] {
    background: var(--surface);
    color: var(--ink);
    box-shadow: var(--shadow);
    font-weight: 600;
  }
  .who {
    width: min(300px, 100%);
  }
  .field.sm {
    width: auto;
  }
  .field.grow {
    flex: 1;
    min-width: 200px;
  }
  .acts {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .hint,
  .warn {
    flex-basis: 100%;
    margin: 0;
  }
  .warn {
    color: var(--danger, #b03a2e);
  }
  .stage {
    display: flex;
    justify-content: center;
  }
  .poster-svg {
    width: 100%;
    height: auto;
    max-height: 78vh;
    background: transparent;
    box-shadow: var(--shadow-raised, 0 10px 30px rgba(0, 0, 0, 0.15));
    border-radius: 4px;
  }
  @media print {
    .poster-page {
      padding: 0 !important;
      margin: 0 !important;
      max-width: none !important;
    }
    .poster-svg {
      box-shadow: none;
      border-radius: 0;
      max-height: none;
      display: block;
    }
  }
</style>
