// The Shard Golem (7056): a squat, slow giant of dark stone studded with bright crystal
// shards, the crystals on its back and shoulders reflecting Formulae. Moves: sg_slam (a long
// two-fist overhead slam) and sg_sweep (a wide one-arm sweep). Strike frames come from
// game/data/moves/enemies/shard_golem.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const STONE = 0x4d4a55;
const DARK = 0x2f2d36;
const CRYSTAL = 0x7fd6e8;
const CRYSTAL_DEEP = 0x4a8fd0;

function build(): Rig {
  const rig = biped({ leg: 1.0, torso: 0.9, neck: 0.05, shoulder: 0.62, hip: 0.24, upperArm: 0.62, foreArm: 0.6 });
  rig.restRot('spine', [0.2, 0, 0]);
  rig.box('hips', [0.7, 0.3, 0.45], [0, 0, 0], DARK);
  rig.box('spine', [1.0, 0.95, 0.55], [0, 0.48, 0], STONE);
  rig.box('spine', [0.7, 0.4, 0.2], [0, 0.55, 0.3], DARK);
  rig.sphere('head', 0.24, [0, 0.1, 0.04], STONE, { squash: [1.15, 0.85, 1] });
  rig.box('head', [0.3, 0.05, 0.05], [0, 0.12, 0.2], CRYSTAL, { glow: 1.4 });
  // Crystal shards: a cluster across the back and a spur on each shoulder and the head.
  for (const [x, y, z, h, lean] of [
    [0.25, 0.85, -0.3, 0.7, 0.3],
    [-0.2, 0.9, -0.3, 0.8, -0.25],
    [0.0, 0.95, -0.32, 0.95, 0.0],
    [0.4, 0.6, -0.3, 0.55, 0.6],
    [-0.4, 0.62, -0.3, 0.5, -0.6]
  ] as const) {
    rig.cone('spine', 0.1, h, [x, y + h / 2 - 0.1, z], y > 0.9 ? CRYSTAL : CRYSTAL_DEEP, { rot: [-0.3, 0, -lean], glow: 0.7 });
  }
  rig.cone('head', 0.06, 0.28, [0, 0.38, -0.02], CRYSTAL, { glow: 0.8 });
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.cone(`shoulder${side}`, 0.09, 0.42, [s * 0.1, 0.28, 0], CRYSTAL_DEEP, { rot: [0, 0, -s * 0.5], glow: 0.6 });
    rig.box(`shoulder${side}`, [0.36, 0.34, 0.36], [s * 0.03, 0.02, 0], STONE);
    rig.cyl(`shoulder${side}`, 0.15, 0.17, 0.62, [0, -0.31, 0], STONE);
    rig.cyl(`elbow${side}`, 0.15, 0.19, 0.6, [0, -0.3, 0], DARK);
    // A huge stone fist with a crystal knuckle.
    rig.box(`hand${side}`, [0.3, 0.3, 0.32], [0, -0.06, 0.02], STONE);
    rig.cone(`hand${side}`, 0.06, 0.2, [0, -0.06, 0.2], CRYSTAL, { rot: [Math.PI / 2, 0, 0], glow: 0.7 });
    rig.cyl(`hip${side}`, 0.19, 0.17, 0.5, [0, -0.25, 0], STONE);
    rig.cyl(`knee${side}`, 0.17, 0.19, 0.5, [0, -0.25, 0], DARK);
    rig.box(`foot${side}`, [0.32, 0.14, 0.5], [0, 0.07, 0.1], STONE);
  }
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 0.9) * 0.008, 0],
  spine: [Math.sin(t * 0.9) * 0.02, Math.sin(t * 0.4) * 0.04, 0],
  head: [Math.sin(t * 0.7) * 0.05, Math.sin(t * 0.35) * 0.25, 0],
  shoulderL: [Math.sin(t * 0.8) * 0.03, 0, 0.18],
  shoulderR: [Math.sin(t * 0.8 + 1) * 0.03, 0, -0.18],
  elbowL: [-0.2],
  elbowR: [-0.2]
});

const walk = (phase: number, amp: number): Pose => bipedGait(phase, amp * 0.75, 0.25);

const slam: Choreo = {
  total: 92,
  keys: (total) =>
    attack(36, 40, total, {
      // A long, straining rise: both fists high over the head, back arched.
      ready: { shoulderL: [-1.4, 0, 0.1], shoulderR: [-1.4, 0, -0.1], elbowL: [-0.4], elbowR: [-0.4], spine: [-0.1] },
      wind: {
        shoulderL: [-3.0, 0, 0.12],
        shoulderR: [-3.0, 0, -0.12],
        elbowL: [-0.25],
        elbowR: [-0.25],
        spine: [-0.5, 0, 0],
        head: [-0.2],
        kneeL: [0.25],
        kneeR: [0.25]
      },
      strike: {
        shoulderL: [-1.05, 0, 0.1],
        shoulderR: [-1.05, 0, -0.1],
        elbowL: [-0.05],
        elbowR: [-0.05],
        spine: [0.65, 0, 0],
        head: [0.3],
        hips: [0, 0, 0, 0, -0.14, 0.2],
        kneeL: [0.5],
        kneeR: [0.5]
      }
    })
};

const sweep: Choreo = {
  total: 80,
  keys: (total) =>
    attack(30, 36, total, {
      wind: { shoulderR: [-0.9, 0.6, -2.0], elbowR: [-0.3], spine: [0, -0.9, 0], hips: [0, -0.2, 0], head: [0, 0.4, 0] },
      strike: { shoulderR: [-1.5, -0.4, 1.0], elbowR: [-0.05], spine: [0.1, 0.9, 0], hips: [0, 0.35, 0, 0, -0.05, 0.1], head: [0, -0.3, 0] }
    })
};

export const shardGolemDef: ModelDef = {
  id: 'shard_golem',
  height: 2.8,
  build,
  idle,
  walk,
  stride: 1.6,
  moves: { sg_slam: slam, sg_sweep: sweep },
  fallback: slam,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.4 * u], head: [0.4 * u], kneeL: [0.8 * u], kneeR: [0.8 * u], shoulderL: [-0.3 * u], shoulderR: [-0.3 * u] })
};
