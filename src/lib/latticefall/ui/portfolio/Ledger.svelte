<script lang="ts">
  // The Wayfarer's Ledger: the key item every player holds from the first tick
  // (docs/game/08-portfolio-integration.md, "Résumé"). Its Read action opens the résumé in a
  // new tab and earns the Reader mark. It also lists how to reach the owner and every codex
  // entry found so far, so no fact is locked behind play.
  import { createEventDispatcher, onMount } from 'svelte';
  import { name } from '$lib/content';
  import { entryByKey } from '../../codex/entries';
  import { codex } from '../../codex/store';
  import FactsPanel from './FactsPanel.svelte';
  import { contactItems, resumeHref } from './resume';

  const dispatch = createEventDispatcher<{ close: void }>();
  const contacts = contactItems();

  $: found = $codex.entries.map((k) => entryByKey(k)).filter((e) => e !== undefined);
  $: isReader = $codex.marks.includes('reader');

  let dialog: HTMLDivElement;
  let readButton: HTMLAnchorElement;

  onMount(() => readButton?.focus());

  function read(): void {
    codex.mark('reader');
  }

  function onKey(e: KeyboardEvent): void {
    if (e.code === 'Escape') {
      e.preventDefault();
      dispatch('close');
    } else if (e.code === 'Tab') {
      // Keep focus inside the dialog.
      const items = Array.from(dialog.querySelectorAll<HTMLElement>('a[href], button'));
      if (!items.length) return;
      const first = items[0];
      const last = items[items.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }
</script>

<!-- Escape and the Close button are the keyboard routes; the scrim click is a mouse extra. -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="scrim" role="presentation" on:click|self={() => dispatch('close')}>
  <div
    class="ledger"
    bind:this={dialog}
    role="dialog"
    aria-modal="true"
    aria-labelledby="ledger-title"
    tabindex="-1"
    on:keydown={onKey}
  >
    <header class="head">
      <div>
        <p class="eyebrow">Wayfarer's Ledger</p>
        <h2 id="ledger-title">{name}</h2>
      </div>
      <button type="button" class="close" on:click={() => dispatch('close')}>Close</button>
    </header>

    <section aria-labelledby="ledger-resume">
      <h3 id="ledger-resume">Résumé</h3>
      <p class="line">The Ledger opens the résumé as a PDF in a new tab.</p>
      <p class="actions">
        <a
          class="btn primary"
          href={resumeHref()}
          target="_blank"
          rel="noopener noreferrer"
          bind:this={readButton}
          on:click={read}>Read the résumé</a
        >
        {#if isReader}<span class="mark">Reader mark earned</span>{/if}
      </p>
    </section>

    <section aria-labelledby="ledger-contact">
      <h3 id="ledger-contact">How to reach the Wayfarer</h3>
      <ul class="list">
        {#each contacts as item}
          <li><FactsPanel {item} level={4} /></li>
        {/each}
      </ul>
    </section>

    <section aria-labelledby="ledger-found">
      <h3 id="ledger-found">Found in the world</h3>
      {#if found.length === 0}
        <p class="line">Nothing yet. Relics and Chronicle Pages you pick up are written here.</p>
      {:else}
        <ul class="list">
          {#each found as entry}
            {#if entry}
              <li>
                {#if entry.flavor}<p class="flavor">{entry.flavor}</p>{/if}
                <FactsPanel item={entry.item} level={4} />
              </li>
            {/if}
          {/each}
        </ul>
      {/if}
    </section>
  </div>
</div>

<style>
  .scrim {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--s-4);
    background: var(--scrim);
    z-index: 20;
  }

  .ledger {
    width: min(560px, 100%);
    max-height: 100%;
    overflow-y: auto;
    padding: var(--s-5);
    background: var(--bg);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-md);
    color: var(--ink);
    font-family: var(--font-sans);
    box-shadow: 0 12px 40px var(--shadow-color);
  }

  .head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-4);
    margin-bottom: var(--s-4);
  }

  .eyebrow {
    margin: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  h2 {
    margin: var(--s-1) 0 0;
    font-family: var(--font-display);
    font-size: var(--text-xl);
    font-weight: 600;
  }

  h3 {
    margin: var(--s-5) 0 var(--s-2);
    font-size: var(--text-base);
    font-weight: 600;
  }

  .line {
    margin: 0 0 var(--s-3);
    color: var(--ink-2);
    font-size: var(--text-sm);
  }

  .flavor {
    margin: 0 0 var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-serif);
    font-style: italic;
    font-size: var(--text-sm);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-3);
    margin: 0;
  }

  .mark {
    color: var(--moss);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .list {
    display: grid;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .btn,
  .close {
    padding: var(--s-2) var(--s-4);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    text-decoration: none;
    cursor: pointer;
  }

  .btn.primary {
    background: var(--fjord);
    border-color: var(--fjord);
    color: var(--ink-inverse);
  }

  .btn:hover,
  .close:hover {
    border-color: var(--ink-3);
  }

  .btn:focus-visible,
  .close:focus-visible {
    outline: 2px solid var(--fjord);
    outline-offset: 2px;
  }
</style>
