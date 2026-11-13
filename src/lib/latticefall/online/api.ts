// The Worker calls for the online features, under `/api/latticefall/`. Reads need no
// sign-in. Writes send the bearer from `loadToken` (the same one the splitter and fitness
// pages use) and throw `AuthError` when there isn't one or the server says 401. No Three.js
// or WASM import. The features are optional: callers treat every failure as "no online
// data" and carry on.

import { getApiBase } from '$lib/apiBase';
import { AuthError, OfflineError, loadToken } from '$lib/splitterApi';
import { anonId } from './ids';

export { AuthError, OfflineError };

/** The server said the write was rate limited. Retry later. */
export class RateLimitError extends Error {}

/** One recorded sample: `[x, y, z, moveId]`. */
export type Tick = [number, number, number, number];

export interface OnlineMessage {
  id: number;
  area: string;
  x: number;
  y: number;
  z: number;
  template_id: number;
  word_ids: number[];
  rating: number;
}

export interface OnlineAfterimage {
  id: number;
  area: string;
  x: number;
  y: number;
  z: number;
  cause: string;
  /** At most 150 ticks. */
  snippet: Tick[];
}

/** A Mark and when it was earned, in milliseconds since the epoch. */
export interface OnlineMark {
  id: string;
  at: number;
}

export interface CloudSave {
  /** Base64 of the `LfWorld.save()` blob. */
  data: string;
  scenario: string;
  /** The world's seed as a decimal string (a u64). */
  seed: string;
  hash: string;
  saved_at: number;
}

export interface SpeedrunEntry {
  name: string;
  picture: string | null;
  best_ms: number;
}

const BASE = '/api/latticefall';

async function request(path: string, init: RequestInit, signedIn: boolean): Promise<Response> {
  const headers: Record<string, string> = {};
  if (init.body) headers['Content-Type'] = 'application/json';
  if (signedIn) {
    const token = loadToken();
    if (!token) throw new AuthError('not signed in');
    headers.Authorization = `Bearer ${token}`;
  }
  let res: Response;
  try {
    res = await fetch(`${getApiBase()}${BASE}${path}`, { ...init, headers });
  } catch {
    throw new OfflineError('offline');
  }
  if (res.status === 401) throw new AuthError('session expired');
  if (res.status === 429) throw new RateLimitError('rate limited');
  return res;
}

async function json<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error((await res.text().catch(() => '')) || `request failed (${res.status})`);
  return (await res.json()) as T;
}

const q = (area: string) => `?area=${encodeURIComponent(area)}`;

// Reads (anonymous)

export async function fetchMessages(area: string): Promise<OnlineMessage[]> {
  const body = await json<{ messages: OnlineMessage[] }>(await request(`/messages${q(area)}`, {}, false));
  return body.messages;
}

export async function fetchAfterimages(area: string): Promise<OnlineAfterimage[]> {
  const body = await json<{ afterimages: OnlineAfterimage[] }>(
    await request(`/afterimages${q(area)}`, {}, false)
  );
  return body.afterimages;
}

/** Position trails of other players, with no identity. */
export async function fetchWanderers(area: string): Promise<Tick[][]> {
  const body = await json<{ wanderers: Tick[][] }>(await request(`/wanderers${q(area)}`, {}, false));
  return body.wanderers.filter((t) => Array.isArray(t));
}

export async function fetchSpeedrunLeaderboard(): Promise<SpeedrunEntry[]> {
  const body = await json<{ entries: SpeedrunEntry[] }>(await request('/speedrun/leaderboard', {}, false));
  return body.entries;
}

// Writes (need a token)

export async function postMessage(m: {
  area: string;
  x: number;
  y: number;
  z: number;
  templateId: number;
  wordIds: number[];
}): Promise<number | null> {
  const body = await json<{ id: number | null }>(
    await request(
      '/messages',
      {
        method: 'POST',
        body: JSON.stringify({
          anon_id: anonId(),
          area: m.area,
          x: m.x,
          y: m.y,
          z: m.z,
          template_id: m.templateId,
          word_ids: m.wordIds
        })
      },
      true
    )
  );
  return body.id;
}

/** Rates a message; returns its new rating. */
export async function rateMessage(id: number, value: 1 | -1): Promise<number> {
  const body = await json<{ rating: number }>(
    await request(`/messages/${id}/rate`, { method: 'POST', body: JSON.stringify({ value }) }, true)
  );
  return body.rating;
}

export async function postAfterimage(a: {
  area: string;
  x: number;
  y: number;
  z: number;
  cause: string;
  snippet: Tick[];
}): Promise<void> {
  await json(
    await request(
      '/afterimages',
      {
        method: 'POST',
        body: JSON.stringify({ anon_id: anonId(), ...a, snippet: a.snippet.slice(-150) })
      },
      true
    )
  );
}

export async function postWanderer(area: string, trail: Tick[]): Promise<void> {
  await json(
    await request(
      '/wanderers',
      { method: 'POST', body: JSON.stringify({ anon_id: anonId(), area, trail: trail.slice(-120) }) },
      true
    )
  );
}

export async function putCloudSave(save: CloudSave): Promise<void> {
  await json(await request('/save', { method: 'PUT', body: JSON.stringify(save) }, true));
}

/** The cloud save, or `null` when the player has none. */
export async function getCloudSave(): Promise<CloudSave | null> {
  const res = await request('/save', {}, true);
  if (res.status === 404) return null;
  return json<CloudSave>(res);
}

export async function deleteCloudSave(): Promise<void> {
  await json(await request('/save', { method: 'DELETE' }, true));
}

export async function fetchMarks(): Promise<OnlineMark[]> {
  const body = await json<{ marks: OnlineMark[] }>(await request('/marks', {}, true));
  return body.marks;
}

/** Sends local Marks and returns the merged set. */
export async function putMarks(marks: OnlineMark[]): Promise<OnlineMark[]> {
  const body = await json<{ marks: OnlineMark[] }>(
    await request('/marks', { method: 'PUT', body: JSON.stringify({ marks }) }, true)
  );
  return body.marks;
}

/** Submits a B12 clear in in-game milliseconds; returns the player's best and rank. */
export async function postSpeedrun(
  ms: number,
  contentHash: string
): Promise<{ best_ms: number; rank: number }> {
  return json(
    await request('/speedrun', { method: 'POST', body: JSON.stringify({ ms, content_hash: contentHash }) }, true)
  );
}

/** One batch of telemetry. Anonymous; the caller must have checked the opt-in. */
export async function postTelemetryBatch(
  sessionId: string,
  events: Record<string, string | number | boolean>[]
): Promise<void> {
  await json(
    await request(
      '/telemetry',
      {
        method: 'POST',
        body: JSON.stringify({ anon_id: anonId(), session: sessionId, consent: true, events })
      },
      false
    )
  );
}
