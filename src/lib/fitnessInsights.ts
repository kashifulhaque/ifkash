// Pure trend analysis for the fitness pages: bodyweight rates, smoothing,
// projections and training cadence. I/O-free and free of Svelte imports, like
// `fitnessMetrics.ts` — the API wrappers stay in `workoutApi.ts`.
//
// Two different smoothers are used deliberately:
//   - a time-aware EMA gives the "trend weight" a single number can be read
//     off, and it survives the irregular logging a real week produces;
//   - a least-squares fit over a fixed window gives the *rate*, because a rate
//     read off two endpoints is mostly water weight.

import type { BodyweightEntry, WeeklyAverage } from '$lib/workout';
import { goalDeficit, type Profile } from '$lib/fitnessMetrics';

/** Energy in a kilogram of body tissue lost on a cut (~mostly fat). */
export const KCAL_PER_KG = 7700;

const MS_PER_DAY = 86_400_000;

/** A logged day, `t` in whole days so gaps in logging stay honest. */
export type DailyPoint = { date: string; kg: number; t: number };

function toDays(iso: string): number {
  return Math.round(Date.parse(iso + 'T00:00:00Z') / MS_PER_DAY);
}

function toISO(days: number): string {
  return new Date(days * MS_PER_DAY).toISOString().slice(0, 10);
}

function round(v: number, dp = 1): number {
  const f = 10 ** dp;
  return Math.round(v * f) / f;
}

/**
 * Entries → one ascending point per calendar day. Several weigh-ins on one day
 * are averaged; undated or non-positive rows are dropped.
 */
export function dailyPoints(entries: BodyweightEntry[]): DailyPoint[] {
  const byDate = new Map<string, { sum: number; n: number }>();
  for (const e of entries) {
    if (!e.date || !(e.weight_g > 0)) continue;
    const b = byDate.get(e.date) ?? { sum: 0, n: 0 };
    b.sum += e.weight_g;
    b.n += 1;
    byDate.set(e.date, b);
  }
  return [...byDate.entries()]
    .sort((a, b) => (a[0] < b[0] ? -1 : 1))
    .map(([date, b]) => ({ date, kg: b.sum / b.n / 1000, t: toDays(date) }));
}

/** Points within `days` of the most recent one (`days <= 0` means all of them). */
export function withinDays(points: DailyPoint[], days: number): DailyPoint[] {
  if (days <= 0 || points.length === 0) return points;
  const cutoff = points[points.length - 1].t - days;
  return points.filter((p) => p.t >= cutoff);
}

// ---- trend fitting ---------------------------------------------------------

export type TrendFit = {
  /** kg per day; negative while losing. */
  slope: number;
  /** kg at `t = 0` (days since the Unix epoch), so `slope * t + intercept`. */
  intercept: number;
  /** Share of the variance the line explains, 0–1. Low = noisy or flat. */
  r2: number;
  n: number;
  spanDays: number;
};

/**
 * Least-squares fit of kg against day. Needs two points spread over at least
 * one day; a same-day cluster has no slope to find and returns null.
 */
export function linearFit(points: DailyPoint[]): TrendFit | null {
  const n = points.length;
  if (n < 2) return null;
  const meanT = points.reduce((s, p) => s + p.t, 0) / n;
  const meanKg = points.reduce((s, p) => s + p.kg, 0) / n;
  let num = 0;
  let den = 0;
  for (const p of points) {
    num += (p.t - meanT) * (p.kg - meanKg);
    den += (p.t - meanT) ** 2;
  }
  if (den === 0) return null;
  const slope = num / den;
  const intercept = meanKg - slope * meanT;
  let ssRes = 0;
  let ssTot = 0;
  for (const p of points) {
    ssRes += (p.kg - (slope * p.t + intercept)) ** 2;
    ssTot += (p.kg - meanKg) ** 2;
  }
  return {
    slope,
    intercept,
    r2: ssTot === 0 ? 0 : Math.max(0, 1 - ssRes / ssTot),
    n,
    spanDays: points[n - 1].t - points[0].t
  };
}

/** Root-mean-square distance of the weigh-ins from the fitted line, in kg. */
export function residualSd(points: DailyPoint[], fit: TrendFit | null): number | null {
  if (!fit || points.length < 3) return null;
  const ss = points.reduce((s, p) => s + (p.kg - (fit.slope * p.t + fit.intercept)) ** 2, 0);
  return round(Math.sqrt(ss / (points.length - 2)), 2);
}

/**
 * Time-aware exponential moving average: each step decays by the actual number
 * of days elapsed, so a five-day gap doesn't get the weight of a daily entry.
 */
