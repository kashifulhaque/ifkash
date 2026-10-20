import { getApiBase } from '$lib/apiBase';
import type { Group, GroupDetail, ShareInput, SplitMode } from '$lib/splitter';

// The signed-in Google ID token, held in module state and persisted so the
// session survives reloads. Friends never log in; this is the bookkeeper.
// API calls prefer a minted session (`ifkash_session`) when one is still
// valid: the Google token dies after about an hour, and the session does not.
export const SPLITTER_CRED_KEY = 'splitter_google_credential';
const SESSION_KEY = 'ifkash_session';

export type MintedSession = { token: string; exp: number; sub: string };

let googleToken: string | null = null;
let googleLoaded = false;
let session: MintedSession | null = null;
let sessionLoaded = false;

function jwtExpMs(token: string | null): number | null {
  if (!token) return null;
  try {
    const part = token.split('.')[1];
    if (!part) return null;
    const json = atob(part.replace(/-/g, '+').replace(/_/g, '/'));
    const payload = JSON.parse(json) as { exp?: unknown };
    return typeof payload.exp === 'number' ? payload.exp * 1000 : null;
  } catch {
    return null;
  }
}

export function setToken(t: string | null) {
  googleToken = t;
  googleLoaded = true;
  try {
    if (t) localStorage.setItem(SPLITTER_CRED_KEY, t);
    else localStorage.removeItem(SPLITTER_CRED_KEY);
  } catch {
    /* storage unavailable */
  }
  if (!t) clearSession();
}

/** The Google ID token, including one that has already expired. */
export function loadGoogleToken(): string | null {
  if (!googleLoaded) {
    googleLoaded = true;
    try {
      googleToken = localStorage.getItem(SPLITTER_CRED_KEY);
    } catch {
      googleToken = null;
    }
  }
  return googleToken;
}

export function loadSession(): MintedSession | null {
  if (!sessionLoaded) {
    sessionLoaded = true;
    try {
      const raw = localStorage.getItem(SESSION_KEY);
      session = raw ? (JSON.parse(raw) as MintedSession) : null;
    } catch {
      session = null;
    }
  }
  return session;
}

export function saveSession(next: MintedSession) {
  session = next;
  sessionLoaded = true;
  try {
    localStorage.setItem(SESSION_KEY, JSON.stringify(next));
  } catch {
    /* storage unavailable */
  }
}

export function clearSession() {
  session = null;
  sessionLoaded = true;
  try {
    localStorage.removeItem(SESSION_KEY);
  } catch {
    /* storage unavailable */
  }
}

/**
 * The bearer to send. A live session wins. Otherwise a Google ID token that
 * has not expired yet. `local-dev` is the Worker bypass and has no expiry.
 */
export function loadToken(): string | null {
  const minted = loadSession();
  if (minted && minted.token && minted.exp * 1000 > Date.now() + 5_000) return minted.token;
  const google = loadGoogleToken();
  if (!google) return null;
  if (google === 'local-dev') return google;
  const exp = jwtExpMs(google);
  if (exp === null || exp > Date.now() + 5_000) return google;
  return null;
}

/** Thrown on a 401 so the UI can drop the token and re-show the sign-in button. */
export class AuthError extends Error {}

/** The request never reached the server. The caller should keep local data. */
export class OfflineError extends Error {}

async function api<T>(path: string, init: RequestInit = {}): Promise<T> {
  const token = loadToken();
  if (!token) throw new AuthError('not signed in');
  let res: Response;
  try {
    res = await fetch(`${getApiBase()}/api/splitter${path}`, {
      ...init,
      headers: {
        ...(init.body ? { 'Content-Type': 'application/json' } : {}),
        Authorization: `Bearer ${token}`,
        ...(init.headers ?? {})
      }
    });
  } catch {
    throw new OfflineError('offline');
  }
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

export const splitterApi = {
  listGroups: () => api<Group[]>('/groups'),

  createGroup: (name: string, currency: string, members: string[]) =>
    api<{ id: number }>('/groups', {
      method: 'POST',
      body: JSON.stringify({ name, currency, members })
    }),

  getGroup: (id: number) => api<GroupDetail>(`/groups/${id}`),

  deleteGroup: (id: number) => api<{ id: number }>(`/groups/${id}`, { method: 'DELETE' }),

  addMember: (groupId: number, name: string) =>
    api<{ id: number }>(`/groups/${groupId}/members`, {
      method: 'POST',
      body: JSON.stringify({ name })
    }),

  deleteMember: (memberId: number) =>
    api<{ id: number }>(`/members/${memberId}`, { method: 'DELETE' }),

  addExpense: (
    groupId: number,
    payload: {
      payer_id: number;
      amount_cents: number;
      description: string;
      split_type: SplitMode;
      shares: ShareInput[];
    }
  ) =>
    api<{ id: number }>(`/groups/${groupId}/expenses`, {
      method: 'POST',
      body: JSON.stringify(payload)
    }),

  deleteExpense: (expenseId: number) =>
    api<{ id: number }>(`/expenses/${expenseId}`, { method: 'DELETE' })
};
