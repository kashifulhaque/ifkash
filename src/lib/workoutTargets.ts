// The working weight to load for each plan lift today, from the training log.
// Pure and I/O-free like `workoutReport.ts`; the logging view shows the result
// next to the scheme and prefills every set with it.
//
// Why: the log shows most lifts ramped inside the working sets, for example
// bench 15 × 8, 15 × 8, 17.5 × 8, 20 × 6. Only the last set or two were near
// the rep range, so half of each session's sets did little. The plan asks for
// straight sets, so this picks one load for all of them:
//   - After a session of straight sets, apply double progression: every set at
//     the top of the range means one step up; otherwise stay and add reps.
//   - After a ramped or partial session, estimate a max from the best recent
//     set (Epley) and take the load that max supports for the top of the range
//     plus one rep in reserve.
// Loads snap to weights already logged on that implement, because machine
// stacks and cable columns move in odd steps (3.75 kg, 2.27 kg) that rounding
// to 2.5 kg misses.

import { parseScheme, setScore, loadStep } from '$lib/workoutReport';
import type { Equipment, LogSet, TrainingLog } from '$lib/workout';

export type LoadTarget = {
  kg: number;
  /** One short sentence on where the number comes from. */
  reason: string;
  /** Warm-up loads for key lifts, lightest first. Empty for accessories. */
  warmups: number[];
};

// Names the plan renamed. History under the old name still counts, on the
// same implement only.
const ALIASES: Record<string, string[]> = {
  Squat: ['Barbell squat', 'Smith barbell squat'],
  'Bicep curls': ['Dumbbell bicep curls'],
  'Shoulder press': ['Shoulder press (machine)'],
  'Incline dumbbell press': ['Incline dumbbell press (adjustable bench)']
};

/** Sessions of a lift to read for the estimate. */
const RECENT_SESSIONS = 3;
/** Ignore history older than this; it says little about today's strength. */
const WINDOW_DAYS = 56;
/** After this long away from a lift, start 10 % lighter. */
const STALE_DAYS = 21;
/** A logged weight up to this far over the estimate still counts as a match. */
const SNAP_TOLERANCE = 1.05;

const MS_PER_DAY = 86_400_000;
const toDays = (iso: string): number => Math.round(Date.parse(iso + 'T00:00:00Z') / MS_PER_DAY);
const kgText = (kg: number): string => String(Math.round(kg * 100) / 100);

/**
 * The load for every working set of `exercise` on `equipment` today. Returns
 * null when the scheme has no rep range or the lift has no recent weighted
 * history on that implement.
 */
export function loadTarget(
  log: TrainingLog,
  exercise: string,
  equipment: Equipment,
  scheme: string,
  today: string,
  keyLift = false
): LoadTarget | null {
  const range = parseScheme(scheme);
  if (!range || !exercise) return null;

  const names = new Set([exercise, ...(ALIASES[exercise] ?? [])]);
  const matches = (s: LogSet) =>
    names.has(s.exercise) &&
    s.weight_g > 0 &&
    s.reps > 0 &&
    s.date < today &&
    (!equipment || !s.equipment || s.equipment === equipment);
  const all = log.sets.filter(matches);
  if (!all.length) return null;

  // Every load this lift has used on the implement: the grid to snap to.
  const grid = [...new Set(all.map((s) => s.weight_g / 1000))].sort((a, b) => a - b);

  const cutoff = toDays(today) - WINDOW_DAYS;
  const bySession = new Map<number, LogSet[]>();
  for (const s of all) {
    if (toDays(s.date) < cutoff) continue;
    const list = bySession.get(s.session_id) ?? [];
    list.push(s);
    bySession.set(s.session_id, list);
  }
  // `log.sets` is oldest first, so insertion order is session order.
  const sessions = [...bySession.values()].slice(-RECENT_SESSIONS);
  if (!sessions.length) return null;

  const last = sessions[sessions.length - 1];
  const daysAway = toDays(today) - toDays(last[0].date);
  const top = Math.max(...last.map((s) => s.weight_g)) / 1000;
  const working = last.filter((s) => s.weight_g / 1000 === top);
  const warm = (kg: number) => (keyLift ? warmups(kg, grid) : []);

  // Straight sets last time: double progression.
  if (working.length >= range.sets && daysAway <= STALE_DAYS) {
    if (working.every((s) => s.reps >= range.hi)) {
      const kg = stepUp(top, grid, equipment);
      return {
        kg,
        reason: `Every set reached ${range.hi} at ${kgText(top)} kg last time, so go up a step.`,
        warmups: warm(kg)
      };
    }
    const avg = working.reduce((n, s) => n + s.reps, 0) / working.length;
    if (working.every((s) => s.reps < range.lo) && avg <= range.lo - 2) {
      const kg = stepDown(top, grid, equipment);
      return {
        kg,
        reason: `Every set was well under ${range.lo} at ${kgText(top)} kg last time, so drop a step.`,
        warmups: warm(kg)
      };
    }
    return {
      kg: top,
      reason: `Stay at ${kgText(top)} kg and add reps until every set reaches ${range.hi}.`,
      warmups: warm(top)
    };
  }

  // Ramped or partial: estimate from the best recent set.
  let best: LogSet | null = null;
  for (const s of sessions.flat()) {
    if (!best || setScore('weighted', s.reps, s.weight_g) > setScore('weighted', best.reps, best.weight_g)) best = s;
  }
  if (!best) return null;
  const max = setScore('weighted', best.reps, best.weight_g);
  let raw = max / (1 + (range.hi + 1) / 30);
  let reason = `Estimated from your best recent set, ${kgText(best.weight_g / 1000)} kg × ${best.reps}. Use it for every set.`;
  if (daysAway > STALE_DAYS) {
    raw *= 0.9;
    reason += ` Lowered 10 % because you last did this ${daysAway} days ago.`;
  }
  const kg = snap(raw, grid);
  return { kg, reason, warmups: warm(kg) };
}

/** The heaviest logged load at or just over `kg`, or `kg` rounded down to 2.5. */
function snap(kg: number, grid: number[]): number {
  const fit = grid.filter((w) => w <= kg * SNAP_TOLERANCE);
  if (fit.length) return fit[fit.length - 1];
  return Math.max(2.5, Math.floor(kg / 2.5) * 2.5);
}

/** The next logged load up, if it's a sensible jump; otherwise one plate step. */
function stepUp(kg: number, grid: number[], equipment: Equipment): number {
  const next = grid.find((w) => w > kg + 0.5);
  if (next !== undefined && next <= kg * 1.15) return next;
  return Math.round((kg + loadStep(equipment)) * 2) / 2;
}

function stepDown(kg: number, grid: number[], equipment: Equipment): number {
  const lower = grid.filter((w) => w < kg - 0.5);
  if (lower.length) return lower[lower.length - 1];
  return Math.max(0, Math.round((kg - loadStep(equipment)) * 2) / 2);
}

/**
 * Two warm-up loads, about 50 % and 75 % of the working load, for 8 and then
 * 5 reps. Warm-ups aren't logged, so they don't inflate the set count. Light
 * working loads need one warm-up at most.
 */
function warmups(kg: number, grid: number[]): number[] {
  const pick = (target: number) => {
    const fit = grid.filter((w) => w <= target);
    return fit.length ? fit[fit.length - 1] : Math.floor(target / 2.5) * 2.5;
  };
  const out = [pick(kg * 0.5), pick(kg * 0.75)].filter((w) => w > 0 && w < kg);
  return kg < 20 ? [...new Set(out)].slice(-1) : [...new Set(out)];
}
