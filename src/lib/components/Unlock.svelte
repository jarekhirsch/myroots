<script lang="ts">
  // Password of a protected family website (the whole website or chosen people).
  import { t, tn } from '../i18n/index.svelte';
  import { tryPassword, rememberKey, type LockHeader } from '../sitelock';
  import LockIcon from '../icons/LockKey.svelte';

  let { header, page = false }: { header: LockHeader; page?: boolean } = $props();

  let password = $state('');
  let remember = $state(false);
  let busy = $state(false);
  let wrong = $state(false);

  async function submit(e: Event) {
    e.preventDefault();
    if (!password || busy) return;
    busy = true;
    wrong = false;
    const key = await tryPassword(header, password).catch(() => null);
    if (!key) {
      busy = false;
      wrong = true;
      return;
    }
    rememberKey(header, key, remember);
    location.reload();
  }
</script>

<form class="unlock" class:page onsubmit={submit}>
  <span class="ic" aria-hidden="true"><LockIcon size={26} strokeWidth={1.8} /></span>
  {#if page}
    {#if header.title}<h1>{header.title}</h1>{/if}
    {#if header.subtitle}<p class="sub">{header.subtitle}</p>{/if}
    <p class="lead">{t('This family website is protected with a password.')}</p>
  {:else}
    <p class="lead">{tn('%d person on this website is protected with a password.', '%d people on this website are protected with a password.', header.people)}</p>
  {/if}
  <label class="field">
    <span class="lbl">{t('Password')}</span>
    <!-- svelte-ignore a11y_autofocus -->
    <input type="password" bind:value={password} autocomplete="current-password" autofocus aria-invalid={wrong} />
  </label>
  {#if header.hint}<p class="hint">{t('Hint: %s', header.hint)}</p>{/if}
  {#if wrong}<p class="err" role="alert">{t('Wrong password. Try again.')}</p>{/if}
  <label class="check"><input type="checkbox" bind:checked={remember} /> {t('Remember on this device')}</label>
  <button class="btn primary" disabled={busy || !password}>{busy ? t('Checking…') : t('Unlock')}</button>
</form>

<style>
  .unlock {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: min(380px, 100%);
    margin: 0 auto;
  }
  .unlock.page {
    min-height: 100vh;
    justify-content: center;
    padding: 24px 16px;
    text-align: center;
  }
  .page .field,
  .page .check {
    text-align: start;
  }
  .ic {
    width: 56px;
    height: 56px;
    border-radius: 18px;
    display: grid;
    place-items: center;
    margin: 0 auto 4px;
    background: var(--ink-soft);
    color: var(--ink);
  }
  .unlock:not(.page) .ic {
    margin: 0;
  }
  h1 {
    font-size: 1.5rem;
    margin: 0;
  }
  .sub {
    color: var(--muted);
    margin: -6px 0 0;
  }
  .lead {
    margin: 0;
  }
  .hint {
    color: var(--muted);
    font-size: 0.88rem;
    margin: -4px 0 0;
  }
  .err {
    color: var(--danger, #b3261e);
    margin: 0;
    font-size: 0.92rem;
  }
  .page .btn {
    justify-content: center;
  }
</style>
