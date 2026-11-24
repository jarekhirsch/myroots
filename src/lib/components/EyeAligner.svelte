<script lang="ts">
  // Setting the eyes on one photo: two markers to drag onto the centres of
  // the eyes, with a live preview of how the photo will line up.
  import Modal from './Modal.svelte';
  import { loadPhoto, drawPhoto, W, H } from '../aging';
  import { detectEyes } from '../faces';
  import { photoUrl } from '../aging';
  import { t } from '../i18n/index.svelte';
  import Wand from '../icons/MagicWand.svelte';
  import ZoomIn from '../icons/MagnifyingGlassPlus.svelte';
  import ZoomOut from '../icons/MagnifyingGlassMinus.svelte';
  import type { AgingFrame } from '../types';

  let { frame, onsave, onclose }: { frame: AgingFrame; onsave: (eyes: [number, number, number, number]) => void; onclose: () => void } = $props();

  const start = $state.snapshot(frame);
  let eyes = $state<[number, number, number, number]>(start.eyes ? [...start.eyes] : [0.42, 0.4, 0.58, 0.4]);
  const unset = !start.eyes;
  let iw = $state(1);
  let ih = $state(1);
  // the visible part of the photo (fractions): zoomed on the face when it is known
  let zoomed = $state(!!start.eyes);
  let view = $state({ x: 0, y: 0, w: 1, h: 1 });
  function frameFace() {
    if (!zoomed) {
      view = { x: 0, y: 0, w: 1, h: 1 };
      return;
    }
    const d = Math.hypot((eyes[2] - eyes[0]) * iw, (eyes[3] - eyes[1]) * ih);
    const side = Math.max(d * 4.2, Math.min(iw, ih) * 0.22);
    const w = Math.min(1, side / iw);
    const h = Math.min(1, (side * 1.2) / ih);
    const cx = ((eyes[0] + eyes[2]) / 2) * iw;
    const cy = ((eyes[1] + eyes[3]) / 2) * ih + side * 0.12;
    const x = Math.min(1 - w, Math.max(0, cx / iw - w / 2));
    const y = Math.min(1 - h, Math.max(0, cy / ih - h / 2));
    view = { x, y, w, h };
  }
  const boxRatio = $derived((view.w * iw) / (view.h * ih));
  let src = $state('');
  let box: HTMLDivElement | undefined = $state();
  let preview: HTMLCanvasElement | undefined = $state();
  let img: HTMLImageElement | null = null;
  let drag: 0 | 1 | null = null;
  let finding = $state(false);
  let notFound = $state(false);

  $effect(() => {
    photoUrl(frame.media).then((u) => (src = u));
    loadPhoto(frame.media).then((i) => {
      img = i;
      iw = i.naturalWidth;
      ih = i.naturalHeight;
      frameFace();
      paint();
    });
  });

  function paint() {
    const ctx = preview?.getContext('2d');
    if (!ctx || !img) return;
    ctx.fillStyle = '#1b1f1d';
    ctx.fillRect(0, 0, W, H);
    drawPhoto(ctx, { frame: { ...start, eyes }, img }, { gray: false, zoom: 0, alpha: 1 });
    // guide: where the eyes land
    ctx.strokeStyle = 'rgba(255,255,255,0.55)';
    ctx.setLineDash([10, 10]);
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.moveTo(0, H * 0.42);
    ctx.lineTo(W, H * 0.42);
    ctx.stroke();
  }
  $effect(() => {
    void eyes[0], eyes[1], eyes[2], eyes[3];
    paint();
  });

  function pos(e: PointerEvent): [number, number] {
    const r = box!.getBoundingClientRect();
    const fx = (e.clientX - r.left) / r.width;
    const fy = (e.clientY - r.top) / r.height;
    return [Math.min(1, Math.max(0, view.x + fx * view.w)), Math.min(1, Math.max(0, view.y + fy * view.h))];
  }
  function down(i: 0 | 1, e: PointerEvent) {
    drag = i;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    e.preventDefault();
  }
  function move(e: PointerEvent) {
    if (drag == null) return;
    const [x, y] = pos(e);
    const next = [...eyes] as typeof eyes;
    next[drag * 2] = x;
    next[drag * 2 + 1] = y;
    eyes = next;
  }
  function up() {
    if (drag == null) return;
    drag = null;
    // keep left on the left
    if (eyes[0] > eyes[2]) eyes = [eyes[2], eyes[3], eyes[0], eyes[1]];
  }
  function nudge(i: 0 | 1, e: KeyboardEvent) {
    const d = e.shiftKey ? 0.01 : 0.002;
    const k = { ArrowLeft: [-d, 0], ArrowRight: [d, 0], ArrowUp: [0, -d], ArrowDown: [0, d] }[e.key];
    if (!k) return;
    e.preventDefault();
    const next = [...eyes] as typeof eyes;
    next[i * 2] = Math.min(1, Math.max(0, next[i * 2] + k[0]));
    next[i * 2 + 1] = Math.min(1, Math.max(0, next[i * 2 + 1] + k[1]));
    eyes = next;
  }

  async function auto() {
    finding = true;
    notFound = false;
    try {
      const e = await detectEyes(src);
      if (e) {
        eyes = e;
        zoomed = true;
        frameFace();
      } else notFound = true;
    } catch {
      notFound = true;
    } finally {
      finding = false;
    }
  }
