// Turns the core's pickup events into codex unlocks. The core decides that a pickup was
// taken (`lf-core` `pickups`, event kind 17); this only records it. No Three.js or WASM
// import: it reads the raw event words through the plain-TypeScript decoder.

import { decodeEvents } from '../engine/events';
import { entryByKey, PICKUP_ENTRY, type CodexEntry } from './entries';
import type { CodexStore } from './store';

/** Unlocks the entry of every pickup in `words`. Returns the entries that were new. */
export function unlockFromEvents(words: ArrayLike<number>, store: CodexStore): CodexEntry[] {
  const fresh: CodexEntry[] = [];
  for (const ev of decodeEvents(words)) {
    if (ev.kind !== 'pickup') continue;
    const key = PICKUP_ENTRY[ev.item];
    const entry = key ? entryByKey(key) : undefined;
    if (entry && store.unlock(entry.key)) fresh.push(entry);
  }
  return fresh;
}
