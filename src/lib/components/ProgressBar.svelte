<script lang="ts">
  // Bar of long file work (see `progress` / `withProgress` in ui.svelte.ts).
  import { progress } from '../ui.svelte';
  import { t } from '../i18n/index.svelte';
  const pct = $derived(progress.total ? Math.min(100, Math.round((progress.done / progress.total) * 100)) : null);
</script>

{#if progress.active}
  <div class="pbar no-print" role="progressbar" aria-label={progress.label || t('Working…')} aria-valuemin="0" aria-valuemax="100" aria-valuenow={pct ?? undefined}>
    <div class="track"><div class="fill" class:unknown={pct == null} style:width={pct == null ? undefined : `${pct}%`}></div></div>
    <div class="card">
      <span class="lbl">{progress.label || t('Working…')}</span>
      {#if pct != null}<span class="num">{progress.total > 1 && progress.total < 100000 ? `${progress.done} / ${progress.total} · ` : ''}{pct}%</span>{/if}
    </div>
  </div>
{/if}

<style>
  .pbar {
    position: fixed;
    inset-inline: 0;
    top: 0;
    z-index: 1000;
    pointer-events: none;
  }
  .track {
    height: 3px;
    background: color-mix(in srgb, var(--ink) 12%, transparent);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--ink);
    transition: width 0.25s ease;
  }
  .fill.unknown {
    width: 30%;
    animation: slide 1.2s ease-in-out infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(340%);
    }
  }
  .card {
    position: absolute;
    top: 10px;
    inset-inline-end: 14px;
    display: flex;
    gap: 10px;
    align-items: center;
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow);
    border-radius: var(--radius);
    padding: 8px 12px;
    font-size: 0.88rem;
  }
  .num {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  @media (prefers-reduced-motion: reduce) {
    .fill.unknown {
      animation: none;
      width: 100%;
      opacity: 0.4;
    }
  }
</style>
