<script lang="ts">
  // The plain facts panel (docs/game/08-portfolio-integration.md, rule 3): the entry exactly
  // as `content.ts` writes it, with no fantasy voice. Used by the Ledger, and later by the
  // codex pages and item cards.
  import type { LootItem } from '$lib/content';

  export let item: LootItem;
  /** Heading level, so the panel nests under whatever heading holds it. */
  export let level: 2 | 3 | 4 = 3;
</script>

<article class="facts">
  <svelte:element this={`h${level}`} class="facts-title">
    {item.title}
    {#if item.badge}<span class="facts-badge">{item.badge}</span>{/if}
  </svelte:element>
  {#if item.subtitle}<p class="facts-sub">{item.subtitle}</p>{/if}
  {#if item.body}<p class="facts-body">{item.body}</p>{/if}
  {#if item.links.length}
    <ul class="facts-links">
      {#each item.links as link}
        <li><a href={link.url} target="_blank" rel="noopener noreferrer">{link.label}</a></li>
      {/each}
    </ul>
  {/if}
</article>

<style>
  .facts {
    padding: var(--s-3) var(--s-4);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    font-family: var(--font-sans);
    color: var(--ink);
  }

  .facts-title {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--s-2);
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--text-md);
    font-weight: 600;
    line-height: 1.25;
  }

  .facts-badge {
    padding: 1px var(--s-2);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-pill);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
  }

  .facts-sub {
    margin: var(--s-1) 0 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  .facts-body {
    margin: var(--s-2) 0 0;
    color: var(--ink-2);
    font-size: var(--text-base);
    line-height: 1.5;
    max-width: var(--measure);
  }

  .facts-links {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2) var(--s-4);
    margin: var(--s-3) 0 0;
    padding: 0;
    list-style: none;
    font-size: var(--text-sm);
  }

  .facts-links a {
    color: var(--fjord);
    text-underline-offset: 3px;
  }

  .facts-links a:hover {
    color: var(--fjord-strong);
  }
</style>
