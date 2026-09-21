// Shared workout data + helpers. The day templates live here so the plan view
// and the tracker on /fitness/workout read from one source of truth.

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
  /**
   * What to do when the prescribed implement is taken, or a heavier option
   * to use when it is free. The rack and the Smith machine are contested at
   * this gym, so the plan prescribes what is reliably available and names the
   * upgrade rather than the other way round.
   */
  alt?: string;
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

// Four lifting days a week (upper / lower / upper / lower) plus an optional
// cardio-only day, replacing the six-day push / pull / legs split.
//
// Why: thirteen weeks of logs showed 5-6 gym days a week but only 5-6 of each
// day's 8 exercises done, core skipped every time, deadlift dropped after the
// heavy-single weeks, and loads flat or falling on every priority lift.
// Attendance was never the problem; the sessions were too long to finish and
// the week too long to recover from. Each day here is 6 exercises and 16-18
// working sets, about 45 minutes of lifting plus a short cardio bout, so a
// complete week is four finished sessions rather than six partial ones.
//
// Volume lands at 10-14 weekly sets per muscle group, which holds muscle in a
// deficit; extra sets on a cut cost recovery without adding shape. Every
// weighted slot names one implement and keeps it for the block, because the
// same exercise had been logged on barbell, dumbbell, Smith, and machine in
// turn and no load progression was readable. The barbell and Smith machine
// are contested here, so the pressing and squatting slots prescribe dumbbells
// and machines, which are always free, and name the barbell as the upgrade
// when it is. No heavy singles anywhere: straight sets in a rep range, add
// load when every set hits the top of it.

/** Day 1: horizontal press and vertical pull lead. */
export const UPPER_A: Exercise[] = [
  {
    name: 'Bench press',
    scheme: '4×8-10',
    kind: 'weighted',
    equipment: 'Dumbbell',
    priority: true,
    alt: 'barbell 4×6-8 if a bench is free'
  },
  { name: 'Lat pulldown', scheme: '3×8-10', kind: 'weighted', equipment: 'Machine', priority: true },
  {
    name: 'Shoulder press',
    scheme: '3×8-10',
    kind: 'weighted',
    equipment: 'Machine',
    priority: true
  },
  { name: 'Row machine', scheme: '3×10-12', kind: 'weighted', equipment: 'Machine' },
  // Side delts widen the shoulder line; the V-taper does more for a lean read
  // than arm size does, so they run on both upper days.
  { name: 'Lateral raises', scheme: '3×12-15', kind: 'weighted', equipment: 'Dumbbell' },
  { name: 'Cable tricep pushdown', scheme: '2×12-15', kind: 'weighted', equipment: 'Cable' }
];

/** Day 2: quad emphasis. Leg press leads because it is the heaviest quad
 *  load that never needs a rack; the squat is dumbbell-limited, so it runs
 *  second at higher reps. */
export const LOWER_A: Exercise[] = [
  { name: 'Leg press', scheme: '3×8-10', kind: 'weighted', equipment: 'Machine', priority: true },
  {
    name: 'Squat',
    scheme: '3×8-10',
    kind: 'weighted',
    equipment: 'Dumbbell',
    priority: true,
    alt: 'goblet or two dumbbells; barbell 3×6-8 if the rack is free'
  },
  {
    name: 'Romanian deadlift',
    scheme: '3×8-10',
    kind: 'weighted',
    equipment: 'Dumbbell',
    priority: true,
    alt: 'barbell if one is free'
  },
  { name: 'Leg curls', scheme: '3×10-12', kind: 'weighted', equipment: 'Machine' },
  { name: 'Calf raises', scheme: '3×12-15', kind: 'weighted', equipment: 'Machine' },
  { name: 'Hanging leg raise', scheme: '3×10-12', kind: 'bodyweight', equipment: 'Bodyweight' }
];

/** Day 3: horizontal pull and incline press lead. */
export const UPPER_B: Exercise[] = [
  { name: 'Row machine', scheme: '4×8-10', kind: 'weighted', equipment: 'Machine', priority: true },
  {
    name: 'Incline dumbbell press',
    scheme: '3×8-10',
    kind: 'weighted',
    equipment: 'Dumbbell',
    priority: true
  },
  { name: 'Lat pulldown', scheme: '3×10-12', kind: 'weighted', equipment: 'Machine' },
  { name: 'Lateral raises', scheme: '3×12-15', kind: 'weighted', equipment: 'Dumbbell' },
  // Posture and rear delts. Cheap to do, and it changes how the upper body
  // reads standing still.
  { name: 'Face pulls', scheme: '2×15', kind: 'weighted', equipment: 'Cable' },
  { name: 'Bicep curls', scheme: '2×10-12', kind: 'weighted', equipment: 'Dumbbell' }
];

