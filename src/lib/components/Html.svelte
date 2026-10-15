<script lang="ts">
  import DOMPurify from 'dompurify';
  // links open outside the app and never get access to it
  if (!(DOMPurify as any).__myroots) {
    (DOMPurify as any).__myroots = true;
    DOMPurify.addHook('afterSanitizeAttributes', (node: Element) => {
      if (node.tagName === 'A') {
        const href = node.getAttribute('href') || '';
        if (href && !/^(https?:|mailto:|#)/i.test(href)) node.removeAttribute('href');
        if (/^(https?:|mailto:)/i.test(href)) {
          node.setAttribute('target', '_blank');
          node.setAttribute('rel', 'noopener noreferrer');
        } else node.removeAttribute('target');
      }
    });
  }
  let { html = '', class: cls = 'prose' }: { html?: string; class?: string } = $props();
  const clean = $derived(
    DOMPurify.sanitize(html || '', {
      ALLOWED_TAGS: ['p', 'br', 'b', 'strong', 'i', 'em', 'u', 's', 'a', 'ul', 'ol', 'li', 'blockquote', 'h2', 'h3', 'h4', 'span', 'div', 'sup', 'sub', 'img', 'figure', 'figcaption', 'hr'],
      ALLOWED_ATTR: ['href', 'title', 'src', 'alt', 'target', 'rel'],
    }),
  );
</script>

<div class={cls}>{@html clean}</div>
