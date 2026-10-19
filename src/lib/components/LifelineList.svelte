<script lang="ts">
  // A life on an age scale: own facts and events of close relatives, spaced
  // by the years between them, with a tick at every full decade of life.
  import { t, tx, tn } from '../i18n/index.svelte';
  import { fmtDate, endCauseLabel, year } from '../format';
  import { openLightbox } from '../ui.svelte';
  import Html from './Html.svelte';
  import Img from './Img.svelte';
  import Weather from './Weather.svelte';
  import { app } from '../api.svelte';
  import { exactDay, refreshPlaces } from '../weather';
  import { POLAND, historyShown, setHistoryShown, type HistoryEvent } from '../history';
  import Landmark from '@lucide/svelte/icons/landmark';
  import type { Lifeline, LEvent } from '../types';

  let { data, ruler = true, weather = true, history = true }: { data: Lifeline; ruler?: boolean; weather?: boolean; history?: boolean } = $props();
  let histOn = $state(historyShown());
  // coordinates may have changed on the map since the last time
  refreshPlaces();
  const viewer = $derived(!!app.info?.viewer);
  const fetchOn = $derived(!!app.info?.weather_external);

  let open = $state<Record<number, boolean>>({});

  function title(e: LEvent): string {
    switch (e.kind) {
      case 'birth':
        return t('Birth');
      case 'death':
        return t('Death');
      case 'presumed_death':
        return t('Presumed death');
      case 'event':
        return e.name || t('Event');
      case 'marriage':
        return t('Marriage to %s', e.related_name || '?');
      case 'marriage_end':
        return t('End of marriage to %s', e.related_name || '?');
      case 'child_birth':
        return t('Birth of a child: %s', e.related_name);
      case 'grandchild_birth':
        return t('Birth of a grandchild: %s', e.related_name);
      case 'sibling_birth':
        return t('Birth of a sibling: %s', e.related_name);
      case 'sibling_death':
        return t('Death of a sibling: %s', e.related_name);
      case 'parent_death':
        return t('Death of %1$s: %2$s', e.role === 'mother' ? tx('genitive, as in “Death of %s”', 'mother') : tx('genitive, as in “Death of %s”', 'father'), e.related_name);
      case 'grandparent_death':
        return t('Death of a grandparent: %s', e.related_name);
      default:
        return e.name || e.kind;
    }
  }

  function ageText(e: LEvent): string {
    if (e.age == null || e.kind === 'birth') return '';
    if (e.age < 0) return t('This event took place %1$d years before %2$s was born', Math.abs(e.age), data.name);
    return data.gender === 'F' ? tx('female', '%1$s was %2$d years old at the time', data.name, e.age) : tx('male', '%1$s was %2$d years old at the time', data.name, e.age);
  }

  const own = new Set(['birth', 'death', 'presumed_death', 'event', 'marriage', 'marriage_end']);

  type Row = { tick: number; gap: number } | { e: LEvent; i: number; gap: number } | { h: HistoryEvent; age: number; gap: number };

  // "YYYY-MM-DD" sort key of a (qualified) date; null when it has no year
  function keyOf(v: string): string | null {
    const m = (v || '').match(/(\d{4})(?:-(\d{2}))?(?:-(\d{2}))?/);
    return m ? `${m[1]}-${m[2] || '00'}-${m[3] || '00'}` : null;
  }
  function yearsBetween(a: string, b: string): number {
    const [ay, am, ad] = a.split('-').map(Number);
    const [by, bm, bd] = b.split('-').map(Number);
    let y = by - ay;
    if (am && ad && bm && bd && (bm < am || (bm === am && bd < ad))) y--;
    return y;
  }
  // events of the history of Poland within this life
  const background = $derived.by(() => {
    if (!history) return [] as { h: HistoryEvent; key: string; age: number }[];
    const birth = data.events.find((e) => e.kind === 'birth');
    const from = birth ? keyOf(birth.date) : null;
    if (!from) return [];
    const death = data.events.find((e) => e.kind === 'death' && keyOf(e.date));
    let to = death ? keyOf(death.date) : null;
    if (!to && data.living) to = new Date().toISOString().slice(0, 10);
    if (!to) to = data.events.filter((e) => own.has(e.kind)).map((e) => keyOf(e.date)).filter((k): k is string => !!k).sort().pop() || null;
    if (!to || to <= from) return [];
    return POLAND.map((h) => ({ h, key: keyOf(h.date)! })).filter((x) => x.key > from && x.key <= to!).map((x) => ({ ...x, age: yearsBetween(from, x.key) }));
  });

  const rows = $derived.by(() => {
    const out: Row[] = [];
    let last: number | null = null;
    const gap = (a: number) => (ruler && last != null ? Math.min(64, Math.max(0, (a - last) * 2.6)) : 0);
    const ticks = (a: number) => {
      if (!ruler) return;
      for (let d = (last == null ? 0 : Math.floor(last / 10) * 10) + 10; d < a; d += 10) {
        if (last != null && d > last) {
          out.push({ tick: d, gap: gap(d) });
          last = d;
        }
      }
    };
    const hist = histOn ? [...background] : [];
    const flush = (until: string | null) => {
      while (hist.length && (until == null || hist[0].key < until)) {
        const x = hist.shift()!;
        ticks(x.age);
        out.push({ h: x.h, age: x.age, gap: gap(x.age) });
        last = x.age;
      }
    };
    data.events.forEach((e, i) => {
      const k = keyOf(e.date);
      if (k) flush(k);
      const a = e.age != null && e.age >= 0 ? e.age : null;
      if (a != null) ticks(a);
      out.push({ e, i, gap: a != null ? gap(a) : 0 });
      if (a != null) last = a;
    });
    flush(null);
    return out;
  });
  function left(e: LEvent): string {
    if (e.kind === 'birth' || e.age == null || e.age < 0) return String(year(e.date) || fmtDate(e.date));
    return tn('%d year', '%d years', e.age);
  }
