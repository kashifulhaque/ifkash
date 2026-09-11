// Roadmap progress, persisted in localStorage. Same shape as the game's
// `progress.ts`: a flat list of ids, validated against the curriculum on load
// so a renamed or removed item never leaves a stale entry behind.
//
// Call these from `onMount`, never during SSR.

import { ALL_ITEMS, MODULES } from './curriculum';

const DONE_KEY = 'roadmap_done';
const OPEN_KEY = 'roadmap_open';

function readIds(key: string, valid: Set<string>): Set<string> {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return new Set();
    const ids = JSON.parse(raw);
    if (!Array.isArray(ids)) return new Set();
    return new Set(ids.filter((id): id is string => typeof id === 'string' && valid.has(id)));
  } catch {
    return new Set();
  }
}

function writeIds(key: string, ids: Set<string>): void {
  try {
    localStorage.setItem(key, JSON.stringify([...ids]));
  } catch {
    /* storage unavailable */
  }
}

/** Ids of the curriculum items marked done. */
export function loadDone(): Set<string> {
  return readIds(DONE_KEY, new Set(ALL_ITEMS.map((i) => i.id)));
}

export function saveDone(done: Set<string>): void {
  writeIds(DONE_KEY, done);
}

/** Ids of the modules the reader has expanded. */
export function loadOpen(): Set<string> {
  return readIds(OPEN_KEY, new Set(MODULES.map((m) => m.id)));
}

export function saveOpen(open: Set<string>): void {
  writeIds(OPEN_KEY, open);
}
