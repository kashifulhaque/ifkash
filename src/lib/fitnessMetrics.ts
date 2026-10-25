// Pure body-metric math shared by the fitness pages. No auth, no I/O — just the
// profile shape plus BMI / BMR / TDEE, daily nutrition targets, and the MET
// and per-set burn estimates.

import type { Equipment, ExerciseKind } from '$lib/workout';

export type Sex = 'male' | 'female';
export type Goal = 'cut_moderate' | 'cut_aggressive' | 'maintain';

export type Profile = {
  height_cm: number;
  sex: Sex;
  age_years: number;
  activity: number; // TDEE multiplier over BMR
  goal: Goal;
  latest_weight_g?: number | null; // most recent logged bodyweight (grams)
};

export const DEFAULT_PROFILE: Profile = {
  height_cm: 179,
  sex: 'male',
  age_years: 28,
  activity: 1.375, // Light — lifts daily but otherwise desk/sedentary (low NEAT)
  goal: 'cut_moderate',
  latest_weight_g: null
};

// kcal/day removed from (or added to) TDEE per goal.
const GOAL_DEFICIT: Record<Goal, number> = {
  cut_moderate: 750,
  cut_aggressive: 1000,
  maintain: 0
};

/** kcal/day the goal asks to be removed from TDEE (0 when maintaining). */
export function goalDeficit(goal: Goal): number {
  return GOAL_DEFICIT[goal] ?? 0;
}

export const GOAL_LABELS: Record<Goal, string> = {
  cut_moderate: 'Moderate cut (−750 kcal)',
  cut_aggressive: 'Aggressive cut (−1000 kcal)',
  maintain: 'Maintain'
};

export const ACTIVITY_OPTIONS: { value: number; label: string }[] = [
  { value: 1.2, label: 'Sedentary (×1.2)' },
  { value: 1.375, label: 'Light (×1.375)' },
  { value: 1.55, label: 'Moderate (×1.55)' },
  { value: 1.725, label: 'Very active (×1.725)' }
];

export type Metrics = {
  bmi: number;
  bmiCategory: string;
  bmr: number;
  tdee: number;
};

/** BMI category from the standard WHO cutoffs. */
function bmiCategory(bmi: number): string {
  if (bmi < 18.5) return 'underweight';
  if (bmi < 25) return 'normal';
  if (bmi < 30) return 'overweight';
  return 'obese';
}

/** BMI / BMR (Mifflin-St Jeor) / TDEE for a bodyweight in kg. */
export function computeMetrics(weightKg: number, p: Profile): Metrics {
  const heightM = p.height_cm / 100;
  const bmi = heightM > 0 ? weightKg / (heightM * heightM) : 0;
  const bmr =
    10 * weightKg + 6.25 * p.height_cm - 5 * p.age_years + (p.sex === 'male' ? 5 : -161);
  const tdee = bmr * p.activity;
  return {
    bmi: Math.round(bmi * 10) / 10,
    bmiCategory: bmiCategory(bmi),
    bmr: Math.round(bmr),
    tdee: Math.round(tdee)
  };
}

// ---- energy expenditure (MET-based) ----------------------------------------

/**
 * Calories burnt for `minutes` of activity at a given MET, for a bodyweight in
 * kg. Standard MET formula: kcal = MET × 3.5 × kg / 200 × minutes. Rounded to a
 * whole kcal; returns 0 for non-positive weight or duration.
 */
export function metKcal(met: number, weightKg: number, minutes: number): number {
  if (weightKg <= 0 || minutes <= 0 || met <= 0) return 0;
  return Math.round(((met * 3.5 * weightKg) / 200) * minutes);
}

// ---- strength-set burn ------------------------------------------------------

