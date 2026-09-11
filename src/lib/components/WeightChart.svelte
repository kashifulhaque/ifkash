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
      <defs>
        <linearGradient id="wc-fill" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" class="fill-top" />
          <stop offset="1" class="fill-bottom" />
        </linearGradient>
      </defs>

      {#each geo.ticks as t}
        <line class="grid" x1={PAD.left} y1={t.y} x2={W - PAD.right} y2={t.y} />
        <text class="axis y" x={PAD.left - 8} y={t.y + 3.5}>{t.label}</text>
      {/each}

      {#each geo.dateTicks as t}
        <text class="axis x" x={t.x} y={H - PAD.bottom + 16}>{t.label}</text>
      {/each}

      {#if geo.showTarget && targetKg !== null}
        <line class="target" x1={PAD.left} y1={geo.y(targetKg)} x2={W - PAD.right} y2={geo.y(targetKg)} />
        <text class="target-label" x={W - PAD.right} y={geo.y(targetKg) - 5}>{targetLabel}</text>
      {/if}

      {#if geo.area}<polygon class="area" points={geo.area} fill="url(#wc-fill)" />{/if}

      {#if geo.trend}
        <line class="trend" x1={geo.trend.x1} y1={geo.trend.y1} x2={geo.trend.x2} y2={geo.trend.y2} />
        <line class="now" x1={geo.trend.nowX} y1={PAD.top} x2={geo.trend.nowX} y2={H - PAD.bottom} />
      {/if}

      <polyline class="ema" points={geo.line} />

      {#each geo.dots as d}
        <circle class="dot" cx={d.cx} cy={d.cy} r="2.5" />
      {/each}

      {#if hover}
        <line class="guide" x1={hover.cx} y1={PAD.top} x2={hover.cx} y2={H - PAD.bottom} />
        <circle class="dot hot" cx={hover.cx} cy={hover.cy} r="4" />
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
  .chart { display: flex; flex-direction: column; gap: 0.6rem; }

  .chart-head { display: flex; align-items: center; gap: 0.75rem; }
  .chart-title {
    font-family: var(--font-mono);
    font-size: 0.7rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
  }
  .range-switch {
    margin-left: auto;
    display: flex;
    gap: 0.15rem;
    padding: 0.15rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.4rem;
  }
  .range-switch button {
    appearance: none;
    border: none;
    background: none;
    padding: 0.2rem 0.5rem;
    border-radius: 0.3rem;
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--text-tertiary);
    cursor: var(--cursor-pointer);
  }
  .range-switch button:hover { color: var(--text-primary); background: var(--bg-surface-hover); }
  .range-switch button.on {
    color: var(--text-primary);
    background: var(--blueprint-tint);
    border: 1px solid var(--blueprint);
    padding: calc(0.2rem - 1px) calc(0.5rem - 1px);
  }

  .plot {
    display: block;
    width: 100%;
    height: auto;
    touch-action: pan-y;
  }

  .grid { stroke: var(--border-subtle); stroke-width: 1; vector-effect: non-scaling-stroke; }
  .axis {
    font-family: var(--font-mono);
    font-size: 9px;
    fill: var(--text-faint);
  }
  .axis.y { text-anchor: end; }
  .axis.x { text-anchor: middle; }

  .area { stroke: none; }
  .fill-top { stop-color: var(--blueprint); stop-opacity: 0.22; }
  .fill-bottom { stop-color: var(--blueprint); stop-opacity: 0; }

  .ema {
    fill: none;
    stroke: var(--blueprint);
    stroke-width: 2;
    stroke-linejoin: round;
    stroke-linecap: round;
    vector-effect: non-scaling-stroke;
  }
  .dot { fill: var(--blueprint); opacity: 0.4; }
  .dot.hot { fill: var(--blueprint); opacity: 1; }

  .trend {
    stroke: var(--text-secondary);
    stroke-width: 1.5;
    stroke-dasharray: 5 4;
    opacity: 0.75;
    vector-effect: non-scaling-stroke;
  }
  .now { stroke: var(--border); stroke-width: 1; vector-effect: non-scaling-stroke; }

  .target {
    stroke: var(--ok, #44d288);
    stroke-width: 1;
    stroke-dasharray: 2 3;
    vector-effect: non-scaling-stroke;
  }
  .target-label {
    font-family: var(--font-mono);
    font-size: 9px;
    fill: var(--ok, #44d288);
    text-anchor: end;
  }

  .guide { stroke: var(--border-strong); stroke-width: 1; vector-effect: non-scaling-stroke; }
  .readout {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    fill: var(--text-primary);
    paint-order: stroke;
    stroke: var(--bg);
    stroke-width: 3px;
    stroke-linejoin: round;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem 0.5rem;
    margin: 0;
    font-size: 0.7rem;
    color: var(--text-tertiary);
  }
  .key { display: inline-block; flex: none; }
  .dot-key { width: 7px; height: 7px; border-radius: 50%; background: var(--blueprint); opacity: 0.5; }
  .line-key { width: 16px; height: 2px; background: var(--blueprint); }
  .trend-key {
    width: 16px;
    height: 0;
    border-top: 2px dashed var(--text-secondary);
    margin-left: 0.35rem;
  }

  .empty { margin: 0; font-size: 0.8125rem; color: var(--text-tertiary); }

  @media (max-width: 560px) {
    .axis { font-size: 11px; }
    .readout { font-size: 12px; }
    .target-label { font-size: 11px; }
  }
</style>
