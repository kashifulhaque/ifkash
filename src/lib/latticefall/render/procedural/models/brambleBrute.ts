// The Bramble Brute (7083): a huge husk wrapped in thorny bramble, slow and long-armed. Moves:
// bb_swing_1 (a heavy swing), bb_swing_2 (the backhand that chains from it) and bb_slam (the
// two-fisted overhead slam that ends the combo). Strike frames come from
// game/data/moves/enemies/bramble_brute.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const SKIN = 0x9a9482;
const CLOTH = 0x4d493c;
const VINE = 0x4a5a2a;
const THORN = 0x8d3b24;
const DARK = 0x16140f;

function build(): Rig {
  const rig = biped({ leg: 1.05, torso: 0.9, neck: 0.08, shoulder: 0.5, hip: 0.2, upperArm: 0.7, foreArm: 0.75 });
  rig.restRot('spine', [0.35, 0, 0]);
  rig.box('hips', [0.6, 0.3, 0.4], [0, 0, 0], CLOTH);
  rig.box('spine', [0.8, 0.9, 0.46], [0, 0.48, 0], SKIN);
  rig.box('spine', [0.84, 0.4, 0.5], [0, 0.28, 0.01], CLOTH);
  rig.sphere('head', 0.24, [0, 0.14, 0.04], SKIN, { squash: [0.95, 1.1, 1] });
  for (const s of [1, -1]) rig.box('head', [0.09, 0.09, 0.05], [s * 0.09, 0.18, 0.21], DARK);
  rig.box('head', [0.16, 0.07, 0.05], [0, 0.03, 0.22], DARK);
  // Bramble: vines coiled across the torso and limbs, and thorns thrust out of them.
  rig.box('spine', [0.9, 0.07, 0.52], [0, 0.45, 0], VINE, { rot: [0, 0, 0.3] });
  rig.box('spine', [0.9, 0.07, 0.52], [0, 0.7, 0], VINE, { rot: [0, 0, -0.3] });
  for (const [x, y, z, h, rx, rz] of [
    [0.3, 0.9, -0.2, 0.45, -0.4, -0.3],
    [-0.25, 0.95, -0.2, 0.5, -0.4, 0.3],
    [0.0, 1.0, -0.22, 0.6, -0.5, 0.0],
    [0.4, 0.55, 0.2, 0.3, 0.6, -0.9],
    [-0.4, 0.6, 0.22, 0.28, 0.6, 0.9],
    [0.42, 0.3, -0.2, 0.34, -0.5, -0.8]
  ] as const) {
    rig.cone('spine', 0.06, h, [x, y + h / 2 - 0.05, z], THORN, { rot: [rx, 0, rz] });
  }
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.sphere(`shoulder${side}`, 0.2, [s * 0.02, 0, 0], VINE);
    rig.cyl(`shoulder${side}`, 0.12, 0.14, 0.7, [0, -0.35, 0], SKIN);
    rig.cyl(`elbow${side}`, 0.12, 0.14, 0.75, [0, -0.37, 0], SKIN);
    rig.cyl(`elbow${side}`, 0.15, 0.15, 0.08, [0, -0.2, 0], VINE);
    for (const dz of [-0.4, -0.2]) rig.cone(`elbow${side}`, 0.035, 0.2, [s * 0.14, dz, 0], THORN, { rot: [0, 0, -s * 1.4] });
    rig.box(`hand${side}`, [0.22, 0.24, 0.2], [0, -0.04, 0], SKIN);
    for (const dx of [-0.07, 0, 0.07]) rig.cone(`hand${side}`, 0.025, 0.16, [dx, -0.2, 0.04], THORN, { rot: [Math.PI, 0, 0] });
    rig.cyl(`hip${side}`, 0.14, 0.12, 0.52, [0, -0.26, 0], CLOTH);
    rig.cyl(`knee${side}`, 0.11, 0.1, 0.52, [0, -0.26, 0], SKIN);
    rig.box(`foot${side}`, [0.2, 0.1, 0.38], [0, 0.05, 0.08], SKIN);
  }
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.0) * 0.01, 0],
  spine: [Math.sin(t * 1.0) * 0.03, Math.sin(t * 0.5) * 0.05, 0],
  head: [0.1 + Math.sin(t * 0.8) * 0.05, Math.sin(t * 0.4) * 0.2, 0],
  shoulderL: [Math.sin(t * 0.9) * 0.04, 0, 0.16],
  shoulderR: [Math.sin(t * 0.9 + 1) * 0.04, 0, -0.16],
  elbowL: [-0.14],
  elbowR: [-0.14]
});

const walk = (phase: number, amp: number): Pose => ({
  ...bipedGait(phase, amp * 0.7, 0.35),
  head: [0.1, Math.sin(phase) * 0.15 * amp, 0]
});

const swing1: Choreo = {
  total: 70,
  keys: (total) =>
    attack(24, 28, total, {
      wind: { shoulderR: [-2.6, 0, -1.0], elbowR: [-0.6], spine: [-0.15, -0.7, 0], head: [0, 0.3, 0] },
      strike: { shoulderR: [-1.3, 0, 0.7], elbowR: [-0.1], spine: [0.4, 0.8, 0], hips: [0, 0.25, 0, 0, 0, 0.12] }
    })
};

const swing2: Choreo = {
  total: 64,
  keys: (total) =>
    attack(22, 26, total, {
      wind: { shoulderL: [-1.5, 0, 1.7], elbowL: [-0.4], spine: [0, 0.7, 0], head: [0, -0.3, 0] },
      strike: { shoulderL: [-1.4, 0, -0.6], elbowL: [-0.1], spine: [0.35, -0.8, 0], hips: [0, -0.25, 0, 0, 0, 0.12] }
    })
};

const slam: Choreo = {
  total: 90,
  keys: (total) =>
    attack(34, 38, total, {
      wind: {
        shoulderL: [-3.0, 0, 0.15],
        shoulderR: [-3.0, 0, -0.15],
        elbowL: [-0.3],
        elbowR: [-0.3],
        spine: [-0.6, 0, 0],
        head: [-0.2],
        kneeL: [0.3],
        kneeR: [0.3]
      },
      strike: {
        shoulderL: [-1.0, 0, 0.12],
        shoulderR: [-1.0, 0, -0.12],
        elbowL: [-0.05],
        elbowR: [-0.05],
        spine: [0.7, 0, 0],
        hips: [0, 0, 0, 0, -0.16, 0.22],
        kneeL: [0.5],
        kneeR: [0.5]
      }
    })
};

export const brambleBruteDef: ModelDef = {
  id: 'bramble_brute',
  height: 2.9,
  build,
  idle,
  walk,
  stride: 1.7,
  moves: { bb_swing_1: swing1, bb_swing_2: swing2, bb_slam: slam },
  fallback: swing1,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.5 * u], head: [0.4 * u], kneeL: [0.8 * u], kneeR: [0.9 * u], shoulderL: [-0.3 * u], shoulderR: [-0.3 * u] })
};
