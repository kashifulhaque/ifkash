// A shared humanoid skeleton and gait for the procedural biped models. Each model sizes the
// skeleton and adds its own meshes to the named bones.
//
// Bones (right is -X, since models face +Z): hips > spine > head, spine > shoulderL/R >
// elbowL/R > handL/R, hips > hipL/R > kneeL/R > footL/R.

import { Rig, type Pose } from './rig';

export interface BipedDims {
  /** Height of the hips above the ground; the thigh and shin each take half. */
  leg: number;
  torso: number;
  neck: number;
  /** Half the shoulder width and half the hip width. */
  shoulder: number;
  hip: number;
  upperArm: number;
  foreArm: number;
}

export function biped(d: BipedDims): Rig {
  const rig = new Rig();
  rig.bone('hips', null, [0, d.leg, 0]);
  rig.bone('spine', 'hips', [0, 0.02, 0]);
  rig.bone('head', 'spine', [0, d.torso + d.neck, 0]);
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.bone(`shoulder${side}`, 'spine', [s * d.shoulder, d.torso - 0.08, 0]);
    rig.bone(`elbow${side}`, `shoulder${side}`, [0, -d.upperArm, 0]);
    rig.bone(`hand${side}`, `elbow${side}`, [0, -d.foreArm, 0]);
    rig.bone(`hip${side}`, 'hips', [s * d.hip, 0, 0]);
    rig.bone(`knee${side}`, `hip${side}`, [0, -d.leg / 2, 0]);
    rig.bone(`foot${side}`, `knee${side}`, [0, -d.leg / 2, 0]);
  }
  return rig;
}

/** Walking: legs and arms swing in opposition, the body bobs. `amp` scales the swing. */
export function bipedGait(phase: number, amp: number, arms = 0.55): Pose {
  const s = Math.sin(phase);
  const c = Math.cos(phase);
  const sw = 0.65 * amp;
  return {
    hips: [0, 0.08 * amp * s, 0, 0, Math.abs(c) * 0.04 * amp - 0.02 * amp, 0],
    hipL: [-s * sw],
    hipR: [s * sw],
    kneeL: [Math.max(0, -c) * 0.9 * amp + 0.04],
    kneeR: [Math.max(0, c) * 0.9 * amp + 0.04],
    shoulderL: [s * sw * arms],
    shoulderR: [-s * sw * arms]
  };
}

/** Adds `b` onto `a` bone by bone (rotations and offsets add). */
export function addPose(a: Pose, b: Pose): Pose {
  const out: Pose = { ...a };
  for (const [k, v] of Object.entries(b)) {
    const base = a[k];
    if (!base) {
      out[k] = v;
      continue;
    }
    const len = Math.max(base.length, v.length);
    out[k] = Array.from({ length: len }, (_, i) => (base[i] ?? 0) + (v[i] ?? 0));
  }
  return out;
}
