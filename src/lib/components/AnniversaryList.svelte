<script lang="ts">
  import Avatar from './Avatar.svelte';
  import { t, tn } from '../i18n/index.svelte';
  import { fmtDate } from '../format';
  import type { Anniversary } from '../types';
  let { items }: { items: Anniversary[] } = $props();

  function label(a: Anniversary): string {
    if (a.kind === 'birth') return a.living ? t('%d. birthday', a.years) : t('%d. birth anniversary', a.years);
    if (a.kind === 'death') return t('%d. death anniversary', a.years);
    return t('%d. wedding anniversary', a.years);
  }
  function when(a: Anniversary): string {
    if (a.days_left === 0) return t('today');
    if (a.days_left === 1) return t('tomorrow');
    return tn('in %d day', 'in %d days', a.days_left);
  }
</script>

<div class="list">
  {#each items as a (a.kind + a.person + a.date)}
    <a class="row" href="#/person/{a.person}">
      <Avatar photo={a.photo} name={a.name} size={38} />
      <span class="grow">
        <span class="title name">{a.name}{#if a.partner_name} <span class="muted">&amp;</span> {a.partner_name}{/if}</span>
        <span class="sub">{label(a)} · {fmtDate(a.date)}</span>
      </span>
      <span class="tag" class:heirloom={a.days_left <= 1}>{when(a)}</span>
    </a>
  {/each}
</div>

<style>
  .title,
  .sub {
    display: block;
  }
</style>
