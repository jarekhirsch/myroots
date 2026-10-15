<script lang="ts">
  import { mediaSrc } from '../media';
  let { id, thumb = true, alt = '', class: cls = '', style = '' }: { id: number | null | undefined; thumb?: boolean; alt?: string; class?: string; style?: string } = $props();
  let src = $state('');
  $effect(() => {
    const cur = id;
    const th = thumb;
    src = '';
    mediaSrc(cur, th).then((u) => {
      if (cur === id) src = u;
    });
  });
</script>

{#if src}
  <img {src} {alt} class={cls} {style} loading="lazy" decoding="async" />
{:else}
  <span class="ph {cls}" {style} aria-hidden="true"></span>
{/if}

<style>
  .ph {
    display: inline-block;
    background: var(--ink-soft);
  }
</style>