export function emaSeries(points: DailyPoint[], halfLifeDays = 7): DailyPoint[] {
  if (points.length === 0) return [];
  const out: DailyPoint[] = [{ ...points[0] }];
  for (let i = 1; i < points.length; i++) {
    const dt = Math.max(1, points[i].t - points[i - 1].t);
    const alpha = 1 - 0.5 ** (dt / halfLifeDays);
    const prev = out[i - 1].kg;
    out.push({ ...points[i], kg: prev + alpha * (points[i].kg - prev) });
  }
  return out;
}

// ---- rate over a window ----------------------------------------------------

export type RateBand = 'gaining' | 'holding' | 'slow' | 'sustainable' | 'aggressive' | 'very fast';

/**
 * Where a loss rate sits relative to the usual cut guidance: 0.5–1.0 % of
 * bodyweight per week keeps muscle, past 1.25 % starts costing it.
 */
export function rateBand(pctPerWeek: number | null): RateBand | null {
  if (pctPerWeek === null) return null;
  if (pctPerWeek > 0.1) return 'gaining';
  const loss = -pctPerWeek;
  if (loss < 0.1) return 'holding';
  if (loss < 0.5) return 'slow';
  if (loss <= 1.0) return 'sustainable';
  if (loss <= 1.25) return 'aggressive';
  return 'very fast';
}

export type Rate = {
  label: string;
  windowDays: number;
  kgPerWeek: number | null;
  /** Percent of current bodyweight per week; negative while losing. */
  pctPerWeek: number | null;
  /** Energy balance the rate implies, kcal/day; positive means a deficit. */
  kcalPerDay: number | null;
  band: RateBand | null;
  r2: number | null;
  n: number;
  spanDays: number;
};

/**
 * Loss rate over the last `windowDays`. `null` rates mean not enough spread to
 * fit a line yet — the UI shows a dash rather than a made-up number.
 */
export function rateOver(points: DailyPoint[], windowDays: number, label: string): Rate {
  const win = withinDays(points, windowDays);
  const fit = linearFit(win);
  const current = win.length ? win[win.length - 1].kg : 0;
  if (!fit || current <= 0) {
    return {
      label,
      windowDays,
      kgPerWeek: null,
      pctPerWeek: null,
      kcalPerDay: null,
      band: null,
      r2: null,
      n: win.length,
      spanDays: win.length ? win[win.length - 1].t - win[0].t : 0
    };
  }
  const kgPerWeek = fit.slope * 7;
  const pctPerWeek = (kgPerWeek / current) * 100;
  return {
    label,
    windowDays,
    kgPerWeek: round(kgPerWeek, 2),
    pctPerWeek: round(pctPerWeek, 2),
    // kg/day × kcal/kg = kcal/day. A negative slope (losing) is a positive deficit.
    kcalPerDay: Math.round(-fit.slope * KCAL_PER_KG),
    band: rateBand(round(pctPerWeek, 2)),
    r2: round(fit.r2, 2),
    n: fit.n,
    spanDays: fit.spanDays
  };
}

// ---- projections -----------------------------------------------------------

export type Projection = {
  label: string;
  targetKg: number;
  /** Days from the last weigh-in; null when the trend never gets there. */
  days: number | null;
  weeks: number | null;
  date: string | null;
  /** True once the target is already met. */
  reached: boolean;
};

/** Bodyweight (kg) that puts a given height at a given BMI. */
export function weightForBmi(bmi: number, heightCm: number): number {
  const m = heightCm / 100;
  return round(bmi * m * m, 1);
}

/**
 * When a downward trend reaches `targetKg`, extrapolating the fitted slope from
 * the last weigh-in. `reached` means the weight is already at or under the
 * target; `days: null` means the current trend never gets there (flat, rising,
 * or so slow the date would be years out and say nothing useful).
 */
export function projectTo(
  label: string,
  targetKg: number,
  fromKg: number,
  fromDate: string,
  kgPerDay: number
): Projection {
  const base: Projection = { label, targetKg, days: null, weeks: null, date: null, reached: false };
  if (fromKg <= targetKg + 0.05) {
    return { ...base, reached: true, days: 0, weeks: 0, date: fromDate };
  }
  if (!Number.isFinite(kgPerDay) || kgPerDay >= 0) return base;
  const days = (targetKg - fromKg) / kgPerDay;
  if (days <= 0 || days > 1826) return base;
  return {
    label,
    targetKg,
    days: Math.round(days),
    weeks: round(days / 7, 1),
    date: toISO(toDays(fromDate) + Math.round(days)),
    reached: false
  };
}

