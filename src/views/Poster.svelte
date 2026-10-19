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
  import { PAPERS, fan, sector, midArc, descendants, type FanCell } from '../lib/poster';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import Printer from '@lucide/svelte/icons/printer';
  import Download from '@lucide/svelte/icons/download';
  import FanIcon from '@lucide/svelte/icons/fan';
  import Tree from '@lucide/svelte/icons/network';
  import fontLatin from '@fontsource-variable/literata/files/literata-latin-wght-normal.woff2?url';
  import fontExt from '@fontsource-variable/literata/files/literata-latin-ext-wght-normal.woff2?url';
  import { svgToPdf } from '../lib/pdf';
  import { AUTHOR } from '../lib/credits';
  import FileDown from '@lucide/svelte/icons/file-down';
  import type { TreeNode } from '../lib/types';

  let { id = null }: { id?: number | null } = $props();

  let nodes = $state<TreeNode[]>([]);
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
  const tooSmall = $derived(!!D && D.box[1] * 0.24 < 1.6);

  /* ---------- fan text ---------- */
  const SERIF = "'Literata Variable', Literata, Georgia, serif";
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
    const [a, b] = await Promise.all([dataUrl(fontLatin), dataUrl(fontExt)]);
    const style = document.createElementNS('http://www.w3.org/2000/svg', 'style');
    style.textContent = `@font-face{font-family:'Literata Variable';font-weight:200 900;src:url(${a}) format('woff2');unicode-range:U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD}@font-face{font-family:'Literata Variable';font-weight:200 900;src:url(${b}) format('woff2');unicode-range:U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF}`;
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
        <select bind:value={gensFan}>{#each [3, 4, 5, 6, 7, 8] as g (g)}<option value={g}>{g}</option>{/each}</select>
      {:else}
        <select bind:value={gensDesc}>{#each [2, 3, 4, 5, 6] as g (g)}<option value={g}>{g}</option>{/each}</select>
      {/if}
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
      {@const s = D.scale}
      <svg bind:this={svgEl} class="poster-svg" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {D.W} {D.H}" width="{D.W}mm" height="{D.H}mm" font-family={SERIF}>
        <rect width={D.W} height={D.H} fill="#fbf8f2" />
        <text x={D.W / 2} y={D.titleY} font-size={Math.min(D.W, D.H) * 0.042} text-anchor="middle" fill="#2a2420" font-weight="600">{heading}</text>
        <text x={D.W / 2} y={D.titleY + Math.min(D.W, D.H) * 0.03} font-size={Math.min(D.W, D.H) * 0.016} text-anchor="middle" fill="#6b6259">{subtitle}</text>
        {#each D.links as l, i (i)}
          {@const my = (l.y1 + l.y2) / 2}
          <path d="M{l.x1} {l.y1} V{my} H{l.x2} V{l.y2}" fill="none" stroke="#b9ad9c" stroke-width={Math.max(0.25, 0.6 * s)} />
        {/each}
        {#each D.nodes as n, i (i)}
          {@const [bw, bh] = D.box}
          <g transform="translate({n.x - bw / 2} {n.y})">
            <rect width={bw} height={bh} rx={bh * 0.14} fill="#fff" stroke="#d9cfbf" stroke-width={Math.max(0.2, 0.4 * s)} />
            <rect width={bh * 0.12} height={bh} rx={bh * 0.06} fill={n.person.gender === 'F' ? '#b7867a' : n.person.gender === 'M' ? '#6f8fa6' : '#a99f92'} />
            <text x={bw / 2 + bh * 0.06} y={bh * 0.36} font-size={bh * 0.24} text-anchor="middle" fill="#2a2420" font-weight="600">{[n.person.first, n.person.last].filter(Boolean).join(' ')}</text>
            <text x={bw / 2 + bh * 0.06} y={bh * 0.62} font-size={bh * 0.19} text-anchor="middle" fill="#6b6259">{n.person.years}</text>
            {#if n.partners.length}
              <text x={bw / 2 + bh * 0.06} y={bh * 0.86} font-size={bh * 0.16} text-anchor="middle" fill="#6b6259">∞ {n.partners[0]}{n.partners.length > 1 ? ` +${n.partners.length - 1}` : ''}</text>
            {/if}
          </g>
        {/each}
        <text x={D.W - Math.min(D.W, D.H) * 0.02} y={D.H - Math.min(D.W, D.H) * 0.02} font-size={Math.max(1.6, Math.min(D.W, D.H) * 0.009)} text-anchor="end" fill="#a39a8f">{credit}</text>
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
    background: #fbf8f2;
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
