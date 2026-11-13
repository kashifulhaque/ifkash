<svelte:head>
  <title>Codex | Kashiful Haque</title>
  <meta name="description" content="Kashiful Haque: ML engineer. Portfolio, and a small action RPG." />
  <meta property="og:title" content="Codex | Kashiful Haque" />
  <meta property="og:description" content="Kashiful Haque: ML engineer. Portfolio, and a small action RPG." />
  <meta property="og:type" content="website" />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { codex } from '$lib/latticefall/codex/store';
  import { GROUPS, entriesIn, entryHref } from './codexPages';

  // Every portfolio entry is listed with its full title, with or without JavaScript. The found
  // state is applied only after mount, so the prerendered HTML and the first client render match.
  let mounted = false;
  onMount(() => {
    codex.reload();
    mounted = true;
  });

  const groups = GROUPS.map((g) => ({ ...g, entries: entriesIn(g.section) }));
  const total = groups.reduce((n, g) => n + g.entries.length, 0);
  $: found = groups.reduce((n, g) => n + g.entries.filter((e) => $codex.entries.includes(e.key)).length, 0);
</script>

<article class="codex">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <span aria-current="page">Codex</span>
    </nav>
    <h1 class="page-title">Codex</h1>
    <p class="lead">
      The portfolio in plain text: every role, project, and page. Entries you find in the game
      are marked as found there.
    </p>
    {#if mounted}
      <p class="eyebrow" aria-live="polite">{found} of {total} found in the world</p>
    {/if}
  </header>

  {#each groups as group (group.section)}
    <section class="codex-group" aria-labelledby={`codex-${group.section}`}>
      <h2 id={`codex-${group.section}`} class="group-title">{group.label}</h2>
      <ul class="codex-list">
        {#each group.entries as entry (entry.key)}
          <li class="codex-item" class:locked={mounted && !$codex.entries.includes(entry.key)}>
            <a class="codex-link" href={entryHref(entry)}>
              <span class="codex-name">{entry.item.title}</span>
              {#if entry.item.subtitle}<span class="codex-sub">{entry.item.subtitle}</span>{/if}
            </a>
            {#if mounted}
              <span class="codex-state">{$codex.entries.includes(entry.key) ? 'Found' : 'Not found yet'}</span>
            {/if}
          </li>
        {/each}
      </ul>
    </section>
  {/each}
</article>

<style>
  .codex {
    display: grid;
    gap: var(--s-7);
    padding-bottom: var(--s-9);
  }

  .group-title {
    margin: 0 0 var(--s-3);
    color: var(--ink);
    font-family: var(--font-display);
    font-size: var(--text-xl);
    font-weight: 500;
    letter-spacing: -0.02em;
  }

  .codex-list {
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }

  .codex-item {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-2) var(--s-4);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .codex-link {
    display: grid;
    gap: 2px;
    color: var(--ink);
    text-decoration: none;
  }

  .codex-link:hover .codex-name {
    color: var(--fjord-strong);
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  .codex-name {
    font-family: var(--font-display);
    font-size: var(--text-md);
    font-weight: 500;
  }

  .codex-sub {
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  .codex-state {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  /* Locked entries stay readable and linked; only their state label is quieter. */
  .codex-item.locked .codex-name {
    color: var(--ink-2);
  }
  .codex-item.locked .codex-state {
    color: var(--ink-3);
    font-style: italic;
  }
  .codex-item:not(.locked) .codex-state {
    color: var(--moss);
  }
</style>
