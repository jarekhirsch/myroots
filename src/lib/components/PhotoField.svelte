<script lang="ts">
  // A single image (profile photo, cover, event picture).
  import Img from './Img.svelte';
  import MediaLibrary from './MediaLibrary.svelte';
  import { pickImages } from '../media';
  import { t } from '../i18n/index.svelte';
  import { toast } from '../ui.svelte';
  import Upload from '../icons/UploadSimple.svelte';
  import Library from '../icons/Images.svelte';
  import X from '../icons/X.svelte';

  let {
    value = $bindable(null),
    size = 120,
    round = false,
    onfile,
  }: { value?: number | null; size?: number; round?: boolean; onfile?: (file: File) => void } = $props();
  let lib = $state(false);
  let busy = $state(false);

  async function upload() {
    busy = true;
    try {
      const ids = await pickImages(false, (f) => onfile?.(f));
      if (ids.length) value = ids[0];
    } catch {
      toast(t('This file could not be read as an image.'), 'error');
    } finally {
      busy = false;
    }
  }
</script>

<div class="pf">
  <div class="prev" class:round style="--s:{size}px">
    {#if value}
      <Img id={value} alt="" />
    {:else}
      <span class="none small faint">{t('no photo')}</span>
    {/if}
  </div>
  <div class="btns">
    <button type="button" class="btn sm" onclick={upload} disabled={busy}><Upload size={15} /> {t('Upload')}</button>
    <button type="button" class="btn sm ghost" onclick={() => (lib = true)}><Library size={15} /> {t('From library')}</button>
    {#if value}<button type="button" class="btn sm ghost" onclick={() => (value = null)}><X size={15} /> {t('Remove')}</button>{/if}
  </div>
</div>

{#if lib}
  <MediaLibrary multiple={false} onpick={(ids) => (value = ids[0] ?? value)} onclose={() => (lib = false)} />
{/if}

<style>
  .pf {
    display: flex;
    gap: 14px;
    align-items: center;
    flex-wrap: wrap;
  }
  .prev {
    width: var(--s);
    height: var(--s);
    border-radius: 10px;
    overflow: hidden;
    background: var(--ink-soft);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .prev.round {
    border-radius: 50%;
  }
  .prev :global(img),
  .prev :global(.ph) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .none {
    padding: 8px;
    text-align: center;
  }
  .btns {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: flex-start;
  }
</style>
