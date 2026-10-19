<script lang="ts">
  // A scanned album page → separate, straightened photos.
  import Modal from './Modal.svelte';
  import { findPhotos, corners, cutOut, type Box } from '../albumsplit';
  import { prepareImage } from '../media';
  import { backend } from '../backend';
  import { toast } from '../ui.svelte';
  import { errorText } from '../errors';
  import { t, tn } from '../i18n/index.svelte';

  let { file, onadd, onclose }: { file: File; onadd: (ids: number[]) => void; onclose: () => void } = $props();

  let url = $state('');
  let img: HTMLImageElement | null = null;
  let size = $state<[number, number]>([1, 1]);
  let boxes = $state<(Box & { on: boolean })[]>([]);
  let ready = $state(false);
  let busy = $state(false);
  let svg: SVGSVGElement | undefined = $state();

  $effect(() => {
    const u = URL.createObjectURL(file);
    url = u;
    const im = new Image();
    im.onload = () => {
      img = im;
      size = [im.naturalWidth, im.naturalHeight];
      setTimeout(() => {
        try {
          boxes = findPhotos(im).map((b) => ({ ...b, on: true }));
        } catch {
          boxes = [];
        }
        ready = true;
      }, 30);
    };
    im.onerror = () => {
      toast(t('This file could not be read as an image.'), 'error');
      onclose();
    };
    im.src = u;
    return () => URL.revokeObjectURL(u);
  });

  const chosen = $derived(boxes.filter((b) => b.on).length);
  const poly = (b: Box) => corners(b, size[0], size[1]).map((p) => p.join(',')).join(' ');

  /* drawing a missing photo by hand */
  let start: [number, number] | null = null;
  let draft = $state<Box | null>(null);
  function pt(e: PointerEvent): [number, number] {
    const r = svg!.getBoundingClientRect();
    return [Math.min(1, Math.max(0, (e.clientX - r.left) / r.width)), Math.min(1, Math.max(0, (e.clientY - r.top) / r.height))];
  }
  function down(e: PointerEvent) {
    if ((e.target as Element).closest('.bx')) return;
    start = pt(e);
    svg!.setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!start) return;
    const p = pt(e);
    const ar = size[1] / size[0];
    draft = { cx: (start[0] + p[0]) / 2, cy: (start[1] + p[1]) / 2, w: Math.abs(p[0] - start[0]), h: Math.abs(p[1] - start[1]) * ar, angle: 0 };
  }
  function up() {
    if (draft && draft.w > 0.03 && draft.h > 0.03) boxes = [...boxes, { ...draft, on: true }];
    start = null;
    draft = null;
  }

  async function add() {
    if (!img) return;
    busy = true;
    const ids: number[] = [];
    const base = file.name.replace(/\.[^.]+$/, '') || 'album';
    try {
      let n = 0;
      for (const b of boxes.filter((x) => x.on)) {
        n++;
        const blob = await cutOut(img, b);
        ids.push(await backend.addMedia(await prepareImage(blob, `${base}-${n}.jpg`)));
      }
      onadd(ids);
      toast(tn('Added %d photo.', 'Added %d photos.', ids.length));
      onclose();
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
</script>

<Modal title={t('Split an album page')} {onclose} wide>
  <p class="hint">
    {#if !ready}
      {t('Looking for photos on the page…')}
    {:else if boxes.length}
      {tn('Found %d photo. Click a frame to leave it out, or draw a frame around a photo that was missed.', 'Found %d photos. Click a frame to leave it out, or draw a frame around a photo that was missed.', boxes.length)}
    {:else}
      {t('No separate photos were found. Draw a frame around each photo with the mouse or a finger.')}
    {/if}
  </p>
  <div class="page">
    <div class="wrap">
    {#if url}<img src={url} alt="" />{/if}
    <svg bind:this={svg} viewBox="0 0 {size[0]} {size[1]}" preserveAspectRatio="none" role="presentation" onpointerdown={down} onpointermove={move} onpointerup={up}>
      {#each boxes as b, i (i)}
        <g class="bx" class:off={!b.on} role="button" tabindex="0" aria-pressed={b.on} aria-label={t('Photo %d', i + 1)} onclick={() => (b.on = !b.on)} onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), (b.on = !b.on))}>
          <polygon points={poly(b)} />
          <text x={b.cx * size[0]} y={b.cy * size[1]} font-size={size[0] * 0.04} stroke-width={size[0] * 0.004}>{i + 1}</text>
        </g>
      {/each}
      {#if draft}<polygon class="draft" points={poly(draft)} />{/if}
    </svg>
    </div>
  </div>
  {#snippet footer()}
    <button type="button" class="btn" onclick={onclose}>{t('Cancel')}</button>
    <button type="button" class="btn primary" disabled={!chosen || busy} onclick={add}>{tn('Add %d photo', 'Add %d photos', chosen)}</button>
  {/snippet}
</Modal>

<style>
  .page {
    display: flex;
    justify-content: center;
    line-height: 0;
  }
  /* the picture and the frames share one cell: the frames always match the picture */
  .wrap {
    display: grid;
    max-width: 100%;
  }
  .wrap > :global(*) {
    grid-area: 1 / 1;
  }
  .page img {
    display: block;
    max-width: 100%;
    max-height: 62vh;
    object-fit: contain;
    border-radius: 4px;
    user-select: none;
    -webkit-user-drag: none;
  }
  svg {
    width: 100%;
    height: 100%;
    touch-action: none;
    cursor: crosshair;
  }
  polygon {
    fill: color-mix(in srgb, var(--heirloom, #d6b36a) 16%, transparent);
    stroke: var(--heirloom, #d6b36a);
    stroke-width: 0.5%;
    vector-effect: non-scaling-stroke;
  }
  .bx {
    cursor: pointer;
  }
  .bx polygon {
    stroke-width: 3px;
  }
  .bx.off polygon {
    fill: rgba(0, 0, 0, 0.35);
    stroke: #fff;
    stroke-dasharray: 8 6;
  }
  .bx:focus-visible polygon {
    stroke: var(--ink);
  }
  text {
    fill: #fff;
    font-weight: 700;
    text-anchor: middle;
    dominant-baseline: middle;
    paint-order: stroke;
    stroke: rgba(0, 0, 0, 0.6);
  }
  .draft {
    stroke-width: 2px;
  }
  .hint {
    margin-top: 0;
  }
</style>
