// Several family trees: the list, switching and keeping the names in step.
import { backend } from './backend';
import { app, call, refreshInfo } from './api.svelte';
import type { TreeInfo, TreeList } from './backend/types';

export const trees = $state({ list: [] as TreeInfo[], current: '', loaded: false });

export function hasTrees(): boolean {
  return !!backend?.trees && !backend.readonly;
}

/** The name shown for a tree (the open one uses the name from its settings). */
export function treeLabel(tr: TreeInfo, fallback: string): string {
  if (tr.id === trees.current && app.info?.tree_name) return app.info.tree_name;
  return tr.name || fallback;
}

export async function loadTrees(): Promise<TreeList | null> {
  if (!hasTrees()) return null;
  const r = await backend.trees!.list();
  trees.list = r.trees;
  trees.current = r.current;
  trees.loaded = true;
  return r;
}

/**
 * After start: the name typed when a tree was created goes into its settings,
 * and a name changed in the settings goes into the list of trees.
 */
export async function syncTreeName() {
  const r = await loadTrees();
  if (!r) return;
  const cur = r.trees.find((x) => x.id === r.current);
  if (!cur) return;
  const own = (app.info?.tree_name || '').trim();
  if (!own && cur.name && app.settings) {
    await call('settings.save', { settings: { ...app.settings, tree_name: cur.name } });
    await refreshInfo();
  } else if (own && own !== cur.name) {
    await backend.trees!.rename(cur.id, own);
    cur.name = own;
    trees.list = [...r.trees];
  }
}

/** Opens another tree: everything is saved first, then the app starts again on it. */
export async function openTree(id: string) {
  await backend.flush?.();
  await backend.trees!.open(id);
  location.hash = '#/';
  location.reload();
}

export async function createTree(name: string, open = true): Promise<string> {
  const id = await backend.trees!.create(name);
  if (open) await openTree(id);
  else await loadTrees();
  return id;
}

export async function renameTree(id: string, name: string) {
  await backend.trees!.rename(id, name);
  if (id === trees.current && app.settings) {
    await call('settings.save', { settings: { ...app.settings, tree_name: name.trim() } });
    await refreshInfo();
  }
  await loadTrees();
}

export async function removeTree(id: string) {
  await backend.trees!.remove(id);
  await loadTrees();
}
