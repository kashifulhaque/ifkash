// Training reports for /fitness/workout: a day, week, or month of logged sets
// read against the plan, compared with another period, and turned into a few
// plain-language notes. Pure and I/O-free like `fitnessInsights.ts`; the page
// fetches the log through `workoutApi.listLog()` and only formats what this
// module returns.
//
// Three rules keep the output short enough to read at a glance:
//   - every number is read against the plan (`DAY_TEMPLATES`), so "18 sets"
//     always comes with "of 17 planned";
//   - loads are only compared within one exercise *and* one implement, the
//     same rule the logging view uses;
//   - notes are capped at three per column and ranked, so the page never
//     turns into a wall of advice.

import {
  DAY_TEMPLATES,
  DAY_LABELS,
  LIFT_DAYS,
  CARDIO_DEFAULTS,
  setsFromScheme,
  exerciseKind,
  exerciseEquipment,
  gramsToKg,
  type DayLabel,
  type Equipment,
  type ExerciseKind,
  type LogSet,
  type LogCardio,
  type TrainingLog,
  type BodyweightEntry
} from '$lib/workout';

export type Period = 'day' | 'week' | 'month';

const MS_PER_DAY = 86_400_000;
const toDays = (iso: string): number => Math.round(Date.parse(iso + 'T00:00:00Z') / MS_PER_DAY);
const toISO = (days: number): string => new Date(days * MS_PER_DAY).toISOString().slice(0, 10);
/** `toDays` counts from the epoch, a Thursday, so Mondays fall on t ≡ 4 (mod 7). */
const mondayOf = (t: number): number => t - ((((t - 4) % 7) + 7) % 7);
const pad = (v: number): string => String(v).padStart(2, '0');
const round = (v: number, dp = 0): number => {
  const f = 10 ** dp;
  return Math.round(v * f) / f;
};

function fmt(iso: string, opts: Intl.DateTimeFormatOptions): string {
  return new Date(iso + 'T00:00:00Z').toLocaleDateString('en-GB', { timeZone: 'UTC', ...opts });
}

// ---- periods ---------------------------------------------------------------

export type Range = {
  period: Period;
  /** Inclusive ISO bounds. */
  start: string;
  end: string;
  days: number;
  /** "Sat 26 Sep", "21–27 Sep", "September 2026". */
  label: string;
  /** "today", "last week", … or '' when the period is further back. */
  relative: string;
  /** True while the period contains today — its numbers are still filling in. */
  current: boolean;
  /** True once the period has fully ended. */
  past: boolean;
};

const RELATIVE: Record<Period, [string, string]> = {
  day: ['today', 'yesterday'],
  week: ['this week', 'last week'],
  month: ['this month', 'last month']
};

/** The day, week (Monday to Sunday), or calendar month that contains `anchor`. */
export function periodRange(period: Period, anchor: string, today: string): Range {
  const t = toDays(anchor);
  let s: number;
  let e: number;
  if (period === 'day') {
    s = e = t;
  } else if (period === 'week') {
    s = mondayOf(t);
    e = s + 6;
  } else {
    const [y, m] = anchor.split('-').map(Number);
    s = toDays(`${y}-${pad(m)}-01`);
    e = toDays(m === 12 ? `${y + 1}-01-01` : `${y}-${pad(m + 1)}-01`) - 1;
  }
  const start = toISO(s);
  const end = toISO(e);
  const nowT = toDays(today);
  const thisYear = today.slice(0, 4);
  const year = start.slice(0, 4) !== thisYear || end.slice(0, 4) !== thisYear;

  let label: string;
  if (period === 'day') {
    label = fmt(start, { weekday: 'short', day: 'numeric', month: 'short', ...(year ? { year: 'numeric' } : {}) });
  } else if (period === 'week') {
    const sameMonth = start.slice(0, 7) === end.slice(0, 7);
    label = sameMonth
      ? `${fmt(start, { day: 'numeric' })}–${fmt(end, { day: 'numeric', month: 'short' })}`
      : `${fmt(start, { day: 'numeric', month: 'short' })} – ${fmt(end, { day: 'numeric', month: 'short' })}`;
    if (year) label += ` ${end.slice(0, 4)}`;
  } else {
    label = fmt(start, { month: 'long', year: 'numeric' });
  }

  // How many periods back from the one containing today.
  const todayRange = period === 'day' ? nowT : period === 'week' ? mondayOf(nowT) : null;
  let back: number;
  if (todayRange !== null) back = Math.round((todayRange - s) / (period === 'day' ? 1 : 7));
  else {
    const [ty, tm] = today.split('-').map(Number);
    const [ay, am] = start.split('-').map(Number);
    back = ty * 12 + tm - (ay * 12 + am);
  }
  const relative = back === 0 ? RELATIVE[period][0] : back === 1 ? RELATIVE[period][1] : '';

  return {
    period,
    start,
    end,
    days: e - s + 1,
    label,
    relative,
    current: s <= nowT && nowT <= e,
    past: e < nowT
  };
}

/**
 * The first `days` days of a range, for comparing a period still in progress
 * like-for-like: Monday to Saturday against last Monday to Saturday.
 */
export function clipRange(range: Range, days: number): Range {
  if (days >= range.days) return range;
  const s = toDays(range.start);
  const end = toISO(s + Math.max(1, days) - 1);
  const short = (iso: string) => fmt(iso, { day: 'numeric', month: 'short' });
  return {
    ...range,
    end,
    days: Math.max(1, days),
    label:
      range.start === end
        ? short(end)
        : range.start.slice(0, 7) === end.slice(0, 7)
          ? `${fmt(range.start, { day: 'numeric' })}–${short(end)}`
          : `${short(range.start)} – ${short(end)}`,
    past: true
  };
}

