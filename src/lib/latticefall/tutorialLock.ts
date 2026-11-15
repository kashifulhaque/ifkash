// The live game is only the tutorial for now: Waking Hollow, from Lantern L01 to the Hollow
// Sentry (B01). The other areas stay built and testable with `?dev=1`, but a player can't
// start in them, resume into them, or travel to them. The simulation core doesn't know about
// the lock; only the browser host enforces it. No Three.js or WASM imports.

/** The one area a player can enter while the lock is on. */
export const TUTORIAL_AREA = 'waking_hollow';

let locked = true;

/** Turns the lock on (players) or off (`?dev=1`). The engine calls this once at start. */
export function setAreaLock(on: boolean): void {
  locked = on;
}

/** Whether the lock is on. */
export function isAreaLockOn(): boolean {
  return locked;
}

/** Whether a player is kept out of `area`. */
export function areaLocked(area: string): boolean {
  return locked && area !== TUTORIAL_AREA;
}
