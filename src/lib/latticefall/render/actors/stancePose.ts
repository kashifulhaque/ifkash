// Stance poses: bone offsets that the actor renderer lays over locomotion while the core
// reports the player guarding (`FLAG_GUARDING`) or two-handing (`FLAG_TWO_HANDING`). The
// animation library has no block or two-hand clips, so, like the gestures, the poses are
// built from rotations. Pure math: no Three.js import, and nothing writes simulation state.
// Spec: docs/game/03-combat.md ("Guard") and docs/game/07-presentation.md ("Characters and
// animation"); issues #18 and #19.

import type { BoneOffset } from '../../ui/gestures/gesturePose';

/** Seconds a stance takes to come in or go out: about the guard's 3-tick raise (`locomotion.ron`). */
export const STANCE_EASE_SECONDS = 0.12;

// The angles were fitted in the browser against hand targets in the body frame (meters,
// +Z forward, feet at 0), with the elbows bending on their X hinge only.

/**
 * Guard: the weapon held upright in front of the chest and the off forearm across it, at about
 * chest height (hands at 1.2 to 1.35 m, 0.3 m in front). A shield in the off hand faces forward.
 */
const GUARD: readonly BoneOffset[] = [
  { bone: 'UpperArm.L', x: -1.65, y: -0.47, z: -1.94 },
  { bone: 'LowerArm.L', x: -1.84, y: 0, z: 0 },
  { bone: 'UpperArm.R', x: -0.87, y: 0.17, z: 1.13 },
  { bone: 'LowerArm.R', x: -0.9, y: 0, z: 0 }
];

/** Two-handing: both hands together on the grip at the body's center line, blade upright. */
const TWO_HAND: readonly BoneOffset[] = [
  { bone: 'UpperArm.L', x: -0.01, y: 0.31, z: -0.87 },
  { bone: 'LowerArm.L', x: -0.17, y: 0, z: 0 },
  { bone: 'UpperArm.R', x: 0.15, y: -0.4, z: 0.93 },
  { bone: 'LowerArm.R', x: -0.11, y: 0, z: 0 }
];

/** Meters the hips sink while guarding. */
export const GUARD_HIPS_DROP = 0.05;

/** Moves `w` (0 to 1) toward `on` at the ease rate. */
export function easeStance(w: number, on: boolean, dt: number): number {
  const step = dt / STANCE_EASE_SECONDS;
  return on ? Math.min(1, w + step) : Math.max(0, w - step);
}

/**
 * The offsets for guard weight `guard` and two-hand weight `twoHand` (each 0 to 1). The guard
 * replaces the two-hand stance as it raises, because a raised guard is the pose to read.
 */
export function stancePose(guard: number, twoHand: number): BoneOffset[] {
  const out: BoneOffset[] = [];
  const s = (x: number) => x * x * (3 - 2 * x);
  const g = s(Math.min(Math.max(guard, 0), 1));
  const h = s(Math.min(Math.max(twoHand, 0), 1)) * (1 - g);
  const bones = new Map<string, BoneOffset>();
  const add = (set: readonly BoneOffset[], w: number) => {
    if (w <= 0) return;
    for (const o of set) {
      const cur = bones.get(o.bone) ?? { bone: o.bone, x: 0, y: 0, z: 0 };
      cur.x += o.x * w;
      cur.y += o.y * w;
      cur.z += o.z * w;
      bones.set(o.bone, cur);
    }
  };
  add(GUARD, g);
  add(TWO_HAND, h);
  for (const o of bones.values()) out.push(o);
  return out;
}
