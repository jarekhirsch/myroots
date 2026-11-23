<script lang="ts">
  import { assetUrl } from './lib/backend/assets';
  import { t } from './lib/i18n/index.svelte';
  let { error = '', code = '' }: { error?: string; code?: string } = $props();
  // a plain explanation first; the technical text below helps when reporting the problem
  const hint = $derived(
    /^wp_http_5\d\d$/.test(code)
      ? t('The server could not send your data. Try again in a moment; if it keeps happening, the hosting may need more PHP memory (memory_limit).')
      : /fetch|network|load failed/i.test(error)
        ? t('No connection. Check the internet and try again.')
        : '',
  );
</script>

<div class="boot">
  <img src={assetUrl('icon.svg')} alt="" width="56" height="56" />
  {#if error}
    <h1>{t('MyRoots could not start')}</h1>
    {#if hint}<p>{hint}</p>{/if}
    <p class="muted small">{error}</p>
    <button class="btn" onclick={() => location.reload()}>{t('Try again')}</button>
  {:else}
    <div class="spin" aria-label={t('Loading…')}></div>
  {/if}
</div>

<style>
  .boot {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 24px;
    text-align: center;
  }
  h1 {
    font-size: 1.4rem;
  }
  .spin {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: 3px solid var(--line);
    border-top-color: var(--heirloom);
    animation: s 0.8s linear infinite;
  }
  @keyframes s {
    to {
      transform: rotate(360deg);
    }
  }
</style>
