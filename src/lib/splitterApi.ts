import { getApiBase } from '$lib/apiBase';
import type { Group, GroupDetail, ShareInput, SplitMode } from '$lib/splitter';

// The signed-in Google ID token, held in module state and persisted so the
// session survives reloads. Friends never log in; this is the bookkeeper.
export const SPLITTER_CRED_KEY = 'splitter_google_credential';

let token: string | null = null;

export function setToken(t: string | null) {
  token = t;
  try {
    if (t) localStorage.setItem(SPLITTER_CRED_KEY, t);
    else localStorage.removeItem(SPLITTER_CRED_KEY);
  } catch {
    /* storage unavailable */
  }
}

export function loadToken(): string | null {
  if (token) return token;
  try {
    token = localStorage.getItem(SPLITTER_CRED_KEY);
  } catch {
    token = null;
  }
  return token;
}

/** Thrown on a 401 so the UI can drop the token and re-show the sign-in button. */
export class AuthError extends Error {}

async function api<T>(path: string, init: RequestInit = {}): Promise<T> {
  if (!token) throw new AuthError('not signed in');
  const res = await fetch(`${getApiBase()}/api/splitter${path}`, {
    ...init,
    headers: {
      ...(init.body ? { 'Content-Type': 'application/json' } : {}),
      Authorization: `Bearer ${token}`,
      ...(init.headers ?? {})
    }
  });
  if (res.status === 401) {
    setToken(null);
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
