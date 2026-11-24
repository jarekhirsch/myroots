<script lang="ts">
  // "Back" to the previous view (or, after opening a link directly, to the
  // natural parent: profile → people, edit → profile …).
  import { back, canGoBack, route } from '../router.svelte';
  import { t } from '../i18n/index.svelte';
  import ArrowLeft from '../icons/ArrowLeft.svelte';

  let { pill = false }: { pill?: boolean } = $props();
  const show = $derived((void route.path, void route.query, canGoBack()));
</script>

{#if show}
  <button type="button" class="back no-print" class:pill onclick={() => back()} title={t('Back')} aria-label={t('Back')}>
    <ArrowLeft size={17} strokeWidth={2} />
    <span>{t('Back')}</span>
  </button>
{/if}

<style>
  .back {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 0;
    background: none;
    font: inherit;
    font-size: 0.9rem;
    font-weight: 560;
    color: var(--muted);
    padding: 6px 12px 6px 8px;
    margin-inline-start: -8px;
    border-radius: 999px;
    cursor: pointer;
  }
  .back:hover {
    color: var(--ink);
    background: color-mix(in srgb, var(--ink) 7%, transparent);
  }
  .back:focus-visible {
    outline: 2px solid var(--ink);
    outline-offset: 2px;
  }
  /* inside the floating toolbars of the tree and the map */
  .pill {
    margin: 0;
    background: var(--surface);
    box-shadow: var(--shadow);
    color: var(--ink);
    padding: 8px 14px 8px 11px;
  }
  .pill:hover {
    background: var(--surface);
  }
  :global([dir='rtl']) .back :global(svg) {
    transform: scaleX(-1);
  }
</style>
