// Dev-only camera helpers, switched on by URL parameters. WP-014 replaces the parameter
// parsing with the shared dev-URL parser; until then this reads what it needs itself.
//
//   &cam=orbit          free orbit camera (the normal third-person behavior)
//   &cam=orbit-script   ignores input and sweeps the camera around the test field's ledge:
//                       two full circles at each of three spots (the ledge wall, the
//                       steep ridge, the top edge of the 10 m drop), pitch cycling its full range
//   &camt=SECONDS      with orbit-script, holds the script at that time (0 to 72)
//   &fakelock=X,Y,Z     frames a fixed point as if it were the locked-on actor's feet
//   &scenario=NAME      starts that scenario (default m0_flat); test_field has the ledge
//
// `window.__lfCamera` exposes `debug()` (camera clearance and on-screen flags) and
// `sweep()`, which runs the orbit script offline and returns the least clearance seen.

import { Vector3 } from 'three';
import type { CameraDebug, CameraRig } from './CameraRig';
import { PITCH_MAX, PITCH_MIN } from './limits';
import type { DrawActor } from '../../engine/snapshot';

export interface DevCameraOptions {
  /** True for any `&cam=` value: draw the level proxy so walls are visible. */
  active: boolean;
  /** `&cam=orbit-script` is on. */
  script: boolean;
  fakeLock: Vector3 | null;
  /** `&camt=SECONDS` freezes the orbit script at that time, for screenshots. */
  scriptAt: number | null;
  scenario: string | null;
}

export function parseDevCamera(search: string): DevCameraOptions {
  const p = new URLSearchParams(search);
  const cam = p.get('cam');
  const lock = p.get('fakelock')?.split(',').map(Number);
  const fakeLock = lock && lock.length === 3 && lock.every(Number.isFinite) ? new Vector3(lock[0], lock[1], lock[2]) : null;
  const at = p.get('camt');
  const scriptAt = at !== null && Number.isFinite(Number(at)) ? Number(at) : null;
  return { active: cam !== null, script: cam === 'orbit-script', fakeLock, scriptAt, scenario: p.get('scenario') };
}

/** Seconds for one full circle of the script. */
const CIRCLE_S = 12;
const CIRCLES_PER_STAGE = 2;
/**
 * Orbit pivots, each hard against geometry: the landing at the foot of the ledge wall
 * (z = 30), the base of the steep ridge (x = 20..28, slope from z = -8), and the top edge of the
 * 10 m drop (z = 40).
 */
const STAGES = [new Vector3(0, 9 + 1.6, 29.6), new Vector3(24, 1.6, -8.6), new Vector3(0, 10 + 1.6, 39.6)];
export const SCRIPT_LENGTH_S = STAGES.length * CIRCLES_PER_STAGE * CIRCLE_S;

export interface ScriptPose {
  pivot: Vector3;
  /** Which pivot this is. The pivots are far apart, so the rig is reset when it changes. */
  stage: number;
  yaw: number;
  pitch: number;
}

/** Where the orbit script has the camera `t` seconds in. Loops after `SCRIPT_LENGTH_S`. */
export function scriptPose(t: number): ScriptPose {
  const s = t % SCRIPT_LENGTH_S;
  const stage = Math.min(STAGES.length - 1, Math.floor(s / (CIRCLES_PER_STAGE * CIRCLE_S)));
  const local = s - stage * CIRCLES_PER_STAGE * CIRCLE_S;
  const yaw = (local / CIRCLE_S) * Math.PI * 2;
  // Pitch sweeps its full clamped range three times per circle, out of phase with the yaw.
  const wave = Math.sin((local / CIRCLE_S) * Math.PI * 2 * 3 + 0.7);
  const mid = (PITCH_MAX + PITCH_MIN) / 2;
  const half = (PITCH_MAX - PITCH_MIN) / 2;
  return { pivot: STAGES[stage], stage, yaw, pitch: mid + wave * half };
}

declare global {
  interface Window {
    __lfCamera?: {
      debug(): CameraDebug;
      at(t: number): CameraDebug & { x: number; y: number; z: number };
      sweep(): {
        frames: number;
        minClearance: number;
        minDistance: number;
        worstT: number;
        worstPos: number[];
        worst: ScriptPose | null;
      };
    };
  }
}

/**
 * Registers `window.__lfCamera`. `sweep` steps the orbit script offline at 60 fps against
 * the live level and reports the least clearance seen; a camera inside a wall reads 0.
 */
export function exposeDebug(rig: CameraRig, drawn: () => DrawActor[]): void {
  // Before the first rendered frame there are no actors; stand in a player so the rig runs.
  const dummy: DrawActor = { id: 1, kind: 0, flags: 0, x: 0, y: 0, z: 0, yaw: 0, moveId: 0, moveFrame: 0, hpFrac: 1, poiseFrac: 1, statusMask: 0 };
  const current = (): DrawActor[] => (drawn().length ? drawn() : [dummy]);
  window.__lfCamera = {
    debug: () => rig.debug(true),
    at(t) {
      const pose = scriptPose(t);
      rig.reset();
      rig.update(current(), pose.yaw, pose.pitch, 1000 / 60, pose.pivot);
      rig.reset();
      const p = rig.camera.position;
      return { x: p.x, y: p.y, z: p.z, ...rig.debug(true) };
    },
    sweep() {
      const actors = current();
      rig.reset();
      let min = Infinity;
      let worst: ScriptPose | null = null;
      let frames = 0;
      let stage = -1;
      let worstT = 0;
      let worstPos: number[] = [];
      let minDist = Infinity;
      for (let t = 0; t < SCRIPT_LENGTH_S; t += 1 / 60) {
        const pose = scriptPose(t);
        if (pose.stage !== stage) {
          stage = pose.stage;
          rig.reset();
        }
        rig.update(actors, pose.yaw, pose.pitch, 1000 / 60, pose.pivot);
        const d = rig.debug(true);
        minDist = Math.min(minDist, d.distance);
        if (d.clearance < min) {
          min = d.clearance;
          worst = pose;
          worstT = t;
          worstPos = rig.camera.position.toArray();
        }
        frames++;
      }
      rig.reset();
      return { frames, minClearance: min, minDistance: minDist, worstT, worstPos, worst };
    }
  };
}
