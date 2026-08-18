<script lang="ts">
  import { onMount } from 'svelte';

  /* ─────────────────────────────────────────────────────────
   * LOADING STATE — pixel-grid loader for long-running work
   *
   * Variants:
   *   Drive  — square cells, chevron wavefront driving right;
   *            the 650ms cycle is shorter than the sweep, so
   *            two fronts are always in flight
   *   Dots   — same wavefront, circular cells
   *   Orbit  — a comet lapping the grid perimeter
   *
   * Paired with a shimmering label and a live elapsed timer
   * in mono tabular figures. Reduced motion freezes the grid
   * to its dim state; the timer still ticks.
   * ───────────────────────────────────────────────────────── */

  export let label = 'Churning';
  export let variant: 'Drive' | 'Dots' | 'Orbit' = 'Drive';

  const chevron = Array.from({ length: 9 }, (_, i) => {
    const r = Math.floor(i / 3), c = i % 3;
    return (c + Math.abs(r - 1)) * 90;
  });

  const ORBIT_ORDER = [0, 1, 2, 5, 8, 7, 6, 3];
  const orbit: (number | null)[] = Array.from({ length: 9 }, (_, i) => {
    const k = ORBIT_ORDER.indexOf(i);
    return k === -1 ? null : k * 110;
  });

  const PATTERNS: Record<string, { delays: (number | null)[]; dur: number; round: boolean }> = {
    Drive: { delays: chevron, dur: 650, round: false },
    Dots: { delays: chevron, dur: 650, round: true },
    Orbit: { delays: orbit, dur: 950, round: false },
  };

  let ds = 0;
  onMount(() => {
    const t = setInterval(() => (ds += 1), 100);
    return () => clearInterval(t);
  });

  $: total = ds / 10;
  $: elapsed =
    total < 60 ? `${total.toFixed(1)}s` : `${Math.floor(total / 60)}m ${(total % 60).toFixed(1)}s`;
  $: pattern = PATTERNS[variant] ?? PATTERNS.Drive;
</script>

<div class="loader">
  <span class="grid">
    {#each pattern.delays as d}
      <span
        class="cell"
        class:round={pattern.round}
        style={d === null
          ? 'opacity: 0.07; animation: none'
          : `opacity: 0.15; animation-delay: ${d}ms; animation-duration: ${pattern.dur}ms`}
      ></span>
    {/each}
  </span>
  <span class="label">{label}</span>
  <span class="timer">{elapsed}</span>
</div>

<style>
  .loader {
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(3, 4px);
    grid-auto-rows: 4px;
    gap: 1.5px;
  }

  .cell {
    background: var(--ink);
    border-radius: 1px;
    animation: pixel-on ease-in-out infinite;
  }

  .cell.round {
    border-radius: 9999px;
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    color: transparent;
    -webkit-background-clip: text;
    background-clip: text;
    background-image: linear-gradient(
      90deg,
      var(--text-faint) 35%,
      var(--ink) 50%,
      var(--text-faint) 65%
    );
    background-size: 200% 100%;
    animation: shimmer-text 1.4s linear infinite;
    white-space: nowrap;
  }

  .timer {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  @keyframes pixel-on {
    0%,
    100% {
      opacity: 0.15;
    }
    45% {
      opacity: 1;
    }
  }

  @keyframes shimmer-text {
    from {
      background-position: 200% 0;
    }
    to {
      background-position: -200% 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .cell,
    .label {
      animation: none !important;
    }
    .cell {
      opacity: 0.07 !important;
    }
  }
</style>
