<script lang="ts" module>
  import { call } from '../api.svelte';
  import type { PlaceUse } from '../types';
  let cache: { version: number; names: string[] } | null = null;
  let pending: Promise<string[]> | null = null;
  export async function placeNames(version: number): Promise<string[]> {
    if (cache && cache.version === version) return cache.names;
    if (!pending) {
      pending = call<PlaceUse[]>('places.list').then((v) => {
        const names = v.map((p) => p.name);
        cache = { version, names };
        pending = null;
        return names;
      });
    }
    return pending;
  }
</script>

<script lang="ts">
  // Text input with suggestions from places already used in the tree.
  import { app } from '../api.svelte';
  let { value = $bindable(''), id = undefined }: { value?: string; id?: string } = $props();
  const listId = 'places-' + Math.random().toString(36).slice(2);
  let names = $state.raw<string[]>([]);
  let loaded = false;
  function load() {
    if (loaded) return;
    loaded = true;
    placeNames(app.version).then((n) => (names = n));
  }
</script>

<input {id} type="text" bind:value list={listId} onfocus={load} autocomplete="off" />
<datalist id={listId}>
  {#each names as n}<option value={n}></option>{/each}
</datalist>
