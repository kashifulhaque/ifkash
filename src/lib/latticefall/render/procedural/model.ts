// What a procedural enemy model is: a rig builder, locomotion poses, and move choreography.
// The framework (`ProceduralActor`, in `actor.ts`) turns a snapshot actor into a pose from
// these. Pure data plus Three.js meshes made through `Rig`.

import type { Choreo } from './pose';
import type { Pose, Rig } from './rig';

export interface ModelDef {
  /** Content id of the enemy definition, for the dev preview (`?preview=husk`). */
  id: string;
  /** Standing height in meters, for camera framing and the preview layout. */
  height: number;
  build(): Rig;
  /** Standing animation at `t` seconds. */
  idle(t: number): Pose;
  /** Moving animation at a gait phase in radians, `amp` 0 to 1 from ground speed. */
  walk(phase: number, amp: number): Pose;
  /** Meters traveled per full gait cycle: the gait phase advances with ground speed. */
  stride: number;
  /**
   * Move choreography by the move's `anim` name. A key that isn't an exact match is tried as a
   * substring of the anim (so `slam` covers `b01_overhead_slam`), in listed order.
   */
  moves: Record<string, Choreo>;
  /** Used for a move whose `anim` matches nothing above, so no move looks like locomotion. */
  fallback: Choreo;
  /** Falls over. `u` runs 0 to 1 over the death animation. Added to the idle pose. */
  death(u: number): Pose;
  /** Rotation axis for the whole-body fall: 'x' tips back, 'z' tips onto the side. */
  fallAxis: 'x' | 'z';
  /**
   * Optional pose while the actor is locked out and not in a move (a stagger, a stance break, a
   * stun): `t` is seconds in that state. Models without it keep their idle.
   */
  stagger?(t: number): Pose;
  /** Optional per-frame hook after the pose is applied (an arrow vanishing, a lantern flaring). */
  tick?(rig: Rig, ctx: { frame: number; total: number; inMove: boolean; anim: string }): void;
}
