<script lang="ts">
  // A quiet bar at the bottom: a new version of the desktop app is ready.
  import { upd, installUpdate, newsUrl } from '../updates.svelte';
  import { t, locale } from '../i18n/index.svelte';
  import Download from '@lucide/svelte/icons/download';
  import X from '@lucide/svelte/icons/x';
</script>

{#if upd.available && !upd.dismissed}
  <div class="upd no-print" role="status">
    <Download size={18} />
    <span class="grow">
      {#if upd.installing}
        {t('Installing MyRoots %s…', upd.available.version)} {upd.progress ? `${Math.round(upd.progress * 100)}%` : ''}
      {:else}
        {t('A new version of MyRoots is ready: %s.', upd.available.version)}
        <a href={newsUrl(locale().slice(0, 2))} target="_blank" rel="noopener noreferrer">{t("What's new")}</a>
      {/if}
    </span>
    {#if !upd.installing}
      <button class="btn sm primary" onclick={installUpdate}>{t('Install and restart')}</button>
      <button class="btn sm ghost icon" onclick={() => (upd.dismissed = true)} aria-label={t('Later')}><X size={16} /></button>
    {/if}
  </div>
{/if}

<style>
  .upd {
    position: fixed;
    inset-inline: 0;
    bottom: 0;
    margin: 0 auto 14px;
    max-width: 640px;
    width: calc(100% - 24px);
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px 10px 16px;
    background: var(--surface);
    color: var(--text);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    z-index: 900;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  @media (max-width: 900px) {
    .upd {
      bottom: calc(var(--tabbar) + env(safe-area-inset-bottom));
    }
  }
</style>
