<script lang="ts">
  // Horizontal bars (ranked lists) or columns (distributions). One hue, values as text.
  import { fmtNumber } from '../format';
  let {
    data,
    columns = false,
    empty = '',
    unit = (n: number) => fmtNumber(n, 0),
    hrefFor,
  }: { data: [string, number][]; columns?: boolean; empty?: string; unit?: (n: number) => string; hrefFor?: (label: string) => string } = $props();
  const max = $derived(Math.max(1, ...data.map((d) => d[1])));
</script>

{#if !data.length || data.every((d) => !d[1])}
  <p class="muted small">{empty}</p>
{:else if columns}
  <div class="cols">
    {#each data as [label, v], i (label)}
      <div class="col" title="{label}: {unit(v)}">
        <span class="val" class:show={data.length <= 14}>{v ? unit(v) : ''}</span>
        <span class="track"><span class="bar" style="height:{(v / max) * 100}%"></span></span>
        <span class="lab">{label}</span>
      </div>
    {/each}
  </div>
{:else}
  <table class="bars">
    <tbody>
      {#each data as [label, v] (label)}
        <tr>
          <th scope="row">{#if hrefFor}<a href={hrefFor(label)}>{label}</a>{:else}{label}{/if}</th>
          <td class="tr"><span class="bar" style="width:{Math.max(2, (v / max) * 100)}%"></span></td>
          <td class="n">{unit(v)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  .bars {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }
  .bars th {
    text-align: start;
    font-weight: 450;
    padding: 4px 10px 4px 0;
    white-space: nowrap;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bars th a {
    color: inherit;
    text-decoration: none;
  }
  .bars th a:hover {
    text-decoration: underline;
  }
  .tr {
    width: 100%;
    padding: 4px 0;
  }
  .bars .bar {
    display: block;
    height: 12px;
    background: var(--ink);
    border-radius: 0 4px 4px 0;
  }
  :global([dir='rtl']) .bars .bar {
    border-radius: 4px 0 0 4px;
  }
  .n {
    padding-inline-start: 10px;
    text-align: end;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .cols {
    display: flex;
    align-items: stretch;
    gap: 2px;
    height: 180px;
  }
  .col {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .track {
    flex: 1;
    width: 100%;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    border-bottom: 1px solid var(--line-strong);
  }
  .cols .bar {
    display: block;
    width: min(70%, 34px);
    background: var(--ink);
    border-radius: 4px 4px 0 0;
    min-height: 0;
  }
  .col:hover .bar {
    background: var(--ink-2);
  }
  .val {
    font-size: 0.72rem;
    color: var(--muted);
    visibility: hidden;
    font-variant-numeric: tabular-nums;
    height: 1.1em;
  }
  .val.show,
  .col:hover .val {
    visibility: visible;
  }
  .lab {
    font-size: 0.7rem;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
</style>
