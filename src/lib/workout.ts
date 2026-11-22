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
  /**
   * A cue for how to run the slot: grip, setup, a per-side count, or which
   * lift it is supersetted with. Not a substitute; that is `alt`.
   */
  note?: string;
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

// Four compact lifting days per week, with an optional fifth day for easy
// cardio and core. Leave a rest / walking day between the two pairs:
//
//   Mon Upper A · Tue Lower A · Wed rest · Thu Upper B · Fri Lower B
//   Sat optional cardio + core · Sun rest
//
// The lifting sessions stay at four trackable exercise slots. After a recent
// training break, consistency and adding reps matter more than crushing volume.
// The optional day never makes up a missed lift; just resume the next session.
// missed lift; just resume the next planned session.
//
// The gym: crosstrainer, cycle, treadmill, incline/shoulder-press machine,
// lat pulldown, cable row, pec fly / reverse pec deck, leg raise, leg curl,
// leg press, cable stack, Smith machine, barbell, back extension, benches,
// dumbbells, kettlebell, and mat.

/**
 * How to run every session, shown with the plan. The weekly minimum is four
 * short lifting sessions; the fifth day is optional.
 */
export const PLAN_RULES: string[] = [
  'Keep lifting sessions to about 40–50 min. Warm up with 5 min easy movement and 1–2 lighter ramp-up sets for the first lift.',
  'After a break of 2+ weeks, do 2 working sets per exercise for your first two weeks back; use easy loads and finish each set with 3 good reps left. Then build to the listed sets and leave 1–2 reps in reserve.',
  'Rest about 2 min after presses, rows, squats and hinges; 60–90 sec on accessories. Keep transitions simple and leave the gym on time.',
  'Use the rep range shown. When every set reaches its top number with clean form, add the smallest available weight and build reps again.',
  'If you miss a day, do the next session in order; do not double up or cram. The optional fifth day is a bonus, never a make-up day.',
  'Aim for a repeatable daily step target (start at 7,000; build toward 8,000–10,000). Sleep and a manageable food deficit matter more for fat loss than extra gym time.'
];

/** Day 1 (Mon): balanced upper, with a small shoulder accessory. */
export const UPPER_A: Exercise[] = [
  { name: 'Dumbbell bench press', scheme: '3×8-12', kind: 'weighted', equipment: 'Dumbbell', priority: true },
  { name: 'Lat pulldown', scheme: '3×8-12', kind: 'weighted', equipment: 'Machine', priority: true, note: 'comfortable shoulder-width grip' },
  { name: 'Seated cable row', scheme: '2×10-12', kind: 'weighted', equipment: 'Machine' },
  { name: 'Dumbbell lateral raise', scheme: '2×12-15', kind: 'weighted', equipment: 'Dumbbell' }
];

/** Day 2 (Tue): leg press and hamstring focus. */
export const LOWER_A: Exercise[] = [
  { name: 'Leg press', scheme: '3×8-12', kind: 'weighted', equipment: 'Machine', priority: true, note: 'controlled depth; keep lower back on pad' },
  { name: 'Dumbbell Romanian deadlift', scheme: '3×8-12', kind: 'weighted', equipment: 'Dumbbell', priority: true },
  { name: 'Leg curls', scheme: '2×10-15', kind: 'weighted', equipment: 'Machine' },
  { name: 'Dumbbell calf raise', scheme: '2×10-15', kind: 'weighted', equipment: 'Dumbbell' }
];

/** Day 3 (Thu): incline press and upper-back work. */
export const UPPER_B: Exercise[] = [
  { name: 'Incline chest press', scheme: '3×8-12', kind: 'weighted', equipment: 'Machine', priority: true },
  { name: 'Cable row', scheme: '3×8-12', kind: 'weighted', equipment: 'Machine' },
  { name: 'Reverse pec deck', scheme: '2×12-15', kind: 'weighted', equipment: 'Machine' },
  { name: 'Dumbbell bicep curl', scheme: '2×10-15', kind: 'weighted', equipment: 'Dumbbell' }
];

/** Day 4 (Fri): squat pattern, hinge, and a little core. */
export const LOWER_B: Exercise[] = [
  { name: 'Smith machine squat', scheme: '3×8-12', kind: 'weighted', equipment: 'Smith', priority: true, note: 'comfortable depth; stop before form changes' },
  { name: 'Barbell Romanian deadlift', scheme: '3×8-10', kind: 'weighted', equipment: 'Barbell', priority: true },
  { name: 'Leg curls', scheme: '2×10-15', kind: 'weighted', equipment: 'Machine' },
  { name: 'Lying leg raise', scheme: '2×8-12', kind: 'bodyweight', equipment: 'Bodyweight', note: 'bend knees if needed; move without swinging' }
];

