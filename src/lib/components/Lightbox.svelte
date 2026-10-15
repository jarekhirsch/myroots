<script lang="ts">
  import { ui } from '../ui.svelte';
  import { t } from '../i18n/index.svelte';
  import Img from './Img.svelte';
  import X from '@lucide/svelte/icons/x';
  import Left from '@lucide/svelte/icons/chevron-left';
  import Right from '@lucide/svelte/icons/chevron-right';

  const lb = $derived(ui.lightbox!);
  const item = $derived(lb.items[lb.index]);

  function close() {
    ui.lightbox = null;
  }
  function step(d: number) {
    if (!ui.lightbox) return;
    const n = ui.lightbox.items.length;
    ui.lightbox.index = (ui.lightbox.index + d + n) % n;
  }
  $effect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
      else if (e.key === 'ArrowRight') step(document.dir === 'rtl' ? -1 : 1);
      else if (e.key === 'ArrowLeft') step(document.dir === 'rtl' ? 1 : -1);
    };
    window.addEventListener('keydown', onKey);
    document.body.style.overflow = 'hidden';
    return () => {
      window.removeEventListener('keydown', onKey);
      document.body.style.overflow = '';
    };
  });

  let x0 = 0;
</script>

<div
  class="lb"
  role="dialog"
  tabindex="-1"
  aria-modal="true"
  aria-label={t('Gallery')}
  ontouchstart={(e) => (x0 = e.touches[0].clientX)}
  ontouchend={(e) => {
    const dx = e.changedTouches[0].clientX - x0;
    if (Math.abs(dx) > 50) step(dx < 0 ? 1 : -1);
  }}
>
  <button class="x" onclick={close} aria-label={t('Close')}><X size={24} /></button>
  {#if lb.items.length > 1}
    <button class="nav prev" onclick={() => step(-1)} aria-label={t('Previous')}><Left size={30} /></button>
    <button class="nav next" onclick={() => step(1)} aria-label={t('Next')}><Right size={30} /></button>
  {/if}
  <figure role="presentation" onclick={(e) => e.target === e.currentTarget && close()}>
    {#key item.media}
      <Img id={item.media} thumb={false} alt={item.caption} />
    {/key}
    {#if item.caption || lb.items.length > 1}
      <figcaption>
        {item.caption}
        {#if lb.items.length > 1}<span class="n">{lb.index + 1} / {lb.items.length}</span>{/if}
      </figcaption>
    {/if}
  </figure>
</div>

<style>
  .lb {
    position: fixed;
    inset: 0;
    z-index: 3000;
    background: rgba(8, 12, 16, 0.94);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  figure {
    margin: 0;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 56px 64px 24px;
    gap: 12px;
  }
  figure :global(img) {
    max-width: 100%;
    max-height: calc(100% - 50px);
    object-fit: contain;
    border-radius: 4px;
  }
  figure :global(.ph) {
    width: 200px;
    height: 200px;
    background: rgba(255, 255, 255, 0.06);
  }
  figcaption {
    color: #e8edf2;
    font-size: 0.95rem;
    text-align: center;
    max-width: 70ch;
  }
  .n {
    color: #9aa7b3;
    margin-inline-start: 10px;
    font-size: 0.85rem;
  }
  button {
    position: absolute;
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    border: 0;
    border-radius: 50%;
    width: 46px;
    height: 46px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 2;
  }
  button:hover {
    background: rgba(255, 255, 255, 0.18);
  }
  .x {
    top: 12px;
    inset-inline-end: 12px;
  }
  .prev {
    inset-inline-start: 12px;
  }
  .next {
    inset-inline-end: 12px;
  }
  :global([dir='rtl']) .nav :global(svg) {
    transform: scaleX(-1);
  }
  @media (max-width: 640px) {
    figure {
      padding: 56px 8px 16px;
    }
    .nav {
      top: auto;
      bottom: 14px;
    }
  }
</style>
