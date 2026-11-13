// Cloud saves and Mark sync on top of `api.ts`. No Three.js or WASM import. The local
// IndexedDB slots stay the source of truth; the cloud copy is a backup the player can pull
// on another device.

import { get } from 'svelte/store';
import { codex } from '../codex/store';
import { getCloudSave, putCloudSave, putMarks, type OnlineMark } from './api';

export function bytesToBase64(bytes: Uint8Array): string {
  let s = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(s);
}

export function base64ToBytes(b64: string): Uint8Array {
  const s = atob(b64);
  const out = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i);
  return out;
}

/** The part of a local save slot record that goes to the cloud (`SaveRecord` fits). */
export interface UploadableSave {
  data: Uint8Array;
  /** Milliseconds since the epoch. */
  savedAt: number;
  scenario: string;
  /** The world's seed as a decimal string (a u64). */
  seed: string;
  hash: string;
}

/** Uploads a local save. Throws `AuthError` when signed out, `OfflineError` offline. */
export async function uploadSave(save: UploadableSave): Promise<void> {
  await putCloudSave({
    data: bytesToBase64(save.data),
    scenario: save.scenario,
    seed: save.seed,
    hash: save.hash,
    saved_at: save.savedAt
  });
}

/** The cloud save decoded for `LfWorld.fromSave`, or `null` when there is none. */
export async function downloadSave(): Promise<UploadableSave | null> {
  const cloud = await getCloudSave();
  if (!cloud) return null;
  return {
    data: base64ToBytes(cloud.data),
    savedAt: cloud.saved_at,
    scenario: cloud.scenario,
    seed: cloud.seed,
    hash: cloud.hash
  };
}

// Mark sync. The codex store keeps Mark names without times, so the first time this
// module sees a Mark it records the time in its own key.

const TIMES_KEY = 'ifkash-latticefall-mark-times';

function readTimes(): Record<string, number> {
  try {
    const raw = JSON.parse(localStorage.getItem(TIMES_KEY) ?? '{}') as Record<string, unknown>;
    const out: Record<string, number> = {};
    for (const [k, v] of Object.entries(raw)) if (typeof v === 'number') out[k] = v;
    return out;
  } catch {
    return {};
  }
}

function writeTimes(times: Record<string, number>): void {
  try {
    localStorage.setItem(TIMES_KEY, JSON.stringify(times));
  } catch {
    /* storage unavailable */
  }
}

/**
 * Merges the local Marks with the server's: a union, with the earliest time winning.
 * Marks the server had that this device lacks are added to the codex. Returns the merged
 * list. Call after sign-in and after earning a Mark. Throws `AuthError` when signed out.
 */
export async function syncMarks(): Promise<OnlineMark[]> {
  codex.reload();
  const local = get(codex).marks;
  const times = readTimes();
  const now = Date.now();
  for (const id of local) times[id] ??= now;
  const merged = await putMarks(local.map((id) => ({ id, at: times[id] })));
  for (const m of merged) {
    times[m.id] = Math.min(times[m.id] ?? m.at, m.at);
    codex.mark(m.id);
  }
  writeTimes(times);
  return merged;
}