/** Weight the trend lands on `weeks` from the last weigh-in. */
export function projectForward(fromKg: number, kgPerDay: number, weeks: number): number {
  return round(fromKg + kgPerDay * weeks * 7, 1);
}

// ---- logging consistency ---------------------------------------------------

export type Consistency = {
  entries: number;
  firstDate: string | null;
  lastDate: string | null;
  spanDays: number;
  /** Weigh-ins in the last 30 days, and that as a share of the days. */
  last30: number;
  coverage30: number;
  /** Consecutive days logged up to the latest entry. */
  streak: number;
  daysSinceLast: number | null;
  longestGap: number;
};

export function consistency(points: DailyPoint[], today?: string): Consistency {
  if (points.length === 0) {
    return {
      entries: 0,
      firstDate: null,
      lastDate: null,
      spanDays: 0,
      last30: 0,
      coverage30: 0,
      streak: 0,
      daysSinceLast: null,
      longestGap: 0
    };
  }
  const first = points[0];
  const last = points[points.length - 1];
  const nowT = today ? toDays(today) : last.t;

  let streak = 1;
  for (let i = points.length - 1; i > 0; i--) {
    if (points[i].t - points[i - 1].t === 1) streak++;
    else break;
  }

  let longestGap = 0;
  for (let i = 1; i < points.length; i++) {
    longestGap = Math.max(longestGap, points[i].t - points[i - 1].t);
  }

  const last30 = points.filter((p) => p.t > nowT - 30).length;
  const window = Math.min(30, Math.max(1, nowT - first.t + 1));
  return {
    entries: points.length,
    firstDate: first.date,
    lastDate: last.date,
    spanDays: last.t - first.t,
    last30,
    coverage30: Math.round((last30 / window) * 100),
    streak,
    daysSinceLast: Math.max(0, nowT - last.t),
    longestGap
  };
}

// ---- weekly extremes -------------------------------------------------------

export type WeekChange = { weekStart: string; delta: number };

/** Best (largest drop) and worst (largest gain) week-over-week change. */
export function weeklyExtremes(weekly: WeeklyAverage[]): {
  best: WeekChange | null;
  worst: WeekChange | null;
} {
  let best: WeekChange | null = null;
  let worst: WeekChange | null = null;
  for (let i = 1; i < weekly.length; i++) {
    const delta = round(weekly[i].avgKg - weekly[i - 1].avgKg, 1);
    const c = { weekStart: weekly[i].weekStart, delta };
    if (!best || delta < best.delta) best = c;
    if (!worst || delta > worst.delta) worst = c;
  }
  return { best, worst };
}

// ---- the whole picture -----------------------------------------------------

export type WeightInsights = {
  points: DailyPoint[];
  ema: DailyPoint[];
  /** EMA of the latest weigh-in — the number to read instead of the scale. */
  trendKg: number | null;
  latestKg: number | null;
  latestDate: string | null;
  startKg: number | null;
  totalChange: number | null;
  /** Total change as a percent of the starting weight. */
  totalChangePct: number | null;
  /** Fits over 14 / 28 / 90 days plus the whole log. */
  rates: Rate[];
  /** The 28-day fit — the one headline numbers and projections are based on. */
  primary: Rate;
  fit: TrendFit | null;
  volatility: number | null;
  /** Deficit the goal asks for, and how the measured rate compares (kcal/day). */
  plannedDeficit: number;
  deficitGap: number | null;
  /** A flat recent trend inside a log that was falling — worth naming. */
  stalled: boolean;
  projections: Projection[];
  forecast: { weeks: number; kg: number }[];
  consistency: Consistency;
  extremes: { best: WeekChange | null; worst: WeekChange | null };
};

/**
 * Everything the trend panel renders, from the raw entries plus the body
 * profile. `weekly` comes from `weeklyAverages()` so the table and these
 * insights agree; `today` dates the projections and the logging streak.
 */
