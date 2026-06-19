// Pure body-metric math shared by the workout and meals pages. No auth, no I/O —
// just the profile shape plus BMI / BMR / TDEE and daily nutrition targets.

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
  activity: 1.55,
  goal: 'cut_moderate',
  latest_weight_g: null
};

// kcal/day removed from (or added to) TDEE per goal.
const GOAL_DEFICIT: Record<Goal, number> = {
  cut_moderate: 750,
  cut_aggressive: 1000,
  maintain: 0
};

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
