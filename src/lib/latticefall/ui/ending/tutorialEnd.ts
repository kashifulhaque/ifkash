// The end of the tutorial: while only Waking Hollow is open (see `tutorialLock.ts`), defeating
// the Hollow Sentry (B01) shows a card that says the tutorial is done and more areas are
// coming. Read-only: it never answers the core.

import { writable, type Readable } from 'svelte/store';
import { BOSS_DEFEATED, type GameEvent } from '../../engine/events';
import { TUTORIAL_AREA, isAreaLockOn } from '../../tutorialLock';

/** How long the card waits after the defeat, in milliseconds, so the fall plays out first. */
export const TUTORIAL_END_DELAY_MS = 2600;

const open = writable(false);
let timer: ReturnType<typeof setTimeout> | null = null;

/** True while the end-of-tutorial card is on screen. */
export const tutorialEndOpen: Readable<boolean> = { subscribe: open.subscribe };

/** Shows the card after `delayMs`. A second call while one is pending changes nothing. */
export function showTutorialEnd(delayMs: number = TUTORIAL_END_DELAY_MS): void {
  if (timer !== null) return;
  timer = setTimeout(() => {
    timer = null;
    open.set(true);
  }, delayMs);
}

/** Hides the card and cancels a pending one. */
export function closeTutorialEnd(): void {
  if (timer !== null) clearTimeout(timer);
  timer = null;
  open.set(false);
}

/** Feeds a tick's decoded events: a boss defeat in the tutorial area, with the lock on, shows the card. */
export function tutorialEndFromEvents(events: readonly GameEvent[], area: string): void {
  if (area !== TUTORIAL_AREA || !isAreaLockOn()) return;
  for (const e of events) {
    if (e.kind === 'boss' && e.what === BOSS_DEFEATED) showTutorialEnd();
  }
}
