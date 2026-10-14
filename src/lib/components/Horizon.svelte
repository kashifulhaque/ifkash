<script lang="ts">
  import { onMount } from "svelte";

  // A fjord at the current hour in Bengaluru. The sun travels a fixed arc
  // between a nominal 06:15 sunrise and 18:20 sunset (IST); outside those
  // hours it sits below the waterline and a crescent moon takes its place.

  const W = 1200;
  const H = 260;
  const WATER = 196;
  const SUNRISE = 6 * 60 + 15;
  const SUNSET = 18 * 60 + 20;

  // Deterministic ridgelines: a few summed sines, sampled every 8 units.
  function ridge(base: number, amp: number, waves: [number, number, number][]): string {
    let d = `M0 ${WATER}`;
    for (let x = 0; x <= W; x += 8) {
      let y = base;
      for (const [freq, phase, weight] of waves) {
        y -= Math.sin((x / W) * Math.PI * freq + phase) * amp * weight;
      }
      d += ` L${x} ${y.toFixed(1)}`;
    }
    return d + ` L${W} ${WATER} Z`;
  }

  const far = ridge(132, 34, [
    [1.4, 0.6, 1],
    [4.2, 1.9, 0.35],
    [9.5, 0.2, 0.12],
  ]);
  const near = ridge(168, 30, [
    [2.1, 2.4, 0.9],
    [5.3, 0.4, 0.4],
    [13, 1.1, 0.1],
  ]);

  let minutes = 14 * 60;
  let label = "";

  $: day = minutes >= SUNRISE && minutes <= SUNSET;
  $: t = day
    ? (minutes - SUNRISE) / (SUNSET - SUNRISE)
    : ((minutes - SUNSET + 1440) % 1440) / (1440 - (SUNSET - SUNRISE));
  // Arc from the left shore to the right shore, peaking well above the ridges.
  $: bodyX = 120 + t * (W - 240);
  $: bodyY = WATER - Math.sin(Math.PI * t) * 150;
  $: phase = day
    ? t < 0.2
      ? "Sunrise"
      : t < 0.45
        ? "Morning"
        : t < 0.6
          ? "Midday"
          : t < 0.85
            ? "Afternoon"
            : "Dusk"
    : "Night";

  onMount(() => {
    const fmt = new Intl.DateTimeFormat("en-GB", {
      hour: "2-digit",
      minute: "2-digit",
      hourCycle: "h23",
      timeZone: "Asia/Kolkata",
    });
    const tick = () => {
      const parts = fmt.formatToParts(new Date());
      const h = Number(parts.find((p) => p.type === "hour")?.value ?? 0);
      const m = Number(parts.find((p) => p.type === "minute")?.value ?? 0);
      minutes = h * 60 + m;
      label = `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
    };
    tick();
    const id = setInterval(tick, 30_000);
    return () => clearInterval(id);
  });

  const reflections = Array.from({ length: 9 }, (_, i) => i);
</script>

<figure class="horizon" class:night={!day}>
  <svg viewBox="0 0 {W} {H}" preserveAspectRatio="xMidYMax slice" role="img" aria-label="{phase} over a fjord, drawn for the current time in Bengaluru">
    <defs>
      <clipPath id="above-water"><rect x="0" y="0" width={W} height={WATER} /></clipPath>
    </defs>

    <!-- the sun's (or moon's) path -->
    <path
      class="arc"
      d="M120 {WATER} Q {W / 2} {WATER - 300} {W - 120} {WATER}"
    />

    <g clip-path="url(#above-water)">
      {#if day}
        <circle class="sun" cx={bodyX} cy={bodyY} r="20" />
      {:else}
        <circle class="moon" cx={bodyX} cy={bodyY} r="14" />
        <circle class="moon-cut" cx={bodyX + 6} cy={bodyY - 4} r="12" />
      {/if}
      <path class="ridge far" d={far} />
      <path class="ridge near" d={near} />
    </g>

    <line class="water" x1="0" y1={WATER} x2={W} y2={WATER} />
    {#each reflections as i}
      <line
        class="ripple"
        x1={bodyX - 28 + (i % 3) * 6}
        x2={bodyX + 28 - (i % 3) * 6}
        y1={WATER + 10 + i * 7}
        y2={WATER + 10 + i * 7}
        style="opacity: {0.9 - i * 0.09}"
      />
    {/each}
  </svg>

  <figcaption>
    <span>{phase} in Bengaluru</span>
    {#if label}<span class="time">{label} IST</span>{/if}
  </figcaption>
</figure>

<style>
  .horizon {
    position: relative;
    margin: 0;
    overflow-x: clip;
    border-bottom: 1px solid var(--line);
  }

  svg {
    width: 100%;
    height: clamp(180px, 22vw, 260px);
    overflow: visible;
  }

  .arc {
    fill: none;
    stroke: var(--line-strong);
    stroke-width: 1;
    stroke-dasharray: 2 6;
    vector-effect: non-scaling-stroke;
  }

  .sun {
    fill: var(--clay);
    transition: cx 1.2s var(--ease-out), cy 1.2s var(--ease-out);
  }

  .moon {
    fill: var(--ink-2);
  }
  .moon-cut {
    fill: var(--bg);
  }

  .ridge {
    stroke-width: 1;
    stroke-linejoin: round;
    vector-effect: non-scaling-stroke;
  }
  .ridge.far {
    fill: var(--surface-2);
    stroke: var(--line-strong);
  }
  .ridge.near {
    fill: var(--bg);
    stroke: var(--ink-3);
  }

  .water {
    stroke: var(--ink);
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
  }

  .ripple {
    stroke: var(--clay);
    stroke-width: 1.5;
    stroke-linecap: round;
    vector-effect: non-scaling-stroke;
  }
  .night .ripple {
    stroke: var(--ink-3);
  }

  figcaption {
    position: absolute;
    right: 0;
    bottom: var(--s-3);
    display: flex;
    gap: var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .time {
    color: var(--ink-2);
  }

  @media (max-width: 640px) {
    figcaption {
      position: static;
      padding: var(--s-2) 0;
    }
  }
</style>
