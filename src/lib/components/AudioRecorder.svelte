<script lang="ts">
  // Recording from the microphone: record, listen, keep or try again.
  import Modal from './Modal.svelte';
  import { recorderMime, storeAudio, fmtTime } from '../audio';
  import { t } from '../i18n/index.svelte';
  import Mic from '@lucide/svelte/icons/mic';
  import Square from '@lucide/svelte/icons/square';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';

  let { name = '', onsave, onclose }: { name?: string; onsave: (media: number, title: string) => void; onclose: () => void } = $props();

  type State = 'idle' | 'asking' | 'recording' | 'done' | 'saving' | 'denied';
  let st = $state<State>('idle');
  let seconds = $state(0);
  let level = $state(0);
  let blob: Blob | null = null;
  let preview = $state('');
  let title = $state('');
  let rec: MediaRecorder | null = null;
  let stream: MediaStream | null = null;
  let ctx: AudioContext | null = null;
  let raf = 0;
  let timer = 0;
  const bars = $state<number[]>(Array(48).fill(0));

  async function start() {
    st = 'asking';
    try {
      stream = await navigator.mediaDevices.getUserMedia({ audio: { echoCancellation: true, noiseSuppression: true } });
    } catch {
      st = 'denied';
      return;
    }
    const mime = recorderMime();
    const chunks: Blob[] = [];
    rec = new MediaRecorder(stream, mime ? { mimeType: mime } : undefined);
    rec.ondataavailable = (e) => e.data.size && chunks.push(e.data);
    rec.onstop = () => {
      blob = new Blob(chunks, { type: rec?.mimeType || mime || 'audio/webm' });
      if (preview) URL.revokeObjectURL(preview);
      preview = URL.createObjectURL(blob);
      st = 'done';
    };
    // level meter
    ctx = new AudioContext();
    const an = ctx.createAnalyser();
    an.fftSize = 512;
    ctx.createMediaStreamSource(stream).connect(an);
    const data = new Uint8Array(an.fftSize);
    const loop = () => {
      an.getByteTimeDomainData(data);
      let peak = 0;
      for (const v of data) peak = Math.max(peak, Math.abs(v - 128) / 128);
      level = peak;
      bars.push(Math.min(1, peak * 1.6));
      bars.shift();
      raf = requestAnimationFrame(loop);
    };
    loop();
    seconds = 0;
    const t0 = performance.now();
    timer = window.setInterval(() => (seconds = (performance.now() - t0) / 1000), 200);
    rec.start(1000);
    st = 'recording';
  }

  function release() {
    cancelAnimationFrame(raf);
    clearInterval(timer);
    stream?.getTracks().forEach((tr) => tr.stop());
    stream = null;
    ctx?.close().catch(() => {});
    ctx = null;
  }

  function stop() {
    rec?.stop();
    release();
    bars.fill(0);
  }

  function again() {
    blob = null;
    if (preview) URL.revokeObjectURL(preview);
    preview = '';
    start();
  }

  async function save() {
    if (!blob) return;
    st = 'saving';
    const ext = blob.type.includes('mp4') ? 'm4a' : blob.type.includes('ogg') ? 'ogg' : 'webm';
    const stamp = new Date().toISOString().slice(0, 10);
    const id = await storeAudio(blob, `${name || 'recording'}-${stamp}.${ext}`, blob.type);
    onsave(id, title.trim());
  }

  function close() {
    if (st === 'recording') rec?.stop();
    release();
    if (preview) URL.revokeObjectURL(preview);
    onclose();
  }
  $effect(() => () => release());
</script>

<Modal title={t('Record a voice')} onclose={close}>
  <div class="rec">
    <div class="meter" class:live={st === 'recording'} aria-hidden="true">
      {#each bars as b, i (i)}<span style="--h:{Math.max(0.06, b)}"></span>{/each}
    </div>
    <div class="time" aria-live="polite">{fmtTime(seconds)}</div>

    {#if st === 'denied'}
      <p class="err">{t('No access to the microphone. Allow it in the system or browser settings and try again.')}</p>
      <button type="button" class="btn" onclick={start}>{t('Try again')}</button>
    {:else if st === 'idle' || st === 'asking'}
      <p class="hint">{t('Ask a question and let them talk; you can listen before saving.')}</p>
      <button type="button" class="big rec-btn" onclick={start} disabled={st === 'asking'} aria-label={t('Start recording')}><Mic size={26} /></button>
    {:else if st === 'recording'}
      <button type="button" class="big stop-btn" onclick={stop} aria-label={t('Stop recording')}><Square size={22} fill="currentColor" /></button>
      <p class="hint">{t('Recording…')} <span class="dot" style="opacity:{0.4 + level}"></span></p>
    {:else}
      <audio controls src={preview}></audio>
      <label class="field full"><span class="lbl">{t('Title')}</span><input type="text" bind:value={title} placeholder={t('e.g. Grandma tells about the war')} onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), save())} /></label>
      <div class="row">
        <button type="button" class="btn ghost" onclick={again} disabled={st === 'saving'}><RotateCcw size={15} /> {t('Record again')}</button>
        <button type="button" class="btn primary" onclick={save} disabled={st === 'saving'}>{t('Save recording')}</button>
      </div>
    {/if}
  </div>
</Modal>

<style>
  .rec {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    padding: 6px 0 4px;
    text-align: center;
  }
  .meter {
    display: flex;
    align-items: center;
    gap: 3px;
    height: 64px;
    width: 100%;
    justify-content: center;
  }
  .meter span {
    width: 4px;
    border-radius: 2px;
    height: calc(var(--h) * 100%);
    background: color-mix(in srgb, var(--ink) 25%, transparent);
    transition: height 0.08s linear;
  }
  .meter.live span {
    background: var(--ink);
  }
  .time {
    font-size: 2rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
  }
  .big {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    border: 0;
    display: grid;
    place-items: center;
    cursor: pointer;
    color: #fff;
  }
  .rec-btn {
    background: #c0392b;
    box-shadow: 0 0 0 6px color-mix(in srgb, #c0392b 18%, transparent);
  }
  .stop-btn {
    background: var(--ink);
  }
  .big:focus-visible {
    outline: 2px solid var(--ink);
    outline-offset: 4px;
  }
  .hint {
    color: var(--muted);
    margin: 0;
    max-width: 40ch;
  }
  .dot {
    display: inline-block;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #c0392b;
    vertical-align: middle;
  }
  .err {
    color: var(--danger, #b03a2e);
    margin: 0;
  }
  audio {
    width: 100%;
  }
  .field {
    width: 100%;
    text-align: start;
  }
  .row {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    width: 100%;
  }
</style>
