<script lang="ts">
  import type { Snippet } from 'svelte';
  import X from '../icons/X.svelte';
  import { t } from '../i18n/index.svelte';
  let { title = '', onclose, children, footer, wide = false }: { title?: string; onclose: () => void; children: Snippet; footer?: Snippet; wide?: boolean } = $props();
  let dlg: HTMLDivElement;
  $effect(() => {
    const prev = document.activeElement as HTMLElement | null;
    dlg?.querySelector<HTMLElement>('input,select,textarea,button:not(.x)')?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onclose();
    };
    window.addEventListener('keydown', onKey);
    document.body.style.overflow = 'hidden';
    return () => {
      window.removeEventListener('keydown', onKey);
      document.body.style.overflow = '';
      prev?.focus?.();
    };
  });
</script>

<div class="ov" role="presentation" onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="dlg" class:wide role="dialog" tabindex="-1" aria-modal="true" aria-label={title} bind:this={dlg}>
    <header>
      <h2>{title}</h2>
      <button type="button" class="btn ghost icon x" onclick={onclose} aria-label={t('Close')}><X size={18} /></button>
    </header>
    <div class="body">{@render children()}</div>
    {#if footer}<footer>{@render footer()}</footer>{/if}
  </div>
</div>

<style>
  .ov {
    position: fixed;
    inset: 0;
    background: rgba(15, 26, 21, 0.42);
    backdrop-filter: blur(3px);
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
  }
  .dlg {
    background: var(--surface);
    border-radius: 24px;
    box-shadow: var(--shadow-pop);
    width: min(560px, 100%);
    max-height: calc(100vh - 32px);
    display: flex;
    flex-direction: column;
  }
  .dlg.wide {
    width: min(920px, 100%);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 18px 16px 6px 24px;
  }
  header h2 {
    margin: 0;
    font-size: 1.2rem;
  }
  .body {
    padding: 8px 24px 20px;
    overflow: auto;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 24px 20px;
  }
  @media (max-width: 640px) {
    .ov {
      align-items: flex-end;
      padding: 0;
    }
    .dlg,
    .dlg.wide {
      width: 100%;
      border-radius: 24px 24px 0 0;
      max-height: 92vh;
    }
  }
</style>
