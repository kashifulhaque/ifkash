// Shared workout data + helpers. The day templates live here so the static plan
// page (/workout) and the tracker (/workout/log) read from one source of truth.


// `kind` drives how the set-row UI renders:
//  - 'weighted'   → weight × reps inputs (e.g. bench press)
//  - 'bodyweight' → reps only; weight hidden (e.g. crunches)
//  - 'time'       → seconds only; weight hidden (e.g. plank hold)
// The persisted shape is unchanged (weight_g + reps); for time exercises the
// seconds go in the `reps` column and weight_g stays 0.
export type ExerciseKind = 'weighted' | 'bodyweight' | 'time';
export type Exercise = {
  name: string;
  scheme: string;
  kind: ExerciseKind;
  /** Default implement for the template row; the user can override per session. */
  equipment?: Equipment;
  /**
   * The lifts that actually drive the result — compounds and the two
   * previously-neglected patterns. Progress these first and never drop them to
   * save time; the unmarked rows are accessories and are the ones to cut on a
   * short day. Surfaced in the UI so priority is visible while logging.
   */
  priority?: boolean;
};

// Which implement a set was performed with. The same exercise name can be run
// on different equipment — a machine pec fly and dumbbell flyes are 50 kg apart
// — so loads are only comparable within one implement. '' means unspecified
// (every set logged before this field existed).
export type Equipment = '' | 'Barbell' | 'Dumbbell' | 'Machine' | 'Cable' | 'Smith' | 'Bodyweight';

export const EQUIPMENT_OPTIONS: Equipment[] = [
  'Barbell',
  'Dumbbell',
  'Machine',
  'Cable',
  'Smith',
  'Bodyweight'
];

// Tuned for a lean/athletic build on a cut: compounds carry the session and get
// the heavy, low-rep work (that load is what preserves muscle in a deficit),
// isolation is trimmed to what supports shape. Pec fly and tricep pushdown drop
// from 3 sets to 2 — pec fly had accumulated more sets than bench press, which
// is bodybuilding volume for an isolation movement.
export const PUSH: Exercise[] = [
  { name: 'Bench press', scheme: '4×6-8', kind: 'weighted', equipment: 'Barbell', priority: true },
  {
    name: 'Shoulder press',
    scheme: '3×8-10',
    kind: 'weighted',
    equipment: 'Dumbbell',
    priority: true
  },
  { name: 'Incline dumbbell press', scheme: '3×10', kind: 'weighted', equipment: 'Dumbbell' },
  // Side delts widen the shoulder line — the V-taper does more for an athletic
  // read than arm size does. Moved here from Pull, where it sat unused.
  { name: 'Lateral raises', scheme: '3×12-15', kind: 'weighted', equipment: 'Dumbbell' },
  { name: 'Hanging leg raise', scheme: '3×12', kind: 'bodyweight', equipment: 'Bodyweight' },
  { name: 'Crunches', scheme: '3×15', kind: 'bodyweight', equipment: 'Bodyweight' },
  { name: 'Pec fly', scheme: '2×12-15', kind: 'weighted', equipment: 'Machine' },
  { name: 'Cable tricep pushdown', scheme: '2×12-15', kind: 'weighted', equipment: 'Cable' }
];

// Face pulls now cover the rear-delt work that was split across three
// overlapping entries (face pulls / rear delts / cable rows-face pulls).
export const PULL: Exercise[] = [
  {
    name: 'Deadlift',
    scheme: '50×5 60×3 80×2 90×1 100×1',
    kind: 'weighted',
    equipment: 'Barbell',
    priority: true
  },
  { name: 'Lat pulldown', scheme: '4×8-10', kind: 'weighted', equipment: 'Cable', priority: true },
  { name: 'Row machine', scheme: '3×10', kind: 'weighted', equipment: 'Machine', priority: true },
  // Posture + rear delts. Cheap to do, disproportionate effect on how the
  // upper body reads standing still.
  { name: 'Face pulls', scheme: '3×15', kind: 'weighted', equipment: 'Cable', priority: true },
  { name: 'Bicep curls', scheme: '2×12', kind: 'weighted', equipment: 'Dumbbell' },
  { name: 'Plank', scheme: '3×60s', kind: 'time', equipment: 'Bodyweight' },
  { name: 'Russian twists', scheme: '3×20', kind: 'bodyweight', equipment: 'Bodyweight' },
  { name: 'Back extension', scheme: '3×15', kind: 'bodyweight', equipment: 'Bodyweight' }
];

