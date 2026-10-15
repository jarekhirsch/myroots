<script lang="ts">
  // Plays "Through the years": the photos aligned on the eyes, cross-fading
  // from the oldest to the newest, optionally with a recording, and records
  // it to a video file.
  import { untrack } from 'svelte';
  import { W, H, loadPhoto, drawPhoto, drawCaptions, totalTime, at, frameYear, type Loaded, type Timing } from '../aging';
  import { audioUrl } from '../audio';
  import { backend } from '../backend';
  import { toast } from '../ui.svelte';
  import { t, tn } from '../i18n/index.svelte';
  import { year as yearOf } from '../format';
  import { slug, stamp } from '../backend/util';
  import Play from '@lucide/svelte/icons/play';
  import Pause from '@lucide/svelte/icons/pause';
  import Download from '@lucide/svelte/icons/download';
  import type { AgingFrame } from '../types';

  let {
    frames,
    birth = '',
    name = '',
    audio = null,
    compact = false,
  }: { frames: AgingFrame[]; birth?: string; name?: string; audio?: number | null; compact?: boolean } = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let loaded = $state<Loaded[]>([]);
  let playing = $state(false);
  let time = $state(0);
  let speed = $state<'slow' | 'normal' | 'fast'>('normal');
  let gray = $state(false);
  let exporting = $state(0); // progress 0..1 while recording a video
  let audioDur = $state(0);
  let sound: HTMLAudioElement | null = null;
  let raf = 0;
  let last = 0;

  const SPEEDS: Record<string, Timing> = { slow: { hold: 2.6, fade: 1.3 }, normal: { hold: 1.5, fade: 0.9 }, fast: { hold: 0.8, fade: 0.5 } };
  const timing = $derived.by<Timing>(() => {
    const base = SPEEDS[speed];
    const n = loaded.length;
    // with a recording: the photos last as long as the voice (within reason)
    if (audio && audioDur > 0 && n) {
      const hold = (audioDur - (n - 1) * base.fade) / n;
      return { hold: Math.min(8, Math.max(0.8, hold)), fade: base.fade };
    }
    return base;
  });
  const total = $derived(totalTime(loaded.length, timing));
  const birthYear = $derived(yearOf(birth));

  // load the photos (in order of the frames)
  $effect(() => {
    const fs = frames.slice();
    untrack(() => {
      Promise.all(fs.map((f) => loadPhoto(f.media).then((img) => ({ frame: f, img })).catch(() => null))).then((v) => {
        loaded = v.filter((x): x is Loaded => !!x);
        time = Math.min(time, totalTime(loaded.length, timing));
        draw();
      });
    });
  });

  // the recording that plays along
  $effect(() => {
    const id = audio;
    sound?.pause();
    sound = null;
    audioDur = 0;
    if (!id) return;
    audioUrl(id).then((u) => {
      if (!u || id !== audio) return;
      const a = new Audio(u);
      a.preload = 'metadata';
      a.onloadedmetadata = () => {
        if (Number.isFinite(a.duration)) audioDur = a.duration;
        else {
          a.currentTime = 1e101;
          a.ontimeupdate = () => {
            if (Number.isFinite(a.duration)) {
              audioDur = a.duration;
              a.ontimeupdate = null;
              a.currentTime = 0;
            }
          };
        }
      };
      sound = a;
    });
  });

  function caption(f: AgingFrame) {
    const y = frameYear(f);
    return y == null ? '' : String(y);
  }
  function age(f: AgingFrame) {
    const y = frameYear(f);
    if (y == null || birthYear == null || y < birthYear) return '';
    const a = y - birthYear;
    return a === 0 ? t('birth year') : tn('%d year', '%d years', a);
  }

  function draw() {
    const ctx = canvas?.getContext('2d');
    if (!ctx) return;
    ctx.fillStyle = '#1b1f1d';
    ctx.fillRect(0, 0, W, H);
    if (!loaded.length) return;
    const p = at(loaded.length, timing, time);
    const a = loaded[p.i];
    const b = p.mix > 0 ? loaded[p.i + 1] : null;
    const zoomA = Math.min(1, (time - p.i * (timing.hold + timing.fade)) / (timing.hold + timing.fade * 2));
    drawPhoto(ctx, a, { gray, zoom: zoomA, alpha: 1 });
    if (b) drawPhoto(ctx, b, { gray, zoom: 0, alpha: p.mix });
    drawCaptions(ctx, a.frame, b?.frame || null, p.mix, { name, year: caption, age });
  }

  $effect(() => {
    // redraw on option changes while paused
    void gray;
    void timing;
    if (!playing) untrack(draw);
  });

  function loop(now: number) {
    const dt = last ? (now - last) / 1000 : 0;
    last = now;
    time = Math.min(total, time + dt);
    draw();
    if (exporting) exporting = Math.max(0.001, time / total);
    if (time >= total) {
      stop();
      return;
    }
    raf = requestAnimationFrame(loop);
  }

  function play() {
    if (loaded.length < 1) return;
    if (time >= total) time = 0;
    playing = true;
    last = 0;
    if (sound) {
      if (time < (audioDur || Infinity)) {
        sound.currentTime = time;
        sound.play().catch(() => {});
      }
    }
    raf = requestAnimationFrame(loop);
  }

  function stop() {
    playing = false;
    cancelAnimationFrame(raf);
    sound?.pause();
    if (exportDone) exportDone();
  }

  function toggle() {
    if (playing) stop();
    else play();
  }

  function seek(e: Event) {
    time = +(e.target as HTMLInputElement).value;
    if (sound && playing) sound.currentTime = Math.min(time, audioDur || time);
    if (!playing) draw();
  }

  $effect(() => () => {
    cancelAnimationFrame(raf);
    sound?.pause();
  });

  /* ---------- video ---------- */
  let exportDone: (() => void) | null = null;
  const canExport = typeof MediaRecorder !== 'undefined' && typeof HTMLCanvasElement !== 'undefined' && 'captureStream' in HTMLCanvasElement.prototype;

  function videoMime(): string {
    // MP4 (H.264) plays everywhere; WebM where H.264 recording is missing; plain MP4 for Safari
    for (const m of ['video/mp4;codecs=avc1.42E01E,mp4a.40.2', 'video/mp4;codecs=avc1', 'video/webm;codecs=vp9,opus', 'video/webm;codecs=vp8,opus', 'video/webm', 'video/mp4'])
      if (MediaRecorder.isTypeSupported(m)) return m;
    return '';
  }

  async function exportVideo() {
    if (!canvas || !loaded.length || exporting) return;
    try {
      canvas.toDataURL(); // throws when the photos come from another origin (website opened from a disk)
    } catch {
      toast(t('This browser cannot record a video of these photos.'), 'error');
      return;
    }
    stop();
    time = 0;
    draw();
    const stream = canvas.captureStream(30);
    let actx: AudioContext | null = null;
    let el: HTMLAudioElement | null = null;
    if (audio) {
      const u = await audioUrl(audio);
      if (u) {
        actx = new AudioContext();
        el = new Audio(u);
        const dest = actx.createMediaStreamDestination();
        actx.createMediaElementSource(el).connect(dest);
        dest.stream.getAudioTracks().forEach((tr) => stream.addTrack(tr));
      }
    }
    const mime = videoMime();
    const chunks: Blob[] = [];
    const rec = new MediaRecorder(stream, { mimeType: mime || undefined, videoBitsPerSecond: 6_000_000 });
    rec.ondataavailable = (e) => e.data.size && chunks.push(e.data);
    const finished = new Promise<void>((r) => (rec.onstop = () => r()));
    exporting = 0.001;
    const quiet = sound;
    sound = null; // the recording goes into the file, not the speakers
    exportDone = () => {
      exportDone = null;
      setTimeout(() => rec.state !== 'inactive' && rec.stop(), 300);
    };
    rec.start(500);
    el?.play().catch(() => {});
    play();
    await finished;
    el?.pause();
    actx?.close().catch(() => {});
    sound = quiet;
    exporting = 0;
    const type = rec.mimeType || mime || 'video/webm';
    const ext = type.includes('mp4') ? 'mp4' : 'webm';
    const blob = new Blob(chunks, { type });
    const where = await backend.saveFile(`${slug(name || 'myroots')}-${t('through-the-years')}-${stamp()}.${ext}`, new Uint8Array(await blob.arrayBuffer()), type.split(';')[0]);
    if (where) toast(t('Saved: %s', where));
  }

  const ticks = $derived.by(() => {
    const step = timing.hold + timing.fade;
    return loaded.map((l, i) => ({ at: total ? (i * step) / total : 0, label: caption(l.frame) }));
  });
