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

// Four lifting days a week (Upper A / Lower A / Upper B / Lower B) plus a
// light fifth day of conditioning and pump work, laid out so no more than two
// lifting days ever run back to back:
//
//   Mon Upper A · Tue Lower A · Wed rest (walk) · Thu Upper B · Fri Lower B
//   Sat Conditioning + pump · Sun rest
//
// A missed lifting day moves to Wednesday or Saturday rather than doubling
// up; Saturday's own work is the part to drop when it does.
//
// Why: a six-day push / pull / legs split run back to back was too much to
// recover from, and the first four-day version left gaps: chest, arms, calves
// and core under target, the only two hinges 48 hours apart, a squat stacked
// behind leg press, and no single-leg or glute work. Fat loss has also slowed
// after the first 10 kg (95 kg to 85 kg in three months), which is mostly a
// food and daily-steps question, so the fifth day adds 35-40 minutes of easy
// cardio and some pump work rather than another heavy session, and the plan
// asks for 8-10k steps a day outside the gym.
//
// Each lifting day is one heavy compound per pattern, then a second compound
// or a single-leg lift, then isolation, so one hinge (Friday) and one squat
// (Tuesday) a week. Counted the way the report counts them (full sets for
// the target muscle, half for helpers, Saturday included), chest, triceps,
// quads, hamstrings and biceps land at 10-13 weekly sets, calves at 8 and
// glutes at 9; back (17) and shoulders (20.5) run higher because the side
// and rear delt work lands there. That holds muscle in a deficit without
// costing more recovery than a cut can afford. Every weighted slot names one implement and
// keeps it for the block so the loads stay comparable; the `alt` names the
// substitute when that implement is taken, on the same scheme. An exercise
// that appears on two days has the same scheme on both, or is a separate
// entry, so the load suggestions never read a 4-set day against a 3-set one.
//
// The gym: shoulder press machine (also an incline press, not flat), lat
// pulldown station with a seated cable row, pec fly / rear delt machine, leg
// extension / leg curl machine, leg press (also the calf raise), cable
// stack, Smith machine, back extension, benches, dumbbells, barbells, bands,
// mats, and a crosstrainer, cycle and treadmill. No hanging bar, no flat chest
// machine, no separate row or calf machine.

/**
 * How to run every session, shown with the plan. Kept as plain text: the
 * load suggestions in `workoutTargets.ts` apply the progression rule, and
 * the rest is on you.
 */
export const PLAN_RULES: string[] = [
  'Warm up: 5 min easy cardio, then band pull-aparts and hip / shoulder mobility. Ramp-up sets on key lifts are not logged.',
  'Rest 2-3 min on compounds, 60-90 s on isolation.',
  'Compounds: stop 1-2 reps short of failure. Isolation: the last set can go to 0-1 reps in reserve.',
  'Double progression: stay at a load until every set hits the top of the range, then add the smallest step (2 kg on dumbbells, 2.5 kg elsewhere).',
  'Timed and bodyweight moves: add 5-10 s or 1-2 reps a session up to the top of the range, then move to a harder variant.',
  'Deload every 6-8 weeks, or sooner if most key lifts stall two sessions running: half the sets, same loads, for one week.',
  'After 21+ days away from a lift, start 10 % lighter.',
  'Outside the gym: 8-10k steps a day. Slow fat loss is fixed in the kitchen and in daily steps, not with more sets.'
];

/** Day 1 (Mon): horizontal press and horizontal pull lead. */
export const UPPER_A: Exercise[] = [
  {
    name: 'Bench press',
    scheme: '4×6-10',
    kind: 'weighted',
    equipment: 'Dumbbell',
    priority: true,
    alt: 'Smith or barbell bench, same 4×6-10, if one is free'
  },
  {
    name: 'Seated cable row',
    scheme: '4×8-10',
    kind: 'weighted',
    equipment: 'Machine',
    priority: true,
    note: 'on the lat pulldown station'
  },
  {
    name: 'Shoulder press',
    scheme: '3×8-10',
    kind: 'weighted',
    equipment: 'Machine',
    priority: true
  },
  { name: 'Lat pulldown', scheme: '3×8-10', kind: 'weighted', equipment: 'Machine', note: 'wide grip' },
  // Side delts widen the shoulder line; the V-taper does more for a lean read
  // than arm size does, so they show up three times a week.
  { name: 'Lateral raises', scheme: '3×12-15', kind: 'weighted', equipment: 'Dumbbell' },
  {
    name: 'Cable tricep pushdown',
    scheme: '2×10-12',
    kind: 'weighted',
    equipment: 'Cable',
    note: 'rope; superset with bicep curls'
  },
  {
    name: 'Bicep curls',
    scheme: '2×10-12',
    kind: 'weighted',
    equipment: 'Dumbbell',
    note: 'superset with pushdowns'
  }
];

