// The third-person camera. It reads the player and the lock target from the snapshot and
// the level through the core's read-only `raycast`, and never writes simulation state.
//
// Free camera: orbit the pivot (1.6 m above the player's feet) by yaw and pitch. Lock-on:
// aim at the midpoint of the player and the target, turn behind the player along the line
// to the target, and pull back until both actors fit on screen. Both modes then run a
// conical collision cast from the pivot toward the camera, so the camera stops short of
// walls instead of clipping into them.

import { PerspectiveCamera, Vector3 } from 'three';
import { FLAG_LOCKED, lerpAngle, type DrawActor } from '../../engine/snapshot';
import {
  DEFAULT_CAMERA_SETTINGS,
  PIVOT_HEIGHT,
  PITCH_MAX,
  PITCH_MIN,
  clampSettings,
  type CameraSettings
} from './limits';

/** Distance along `dir` (in units of its length) to the first level hit within `max`. */
export type RaycastFn = (origin: Vector3, dir: Vector3, max: number) => number | undefined;

/** Radius of the camera's collision "sphere": larger than the near plane's corner distance. */
const COLLISION_RADIUS = 0.3;
/** The camera keeps at least this far from any surface (the near plane's corners sit about 0.12 m out). */
const CLEARANCE = 0.25;
/** Rays in the collision ring, plus one down the middle. */
const RING_RAYS = 8;
/** The camera never comes closer to the pivot than this, even in a corner. */
const MIN_DISTANCE = 0.45;
/** Time constants in seconds for exponential smoothing. */
const TAU_PIVOT = 0.08;
const TAU_FOCUS = 0.12;
const TAU_LOCK_YAW = 0.12;
const TAU_LOCK_DIST = 0.25;
const TAU_COLLISION_OUT = 0.3;
/** Sideways camera offset while locked on, in meters, for an over-the-shoulder view. */
const LOCK_SHOULDER = 0.6;
/** Both actors must land inside this fraction of the screen's half-extent. */
const FRAME_MARGIN = 0.75;
const PLAYER_HEIGHT = 1.8;
/** Assumed target size until the core exposes `camera_radius`. */
const TARGET_HEIGHT = 2.2;
const TARGET_RADIUS = 0.8;
/** Never pull back further than this multiple of the base distance when framing. */
const MAX_PULL_BACK = 3.5;
const WORLD_UP = new Vector3(0, 1, 0);

function smooth(dtSec: number, tau: number): number {
  return 1 - Math.exp(-dtSec / tau);
}

export interface CameraDebug {
  /** Shortest distance from the camera to level geometry over a sphere of probe rays. */
  clearance: number;
  /** Camera distance from the pivot after collision. */
  distance: number;
  locked: boolean;
  playerOnScreen: boolean;
  targetOnScreen: boolean;
}

export interface CameraPose {
  /** Yaw the camera ended up with. While locked on this differs from the input yaw. */
  yaw: number;
  pitch: number;
  locked: boolean;
}

export class CameraRig {
  readonly camera: PerspectiveCamera;
  private raycast: RaycastFn;
  private settings: CameraSettings;

  private pivot = new Vector3();
  private focus = new Vector3();
  private yaw = 0;
  private lockDist = 0;
  private collisionFrac = 1;
  private init = false;
  private wasLocked = false;
  /** Dev screenshots set this to stand in for a locked-on actor. */
  fakeTarget: Vector3 | null = null;

  // Scratch values, reused every frame.
  private a = new Vector3();
  private b = new Vector3();
  private c = new Vector3();
  private u = new Vector3();
  private v = new Vector3();
  private dir = new Vector3();
  private desired = new Vector3();
  private pivotGoal = new Vector3();
  private focusGoal = new Vector3();
  private lastPlayer = new Vector3();
  private lastTarget = new Vector3();
  private lastLocked = false;
  private lastDist = 0;

