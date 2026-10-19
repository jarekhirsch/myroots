<script lang="ts">
  // The weather on the day of an event (icon and temperatures; details in the tooltip).
  import { weatherFor, weatherKind, weatherLabel } from '../weather';
  import { fmtNumber } from '../format';
  import { t } from '../i18n/index.svelte';
  import Sun from '@lucide/svelte/icons/sun';
  import CloudSun from '@lucide/svelte/icons/cloud-sun';
  import Cloud from '@lucide/svelte/icons/cloud';
  import CloudFog from '@lucide/svelte/icons/cloud-fog';
  import CloudDrizzle from '@lucide/svelte/icons/cloud-drizzle';
  import CloudRain from '@lucide/svelte/icons/cloud-rain';
  import CloudSnow from '@lucide/svelte/icons/cloud-snow';
  import CloudLightning from '@lucide/svelte/icons/cloud-lightning';
  import type { DayWeather } from '../types';

  let { place, date, fetch = false, onshown }: { place: string; date: string; fetch?: boolean; onshown?: () => void } = $props();
  let w = $state<DayWeather | null>(null);

  $effect(() => {
    const pl = place;
    const d = date;
    const f = fetch;
    w = null;
    weatherFor(pl, d, f).then((r) => {
      if (pl !== place || d !== date) return;
      w = r;
      if (r) onshown?.();
    });
  });

  const ICONS = { clear: Sun, fair: CloudSun, cloudy: Cloud, fog: CloudFog, drizzle: CloudDrizzle, rain: CloudRain, snow: CloudSnow, storm: CloudLightning };
  const Icon = $derived(w ? ICONS[weatherKind(w.code)] : Cloud);
  const deg = (v: number | null) => (v == null ? '?' : `${Math.round(v)}°`);
  const tip = $derived.by(() => {
    if (!w) return '';
    const parts = [weatherLabel(w.code)];
    if (w.tmin != null && w.tmax != null) parts.push(t('from %1$s to %2$s °C', fmtNumber(w.tmin), fmtNumber(w.tmax)));
    if (w.snow != null && w.snow >= 0.5) parts.push(t('snow %s cm', fmtNumber(w.snow)));
    else if (w.rain != null && w.rain >= 0.2) parts.push(t('precipitation %s mm', fmtNumber(w.rain)));
    return t('Weather that day: %s', parts.join(', '));
  });
</script>

{#if w}
  <span class="wx" title={tip} role="img" aria-label={tip}>
    <Icon size={15} strokeWidth={1.8} />
    <span class="tt">{deg(w.tmax)}&nbsp;<span class="lo">/&nbsp;{deg(w.tmin)}</span></span>
  </span>
{/if}

<style>
  .wx {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 8px 1px 6px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--ink) 6%, transparent);
    color: var(--ink);
    font-size: 0.8rem;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    vertical-align: 1px;
    cursor: help;
  }
  .lo {
    color: var(--muted);
  }
</style>