/** Days from the range's start through `today`, inclusive. */
export function elapsedDays(range: Range, today: string): number {
  return Math.min(range.days, toDays(today) - toDays(range.start) + 1);
}

/** Anchor date `steps` periods before (negative) or after the range's start. */
export function shiftAnchor(range: Range, steps: number): string {
  const s = toDays(range.start);
  if (range.period === 'day') return toISO(s + steps);
  if (range.period === 'week') return toISO(s + steps * 7);
  const [y, m] = range.start.split('-').map(Number);
  const idx = y * 12 + (m - 1) + steps;
  return `${Math.floor(idx / 12)}-${pad((idx % 12) + 1)}-01`;
}

/**
 * The period to compare against by default. Weeks and months compare with the
 * one before. A day compares with the last session of the same day label, so
 * Upper A reads against the previous Upper A rather than yesterday's legs.
 */
export function defaultCompareAnchor(range: Range, log: TrainingLog): string {
  if (range.period === 'day') {
    const label = log.sets.find((s) => s.date === range.start)?.day_label
      ?? log.cardio.find((c) => c.date === range.start)?.day_label;
    if (label) {
      const prior = [...log.sets, ...log.cardio]
        .filter((x) => x.day_label === label && x.date < range.start)
        .map((x) => x.date)
        .sort();
      if (prior.length) return prior[prior.length - 1];
    }
  }
  return shiftAnchor(range, -1);
}

/** Every date with a logged set or bout, ascending. */
export function loggedDates(log: TrainingLog): string[] {
  return [...new Set([...log.sets, ...log.cardio].map((x) => x.date))].sort();
}

// ---- scoring a set ---------------------------------------------------------

/**
 * One comparable number per set. Weighted sets use the Epley estimated max,
 * so 22 kg × 10 and 24 kg × 8 land on the same scale; reps past 15 are capped
 * because the formula stops meaning much there. Bodyweight sets score reps;
 * timed sets score seconds.
 */
export function setScore(kind: ExerciseKind, reps: number, weight_g: number): number {
  if (reps <= 0) return 0;
  if (kind !== 'weighted' || weight_g <= 0) return reps;
  return (weight_g / 1000) * (1 + Math.min(reps, 15) / 30);
}

const liftKey = (exercise: string, equipment: Equipment): string => `${exercise}|${equipment}`;

export function setText(kind: ExerciseKind, reps: number, weight_g: number): string {
  if (kind === 'time') return `${reps}s`;
  return weight_g > 0 ? `${gramsToKg(weight_g)} kg × ${reps}` : `${reps} reps`;
}

// ---- muscle groups ---------------------------------------------------------

export type Muscle =
  | 'chest'
  | 'back'
  | 'shoulders'
  | 'biceps'
  | 'triceps'
  | 'quads'
  | 'hamstrings'
  | 'glutes'
  | 'calves'
  | 'core';

export const MUSCLES: Muscle[] = [
  'chest',
  'back',
  'shoulders',
  'biceps',
  'triceps',
  'quads',
  'hamstrings',
  'glutes',
  'calves',
  'core'
];

type MuscleShare = Partial<Record<Muscle, number>>;

// A set counts fully toward the muscle it's for and half toward the ones that
// assist, the usual way weekly volume is tallied.
const MUSCLE_MAP: Record<string, MuscleShare> = {
  'Bench press': { chest: 1, triceps: 0.5, shoulders: 0.5 },
  'Incline dumbbell press': { chest: 1, shoulders: 0.5, triceps: 0.5 },
  'Shoulder press': { shoulders: 1, triceps: 0.5 },
  'Lat pulldown': { back: 1, biceps: 0.5 },
  'Row machine': { back: 1, biceps: 0.5 },
  'Lateral raises': { shoulders: 1 },
  'Face pulls': { shoulders: 1, back: 0.5 },
  'Cable tricep pushdown': { triceps: 1 },
  'Bicep curls': { biceps: 1 },
  'Leg press': { quads: 1, glutes: 0.5 },
  Squat: { quads: 1, glutes: 0.5 },
  'Leg extension': { quads: 1 },
  'Romanian deadlift': { hamstrings: 1, glutes: 0.5 },
  Deadlift: { hamstrings: 1, glutes: 0.5, back: 0.5 },
  'Leg curls': { hamstrings: 1 },
  'Calf raises': { calves: 1 },
  'Hanging leg raise': { core: 1 },
  Plank: { core: 1 }
};

/**
 * Which muscles an exercise trains. Names outside the plan (custom rows, the
 * old push/pull/legs lifts) fall back to keyword matching; anything that
 * matches nothing is left out of the volume count rather than guessed.
 */
export function musclesFor(name: string): MuscleShare {
  const known = MUSCLE_MAP[name];
  if (known) return known;
  const n = name.toLowerCase();
  if (/calf/.test(n)) return { calves: 1 };
  if (/plank|crunch|\bab|core|leg raise|sit-?up/.test(n)) return { core: 1 };
  if (/leg curl|hamstring|rdl|deadlift/.test(n)) return { hamstrings: 1, glutes: 0.5 };
  if (/leg press|squat|lunge|leg extension|split/.test(n)) return { quads: 1, glutes: 0.5 };
  if (/glute|hip thrust/.test(n)) return { glutes: 1 };
  if (/tricep|pushdown|dip|skull/.test(n)) return { triceps: 1 };
  if (/curl/.test(n)) return { biceps: 1 };
  if (/bench|chest|fly|pec|push-?up/.test(n)) return { chest: 1, triceps: 0.5 };
  if (/row|pulldown|pull-?up|chin|lat\b/.test(n)) return { back: 1, biceps: 0.5 };
  if (/shoulder|overhead|lateral|delt|face pull|raise/.test(n)) return { shoulders: 1 };
  return {};
}