  constructor(camera: PerspectiveCamera, raycast: RaycastFn, settings: CameraSettings = DEFAULT_CAMERA_SETTINGS) {
    this.camera = camera;
    this.raycast = raycast;
    this.settings = clampSettings(settings);
    this.applyFov();
  }

  getSettings(): CameraSettings {
    return { ...this.settings };
  }

  setSettings(s: Partial<CameraSettings>): void {
    this.settings = clampSettings({ ...this.settings, ...s });
    this.applyFov();
  }

  /** Pushes the field of view to the camera. Call after anything resets the projection. */
  applyFov(): void {
    this.camera.fov = this.settings.fov;
    this.camera.updateProjectionMatrix();
  }

  /** Forget smoothing state so the next update snaps (teleports, scene changes). */
  reset(): void {
    this.init = false;
  }

  /**
   * Moves the camera for one rendered frame. `actors[0]` is the player; any other actor
   * flagged `FLAG_LOCKED` is the lock target. `pivotOverride` replaces the player's pivot
   * (the dev orbit script uses it). Returns the pose the camera used.
   */
  update(actors: DrawActor[], inYaw: number, inPitch: number, frameMs: number, pivotOverride?: Vector3): CameraPose {
    const dt = Math.min(frameMs, 100) / 1000;
    const player = actors[0];
    if (!player) return { yaw: inYaw, pitch: inPitch, locked: false };

    const pitch = Math.min(PITCH_MAX, Math.max(PITCH_MIN, inPitch));
    this.lastPlayer.set(player.x, player.y, player.z);
    const lockedActor = actors.find((a) => a.id !== player.id && (a.flags & FLAG_LOCKED) !== 0);
    let target: Vector3 | null = null;
    if (lockedActor) target = this.lastTarget.set(lockedActor.x, lockedActor.y, lockedActor.z);
    else if (this.fakeTarget) target = this.lastTarget.copy(this.fakeTarget);
    const locked = target !== null && !pivotOverride;

    // Smoothed pivot, with a safety cast so the lag can't carry it through a wall.
    this.pivotGoal.copy(pivotOverride ?? this.a.set(player.x, player.y + PIVOT_HEIGHT, player.z));
    if (!this.init) {
      this.pivot.copy(this.pivotGoal);
      this.focus.copy(this.pivotGoal);
      this.yaw = inYaw;
      this.collisionFrac = 1;
      this.lockDist = 0;
      this.init = true;
    } else {
      this.b.copy(this.pivot).lerp(this.pivotGoal, smooth(dt, TAU_PIVOT));
      this.dir.subVectors(this.b, this.pivotGoal);
      if (this.dir.lengthSq() > 1e-8 && this.raycast(this.pivotGoal, this.dir, 1) !== undefined) {
        this.pivot.copy(this.pivotGoal);
      } else {
        this.pivot.copy(this.b);
      }
    }

    // Yaw and the point the camera looks at.
    let yaw = inYaw;
    let lockPull = 0;
    if (locked && target) {
      const dx = target.x - player.x;
      const dz = target.z - player.z;
      const sep = Math.hypot(dx, dz);
      // The camera sits at (sin yaw, cos yaw) around the player, so forward is (-sin, -cos).
      const goalYaw = sep > 0.25 ? Math.atan2(-dx, -dz) : this.yaw;
      const from = this.wasLocked ? this.yaw : inYaw;
      this.yaw = lerpAngle(from, goalYaw, smooth(dt, TAU_LOCK_YAW));
      yaw = this.yaw;
      this.focusGoal.set(
        (player.x + target.x) / 2,
        (this.pivotGoal.y + target.y + TARGET_HEIGHT * 0.5) / 2,
        (player.z + target.z) / 2
      );
      lockPull = sep / 2;
    } else {
      this.yaw = inYaw;
      this.focusGoal.copy(this.pivotGoal);
    }
    if (!locked) this.lockDist = 0;
    this.wasLocked = locked;
    if (locked) this.focus.lerp(this.focusGoal, smooth(dt, TAU_FOCUS));
    else this.focus.copy(this.pivot);

    // Desired camera position before collision.
    const cp = Math.cos(pitch);
    this.dir.set(Math.sin(yaw) * cp, Math.sin(pitch), Math.cos(yaw) * cp);
    const base = this.settings.distance;
    if (locked && target) {
      this.c.set(Math.cos(yaw), 0, -Math.sin(yaw)); // camera-right
      const goal = this.fitLock(player, target, base + lockPull);
      this.lockDist += (goal - this.lockDist) * (this.lockDist === 0 ? 1 : smooth(dt, TAU_LOCK_DIST));
      this.desired.copy(this.focus).addScaledVector(this.dir, this.lockDist).addScaledVector(this.c, LOCK_SHOULDER);
    } else {
      this.desired.copy(this.pivot).addScaledVector(this.dir, base);
    }

    // Collision: cast from the pivot toward the camera. Pull in at once, ease back out.
    const span = this.a.subVectors(this.desired, this.pivot).length();
    const minFrac = span > 1e-4 ? Math.min(1, MIN_DISTANCE / span) : 1;
    const castFrac = Math.max(minFrac, this.collisionFraction(this.pivot, this.desired));
    const goalFrac = Math.max(minFrac, this.keepClear(this.pivot, this.desired, castFrac));
    if (goalFrac < this.collisionFrac) this.collisionFrac = goalFrac;
    else this.collisionFrac += (goalFrac - this.collisionFrac) * smooth(dt, TAU_COLLISION_OUT);
    this.a.subVectors(this.desired, this.pivot);
    this.camera.position.copy(this.pivot).addScaledVector(this.a, this.collisionFrac);
    this.camera.lookAt(this.focus);
    this.camera.updateMatrixWorld();

    this.lastLocked = locked;
    this.lastDist = span * this.collisionFrac;
    return { yaw, pitch, locked };
  }

