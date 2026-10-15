<script lang="ts">
  // Portrait of a person: a soft rounded square. Without a photo, the initials
  // sit on one of six gentle colours derived from the name (stable per person).
  import Img from './Img.svelte';
  import { initials } from '../format';
  import { tone } from '../text';
  // `fill`: a portrait tile that takes the width of its container (4:5).
  let { photo = null, name = '', gender = '', size = 40, anon = false, fill = false }: { photo?: number | null; name?: string; gender?: string; size?: number; anon?: boolean; fill?: boolean } = $props();

  const n = $derived(anon ? 1 : tone(name || '?'));
</script>

<span class="av" class:fill class:photo={photo && !anon} style="--s:{size}px; --bg: var(--p{n}); --fg: var(--p{n}t)" data-g={gender || undefined} aria-hidden="true">
  {#if photo && !anon}
    <Img id={photo} alt="" />
  {:else}
    <span class="ini">{anon ? '?' : initials(name)}</span>
  {/if}
</span>

<style>
  .av {
    width: var(--s);
    height: var(--s);
    border-radius: calc(var(--s) * 0.32);
    overflow: hidden;
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--bg);
    color: var(--fg);
    font-size: calc(var(--s) * 0.36);
    font-weight: 650;
    letter-spacing: 0.01em;
    position: relative;
  }
  .av.fill {
    width: 100%;
    height: auto;
    aspect-ratio: 4 / 5;
    border-radius: var(--radius-lg);
    font-size: clamp(2rem, 22cqi, 3.4rem);
    font-weight: 600;
    letter-spacing: 0.02em;
  }
  .av.photo {
    background: var(--surface-2);
  }
  .av :global(img),
  .av :global(.ph) {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
</style>