/** Day 2 (Tue): quad emphasis. The Smith squat leads while it is fresh; the
 *  split squat adds the single-leg and glute work the plan had none of. */
export const LOWER_A: Exercise[] = [
  {
    name: 'Squat',
    scheme: '3×6-10',
    kind: 'weighted',
    equipment: 'Smith',
    priority: true,
    alt: 'barbell back squat, same 3×6-10, if the Smith is taken'
  },
  {
    name: 'Bulgarian split squat',
    scheme: '3×8-10 each leg',
    kind: 'weighted',
    equipment: 'Dumbbell',
    priority: true
  },
  { name: 'Leg extension', scheme: '3×10-15', kind: 'weighted', equipment: 'Machine' },
  // Same entry and scheme on both lower days, so progression reads cleanly.
  { name: 'Leg curls', scheme: '3×10-12', kind: 'weighted', equipment: 'Machine' },
  { name: 'Calf raises', scheme: '4×12-15', kind: 'weighted', equipment: 'Machine', note: 'on the leg press' },
  // No hanging bar here; lying on a mat is the version that fits.
  { name: 'Lying leg raise', scheme: '3×10-15', kind: 'bodyweight', equipment: 'Bodyweight' }
];

/** Day 3 (Thu): incline press and vertical pull lead. */
export const UPPER_B: Exercise[] = [
  {
    name: 'Incline press',
    scheme: '3×8-10',
    kind: 'weighted',
    equipment: 'Machine',
    priority: true,
    note: 'shoulder press machine in incline mode',
    alt: 'incline dumbbell press, same 3×8-10, if the machine is taken'
  },
  // A separate entry from Upper A's wide-grip pulldown: different grip,
  // different loads, different scheme.
  {
    name: 'Close-grip lat pulldown',
    scheme: '4×8-10',
    kind: 'weighted',
    equipment: 'Machine',
    priority: true,
    note: 'neutral (V-handle) or close grip'
  },
  {
    name: 'Chest-supported row',
    scheme: '3×10-12',
    kind: 'weighted',
    equipment: 'Dumbbell',
    note: 'chest on an incline bench'
  },
  { name: 'Pec fly', scheme: '3×12-15', kind: 'weighted', equipment: 'Machine' },
  // Rear delts and posture. Cheap to do, and it changes how the upper body
  // reads standing still.
  {
    name: 'Reverse pec deck',
    scheme: '3×15-20',
    kind: 'weighted',
    equipment: 'Machine',
    note: 'pec fly machine set for rear delts'
  },
  { name: 'Cable lateral raise', scheme: '2×12-15', kind: 'weighted', equipment: 'Cable' },
  {
    name: 'Incline dumbbell curl',
    scheme: '2×10-12',
    kind: 'weighted',
    equipment: 'Dumbbell',
    note: 'superset with overhead extensions'
  },
  {
    name: 'Overhead cable tricep extension',
    scheme: '2×10-12',
    kind: 'weighted',
    equipment: 'Cable',
    note: 'superset with incline curls'
  }
];

