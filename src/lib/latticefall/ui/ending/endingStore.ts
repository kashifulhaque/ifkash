// The ending's trigger (docs/game/08-portfolio-integration.md, "Contact": "After B12, the ending
// hands the player the Courier's Letter"). The core raises the `boss.b12_the_unspooled.defeated` flag and
// sends B12's `Loom` event kind `CLEARED` with slot 1 once, when the Unspooled falls; the second
// death is the defeat, so the Weaver's first fall doesn't send it. This store turns that one event
// into "show the letter". Read-only: it never answers the core.

import { writable, type Readable } from 'svelte/store';
import { LOOM_CLEARED, type GameEvent } from '../../engine/events';

/** How long the letter waits after the defeat, in milliseconds, so the fall plays out first. */
export const ENDING_DELAY_MS = 2600;

const open = writable(false);
let timer: ReturnType<typeof setTimeout> | null = null;

/** True while the Courier's Letter is on screen. */
export const endingOpen: Readable<boolean> = { subscribe: open.subscribe };

/** Shows the letter after `delayMs`. A second call while one is pending changes nothing. */
export function showEnding(delayMs: number = ENDING_DELAY_MS): void {
  if (timer !== null) return;
  timer = setTimeout(() => {
    timer = null;
    open.set(true);
  }, delayMs);
}

/** Hides the letter and cancels a pending one. */
export function closeEnding(): void {
  if (timer !== null) clearTimeout(timer);
  timer = null;
  open.set(false);
}

/** Feeds a tick's decoded events: B12's defeat (the `Loom` event `CLEARED` with slot 1) starts the ending. */
export function endingFromEvents(events: readonly GameEvent[]): void {
  for (const e of events) {
    if (e.kind === 'loom' && e.what === LOOM_CLEARED && e.slot === 1) showEnding();
  }
}
