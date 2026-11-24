<script lang="ts">
  // "Report a problem": a description plus technical details (version, system,
  // last errors), sent by e-mail or as a GitHub issue. Nothing from the family
  // tree is included except the number of people.
  import Modal from './Modal.svelte';
  import { app, call } from '../api.svelte';
  import { backend } from '../backend';
  import { t, lang } from '../i18n/index.svelte';
  import { toast } from '../ui.svelte';
  import { recentErrors } from '../errorlog';
  import { SOURCE_URL, REPORT_EMAIL } from '../credits';
  import Mail from '../icons/EnvelopeSimple.svelte';
  import External from '../icons/ArrowSquareOut.svelte';
  import Copy from '../icons/Copy.svelte';

  let { onclose }: { onclose: () => void } = $props();
  let what = $state('');
  let expected = $state('');
  let tech = $state(true);
  let people = $state<number | null>(null);
  call<{ persons: number }>('overview').then((o) => (people = o.persons), () => {});

  const details = $derived.by(() => {
    const errs = recentErrors().slice(-5);
    return [
      `MyRoots ${app.info?.version || '?'} (${backend.platform})`,
      `${navigator.userAgent}`,
      `${t('Language')}: ${lang()}, ${t('Screen')}: ${window.innerWidth}×${window.innerHeight}`,
      `${t('People in the tree')}: ${people ?? '?'}`,
      ...(errs.length ? ['', t('Last errors:'), ...errs.map((e) => `${e.at.slice(11, 19)} ${e.message}`)] : []),
    ].join('\n');
  });
  const subject = $derived(`MyRoots ${app.info?.version || ''}: ${what.trim().split('\n')[0].slice(0, 70) || t('Problem report')}`);
  const body = $derived(
    [what.trim(), expected.trim() ? `\n${t('What I expected:')}\n${expected.trim()}` : '', tech ? `\n---\n${details}` : ''].filter(Boolean).join('\n'),
  );
  // e-mail programs cut very long links: the technical part is shortened first
  const mailto = $derived(`mailto:${REPORT_EMAIL}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body.slice(0, 1800))}`);
  const issue = $derived(`${SOURCE_URL}/issues/new?title=${encodeURIComponent(subject)}&body=${encodeURIComponent(body.slice(0, 6000))}`);

  async function copy() {
    try {
      await navigator.clipboard.writeText(`${subject}\n\n${body}`);
      toast(t('Copied.'));
    } catch {
      toast(t('Could not copy. Select the text and copy it by hand.'), 'error');
    }
  }
</script>

<Modal title={t('Report a problem')} {onclose}>
  <div class="rep">
    <label class="field">
      <span>{t('What happened?')}</span>
      <textarea rows="4" bind:value={what} placeholder={t('e.g. After clicking “Publish” nothing happens.')}></textarea>
    </label>
    <label class="field">
      <span>{t('What did you expect? (optional)')}</span>
      <textarea rows="2" bind:value={expected}></textarea>
    </label>
    <label class="check"><input type="checkbox" bind:checked={tech} /> {t('Attach technical details (no data from your tree)')}</label>
    {#if tech}<pre class="small">{details}</pre>{/if}
  </div>
  {#snippet footer()}
    <button class="btn ghost" onclick={copy}><Copy size={16} /> {t('Copy')}</button>
    <a class="btn" class:disabled={!what.trim()} href={issue} target="_blank" rel="noopener"><External size={16} /> GitHub</a>
    <a class="btn primary" class:disabled={!what.trim()} href={mailto}><Mail size={16} /> {t('Send by e-mail')}</a>
  {/snippet}
</Modal>

<style>
  .rep {
    display: grid;
    gap: 12px;
  }
  textarea {
    width: 100%;
  }
  pre {
    margin: 0;
    padding: 10px 12px;
    border-radius: 10px;
    background: var(--bg-soft, rgba(0, 0, 0, 0.04));
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 180px;
    overflow: auto;
  }
  .check {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  a.disabled {
    pointer-events: none;
    opacity: 0.5;
  }
</style>
