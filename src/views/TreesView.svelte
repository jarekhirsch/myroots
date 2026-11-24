<script lang="ts">
  // Several separate family trees: the list, a new tree, switching, renaming, deleting.
  import { summaries } from '../lib/api.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { toast, confirmDialog, promptDialog } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import { backend } from '../lib/backend';
  import { trees, loadTrees, treeLabel, openTree, createTree, renameTree, removeTree, hasTrees } from '../lib/trees.svelte';
  import FolderTree from '../lib/icons/Folders.svelte';
  import Plus from '../lib/icons/Plus.svelte';
  import Pencil from '../lib/icons/PencilSimple.svelte';
  import Trash from '../lib/icons/Trash.svelte';
  import Check from '../lib/icons/Check.svelte';

  let busy = $state(false);
  let people = $state<number | null>(null);
  $effect(() => {
    summaries().then((v) => (people = v.length)).catch(() => {});
  });
  let name = $state('');
  const wp = backend.platform === 'wp';

  $effect(() => {
    if (hasTrees()) loadTrees().catch((e) => toast(errorText(e), 'error'));
  });

  async function run(f: () => Promise<unknown>) {
    busy = true;
    try {
      await f();
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }

  function add(e: Event) {
    e.preventDefault();
    const n = name.trim();
    if (!n) return;
    run(() => createTree(n, true));
  }

  async function rename(id: string, current: string) {
    const n = await promptDialog(t('Rename the family tree'), t('Name of the family tree'), current, t('Save'));
    if (n == null || !n.trim()) return;
    run(async () => {
      await renameTree(id, n);
      toast(t('Saved.'));
    });
  }

  async function remove(id: string, label: string) {
    if (!(await confirmDialog(t('Delete the tree “%s”?', label), t('All people, relations, sources and photos of this tree will be deleted. This cannot be undone. Make a backup first if you are not sure.'), t('Delete'), true))) return;
    run(async () => {
      await removeTree(id);
      toast(t('The tree was deleted.'));
    });
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Family trees')}</h1>
      <p class="lead">{wp ? t('Keep separate trees on this site, for example the families of both parents. Each tree has its own people, photos and its own family page.') : t('Keep separate trees on this device, for example the families of both parents. Each tree has its own people, photos, settings and backups.')}</p>
    </div>
  </div>

  {#if !hasTrees()}
    <p class="muted">{t('Several trees are not available here.')}</p>
  {:else}
    <div class="list" aria-busy={busy}>
      {#each trees.list as tr, i (tr.id)}
        {@const label = treeLabel(tr, t('Family tree %d', i + 1))}
        {@const open = tr.id === trees.current}
        <div class="row">
          <span class="ic" class:on={open}><FolderTree size={20} strokeWidth={1.8} /></span>
          <div class="grow">
            <div class="title">{label}</div>
            <div class="sub">
              {#if open}{#if people != null}{tn('%d person', '%d people', people)}{/if}{:else if tr.created}{t('Created %s', new Date(tr.created).toLocaleDateString())}{/if}
            </div>
          </div>
          {#if open}
            <span class="tag ok"><Check size={14} /> {t('Open now')}</span>
          {:else}
            <button class="btn sm" disabled={busy} onclick={() => run(() => openTree(tr.id))}>{t('Open')}</button>
          {/if}
          <button class="btn ghost icon" disabled={busy} onclick={() => rename(tr.id, label)} aria-label={t('Rename')} title={t('Rename')}><Pencil size={16} /></button>
          {#if !open}
            <button class="btn ghost icon" disabled={busy} onclick={() => remove(tr.id, label)} aria-label={t('Delete')} title={t('Delete')}><Trash size={16} /></button>
          {/if}
        </div>
      {/each}
    </div>

    <section class="panel mt">
      <h2>{t('New family tree')}</h2>
      <p class="small muted">{t('A new, empty tree. You can start from scratch or import a GEDCOM file or a backup into it.')}</p>
      <form class="add" onsubmit={add}>
        <input type="text" bind:value={name} placeholder={t('e.g. The Kowalski family')} aria-label={t('Name of the family tree')} maxlength="120" />
        <button class="btn primary" disabled={busy || !name.trim()}><Plus size={16} /> {t('Create and open')}</button>
      </form>
    </section>
  {/if}
</div>

<style>
  .ic {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    background: var(--surface-2);
    color: var(--muted);
    flex-shrink: 0;
  }
  .ic.on {
    background: var(--ink-soft);
    color: var(--ink);
  }
  .mt {
    margin-top: 18px;
  }
  .add {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    margin-top: 10px;
  }
  .add input {
    flex: 1 1 240px;
    min-width: 0;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
</style>