/** Weekly sets per muscle the four lifting days prescribe — the volume target. */
export const PLAN_WEEKLY_SETS: Record<Muscle, number> = (() => {
  const out = Object.fromEntries(MUSCLES.map((m) => [m, 0])) as Record<Muscle, number>;
  for (const day of LIFT_DAYS) {
    for (const ex of DAY_TEMPLATES[day]) {
      const sets = setsFromScheme(ex.scheme);
      for (const [m, share] of Object.entries(musclesFor(ex.name))) out[m as Muscle] += sets * (share ?? 0);
    }
  }
  return out;
})();

/** Weekly cardio minutes the plan's four lifting days add up to. */
export const PLAN_WEEKLY_CARDIO = LIFT_DAYS.reduce((n, d) => n + CARDIO_DEFAULTS[d].minutes, 0);

// ---- rep ranges ------------------------------------------------------------

export type RepRange = { sets: number; lo: number; hi: number };

/** "4×8-10" → 4 sets of 8–10; "3×5" → 3 of 5; "3×45s" → 3 of 45 (seconds). */
export function parseScheme(scheme: string): RepRange | null {
  const m = scheme.match(/^(\d+)\s*×\s*(\d+)(?:\s*[-–]\s*(\d+))?/);
  if (!m) return null;
  const lo = Number(m[2]);
  return { sets: Number(m[1]), lo, hi: m[3] ? Number(m[3]) : lo };
}

/** The prescribed scheme for an exercise on a day, falling back to any day. */
function schemeFor(exercise: string, dayLabel: string): string {
  const onDay = DAY_TEMPLATES[dayLabel as DayLabel]?.find((e) => e.name === exercise);
  if (onDay) return onDay.scheme;
  for (const d of LIFT_DAYS) {
    const e = DAY_TEMPLATES[d].find((x) => x.name === exercise);
    if (e) return e.scheme;
  }
  return '';
}

const PRIORITY = new Set(
  LIFT_DAYS.flatMap((d) => DAY_TEMPLATES[d].filter((e) => e.priority).map((e) => e.name))
);
/**
 * A key lift on its prescribed implement (or an untagged older set). The same
 * name on another implement is a stand-in for a day, not the lift to track.
 */
export const isKeyLift = (exercise: string, equipment: Equipment = ''): boolean =>
  PRIORITY.has(exercise) && (!equipment || equipment === exerciseEquipment(exercise));

// ---- per-session view ------------------------------------------------------

type SessionSets = {
  id: number;
  date: string;
  day_label: string;
  /** exercise|equipment → its sets in logged order. */
  lifts: Map<string, { exercise: string; equipment: Equipment; sets: LogSet[] }>;
  cardio: LogCardio[];
};

/** Regroup the flat log by session, oldest first. */
function bySession(log: TrainingLog): SessionSets[] {
  const map = new Map<number, SessionSets>();
  const get = (x: { session_id: number; date: string; day_label: string }) => {
    let s = map.get(x.session_id);
    if (!s) {
      s = { id: x.session_id, date: x.date, day_label: x.day_label, lifts: new Map(), cardio: [] };
      map.set(x.session_id, s);
    }
    return s;
  };
  for (const set of log.sets) {
    const s = get(set);
    const k = liftKey(set.exercise, set.equipment ?? '');
    let l = s.lifts.get(k);
    if (!l) {
      l = { exercise: set.exercise, equipment: set.equipment ?? '', sets: [] };
      s.lifts.set(k, l);
    }
    l.sets.push(set);
  }
  for (const c of log.cardio) get(c).cardio.push(c);
  return [...map.values()].sort((a, b) => (a.date === b.date ? a.id - b.id : a.date < b.date ? -1 : 1));
}

/** The best set of a block by `setScore`. */
function bestOf(kind: ExerciseKind, sets: { reps: number; weight_g: number }[]) {
  let best: { reps: number; weight_g: number; score: number } | null = null;
  for (const s of sets) {
    const score = setScore(kind, s.reps, s.weight_g);
    if (score > 0 && (!best || score > best.score)) best = { reps: s.reps, weight_g: s.weight_g, score };
  }
  return best;
}

// ---- the report ------------------------------------------------------------

export type LiftSummary = {
  key: string;
  exercise: string;
  equipment: Equipment;
  kind: ExerciseKind;
  key_lift: boolean;
  sets: number;
  /** The period's best set and its score. */
  best: { reps: number; weight_g: number; score: number } | null;
  /** Every set of the latest session in the period, for the day view. */
  lastSets: { reps: number; weight_g: number }[];
  /** The most recent session before the period that did this lift. */
  before: { date: string; score: number; reps: number; weight_g: number } | null;
  /** Share change of `best` vs `before`; null without both. */
  change: number | null;
  /** The best set beat every earlier session of this lift. */
  pr: boolean;
};

export type MissedLift = { exercise: string; day_label: string; date: string; key_lift: boolean };

export type ScoreParts = {
  attendance: number;
  completion: number;
  cardio: number;
  progress: number;
};

