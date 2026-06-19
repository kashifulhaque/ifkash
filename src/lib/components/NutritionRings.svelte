<script lang="ts">
  import type { Totals } from '$lib/mealsApi';
  import type { NutritionTargets } from '$lib/fitnessMetrics';

  export let totals: Totals;
  export let targets: NutritionTargets;

  // Concentric rings, Apple-watch style: calories outermost, then protein, carbs,
  // fat. Each ring is an arc whose length is the consumed/target fraction.
  const SIZE = 200;
  const CENTER = SIZE / 2;
  const STROKE = 16;
  const GAP = 6;

  type RingDef = {
    key: string;
    label: string;
    color: string;
    unit: string;
    consumed: number;
    target: number;
  };

  $: rings = [
    { key: 'calories', label: 'Calories', color: 'var(--blueprint)', unit: 'kcal', consumed: totals.calories, target: targets.calories },
    { key: 'protein', label: 'Protein', color: '#e0b341', unit: 'g', consumed: totals.protein_g, target: targets.protein_g },
    { key: 'carbs', label: 'Carbs', color: '#5bb98c', unit: 'g', consumed: totals.carbs_g, target: targets.carbs_g },
    { key: 'fat', label: 'Fat', color: '#e06c8c', unit: 'g', consumed: totals.fat_g, target: targets.fat_g }
  ] as RingDef[];

  const radiusFor = (i: number) => CENTER - STROKE / 2 - i * (STROKE + GAP);
  const circ = (r: number) => 2 * Math.PI * r;
  const pct = (r: RingDef) => (r.target > 0 ? Math.min(1, r.consumed / r.target) : 0);
  const round = (v: number) => Math.round(v);
</script>

<div class="rings">
  <svg viewBox="0 0 {SIZE} {SIZE}" class="dial" role="img" aria-label="Daily nutrition progress rings">
    {#each rings as r, i}
      {@const radius = radiusFor(i)}
      {@const c = circ(radius)}
      <circle
        class="track"
        cx={CENTER}
        cy={CENTER}
        r={radius}
        stroke-width={STROKE}
        style="stroke: {r.color};"
      />
      <circle
        class="value"
        cx={CENTER}
        cy={CENTER}
        r={radius}
        stroke-width={STROKE}
        stroke-dasharray="{c * pct(r)} {c}"
        transform="rotate(-90 {CENTER} {CENTER})"
        style="stroke: {r.color};"
      />
    {/each}
    <text x={CENTER} y={CENTER - 6} class="kcal-num" text-anchor="middle">{round(totals.calories)}</text>
    <text x={CENTER} y={CENTER + 14} class="kcal-of" text-anchor="middle">/ {round(targets.calories)} kcal</text>
  </svg>

  <ul class="legend">
    {#each rings as r}
      <li>
        <span class="dot" style="background: {r.color};"></span>
        <span class="leg-label">{r.label}</span>
        <span class="leg-val">{round(r.consumed)} / {round(r.target)} {r.unit}</span>
        <span class="leg-pct">{Math.round(pct(r) * 100)}%</span>
      </li>
    {/each}
  </ul>
</div>

<style>
  .rings {
    display: flex;
    align-items: center;
    gap: 24px;
    flex-wrap: wrap;
    justify-content: center;
  }
  .dial { width: 200px; height: 200px; flex: none; }
  .track { fill: none; opacity: 0.14; }
  .value {
    fill: none;
    stroke-linecap: round;
    transition: stroke-dasharray 0.5s ease;
  }
  .kcal-num { fill: var(--ink); font-family: var(--font-mono); font-size: 1.5rem; }
  .kcal-of { fill: var(--ink-mute); font-family: var(--font-mono); font-size: 0.62rem; letter-spacing: 0.06em; }

  .legend { list-style: none; margin: 0; padding: 0; min-width: 200px; flex: 1; }
  .legend li {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 8px 10px;
    padding: 7px 0;
    border-bottom: 1px solid var(--rule-soft);
    font-family: var(--font-mono);
    font-size: 0.78rem;
  }
  .legend li:last-child { border-bottom: none; }
  .dot { width: 10px; height: 10px; border-radius: 50%; display: inline-block; }
  .leg-label { color: var(--ink-soft); }
  .leg-val { color: var(--ink); text-align: right; }
  .leg-pct {
    grid-column: 2 / 4;
    color: var(--ink-mute);
    font-size: 0.66rem;
    text-align: right;
  }

  @media (max-width: 520px) {
    .rings { flex-direction: column; gap: 16px; }
    .legend { width: 100%; }
  }
</style>
