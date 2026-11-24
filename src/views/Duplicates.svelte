<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { go } from '../lib/router.svelte';
  import { t } from '../lib/i18n/index.svelte';
  import { fieldLabel, fmtDate, genderLabel } from '../lib/format';
  import { toast, confirmDialog } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import { stripHtml } from '../lib/text';
  import Modal from '../lib/components/Modal.svelte';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import Img from '../lib/components/Img.svelte';
  import Merge from '../lib/icons/GitMerge.svelte';
  import type { MergeSuggestion, Comparison, MergeItem, Person } from '../lib/types';

  const sug = useLoad(() => call<MergeSuggestion[]>('merge.suggestions'));
  let ma = $state<number | null>(null);
  let mb = $state<number | null>(null);

  let cmp = $state<Comparison | null>(null);
  let pair = $state<[number, number] | null>(null);
  let scalar = $state<Record<string, 'a' | 'b'>>({});
  let mother = $state<number | null>(null);
  let father = $state<number | null>(null);
  let keep = $state<'a' | 'b'>('a');
  let avatar = $state<'a' | 'b'>('a');
  let picks = $state<Record<string, string[]>>({});
  let busy = $state(false);

  const LISTS: [keyof Comparison, () => string][] = [
    ['children', () => t('Children')],
    ['partners', () => t('Spouses / partners')],
    ['events', () => t('Events')],
    ['gallery', () => t('Gallery')],
    ['clans', () => t('Clans')],
    ['sources', () => t('Sources')],
  ];

  function val(p: Person, f: string): string {
    const v = (p as any)[f];
    if (!v) return '';
    if (f.endsWith('_date')) return fmtDate(v);
    if (f === 'gender') return genderLabel(v);
    if (f === 'biography' || f === 'notes') return stripHtml(v).slice(0, 160);
    return String(v);
  }

  async function open(a: number, b: number) {
    try {
      const c = await call<Comparison>('merge.compare', { a, b });
      pair = [a, b];
      const sc: Record<string, 'a' | 'b'> = {};
      for (const f of c.fields) sc[f] = !(c.a as any)[f] && (c.b as any)[f] ? 'b' : 'a';
      scalar = sc;
      mother = c.mother_a?.[0] ?? c.mother_b?.[0] ?? null;
      father = c.father_a?.[0] ?? c.father_b?.[0] ?? null;
      keep = 'a';
      avatar = !c.a.photo && c.b.photo ? 'b' : 'a';
      const pk: Record<string, string[]> = {};
      for (const [k] of LISTS) pk[k as string] = (c[k] as MergeItem[]).map((i) => i.value);
      picks = pk;
      cmp = c;
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }

  async function dismiss(s: MergeSuggestion) {
    await call('merge.dismiss', { a: s.a, b: s.b });
    toast(t('Marked as different people.'));
  }

  async function execute() {
    if (!cmp || !pair) return;
    if (!(await confirmDialog(t('Merge people'), t('The two records will become one. The other record will be deleted; its relations, photos and sources move to the merged person according to your choices.'), t('Merge'), true))) return;
    busy = true;
    try {
      const selection = {
        scalar,
        mother,
        father,
        children: picks.children.map(Number),
        partners: picks.partners.map(Number),
        events: picks.events,
        gallery: picks.gallery.map(Number),
        clans: picks.clans,
        sources: picks.sources.map(Number),
        avatar,
      };
      const r = await call<{ id: number }>('merge.execute', { a: pair[0], b: pair[1], keep: keep === 'a' ? pair[0] : pair[1], selection });
      cmp = null;
      toast(t('People merged.'));
      go(`/person/${r.id}`);
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }

  function sideTag(s: string) {
    return s === 'A+B' ? 'A+B' : s;
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Possible duplicates')}</h1>
      <p class="lead">{t('People who may be recorded twice, e.g. after importing several files. Compare them and merge or dismiss.')}</p>
    </div>
  </div>

  <div class="panel manual">
    <span class="lbl">{t('Merge a duplicate person')}</span>
    <div class="mrow">
      <PersonPicker bind:value={ma} exclude={mb ? [mb] : []} placeholder={t('First person…')} />
      <PersonPicker bind:value={mb} exclude={ma ? [ma] : []} placeholder={t('Second person…')} />
      <button class="btn" disabled={!ma || !mb} onclick={() => open(ma!, mb!)}>{t('Compare')}</button>
    </div>
  </div>

  {#if sug.data && !sug.data.length}
    <div class="panel empty">{t('No possible duplicates found.')}</div>
  {:else if sug.data}
    <div class="list">
      {#each sug.data as s (s.a + '-' + s.b)}
        <div class="row pr">
          <span class="grow">
            <a href="#/person/{s.a}" class="name">{s.name_a}</a> <span class="muted small">{s.years_a}</span>
            <span class="faint"> ⇄ </span>
            <a href="#/person/{s.b}" class="name">{s.name_b}</a> <span class="muted small">{s.years_b}</span>
          </span>
          <span class="tag" class:heirloom={s.score >= 80}>{s.score}%</span>
          <button class="btn sm" onclick={() => open(s.a, s.b)}><Merge size={15} /> {t('Compare')}</button>
          <button class="btn sm ghost" onclick={() => dismiss(s)}>{t('Not a duplicate')}</button>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if cmp}
  <Modal title={t('Merge people')} onclose={() => (cmp = null)} wide>
    <div class="keep">
      <span class="lbl">{t('Keep the record of')}</span>
      <label class="check"><input type="radio" bind:group={keep} value="a" /> A: {cmp.a.first_name} {cmp.a.last_name} (#{cmp.a.id})</label>
      <label class="check"><input type="radio" bind:group={keep} value="b" /> B: {cmp.b.first_name} {cmp.b.last_name} (#{cmp.b.id})</label>
    </div>
    <table class="data cmp">
      <thead><tr><th></th><th>A</th><th>B</th></tr></thead>
      <tbody>
        <tr>
          <th>{t('Profile photo')}</th>
          <td><label class="pick"><input type="radio" bind:group={avatar} value="a" disabled={!cmp.a.photo} />{#if cmp.a.photo}<Img id={cmp.a.photo} class="dup-thumb" />{:else}—{/if}</label></td>
          <td><label class="pick"><input type="radio" bind:group={avatar} value="b" disabled={!cmp.b.photo} />{#if cmp.b.photo}<Img id={cmp.b.photo} class="dup-thumb" />{:else}—{/if}</label></td>
        </tr>
        {#each cmp.fields as f (f)}
          {#if val(cmp.a, f) || val(cmp.b, f)}
            <tr class:diff={val(cmp.a, f) !== val(cmp.b, f)}>
              <th>{fieldLabel(f)}</th>
              <td><label class="pick"><input type="radio" bind:group={scalar[f]} value="a" /> <span>{val(cmp.a, f) || '—'}</span></label></td>
              <td><label class="pick"><input type="radio" bind:group={scalar[f]} value="b" /> <span>{val(cmp.b, f) || '—'}</span></label></td>
            </tr>
          {/if}
        {/each}
        {#if cmp.father_a || cmp.father_b}
          <tr>
            <th>{t('Father')}</th>
            <td><label class="pick"><input type="radio" bind:group={father} value={cmp.father_a?.[0] ?? null} /> {cmp.father_a?.[1] || '—'}</label></td>
            <td><label class="pick"><input type="radio" bind:group={father} value={cmp.father_b?.[0] ?? null} /> {cmp.father_b?.[1] || '—'}</label></td>
          </tr>
        {/if}
        {#if cmp.mother_a || cmp.mother_b}
          <tr>
            <th>{t('Mother')}</th>
            <td><label class="pick"><input type="radio" bind:group={mother} value={cmp.mother_a?.[0] ?? null} /> {cmp.mother_a?.[1] || '—'}</label></td>
            <td><label class="pick"><input type="radio" bind:group={mother} value={cmp.mother_b?.[0] ?? null} /> {cmp.mother_b?.[1] || '—'}</label></td>
          </tr>
        {/if}
      </tbody>
    </table>
    {#each LISTS as [k, label] (k)}
      {@const items = cmp[k] as MergeItem[]}
      {#if items.length}
        <h3 class="lh">{label()}</h3>
        <div class="items">
          {#each items as it (it.value)}
            <label class="check"><input type="checkbox" bind:group={picks[k as string]} value={it.value} /> {#if it.media}<Img id={it.media} class="dup-thumb" />{/if} {it.label || '—'} <span class="tag">{sideTag(it.in)}</span></label>
          {/each}
        </div>
      {/if}
    {/each}
    {#snippet footer()}
      <button class="btn" onclick={() => (cmp = null)}>{t('Cancel')}</button>
      <button class="btn primary" onclick={execute} disabled={busy}><Merge size={16} /> {t('Merge')}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .manual {
    margin-bottom: 18px;
  }
  .lbl {
    display: block;
    font-size: 0.86rem;
    color: var(--muted);
    font-weight: 500;
    margin-bottom: 8px;
  }
  .mrow {
    display: grid;
    grid-template-columns: 1fr 1fr auto;
    gap: 8px;
  }
  .pr .grow {
    white-space: normal;
  }
  .keep {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
    margin-bottom: 14px;
    align-items: center;
  }
  .keep .lbl {
    margin: 0;
  }
  .cmp th {
    width: 22%;
  }
  .cmp tr.diff td {
    background: var(--heirloom-soft);
  }
  .pick {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    cursor: pointer;
  }
  .pick input {
    margin-top: 3px;
  }
  .lh {
    margin-top: 16px;
    font-size: 0.95rem;
  }
  .items {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .items .check {
    align-items: center;
  }
  :global(.dup-thumb) {
    width: 36px;
    height: 36px;
    object-fit: cover;
    border-radius: 10px;
  }
  @media (max-width: 640px) {
    .mrow {
      grid-template-columns: 1fr;
    }
  }
</style>
