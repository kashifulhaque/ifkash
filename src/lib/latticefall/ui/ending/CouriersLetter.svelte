<script lang="ts">
  // The ending screen: the Courier's Letter (docs/game/08-portfolio-integration.md, "Contact"). It
  // opens after B12 falls and shows the same contact entries the Courier's menu and the landing
  // card do, as plain facts with no fantasy voice (rule 3 of that page). DOM only; it reads the
  // ending store and never writes the simulation.
  import { onDestroy, onMount, tick } from 'svelte';
  import FactsPanel from '../portfolio/FactsPanel.svelte';
  import { contactItems } from '../portfolio/resume';
  import { closeEnding, endingOpen, showEnding } from './endingStore';

  const contacts = contactItems();

  let dialog: HTMLElement | undefined;
  let closeButton: HTMLButtonElement | undefined;
  let wasOpen = false;
  let returnFocus: Element | null = null;

  onMount(() => {
    // Dev only: `&dev=1&hazard=ending` shows the letter at once, for a screenshot.
    const p = new URLSearchParams(window.location.search);
    if (p.get('dev') === '1' && p.get('hazard') === 'ending') showEnding(0);
  });

  onDestroy(closeEnding);

  $: if ($endingOpen && !wasOpen) {
    wasOpen = true;
    returnFocus = document.activeElement;
    // A captured mouse would swallow the buttons, so hand the pointer back.
    document.exitPointerLock?.();
    void tick().then(() => closeButton?.focus());
  } else if (!$endingOpen && wasOpen) {
    wasOpen = false;
    if (returnFocus instanceof HTMLElement) returnFocus.focus();
  }

  function onKeydown(e: KeyboardEvent): void {
    if (!$endingOpen) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      closeEnding();
    } else if (e.key === 'Tab' && dialog) {
      // Keep Tab inside the letter.
      const items = Array.from(dialog.querySelectorAll<HTMLElement>('a[href], button:not([disabled])'));
      if (items.length === 0) return;
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

<svelte:window on:keydown|capture={onKeydown} />

{#if $endingOpen}
  <div class="scrim">
    <section class="letter" bind:this={dialog} role="dialog" aria-modal="true" aria-labelledby="letter-title">
      <p class="eyebrow">The thread is tied</p>
      <h2 id="letter-title">The Courier's Letter</h2>
      <p class="lede">
        The Courier's satchel held one more letter, addressed to whoever reached the top of the Spire. It says where to write
        back.
      </p>
      {#if contacts.length}
        <ul class="contacts">
          {#each contacts as item}
            <li><FactsPanel {item} level={3} /></li>
          {/each}
        </ul>
      {/if}
      <div class="actions">
        <button type="button" class="btn primary" bind:this={closeButton} on:click={closeEnding}>Close the letter</button>
      </div>
    </section>
  </div>
{/if}

<style>
  .scrim {
    position: absolute;
    inset: 0;
    z-index: 60;
    display: grid;
    place-items: center;
    padding: var(--s-4);
    background: color-mix(in srgb, var(--bg) 72%, transparent);
    backdrop-filter: blur(3px);
    pointer-events: auto;
    overflow-y: auto;
    animation: fade-in 0.6s ease both;
  }

  .letter {
    box-sizing: border-box;
    width: min(560px, 100%);
    max-height: 100%;
    overflow-y: auto;
    padding: var(--s-6);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-md);
    color: var(--ink);
    font-family: var(--font-sans);
    box-shadow: 0 18px 60px rgb(0 0 0 / 0.3);
    animation: rise 0.7s ease both;
  }

  .eyebrow {
    margin: 0 0 var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--text-2xl);
    font-weight: 600;
    line-height: 1.1;
  }

  .lede {
    margin: var(--s-3) 0 var(--s-4);
    color: var(--ink-2);
    font-size: var(--text-md);
    line-height: 1.5;
  }

  .contacts {
    display: grid;
    gap: var(--s-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: var(--s-5);
  }

  .btn {
    padding: var(--s-2) var(--s-4);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    cursor: pointer;
  }

  .btn.primary {
    background: var(--fjord);
    border-color: var(--fjord);
    color: var(--bg);
  }

  .btn:focus-visible,
  .letter :global(a:focus-visible) {
    outline: 2px solid var(--fjord);
    outline-offset: 2px;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .scrim,
    .letter {
      animation: none;
    }
  }
</style>
