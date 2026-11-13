// B02's intro camera move (docs/game/07-presentation.md, "Camera": "Boss intro: A short scripted move
// along a spline"; docs/game/05-enemies-and-bosses.md, "Fog gate": a skippable 3 s camera move).
//
// The core seals the fog gate and waits `intro_ticks` (90, 3 s) before the fight starts; the
// `Boss` events (kinds 0 and 1) bracket that wait. This module draws the wait: the camera leaves
// the player, swings low in front of the Briar Stag, circles up past its antlers, and settles back
// in a longer shot. When the fight starts it eases into the normal rig's pose over half a second.
//
// It is a pure override of the draw camera: it runs after the rig has placed it each frame, reads
// that pose, and blends toward or away from it. Nothing here reads input for the simulation, and
// the rig keeps running underneath, so there is no jump at the end. It is skipped (Escape, Enter,
// or a click) and refused under reduced motion, as the shake is.

import { CatmullRomCurve3, Matrix4, Quaternion, Vector3, type PerspectiveCamera } from 'three';
import { TICK_HZ, type Vec3 } from '../hazards/arenas';

/** The wait the core holds, in ticks (`BossDef::intro_ticks` default). */
const INTRO_TICKS = 90;
/** Seconds the camera takes to ease back to the rig once the fight starts or the move ends. */
const BLEND_OUT = 0.5;
/** How tall the Stag is, in meters (the model's `height`). */
const STAG_HEIGHT = 3.3;

const UP = new Vector3(0, 1, 0);

const smooth = (x: number) => {
  const t = Math.min(Math.max(x, 0), 1);
  return t * t * (3 - 2 * t);
};

export class IntroCamera {
  private startTick: number | null = null;
  /** The tick the blend back to the rig began at, or null while the move runs. */
  private endTick: number | null = null;
  private path: CatmullRomCurve3 | null = null;
  private look: CatmullRomCurve3 | null = null;
  private skipRequested = false;
  private readonly offSkip: () => void;
  private readonly rigPos = new Vector3();
  private readonly rigQuat = new Quaternion();
  private readonly pos = new Vector3();
  private readonly target = new Vector3();
  private readonly quat = new Quaternion();
  private readonly mat = new Matrix4();

  constructor(private readonly reduced: () => boolean) {
    // Escape, Enter or a click cuts to the fight. The listeners only set a flag; `apply` acts on it.
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' || e.key === 'Enter') this.skipRequested = true;
    };
    const onPointer = () => {
      this.skipRequested = true;
    };
    if (typeof window !== 'undefined') {
      window.addEventListener('keydown', onKey);
      window.addEventListener('pointerdown', onPointer);
      this.offSkip = () => {
        window.removeEventListener('keydown', onKey);
        window.removeEventListener('pointerdown', onPointer);
      };
    } else this.offSkip = () => {};
  }

  get active(): boolean {
    return this.startTick !== null;
  }

  /** The gate sealed at game tick `now`; the Stag stands at `stag`, facing `yaw` (0 faces +Z). */
  begin(now: number, stag: Vec3, yaw: number): void {
    if (this.reduced()) return;
    this.startTick = now;
    this.endTick = null;
    this.skipRequested = false;
    // Built on the first `apply`, when the rig's camera position is known.
    this.path = null;
    this.look = null;
    this.stag = stag;
    this.yaw = yaw;
  }

  private stag: Vec3 = [0, 0, 0];
  private yaw = 0;

  /** The fight started (or the player died): ease back to the rig. */
  end(now: number): void {
    if (this.startTick !== null && this.endTick === null) this.endTick = now;
  }

  /** Builds the spline from the rig camera's first pose to a few shots around the Stag. */
  private build(start: Vector3): void {
    const [sx, sy, sz] = this.stag;
    const fx = Math.sin(this.yaw);
    const fz = Math.cos(this.yaw);
    // Right of the Stag's facing, on the ground plane.
    const rx = fz;
    const rz = -fx;
    const at = (f: number, r: number, h: number) => new Vector3(sx + fx * f + rx * r, sy + h, sz + fz * f + rz * r);
    this.path = new CatmullRomCurve3(
      [start.clone(), at(11, 5, 1.1), at(8.5, -3.5, 1.7), at(13, 0.5, STAG_HEIGHT * 0.95)],
      false,
      'centripetal'
    );
    // Where it looks: toward the Stag from the start, then its head and antlers, then its chest.
    this.look = new CatmullRomCurve3(
      [at(2, 0, 1.4), at(0.5, 0, 1.9), at(0.2, 0, STAG_HEIGHT * 0.85), at(0, 0, STAG_HEIGHT * 0.55)],
      false,
      'centripetal'
    );
  }

  /**
   * Overrides `camera` for this frame. Call after the rig has placed it. `now` is the game clock in
   * ticks (fractional between ticks, so a paused game holds the shot).
   */
  apply(camera: PerspectiveCamera, now: number): void {
    if (this.startTick === null) return;
    if (this.skipRequested) {
      this.stop();
      return;
    }
    this.rigPos.copy(camera.position);
    this.rigQuat.copy(camera.quaternion);
    if (!this.path || !this.look) this.build(this.rigPos);
    const progress = Math.min(Math.max((now - this.startTick) / INTRO_TICKS, 0), 1);
    // The move's own end: if the fight's start event is late, ease out anyway.
    if (progress >= 1 && this.endTick === null) this.endTick = this.startTick + INTRO_TICKS;
    const eased = smooth(progress);
    this.path!.getPoint(eased, this.pos);
    this.look!.getPoint(eased, this.target);
    this.mat.lookAt(this.pos, this.target, UP);
    this.quat.setFromRotationMatrix(this.mat);
    // The blend: full strength during the move, easing away after it ends.
    let weight = 1;
    if (this.endTick !== null) {
      const back = (now - this.endTick) / TICK_HZ / BLEND_OUT;
      if (back >= 1) {
        this.stop();
        return;
      }
      weight = 1 - smooth(back);
    }
    camera.position.lerpVectors(this.rigPos, this.pos, weight);
    camera.quaternion.slerpQuaternions(this.rigQuat, this.quat, weight);
  }

  /** Ends the move at once. */
  stop(): void {
    this.startTick = null;
    this.endTick = null;
    this.path = null;
    this.look = null;
    this.skipRequested = false;
  }

  dispose(): void {
    this.offSkip();
    this.stop();
  }
}
