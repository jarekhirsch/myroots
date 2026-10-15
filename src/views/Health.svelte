<script lang="ts">
  import { call, summaries } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { fmtNumber } from '../lib/format';
  import Avatar from '../lib/components/Avatar.svelte';
  import Check from '@lucide/svelte/icons/circle-check';
  import Alert from '@lucide/svelte/icons/circle-alert';
  import Warn from '@lucide/svelte/icons/triangle-alert';
  import type { Issue, Completeness, Summary } from '../lib/types';

  let tab = $state<'issues' | 'complete'>('issues');
  const issues = useLoad(() => call<Issue[]>('health.issues'));
  const comp = useLoad(() => call<Completeness>('health.completeness'));
  const people = useLoad(() => summaries());
  const byId = $derived(new Map((people.data || []).map((p) => [p.id, p])));
  let crit = $state<string | null>(null);

  const TYPE: Record<string, () => string> = {
    death_before_birth: () => t('Death before birth'),
    implausible_age: () => t('Implausible age'),
    parent_younger: () => t('Parent younger than child'),
    parent_too_young: () => t('Parent too young'),
    parent_too_old: () => t('Parent too old'),
    parent_dead_before_birth: () => t('Parent died before the child was born'),
    marriage_before_birth: () => t('Marriage before birth'),
    marriage_too_young: () => t('Marriage at too young an age'),
    marriage_after_death: () => t('Marriage after death'),
    marriage_end_before_start: () => t('Marriage ended before it began'),
    gender_role_conflict_father: () => t('Gender and parent role conflict'),
    gender_role_conflict_mother: () => t('Gender and parent role conflict'),
    ancestor_cycle: () => t('Ancestor loop'),
  };

  function role(r: string) {
    return r === 'mother' ? t('Mother') : t('Father');
  }

  function message(i: Issue): string {
    const a = i.args;
    switch (i.type) {
      case 'death_before_birth':
        return t('The date of death (%1$d) is earlier than the date of birth (%2$d).', a[0], a[1]);
      case 'implausible_age':
        return t('A lifespan of %d years exceeds a plausible threshold — check the dates.', a[0]);
      case 'parent_younger':
        return t('%1$s (%2$s) was born in the same year as the child or later.', role(a[0]), a[1]);
      case 'parent_too_young':
        return t('%1$s (%2$s) was only %3$d years old when the child was born.', role(a[0]), a[1], a[2]);
      case 'parent_too_old':
        return t('%1$s (%2$s) was as old as %3$d when the child was born — check the dates.', role(a[0]), a[1], a[2]);
      case 'parent_dead_before_birth':
        return t('%1$s (%2$s) died in %3$d, before the child was born (%4$d).', role(a[0]), a[1], a[2], a[3]);
      case 'gender_role_conflict_father':
        return t('Person “%s” is set as the father but is female.', a[0]);
      case 'gender_role_conflict_mother':
        return t('Person “%s” is set as the mother but is male.', a[0]);
      case 'marriage_before_birth':
        return t('Marriage in %1$d (to %2$s) before the person was born (%3$d).', a[0], a[1], a[2]);
      case 'marriage_too_young':
        return t('Age at marriage: %1$d years (to %2$s) — check the dates.', a[0], a[1]);
      case 'marriage_after_death':
        return t("Marriage in %1$d (to %2$s) after the person's death (%3$d).", a[0], a[1], a[2]);
      case 'marriage_end_before_start':
        return t('Marriage ended (%1$d) before it began (%2$d) — with %3$s.', a[0], a[1], a[2]);
      case 'ancestor_cycle':
        return t('Ancestor loop: %s — the person is their own ancestor. The tree will not render correctly.', (a[0] || []).join(' → '));
      default:
        return i.type;
    }
  }

  const CRIT: Record<string, () => string> = {
    birth_date: () => t('Date of birth'),
    birth_place: () => t('Place of birth'),
    gender: () => t('Gender'),
    death_date: () => t('Date of death (deceased people)'),
    death_place: () => t('Place of death (deceased people)'),
    biography: () => t('Biography'),
    photo: () => t('Profile photo'),
    source: () => t('Linked source'),
    parent: () => t('At least one parent'),
  };

  const errors = $derived((issues.data || []).filter((i) => i.severity === 'error'));
  const warns = $derived((issues.data || []).filter((i) => i.severity !== 'error'));
  const missingList = $derived(crit && comp.data ? (comp.data.missing[crit] || []).map((id) => byId.get(id)).filter((x): x is Summary => !!x) : []);
  const weakest = $derived(
    comp.data
      ? Object.entries(comp.data.scores)
          .sort((a, b) => a[1] - b[1])
          .slice(0, 12)
          .map(([id, s]) => [byId.get(Number(id)), s] as [Summary | undefined, number])
          .filter((x) => x[0])
      : [],
  );
</script>