</script>

<div class="ag" class:compact>
  <div class="stage">
    <canvas bind:this={canvas} width={W} height={H} aria-label={t('Through the years')}></canvas>
    {#if loaded.length}
      <button class="big" class:hide={playing} onclick={toggle} aria-label={playing ? t('Pause') : t('Play')}>
        {#if playing}<Pause size={30} fill="currentColor" />{:else}<Play size={30} fill="currentColor" />{/if}
      </button>
      <button class="hit" onclick={toggle} tabindex="-1" aria-hidden="true"></button>
    {/if}
    {#if exporting}
      <div class="rec-badge">{t('Recording the video…')} {Math.round(exporting * 100)}%</div>
    {/if}
  </div>

  {#if loaded.length}
    <div class="bar">
      <button class="btn ghost icon sm" onclick={toggle} aria-label={playing ? t('Pause') : t('Play')} disabled={!!exporting}>
        {#if playing}<Pause size={16} />{:else}<Play size={16} />{/if}
      </button>
      <div class="scrub">
        <input type="range" min="0" max={total} step="0.01" value={time} oninput={seek} aria-label={t('Position')} disabled={!!exporting} style="--p:{total ? (time / total) * 100 : 0}%" />
        {#if !compact}
          <div class="ticks" aria-hidden="true">
            {#each ticks as tk, i (i)}<span style="left:{tk.at * 100}%">{tk.label}</span>{/each}
          </div>
        {/if}
      </div>
    </div>
    {#if !compact}
      <div class="opts">
        {#if !(audio && audioDur)}
          <div class="seg" role="group" aria-label={t('Speed')}>
            <button aria-pressed={speed === 'slow'} onclick={() => (speed = 'slow')}>{t('Slow')}</button>
            <button aria-pressed={speed === 'normal'} onclick={() => (speed = 'normal')}>{t('Normal')}</button>
            <button aria-pressed={speed === 'fast'} onclick={() => (speed = 'fast')}>{t('Fast')}</button>
          </div>
        {:else}
          <span class="small muted">{t('The photos last as long as the recording.')}</span>
        {/if}
        <label class="check small"><input type="checkbox" bind:checked={gray} /> {t('Black and white')}</label>
        {#if canExport}
          <button class="btn sm" onclick={exportVideo} disabled={!!exporting}><Download size={15} /> {t('Save as video')}</button>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .ag {
    width: 100%;
  }
  .compact {
    max-width: 380px;
  }
  .stage {
    position: relative;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: #1b1f1d;
    box-shadow: var(--shadow);
  }
  canvas {
    display: block;
    width: 100%;
    height: auto;
    aspect-ratio: 4 / 5;
  }
  .hit {
    position: absolute;
    inset: 0;
    background: none;
    border: 0;
    cursor: pointer;
  }
  .big {
    position: absolute;
    z-index: 2;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 76px;
    height: 76px;
    border-radius: 50%;
    border: 0;
    display: grid;
    place-items: center;
    color: #fff;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(6px);
    cursor: pointer;
    transition: opacity 0.25s;
  }
  .big :global(svg) {
    margin-inline-start: 4px;
  }
  .big.hide {
    opacity: 0;
    pointer-events: none;
  }
  .stage:hover .big.hide {
    opacity: 0.85;
    pointer-events: auto;
  }
  .big:focus-visible {
    outline: 2px solid #fff;
    outline-offset: 3px;
    opacity: 1;
  }
  .rec-badge {
    position: absolute;
    top: 12px;
    right: 12px;
    background: #c0392b;
    color: #fff;
    border-radius: 999px;
    padding: 5px 12px;
    font-size: 0.82rem;
    font-weight: 600;
  }
  .bar {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-top: 10px;
  }
  .scrub {
    flex: 1;
    min-width: 0;
    padding-top: 13px;
  }
  .scrub input {
    width: 100%;
    appearance: none;
    -webkit-appearance: none;
    height: 4px;
    border-radius: 2px;
    padding: 0;
    border: 0;
    box-shadow: none;
    min-height: 0;
    background: linear-gradient(to right, var(--ink) var(--p), color-mix(in srgb, var(--ink) 15%, transparent) var(--p));
    cursor: pointer;
  }
  .scrub input::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--ink);
  }
  .scrub input::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border: 0;
    border-radius: 50%;
    background: var(--ink);
  }
  .ticks {
    position: relative;
    height: 18px;
    margin-top: 6px;
  }
  .ticks span {
    position: absolute;
    transform: translateX(-50%);
    font-size: 0.72rem;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .ticks span:first-child {
    transform: none;
  }
  .opts {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 16px;
    margin-top: 8px;
  }
  .seg {
    display: flex;
    gap: 2px;
    padding: 3px;
    border-radius: 999px;
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  .seg button {
    border: 0;
    background: none;
    font: inherit;
    font-size: 0.85rem;
    color: var(--muted);
    padding: 5px 12px;
    border-radius: 999px;
    cursor: pointer;
  }
  .seg button[aria-pressed='true'] {
    background: var(--ink);
    color: var(--surface);
    font-weight: 600;
  }
  .opts .btn {
    margin-inline-start: auto;
  }
</style>
