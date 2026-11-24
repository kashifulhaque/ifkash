// Picks the training signals worth a research note on the insights page. Pure: no I/O.
// The Worker (api/src/handlers/fitness_research.rs) turns each signal into a fixed
// question and answers it from evidence-based sources through Exa.

import type { ResearchSignal } from '$lib/exaApi';
import type { RateBand } from '$lib/fitnessInsights';
import type { BodyweightEntry, TrainingLog } from '$lib/workout';
import { buildReport, periodRange, progressions, shiftAnchor } from '$lib/workoutReport';

/** A signal plus the line the page shows above its note: what in the log raised it. */
export type Suggestion = { signal: ResearchSignal; why: string };

const MAX = 3;

/**
 * Up to three suggestions, most pressing first: a long break, a stalled key lift,
 * the rate of weight change, the most under-trained muscle last week, and low cardio.
 */
export function researchSuggestions(input: {
  log: TrainingLog;
  bodyweight: BodyweightEntry[];
  today: string;
  band: RateBand | null;
  daysSinceLast: number | null;
}): Suggestion[] {
  const { log, bodyweight, today, band, daysSinceLast } = input;
  const out: Suggestion[] = [];

  if (daysSinceLast !== null && daysSinceLast >= 7) {
    out.push({
      signal: { kind: 'break', days: daysSinceLast },
      why: `${daysSinceLast} days since your last session.`
    });
  }

  const stall = progressions(log)
    .filter((p) => p.kind === 'stall')
    .sort((a, b) => Number(b.key_lift) - Number(a.key_lift))[0];
  if (stall) {
    out.push({
      signal: { kind: 'stall', exercise: stall.exercise },
      why: `Your best ${stall.exercise} set hasn't moved in four sessions.`
    });
  }

  if (band === 'aggressive' || band === 'very fast') {
    out.push({
      signal: { kind: 'weight_rate', band },
      why: 'You are losing more than 1 % of your bodyweight a week.'
    });
  } else if (band === 'holding' || band === 'slow') {
    out.push({
      signal: { kind: 'weight_rate', band },
      why: band === 'holding' ? 'Your weight trend is flat this month.' : 'You are losing under 0.5 % a week.'
    });
  }

  // Last full week, so a week still in progress doesn't read as under-trained.
  const lastWeek = periodRange('week', shiftAnchor(periodRange('week', today, today), -1), today);
  const report = buildReport(log, lastWeek, bodyweight);
  if (report.liftSessions > 0) {
    const low = report.muscles
      .filter((m) => m.target > 0 && m.sets < m.target * 0.6)
      .sort((a, b) => a.sets / a.target - b.sets / b.target)[0];
    if (low) {
      out.push({
        signal: { kind: 'low_volume', muscle: low.muscle },
        why: `${cap(low.muscle)} got ${round(low.sets)} of ${round(low.target)} planned sets last week.`
      });
    }
    if (report.cardioTarget > 0 && report.cardioMin < report.cardioTarget * 0.5) {
      out.push({
        signal: { kind: 'low_cardio' },
        why: `${Math.round(report.cardioMin)} of ${Math.round(report.cardioTarget)} planned cardio minutes last week.`
      });
    }
  }

  return out.slice(0, MAX);
}

const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
const round = (v: number) => Math.round(v * 10) / 10;
