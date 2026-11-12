// Mapping from a move's `anim` name to a clip, scrubbing by `move_frame`, and the phase used
// by the generic pose when no clip exists. Pure: no Three.js, no I/O.
// Spec: docs/game/07-presentation.md, "Characters and animation".

/** What the renderer needs to know about the move an actor is performing. */
export interface MoveAnimInfo {
  /** The move's `anim` field. */
  anim: string;
  /** The move's `total_frames`. */
  totalFrames: number;
  /** First frame of the strike (the first hitbox frame), if known. */
  strikeStart?: number;
  /** First frame of the recovery (after the last hitbox frame), if known. */
  recoveryStart?: number;
}

/** Looks up a `move_id` from the snapshot. Returns undefined for "no move" (locomotion). */
export type MoveResolver = (moveId: number) => MoveAnimInfo | undefined;

/**
 * Move `anim` names the Universal Animation Library can cover today. Anything not listed
 * (and any listed clip the library file lacks) plays the generic pose and warns in dev.
 */
export const ANIM_TO_CLIP: Readonly<Record<string, string>> = {
  sword_slash_r: 'Sword_Attack',
  sword_slash_l: 'Sword_Attack',
  roll: 'Roll',
  hit_chest: 'Hit_Chest',
  death: 'Death01',
  interact: 'Interact'
};

/** The clip name for a move `anim`, or undefined when there is no mapping. */
export function clipForAnim(anim: string): string | undefined {
  return ANIM_TO_CLIP[anim];
}

export type MovePhase = 'windup' | 'strike' | 'recovery';

/** Which third of the move this frame is in, from the move's own frame data when known. */
export function movePhase(info: MoveAnimInfo, frame: number): MovePhase {
  const strike = info.strikeStart ?? Math.round(info.totalFrames / 3);
  const recovery = info.recoveryStart ?? Math.round((info.totalFrames * 2) / 3);
  if (frame < strike) return 'windup';
  if (frame < recovery) return 'strike';
  return 'recovery';
}

/**
 * Clip time for a move frame. The clip is retimed to fit the move: the frame data never
 * changes to fit the animation. `frame` may be fractional (interpolated between ticks).
 */
export function moveClipTime(clipSeconds: number, frame: number, totalFrames: number): number {
  const u = totalFrames > 0 ? frame / totalFrames : 0;
  return Math.min(Math.max(u, 0), 1) * clipSeconds;
}