  /**
   * The ray ring above stops the camera where its cone touches a surface, which leaves the
   * camera itself touching too, since the cone narrows toward the pivot. This walks the
   * camera back along its line until a sphere of probe rays around it finds a surface no
   * closer than `CLEARANCE`, or it reaches the minimum distance. Returns the new fraction.
   */
  private keepClear(from: Vector3, to: Vector3, frac: number): number {
    const span = this.b.subVectors(to, from).length();
    if (span < 1e-4) return frac;
    const minFrac = Math.min(1, MIN_DISTANCE / span);
    const p = this.v;
    let f = frac;
    for (let i = 0; i < 40 && f > minFrac; i++) {
      p.copy(from).addScaledVector(this.b.subVectors(to, from), f);
      const c = this.clearanceAt(p, CLEARANCE);
      if (c >= CLEARANCE) break;
      f = Math.max(minFrac, f - Math.max(0.04, CLEARANCE - c) / span);
    }
    return f;
  }

  /** Distance from `pos` to the nearest surface along 26 directions, or `max` if none is closer. */
  private clearanceAt(pos: Vector3, max: number): number {
    let best = max;
    const d = this.u;
    for (let x = -1; x <= 1; x++)
      for (let y = -1; y <= 1; y++)
        for (let z = -1; z <= 1; z++) {
          if (x === 0 && y === 0 && z === 0) continue;
          d.set(x, y, z).normalize();
          const h = this.raycast(pos, d, best);
          if (h !== undefined && h < best) best = h;
        }
    return best;
  }

  /** Smallest fraction in (0, 1] of the way from `from` to `to` at which a ray ring hits the level. */
  private collisionFraction(from: Vector3, to: Vector3): number {
    const axis = this.dir.subVectors(to, from);
    if (axis.lengthSq() < 1e-8) return 1;
    axis.normalize();
    // Any two unit vectors perpendicular to the axis.
    const u = this.u.copy(Math.abs(axis.y) > 0.95 ? this.b.set(1, 0, 0) : WORLD_UP).cross(axis).normalize();
    const v = this.v.crossVectors(axis, u);
    const ray = this.c;
    let best = this.raycast(from, ray.subVectors(to, from), 1) ?? 1;
    for (let i = 0; i < RING_RAYS; i++) {
      const t = (i / RING_RAYS) * Math.PI * 2;
      ray
        .copy(to)
        .addScaledVector(u, Math.cos(t) * COLLISION_RADIUS)
        .addScaledVector(v, Math.sin(t) * COLLISION_RADIUS)
        .sub(from);
      const h = this.raycast(from, ray, 1);
      if (h !== undefined) best = Math.min(best, h);
    }
    return best;
  }

