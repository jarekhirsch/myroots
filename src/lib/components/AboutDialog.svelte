<script lang="ts">
  // About MyRoots: author, licence, the components it is built with.
  import Modal from './Modal.svelte';
  import { app } from '../api.svelte';
  import { t } from '../i18n/index.svelte';
  import { AUTHOR, YEAR, LICENSE_URL, SOURCE_URL, THIRD_PARTY, authorUrl } from '../credits';

  let { onclose }: { onclose: () => void } = $props();
  const viewer = $derived(!!app.info?.viewer);
  const medium = $derived(viewer ? 'family-website' : 'app');
</script>

<Modal title={t('About MyRoots')} {onclose}>
  <div class="about">
    <div class="id">
      <img src="./icon.svg" alt="" width="56" height="56" />
      <div>
        <div class="nm">MyRoots <span class="v">{app.info?.version || ''}</span></div>
        <div class="by">© {YEAR} <a href={authorUrl(medium, 'about')} target="_blank" rel="noopener">{AUTHOR}</a> · <a href={authorUrl(medium, 'about-domain')} target="_blank" rel="noopener">hirsch.fi</a></div>
      </div>
    </div>

    <h3>{t('Licence')}</h3>
    <p>
      {t('MyRoots is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License, version 3 or (at your option) any later version, with the additional terms below.')}
      {t('It is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.')}
    </p>
    <p><a href={LICENSE_URL} target="_blank" rel="noopener">GNU General Public License v3.0</a> · <a href={SOURCE_URL} target="_blank" rel="noopener">{t('Source code')}</a></p>

    <h3>{t('Additional terms (section 7)')}</h3>
    <ul class="terms">
      <li>{t('The credit “© Jarek Hirsch” with the link to hirsch.fi must stay in this window in every copy and every modified version.')}</li>
      <li>{t('A modified version must be clearly marked as different from the original MyRoots.')}</li>
      <li>{t('The licence gives no rights to the name and logo of MyRoots: modified versions use their own.')}</li>
    </ul>

    <h3>{t('Built with')}</h3>
    <ul class="deps">
      {#each THIRD_PARTY as d (d.name)}
        <li><a href={d.url} target="_blank" rel="noopener">{d.name}</a><span>{d.license}</span></li>
      {/each}
    </ul>
    <p class="small muted">{t('Your family data is kept on your device, not on a MyRoots server.')}</p>
  </div>
</Modal>

<style>
  .about {
    max-width: 560px;
  }
  .id {
    display: flex;
    gap: 14px;
    align-items: center;
    margin-bottom: 8px;
  }
  .nm {
    font-size: 1.3rem;
    font-weight: 650;
  }
  .v {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .by {
    color: var(--muted);
  }
  h3 {
    font-size: 0.95rem;
    margin: 18px 0 6px;
  }
  p {
    margin: 0 0 8px;
    line-height: 1.5;
  }
  .terms {
    margin: 0 0 8px;
    padding-inline-start: 18px;
    line-height: 1.5;
  }
  .deps {
    list-style: none;
    margin: 0 0 12px;
    padding: 0;
    font-size: 0.9rem;
  }
  .deps li {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 5px 0;
    border-bottom: 1px solid var(--line);
  }
  .deps span {
    color: var(--muted);
    white-space: nowrap;
  }
</style>
