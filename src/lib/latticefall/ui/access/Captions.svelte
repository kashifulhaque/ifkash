<script lang="ts">
  // Subtitles for important sounds (WP-082): short bracketed lines near the bottom of the
  // screen, from the simulation's events (see captions.ts). They scale with the text size
  // setting and are read aloud politely by screen readers.
  import type { Readable } from 'svelte/store';
  import type { Caption } from './captions';

  export let captions: Readable<Caption[]>;
  /** The Subtitles setting. */
  export let enabled = true;
</script>

<div class="captions" role="log" aria-live="polite" aria-label="Captions">
  {#if enabled}
    {#each $captions as c (c.id)}
      <p class="caption"><span class="mark" aria-hidden="true">&#9834;</span>[{c.text}]</p>
    {/each}
  {/if}
</div>

<style>
  .captions {
    position: absolute;
    left: 50%;
    bottom: 22%;
    transform: translateX(-50%);
    z-index: 5;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: min(40rem, 92vw);
    pointer-events: none;
    font-family: var(--font-sans);
  }

  .caption {
    margin: 0;
    padding: 4px var(--s-3);
    font-size: calc(var(--text-base) * var(--lf-text, 1));
    line-height: 1.35;
    text-align: center;
    color: var(--ink);
    background: color-mix(in srgb, var(--surface) 92%, transparent);
    border: 1px solid var(--line-strong);
    border-radius: 3px;
  }

  .mark {
    margin-right: 0.5em;
    color: var(--ink-3);
  }
</style>
