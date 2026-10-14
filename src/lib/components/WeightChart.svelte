<script lang="ts">
  /* ─────────────────────────────────────────────────────────
   * WEIGHT CHART — daily weigh-ins, smoothed trend, forecast
   *
   * Three layers, in reading order:
   *   · faint dots        — every logged weigh-in, noise included
   *   · solid line + fill — the time-aware EMA, the trend weight
   *   · dashed ray        — the least-squares fit for the visible
   *                         range, extended two weeks past today
   *
   * A range switch reframes all three. Hovering (or arrowing
   * through) the plot reads out the weigh-in nearest the cursor.
   * Colours come from the theme tokens, so it tracks light/dark.
   * ───────────────────────────────────────────────────────── */

  import { linearFit, withinDays, type DailyPoint } from '$lib/fitnessInsights';

  export let points: DailyPoint[] = [];
  export let ema: DailyPoint[] = [];
  /** Optional reference line (e.g. the BMI 25 weight), drawn when it's in view. */
  export let targetKg: number | null = null;
  export let targetLabel = 'target';

  const W = 640;
  const H = 260;
  const PAD = { top: 16, right: 16, bottom: 28, left: 46 };
  /** Days of fitted trend drawn past the last weigh-in. */
  const FORECAST_DAYS = 14;

  const RANGES = [
    { label: '30d', days: 30 },
    { label: '90d', days: 90 },
    { label: '6m', days: 182 },
    { label: 'all', days: 0 }
  ];
  let range = 90;

  // Only offer a range once there's history that reaches past the shorter one,
  // so the switch never shows four buttons that all draw the same line.
  $: span = points.length > 1 ? points[points.length - 1].t - points[0].t : 0;
  $: ranges = RANGES.filter((r, i) => r.days === 0 || i === 0 || span > RANGES[i - 1].days);
  // A short log can drop the default 90d button; fall back to `all` so the
  // switch always shows which range is active.
  $: if (!ranges.some((r) => r.days === range)) range = 0;

  $: view = withinDays(points, range);
  $: viewEma = range <= 0 ? ema : ema.filter((p) => view.length > 0 && p.t >= view[0].t);
  $: fit = linearFit(view);

  const fmtDay = (iso: string) =>
    new Date(iso + 'T00:00:00').toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });

  /** Smallest step from a round set that gives at most `maxTicks` gridlines. */
  function niceStep(span: number, maxTicks = 5): number {
    const steps = [0.2, 0.25, 0.5, 1, 2, 2.5, 5, 10];
    return steps.find((st) => span / st <= maxTicks - 1) ?? 20;
  }

  // Scales. The y domain pads the data by 0.4 kg so the line never rides the
  // frame, and stretches to take in the forecast and the target line when that
  // one is close enough not to flatten everything else.
  $: geo = (() => {
    if (view.length === 0) return null;
    const t0 = view[0].t;
    const tLast = view[view.length - 1].t;
    const t1 = tLast + (fit ? FORECAST_DAYS : 0);
    const tSpan = Math.max(1, t1 - t0);

    const vals = [...view.map((p) => p.kg), ...viewEma.map((p) => p.kg)];
    if (fit) vals.push(fit.slope * t1 + fit.intercept);
    let lo = Math.min(...vals) - 0.2;
    let hi = Math.max(...vals) + 0.2;
    // The goal line only earns space when it is close; dragging the domain out
    // to a target several kg away would flatten the weigh-ins into a smear.
    const showTarget = targetKg !== null && targetKg > lo - 1.5 && targetKg < hi + 1.5;
    if (showTarget && targetKg !== null) {
      lo = Math.min(lo, targetKg - 0.2);
      hi = Math.max(hi, targetKg + 0.2);
    }
    // Snap the domain out to round kg so the axis reads 84, 85, 86 rather than
    // 84.3, 86.6 — the gridlines are there to be compared against.
    const step = niceStep(Math.max(0.8, hi - lo));
    lo = Math.floor(lo / step) * step;
    hi = Math.ceil(hi / step) * step;
    const kgSpan = hi - lo;
    const dp = step < 1 ? 1 : 0;

    const x = (t: number) =>
      PAD.left + ((t - t0) / tSpan) * (W - PAD.left - PAD.right);
    const y = (kg: number) =>
      PAD.top + (1 - (kg - lo) / kgSpan) * (H - PAD.top - PAD.bottom);

    const ticks = Array.from({ length: Math.round(kgSpan / step) + 1 }, (_, i) => {
      const kg = lo + step * i;
      return { kg, y: y(kg), label: kg.toFixed(dp) };
    });

    const dateTicks = (() => {
      const n = Math.min(4, view.length);
      if (n < 2) return [{ t: t0, x: x(t0), label: fmtDay(view[0].date) }];
      return Array.from({ length: n }, (_, i) => {
        const p = view[Math.round((i / (n - 1)) * (view.length - 1))];
        return { t: p.t, x: x(p.t), label: fmtDay(p.date) };
      });
    })();

    const line = viewEma.map((p) => `${x(p.t).toFixed(1)},${y(p.kg).toFixed(1)}`).join(' ');
    const baseY = (H - PAD.bottom).toFixed(1);
    const area = viewEma.length
      ? `${x(viewEma[0].t).toFixed(1)},${baseY} ${line} ${x(viewEma[viewEma.length - 1].t).toFixed(1)},${baseY}`
      : '';

    const trend = fit
      ? {
          x1: x(t0),
          y1: y(fit.slope * t0 + fit.intercept),
          x2: x(t1),
          y2: y(fit.slope * t1 + fit.intercept),
          nowX: x(tLast)
        }
      : null;

    return {
      x,
      y,
      lo,
      hi,
      ticks,
      dateTicks,
      line,
      area,
      trend,
      showTarget,
      dots: view.map((p) => ({ ...p, cx: x(p.t), cy: y(p.kg) }))
    };
  })();

  // ---- hover readout -------------------------------------------------------

  let plot: SVGSVGElement;
  let hoverIdx: number | null = null;

  function onMove(e: PointerEvent) {
    if (!geo || view.length === 0) return;
    const rect = plot.getBoundingClientRect();
    if (rect.width === 0) return;
    const vx = ((e.clientX - rect.left) / rect.width) * W;
    let best = 0;
    let bestD = Infinity;
    geo.dots.forEach((d, i) => {
      const dist = Math.abs(d.cx - vx);
      if (dist < bestD) {
        bestD = dist;
        best = i;
      }
    });
    hoverIdx = best;
  }

  $: hover = geo && hoverIdx !== null ? geo.dots[hoverIdx] ?? null : null;
  // Flip the label to the left of the guide near the right edge.
  $: hoverFlip = hover ? hover.cx > W - 110 : false;
