// Local copy of the workout log, plus the edits that have not reached the
// server yet. The gym connection drops; a set typed there has to survive a
// refresh and upload when the network comes back. Drafts win over the cached
// server copy for the same date and day, because they are the newer edit.

import type { Profile } from '$lib/fitnessMetrics';
import type { ExercisePayload, CardioPayload } from '$lib/workoutApi';
import type { BodyweightEntry, SessionDetail, SessionSummary, TrainingLog } from '$lib/workout';

const DRAFT_KEY = 'ifkash_workout_drafts';
const BW_KEY = 'ifkash_workout_bw';
const CACHE_KEY = 'ifkash_workout_cache';
const MAX_DETAILS = 40;

export type SessionDraft = {
  date: string;
  day_label: string;
  exercises: ExercisePayload[];
  cardio: CardioPayload[];
  /** Milliseconds. A sync deletes the draft only when this still matches. */
  updated: number;
};

export type BwDraft = { date: string; weight_g: number; updated: number };

export type WorkoutCache = {
  sessions: SessionSummary[];
  bodyweight: BodyweightEntry[];
  log: TrainingLog;
  details: SessionDetail[];
  profile: Profile | null;
};

const EMPTY_CACHE: WorkoutCache = {
  sessions: [],
  bodyweight: [],
  log: { sets: [], cardio: [] },
  details: [],
  profile: null
};

function readJson<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function writeJson(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* quota or private mode — the in-memory copy still holds this visit */
  }
}

export function listDrafts(): SessionDraft[] {
  return readJson<SessionDraft[]>(DRAFT_KEY, []);
}

export function draftFor(date: string, day: string): SessionDraft | undefined {
  return listDrafts().find((d) => d.date === date && d.day_label === day);
}

export function hasPending(): boolean {
  return listDrafts().length > 0 || listBwDrafts().length > 0;
}

/** Write a draft, or drop it when the day has nothing left to sync. */
export function putDraft(draft: SessionDraft | null, key: { date: string; day: string }) {
  const rest = listDrafts().filter((d) => !(d.date === key.date && d.day_label === key.day));
  const keep =
    draft && (draft.exercises.length > 0 || draft.cardio.length > 0) ? [...rest, draft] : rest;
  writeJson(DRAFT_KEY, keep);
}

/**
 * Remove the draft only if the device has not recorded a newer edit while
 * the upload was in flight.
 */
export function ackDraft(date: string, day: string, updated: number) {
  const current = draftFor(date, day);
  if (current && current.updated !== updated) return;
  putDraft(null, { date, day });
}

export function listBwDrafts(): BwDraft[] {
  return readJson<BwDraft[]>(BW_KEY, []);
}

export function bwDraftFor(date: string): BwDraft | undefined {
  return listBwDrafts().find((d) => d.date === date);
}

export function putBwDraft(draft: BwDraft) {
  const rest = listBwDrafts().filter((d) => d.date !== draft.date);
  writeJson(BW_KEY, [...rest, draft]);
}

export function ackBwDraft(date: string, updated: number) {
  const current = bwDraftFor(date);
  if (current && current.updated !== updated) return;
  writeJson(
    BW_KEY,
    listBwDrafts().filter((d) => d.date !== date)
  );
}

/** Stable negative id so a not-yet-uploaded day can sit in the session list. */
export function draftSessionId(date: string, day: string): number {
  let h = 0;
  const s = `${date}|${day}`;
  for (let i = 0; i < s.length; i++) h = (Math.imul(h, 31) + s.charCodeAt(i)) | 0;
  return h < 0 ? h : -h - 1;
}

/** Sessions from the server, with any unsynced day added so the week chip ticks. */
export function mergeDraftSessions(sessions: SessionSummary[]): SessionSummary[] {
  const out = [...sessions];
  for (const d of listDrafts()) {
    if (!d.exercises.length && !d.cardio.length) continue;
    if (out.some((s) => s.date === d.date && s.day_label === d.day_label)) continue;
    out.unshift({
      id: draftSessionId(d.date, d.day_label),
      day_label: d.day_label,
      date: d.date,
      notes: '',
      created: new Date(d.updated).toISOString()
    });
  }
  return out;
}

export function draftDetail(draft: SessionDraft): SessionDetail {
  const sets = draft.exercises.flatMap((ex) =>
    ex.sets.map((s, i) => ({
      id: 0,
      exercise: ex.exercise,
      equipment: ex.equipment,
      set_index: i + 1,
      reps: s.reps,
      weight_g: s.weight_g
    }))
  );
  return {
    session: {
      id: draftSessionId(draft.date, draft.day_label),
      day_label: draft.day_label,
      date: draft.date,
      notes: '',
      created: new Date(draft.updated).toISOString()
    },
    sets,
    cardio: draft.cardio.map((c, i) => ({
      id: 0,
      kind: c.kind,
      minutes: c.minutes,
      kcal: c.kcal,
      entry_index: i + 1
    }))
  };
}

export function readCache(): WorkoutCache {
  const cached = readJson<Partial<WorkoutCache>>(CACHE_KEY, {});
  return {
    sessions: cached.sessions ?? [],
    bodyweight: cached.bodyweight ?? [],
    log: cached.log ?? EMPTY_CACHE.log,
    details: cached.details ?? [],
    profile: cached.profile ?? null
  };
}

export function writeCache(next: WorkoutCache) {
  writeJson(CACHE_KEY, next);
}

export function cacheDetail(detail: SessionDetail, cache: WorkoutCache): WorkoutCache {
  const details = [detail, ...cache.details.filter((d) => d.session.id !== detail.session.id)].slice(
    0,
    MAX_DETAILS
  );
  return { ...cache, details };
}

export function cachedDetail(id: number): SessionDetail | undefined {
  return readCache().details.find((d) => d.session.id === id);
}