</script>

<Modal title={t('Set the eyes')} onclose={onclose} wide>
  <p class="hint">
    {unset ? t('No face was found automatically. Drag the two circles onto the centres of the eyes.') : t('Drag the circles onto the centres of the eyes. Arrow keys move them precisely.')}
  </p>
  <div class="cols">
    <div class="pic">
      {#if src}
        <div class="box" bind:this={box} style="aspect-ratio:{boxRatio};width:min(100%, calc(60vh * {boxRatio}))" onpointermove={move} onpointerup={up} onpointercancel={up} role="presentation">
          <img
            {src}
            alt=""
            draggable="false"
            style="width:{100 / view.w}%;height:{100 / view.h}%;left:{(-view.x / view.w) * 100}%;top:{(-view.y / view.h) * 100}%"
          />
          {#each [0, 1] as i (i)}
            <button
              type="button"
              class="eye"
              style="left:{((eyes[i * 2] - view.x) / view.w) * 100}%;top:{((eyes[i * 2 + 1] - view.y) / view.h) * 100}%"
              onpointerdown={(e) => down(i as 0 | 1, e)}
              onkeydown={(e) => nudge(i as 0 | 1, e)}
              aria-label={i === 0 ? t('Eye on the left') : t('Eye on the right')}
            ></button>
          {/each}
        </div>
      {/if}
    </div>
    <div class="side">
      <span class="lbl-s">{t('Preview')}</span>
      <canvas bind:this={preview} width={W} height={H}></canvas>
      <button class="btn sm ghost" onclick={auto} disabled={finding || !src}><Wand size={15} /> {finding ? t('Looking for the face…') : t('Find automatically')}</button>
      <button class="btn sm ghost" onclick={() => ((zoomed = !zoomed), frameFace())} disabled={!src}>
        {#if zoomed}<ZoomOut size={15} /> {t('Whole photo')}{:else}<ZoomIn size={15} /> {t('Zoom in on the face')}{/if}
      </button>
      {#if notFound}<p class="small muted">{t('No face found on this photo.')}</p>{/if}
    </div>
  </div>
  {#snippet footer()}
    <button class="btn ghost" onclick={onclose}>{t('Cancel')}</button>
    <button class="btn primary" onclick={() => onsave(eyes)}>{t('Save')}</button>
  {/snippet}
</Modal>

<style>
  .hint {
    margin: 0 0 12px;
    color: var(--muted);
  }
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 200px;
    gap: 18px;
    align-items: start;
  }
  .pic {
    display: flex;
    justify-content: center;
    background: #1b1f1d;
    border-radius: var(--radius-lg);
    padding: 8px;
  }
  .box {
    position: relative;
    overflow: hidden;
    touch-action: none;
    user-select: none;
  }
  .box img {
    position: absolute;
    display: block;
    max-width: none;
    pointer-events: none;
  }
  .eye {
    position: absolute;
    width: 26px;
    height: 26px;
    margin: -13px 0 0 -13px;
    border-radius: 50%;
    border: 2px solid #fff;
    background: rgba(255, 255, 255, 0.12);
    box-shadow:
      0 0 0 2px rgba(0, 0, 0, 0.55),
      inset 0 0 0 1px rgba(0, 0, 0, 0.4);
    cursor: grab;
    padding: 0;
    touch-action: none;
  }
  .eye::after {
    content: '';
    position: absolute;
    left: 50%;
    top: 50%;
    width: 4px;
    height: 4px;
    margin: -2px 0 0 -2px;
    border-radius: 50%;
    background: #fff;
  }
  .eye:active {
    cursor: grabbing;
  }
  .eye:focus-visible {
    outline: 3px solid var(--accent, #f4c430);
    outline-offset: 2px;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .side canvas {
    width: 100%;
    height: auto;
    border-radius: 10px;
    background: #1b1f1d;
  }
  @media (max-width: 640px) {
    .cols {
      grid-template-columns: minmax(0, 1fr);
    }
    .side canvas {
      max-width: 160px;
    }
  }
</style>