// A rough burn for one logged set, from what the set actually was: the load,
// the reps (or seconds for a hold), the implement, and the lifter's
// bodyweight. Approximate by design. The goal is that a heavy squat set
// reads higher than a light lateral raise, not a number to trust to the kcal.
//
// Per set, three parts, all gross (resting metabolism included, like every
// MET figure in this file):
//   1. Active time: reps × SECONDS_PER_REP (or the hold's seconds) at the
//      movement's MET. Big compounds sit near 6 MET, isolation near 3.5.
//   2. Mechanical work: (load + the share of bodyweight the lift also moves)
//      × reps × range of motion × g, divided by muscle efficiency, in kcal,
//      times an implement factor (a guided machine or Smith path asks less of
//      the stabilisers than free weights).
//   3. Rest: the rest that follows the set at REST_MET, using the plan's rest
//      times (about 2 min after compounds, 1 min after isolation).
//
// Dumbbell loads are logged per dumbbell, so a two-dumbbell lift counts twice
// the logged weight. Per-side moves (split squats, dead bugs, side planks)
// are logged per side and count twice.

/** Used when the app has no bodyweight for the user yet. */
export const DEFAULT_BODY_KG = 85;
/** Lifting tempo: about 1 s up, 2 s down, a beat at the ends. */
const SECONDS_PER_REP = 3.5;
/** Standing or sitting between sets. */
const REST_MET = 1.5;
/** Share of the metabolic energy that ends up as mechanical work. */
const MUSCLE_EFFICIENCY = 0.2;
const G = 9.81;
const J_PER_KCAL = 4184;
/** Assumed when a set has a load but no reps (or a hold with no seconds). */
const DEFAULT_REPS = 10;
const DEFAULT_HOLD_S = 30;

/** Mechanical-work multiplier per implement. */
const EQUIPMENT_FACTOR: Record<Equipment, number> = {
  '': 1,
  Barbell: 1,
  Dumbbell: 1,
  Bodyweight: 1,
  Smith: 0.9,
  Cable: 0.9,
  Machine: 0.85
};

type MoveProfile = {
  /** MET while the set is under way. */
  met: number;
  /** How far the load travels per rep, metres. */
  rom: number;
  /** Share of bodyweight lifted along with the load (torso in a squat). */
  body: number;
  /** Rest credited after the set, seconds. */
  rest: number;
  /** Logged per side, so the work happens twice. */
  sides?: 2;
};

// First match wins, so the specific patterns sit above the general ones
// ("leg curl" before "curl", "split squat" before "squat").
const MOVES: [RegExp, MoveProfile][] = [
  [/calf/, { met: 4, rom: 0.08, body: 0, rest: 60 }],
  [/side plank/, { met: 3.8, rom: 0, body: 0, rest: 45, sides: 2 }],
  [/plank|hold|hang\b/, { met: 3.8, rom: 0, body: 0, rest: 60 }],
  [/dead ?bug|bird ?dog/, { met: 3.8, rom: 0.3, body: 0.15, rest: 45, sides: 2 }],
  [/leg raise|knee raise/, { met: 3.8, rom: 0.4, body: 0.3, rest: 60 }],
  [/crunch|sit-?up/, { met: 3.8, rom: 0.25, body: 0.3, rest: 60 }],
  [/back extension|hyperextension/, { met: 4.5, rom: 0.4, body: 0.5, rest: 75 }],
  [/split squat|lunge|step-?up/, { met: 6, rom: 0.45, body: 0.6, rest: 90, sides: 2 }],
  [/squat/, { met: 6, rom: 0.6, body: 0.7, rest: 150 }],
  [/deadlift|rdl/, { met: 6, rom: 0.5, body: 0.4, rest: 150 }],
  [/leg press/, { met: 6, rom: 0.5, body: 0, rest: 120 }],
  [/leg extension|leg curl/, { met: 4, rom: 0.4, body: 0, rest: 75 }],
  [/push-?up/, { met: 5, rom: 0.35, body: 0.65, rest: 90 }],
  [/bench|incline press|chest press|dip/, { met: 5.5, rom: 0.4, body: 0, rest: 120 }],
  [/row/, { met: 5.5, rom: 0.45, body: 0, rest: 120 }],
  [/pull-?up|chin-?up/, { met: 6, rom: 0.5, body: 1, rest: 120 }],
  [/pulldown/, { met: 5, rom: 0.55, body: 0, rest: 120 }],
  [/shoulder press|overhead press|military/, { met: 5, rom: 0.5, body: 0, rest: 120 }],
  [/fly|pec deck|face pull|rear delt/, { met: 3.5, rom: 0.4, body: 0, rest: 60 }],
  [/lateral|front raise/, { met: 3.5, rom: 0.3, body: 0, rest: 60 }],
  [/curl/, { met: 3.5, rom: 0.35, body: 0, rest: 60 }],
  [/tricep|pushdown|extension|skull/, { met: 3.5, rom: 0.35, body: 0, rest: 60 }]
];
/** Anything unrecognised: a middling accessory. */
const DEFAULT_MOVE: MoveProfile = { met: 4.5, rom: 0.4, body: 0, rest: 90 };