</script>

{#if !data.events.length}
  <p class="muted">{t('No events.')}</p>
{:else}
  <ol class="ll" class:ruler>
    {#each rows as r, k (k)}
      {#if 'tick' in r}
        <li class="tick" style="margin-top: {r.gap}px" aria-hidden="true">
          <span class="when">{tn('%d year', '%d years', r.tick)}</span>
          <span class="mark"></span>
        </li>
      {:else if 'h' in r}
        <li class="hist" style="margin-top: {r.gap}px">
          <span class="dot" aria-hidden="true"></span>
          <div class="when"><span class="d">{tn('%d year', '%d years', r.age)}</span></div>
          <div class="what">
            <div class="ttl">{r.h.title()}</div>
            <div class="pl small muted">{fmtDate(r.h.date)}</div>
          </div>
        </li>
      {:else}
        {@const e = r.e}
        {@const i = r.i}
        <li class="ev-{e.kind}" class:own={own.has(e.kind)} style="margin-top: {r.gap}px">
          <span class="dot" aria-hidden="true"></span>
          <div class="when">
            <span class="d" title={ageText(e)}>{left(e)}</span>
          </div>
          <div class="what">
            <div class="ttl">
              {#if e.related && e.kind !== 'event'}
                <a href="#/person/{e.related}">{title(e)}</a>
              {:else}
                {title(e)}
              {/if}
            </div>
            <div class="pl small muted">
              {[fmtDate(e.date), e.place].filter(Boolean).join(', ')}
              {#if weather && e.place && exactDay(e.date)}<Weather place={e.place} date={e.date} fetch={fetchOn && !viewer} />{/if}
            </div>
            {#if e.kind === 'marriage_end' && e.role}<div class="small muted">{endCauseLabel(e.role)}</div>{/if}
            {#if e.image}
              <button class="thumb" onclick={() => openLightbox([{ media: e.image!, caption: title(e) }])} aria-label={t('Enlarge')}>
                <Img id={e.image} alt="" />
              </button>
            {/if}
            {#if e.description}
              {#if open[i]}
                <Html html={e.description} />
              {:else}
                <button class="btn ghost sm more" onclick={() => (open[i] = true)}>{t('Show description')}</button>
              {/if}
            {/if}
          </div>
        </li>
      {/if}
    {/each}
  </ol>
  {#if background.length}
    <button type="button" class="btn ghost sm hist-on no-print" onclick={() => setHistoryShown((histOn = !histOn))} aria-pressed={histOn}>
      <Landmark size={15} /> {histOn ? t('Hide the history of Poland') : t('Show the history of Poland in the background')}
    </button>
  {/if}
{/if}

<style>
  .hist .ttl {
    color: var(--muted);
    font-style: italic;
  }
  .hist .dot {
    width: 9px !important;
    height: 9px !important;
    margin-top: 7px !important;
    border-radius: 2px;
    transform: rotate(45deg);
    background: var(--surface) !important;
    box-shadow: 0 0 0 1.5px var(--muted), 0 0 0 4px var(--surface) !important;
  }
  .hist .when .d {
    color: var(--faint);
  }
  .hist-on {
    margin-top: 12px;
    margin-inline-end: 8px;
  }
  .pl :global(.wx) {
    margin-inline-start: 6px;
  }
  .ll {
    list-style: none;
    margin: 0;
    padding: 0;
    position: relative;
  }
  .ll::before {
    content: '';
    position: absolute;
    top: 10px;
    bottom: 10px;
    inset-inline-start: 99px;
    width: 3px;
    border-radius: 3px;
    background: var(--heirloom-soft);
  }
  li {
    display: grid;
    grid-template-columns: 82px 36px minmax(0, 1fr);
    align-items: start;
    padding: 6px 0;
    position: relative;
  }
  .dot {
    grid-column: 2;
    grid-row: 1;
    justify-self: center;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--ev, var(--faint));
    box-shadow: 0 0 0 3px var(--surface);
    margin-top: 4px;
    position: relative;
    z-index: 1;
  }
  li:not(.own) .dot {
    width: 9px;
    height: 9px;
    margin-top: 7px;
    background: var(--line-strong);
  }
  .tick {
    padding: 0;
    height: 14px;
  }
  .tick .when {
    font-size: 0.75rem;
    color: var(--faint);
    padding: 0;
    line-height: 14px;
  }
  .mark {
    grid-column: 2;
    grid-row: 1;
    justify-self: center;
    align-self: center;
    width: 12px;
    height: 2px;
    border-radius: 2px;
    background: var(--heirloom);
    opacity: 0.55;
  }
  .when {
    grid-column: 1;
    grid-row: 1;
    text-align: end;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text);
    padding-top: 1px;
  }
  li:not(.own) .when {
    font-weight: 450;
    color: var(--muted);
  }
  .what {
    grid-column: 3;
    grid-row: 1;
    min-width: 0;
  }
  .ttl {
    font-weight: 600;
  }
  li:not(.own) .ttl {
    font-weight: 450;
    color: var(--muted);
  }
  .ttl a {
    color: inherit;
  }
  .thumb {
    display: block;
    margin-top: 6px;
    padding: 0;
    border: 0;
    background: none;
    cursor: zoom-in;
  }
  .thumb :global(img),
  .thumb :global(.ph) {
    width: 120px;
    height: 84px;
    object-fit: cover;
    border-radius: 10px;
    display: block;
  }
  .more {
    margin-top: 2px;
    padding-inline: 0;
    color: var(--ink-2);
  }
  @media (max-width: 560px) {
    .ll::before {
      inset-inline-start: 81px;
    }
    li {
      grid-template-columns: 64px 36px minmax(0, 1fr);
    }
  }
</style>
