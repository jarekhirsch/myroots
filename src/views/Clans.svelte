<script lang="ts">
  import { app, call, refreshInfo } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { toast, confirmDialog, promptDialog } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import Plus from '../lib/icons/Plus.svelte';
  import type { ClanInfo } from '../lib/types';

  const clans = useLoad(() => call<ClanInfo[]>('clans.list'));
  let name = $state('');

  async function saveDefs(list: string[]) {
    await call('settings.save', { settings: { ...app.settings!, clans: list } });
    await refreshInfo();
  }
  async function add() {
    const n = name.trim();
    if (!n) return;
    const defs = app.settings?.clans || [];
    if (!defs.includes(n)) await saveDefs([...defs, n]);
    name = '';
    toast(t('Clan “%s” has been added.', n));
  }
  async function rename(c: ClanInfo) {
    const to = (await promptDialog(t('Rename clan'), t('New name'), c.name, t('Rename')))?.trim();
    if (!to || to === c.name) return;
    try {
      const r = await call<{ count: number }>('clans.rename', { from: c.name, to });
      await refreshInfo();
      toast(tn('Renamed for %d person.', 'Renamed for %d people.', r.count));
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }
  async function remove(c: ClanInfo) {
    if (!(await confirmDialog(t('Remove clan “%s”?', c.name), c.count ? tn('It will be removed from %d person. The people stay in the tree.', 'It will be removed from %d people. The people stay in the tree.', c.count) : '', t('Remove'), true))) return;
    await call('clans.delete', { name: c.name });
    await refreshInfo();
  }
  async function define(c: ClanInfo) {
    await saveDefs([...(app.settings?.clans || []), c.name]);
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Clans')}</h1>
      <p class="lead">{t('Clans (family lines) group people, e.g. by surname. They appear in filters, the map, statistics and the book.')}</p>
    </div>
  </div>
  <form class="panel add" onsubmit={(e) => { e.preventDefault(); add(); }}>
    <label class="field grow"><span class="lbl">{t('Clan name')}</span><input type="text" bind:value={name} /></label>
    <button class="btn primary" type="submit" disabled={!name.trim()}><Plus size={16} /> {t('Add')}</button>
  </form>
  {#if clans.data?.length}
    <table class="data panel tbl">
      <thead><tr><th>{t('Clan')}</th><th>{t('People')}</th><th></th></tr></thead>
      <tbody>
        {#each clans.data as c (c.name)}
          <tr>
            <td><a href="#/people?clan={encodeURIComponent(c.name)}" class="name">{c.name}</a>{#if !c.defined} <button class="tag" onclick={() => define(c)} title={t('Add to the list of defined clans')}>+</button>{/if}</td>
            <td>{c.count}</td>
            <td class="act">
              <a class="btn sm ghost" href="#/map?clan={encodeURIComponent(c.name)}">{t('On the map')}</a>
              <a class="btn sm ghost" href="#/tree?clan={encodeURIComponent(c.name)}">{t('In the tree')}</a>
              <button class="btn sm ghost" onclick={() => rename(c)}>{t('Rename')}</button>
              <button class="btn sm ghost danger" onclick={() => remove(c)}>{t('Remove')}</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else if clans.data}
    <div class="panel empty">{t('No clans yet.')}</div>
  {/if}
</div>

<style>
  .add {
    display: flex;
    gap: 10px;
    align-items: flex-end;
    margin-bottom: 18px;
  }
  .grow {
    flex: 1;
  }
  .tbl {
    padding: 0;
  }
  .act {
    text-align: end;
    white-space: nowrap;
  }
</style>