/** Day 5 (Sat): optional easy cardio, then two brief core exercises. */
export const CARDIO_DAY: Exercise[] = [
  { name: 'Lying leg raise', scheme: '2×8-12', kind: 'bodyweight', equipment: 'Bodyweight' },
  { name: 'Plank', scheme: '2×20-40s', kind: 'time', equipment: 'Bodyweight' }
];

export type DayLabel = 'Upper A' | 'Lower A' | 'Upper B' | 'Lower B' | 'Cardio';

/** Every day in week order, including the optional conditioning day. */
export const DAY_LABELS: DayLabel[] = ['Upper A', 'Lower A', 'Upper B', 'Lower B', 'Cardio'];

/** The four lifting days that make a complete week. */
export const LIFT_DAYS: DayLabel[] = ['Upper A', 'Lower A', 'Upper B', 'Lower B'];

export const DAY_TEMPLATES: Record<DayLabel, Exercise[]> = {
  'Upper A': UPPER_A,
  'Lower A': LOWER_A,
  'Upper B': UPPER_B,
  'Lower B': LOWER_B,
  Cardio: CARDIO_DAY
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
  'Upper A': { day: 'Mon', detail: 'chest / back / shoulders', cardio: 'Optional: 5–10 min easy cooldown' },
  'Lower A': { day: 'Tue', detail: 'quads / hamstrings / calves', cardio: 'Optional: 5 min easy cooldown' },
  'Upper B': { day: 'Thu', detail: 'chest / back / rear delts / triceps', cardio: 'Optional: 5–10 min easy cooldown' },
  'Lower B': { day: 'Fri', detail: 'quads / hamstrings / core', cardio: 'Optional: 5 min easy cooldown' },
  Cardio: {
    day: 'Sat',
    detail: 'optional easy cardio + core',
    cardio: '25–35 min easy conversational pace',
    optional: true
  }
};

// Older names that are no longer on any day but still sit in the log, so
// history keeps the right units and implement.
const LEGACY_KIND: Record<string, ExerciseKind> = {
  'Hanging leg raise': 'bodyweight',
  Crunches: 'bodyweight'
};
const LEGACY_EQUIPMENT: Record<string, Equipment> = {
  'Row machine': 'Machine',
  'Incline dumbbell press': 'Dumbbell',
  Deadlift: 'Barbell',
  'Hanging leg raise': 'Bodyweight',
  Crunches: 'Bodyweight'
};

// Name → kind lookup across every template, so history / restored off-template
// rows can be rendered with the right units without a DB column. Defaults to
// 'weighted' for custom-added exercises not in any template.
const EXERCISE_KIND: Record<string, ExerciseKind> = (() => {
  const m: Record<string, ExerciseKind> = { ...LEGACY_KIND };
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
  const m: Record<string, Equipment> = { ...LEGACY_EQUIPMENT };
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

// Suggested short cooldowns on lifting days; the optional fifth day is the
// only dedicated cardio session. Steps happen outside the gym.
export const CARDIO_DEFAULTS: Record<DayLabel, { kind: string; minutes: number }> = {
  'Upper A': { kind: 'Crosstrainer', minutes: 8 },
  'Lower A': { kind: 'Crosstrainer', minutes: 5 },
  'Upper B': { kind: 'Crosstrainer', minutes: 8 },
  'Lower B': { kind: 'Crosstrainer', minutes: 5 },
  Cardio: { kind: 'Crosstrainer', minutes: 30 }
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

/** One logged set, stamped with its session's date and day label (`GET /log`). */
export type LogSet = {
  session_id: number;
  date: string;
  day_label: string;
  exercise: string;
  equipment: Equipment;
  set_index: number;
  reps: number;
  weight_g: number;
};

/** One logged cardio bout, stamped like `LogSet`. */
export type LogCardio = {
  session_id: number;
  date: string;
  day_label: string;
  kind: string;
  minutes: number;
  kcal: number;
};

/** The whole training history, oldest first, as the report view reads it. */
export type TrainingLog = { sets: LogSet[]; cardio: LogCardio[] };

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


/**
 * Group a session detail's flat set rows by exercise and implement, preserving
 * order. Splitting on equipment keeps a machine pec fly and dumbbell flyes as
 * two separate blocks, because their loads aren't comparable.
 */
export function groupSets(detail: SessionDetail) {
  const groups: {
    exercise: string;
    equipment: Equipment;
    sets: { reps: number; weight_g: number }[];
  }[] = [];
  for (const s of detail.sets) {
    const equipment = s.equipment ?? '';
    let g = groups.find((x) => x.exercise === s.exercise && x.equipment === equipment);
    if (!g) {
      g = { exercise: s.exercise, equipment, sets: [] };
      groups.push(g);
    }
    g.sets.push({ reps: s.reps, weight_g: s.weight_g });
  }
  return groups;
}
