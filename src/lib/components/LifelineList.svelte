<script lang="ts">
  // A life on an age scale: own facts and events of close relatives, spaced
  // by the years between them, with a tick at every full decade of life.
  import { t, tx, tn } from '../i18n/index.svelte';
  import { fmtDate, endCauseLabel, year } from '../format';
  import { openLightbox } from '../ui.svelte';
  import Html from './Html.svelte';
  import Img from './Img.svelte';
  import Weather from './Weather.svelte';
  import { app, call, refreshInfo } from '../api.svelte';
  import { confirmDialog, toast } from '../ui.svelte';
  import { errorText } from '../errors';
  import { exactDay, inArchive, refreshPlaces, WEATHER_SOURCE } from '../weather';
  import CloudSun from '@lucide/svelte/icons/cloud-sun';
  import type { Lifeline, LEvent } from '../types';

  let { data, ruler = true, weather = true }: { data: Lifeline; ruler?: boolean; weather?: boolean } = $props();
  // coordinates may have changed on the map since the last time
  refreshPlaces();
  const viewer = $derived(!!app.info?.viewer);
  const fetchOn = $derived(!!app.info?.weather_external);
  let shown = $state(false);
  // events the weather could be shown for (exact day, a place, in the archive)
  const dated = $derived(data.events.filter((e) => e.place && exactDay(e.date) && inArchive(exactDay(e.date)!)).length);

  async function enableWeather() {
    const ok = await confirmDialog(
      t('Show the weather of those days?'),
      t('For events with an exact date (from 1940) at a place with coordinates, the app downloads the historical weather from Open-Meteo.com. Only the coordinates of the place and the date are sent. You can turn it off in the settings.'),
      t('Show weather'),
    );
    if (!ok || !app.settings) return;
    try {
      await call('settings.save', { settings: { ...app.settings, weather_external: true } });
      await refreshInfo();
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }
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

  type Row = { tick: number; gap: number } | { e: LEvent; i: number; gap: number };
  const rows = $derived.by(() => {
    const out: Row[] = [];
    let last: number | null = null;
    const gap = (a: number) => (ruler && last != null ? Math.min(64, Math.max(0, (a - last) * 2.6)) : 0);
    data.events.forEach((e, i) => {
      const a = e.age != null && e.age >= 0 ? e.age : null;
      if (ruler && a != null) {
        for (let d = (last == null ? 0 : Math.floor(last / 10) * 10) + 10; d < a; d += 10) {
          if (last != null && d > last) {
            out.push({ tick: d, gap: gap(d) });
            last = d;
          }
        }
      }
      out.push({ e, i, gap: a != null ? gap(a) : 0 });
      if (a != null) last = a;
    });
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
              {#if weather && e.place && exactDay(e.date)}<Weather place={e.place} date={e.date} fetch={fetchOn && !viewer} onshown={() => (shown = true)} />{/if}
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
  {#if weather && shown}
    <p class="wx-src small muted">{t('Historical weather:')} <a href={WEATHER_SOURCE} target="_blank" rel="noopener noreferrer">Open-Meteo.com</a> (ERA5, CC BY 4.0)</p>
  {:else if weather && !viewer && !fetchOn && dated}
    <button type="button" class="btn ghost sm wx-on no-print" onclick={enableWeather}><CloudSun size={15} /> {t('Show the weather of those days')}</button>
  {/if}
{/if}

<style>
  .wx-src {
    margin: 14px 0 0;
  }
  .wx-on {
    margin-top: 12px;
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
