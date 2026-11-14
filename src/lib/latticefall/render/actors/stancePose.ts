// Stance poses: what the actor renderer blends the upper body toward while the core reports
// the player guarding (`FLAG_GUARDING`) or two-handing (`FLAG_TWO_HANDING`). Each stance is a
// target pose for the arms (and, for the guard, the chest), and the walk or run keeps the
// legs. The arms hold the target instead of swinging with the stride. Pure data and maths:
// no Three.js import, and nothing writes simulation state.
// Spec: docs/game/03-combat.md ("Guard") and docs/game/07-presentation.md ("Characters and
// animation"); issues #18, #19, and #83.

import type { BoneOffset } from '../../ui/gestures/gesturePose';

/** Seconds a stance takes to come in or go out: about the guard's 3-tick raise (`locomotion.ron`). */
export const STANCE_EASE_SECONDS = 0.12;

/** Both arms, shoulder to hand. */
export const ARM_BONES = [
  'Shoulder.L',
  'UpperArm.L',
  'LowerArm.L',
  'Hand.L',
  'Shoulder.R',
  'UpperArm.R',
  'LowerArm.R',
  'Hand.R'
] as const;

/** The guard also takes the chest, so the shoulders square up behind the raised arms. */
export const GUARD_BONES = ['Torso', 'Chest', ...ARM_BONES] as const;

/** A clip frame that stands for a stance. `frame` is a 30 Hz clip frame; omit it to loop the clip. */
export interface StanceClip {
  clip: string;
  frame?: number;
}

/** Guard with a shield in the off hand: the shield forearm raised, the weapon held ready. Loops. */
export const SHIELD_GUARD_CLIP: StanceClip = { clip: 'Idle_Shield_Loop' };

/** Guard without a shield: the weapon braced across the body, held at the block's peak. */
export const WEAPON_GUARD_CLIP: StanceClip = { clip: 'Sword_Block', frame: 10 };

/**
 * Guard pose for when the clip library lacks the guard clips: offsets over the idle clip's
 * first frame, with the weapon upright in front of the chest and the off forearm across it.
 */
export const GUARD_FALLBACK: readonly BoneOffset[] = [
  { bone: 'UpperArm.L', x: -1.65, y: -0.47, z: -1.94 },
  { bone: 'LowerArm.L', x: -1.84, y: 0, z: 0 },
  { bone: 'UpperArm.R', x: -0.87, y: 0.17, z: 1.13 },
  { bone: 'LowerArm.R', x: -0.9, y: 0, z: 0 }
];

/**
 * Two-handing: offsets over the idle clip's first frame. Both hands hold the grip at the
 * right hip, the left below the right, and the blade points forward and up across the body
 * (Elden Ring's two-handed straight-sword stance). Fitted in the browser against the drawn
 * blade: grip at (-0.08, 1.07, 0.26) m in the body frame (+Z forward, -X the character's
 * right), the left hand 11 cm down the grip, and the blade along (0.2, 0.71, 0.67).
 */
export const TWO_HAND: readonly BoneOffset[] = [
  { bone: 'UpperArm.R', x: -0.57, y: -0.27, z: 0.61 },
  { bone: 'LowerArm.R', x: -0.1, y: 0, z: 0 },
  { bone: 'Hand.R', x: -0.2, y: -0.1, z: -1.28 },
  { bone: 'UpperArm.L', x: -0.83, y: 0.23, z: -0.33 },
  { bone: 'LowerArm.L', x: -0.01, y: 0, z: 0 }
];

/** Meters the hips sink while guarding. */
export const GUARD_HIPS_DROP = 0.05;

/** Moves `w` (0 to 1) toward `on` at the ease rate. */
export function easeStance(w: number, on: boolean, dt: number): number {
  const step = dt / STANCE_EASE_SECONDS;
  return on ? Math.min(1, w + step) : Math.max(0, w - step);
}

/**
 * Blend weights for guard weight `guard` and two-hand weight `twoHand` (each 0 to 1), eased.
 * The guard replaces the two-hand stance as it raises, because a raised guard is the pose to
 * read.
 */
export function stanceWeights(guard: number, twoHand: number): { guard: number; twoHand: number } {
  const s = (x: number) => {
    const t = Math.min(Math.max(x, 0), 1);
    return t * t * (3 - 2 * t);
  };
  const g = s(guard);
  return { guard: g, twoHand: s(twoHand) * (1 - g) };
}
