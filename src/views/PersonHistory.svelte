<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t } from '../lib/i18n/index.svelte';
  import { fmtDateTime, fieldLabel, fmtDate } from '../lib/format';
  import { confirmDialog, toast } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import { stripHtml } from '../lib/text';
  import type { Revision, Profile, Person } from '../lib/types';

  let { id }: { id: number } = $props();
  const prof = useLoad(() => call<Profile>('person.profile', { id }));
  const revs = useLoad(() => call<Revision[]>('person.history', { id }));
  let open = $state<number | null>(null);

  const DATE_FIELDS = ['birth_date', 'death_date'];
  function show(p: Person, f: string): string {
    const v = (p as any)[f];
    if (v == null || v === '') return '—';
    if (DATE_FIELDS.includes(f)) return fmtDate(v);
    if (f === 'biography' || f === 'notes') return stripHtml(v).slice(0, 300) || '—';
    if (Array.isArray(v)) return v.length ? (typeof v[0] === 'string' ? v.join(', ') : String(v.length)) : '—';
    if (typeof v === 'object') return '…';
    return String(v);
  }

  async function restore(r: Revision) {
    if (!(await confirmDialog(t('Restore this version?'), t('The current data will be saved in the history, so you can go back again.'), t('Restore')))) return;
    try {
      await call('person.restore', { revision: r.id });
      toast(t('Version restored.'));
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }
</script>

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('History of changes')}</h1>
      {#if prof.data}<p class="lead"><a href="#/person/{id}" class="name">{prof.data.summary.name}</a></p>{/if}
    </div>
  </div>
  <p class="hint">{t('MyRoots keeps the last 20 versions of each person. Each entry shows the state BEFORE the change.')}</p>
  {#if revs.data && !revs.data.length}
    <div class="panel empty">{t('No saved changes yet.')}</div>
  {:else if revs.data}
    <div class="list">
      {#each revs.data as r (r.id)}
        <div class="row rev">
          <div class="grow">
            <div class="title">{fmtDateTime(r.time)}</div>
            <div class="sub">{t('Changed:')} {r.fields.map(fieldLabel).join(', ') || '—'}</div>
            {#if open === r.id}
              <table class="data diff">
                <tbody>
                  {#each r.fields.filter((f) => f in r.data) as f}
                    <tr><th>{fieldLabel(f)}</th><td>{show(r.data, f)}</td></tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
          <div class="actions">
            <button class="btn sm ghost" onclick={() => (open = open === r.id ? null : r.id)}>{open === r.id ? t('Hide') : t('Show')}</button>
            <button class="btn sm" onclick={() => restore(r)}>{t('Restore')}</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .rev {
    align-items: flex-start !important;
  }
  .title,
  .sub {
    display: block;
    white-space: normal !important;
  }
  .diff {
    margin-top: 10px;
  }
  .diff th {
    width: 32%;
  }
</style>
