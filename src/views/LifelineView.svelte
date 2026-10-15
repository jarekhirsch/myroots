<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { go } from '../lib/router.svelte';
  import { t } from '../lib/i18n/index.svelte';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import LifelineList from '../lib/components/LifelineList.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  import NotFound from './NotFound.svelte';
  import type { Lifeline } from '../lib/types';

  let { id = null }: { id?: number | null } = $props();
  let pick = $state<number | null>(null);
  $effect(() => {
    pick = id;
  });
  const ll = useLoad(() => (id ? call<Lifeline>('lifeline', { id }) : Promise.resolve(null)));
</script>

<div class="page">
  <div class="page-head">
    <h1>{t('Lifeline')}</h1>
    <div class="pick"><PersonPicker bind:value={pick} placeholder={t('Search for a person…')} onchange={(v) => v && go(`/lifeline/${v}`)} /></div>
  </div>
  {#if ll.error}
    <NotFound private={(ll.error as any)?.code === 'private'} />
  {:else if ll.data}
    <a class="who" href="#/person/{ll.data.person}">
      <Avatar photo={ll.data.photo} name={ll.data.name} gender={ll.data.gender} size={56} />
      <span><span class="name nm">{ll.data.name}</span><br /><span class="muted">{ll.data.years}</span></span>
    </a>
    <div class="panel"><LifelineList data={ll.data} /></div>
  {:else if !id}
    <div class="panel empty">{t('Choose a person to see their lifeline.')}</div>
  {/if}
</div>

<style>
  .pick {
    width: min(360px, 100%);
  }
  .who {
    display: flex;
    gap: 14px;
    align-items: center;
    text-decoration: none;
    color: var(--text);
    margin-bottom: 16px;
  }
  .nm {
    font-size: 1.3rem;
    font-weight: 620;
  }
</style>
