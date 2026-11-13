// Poses one procedural enemy from snapshot data each frame: locomotion by ground speed, then
// the move's choreography by `move_frame`, then the fall when dead. Render-only.

import { Group } from 'three';
import { addPose } from './biped';
import type { ModelDef } from './model';
import { lerpPose, overlayPose, sampleKeys, type Choreo } from './pose';
import type { Pose, Rig } from './rig';

/** Simulation tick length (30 Hz). Matches `TICK_SECONDS` in `render/actors/ActorRenderer.ts`. */
const TICK_SECONDS = 1 / 30;
/** A larger per-frame move than this (meters) is a teleport, not a speed. */
const TELEPORT_METERS = 3;
const SPEED_SMOOTH_SECONDS = 0.08;
/** Seconds the fall takes. */
const DEATH_SECONDS = 0.7;
/** Frames a move takes to blend in from locomotion, and to blend back out at its end. */
const BLEND_IN_FRAMES = 3;
const BLEND_OUT_FRAMES = 5;

export interface ActorFrame {
  x: number;
  y: number;
  z: number;
  yaw: number;
  dead: boolean;
  /** Hidden in reeds: nothing is drawn (the snapshot's `HIDDEN` flag). */
  hidden?: boolean;
  /** Shimmering and still: drawn faint (the snapshot's `CLOAKED` flag). */
  cloaked?: boolean;
  /** Locked out (the snapshot's `STAGGERED` flag): a model with a `stagger` pose plays it. */
  staggered?: boolean;
}

/** Opacity of a cloaked enemy: enough to catch the eye against a plain wall, no more. */
export const CLOAKED_OPACITY = 0.16;

export interface MoveFrame {
  anim: string;
  total: number;
  /** Fractional frame within the move. */
  frame: number;
}

export function findChoreo(def: ModelDef, anim: string): Choreo {
  const exact = def.moves[anim];
  if (exact) return exact;
  for (const [key, c] of Object.entries(def.moves)) if (anim.includes(key)) return c;
  return def.fallback;
}

const smooth = (x: number) => {
  const t = Math.min(Math.max(x, 0), 1);
  return t * t * (3 - 2 * t);
};

export class ProceduralActor {
  /** The transform the sim drives. */
  readonly holder = new Group();
  readonly rig: Rig;
  private speed = 0;
  private phase = 0;
  private idleT: number;
  private last: [number, number, number] | null = null;
  private deathT = 0;
  /** Seconds the actor has been staggered with no move, for `ModelDef.stagger`. */
  private staggerT = 0;
  /** Hitstop: seconds left in which the drawn pose is frozen. */
  hold = 0;

  constructor(
    readonly def: ModelDef,
    seed = 0
  ) {
    // Offsets the idle sway so a pack doesn't breathe in unison.
    this.idleT = (seed % 7) * 0.9;
    this.rig = def.build();
    // One draw call instead of a dozen. A model with a `tick` hook that hides parts keeps those
    // parts separate (`animatedBones`); one that doesn't say so keeps all its meshes.
    if (!def.tick || def.animatedBones) this.rig.batch(def.animatedBones);
    this.holder.add(this.rig.root);
  }

  /** `speedOverride` pins the ground speed (the dev preview). */
  update(a: ActorFrame, move: MoveFrame | undefined, dt: number, speedOverride?: number): void {
    this.holder.position.set(a.x, a.y, a.z);
    this.holder.rotation.y = a.yaw;
    this.holder.visible = !a.hidden;
    this.rig.setOpacity(a.cloaked && !a.dead ? CLOAKED_OPACITY : 1);
    if (speedOverride !== undefined) this.speed = speedOverride;
    else if (this.last && dt > 0) {
      const d = Math.hypot(a.x - this.last[0], a.z - this.last[2]);
      const raw = d > TELEPORT_METERS ? 0 : d / dt;
      this.speed += (raw - this.speed) * (1 - Math.exp(-dt / SPEED_SMOOTH_SECONDS));
    }
    this.last = [a.x, a.y, a.z];

    if (this.hold > 0) {
      this.hold -= Math.min(dt, this.hold);
      return;
    }
    const def = this.def;
    this.idleT += dt;
    const amp = Math.min(this.speed / 3, 1);
    // Gait phase advances with distance, so feet don't skate at any speed.
    this.phase = (this.phase + (Math.PI * 2 * this.speed * dt) / def.stride) % (Math.PI * 2);
    const moving = smooth(this.speed / 0.5);
    let pose: Pose = def.idle(this.idleT);
    if (moving > 0) pose = lerpPose(pose, addPose(def.idle(this.idleT), def.walk(this.phase, Math.max(amp, 0.35))), moving);

    let glow = 1;
    if (a.dead) {
      this.deathT += dt;
      const u = Math.min(this.deathT / DEATH_SECONDS, 1);
      pose = addPose(pose, def.death(smooth(u)));
      const fall = smooth(u) * 1.5;
      this.rig.root.rotation.set(def.fallAxis === 'x' ? -fall : 0, 0, def.fallAxis === 'z' ? fall : 0);
      this.rig.root.position.y = -0.05 * u;
    } else {
      this.deathT = 0;
      this.rig.root.rotation.set(0, 0, 0);
      this.rig.root.position.y = 0;
      if (a.staggered && !move && def.stagger) {
        this.staggerT += dt;
        pose = overlayPose(pose, def.stagger(this.staggerT), smooth(this.staggerT / 0.12));
      } else this.staggerT = 0;
      if (move) {
        const c = findChoreo(def, move.anim);
        const f = Math.max(0, move.frame);
        const w = Math.min(f / BLEND_IN_FRAMES, 1) * Math.min(Math.max((move.total - f) / BLEND_OUT_FRAMES, 0), 1);
        pose = overlayPose(pose, sampleKeys(c.keys(move.total), f), smooth(w));
        glow = c.glow ? c.glow(f, move.total) : 1;
      }
    }
    this.rig.apply(pose);
    this.rig.setGlow(glow);
    def.tick?.(this.rig, { frame: move?.frame ?? 0, total: move?.total ?? 0, inMove: !!move && !a.dead, anim: move?.anim ?? '' });
  }

  dispose(): void {
    this.rig.dispose();
  }
}

export { TICK_SECONDS };