</script>

<div class="chart">
  <div class="chart-head">
    <span class="chart-title">Weigh-ins &amp; trend</span>
    <div class="range-switch" role="group" aria-label="Chart range">
      {#each ranges as r}
        <button
          type="button"
          class:on={range === r.days}
          on:click={() => (range = r.days)}
          aria-pressed={range === r.days}>{r.label}</button
        >
      {/each}
    </div>
  </div>

  {#if geo && view.length > 1}
    <svg
      bind:this={plot}
      class="plot"
      viewBox="0 0 {W} {H}"
      role="img"
      aria-label="Bodyweight over time with a smoothed trend line and a two-week forecast"
      on:pointermove={onMove}
      on:pointerleave={() => (hoverIdx = null)}
    >
      {#each geo.ticks as t}
        <line class="gridline" x1={PAD.left} y1={t.y} x2={W - PAD.right} y2={t.y} />
        <text class="axis y" x={PAD.left - 8} y={t.y + 3.5}>{t.label}</text>
      {/each}

      {#each geo.dateTicks as t}
        <text class="axis x" x={t.x} y={H - PAD.bottom + 16}>{t.label}</text>
      {/each}

      {#if geo.showTarget && targetKg !== null}
        <line class="target" x1={PAD.left} y1={geo.y(targetKg)} x2={W - PAD.right} y2={geo.y(targetKg)} />
        <text class="target-label" x={W - PAD.right} y={geo.y(targetKg) - 5}>{targetLabel}</text>
      {/if}

      {#if geo.area}<polygon class="area" points={geo.area} />{/if}

      {#if geo.trend}
        <line class="trend" x1={geo.trend.x1} y1={geo.trend.y1} x2={geo.trend.x2} y2={geo.trend.y2} />
        <line class="now" x1={geo.trend.nowX} y1={PAD.top} x2={geo.trend.nowX} y2={H - PAD.bottom} />
      {/if}

      <polyline class="ema" points={geo.line} />

      {#each geo.dots as d}
        <circle class="pt" cx={d.cx} cy={d.cy} r="2.5" />
      {/each}

      {#if hover}
        <line class="guide" x1={hover.cx} y1={PAD.top} x2={hover.cx} y2={H - PAD.bottom} />
        <circle class="pt hot" cx={hover.cx} cy={hover.cy} r="4" />
        <text
          class="readout"
          x={hoverFlip ? hover.cx - 8 : hover.cx + 8}
          y={Math.max(PAD.top + 10, hover.cy - 10)}
          text-anchor={hoverFlip ? 'end' : 'start'}
        >
          {hover.kg.toFixed(1)} kg · {fmtDay(hover.date)}
        </text>
      {/if}
    </svg>

    <p class="legend">
      <span class="key dot-key"></span> weigh-in
      <span class="key line-key"></span> trend (EMA)
      <span class="key trend-key"></span> {range === 0 ? 'all-time' : `${range}-day`} fit,
      forecast {FORECAST_DAYS} days
    </p>
  {:else}
    <p class="empty">
      {points.length === 1
        ? 'One weigh-in logged — add a few more days and the trend line appears here.'
        : 'No weigh-ins yet. Log a bodyweight above to start the chart.'}
    </p>
  {/if}
</div>


<style>
  /* Series follow DESIGN.md: trend fjord (1), goal line moss (2), fitted
     forecast clay (3), raw weigh-ins ink-3 (4). Grid on --line, axis text in
     mono --ink-3. Every colour is a CSS variable, so both themes work. */
  .chart { display: flex; flex-direction: column; gap: var(--s-3); }

  .chart-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2) var(--s-3);
  }
  .chart-title {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .range-switch {
    display: flex;
    gap: 2px;
    margin-left: auto;
    padding: 2px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
  }
  .range-switch button {
    min-width: 2.5rem;
    min-height: 32px;
    padding: 0 var(--s-2);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    appearance: none;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }
  .range-switch button:hover { background: var(--hover-wash); color: var(--ink); }
  .range-switch button.on {
    background: var(--fjord-wash);
    color: var(--fjord);
    font-weight: 500;
  }

  .plot {
    display: block;
    width: 100%;
    height: auto;
    touch-action: pan-y;
  }

  .gridline { stroke: var(--line); stroke-width: 1; vector-effect: non-scaling-stroke; }
  .axis {
    fill: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 9px;
    font-variant-numeric: tabular-nums;
  }
  .axis.y { text-anchor: end; }
  .axis.x { text-anchor: middle; }

  /* A flat wash under the trend, no gradient. */
  .area { fill: var(--fjord); fill-opacity: 0.07; stroke: none; }

  .ema {
    fill: none;
    stroke: var(--fjord);
    stroke-width: 2;
    stroke-linejoin: round;
    stroke-linecap: round;
    vector-effect: non-scaling-stroke;
  }
  .pt { fill: var(--ink-3); opacity: 0.55; }
  .pt.hot { fill: var(--fjord); opacity: 1; }

  .trend {
    stroke: var(--clay);
    stroke-width: 1.5;
    stroke-dasharray: 5 4;
    vector-effect: non-scaling-stroke;
  }
  .now { stroke: var(--line-strong); stroke-width: 1; vector-effect: non-scaling-stroke; }

  .target {
    stroke: var(--moss);
    stroke-width: 1;
    stroke-dasharray: 2 3;
    vector-effect: non-scaling-stroke;
  }
  .target-label {
    fill: var(--moss);
    font-family: var(--font-mono);
    font-size: 9px;
    text-anchor: end;
  }

  .guide { stroke: var(--line-strong); stroke-width: 1; vector-effect: non-scaling-stroke; }
  .readout {
    fill: var(--ink);
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    paint-order: stroke;
    stroke: var(--surface);
    stroke-width: 3px;
    stroke-linejoin: round;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px var(--s-2);
    margin: 0;
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.5;
  }
  .key { display: inline-block; flex: none; }
  .dot-key { width: 7px; height: 7px; border-radius: 50%; background: var(--ink-3); opacity: 0.7; }
  .line-key { width: 16px; height: 2px; margin-left: var(--s-2); background: var(--fjord); }
  .trend-key {
    width: 16px;
    height: 0;
    margin-left: var(--s-2);
    border-top: 2px dashed var(--clay);
  }

  .empty { margin: 0; color: var(--ink-3); font-size: var(--text-sm); }

  @media (max-width: 560px) {
    .axis { font-size: 11px; }
    .readout { font-size: 12px; }
    .target-label { font-size: 11px; }
    .range-switch button { min-height: 36px; }
  }
</style>