export type Report = {
  range: Range;
  sessions: { id: number; date: string; day_label: string; sets: number }[];
  activeDays: number;
  /** Distinct lifting-day labels logged — the week checklist. */
  liftLabels: string[];
  liftSessions: number;
  /** Lifting sessions logged under an older plan's labels. */
  legacySessions: number;
  /**
   * Lifting turn-ups for attendance: distinct plan days plus legacy sessions
   * (capped at 4) for a week, every lifting session otherwise.
   */
  liftCount: number;
  /** Lifting sessions the plan asks for across this range. */
  liftTarget: number;
  sets: number;
  reps: number;
  /** Load × reps over weighted sets, kg. */
  volumeKg: number;
  cardioMin: number;
  cardioKcal: number;
  cardioTarget: number;
  lifts: LiftSummary[];
  muscles: { muscle: Muscle; sets: number; target: number }[];
  /** Planned vs done sets across the sessions logged on a plan day. */
  plannedSets: number;
  doneSets: number;
  plannedExercises: number;
  doneExercises: number;
  missed: MissedLift[];
  prs: LiftSummary[];
  avgBodyweightKg: number | null;
  /** 0–100, or null when nothing is logged. */
  score: number | null;
  parts: ScoreParts | null;
  /** Plan weeks the range spans (7 days = 1). */
  weeks: number;
};

/** Everything the report card shows for one range. `log` must be oldest first. */
export function buildReport(log: TrainingLog, range: Range, bodyweight: BodyweightEntry[]): Report {
  const all = bySession(log);
  const inRange = (d: string) => d >= range.start && d <= range.end;
  const sessions = all.filter((s) => inRange(s.date));
  const weeks = range.days / 7;

  // Walk the whole log once, oldest first, keeping the best-ever score and the
  // latest session for each lift, so PRs and "vs last time" need no second pass.
  const bestEver = new Map<string, number>();
  const lastSeen = new Map<string, { date: string; score: number; reps: number; weight_g: number }>();
  const lifts = new Map<string, LiftSummary>();

  for (const s of all) {
    const within = inRange(s.date);
    if (s.date > range.end) break;
    for (const [k, l] of s.lifts) {
      const kind = exerciseKind(l.exercise);
      const best = bestOf(kind, l.sets);
      if (within) {
        let sum = lifts.get(k);
        if (!sum) {
          const before = lastSeen.get(k) ?? null;
          sum = {
            key: k,
            exercise: l.exercise,
            equipment: l.equipment,
            kind,
            key_lift: isKeyLift(l.exercise, l.equipment),
            sets: 0,
            best: null,
            lastSets: [],
            before,
            change: null,
            pr: false
          };
          lifts.set(k, sum);
        }
        sum.sets += l.sets.length;
        sum.lastSets = l.sets.map((x) => ({ reps: x.reps, weight_g: x.weight_g }));
        if (best && (!sum.best || best.score > sum.best.score)) sum.best = best;
        const prior = bestEver.get(k);
        // A first-ever log isn't a record; it's a baseline.
        if (best && prior !== undefined && best.score > prior * 1.001) sum.pr = true;
      }
      if (best) {
        bestEver.set(k, Math.max(bestEver.get(k) ?? 0, best.score));
        if (!within) lastSeen.set(k, { date: s.date, score: best.score, reps: best.reps, weight_g: best.weight_g });
      }
    }
  }
  for (const l of lifts.values()) {
    if (l.best && l.before && l.before.score > 0) l.change = l.best.score / l.before.score - 1;
  }

  // Plan adherence: what each plan-day session prescribed against what got logged.
  let plannedSets = 0;
  let doneSets = 0;
  let plannedExercises = 0;
  let doneExercises = 0;
  const missed: MissedLift[] = [];
  for (const s of sessions) {
    const template = DAY_TEMPLATES[s.day_label as DayLabel];
    if (!template?.length) continue;
    for (const ex of template) {
      const want = setsFromScheme(ex.scheme);
      const got = [...s.lifts.values()]
        .filter((l) => l.exercise === ex.name)
        .reduce((n, l) => n + l.sets.length, 0);
      plannedSets += want;
      doneSets += Math.min(want, got);
      plannedExercises += 1;
      if (got > 0) doneExercises += 1;
      else missed.push({ exercise: ex.name, day_label: s.day_label, date: s.date, key_lift: !!ex.priority });
    }
  }

  const muscleSets = Object.fromEntries(MUSCLES.map((m) => [m, 0])) as Record<Muscle, number>;
  let sets = 0;
  let reps = 0;
  let volumeKg = 0;
  for (const s of sessions) {
    for (const l of s.lifts.values()) {
      const kind = exerciseKind(l.exercise);
      const share = musclesFor(l.exercise);
      for (const x of l.sets) {
        sets += 1;
        if (kind !== 'time') reps += x.reps;
        if (kind === 'weighted') volumeKg += (x.weight_g / 1000) * x.reps;
        for (const [m, f] of Object.entries(share)) muscleSets[m as Muscle] += f ?? 0;
      }
    }
  }
  const cardio = sessions.flatMap((s) => s.cardio);
  const cardioMin = cardio.reduce((n, c) => n + c.minutes, 0);
  const cardioKcal = cardio.reduce((n, c) => n + c.kcal, 0);

  const liftLabels = [...new Set(sessions.map((s) => s.day_label))].filter((l) =>
    (LIFT_DAYS as string[]).includes(l)
  );
  const liftSessions = sessions.filter((s) => (LIFT_DAYS as string[]).includes(s.day_label)).length;
  // Sessions from before this plan (Push/Pull/Legs, or unlabelled) still count
  // as turning up; they just can't tick a plan day.
  const legacySessions = sessions.filter(
    (s) => !(DAY_LABELS as string[]).includes(s.day_label) && s.lifts.size > 0
  ).length;
  const liftCount =
    range.period === 'week'
      ? Math.min(4, liftLabels.length + legacySessions)
      : liftSessions + legacySessions;

  const dayLabel = sessions[0]?.day_label as DayLabel | undefined;
  const liftTarget = range.period === 'day' ? (dayLabel && DAY_TEMPLATES[dayLabel]?.length ? 1 : 0) : round(4 * weeks, 1);
  const cardioTarget =
    range.period === 'day'
      ? dayLabel && CARDIO_DEFAULTS[dayLabel]
        ? CARDIO_DEFAULTS[dayLabel].minutes
        : 0
      : Math.round(PLAN_WEEKLY_CARDIO * weeks);

  const bw = bodyweight.filter((b) => inRange(b.date) && b.weight_g > 0);
  const avgBodyweightKg = bw.length ? round(bw.reduce((n, b) => n + b.weight_g, 0) / bw.length / 1000, 1) : null;

  const liftList = [...lifts.values()].sort(
    (a, b) => Number(b.key_lift) - Number(a.key_lift) || b.sets - a.sets
  );

  // The score: turning up, finishing what was planned, doing the cardio, and
  // holding or beating the key lifts. Weighted toward attendance because a
  // finished week of average sessions beats two great ones.
  let score: number | null = null;
  let parts: ScoreParts | null = null;
  if (sessions.length) {
    const tracked = liftList.filter((l) => l.key_lift && l.best && l.before);
    const held = tracked.filter((l) => (l.change ?? 0) >= -0.02).length;
    const completion = plannedSets > 0 ? doneSets / plannedSets : 1;
    const cardioShare = cardioTarget > 0 ? Math.min(1, cardioMin / cardioTarget) : 1;
    const progress = tracked.length ? held / tracked.length : 0.75;
    if (range.period === 'day') {
      parts = { attendance: 1, completion, cardio: cardioShare, progress };
      score = Math.round(100 * (0.6 * completion + 0.2 * cardioShare + 0.2 * progress));
    } else {
      const attendance = range.period === 'week' ? liftCount / 4 : Math.min(1, liftCount / liftTarget);
      parts = { attendance, completion, cardio: cardioShare, progress };
      score = Math.round(100 * (0.4 * attendance + 0.25 * completion + 0.15 * cardioShare + 0.2 * progress));
    }
  }

  return {
    range,
    sessions: sessions.map((s) => ({
      id: s.id,
      date: s.date,
      day_label: s.day_label,
      sets: [...s.lifts.values()].reduce((n, l) => n + l.sets.length, 0)
    })),
    activeDays: new Set(sessions.map((s) => s.date)).size,
    liftLabels,
    liftSessions,
    legacySessions,
    liftCount,
    liftTarget,
    sets,
    reps,
    volumeKg: Math.round(volumeKg),
    cardioMin,
    cardioKcal,
    cardioTarget,
    lifts: liftList,
    muscles: MUSCLES.map((m) => ({
      muscle: m,
      sets: round(muscleSets[m], 1),
      target: round(PLAN_WEEKLY_SETS[m] * weeks, 1)
    })),
    plannedSets,
    doneSets,
    plannedExercises,
    doneExercises,
    missed,
    prs: liftList.filter((l) => l.pr),
    avgBodyweightKg,
    score,
    parts,
    weeks
  };
}

