// Spark looks by damage type, and the per-move facts VFX would like from content. Pure data
// and small functions: no Three.js.
//
// The `Hit` event doesn't carry a damage type yet (WP-023 adds damage words). Until then
// `describeHit` asks the optional `HitFxResolver` for the move's physical type, element and
// hitstop, and falls back to Neutral physical with hitstop derived from the motion value.
// Wire the resolver to the content tables (or read the new Hit words) and the sparks change
// colour with no other edit.

import type { Element } from '../../gen/Element';
import type { PhysType } from '../../gen/PhysType';

/** What content says about the hitbox a hit came from. Every field is optional. */
export interface HitFxInfo {
  phys?: PhysType;
  element?: Element | null;
  /** Freeze frames at 30 Hz (`Hitbox.hitstop`). */
  hitstop?: number;
}

/** Looks up a hit's move number and hitbox index. Return undefined when unknown. */
export type HitFxResolver = (moveNum: number, hitbox: number) => HitFxInfo | undefined;

export interface SparkStyle {
  /** Linear 0-1 RGB of the bright core. */
  core: [number, number, number];
  /** Linear 0-1 RGB the sparks cool toward. */
  ember: [number, number, number];
  /** Speed multiplier; blunt hits throw fewer, slower sparks, pierce throws fast thin ones. */
  speed: number;
  /** Count multiplier. */
  count: number;
  /** Spark diameter multiplier. */
  size: number;
}

const rgb = (hex: number): [number, number, number] => [((hex >> 16) & 255) / 255, ((hex >> 8) & 255) / 255, (hex & 255) / 255];

const PHYS: Record<PhysType, SparkStyle> = {
  Neutral: { core: rgb(0xffd25a), ember: rgb(0xe0561e), speed: 1, count: 1, size: 1 },
  Slash: { core: rgb(0xffe9a8), ember: rgb(0xd98a2c), speed: 1.15, count: 1.1, size: 0.9 },
  Blunt: { core: rgb(0xffc070), ember: rgb(0xb8541f), speed: 0.8, count: 0.8, size: 1.3 },
  Pierce: { core: rgb(0xc9f0ff), ember: rgb(0x3f90c8), speed: 1.35, count: 0.8, size: 0.75 }
};

const ELEMENT: Record<Element, SparkStyle> = {
  Fire: { core: rgb(0xffd27a), ember: rgb(0xe8481c), speed: 0.9, count: 1.2, size: 1.2 },
  Storm: { core: rgb(0xf0fbff), ember: rgb(0x4fa8ff), speed: 1.3, count: 1.1, size: 0.85 },
  Aether: { core: rgb(0xf0e6ff), ember: rgb(0x8f6bff), speed: 1, count: 1, size: 1 },
  Radiance: { core: rgb(0xffffe0), ember: rgb(0xffc83d), speed: 1, count: 1, size: 1.1 }
};

/** An element wins over the physical type when a hit has both. */
export function sparkStyle(phys: PhysType, element: Element | null | undefined): SparkStyle {
  return (element && ELEMENT[element]) || PHYS[phys] || PHYS.Neutral;
}

/** Hitstop frames from a motion value, matching the straight-sword data (1.0 is 3, 1.3 is 4). */
export function fallbackHitstop(motionValue: number, chargeFrac: number): number {
  const frames = 3 + (motionValue - 1) * 3 + chargeFrac * 2;
  return Math.min(8, Math.max(2, Math.round(frames)));
}

/** Motion value at and above which a hit is heavy enough to shake the camera. */
export const HEAVY_MOTION_VALUE = 1.25;

/**
 * Camera shake trauma (0 to 1) for a hit, or 0 for none. Only heavy hits shake; a hit that
 * lands on the player shakes a little even when light.
 */
export function hitTrauma(motionValue: number, chargeFrac: number, playerHit: boolean): number {
  let t = 0;
  if (motionValue >= HEAVY_MOTION_VALUE || chargeFrac > 0.5) {
    t = 0.25 + (motionValue - 1) * 0.9 + chargeFrac * 0.4;
  }
  if (playerHit) t = Math.max(t, 0.3);
  return Math.min(0.9, Math.max(0, t));
}
