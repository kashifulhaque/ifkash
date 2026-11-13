<script lang="ts">
  // One codex entry: the plain facts as `content.ts` writes them. Shared by the Chronicle Page
  // and relic routes. The text is in the prerendered HTML; the found state appears after mount.
  import { onMount } from 'svelte';
  import { codex } from '$lib/latticefall/codex/store';
  import type { CodexEntry } from '$lib/latticefall/codex/entries';
  import { GROUPS, entryDescription } from './codexPages';

  export let entry: CodexEntry;

  let mounted = false;
  onMount(() => {
    codex.reload();
    mounted = true;
  });

  $: item = entry.item;
  $: kind = GROUPS.find((g) => g.section === entry.section)?.kind ?? 'Codex entry';
  $: found = $codex.entries.includes(entry.key);
</script>

<svelte:head>
  <title>{item.title} | Codex | Kashiful Haque</title>
  <meta name="description" content={entryDescription(entry)} />
  <meta property="og:title" content={`${item.title} | Codex | Kashiful Haque`} />
  <meta property="og:description" content={entryDescription(entry)} />
  <meta property="og:type" content="article" />
</svelte:head>

<article class="entry">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/codex">Codex</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">{item.title}</span>
    </nav>
    <p class="eyebrow">
      {kind}
      {#if item.badge}<span class="badge">{item.badge}</span>{/if}
    </p>
    <h1 class="page-title">{item.title}</h1>
    {#if item.subtitle}<p class="sub">{item.subtitle}</p>{/if}
    {#if item.body}<p class="lead">{item.body}</p>{/if}
  </header>

  <section class="facts" aria-label="Facts">
    {#if mounted}
      <p class="state" class:found>
        {found ? 'Found in the world.' : 'Not found in the world yet. The facts are here either way.'}
      </p>
    {/if}
    {#if item.links.length}
      <ul class="links">
        {#each item.links as link}
          <li>
            <a href={link.url} target="_blank" rel="noopener noreferrer">
              {link.label} <span aria-hidden="true">↗</span>
            </a>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</article>

<style>
  .entry {
    display: grid;
    gap: var(--s-6);
    padding-bottom: var(--s-9);
  }

  .badge {
    margin-left: var(--s-2);
    padding: 1px var(--s-2);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-pill);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    text-transform: none;
  }

  .sub {
    margin: var(--s-2) 0 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  .facts {
    display: grid;
    gap: var(--s-3);
    padding: var(--s-4);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    max-width: var(--measure);
  }

  .state {
    margin: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .state.found {
    color: var(--moss);
  }

  .links {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2) var(--s-4);
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: var(--text-sm);
  }

  .links a {
    color: var(--fjord);
    text-underline-offset: 3px;
  }

  .links a:hover {
    color: var(--fjord-strong);
  }
</style>
