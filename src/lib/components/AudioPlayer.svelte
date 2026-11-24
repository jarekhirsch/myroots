<script lang="ts">
  // A small player for one recording: play / pause, a scrubbable bar and the time.
  import { audioUrl, fmtTime } from '../audio';
  import { t } from '../i18n/index.svelte';
  import Play from '../icons/Play.svelte';
  import Pause from '../icons/Pause.svelte';

  let { media, title = '', sub = '' }: { media: number; title?: string; sub?: string } = $props();

  let el: HTMLAudioElement | undefined = $state();
  let src = $state('');
  let playing = $state(false);
  let time = $state(0);
  let duration = $state(0);
  let failed = $state(false);
  // recordings from the browser (WebM) have no length in the header: seek to the end once to learn it
  let probing = false;
  function meta() {
    if (!el) return;
    if (Number.isFinite(el.duration)) duration = el.duration;
    else {
      probing = true;
      el.currentTime = 1e101;
    }
  }
  function tick() {
    if (!el) return;
    if (probing) {
      if (Number.isFinite(el.duration)) {
        duration = el.duration;
        probing = false;
        el.currentTime = 0;
      }
      return;
    }
    time = el.currentTime;
  }

  // Only one recording plays at a time.
  const BUS = 'myroots-audio';
  $effect(() => {
    const stop = (e: Event) => {
      if ((e as CustomEvent).detail !== media) el?.pause();
    };
    window.addEventListener(BUS, stop);
    return () => window.removeEventListener(BUS, stop);
  });

  async function toggle() {
    if (!el) return;
    if (!src) {
      src = await audioUrl(media);
      if (!src) {
        failed = true;
        return;
      }
      await new Promise((r) => setTimeout(r, 0));
    }
    if (el.paused) {
      window.dispatchEvent(new CustomEvent(BUS, { detail: media }));
      el.play().catch(() => (failed = true));
    } else el.pause();
  }

  function seek(e: Event) {
    if (!el) return;
    const v = +(e.target as HTMLInputElement).value;
    el.currentTime = v;
    time = v;
  }
  // a preview of the length before the first play
  $effect(() => {
    void media;
    audioUrl(media).then((u) => {
      if (u && !src) src = u;
    });
  });
</script>

<div class="ap" class:playing>
  <button type="button" class="pp" onclick={toggle} aria-label={playing ? t('Pause') : t('Play')} disabled={failed}>
    {#if playing}<Pause size={18} fill="currentColor" />{:else}<Play size={18} fill="currentColor" />{/if}
  </button>
  <div class="body">
    {#if title || sub}
      <div class="ttl"><span class="name">{title}</span>{#if sub}<span class="sub">{sub}</span>{/if}</div>
    {/if}
    <div class="track">
      <input type="range" min="0" max={duration || 1} step="0.1" value={time} oninput={seek} aria-label={t('Position')} disabled={!duration} style="--p:{duration ? (time / duration) * 100 : 0}%" />
      <span class="tm">{failed ? t('Cannot play this file') : `${fmtTime(time)} / ${fmtTime(duration)}`}</span>
    </div>
  </div>
  <audio
    bind:this={el}
    {src}
    preload="metadata"
    onplay={() => (playing = true)}
    onpause={() => (playing = false)}
    onended={() => ((playing = false), (time = 0))}
    ontimeupdate={tick}
    onloadedmetadata={meta}
    ondurationchange={() => el && Number.isFinite(el.duration) && (duration = el.duration)}
    onerror={() => src && (failed = true)}
  ></audio>
</div>

<style>
  .ap {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .pp {
    flex: none;
    width: 42px;
    height: 42px;
    border-radius: 50%;
    border: 0;
    display: grid;
    place-items: center;
    background: var(--ink);
    color: var(--surface);
    cursor: pointer;
    padding: 0;
  }
  .pp :global(svg) {
    margin-inline-start: 2px;
  }
  .playing .pp :global(svg) {
    margin: 0;
  }
  .pp:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .pp:focus-visible {
    outline: 2px solid var(--ink);
    outline-offset: 2px;
  }
  .body {
    flex: 1;
    min-width: 0;
  }
  .ttl {
    display: flex;
    gap: 8px;
    align-items: baseline;
    flex-wrap: wrap;
    margin-bottom: 2px;
  }
  .name {
    font-weight: 600;
  }
  .sub {
    color: var(--muted);
    font-size: 0.85rem;
  }
  .track {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .track input {
    flex: 1;
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
  .track input::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--ink);
  }
  .track input::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border: 0;
    border-radius: 50%;
    background: var(--ink);
  }
  .tm {
    flex: none;
    font-size: 0.8rem;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
</style>
