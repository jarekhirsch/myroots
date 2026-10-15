<script lang="ts">
  // Small rich-text editor (bold, italic, headings, lists, quote, link).
  // Stores the same HTML subset the WordPress plugin used.
  import DOMPurify from 'dompurify';
  import { t } from '../i18n/index.svelte';
  import { promptDialog } from '../ui.svelte';
  import Bold from '@lucide/svelte/icons/bold';
  import Italic from '@lucide/svelte/icons/italic';
  import Heading from '@lucide/svelte/icons/heading-2';
  import ListUl from '@lucide/svelte/icons/list';
  import ListOl from '@lucide/svelte/icons/list-ordered';
  import Quote from '@lucide/svelte/icons/quote';
  import LinkIcon from '@lucide/svelte/icons/link';
  import Eraser from '@lucide/svelte/icons/remove-formatting';

  let { value = $bindable(''), label = '', minHeight = 140 }: { value?: string; label?: string; minHeight?: number } = $props();
  let el: HTMLDivElement;
  let last = '';

  const ALLOWED = {
    ALLOWED_TAGS: ['p', 'br', 'b', 'strong', 'i', 'em', 'u', 's', 'a', 'ul', 'ol', 'li', 'blockquote', 'h2', 'h3', 'h4', 'sup', 'sub', 'hr'],
    ALLOWED_ATTR: ['href', 'title', 'target', 'rel'],
  };

  $effect(() => {
    if (el && value !== last) {
      el.innerHTML = DOMPurify.sanitize(value || '', ALLOWED);
      last = value;
    }
  });

  function sync() {
    let html = el.innerHTML;
    if (!el.textContent?.trim() && !el.querySelector('hr')) html = '';
    html = DOMPurify.sanitize(html, ALLOWED);
    last = html;
    value = html;
  }

  function cmd(name: string, arg?: string) {
    el.focus();
    document.execCommand(name, false, arg);
    sync();
  }

  async function link() {
    const sel = window.getSelection();
    const range = sel && sel.rangeCount ? sel.getRangeAt(0).cloneRange() : null;
    const url = await promptDialog(t('Insert link'), t('Address (URL)'), 'https://');
    if (!url || url === 'https://') return;
    el.focus();
    if (range) {
      sel!.removeAllRanges();
      sel!.addRange(range);
    }
    if (/^(https?:|mailto:)/i.test(url.trim())) cmd('createLink', url.trim());
  }

  function paste(e: ClipboardEvent) {
    // Paste as clean HTML subset (no styles from word processors).
    const html = e.clipboardData?.getData('text/html');
    const text = e.clipboardData?.getData('text/plain') || '';
    e.preventDefault();
    if (html) document.execCommand('insertHTML', false, DOMPurify.sanitize(html, ALLOWED));
    else document.execCommand('insertText', false, text);
    sync();
  }
</script>

<div class="rt">
  <div class="bar" role="toolbar" aria-label={label}>
    <button type="button" onclick={() => cmd('bold')} title={t('Bold')} aria-label={t('Bold')}><Bold size={15} /></button>
    <button type="button" onclick={() => cmd('italic')} title={t('Italic')} aria-label={t('Italic')}><Italic size={15} /></button>
    <button type="button" onclick={() => cmd('formatBlock', 'h3')} title={t('Heading')} aria-label={t('Heading')}><Heading size={15} /></button>
    <button type="button" onclick={() => cmd('insertUnorderedList')} title={t('List')} aria-label={t('List')}><ListUl size={15} /></button>
    <button type="button" onclick={() => cmd('insertOrderedList')} title={t('Numbered list')} aria-label={t('Numbered list')}><ListOl size={15} /></button>
    <button type="button" onclick={() => cmd('formatBlock', 'blockquote')} title={t('Quote')} aria-label={t('Quote')}><Quote size={15} /></button>
    <button type="button" onclick={link} title={t('Insert link')} aria-label={t('Insert link')}><LinkIcon size={15} /></button>
    <button type="button" onclick={() => { cmd('removeFormat'); cmd('formatBlock', 'p'); }} title={t('Clear formatting')} aria-label={t('Clear formatting')}><Eraser size={15} /></button>
  </div>
  <div
    class="ed prose"
    bind:this={el}
    contenteditable="true"
    role="textbox"
    aria-multiline="true"
    aria-label={label}
    tabindex="0"
    style="min-height:{minHeight}px"
    oninput={sync}
    onblur={sync}
    onpaste={paste}
  ></div>
</div>

<style>
  .rt {
    border: 1.5px solid transparent;
    border-radius: 12px;
    background: var(--surface-2);
    overflow: hidden;
  }
  .rt:focus-within {
    background: var(--surface);
    border-color: var(--ink);
    box-shadow: 0 0 0 3px var(--ink-soft);
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    gap: 2px;
    padding: 4px;
    border-bottom: 1px solid var(--line);
    background: transparent;
  }
  .bar button {
    border: 0;
    background: none;
    width: 30px;
    height: 28px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--muted);
    cursor: pointer;
  }
  .bar button:hover {
    background: var(--ink-soft);
    color: var(--text);
  }
  .ed {
    padding: 10px 12px;
    outline: none;
    max-width: none;
    color: var(--text);
    font-size: 0.97rem;
  }
  .ed :global(blockquote) {
    border-inline-start: 3px solid var(--heirloom);
    margin: 0.4em 0;
    padding-inline-start: 12px;
    color: var(--muted);
  }
</style>
