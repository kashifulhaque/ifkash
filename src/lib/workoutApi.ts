import { getApiBase } from '$lib/apiBase';
import { loadToken, AuthError } from '$lib/splitterApi';
import type { SessionSummary, SessionDetail, BodyweightEntry } from '$lib/workout';

// The tracker reuses the splitter's Google sign-in token (same localStorage key,
// same auth flow) so the user signs in once across both tools. Auth helpers
// (setToken/loadToken/AuthError) live in splitterApi.ts.

async function api<T>(path: string, init: RequestInit = {}): Promise<T> {
  const token = loadToken();
  if (!token) throw new AuthError('not signed in');
  const res = await fetch(`${getApiBase()}/api/workout${path}`, {
    ...init,
    headers: {
      ...(init.body ? { 'Content-Type': 'application/json' } : {}),
      Authorization: `Bearer ${token}`,
      ...(init.headers ?? {})
    }
  });
  if (res.status === 401) {
    throw new AuthError('session expired');
  }
  if (!res.ok) {
    let msg = `request failed (${res.status})`;
    try {
      const text = await res.text();
      if (text) msg = text;
    } catch {
      /* ignore */
    }
    throw new Error(msg);
  }
  return (await res.json()) as T;
}

export type ExercisePayload = {
  exercise: string;
  sets: { reps: number; weight_g: number }[];
};

export type CardioPayload = {
  kind: string;
  minutes: number;
  kcal: number;
};

export const workoutApi = {
  listSessions: () => api<SessionSummary[]>('/sessions'),

  createSession: (payload: {
    day_label: string;
    date: string;
    notes: string;
    exercises: ExercisePayload[];
    cardio?: CardioPayload[];
  }) =>
    api<{ id: number }>('/sessions', {
      method: 'POST',
      body: JSON.stringify(payload)
    }),

  /** Upsert the session for (date, day_label) — replaces any existing one for
   *  that day. Used by the merged workout page's auto-save. */
  upsertSession: (payload: {
    day_label: string;
    date: string;
    notes: string;
    exercises: ExercisePayload[];
    cardio?: CardioPayload[];
  }) =>
    api<{ id: number }>('/sessions', {
      method: 'PUT',
      body: JSON.stringify(payload)
    }),

  getSession: (id: number) => api<SessionDetail>(`/sessions/${id}`),

  deleteSession: (id: number) =>
    api<{ id: number }>(`/sessions/${id}`, { method: 'DELETE' }),

  listBodyweight: () => api<BodyweightEntry[]>('/bodyweight'),

  addBodyweight: (date: string, weight_g: number) =>
    api<{ id: number }>('/bodyweight', {
      method: 'POST',
      body: JSON.stringify({ date, weight_g })
    }),

  deleteBodyweight: (id: number) =>
    api<{ id: number }>(`/bodyweight/${id}`, { method: 'DELETE' })
};
