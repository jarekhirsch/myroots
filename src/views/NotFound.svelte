<script lang="ts">
  import { t } from '../lib/i18n/index.svelte';
  import Lock from '../lib/icons/Lock.svelte';
  let { private: priv = false }: { private?: boolean } = $props();
</script>

<div class="page">
  <div class="panel empty">
    {#if priv}
      <Lock size={30} strokeWidth={1.5} />
      <h2>{t('Private content')}</h2>
      <p>{t('This part of the family website is not public.')}</p>
    {:else}
      <h2>{t('Not found')}</h2>
      <p>{t('This page does not exist or has been removed.')}</p>
    {/if}
    <a class="btn" href="#/">{t('Back to start')}</a>
  </div>
</div>

<style>
  .empty :global(svg) {
    color: var(--faint);
    margin-bottom: 8px;
  }
</style>
