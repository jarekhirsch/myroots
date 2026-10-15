<script lang="ts">
  // How one photo will look in "Through the years" (aligned on the eyes).
  import { loadPhoto, drawPhoto } from '../aging';
  import type { AgingFrame } from '../types';

  let { frame, width = 152 }: { frame: AgingFrame; width?: number } = $props();
  let c: HTMLCanvasElement | undefined = $state();
  const h = $derived(Math.round((width * 5) / 4));

  $effect(() => {
    const f = { media: frame.media, date: frame.date, eyes: frame.eyes ? ([...frame.eyes] as AgingFrame['eyes']) : null };
    const el = c;
    if (!el) return;
    loadPhoto(f.media)
      .then((img) => {
        const ctx = el.getContext('2d');
        if (!ctx) return;
        ctx.fillStyle = '#1b1f1d';
        ctx.fillRect(0, 0, el.width, el.height);
        drawPhoto(ctx, { frame: f, img }, { gray: false, zoom: 0, alpha: 1 }, el.width, el.height);
      })
      .catch(() => {});
  });
</script>

<canvas bind:this={c} width={width} height={h}></canvas>

<style>
  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }
</style>
