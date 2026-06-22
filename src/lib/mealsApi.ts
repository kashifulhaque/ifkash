import { getApiBase } from '$lib/apiBase';

// The signed-in Google ID token, held in module state and persisted so the
// session survives reloads. Mirrors the splitter client.
export const MEALS_CRED_KEY = 'meals_google_credential';

let token: string | null = null;

export function setToken(t: string | null) {
  token = t;
  try {
    if (t) localStorage.setItem(MEALS_CRED_KEY, t);
    else localStorage.removeItem(MEALS_CRED_KEY);
  } catch {
    /* storage unavailable */
  }
}

export function loadToken(): string | null {
  if (token) return token;
  try {
    token = localStorage.getItem(MEALS_CRED_KEY);
  } catch {
    token = null;
  }
  return token;
}

/** Thrown on a 401 so the UI can drop the token and re-show the sign-in button. */
export class AuthError extends Error {}

export type MealItem = {
  name: string;
  grams?: number;
  calories?: number;
  protein_g?: number;
  carbs_g?: number;
  fat_g?: number;
};

export type Meal = {
  id: number;
  eaten_on: string;
  eaten_at: string;
  description: string | null;
  calories: number | null;
  protein_g: number | null;
  carbs_g: number | null;
  fat_g: number | null;
  items: MealItem[];
  photo_r2_key: string | null;
};

export type Totals = {
  calories: number;
  protein_g: number;
  carbs_g: number;
  fat_g: number;
};

export type DayResponse = { meals: Meal[]; totals: Totals };

export type DaySummary = {
  eaten_on: string;
  calories: number;
  protein_g: number;
  carbs_g: number;
  fat_g: number;
  count: number;
};

export type MealPatch = {
  description?: string;
  calories?: number;
  protein_g?: number;
  carbs_g?: number;
  fat_g?: number;
};

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  if (!token) throw new AuthError('not signed in');
  const res = await fetch(`${getApiBase()}/api/meals${path}`, {
    ...init,
    headers: {
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

export const mealsApi = {
  // photo is a JPEG Blob (already downscaled client-side).
  analyze: (photo: Blob, hint: string, date: string) => {
    const fd = new FormData();
    fd.append('photo', photo, 'meal.jpg');
    fd.append('eaten_on', date);
    if (hint.trim()) fd.append('hint', hint.trim());
    // No Content-Type header: the browser sets the multipart boundary.
    return request<Meal>('/analyze', { method: 'POST', body: fd });
  },

  list: (date: string) => request<DayResponse>(`?date=${encodeURIComponent(date)}`),

  /** Recent days with per-day totals, newest first — powers the history list. */
  listDays: (limit = 14) => request<DaySummary[]>(`/days?limit=${limit}`),

  /** Fetch a meal's photo with the bearer token and return an object URL.
   *  An <img src> can't send the Authorization header, so we go via fetch. */
  photoUrl: async (id: number): Promise<string | null> => {
    if (!token) throw new AuthError('not signed in');
    const res = await fetch(`${getApiBase()}/api/meals/${id}/photo`, {
      headers: { Authorization: `Bearer ${token}` }
    });
    if (res.status === 401) {
      setToken(null);
      throw new AuthError('session expired');
    }
    if (!res.ok) return null;
    return URL.createObjectURL(await res.blob());
  },

  update: (id: number, patch: MealPatch) =>
    request<{ id: number }>(`/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(patch)
    }),

  remove: (id: number) => request<{ id: number }>(`/${id}`, { method: 'DELETE' })
};
