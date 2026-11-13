<script lang="ts">
  // The in-game timer (WP-088): shown on the HUD when "Show the timer" is on in the settings.
  // It reads the core's tick counter (30 ticks a second) through the Marks host and never the
  // wall clock, so it stops when the game pauses and matches a replay.
  import type { Readable } from 'svelte/store';
  import type { HudSettings } from './hudSettings';
  import { formatTimer } from '../marks/marksData';

  export let ticks: Readable<number>;
  export let settings: Readable<HudSettings>;

  $: shown = $settings.showTimer && $settings.visibility !== 'hidden';
  $: scale = $settings.scale / 100;
  $: text = formatTimer($ticks);
</script>

{#if shown}
  <p class="timer" style="--hs: {scale}" aria-label="In-game time" role="timer" aria-live="off">
    <span class="label">Time</span>
    <span class="value">{text}</span>
  </p>
{/if}

<style>
  .timer {
    position: absolute;
    top: 24px;
    right: 24px;
    display: flex;
    align-items: baseline;
    gap: 0.5em;
    margin: 0;
    padding: 0.25em 0.6em;
    transform: scale(var(--hs, 1));
    transform-origin: top right;
    background: color-mix(in srgb, var(--surface) 82%, transparent);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    pointer-events: none;
    user-select: none;
    z-index: 5;
  }

  .label {
    color: var(--ink-3);
    font-size: var(--text-xs);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .value {
    font-variant-numeric: tabular-nums;
  }
</style>