/** One word for a score, so the number never stands alone. */
export function grade(score: number): string {
  if (score >= 85) return 'excellent';
  if (score >= 70) return 'solid';
  if (score >= 50) return 'decent';
  if (score >= 25) return 'light';
  return 'quiet';
}

// ---- progression -----------------------------------------------------------

export type Progression = {
  exercise: string;
  equipment: Equipment;
  key_lift: boolean;
  /** 'up' = add load/reps, 'down' = too heavy, 'stall' = flat for a while. */
  kind: 'up' | 'down' | 'stall';
  text: string;
};

/** The usual smallest jump on each implement. */
function loadStep(equipment: Equipment): number {
  return equipment === 'Dumbbell' ? 2 : 2.5;
}

/**
 * What to change next time for each plan lift, from its latest session. The
 * plan's rule is double progression: stay at a load until every set reaches
 * the top of the rep range, then add the smallest jump. Also flags lifts
 * whose best set hasn't moved in four sessions.
 */
export function progressions(log: TrainingLog, onlyDay?: string): Progression[] {
  const sessions = bySession(log);
  const history = new Map<string, { date: string; day_label: string; sets: LogSet[]; score: number }[]>();
  for (const s of sessions) {
    for (const [k, l] of s.lifts) {
      const kind = exerciseKind(l.exercise);
      const list = history.get(k) ?? [];
      list.push({ date: s.date, day_label: s.day_label, sets: l.sets, score: bestOf(kind, l.sets)?.score ?? 0 });
      history.set(k, list);
    }
  }

  const out: Progression[] = [];
  const days = onlyDay ? [onlyDay as DayLabel] : LIFT_DAYS;
  const seen = new Set<string>();
  for (const day of days) {
    for (const ex of DAY_TEMPLATES[day] ?? []) {
      // Latest history of this exercise on any implement; the latest implement wins.
      const keys = [...history.keys()].filter((k) => k.startsWith(ex.name + '|'));
      if (!keys.length) continue;
      const k = keys.sort((a, b) => {
        const la = history.get(a)!;
        const lb = history.get(b)!;
        return la[la.length - 1].date < lb[lb.length - 1].date ? 1 : -1;
      })[0];
      if (seen.has(k)) continue;
      seen.add(k);
      const list = history.get(k)!;
      const last = list[list.length - 1];
      const equipment = (k.split('|')[1] ?? '') as Equipment;
      const range = parseScheme(schemeFor(ex.name, last.day_label || day));
      if (!range) continue;
      const label = equipment ? `${ex.name} (${equipment.toLowerCase()})` : ex.name;
      const base = { exercise: ex.name, equipment, key_lift: !!ex.priority };

      if (ex.kind === 'weighted') {
        const top = Math.max(...last.sets.map((s) => s.weight_g));
        if (top <= 0) continue;
        const working = last.sets.filter((s) => s.weight_g === top);
        const kg = top / 1000;
        if (working.length >= range.sets && working.every((s) => s.reps >= range.hi)) {
          const next = Math.round((kg + loadStep(equipment)) * 2) / 2;
          out.push({
            ...base,
            kind: 'up',
            text: `${label}: every set hit ${range.hi} at ${gramsToKg(top)} kg. Go to ${gramsToKg(next * 1000)} kg.`
          });
          continue;
        }
        // Only call it too heavy when it's clearly short: one rep under the
        // range is a normal bad day, not a reason to drop the load.
        const avgReps = working.reduce((n, s) => n + s.reps, 0) / Math.max(1, working.length);
        if (working.length && working.every((s) => s.reps < range.lo) && avgReps <= range.lo - 2) {
          out.push({
            ...base,
            kind: 'down',
            text: `${label}: under ${range.lo} reps at ${gramsToKg(top)} kg. Stay there until every set reaches ${range.lo}, or drop one step.`
          });
          continue;
        }
      } else if (last.sets.length >= range.sets && last.sets.every((s) => s.reps >= range.hi)) {
        out.push({
          ...base,
          kind: 'up',
          text:
            ex.kind === 'time'
              ? `${label}: every hold reached ${range.hi}s. Add 15 seconds.`
              : `${label}: every set hit ${range.hi}. Add reps, slow the lowering, or hold a weight.`
        });
        continue;
      }

      // A stall: four sessions, three weeks or more, and none of the last
      // three beat the first by more than 1 %.
      if (ex.priority && list.length >= 4) {
        const recent = list.slice(-4);
        const spanDays = toDays(recent[3].date) - toDays(recent[0].date);
        if (spanDays >= 14 && recent.slice(1).every((r) => r.score <= recent[0].score * 1.01)) {
          out.push({
            ...base,
            kind: 'stall',
            text: `${label}: no gain in 4 sessions. Add one rep to one set each session before adding load.`
          });
        }
      }
    }
  }
  // Key lifts first, then load increases before the rest.
  const order = { up: 0, stall: 1, down: 2 };
  return out.sort((a, b) => Number(b.key_lift) - Number(a.key_lift) || order[a.kind] - order[b.kind]);
}

