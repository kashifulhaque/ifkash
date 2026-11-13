// B08's legs (enemy ids `b08_forgeheart_colossus_leg_l` and `_leg_r`, registry 37012 and 37013):
// each leg is an actor of its own in the simulation, with a stance meter. They stand at the
// Colossus's sides and carry the hips above them; when a leg's stance breaks it plays
// `colossus_leg_buckle` (the same 60 ticks the body spends kneeling, see `colossus.ts`). Both
// share one model. Render-only.

import type { ModelDef } from '../../procedural/model';
import { lerpPose, type Choreo, type Keyframes } from '../../procedural/pose';
import { Rig, type Pose } from '../../procedural/rig';
import { EMBER, HIP_HEIGHT, IRON, KNEEL_DROP, LAVA, PLATE, SOOT } from './colossus';

/** Thigh and shin length, meters: together they reach the ground from the hip. */
const SEGMENT = HIP_HEIGHT / 2;

function build(): Rig {
  const rig = new Rig();
  rig.bone('hips', null, [0, HIP_HEIGHT, 0]);
  rig.bone('thigh', 'hips', [0, 0, 0]);
  rig.bone('knee', 'thigh', [0, -SEGMENT, 0]);
  rig.bone('foot', 'knee', [0, -SEGMENT, 0]);
  // Thigh: a tapered column with a hip cap that the body's pelvis sits on.
  rig.box('hips', [1.9, 0.5, 2.0], [0, 0.0, 0], PLATE);
  rig.cyl('thigh', 0.95, 0.8, SEGMENT, [0, -SEGMENT / 2, 0], IRON);
  rig.box('thigh', [0.12, SEGMENT * 0.8, 0.06], [0, -SEGMENT / 2, 0.82], LAVA, { glow: 0.9 });
  // Knee: a guard plate and a glowing joint.
  rig.box('knee', [1.7, 0.9, 1.7], [0, 0.05, 0.1], PLATE);
  rig.sphere('knee', 0.35, [0, 0, 0.95], EMBER, { glow: 1.0 });
  // Shin and a heavy foot.
  rig.cyl('knee', 0.8, 0.7, SEGMENT, [0, -SEGMENT / 2, 0], IRON);
  rig.box('knee', [0.12, SEGMENT * 0.7, 0.06], [0, -SEGMENT / 2, 0.74], LAVA, { glow: 0.9 });
  rig.box('foot', [2.1, 0.8, 3.1], [0, 0.4, 0.45], PLATE);
  rig.box('foot', [2.0, 0.12, 3.0], [0, 0.82, 0.45], SOOT);
  rig.box('foot', [1.6, 0.14, 0.14], [0, 0.55, 1.98], EMBER, { glow: 0.8 });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 0.8) * 0.03, 0]
});

/** The leg steps with the Colossus: the thigh swings, the knee folds on the way forward. */
const walk = (phase: number, amp: number): Pose => ({
  thigh: [Math.sin(phase) * 0.35 * amp],
  knee: [Math.max(0, -Math.cos(phase)) * 0.7 * amp + 0.03],
  foot: [-Math.sin(phase) * 0.15 * amp]
});

// Folded under the Colossus: the hips drop by `KNEEL_DROP` while the thigh swings forward and the
// knee bends back, so the sole stays on the ground.
const THIGH = -0.9;
const KNEE = 1.8;
const buckled: Pose = {
  hips: [0, 0, 0, 0, -KNEEL_DROP, 0],
  thigh: [THIGH],
  knee: [KNEE],
  foot: [-(THIGH + KNEE)]
};

const buckle: Choreo = {
  total: 60,
  keys: (): Keyframes => [
    [0, {}],
    [8, buckled],
    [50, buckled],
    [60, lerpPose({}, buckled, 0.15)]
  ],
  glow: (f) => (f < 8 ? 1 + f * 0.2 : f < 50 ? 2.6 : 2.6 - (f - 50) * 0.16)
};

export const colossusLegDef: ModelDef = {
  id: 'b08_forgeheart_colossus_leg',
  height: HIP_HEIGHT,
  build,
  idle,
  walk,
  stride: 6,
  moves: { buckle },
  fallback: buckle,
  fallAxis: 'x',
  // When the Colossus falls its legs give out.
  death: (u) => ({
    hips: [0, 0, 0, 0, -1.5 * u, 0],
    thigh: [-0.8 * u],
    knee: [1.6 * u],
    foot: [-0.8 * u]
  })
};
