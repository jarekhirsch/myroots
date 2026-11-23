<script lang="ts">
  // A folded generation of the family timeline drawn on one canvas: one dot per
  // person at the year of birth. Used for big generations, where thousands of
  // separate links made the start page slow on phones. Clicking a dot opens the
  // person; expanding the generation shows the usual list of links.
  import { tone } from '../text';
  import type { Bar } from '../generations';

  let {
    bars,
    x,
    lanes,
    width,
    onhover,
  }: { bars: Bar[]; x: (year: number) => number; lanes: number; width: number; onhover?: (id: string | null) => void } = $props();

  const H = 12;
  const G = 4;
  const R = H / 2;
  let canvas: HTMLCanvasElement | undefined = $state();
  const height = $derived(Math.max(1, lanes * (H + G) - G));

  $effect(() => {
    const c = canvas;
    if (!c) return;
    // the largest canvas browsers draw safely
    const dpr = Math.min(window.devicePixelRatio || 1, 16000 / Math.max(1, width));
    c.width = Math.ceil(width * dpr);
    c.height = Math.ceil(height * dpr);
    const ctx = c.getContext('2d');
    if (!ctx) return;
    ctx.scale(dpr, dpr);
    const css = getComputedStyle(c);
    const colors = [1, 2, 3, 4, 5, 6].map((n) => css.getPropertyValue(`--p${n}t`).trim() || '#666');
    ctx.globalAlpha = 0.6;
    for (const b of bars) {
      ctx.fillStyle = colors[tone(b.name || '?') - 1];
      ctx.beginPath();
      ctx.arc(x(b.start), b.lane * (H + G) + R, R, 0, Math.PI * 2);
      ctx.fill();
    }
  });

  function at(e: MouseEvent): Bar | null {
    const r = canvas!.getBoundingClientRect();
    const px = e.clientX - r.left;
    const lane = Math.floor((e.clientY - r.top) / (H + G));
    let best: Bar | null = null;
    let dist = R + 2;
    for (const b of bars) {
      if (b.lane !== lane) continue;
      const d = Math.abs(x(b.start) - px);
      if (d < dist) (best = b), (dist = d);
    }
    return best;
  }

  let last: string | null = null;
  function move(e: MouseEvent) {
    const b = at(e);
    const id = b ? b.id : null;
    if (id === last) return;
    last = id;
    canvas!.title = b ? `${b.name} ${b.years}` : '';
    canvas!.style.cursor = b ? 'pointer' : '';
    onhover?.(id);
  }
  function click(e: MouseEvent) {
    const b = at(e);
    if (b) location.hash = `#/person/${b.id}`;
  }
</script>

<canvas
  bind:this={canvas}
  style="width: {width}px; height: {height}px"
  onmousemove={move}
  onmouseleave={() => ((last = null), onhover?.(null))}
  onclick={click}
  aria-hidden="true"
></canvas>

<style>
  canvas {
    display: block;
  }
</style>
