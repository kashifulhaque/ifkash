import { getApiBase } from '$lib/apiBase';
import type { Profile } from '$lib/fitnessMetrics';

// Client for the /api/profile endpoint. The token is passed in (not read from a
// fixed key) so each page can use its own Google credential — the workout page's
// splitter token or the meals page's meals token — both resolving the same user
// server-side.

/** Thrown on a 401 so the caller can drop the token and re-show sign-in. */
export class ProfileAuthError extends Error {}

async function request<T>(token: string, init: RequestInit = {}): Promise<T> {
  const res = await fetch(`${getApiBase()}/api/profile`, {
    ...init,
    headers: {
      ...(init.body ? { 'Content-Type': 'application/json' } : {}),
      Authorization: `Bearer ${token}`,
      ...(init.headers ?? {})
    }
  });
  if (res.status === 401) throw new ProfileAuthError('session expired');
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

export type ProfilePayload = Pick<Profile, 'height_cm' | 'sex' | 'age_years' | 'activity' | 'goal'>;

export const profileApi = {
  get: (token: string) => request<Profile>(token),

  save: (token: string, profile: ProfilePayload) =>
    request<Profile>(token, { method: 'PUT', body: JSON.stringify(profile) })
};
