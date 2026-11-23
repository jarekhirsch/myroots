<script lang="ts">
  // "Suggest a correction" on the family website. There is no server: the
  // suggestion goes by e-mail (with a code the owner pastes into the app) or
  // to an optional HTTPS endpoint configured by the owner.
  import Modal from './Modal.svelte';
  import { app, call } from '../api.svelte';
  import { t } from '../i18n/index.svelte';
  import { toast } from '../ui.svelte';

  let { person = null, personName = '', onclose }: { person?: number | null; personName?: string; onclose: () => void } = $props();

  let message = $state('');
  let author = $state('');
  let email = $state('');
  let trap = $state('');
  let code = $state('');
  let sent = $state(false);
  let busy = $state(false);

  const cfg = $derived(app.info?.suggestions);

  function token() {
    const a = new Uint8Array(12);
    crypto.getRandomValues(a);
    return Array.from(a, (b) => b.toString(16).padStart(2, '0')).join('');
  }

  async function send() {
    if (trap || !message.trim()) return;
    busy = true;
    try {
      const s = { person, person_name: personName, message: message.trim().slice(0, 5000), author: author.trim(), email: email.trim(), token: token(), created: Date.now() };
      code = (await call<{ code: string }>('suggestion.encode', s)).code;
      if (cfg?.endpoint) {
        try {
          const r = await fetch(cfg.endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ ...s, code }) });
          if (r.ok) {
            sent = true;
            return;
          }
        } catch {
          /* fall back to e-mail */
        }
      }
      if (cfg?.email) {
        const subject = t('[MyRoots] New correction suggestion: %s', personName || app.info?.site?.title || '');
        const body = `${message.trim()}\n\n${author ? t('From:') + ' ' + author + '\n' : ''}${email ? t('E-mail:') + ' ' + email + '\n' : ''}\n${t('Code for the MyRoots app (do not change):')}\n${code}\n`;
        location.href = `mailto:${encodeURIComponent(cfg.email)}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
      }
      sent = true;
    } catch {
      toast(t('Sending failed. Please try again.'), 'error');
    } finally {
      busy = false;
    }
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(code);
      toast(t('Copied.'));
    } catch {
      toast(t('Could not copy. Select the text and copy it by hand.'), 'error');
    }
  }
</script>

<Modal title={t('Suggest a correction')} {onclose}>
  {#if sent}
    <p>{cfg?.endpoint && !cfg?.email ? t('Thank you! Your suggestion has been sent for moderation.') : t('Thank you! Your e-mail program should now open with the message ready to send.')}</p>
    {#if code && cfg?.email}
      <p class="small muted">{t('If the e-mail did not open, copy the code below and send it to %s.', cfg.email)}</p>
      <textarea readonly rows="4" class="code">{code}</textarea>
      <button class="btn sm" onclick={copy}>{t('Copy')}</button>
    {/if}
  {:else}
    {#if personName}<p class="muted small">{personName}</p>{/if}
    <form id="sug" onsubmit={(e) => { e.preventDefault(); send(); }}>
      <label class="field">
        <span class="lbl">{t('What should be corrected or added?')}</span>
        <textarea bind:value={message} rows="5" maxlength="5000" placeholder={t('e.g. The date of birth is 12.05.1932, not 1933. I have a copy of the certificate.')} required></textarea>
      </label>
      <div class="form-grid two">
        <label class="field"><span class="lbl">{t('Your name (optional)')}</span><input type="text" bind:value={author} /></label>
        <label class="field"><span class="lbl">{t('Contact e-mail (optional)')}</span><input type="email" bind:value={email} /></label>
      </div>
      <label class="hp" aria-hidden="true">{t('Leave this field empty')}<input type="text" tabindex="-1" autocomplete="off" bind:value={trap} /></label>
    </form>
  {/if}
  {#snippet footer()}
    {#if sent}
      <button class="btn primary" onclick={onclose}>{t('Close')}</button>
    {:else}
      <button class="btn" onclick={onclose}>{t('Cancel')}</button>
      <button class="btn primary" type="submit" form="sug" disabled={busy || !message.trim()}>{t('Send suggestion')}</button>
    {/if}
  {/snippet}
</Modal>

<style>
  .two {
    margin-top: 12px;
  }
  .hp {
    position: absolute;
    left: -9999px;
    width: 1px;
    height: 1px;
    overflow: hidden;
  }
  .code {
    font-family: ui-monospace, monospace;
    font-size: 0.78rem;
    margin-bottom: 8px;
  }
</style>