// ---- notes -----------------------------------------------------------------

export type Note = { text: string };
export type Notes = { good: Note[]; improve: Note[] };

const pct = (v: number): string => `${Math.round(Math.abs(v) * 100)}%`;
const list = (names: string[], max = 3): string =>
  names.length <= max ? joinAnd(names) : `${names.slice(0, max).join(', ')} and ${names.length - max} more`;
function joinAnd(names: string[]): string {
  if (names.length <= 1) return names.join('');
  return `${names.slice(0, -1).join(', ')} and ${names[names.length - 1]}`;
}
const unique = (xs: string[]) => [...new Set(xs)];

/** Consecutive complete weeks (all four lifts) ending with the week of `end`. */
export function completeWeekStreak(log: TrainingLog, end: string): number {
  const byWeek = new Map<number, Set<string>>();
  for (const x of [...log.sets, ...log.cardio]) {
    const w = mondayOf(toDays(x.date));
    const set = byWeek.get(w) ?? new Set();
    set.add(x.day_label);
    byWeek.set(w, set);
  }
  const complete = (w: number) => LIFT_DAYS.every((d) => byWeek.get(w)?.has(d));
  let w = mondayOf(toDays(end));
  // An unfinished current week doesn't break the streak behind it.
  if (!complete(w)) w -= 7;
  let n = 0;
  while (complete(w)) {
    n += 1;
    w -= 7;
  }
  return n;
}

/**
 * What went well and what to work on, three of each at most, ranked. Reads
 * the report against the plan and against the comparison period.
 */
