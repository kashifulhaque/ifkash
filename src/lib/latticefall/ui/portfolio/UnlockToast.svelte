<script lang="ts">
  // A short "Codex entry unlocked" notice, shown for a few seconds after a pickup.
  import { onDestroy } from 'svelte';
  import type { CodexEntry } from '../../codex/entries';

  /** Shows the notice for this entry; null hides it. */
  export function show(entry: CodexEntry): void {
    current = entry;
    clearTimeout(timer);
    timer = setTimeout(() => (current = null), 4500);
  }

  let current: CodexEntry | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  onDestroy(() => clearTimeout(timer));
</script>

{#if current}
  <p class="toast" role="status" aria-live="polite">
    <span class="label">Codex entry unlocked</span>
    <strong>{current.item.title}</strong>
  </p>
{/if}

<style>
  .toast {
    position: absolute;
    left: 50%;
    bottom: calc(var(--s-8) + var(--s-4));
    transform: translateX(-50%);
    display: grid;
    gap: 2px;
    margin: 0;
    padding: var(--s-2) var(--s-4);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    text-align: center;
    z-index: 10;
  }

  .label {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
</style>
