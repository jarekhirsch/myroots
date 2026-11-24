<script lang="ts">
  import { app, call } from '../lib/api.svelte';
  import { route } from '../lib/router.svelte';
  import { t } from '../lib/i18n/index.svelte';
  import { kinshipText, hopTerm } from '../lib/kinship';
  import { errorText } from '../lib/errors';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  import NotFound from './NotFound.svelte';
  import Swap from '../lib/icons/ArrowsLeftRight.svelte';
  import type { KinshipResult } from '../lib/types';

  let a = $state<number | null>(route.query.a ? Number(route.query.a) : null);
  let b = $state<number | null>(route.query.b ? Number(route.query.b) : null);
  let res = $state<KinshipResult | null>(null);
  let err = $state('');
  const blocked = $derived(app.info?.sections?.kinship === false);

  $effect(() => {
    const x = a;
    const y = b;
    res = null;
    err = '';
    if (!x || !y) return;
    history.replaceState(null, '', `#/kinship?a=${x}&b=${y}`);
    call<KinshipResult>('kinship', { a: x, b: y })
      .then((r) => (res = r))
      .catch((e) => (err = errorText(e)));
  });

  const text = $derived(res ? kinshipText(res) : null);
</script>

{#if blocked}
  <NotFound private />
{:else}
  <div class="page">
    <div class="page-head">
      <div>
        <h1>{t('Kinship calculator')}</h1>
        <p class="lead">{t('Choose two people to see how they are related.')}</p>
      </div>
    </div>
    <div class="panel form">
      <div class="p"><PersonPicker bind:value={a} placeholder={t('First person…')} exclude={b ? [b] : []} /></div>
      <button class="btn ghost icon" onclick={() => ([a, b] = [b, a])} aria-label={t('Swap')} title={t('Swap')}><Swap size={18} /></button>
      <div class="p"><PersonPicker bind:value={b} placeholder={t('Second person…')} exclude={a ? [a] : []} /></div>
    </div>

    {#if err}<div class="notice danger">{err}</div>{/if}

    {#if res && text}
      <div class="result">
        <p class="rel name">{text[0]}</p>
        {#if text[1]}<p class="muted">{text[1]}</p>{/if}
      </div>
      {#if res.path.length > 1}
        <h2 class="ph">{t('Path between the people')}</h2>
        <ol class="path">
          {#each res.path as n, i (n.id + ':' + i)}
            <li>
              {#if i > 0}<span class="hop">↓ {hopTerm(n.edge, n.gender)}</span>{/if}
              <a class="node" class:end={i === 0 || i === res.path.length - 1} href="#/person/{n.id}">
                <Avatar photo={n.photo} name={n.name} gender={n.gender} size={38} />
                <span><span class="name nm">{n.name}</span><br /><span class="small muted">{n.years}</span></span>
              </a>
            </li>
          {/each}
        </ol>
      {/if}
    {:else if !a || !b}
      <p class="muted">{t('Select both people.')}</p>
    {/if}
  </div>
{/if}

<style>
  .form {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-bottom: 22px;
  }
  .p {
    flex: 1;
    min-width: 0;
  }
  .result {
    text-align: center;
    padding: 22px 10px 6px;
  }
  .rel {
    font-size: 1.45rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    margin-bottom: 6px;
  }
  .ph {
    font-size: 1rem;
    text-align: center;
    margin-top: 20px;
  }
  .path {
    list-style: none;
    padding: 0;
    margin: 0 auto;
    max-width: 420px;
    display: flex;
    flex-direction: column;
    align-items: stretch;
  }
  .path li {
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .hop {
    font-size: 0.82rem;
    color: var(--heirloom);
    padding: 4px 0;
    font-weight: 550;
  }
  .node {
    display: flex;
    gap: 10px;
    align-items: center;
    background: var(--surface);
    box-shadow: var(--shadow);
    border-radius: 999px;
    padding: 5px 16px 5px 5px;
    text-decoration: none;
    color: var(--text);
    min-width: 240px;
  }
  .node.end {
    border-color: var(--heirloom);
    box-shadow: 0 0 0 1px var(--heirloom);
  }
  .nm {
    font-weight: 620;
  }
  @media (max-width: 640px) {
    .form {
      flex-direction: column;
      align-items: stretch;
    }
    .form .btn {
      align-self: center;
      transform: rotate(90deg);
    }
  }
</style>