export function reportNotes(r: Report, cmp: Report | null, log: TrainingLog): Notes {
  const good: Note[] = [];
  const improve: Note[] = [];
  const add = (to: Note[], text: string) => to.push({ text });
  if (!r.sessions.length) return { good, improve };

  const unit = r.range.period === 'day' ? 'last time' : `the ${r.range.period} before`;

  // Records first: they're the clearest sign the plan is working.
  if (r.prs.length === 1) {
    const p = r.prs[0];
    add(good, `New best on ${p.exercise}: ${setText(p.kind, p.best!.reps, p.best!.weight_g)}.`);
  } else if (r.prs.length > 1) {
    add(good, `${r.prs.length} new bests: ${list(r.prs.map((p) => p.exercise))}.`);
  }

  if (r.range.period === 'day') {
    if (r.plannedExercises > 0 && r.doneExercises === r.plannedExercises) {
      add(good, `Finished all ${r.plannedExercises} exercises on the plan.`);
    }
    const beat = r.lifts.filter((l) => !l.pr && (l.change ?? 0) > 0.01).map((l) => l.exercise);
    if (beat.length) add(good, `Beat last time on ${list(beat)}.`);
  } else {
    if (r.range.period === 'week' && r.liftLabels.length >= 4) add(good, 'All four lifting days done.');
    else if (r.range.period === 'week' && r.liftCount >= 4) {
      add(good, `${r.liftSessions + r.legacySessions} lifting sessions — plenty of turning up.`);
    }
    const streak = completeWeekStreak(log, r.range.end);
    if (streak >= 2 && r.range.period === 'week') {
      // An unfinished current week sits on top of the streak rather than in it.
      const open = r.range.current && r.liftLabels.length < 4;
      add(good, open ? `${streak} complete weeks in a row — finish this one to make it ${streak + 1}.` : `${streak} complete weeks in a row.`);
    }
    if (r.range.period === 'month' && r.liftCount >= r.liftTarget * 0.9) {
      add(good, `${r.liftCount} lifting sessions — right on the plan's ${Math.round(r.liftTarget)}.`);
    }
  }

  if (r.plannedSets > 0 && r.doneSets / r.plannedSets >= 0.95 && r.range.period !== 'day') {
    add(good, `${Math.round((r.doneSets / r.plannedSets) * 100)}% of planned sets done in the sessions logged.`);
  }
  if (r.cardioTarget > 0 && r.cardioMin >= r.cardioTarget) {
    add(good, `Cardio covered: ${r.cardioMin} of ${r.cardioTarget} planned minutes.`);
  }

  // Strength held while the scale moved down — the point of lifting on a cut.
  const keyTracked = r.lifts.filter((l) => l.key_lift && l.change !== null);
  const keyHeld = keyTracked.every((l) => (l.change ?? 0) >= -0.02);
  if (
    cmp?.avgBodyweightKg != null &&
    r.avgBodyweightKg != null &&
    r.avgBodyweightKg < cmp.avgBodyweightKg - 0.2 &&
    keyTracked.length >= 2 &&
    keyHeld
  ) {
    add(good, `Down ${(cmp.avgBodyweightKg - r.avgBodyweightKg).toFixed(1)} kg with key lifts holding — muscle is being kept.`);
  }
  if (cmp && cmp.volumeKg > 0 && r.range.past && r.volumeKg > cmp.volumeKg * 1.1) {
    add(good, `Volume up ${pct(r.volumeKg / cmp.volumeKg - 1)} on ${unit}.`);
  }

  // ---- to work on ----
  const missedKey = unique(r.missed.filter((m) => m.key_lift).map((m) => m.exercise));
  if (missedKey.length) {
    add(improve, `Skipped key ${missedKey.length === 1 ? 'lift' : 'lifts'}: ${list(missedKey)}. Cut an accessory instead.`);
  }

  // A week logged on the old plan's labels can't be checked against this one.
  if (r.range.period === 'week' && r.legacySessions === 0) {
    const left = LIFT_DAYS.filter((d) => !r.liftLabels.includes(d));
    if (left.length && r.range.past) {
      add(improve, `Missed ${list(left)}. The weekend slot is there for a missed lift.`);
    } else if (left.length && r.range.current) {
      add(improve, `Still to do this week: ${list(left)}.`);
    }
  } else if (r.range.period === 'month' && r.range.past && r.liftCount < r.liftTarget * 0.75) {
    add(improve, `${r.liftCount} lifting sessions against a plan of about ${Math.round(r.liftTarget)}.`);
  }

  const dropped = keyTracked
    .filter((l) => (l.change ?? 0) < -0.05)
    .sort((a, b) => (a.change ?? 0) - (b.change ?? 0));
  if (dropped.length) {
    const d = dropped[0];
    const more = dropped.length > 1 ? ` (and ${dropped.length - 1} more)` : '';
    add(
      improve,
      `${d.exercise} fell ${pct(d.change ?? 0)} from ${r.range.period === 'day' ? 'last time' : `its last session before this ${r.range.period}`}${more}. Check sleep and food before changing the plan.`
    );
  }

  // Volume gaps only once the period is over — midweek every muscle is "behind".
  if (r.range.past && r.range.period !== 'day') {
    const low = r.muscles
      .filter((m) => m.target >= 2 && m.sets < m.target * 0.6)
      .sort((a, b) => a.sets / a.target - b.sets / b.target);
    if (low.length) {
      const m = low[0];
      const others = low.length > 1 ? `, and ${list(low.slice(1).map((x) => x.muscle), 2)} ran low too` : '';
      add(improve, `${cap(m.muscle)} got ${fmtSets(m.sets)} of ${fmtSets(m.target)} planned sets${others}.`);
    }
  }

  if (r.cardioTarget > 0 && r.cardioMin < r.cardioTarget * 0.6 && (r.range.past || r.range.period === 'day')) {
    add(improve, `Cardio: ${r.cardioMin} of ${r.cardioTarget} planned minutes.`);
  }

  if (r.range.period === 'day') {
    const skipped = unique(r.missed.filter((m) => !m.key_lift).map((m) => m.exercise));
    if (skipped.length) add(improve, `Skipped ${list(skipped)}.`);
    const worse = r.lifts
      .filter((l) => !l.key_lift && (l.change ?? 0) < -0.05)
      .map((l) => l.exercise);
    if (worse.length) add(improve, `Below last time on ${list(worse)}.`);
  }

  return { good: good.slice(0, 3), improve: improve.slice(0, 3) };
}

const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
const fmtSets = (v: number) => (Number.isInteger(v) ? String(v) : v.toFixed(1));

// ---- fun facts -------------------------------------------------------------

// Reference masses for the "you moved…" line, lightest first.
const THINGS: { name: string; plural: string; kg: number; emoji: string }[] = [
  { name: 'golden retriever', plural: 'golden retrievers', kg: 30, emoji: '🐕' },
  { name: 'fridge', plural: 'fridges', kg: 90, emoji: '🧊' },
  { name: 'grand piano', plural: 'grand pianos', kg: 450, emoji: '🎹' },
  { name: 'car', plural: 'cars', kg: 1_400, emoji: '🚗' },
  { name: 'African elephant', plural: 'African elephants', kg: 6_000, emoji: '🐘' },
  { name: 'T. rex', plural: 'T. rexes', kg: 8_000, emoji: '🦖' },
  { name: 'blue whale', plural: 'blue whales', kg: 150_000, emoji: '🐋' }
];

/** The largest reference thing the tonnage covers at least once. */
export function tonnageLike(kg: number): string | null {
  if (kg < THINGS[0].kg) return null;
  const thing = [...THINGS].reverse().find((t) => kg >= t.kg)!;
  const n = kg / thing.kg;
  const count = n < 10 ? n.toFixed(1).replace(/\.0$/, '') : String(Math.round(n));
  return `${thing.emoji} about ${count} ${count === '1' ? thing.name : thing.plural}`;
}

