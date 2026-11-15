<script lang="ts">
  // The end-of-tutorial card. It opens after the Hollow Sentry falls while only Waking Hollow is
  // open (`tutorialLock.ts`). DOM only; it reads its store and never writes the simulation.
  import { onDestroy, onMount, tick } from 'svelte';
  import { closeTutorialEnd, showTutorialEnd, tutorialEndOpen } from './tutorialEnd';

  let closeButton: HTMLButtonElement | undefined;
  let wasOpen = false;
  let returnFocus: Element | null = null;

  onMount(() => {
    // Dev only: `&dev=1&hazard=tutorial-end` shows the card at once, for a screenshot.
    const p = new URLSearchParams(window.location.search);
    if (p.get('dev') === '1' && p.get('hazard') === 'tutorial-end') showTutorialEnd(0);
  });

  onDestroy(closeTutorialEnd);

  $: if ($tutorialEndOpen && !wasOpen) {
    wasOpen = true;
    returnFocus = document.activeElement;
    // A captured mouse would swallow the buttons, so hand the pointer back.
    document.exitPointerLock?.();
    void tick().then(() => closeButton?.focus());
  } else if (!$tutorialEndOpen && wasOpen) {
    wasOpen = false;
    if (returnFocus instanceof HTMLElement) returnFocus.focus();
  }

  function onKeydown(e: KeyboardEvent): void {
    if (!$tutorialEndOpen) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      closeTutorialEnd();
    }
  }
</script>

<svelte:window on:keydown|capture={onKeydown} />

{#if $tutorialEndOpen}
  <div class="scrim">
    <section class="card" role="dialog" aria-modal="true" aria-labelledby="tutorial-end-title">
      <p class="eyebrow">The Hollow Sentry falls</p>
      <h2 id="tutorial-end-title">You finished the tutorial</h2>
      <p class="lede">
        That's all of Cinderheart that's open for now. The Verge and the rest of the world are still being built. You can keep
        exploring Waking Hollow, or head back to the site.
      </p>
      <div class="actions">
        <a class="btn" href="/">Back to the site</a>
        <button type="button" class="btn primary" bind:this={closeButton} on:click={closeTutorialEnd}>Keep exploring</button>
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

  .card {
    box-sizing: border-box;
    width: min(520px, 100%);
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
    margin: var(--s-3) 0 0;
    color: var(--ink-2);
    font-size: var(--text-md);
    line-height: 1.5;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: var(--s-2);
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
    text-decoration: none;
    cursor: pointer;
  }

  .btn.primary {
    background: var(--fjord);
    border-color: var(--fjord);
    color: var(--bg);
  }

  .btn:focus-visible {
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
    .card {
      animation: none;
    }
  }
</style>