/** A lift that takes one dumbbell (or one per side, logged as such). */
const ONE_DUMBBELL = /single|one[- ]arm|goblet/;

export type SetForKcal = {
  exercise: string;
  equipment: Equipment;
  kind: ExerciseKind;
  /** Reps, or seconds for a timed hold. 0 when not entered. */
  reps: number;
  /** Logged load in grams (per dumbbell for dumbbells). 0 when not entered. */
  weight_g: number;
};

/** kcal per minute at a MET for a bodyweight: MET × 3.5 × kg / 200. */
const kcalPerMin = (met: number, kg: number) => (met * 3.5 * kg) / 200;

/**
 * Estimated kcal for one set, unrounded so a session can be summed and
 * rounded once. `bodyKg` falls back to `DEFAULT_BODY_KG` when not positive.
 */
export function setKcal(set: SetForKcal, bodyKg: number): number {
  const kg = bodyKg > 0 ? bodyKg : DEFAULT_BODY_KG;
  const name = set.exercise.toLowerCase();
  const move = MOVES.find(([re]) => re.test(name))?.[1] ?? DEFAULT_MOVE;
  const sides = move.sides ?? 1;

  const timed = set.kind === 'time';
  const count = set.reps > 0 ? set.reps : timed ? DEFAULT_HOLD_S : DEFAULT_REPS;
  const activeS = (timed ? count : count * SECONDS_PER_REP) * sides;
  const active = kcalPerMin(move.met, kg) * (activeS / 60);
  const rest = kcalPerMin(REST_MET, kg) * (move.rest / 60);
  if (timed) return active + rest;

  // Bodyweight-kind moves never carry a logged load; weighted ones might not.
  const loggedKg = set.kind === 'weighted' ? Math.max(0, set.weight_g) / 1000 : 0;
  const hands = set.equipment === 'Dumbbell' && !ONE_DUMBBELL.test(name) ? 2 : 1;
  const massKg = loggedKg * hands + move.body * kg;
  const joules = massKg * count * sides * move.rom * G;
  const work = (joules / MUSCLE_EFFICIENCY / J_PER_KCAL) * (EQUIPMENT_FACTOR[set.equipment] ?? 1);
  return active + work + rest;
}

/** Rounded kcal for a list of sets. */
export function setsKcal(sets: SetForKcal[], bodyKg: number): number {
  return Math.round(sets.reduce((n, s) => n + setKcal(s, bodyKg), 0));
}

export type NutritionTargets = {
  calories: number;
  protein_g: number;
  carbs_g: number;
  fat_g: number;
};

/**
 * Daily nutrition targets for a bodyweight (kg):
 * calories = TDEE − goal deficit, protein 2.0 g/kg, fat 25% of calories,
 * carbs fill the remainder.
 */
export function nutritionTargets(weightKg: number, p: Profile): NutritionTargets {
  const { tdee } = computeMetrics(weightKg, p);
  const calories = Math.max(1000, Math.round(tdee - GOAL_DEFICIT[p.goal]));
  const protein_g = Math.round(2.0 * weightKg);
  const fat_g = Math.round((0.25 * calories) / 9);
  const carbs_g = Math.max(0, Math.round((calories - protein_g * 4 - fat_g * 9) / 4));
  return { calories, protein_g, carbs_g, fat_g };
}
