<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { go } from '../lib/router.svelte';
  import { t } from '../lib/i18n/index.svelte';
  import { fmtDate, fieldLabel, safeHref } from '../lib/format';
  import { confirmDialog, toast, openLightbox } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import Img from '../lib/components/Img.svelte';
  import Html from '../lib/components/Html.svelte';
  import PersonLink from '../lib/components/PersonLink.svelte';
  import NotFound from './NotFound.svelte';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Trash from '@lucide/svelte/icons/trash';
  import Ext from '@lucide/svelte/icons/external-link';
  import type { SourceView } from '../lib/types';

  let { id }: { id: number } = $props();
  const sv = useLoad(() => call<SourceView>('source.get', { id }));
  const s = $derived(sv.data?.source);
  const scans = $derived((s?.scans || []).map((m, i) => ({ media: m, caption: `${s?.title} (${i + 1})` })));
  const citeMap = $derived(new Map((sv.data?.citations || []).map(([p, f]) => [p.id, f])));

  async function remove() {
    if (!s || !(await confirmDialog(t('Delete %s?', s.title), t('Links to people and citations will be removed. Scans stay in the media library.'), t('Delete'), true))) return;
    try {
      await call('source.delete', { id });
      toast(t('Deleted.'));
      go('/sources', true);
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }

  function isAudio(u: string) {
    return /\.(mp3|m4a|ogg|wav|aac|flac)(\?|$)/i.test(u);
  }
  function isVideo(u: string) {
    return /\.(mp4|webm|mov|m4v)(\?|$)/i.test(u);
  }
</script>

{#if sv.error}
  <NotFound />
{:else if s && sv.data}
  <div class="page">
    <div class="page-head">
      <div>
        <h1>{s.title}</h1>
        {#if s.date}<p class="lead">{fmtDate(s.date)}</p>{/if}
      </div>
      <div class="actions">
        <a class="btn primary" href="#/source/{id}/edit"><Pencil size={16} /> {t('Edit')}</a>
        <button class="btn ghost danger icon" onclick={remove} aria-label={t('Delete')}><Trash size={16} /></button>
      </div>
    </div>
    <div class="grid-2">
      <div>
        {#if s.photo}
          <button class="cover" onclick={() => openLightbox([{ media: s.photo!, caption: s.title }])}><Img id={s.photo} thumb={false} alt={s.title} /></button>
        {/if}
        {#if s.notes}<section class="block"><h2>{t('Notes')}</h2><div class="panel"><Html html={s.notes} /></div></section>{/if}
        {#if safeHref(s.url)}<p><a href={safeHref(s.url)} target="_blank" rel="noopener noreferrer"><Ext size={14} /> {s.url}</a></p>{/if}
        {#if s.media_url}
          <section class="block">
            <h2>{t('Recordings and materials')}</h2>
            {#if isAudio(s.media_url)}<audio controls src={s.media_url}></audio>
            {:else if isVideo(s.media_url)}<!-- svelte-ignore a11y_media_has_caption --><video controls src={s.media_url}></video>
            {:else}<a href={safeHref(s.media_url) || undefined} target="_blank" rel="noopener noreferrer">{s.media_url}</a>{/if}
          </section>
        {/if}
        {#if scans.length}
          <section class="block">
            <h2>{t('Scans')}</h2>
            <div class="gal">
              {#each scans as g, i (g.media)}
                <button onclick={() => openLightbox(scans, i)}><Img id={g.media} alt="" /></button>
              {/each}
            </div>
          </section>
        {/if}
      </div>
      <div>
        <section class="block">
          <h2>{t('People')}</h2>
          <div class="panel ppl">
            {#each sv.data.persons as p (p.id)}
              <div>
                <PersonLink {p} />
                {#if citeMap.get(p.id)?.length}<div class="small muted cf">{t('Confirms:')} {citeMap.get(p.id)!.map(fieldLabel).join(', ')}</div>{/if}
              </div>
            {:else}
              <p class="muted small">{t('Not linked to anyone yet.')}</p>
            {/each}
          </div>
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .cover {
    border: 0;
    padding: 0;
    background: none;
    cursor: zoom-in;
    margin-bottom: 16px;
    display: block;
  }
  .cover :global(img) {
    max-height: 360px;
    border-radius: 12px;
    box-shadow: var(--shadow);
  }
  .gal {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 8px;
  }
  .gal button {
    border: 0;
    padding: 0;
    aspect-ratio: 3 / 4;
    border-radius: 10px;
    overflow: hidden;
    cursor: zoom-in;
    background: var(--ink-soft);
  }
  .gal :global(img),
  .gal :global(.ph) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .ppl {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .cf {
    padding-inline-start: 50px;
    margin-top: -4px;
  }
  audio,
  video {
    width: 100%;
  }
</style>