/** Day 4: deadlift and leg press, hamstring emphasis. */
export const LOWER_B: Exercise[] = [
  // Straight sets, not a pyramid to a single. The singles were the most
  // fatiguing thing in the old week and the lift was dropped within a month.
  // The one slot that still wants a bar: do it first, while one is free.
  {
    name: 'Deadlift',
    scheme: '3×5',
    kind: 'weighted',
    equipment: 'Barbell',
    priority: true,
    alt: 'no bar: dumbbell Romanian deadlift 3×8-10'
  },
  { name: 'Leg press', scheme: '3×10-12', kind: 'weighted', equipment: 'Machine', priority: true },
  { name: 'Leg extension', scheme: '2×12-15', kind: 'weighted', equipment: 'Machine' },
  { name: 'Leg curls', scheme: '2×12-15', kind: 'weighted', equipment: 'Machine' },
  { name: 'Calf raises', scheme: '3×12-15', kind: 'weighted', equipment: 'Machine' },
  { name: 'Plank', scheme: '3×45s', kind: 'time', equipment: 'Bodyweight' }
];

/** Optional day 5: cardio only, no lifting. Logging it is a bonus, not a debt. */
export const CARDIO_ONLY: Exercise[] = [];

export type DayLabel = 'Upper A' | 'Lower A' | 'Upper B' | 'Lower B' | 'Cardio';

/** Every day in week order, including the optional one. */
export const DAY_LABELS: DayLabel[] = ['Upper A', 'Lower A', 'Upper B', 'Lower B', 'Cardio'];

/** The four days that make a complete week. */
export const LIFT_DAYS: DayLabel[] = ['Upper A', 'Lower A', 'Upper B', 'Lower B'];

export const DAY_TEMPLATES: Record<DayLabel, Exercise[]> = {
  'Upper A': UPPER_A,
  'Lower A': LOWER_A,
  'Upper B': UPPER_B,
  'Lower B': LOWER_B,
  Cardio: CARDIO_ONLY
};

export type DayInfo = {
  /** Position in the week, shown on the chip. */
  day: string;
  /** Muscle groups, shown as the chip tooltip. */
  detail: string;
  /** The suggested cardio bout for the day, shown in the day-card header. */
  cardio: string;
  /** True for the day that doesn't count toward a complete week. */
  optional?: boolean;
};

export const DAY_INFO: Record<DayLabel, DayInfo> = {
  'Upper A': {
    day: 'Mon',
    detail: 'chest / back / shoulders / triceps',
    cardio: '20 min crosstrainer intervals: 30s hard / 90s easy'
  },
  'Lower A': {
    day: 'Tue',
    detail: 'quads / hamstrings / calves / core',
    cardio: '10 min crosstrainer, easy: legs are already done'
  },
  'Upper B': {
    day: 'Wed',
    detail: 'back / chest / shoulders / biceps',
    cardio: '20 min crosstrainer, steady'
  },
  'Lower B': {
    day: 'Thu',
    detail: 'hamstrings / quads / calves / core',
    cardio: '10 min crosstrainer, easy'
  },
  Cardio: {
    day: 'Sat / Sun',
    detail: 'the spare slot: a missed lift if there is one, otherwise cardio only',
    cardio: '30 min crosstrainer intervals, or a long walk outside',
    optional: true
  }
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

// The day's suggested cardio bout, matching `DAY_INFO[...].cardio`. Every
// bout is on the crosstrainer: the cycle and treadmill are the two machines
// here that won't get used, and cardio you skip burns nothing. Upper days
// carry the real work; lower days get a short easy spin because the legs
// have just been trained. The optional day is where a longer bout belongs.
// Daily steps happen outside the gym and aren't logged here.
export const CARDIO_DEFAULTS: Record<DayLabel, { kind: string; minutes: number }> = {
  'Upper A': { kind: 'Crosstrainer (intervals)', minutes: 20 },
  'Lower A': { kind: 'Crosstrainer', minutes: 10 },
  'Upper B': { kind: 'Crosstrainer', minutes: 20 },
  'Lower B': { kind: 'Crosstrainer', minutes: 10 },
  Cardio: { kind: 'Crosstrainer (intervals)', minutes: 30 }
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