export type FunFact = { emoji: string; text: string };

/** A few light facts for the breakdown fold. Never more than four. */
export function funFacts(r: Report, log: TrainingLog): FunFact[] {
  const out: FunFact[] = [];
  const like = tonnageLike(r.volumeKg);
  if (like) {
    const [emoji, ...rest] = like.split(' ');
    out.push({ emoji, text: `${r.volumeKg.toLocaleString('en-GB')} kg moved, ${rest.join(' ')}.` });
  }
  if (r.cardioKcal >= 285) {
    out.push({ emoji: '🍕', text: `Cardio burned ~${r.cardioKcal.toLocaleString('en-GB')} kcal, about ${Math.round(r.cardioKcal / 285)} slices of pizza.` });
  }
  if (r.reps >= 100) {
    out.push({ emoji: '🔁', text: `${r.reps.toLocaleString('en-GB')} reps in ${r.sets} sets.` });
  }

  // Across the whole log: the most-trained lift and the favourite weekday.
  const counts = new Map<string, number>();
  for (const s of log.sets) counts.set(s.exercise, (counts.get(s.exercise) ?? 0) + 1);
  const fav = [...counts.entries()].sort((a, b) => b[1] - a[1])[0];
  if (fav && fav[1] >= 20) out.push({ emoji: '❤️', text: `Most-logged lift ever: ${fav[0]}, ${fav[1]} sets.` });

  const dates = loggedDates(log);
  if (dates.length >= 8) {
    const wd = new Array(7).fill(0);
    for (const d of dates) wd[toDays(d) - mondayOf(toDays(d))] += 1; // 0 = Monday
    const names = ['Mondays', 'Tuesdays', 'Wednesdays', 'Thursdays', 'Fridays', 'Saturdays', 'Sundays'];
    const top = wd.indexOf(Math.max(...wd));
    out.push({ emoji: '📅', text: `You train most on ${names[top]} (${wd[top]} of ${dates.length} days).` });
  }
  return out.slice(0, 4);
}

// ---- calendar --------------------------------------------------------------

export type CalendarCell = {
  date: string;
  dom: number;
  /** Mon = 0 … Sun = 6. */
  weekday: number;
  inRange: boolean;
  future: boolean;
  sets: number;
  labels: string[];
  /** 0 = nothing, 1–3 = light to heavy, relative to a planned session. */
  level: number;
};

/**
 * Monday-aligned cells covering the range, padded to whole weeks, for the
 * week strip and month calendar. Shade is sets against a full plan session
 * (about 17), so a full day always reads darkest whatever the month held.
 */
export function calendar(log: TrainingLog, range: Range, today: string): CalendarCell[] {
  const per = new Map<string, { sets: number; labels: Set<string> }>();
  for (const x of log.sets) {
    const p = per.get(x.date) ?? { sets: 0, labels: new Set() };
    p.sets += 1;
    if (x.day_label) p.labels.add(x.day_label);
    per.set(x.date, p);
  }
  for (const c of log.cardio) {
    const p = per.get(c.date) ?? { sets: 0, labels: new Set() };
    if (c.day_label) p.labels.add(c.day_label);
    per.set(c.date, p);
  }
  const s = mondayOf(toDays(range.start));
  const e = mondayOf(toDays(range.end)) + 6;
  const nowT = toDays(today);
  const cells: CalendarCell[] = [];
  for (let t = s; t <= e; t++) {
    const date = toISO(t);
    const p = per.get(date);
    const sets = p?.sets ?? 0;
    const labels = p ? [...p.labels] : [];
    cells.push({
      date,
      dom: Number(date.slice(8)),
      weekday: (t - s) % 7,
      inRange: date >= range.start && date <= range.end,
      future: t > nowT,
      sets,
      labels,
      level: !p ? 0 : sets === 0 ? 1 : sets < 9 ? 1 : sets < 15 ? 2 : 3
    });
  }
  return cells;
}

// ---- keeping the log current -----------------------------------------------

/**
 * Replace one session's rows after an auto-save, matching the upsert's key
 * (date + day label), so the report updates without refetching the log.
 */
export function patchLog(
  log: TrainingLog,
  session: { session_id: number; date: string; day_label: string },
  exercises: { exercise: string; equipment: Equipment; sets: { reps: number; weight_g: number }[] }[],
  cardio: { kind: string; minutes: number; kcal: number }[]
): TrainingLog {
  const keep = <T extends { date: string; day_label: string; session_id: number }>(x: T) =>
    x.session_id !== session.session_id && !(x.date === session.date && x.day_label === session.day_label);
  const sets: LogSet[] = log.sets.filter(keep);
  for (const ex of exercises) {
    ex.sets.forEach((s, i) =>
      sets.push({ ...session, exercise: ex.exercise, equipment: ex.equipment, set_index: i + 1, ...s })
    );
  }
  const bouts: LogCardio[] = [...log.cardio.filter(keep), ...cardio.map((c) => ({ ...session, ...c }))];
  const byDate = (a: { date: string; session_id: number }, b: { date: string; session_id: number }) =>
    a.date === b.date ? a.session_id - b.session_id : a.date < b.date ? -1 : 1;
  // Stable sort keeps each session's set order intact.
  return { sets: sets.sort(byDate), cardio: bouts.sort(byDate) };
}

/** Drop a deleted session's rows. */
export function dropSession(log: TrainingLog, sessionId: number): TrainingLog {
  return {
    sets: log.sets.filter((s) => s.session_id !== sessionId),
    cardio: log.cardio.filter((c) => c.session_id !== sessionId)
  };
}
