<script lang="ts">
  // Voice recordings of a person: upload files or record from the microphone.
  import AudioPlayer from './AudioPlayer.svelte';
  import AudioRecorder from './AudioRecorder.svelte';
  import DateInput from './DateInput.svelte';
  import { pickAudio, canRecord } from '../audio';
  import { toast } from '../ui.svelte';
  import { t } from '../i18n/index.svelte';
  import Upload from '@lucide/svelte/icons/upload';
  import Mic from '@lucide/svelte/icons/mic';
  import X from '@lucide/svelte/icons/x';
  import type { Recording } from '../types';

  let { items = $bindable([]), name = '' }: { items?: Recording[]; name?: string } = $props();
  let recording = $state(false);
  let busy = $state(false);
  const mic = canRecord();

  async function upload() {
    busy = true;
    try {
      const added = await pickAudio();
      items = [...items, ...added.map((a) => ({ media: a.id, title: a.name, date: '' }))];
    } catch (e: any) {
      toast(e?.code === 'audio_too_large' ? t('The file is too large (max. 80 MB).') : t('This file could not be added.'), 'error');
    } finally {
      busy = false;
    }
  }
</script>

<div class="re">
  {#each items as r, i (r.media)}
    <div class="it">
      <AudioPlayer media={r.media} />
      <div class="fields">
        <input type="text" bind:value={r.title} placeholder={t('Title')} aria-label={t('Title')} />
        <div class="date"><DateInput bind:value={r.date} /></div>
      </div>
      <button type="button" class="btn ghost icon sm" onclick={() => (items = items.filter((_, k) => k !== i))} aria-label={t('Remove')} title={t('Remove')}><X size={15} /></button>
    </div>
  {/each}
  <div class="add">
    {#if mic}<button type="button" class="btn sm" onclick={() => (recording = true)}><Mic size={15} /> {t('Record')}</button>{/if}
    <button type="button" class="btn sm ghost" onclick={upload} disabled={busy}><Upload size={15} /> {t('Upload a sound file')}</button>
  </div>
</div>

{#if recording}
  <AudioRecorder
    {name}
    onsave={(media, title) => {
      items = [...items, { media, title, date: new Date().toISOString().slice(0, 10) }];
      recording = false;
    }}
    onclose={() => (recording = false)}
  />
{/if}

<style>
  .re {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .it {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr) auto;
    gap: 12px;
    align-items: center;
    padding: 10px 12px;
    border-radius: var(--radius-lg);
    background: var(--surface-2, color-mix(in srgb, var(--ink) 4%, transparent));
  }
  .fields {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(250px, auto);
    gap: 8px;
  }
  .add {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  @media (max-width: 1000px) {
    .it {
      grid-template-columns: minmax(0, 1fr) auto;
    }
    .fields {
      grid-column: 1 / -1;
      grid-row: 2;
    }
  }
</style>
