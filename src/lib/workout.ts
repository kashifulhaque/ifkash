// Shared workout data + helpers. The day templates live here so the static plan
// page (/workout) and the tracker (/workout/log) read from one source of truth.

export type Exercise = { name: string; scheme: string };

export const PUSH: Exercise[] = [
  { name: 'Smith machine bench press', scheme: '4×8' },
  { name: 'Shoulder press (machine)', scheme: '3×10' },
  { name: 'Incline dumbbell press (adjustable bench)', scheme: '3×10' },
  { name: 'Pec fly', scheme: '3×12' },
  { name: 'Cable tricep pushdown', scheme: '3×12' }
];

export const PULL: Exercise[] = [
  { name: 'Lat pulldown', scheme: '3×10' },
  { name: 'Row machine', scheme: '3×10' },
  { name: 'Deadlift (smith) — ramping pyramid to a top single', scheme: '50×5 60×3 80×2 90×1 100×1' },
  { name: 'Cable rows / face pulls', scheme: '3×12' },
  { name: 'Rear delts', scheme: '3×15' },
  { name: 'Dumbbell bicep curls', scheme: '3×12' }
];

export const LEGS: Exercise[] = [
  { name: 'Smith barbell squat', scheme: '4×8' },
  { name: 'Leg press', scheme: '3×10' },
  { name: 'Leg curls', scheme: '3×12' },
  { name: 'Leg extension', scheme: '3×12' },
  { name: 'Standing calf raise', scheme: '3×15' },
  { name: 'Cable crunch (cable machine)', scheme: '3×15' }
];

// Day 6 — lighter optional session: core work, with extra cardio as the main event.
export const CORE: Exercise[] = [
  { name: 'Plank', scheme: '3×60s' },
  { name: 'Hanging leg raise', scheme: '3×12' },
  { name: 'Cable crunch (cable machine)', scheme: '3×15' },
  { name: 'Russian twists (weighted)', scheme: '3×20' },
  { name: 'Back extension', scheme: '3×15' }
];

export type DayLabel = 'Push' | 'Pull' | 'Legs' | 'Core';

export const DAY_TEMPLATES: Record<DayLabel, Exercise[]> = {
  Push: PUSH,
  Pull: PULL,
  Legs: LEGS,
  Core: CORE
};

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
