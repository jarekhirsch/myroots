<script lang="ts">
  // Qualified date: exact / about / before / after / between (stored as 1920, ~1920, <1920, >1920, 1900..1910).
  import { t } from '../i18n/index.svelte';
  import { fmtDate, parseDate } from '../format';
  let { value = $bindable(''), id = undefined }: { value?: string; id?: string } = $props();

  let q = $state<'exact' | 'abt' | 'bef' | 'aft' | 'bet'>('exact');
  let a = $state('');
  let b = $state('');
  let last = '';

  // Accepts 1920, 1920-05, 1920-05-15, 15.05.1920, 15/05/1920, 05.1920.
  function toIso(s: string): string {
    s = s.trim();
    if (!s) return '';
    let m;
    if ((m = s.match(/^(\d{4})(?:-(\d{1,2}))?(?:-(\d{1,2}))?$/))) {
      return m[1] + (m[2] ? '-' + m[2].padStart(2, '0') : '') + (m[3] ? '-' + m[3].padStart(2, '0') : '');
    }
    if ((m = s.match(/^(\d{1,2})[./\s](\d{1,2})[./\s](\d{4})$/))) return `${m[3]}-${m[2].padStart(2, '0')}-${m[1].padStart(2, '0')}`;
    if ((m = s.match(/^(\d{1,2})[./\s](\d{4})$/))) return `${m[2]}-${m[1].padStart(2, '0')}`;
    return '#';
  }

  $effect(() => {
    if (value === last) return;
    const p = parseDate(value);
    q = p.q;
    a = p.d1;
    b = p.d2;
    last = value;
  });

  const ia = $derived(toIso(a));
  const ib = $derived(toIso(b));
  const invalid = $derived(ia === '#' || (q === 'bet' && ib === '#'));

  function commit() {
    if (invalid) return;
    let v = '';
    if (q === 'bet') v = ia && ib ? `${ia}..${ib}` : ia || ib;
    else if (ia) v = ({ exact: '', abt: '~', bef: '<', aft: '>' } as const)[q] + ia;
    last = v;
    value = v;
  }
</script>

<div class="di">
  <select bind:value={q} onchange={commit} aria-label={t('Date accuracy')}>
    <option value="exact">{t('exactly')}</option>
    <option value="abt">{t('approx.')}</option>
    <option value="bef">{t('before')}</option>
    <option value="aft">{t('after')}</option>
    <option value="bet">{t('between')}</option>
  </select>
  <input {id} type="text" inputmode="numeric" placeholder={t('YYYY-MM-DD')} bind:value={a} oninput={commit} class:bad={ia === '#'} />
  {#if q === 'bet'}
    <span class="and">{t('and')}</span>
    <input type="text" inputmode="numeric" placeholder={t('YYYY-MM-DD')} bind:value={b} oninput={commit} class:bad={ib === '#'} />
  {/if}
</div>
{#if invalid}
  <span class="hint bad-t">{t('Use a year (1920), month (1920-05) or full date (1920-05-15 or 15.05.1920).')}</span>
{:else if value && /[~<>.]/.test(value)}
  <span class="hint">{fmtDate(value)}</span>
{/if}

<style>
  .di {
    display: flex;
    gap: 6px;
    align-items: center;
    min-width: 0;
  }
  .di select {
    width: auto;
    flex: 0 0 auto;
    max-width: 42%;
  }
  .di input {
    flex: 1;
  }
  .and {
    color: var(--muted);
    font-size: 0.85rem;
  }
  .bad {
    border-color: var(--danger) !important;
  }
  .bad-t {
    color: var(--danger);
  }
</style>
