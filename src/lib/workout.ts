// Shared workout data + helpers. The day templates live here so the static plan
// page (/workout) and the tracker (/workout/log) read from one source of truth.

import { strengthKcal, metKcal } from './fitnessMetrics';

// `kind` drives how the set-row UI renders:
//  - 'weighted'   → weight × reps inputs (e.g. bench press)
//  - 'bodyweight' → reps only; weight hidden (e.g. crunches)
//  - 'time'       → seconds only; weight hidden (e.g. plank hold)
// The persisted shape is unchanged (weight_g + reps); for time exercises the
// seconds go in the `reps` column and weight_g stays 0.
export type ExerciseKind = 'weighted' | 'bodyweight' | 'time';
export type Exercise = { name: string; scheme: string; kind: ExerciseKind };

export const PUSH: Exercise[] = [
  { name: 'Bench press', scheme: '4×8', kind: 'weighted' },
  { name: 'Shoulder press', scheme: '3×10', kind: 'weighted' },
  { name: 'Incline dumbbell press', scheme: '3×10', kind: 'weighted' },
  { name: 'Pec fly', scheme: '3×12', kind: 'weighted' },
  { name: 'Cable tricep pushdown', scheme: '3×12', kind: 'weighted' }
];

export const PULL: Exercise[] = [
  { name: 'Lat pulldown', scheme: '3×10', kind: 'weighted' },
  { name: 'Row machine', scheme: '3×10', kind: 'weighted' },
  { name: 'Deadlift', scheme: '50×5 60×3 80×2 90×1 100×1', kind: 'weighted' },
  { name: 'Face pulls', scheme: '3×12', kind: 'weighted' },
  { name: 'Rear delts', scheme: '3×15', kind: 'weighted' },
  { name: 'Bicep curls', scheme: '3×12', kind: 'weighted' }
];

export const LEGS: Exercise[] = [
  { name: 'Barbell squat', scheme: '4×8', kind: 'weighted' },
  { name: 'Leg press', scheme: '3×10', kind: 'weighted' },
  { name: 'Leg curls', scheme: '3×12', kind: 'weighted' },
  { name: 'Leg extension', scheme: '3×12', kind: 'weighted' },
  { name: 'Crunches', scheme: '3×15', kind: 'bodyweight' }
];

// Day 6 — lighter optional session: core work, with extra cardio as the main event.
export const CORE: Exercise[] = [
  { name: 'Plank', scheme: '3×60s', kind: 'time' },
  { name: 'Hanging leg raise', scheme: '3×12', kind: 'bodyweight' },
  { name: 'Crunches', scheme: '3×15', kind: 'bodyweight' },
  { name: 'Russian twists', scheme: '3×20', kind: 'bodyweight' },
  { name: 'Back extension', scheme: '3×15', kind: 'bodyweight' }
];

export type DayLabel = 'Push' | 'Pull' | 'Legs' | 'Core';

export const DAY_TEMPLATES: Record<DayLabel, Exercise[]> = {
  Push: PUSH,
  Pull: PULL,
  Legs: LEGS,
  Core: CORE
};

// Name → kind lookup across every template, so history / restored off-template
// rows can be rendered with the right units without a DB column. Defaults to
// 'weighted' for custom-added exercises not in any template.
const EXERCISE_KIND: Record<string, ExerciseKind> = (() => {
  const m: Record<string, ExerciseKind> = {};
  for (const list of Object.values(DAY_TEMPLATES)) {
    for (const ex of list) m[ex.name] = ex.kind;
  }
  return m;
})();

export function exerciseKind(name: string): ExerciseKind {
  return EXERCISE_KIND[name] ?? 'weighted';
}

/**
 * How many set rows a scheme renders. Normal schemes lead with the set count
 * ("4×8" → 4, "3×15–20" → 3). Pyramid schemes list each set explicitly as
 * weight×reps tokens ("50×5 60×3 80×2 90×1 100×1" → 5), so count those instead.
 */
export function setsFromScheme(scheme: string): number {
  const tokens = scheme.trim().split(/\s+/).filter((t) => t.includes('×'));
  if (tokens.length > 1) return tokens.length;
  const n = parseInt(scheme, 10);
  return Number.isFinite(n) && n > 0 ? n : 3;
}

// ---- cardio ----------------------------------------------------------------

// Cardio machines with a representative MET, used to estimate calories burnt.
// METs from the Compendium of Physical Activities (moderate-effort figures).
export const CARDIO_OPTIONS: { value: string; met: number }[] = [
  { value: 'Cycle', met: 7.0 },
  { value: 'Crosstrainer', met: 5.0 },
  { value: 'Treadmill (incline walk)', met: 6.3 },
  { value: 'Treadmill (run)', met: 9.8 },
  { value: 'Rowing', met: 7.0 },
  { value: 'Stair climber', met: 8.0 },
  { value: 'Other', met: 6.0 }
];

