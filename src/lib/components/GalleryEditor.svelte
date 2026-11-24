<script lang="ts">
  // Ordered list of images with captions.
  import Img from './Img.svelte';
  import MediaLibrary from './MediaLibrary.svelte';
  import { pickImages, pickFile } from '../media';
  import AlbumSplitter from './AlbumSplitter.svelte';
  import Scan from '../icons/Scan.svelte';
  import { t } from '../i18n/index.svelte';
  import Upload from '../icons/UploadSimple.svelte';
  import Library from '../icons/Images.svelte';
  import X from '../icons/X.svelte';
  import Left from '../icons/ArrowLeft.svelte';
  import Right from '../icons/ArrowRight.svelte';
  import type { GalleryItem } from '../types';

  let { items = $bindable([]), captions = true }: { items?: GalleryItem[]; captions?: boolean } = $props();
  let lib = $state(false);
  let busy = $state(false);
  let page = $state<File | null>(null);

  function add(ids: number[]) {
    const have = new Set(items.map((i) => i.media));
    items = [...items, ...ids.filter((id) => !have.has(id)).map((media) => ({ media, caption: '' }))];
  }
  async function upload() {
    busy = true;
    try {
      add(await pickImages(true));
    } finally {
      busy = false;
    }
  }
  function move(i: number, d: number) {
    const j = i + d;
    if (j < 0 || j >= items.length) return;
    const c = [...items];
    [c[i], c[j]] = [c[j], c[i]];
    items = c;
  }
</script>

<div class="ge">
  {#each items as g, i (g.media)}
    <div class="it">
      <div class="im"><Img id={g.media} alt="" /></div>
      {#if captions}<input type="text" placeholder={t('Caption')} bind:value={g.caption} aria-label={t('Caption')} />{/if}
      <div class="ctl">
        <button type="button" class="btn ghost icon sm" onclick={() => move(i, -1)} disabled={i === 0} aria-label={t('Move left')}><Left size={14} /></button>
        <button type="button" class="btn ghost icon sm" onclick={() => move(i, 1)} disabled={i === items.length - 1} aria-label={t('Move right')}><Right size={14} /></button>
        <button type="button" class="btn ghost icon sm" onclick={() => (items = items.filter((_, k) => k !== i))} aria-label={t('Remove')}><X size={14} /></button>
      </div>
    </div>
  {/each}
  <div class="add">
    <button type="button" class="btn sm" onclick={upload} disabled={busy}><Upload size={15} /> {t('Upload photos')}</button>
    <button type="button" class="btn sm ghost" onclick={() => (lib = true)}><Library size={15} /> {t('From library')}</button>
    <button type="button" class="btn sm ghost" onclick={async () => (page = await pickFile('image/*'))}><Scan size={15} /> {t('Album page scan')}</button>
  </div>
</div>

{#if lib}
  <MediaLibrary onpick={add} onclose={() => (lib = false)} />
{/if}
{#if page}
  <AlbumSplitter file={page} onadd={add} onclose={() => (page = null)} />
{/if}

<style>
  .ge {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 12px;
  }
  .it {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--surface-2);
    box-shadow: var(--shadow);
    border-radius: 14px;
    padding: 6px;
  }
  .im {
    aspect-ratio: 4 / 3;
    border-radius: 10px;
    overflow: hidden;
    background: var(--ink-soft);
  }
  .im :global(img),
  .im :global(.ph) {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .it input {
    min-height: 32px;
    font-size: 0.85rem;
    padding: 5px 8px;
  }
  .ctl {
    display: flex;
    justify-content: space-between;
  }
  :global([dir='rtl']) .ctl :global(svg) {
    transform: scaleX(-1);
  }
  .add {
    display: flex;
    flex-direction: column;
    gap: 6px;
    justify-content: center;
    align-items: flex-start;
    border: 1px dashed var(--line-strong);
    border-radius: 14px;
    padding: 12px;
    min-height: 120px;
  }
</style>