// Run twice a week now, not once. Legs were the only muscle group going
// backwards (leg press flat, leg extension −15%) because they were trained at
// half the frequency of push. Squat and RDL are the two patterns that were
// missing entirely; calf raises had never been logged at all.
export const LEGS: Exercise[] = [
  { name: 'Barbell squat', scheme: '4×6-8', kind: 'weighted', equipment: 'Barbell', priority: true },
  {
    name: 'Romanian deadlift',
    scheme: '3×8-10',
    kind: 'weighted',
    equipment: 'Barbell',
    priority: true
  },
  {
    name: 'Leg press',
    scheme: '3×10-12',
    kind: 'weighted',
    equipment: 'Machine',
    priority: true
  },
  { name: 'Leg curls', scheme: '3×12', kind: 'weighted', equipment: 'Machine' },
  { name: 'Calf raises', scheme: '4×12-15', kind: 'weighted', equipment: 'Machine' },
  { name: 'Leg extension', scheme: '2×12-15', kind: 'weighted', equipment: 'Machine' },
];


export type DayLabel = 'Push' | 'Pull' | 'Legs';

export const DAY_TEMPLATES: Record<DayLabel, Exercise[]> = {
  Push: PUSH,
  Pull: PULL,
  Legs: LEGS
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

// Name → default implement, so restored off-template rows and newly added
// exercises start on the right equipment instead of blank.
const EXERCISE_EQUIPMENT: Record<string, Equipment> = (() => {
  const m: Record<string, Equipment> = {};
  for (const list of Object.values(DAY_TEMPLATES)) {
    for (const ex of list) if (ex.equipment) m[ex.name] = ex.equipment;
  }
  return m;
})();

export function exerciseEquipment(name: string): Equipment {
  return EXERCISE_EQUIPMENT[name] ?? '';
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

// Cardio options, limited to what the gym actually has (crosstrainer, treadmill,
// cycle) plus outdoor walking. METs start from the Compendium of Physical
// Activities but are calibrated against logged sessions and a Galaxy Watch 4,
// because the console readouts are optimistic:
//
//   crosstrainer — machine claims 13-15 kcal/min, watch reads 9-12. At ~84 kg
//   the watch midpoint works out to ~7 METs, i.e. vigorous effort rather than
//   the Compendium's 5.0 "moderate" figure. Estimating from the watch keeps the
//   deficit maths honest; trusting the console would overstate the weekly burn
//   by roughly 200 kcal.
//
//   cycle (6.0) and incline walk (6.3) are back-solved from logged bouts —
//   8.8 and 9.5 kcal/min respectively.
export const CARDIO_OPTIONS: { value: string; met: number }[] = [
  { value: 'Crosstrainer', met: 7.0 },
  // Work intervals push the average well above steady state — the highest burn
  // per minute available on the equipment here, and the only option that builds
  // any real conditioning rather than just spending calories.
  { value: 'Crosstrainer (intervals)', met: 8.5 },
  { value: 'Cycle', met: 6.0 },
  { value: 'Cycle (intervals)', met: 8.0 },
  { value: 'Treadmill (incline walk)', met: 6.3 },
  { value: 'Treadmill (run)', met: 9.8 },
  // Not a machine — the NEAT lever. Cheap calories with zero interference with
  // lifting, and it doesn't need the gym.
  { value: 'Walk (outdoor)', met: 3.5 },
  { value: 'Other', met: 6.0 }
];

/** MET for a cardio kind; falls back to a moderate 6.0 for anything unlisted. */
export function cardioMet(kind: string): number {
  return CARDIO_OPTIONS.find((o) => o.value === kind)?.met ?? 6.0;
}

// The day's suggested cardio bout — mirrors the plan's "cardio rule" per focus.
// Treadmill is deliberately not prescribed anywhere: it's the one machine here
// that won't get used, and cardio you skip burns nothing. Crosstrainer carries
// the week, with one interval session for conditioning; leg days get an easy
// spin instead, since the legs have just been worked.
export const CARDIO_DEFAULTS: Record<DayLabel, { kind: string; minutes: number }> = {
  Push: { kind: 'Crosstrainer', minutes: 20 },
  Pull: { kind: 'Crosstrainer (intervals)', minutes: 20 },
  Legs: { kind: 'Cycle', minutes: 15 }
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
  /** '' for sets logged before equipment was tracked. */
  equipment: Equipment;
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