{#snippet group(list: Issue[], sev: string)}
  {#if list.length}
    <section class="block">
      <h2>{sev === 'error' ? t('Errors') : t('Warnings')} <span class="muted small">({list.length})</span></h2>
      <div class="list">
        {#each list as i, k (k)}
          {@const p = byId.get(i.person)}
          <div class="row issue">
            <span class="ic {sev}">{#if sev === 'error'}<Alert size={18} />{:else}<Warn size={18} />{/if}</span>
            <span class="grow">
              <span class="ty">{TYPE[i.type]?.() || i.type}</span>
              <span class="msg">{message(i)}</span>
            </span>
            {#if p}<a class="btn sm" href="#/person/{p.id}/edit">{p.name}</a>{/if}
          </div>
        {/each}
      </div>
    </section>
  {/if}
{/snippet}

<div class="page">
  <div class="page-head">
    <div>
      <h1>{t('Data consistency and completeness')}</h1>
      <p class="lead">{t('MyRoots checks dates and relations for mistakes and shows what is still missing.')}</p>
    </div>
  </div>
  <div class="tabs" role="tablist">
    <button role="tab" aria-selected={tab === 'issues'} onclick={() => (tab = 'issues')}>{t('Data consistency')} {#if issues.data?.length}<span class="tag danger">{issues.data.length}</span>{/if}</button>
    <button role="tab" aria-selected={tab === 'complete'} onclick={() => (tab = 'complete')}>{t('Completeness')}</button>
  </div>

  {#if tab === 'issues'}
    {#if issues.data && !issues.data.length}
      <div class="panel ok"><Check size={22} /> {t('No problems found. The data looks consistent.')}</div>
    {/if}
    {@render group(errors, 'error')}
    {@render group(warns, 'warn')}
  {:else if comp.data}
    <div class="overall panel">
      <div class="big"><b>{comp.data.overall}%</b><span>{t('Overall completeness')}</span></div>
      <p class="muted small">{tn('Analyzed %d person.', 'Analyzed %d people.', comp.data.checked)}</p>
    </div>
    <div class="grid-2">
      <section class="panel">
        <h2>{t('Missing data')}</h2>
        <table class="data crit">
          <tbody>
            {#each comp.data.criteria as c (c)}
              {@const miss = comp.data.missing[c]?.length || 0}
              {@const app_ = comp.data.applicable[c] || 0}
              {@const pct = app_ ? Math.round(((app_ - miss) / app_) * 100) : 100}
              <tr class:on={crit === c}>
                <th>{CRIT[c]?.() || c}</th>
                <td class="meter"><span style="width:{pct}%"></span></td>
                <td class="n">{pct}%</td>
                <td>{#if miss}<button class="btn ghost sm" onclick={() => (crit = crit === c ? null : c)}>{tn('%d person without data', '%d people without data', miss)}</button>{/if}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        {#if crit}
          <h3 class="ml">{CRIT[crit]?.()}</h3>
          <div class="chips">
            {#each missingList as p (p.id)}<a class="tag" href="#/person/{p.id}/edit">{p.name}</a>{/each}
          </div>
        {/if}
      </section>
      <section class="panel">
        <h2>{t('Least complete profiles')}</h2>
        <div class="weak">
          {#each weakest as [p, s] (p!.id)}
            <a class="row" href="#/person/{p!.id}/edit">
              <Avatar photo={p!.photo} name={p!.name} gender={p!.gender} size={30} />
              <span class="grow name">{p!.name}</span>
              <span class="tag" class:warn={s < 50}>{fmtNumber(s, 0)}%</span>
            </a>
          {/each}
        </div>
      </section>
    </div>
  {/if}
</div>

<style>
  .ok {
    display: flex;
    gap: 10px;
    align-items: center;
    color: var(--ok);
  }
  .issue {
    align-items: flex-start !important;
  }
  .ic {
    margin-top: 2px;
  }
  .ic.error {
    color: var(--danger);
  }
  .ic.warn {
    color: var(--warn);
  }
  .ty,
  .msg {
    display: block;
  }
  .ty {
    font-weight: 620;
  }
  .msg {
    font-size: 0.9rem;
    color: var(--muted);
  }
  .overall {
    display: flex;
    align-items: center;
    gap: 22px;
    margin-bottom: 18px;
  }
  .big {
    display: flex;
    flex-direction: column;
  }
  .big b {
    font-size: 2.4rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
  }
  .big span {
    color: var(--muted);
    font-size: 0.9rem;
  }
  .crit th {
    color: var(--text);
    font-weight: 450;
    font-size: 0.9rem;
  }
  .crit tr.on th {
    font-weight: 600;
  }
  .meter {
    width: 30%;
    min-width: 80px;
  }
  .meter span {
    display: block;
    height: 8px;
    border-radius: 0 4px 4px 0;
    background: var(--ink);
  }
  .n {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .ml {
    margin-top: 16px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chips a {
    text-decoration: none;
  }
  .weak .row {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 6px 4px;
    border-radius: 12px;
    color: inherit;
    text-decoration: none;
  }
  .weak .row:hover {
    background: var(--ink-soft);
  }
</style>
