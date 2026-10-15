<script lang="ts">
  import Avatar from './Avatar.svelte';
  import { t } from '../i18n/index.svelte';
  import type { Summary } from '../types';
  let { p, sub = '', size = 42, tag = '' }: { p: Summary; sub?: string; size?: number; tag?: string } = $props();
</script>

{#if p.anon}
  <span class="pl">
    <Avatar anon size={size} />
    <span class="txt"><span class="nm muted">{t('Private')}</span></span>
  </span>
{:else}
  <a class="pl" href="#/person/{p.id}">
    <Avatar photo={p.photo} name={p.name} gender={p.gender} {size} />
    <span class="txt">
      <span class="nm name">{p.name}</span>
      <span class="yr">{sub || p.years}{#if tag}<span class="tag heirloom">{tag}</span>{/if}</span>
    </span>
  </a>
{/if}

<style>
  .pl {
    display: flex;
    align-items: center;
    gap: 12px;
    text-decoration: none;
    color: inherit;
    min-width: 0;
    padding: 6px 8px;
    border-radius: 14px;
  }
  a.pl:hover {
    background: var(--surface-2);
  }
  .txt {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.3;
  }
  .nm {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .yr {
    font-size: 0.84rem;
    color: var(--muted);
    display: flex;
    gap: 6px;
    align-items: center;
  }
</style>