  /**
   * The smallest camera distance from the focus, at or above `start`, at which the player
   * and the target both fit inside `FRAME_MARGIN` of the screen. `this.dir` is the unit
   * vector from the focus toward the camera and `this.c` is camera-right.
   */
  private fitLock(player: DrawActor, target: Vector3, start: number): number {
    const tanV = Math.tan((this.settings.fov * Math.PI) / 360) * FRAME_MARGIN;
    const tanH = tanV * this.camera.aspect;
    const right = this.c;
    const mid = target.y + TARGET_HEIGHT / 2;
    const pts: Array<[number, number, number]> = [
      [player.x, player.y, player.z],
      [player.x, player.y + PLAYER_HEIGHT, player.z],
      [target.x, target.y, target.z],
      [target.x, target.y + TARGET_HEIGHT, target.z],
      [target.x + right.x * TARGET_RADIUS, mid, target.z + right.z * TARGET_RADIUS],
      [target.x - right.x * TARGET_RADIUS, mid, target.z - right.z * TARGET_RADIUS]
    ];
    const camPos = this.a;
    const axis = this.b;
    const camRight = this.u;
    const camUp = this.v;
    const d = new Vector3();
    const maxDist = start + this.settings.distance * MAX_PULL_BACK;
    let dist = start;
    for (let i = 0; i < 24 && dist < maxDist; i++) {
      camPos.copy(this.focus).addScaledVector(this.dir, dist).addScaledVector(right, LOCK_SHOULDER);
      axis.subVectors(this.focus, camPos).normalize();
      camRight.crossVectors(axis, WORLD_UP).normalize();
      camUp.crossVectors(camRight, axis);
      let ok = true;
      for (const [x, y, z] of pts) {
        d.set(x, y, z).sub(camPos);
        const depth = d.dot(axis);
        if (depth < 0.2 || Math.abs(d.dot(camRight)) / depth > tanH || Math.abs(d.dot(camUp)) / depth > tanV) {
          ok = false;
          break;
        }
      }
      if (ok) break;
      dist *= 1.12;
    }
    return Math.min(dist, maxDist);
  }

  /** Dev readout: how close the camera is to the level, and whether both actors are visible. */
  debug(probe: boolean): CameraDebug {
    const cam = this.camera.position;
    let clearance = Infinity;
    if (probe) {
      const dir = this.dir;
      for (let x = -1; x <= 1; x++)
        for (let y = -1; y <= 1; y++)
          for (let z = -1; z <= 1; z++) {
            if (x === 0 && y === 0 && z === 0) continue;
            dir.set(x, y, z).normalize();
            const h = this.raycast(cam, dir, 50);
            if (h !== undefined) clearance = Math.min(clearance, h);
          }
    }
    const onScreen = (p: Vector3): boolean => {
      const q = this.b.copy(p).project(this.camera);
      return q.z > -1 && q.z < 1 && Math.abs(q.x) <= 1 && Math.abs(q.y) <= 1;
    };
    const p = this.a.copy(this.lastPlayer);
    p.y += PIVOT_HEIGHT * 0.6;
    const playerOnScreen = onScreen(p);
    const t = this.c.copy(this.lastTarget);
    t.y += TARGET_HEIGHT / 2;
    return {
      clearance,
      distance: this.lastDist,
      locked: this.lastLocked,
      playerOnScreen,
      targetOnScreen: this.lastLocked ? onScreen(t) : true
    };
  }
}
