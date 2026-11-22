import { app } from './api.svelte';

/**
 * Reactive loader: runs `fn` on mount and again after every data change
 * (app.version) or when a reactive value read synchronously inside `fn`
 * (e.g. a route parameter) changes.
 *
 * The loaded data is kept as it came (`$state.raw`): views only read it, and
 * a deep reactive proxy around tens of thousands of records (big trees) made
 * every list, filter and sort several times slower.
 */
export function useLoad<T>(fn: () => Promise<T>) {
  let data = $state.raw<T | null>(null);
  let error = $state.raw<unknown>(null);
  let loading = $state(true);
  $effect(() => {
    void app.version;
    let alive = true;
    loading = true;
    let p: Promise<T>;
    try {
      p = fn();
    } catch (e) {
      error = e;
      loading = false;
      return;
    }
    p.then(
      (d) => {
        if (!alive) return;
        data = d;
        error = null;
        loading = false;
      },
      (e) => {
        if (!alive) return;
        error = e;
        loading = false;
      },
    );
    return () => {
      alive = false;
    };
  });
  return {
    get data() {
      return data;
    },
    get error() {
      return error;
    },
    get loading() {
      return loading;
    },
  };
}
