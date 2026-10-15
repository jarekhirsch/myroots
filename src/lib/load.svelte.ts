import { app } from './api.svelte';

/**
 * Reactive loader: runs `fn` on mount and again after every data change
 * (app.version) or when a reactive value read synchronously inside `fn`
 * (e.g. a route parameter) changes.
 */
export function useLoad<T>(fn: () => Promise<T>) {
  const s = $state({ data: null as T | null, error: null as unknown, loading: true });
  $effect(() => {
    void app.version;
    let alive = true;
    s.loading = true;
    let p: Promise<T>;
    try {
      p = fn();
    } catch (e) {
      s.error = e;
      s.loading = false;
      return;
    }
    p.then(
      (d) => {
        if (!alive) return;
        s.data = d;
        s.error = null;
        s.loading = false;
      },
      (e) => {
        if (!alive) return;
        s.error = e;
        s.loading = false;
      },
    );
    return () => {
      alive = false;
    };
  });
  return s;
}