/** Day 4 (Fri): the week's one hinge, then hamstrings and glutes. */
export const LOWER_B: Exercise[] = [
  // Straight sets in a rep range, not a pyramid to a single. The one slot that
  // wants a bar: do it first, while one is free.
  {
    name: 'Romanian deadlift',
    scheme: '3×6-10',
    kind: 'weighted',
    equipment: 'Barbell',
    priority: true,
    alt: 'dumbbells, same 3×6-10, if no bar is free'
  },
  { name: 'Leg press', scheme: '3×10-12', kind: 'weighted', equipment: 'Machine', priority: true },
  {
    name: 'Back extension',
    scheme: '3×12-15',
    kind: 'weighted',
    equipment: 'Machine',
    note: 'glute-biased: round the upper back, squeeze the glutes; hold a plate once 15 is easy'
  },
  { name: 'Leg curls', scheme: '3×10-12', kind: 'weighted', equipment: 'Machine' },
  { name: 'Calf raises', scheme: '4×12-15', kind: 'weighted', equipment: 'Machine', note: 'on the leg press' },
  { name: 'Cable crunch', scheme: '3×10-15', kind: 'weighted', equipment: 'Cable' },
  { name: 'Side plank', scheme: '2×30-45s', kind: 'time', equipment: 'Bodyweight', note: 'each side' }
];

/**
 * Day 5 (Sat): conditioning and pump. Zone 2 cardio first (35-40 min at a
 * pace you could talk through), then light shoulder, arm and core work. No
 * legs and no heavy presses or pulls, so Monday's Upper A starts fresh, and
 * nothing here is a key lift. Its sets count toward the weekly volume, but it
 * doesn't count toward a complete week of lifting, and it is the slot to give
 * up when a missed lifting day needs it. The label stays 'Cardio' because
 * logged sessions are stored under it.
 */
export const CARDIO_DAY: Exercise[] = [
  // A separate entry from Upper B's 2×12-15: lighter, higher reps.
  { name: 'Cable lateral raise (pump)', scheme: '3×15-20', kind: 'weighted', equipment: 'Cable' },
  { name: 'Face pulls', scheme: '3×15-20', kind: 'weighted', equipment: 'Cable' },
  { name: 'Cable curl', scheme: '2×12-15', kind: 'weighted', equipment: 'Cable' },
  {
    name: 'Single-arm cable tricep extension',
    scheme: '2×12-15',
    kind: 'weighted',
    equipment: 'Cable'
  },
  { name: 'Dead bug', scheme: '3×10', kind: 'bodyweight', equipment: 'Bodyweight', note: 'each side' },
  { name: 'Plank', scheme: '3×30-60s', kind: 'time', equipment: 'Bodyweight' }
];

export type DayLabel = 'Upper A' | 'Lower A' | 'Upper B' | 'Lower B' | 'Cardio';

/** Every day in week order, including the conditioning day. */
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
  'Upper A': {
    day: 'Mon',
    detail: 'chest / back / shoulders / arms',
    cardio: '15 min crosstrainer or cycle intervals: 30s hard / 90s easy'
  },
  'Lower A': {
    day: 'Tue',
    detail: 'quads / glutes / hamstrings / calves / core',
    cardio: '10 min cycle, easy: legs are already done'
  },
  'Upper B': {
    day: 'Thu',
    detail: 'chest / back / rear delts / shoulders / arms',
    cardio: '15 min incline treadmill walk'
  },
  'Lower B': {
    day: 'Fri',
    detail: 'hamstrings / glutes / quads / calves / core',
    cardio: '10 min cycle, easy'
  },
  Cardio: {
    day: 'Sat',
    detail: 'conditioning + pump: zone 2 cardio, then shoulders, arms and core; or a missed lifting day',
    cardio: '35-40 min zone 2: incline walk, cycle or crosstrainer at a talking pace',
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

// The day's suggested cardio bout, matching `DAY_INFO[...].cardio`. Upper days
// carry the harder work (intervals on Monday, an incline walk on Thursday);
// lower days get a short easy spin because the legs have just been trained.
// Saturday is the long, easy zone 2 bout. Daily steps happen outside the gym
// and aren't logged here.
export const CARDIO_DEFAULTS: Record<DayLabel, { kind: string; minutes: number }> = {
  'Upper A': { kind: 'Crosstrainer (intervals)', minutes: 15 },
  'Lower A': { kind: 'Cycle', minutes: 10 },
  'Upper B': { kind: 'Treadmill (incline walk)', minutes: 15 },
  'Lower B': { kind: 'Cycle', minutes: 10 },
  Cardio: { kind: 'Treadmill (incline walk)', minutes: 35 }
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
