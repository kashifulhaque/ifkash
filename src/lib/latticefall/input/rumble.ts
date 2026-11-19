// Controller rumble through the Gamepad API's `vibrationActuator`. Feature-detected: a
// browser or pad without it does nothing. Render-side only: it reads events the core
// already produced and never feeds anything back.
//
// The setting is saved in `localStorage` under `latticefall-rumble` ("off" turns it off;
// anything else, or nothing, leaves it on). The controls settings page has the switch
// (`ui/menus/ControlsSettings.svelte`).

import { pickPad } from './gamepad';

const KEY = 'latticefall-rumble';

/** A rumble pulse: how long, and how hard each motor runs (0 to 1). */
export interface RumblePulse {
  /** Milliseconds. */
  duration: number;
  /** The low-frequency (heavy) motor. */
  strong: number;
  /** The high-frequency (light) motor. */
  weak: number;
}

/** The pulses the game plays. */
export const RUMBLE = {
  /** The player's attack connects: a light tick. */
  hitLanded: { duration: 60, strong: 0.15, weak: 0.5 },
  /** The player's attack is guarded: lighter still. */
  hitGuarded: { duration: 40, strong: 0.05, weak: 0.3 },
  /** The player blocks a hit. */
  blocked: { duration: 90, strong: 0.35, weak: 0.4 },
  /** The player is hit. */
  hitTaken: { duration: 160, strong: 0.7, weak: 0.6 },
  /** The player's guard breaks. */
  guardBroken: { duration: 220, strong: 1, weak: 0.8 },
  /** The player dies. */
  death: { duration: 320, strong: 1, weak: 1 }
} as const satisfies Record<string, RumblePulse>;

export type RumbleName = keyof typeof RUMBLE;

/** The part of `GamepadHapticActuator` this module calls. */
interface Actuator {
  playEffect?: (type: string, params: Record<string, number>) => Promise<unknown> | void;
}

export function rumbleEnabled(): boolean {
  try {
    return localStorage.getItem(KEY) !== 'off';
  } catch {
    // Storage blocked: rumble stays on for this visit.
    return true;
  }
}

export function setRumbleEnabled(on: boolean): void {
  try {
    if (on) localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, 'off');
  } catch {
    // Private mode or a full quota: the setting lasts for this visit only.
  }
}

/** The effect parameters for a pulse, clamped to what the API accepts. */
export function rumbleParams(p: RumblePulse): { startDelay: number; duration: number; strongMagnitude: number; weakMagnitude: number } {
  const unit = (v: number) => (Number.isFinite(v) ? Math.min(1, Math.max(0, v)) : 0);
  return {
    startDelay: 0,
    duration: Math.min(5000, Math.max(0, Math.round(p.duration))),
    strongMagnitude: unit(p.strong),
    weakMagnitude: unit(p.weak)
  };
}

/**
 * Plays a pulse on the pad in use, if it can rumble and the setting allows it. Returns
 * whether a pulse was sent. A new pulse replaces the one playing, so the strongest recent
 * event is what the player feels.
 */
export function rumble(name: RumbleName, pad: { vibrationActuator?: unknown } | null = pickPad()): boolean {
  if (!pad || !rumbleEnabled()) return false;
  const actuator = pad.vibrationActuator as Actuator | null | undefined;
  if (!actuator || typeof actuator.playEffect !== 'function') return false;
  try {
    const done = actuator.playEffect('dual-rumble', rumbleParams(RUMBLE[name]));
    // Chrome rejects when the page is hidden or another effect preempts this one.
    if (done && typeof (done as Promise<unknown>).catch === 'function') (done as Promise<unknown>).catch(() => {});
    return true;
  } catch {
    return false;
  }
}

/** The fields of a hit event that decide its pulse. */
export interface RumbleHit {
  attacker: number;
  target: number;
  /** 0 connected, 1 guarded, 2 guard broken. */
  result: number;
  /** Bit 1: the hit killed its target. */
  flags: number;
}

/** The pulse for a hit as the player feels it, or null when the player took no part in it. */
export function rumbleForHit(e: RumbleHit, playerId: number): RumbleName | null {
  if (e.target === playerId) {
    if ((e.flags & 2) !== 0) return 'death';
    if (e.result === 2) return 'guardBroken';
    return e.result === 1 ? 'blocked' : 'hitTaken';
  }
  if (e.attacker === playerId) return e.result === 1 ? 'hitGuarded' : 'hitLanded';
  return null;
}
