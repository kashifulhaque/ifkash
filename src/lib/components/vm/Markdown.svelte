<script lang="ts">
  import { renderMarkdown } from '$lib/vmMarkdown';

  export let text = '';

  $: html = renderMarkdown(text);
</script>

<div class="md">{@html html}</div>

<style>
  .md {
    color: var(--ink);
    font-size: 15px;
    line-height: 1.6;
    overflow-wrap: anywhere;
  }
  .md :global(> :first-child) { margin-top: 0; }
  .md :global(> :last-child) { margin-bottom: 0; }
  .md :global(p),
  .md :global(ul),
  .md :global(ol),
  .md :global(pre),
  .md :global(blockquote),
  .md :global(table) { margin: 0 0 var(--s-3); }
  .md :global(ul),
  .md :global(ol) { padding-left: 1.4em; }
  .md :global(li + li) { margin-top: 2px; }
  .md :global(h1),
  .md :global(h2),
  .md :global(h3),
  .md :global(h4) {
    margin: var(--s-4) 0 var(--s-2);
    font-family: var(--font-display);
    font-weight: 500;
    line-height: 1.25;
    letter-spacing: -0.01em;
  }
  .md :global(h1) { font-size: 1.25rem; }
  .md :global(h2) { font-size: 1.125rem; }
  .md :global(h3),
  .md :global(h4) { font-size: 1rem; }
  .md :global(a) {
    color: var(--fjord);
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .md :global(code) {
    padding: 1px 5px;
    border-radius: var(--radius-xs);
    background: var(--code-surface);
    font-family: var(--font-mono);
    font-size: 0.86em;
  }
  .md :global(pre) {
    padding: var(--s-3);
    overflow-x: auto;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--code-surface);
    -webkit-overflow-scrolling: touch;
  }
  .md :global(pre code) {
    padding: 0;
    background: none;
    font-size: 12.5px;
    line-height: 1.5;
    overflow-wrap: normal;
    white-space: pre;
  }
  .md :global(blockquote) {
    padding-left: var(--s-3);
    border-left: 2px solid var(--line-strong);
    color: var(--ink-2);
  }
  .md :global(table) {
    display: block;
    max-width: 100%;
    overflow-x: auto;
    border-collapse: collapse;
    font-size: 14px;
  }
  .md :global(th),
  .md :global(td) {
    padding: 4px var(--s-3);
    border: 1px solid var(--line);
    text-align: left;
  }
  .md :global(th) { background: var(--surface-2); font-weight: 500; }
  .md :global(hr) { border: 0; border-top: 1px solid var(--line); margin: var(--s-4) 0; }
</style>