export function weightInsights(
  entries: BodyweightEntry[],
  profile: Profile,
  weekly: WeeklyAverage[],
  today?: string
): WeightInsights {
  const points = dailyPoints(entries);
  const ema = emaSeries(points);
  // Volatility is read off the recent stretch — a year-old cut says nothing
  // about how much today's weigh-in can bounce.
  const recentPoints = withinDays(points, 60);
  const last = points.length ? points[points.length - 1] : null;
  const first = points.length ? points[0] : null;
  const trendKg = ema.length ? round(ema[ema.length - 1].kg, 1) : null;

  const rates = [
    rateOver(points, 14, 'last 2 weeks'),
    rateOver(points, 28, 'last 4 weeks'),
    rateOver(points, 90, 'last 3 months'),
    rateOver(points, 0, 'all time')
  ];
  // The 4-week fit is long enough to outrun water weight and short enough to
  // describe what is happening now. Fall back to the whole log while it's young.
  const primary = rates[1].kgPerWeek !== null ? rates[1] : rates[3];
  const fit = linearFit(points);

  const plannedDeficit = goalDeficit(profile.goal);
  const deficitGap = primary.kcalPerDay === null ? null : primary.kcalPerDay - plannedDeficit;

  const recent = rates[0].kgPerWeek;
  const longRun = rates[3].kgPerWeek;
  const stalled =
    recent !== null && longRun !== null && Math.abs(recent) < 0.1 && longRun < -0.1;

  // Projections run off the primary rate; targets are BMI-derived so they need
  // no extra state (the profile has no goal-weight field).
  const kgPerDay = primary.kgPerWeek === null ? 0 : primary.kgPerWeek / 7;
  const fromKg = last?.kg ?? 0;
  const fromDate = last?.date ?? today ?? toISO(Math.floor(Date.now() / MS_PER_DAY));
  const projections = last
    ? [
        projectTo('BMI 25 — normal range', weightForBmi(24.9, profile.height_cm), fromKg, fromDate, kgPerDay),
        projectTo('BMI 23', weightForBmi(23, profile.height_cm), fromKg, fromDate, kgPerDay),
        projectTo('BMI 22', weightForBmi(22, profile.height_cm), fromKg, fromDate, kgPerDay)
      ]
    : [];

  const forecast = last && primary.kgPerWeek !== null
    ? [4, 8, 12, 26].map((weeks) => ({ weeks, kg: projectForward(fromKg, kgPerDay, weeks) }))
    : [];

  return {
    points,
    ema,
    trendKg,
    latestKg: last ? round(last.kg, 1) : null,
    latestDate: last?.date ?? null,
    startKg: first ? round(first.kg, 1) : null,
    totalChange: first && last ? round(last.kg - first.kg, 1) : null,
    totalChangePct: first && last && first.kg > 0 ? round(((last.kg - first.kg) / first.kg) * 100, 1) : null,
    rates,
    primary,
    fit,
    volatility: residualSd(recentPoints, linearFit(recentPoints)),
    plannedDeficit,
    deficitGap,
    stalled,
    projections,
    forecast,
    consistency: consistency(points, today),
    extremes: weeklyExtremes(weekly)
  };
}

// ---- training cadence ------------------------------------------------------

export type TrainingCadence = {
  total: number;
  last7: number;
  last28: number;
  /** Sessions per week averaged over the last 28 days. */
  perWeek: number;
  /** Push / Pull / Legs counts over the last 28 days. */
  focus: { label: string; count: number }[];
  daysSinceLast: number | null;
  /** Consecutive days trained up to the most recent session. */
  streak: number;
  longestGap: number;
};

/**
 * Training frequency and focus balance from the session list alone (no per-set
 * detail needed, so no extra requests). Sessions arrive newest-first.
 */
export function trainingCadence(
  sessions: { date: string; day_label: string }[],
  today?: string
): TrainingCadence {
  const dates = [...new Set(sessions.map((s) => s.date).filter(Boolean))]
    .sort()
    .map(toDays);
  const nowT = today ? toDays(today) : dates.length ? dates[dates.length - 1] : 0;

  const inWindow = (iso: string, days: number) => toDays(iso) > nowT - days;
  const last28 = sessions.filter((s) => inWindow(s.date, 28));

  const counts = new Map<string, number>();
  for (const s of last28) counts.set(s.day_label, (counts.get(s.day_label) ?? 0) + 1);

  let streak = dates.length ? 1 : 0;
  for (let i = dates.length - 1; i > 0; i--) {
    if (dates[i] - dates[i - 1] === 1) streak++;
    else break;
  }
  let longestGap = 0;
  for (let i = 1; i < dates.length; i++) longestGap = Math.max(longestGap, dates[i] - dates[i - 1]);

  return {
    total: sessions.length,
    last7: sessions.filter((s) => inWindow(s.date, 7)).length,
    last28: last28.length,
    perWeek: round(last28.length / 4, 1),
    focus: ['Push', 'Pull', 'Legs'].map((label) => ({ label, count: counts.get(label) ?? 0 })),
    daysSinceLast: dates.length ? Math.max(0, nowT - dates[dates.length - 1]) : null,
    streak,
    longestGap
  };
}
