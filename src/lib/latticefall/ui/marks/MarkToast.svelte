<script lang="ts">
  // A short "Mark earned" notice (WP-088), one per Mark, stacked above the Codex notice. The core
  // raised the event; this only shows it.
  import type { MarksHost } from './marksHost';

  export let marks: Pick<MarksHost, 'toasts'>;
  const toasts = marks.toasts;
</script>

<div class="stack" aria-live="polite" role="status">
  {#each $toasts as t (t.id)}
    <p class="toast">
      <span class="label">Mark earned</span>
      <strong>{t.mark.name}</strong>
      <span class="how">{t.mark.how}</span>
    </p>
  {/each}
</div>

<style>
  .stack {
    position: absolute;
    left: 50%;
    bottom: calc(var(--s-8) + var(--s-4) + 4.5rem);
    transform: translateX(-50%);
    display: grid;
    gap: var(--s-2);
    justify-items: center;
    pointer-events: none;
    z-index: 10;
  }

  .toast {
    display: grid;
    gap: 2px;
    max-width: min(26rem, calc(100vw - 2 * var(--s-4)));
    margin: 0;
    padding: var(--s-2) var(--s-4);
    background: var(--surface);
    border: 1px solid var(--moss);
    border-radius: var(--radius-sm);
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    text-align: center;
  }

  .label {
    color: var(--moss);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  strong {
    font-family: var(--font-display);
    font-weight: 500;
  }

  .how {
    color: var(--ink-2);
    font-size: var(--text-sm);
  }
</style>