/** MET for a cardio kind; falls back to a moderate 6.0 for anything unlisted. */
export function cardioMet(kind: string): number {
  return CARDIO_OPTIONS.find((o) => o.value === kind)?.met ?? 6.0;
}

// The day's suggested cardio bout — mirrors the plan's "cardio rule" per focus.
export const CARDIO_DEFAULTS: Record<DayLabel, { kind: string; minutes: number }> = {
  Push: { kind: 'Cycle', minutes: 15 },
  Pull: { kind: 'Crosstrainer', minutes: 15 },
  Legs: { kind: 'Treadmill (incline walk)', minutes: 20 },
  Core: { kind: 'Treadmill (run)', minutes: 25 }
};

// ---- API types -------------------------------------------------------------

export type SessionSummary = {
  id: number;
  day_label: string;
  date: string;
  notes: string;
  created: string;
};

export type WorkoutSet = {
  id: number;
  exercise: string;
  set_index: number;
  reps: number;
  weight_g: number;
};

export type CardioEntry = {
  id: number;
  kind: string;
  minutes: number;
  kcal: number;
  entry_index: number;
};

export type SessionDetail = {
  session: SessionSummary;
  sets: WorkoutSet[];
  cardio?: CardioEntry[];
};

export type BodyweightEntry = {
  id: number;
  date: string;
  weight_g: number;
};

// ---- weight helpers (grams ⇄ kg) -------------------------------------------

/** Parse a kg string (e.g. "62.5") to integer grams; returns 0 on garbage. */
export function kgToGrams(kg: string | number): number {
  const v = typeof kg === 'number' ? kg : parseFloat(kg);
  if (!Number.isFinite(v) || v < 0) return 0;
  return Math.round(v * 1000);
}

/** Format integer grams as a kg string, trimming trailing zeros ("62.5", "60"). */
export function gramsToKg(g: number): string {
  const kg = g / 1000;
  return Number.isInteger(kg) ? String(kg) : kg.toFixed(1).replace(/\.0$/, '');
}

// ---- bodyweight trend ------------------------------------------------------

export type WeeklyAverage = { weekStart: string; avgKg: number; count: number };

/** ISO Monday (YYYY-MM-DD) of the week containing `date`. */
function weekStartOf(date: string): string {
  const d = new Date(date + 'T00:00:00');
  const day = (d.getDay() + 6) % 7; // 0 = Monday
  d.setDate(d.getDate() - day);
  return d.toISOString().slice(0, 10);
}

/**
 * Collapse daily bodyweight entries into weekly averages (oldest first).
 * Matches the cut-goal advice: track weekly average, ignore daily swings.
 */
export function weeklyAverages(entries: BodyweightEntry[]): WeeklyAverage[] {
  const buckets = new Map<string, { sum: number; count: number }>();
  for (const e of entries) {
    const key = weekStartOf(e.date);
    const b = buckets.get(key) ?? { sum: 0, count: 0 };
    b.sum += e.weight_g;
    b.count += 1;
    buckets.set(key, b);
  }
  return [...buckets.entries()]
    .sort((a, b) => (a[0] < b[0] ? -1 : 1))
    .map(([weekStart, b]) => ({
      weekStart,
      avgKg: Math.round((b.sum / b.count / 1000) * 10) / 10,
      count: b.count
    }));
}

// ---- energy burn (MET-based, mirrors the workout page's live estimate) ------

/**
 * Calories burnt in a single saved session: lifting (strengthKcal per logged
 * set) + cardio (stored kcal, falling back to a MET estimate from kind +
 * minutes at `weightKg` when the row has no kcal). Mirrors the per-session calc
 * on /fitness/workout so the meals page can reuse the same number.
 */
export function sessionBurnKcal(weightKg: number, detail: SessionDetail): number {
  const lift = strengthKcal(weightKg, detail.sets.length);
  const card = (detail.cardio ?? []).reduce((n, c) => {
    if (c.kcal > 0) return n + c.kcal;
    if (c.minutes > 0) return n + metKcal(cardioMet(c.kind), weightKg, c.minutes);
    return n;
  }, 0);
  return lift + card;
}

/**
 * Total calories burnt across every session on a given date — sum of
 * `sessionBurnKcal` over `sessions`. Used by the meals page to show today's
 * logged workout burn alongside the TDEE whole-day estimate.
 */
export function dayBurnKcal(weightKg: number, sessions: SessionDetail[]): number {
  return sessions.reduce((n, s) => n + sessionBurnKcal(weightKg, s), 0);
}
