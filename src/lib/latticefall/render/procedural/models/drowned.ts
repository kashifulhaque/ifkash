// The Drowned (7051): a waterlogged, slow-moving corpse with long reaching arms and weed for
// hair. Moves: dr_grab (both arms out, a 24-frame reach; unblockable) and dr_swipe (a slow
// arm swing). Strike frames come from game/data/moves/enemies/drowned.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const SKIN = 0x7f9b94;
const DARK_SKIN = 0x56706b;
const CLOTH = 0x3d4a40;
const WEED = 0x2d5a3a;

function build(): Rig {
  const rig = biped({ leg: 0.78, torso: 0.55, neck: 0.08, shoulder: 0.22, hip: 0.1, upperArm: 0.44, foreArm: 0.48 });
  rig.restRot('spine', [0.35, 0, 0]);
  rig.box('hips', [0.3, 0.18, 0.2], [0, 0, 0], CLOTH);
  rig.box('spine', [0.38, 0.56, 0.22], [0, 0.3, 0], SKIN);
  rig.box('spine', [0.4, 0.26, 0.24], [0, 0.14, 0], CLOTH);
  rig.sphere('head', 0.17, [0, 0.12, 0.03], SKIN, { squash: [0.95, 1.1, 1] });
  // Pale blank eyes and a slack open mouth.
  for (const s of [1, -1]) rig.sphere('head', 0.03, [s * 0.065, 0.14, 0.15], 0xd6efe8, { glow: 0.7 });
  rig.box('head', [0.09, 0.07, 0.03], [0, 0.03, 0.155], 0x14201c);
  // Weed: long strands hanging from the head and shoulders.
  for (const [x, z, h] of [
    [0.1, -0.08, 0.5],
    [-0.1, -0.08, 0.42],
    [0.0, -0.12, 0.6]
  ] as const) {
    rig.cone('head', 0.04, h, [x, 0.12 - h / 2, z], WEED, { rot: [Math.PI, 0, 0] });
  }
  rig.cone('spine', 0.06, 0.4, [0.18, 0.5, -0.1], WEED, { rot: [Math.PI, 0, 0.2] });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.05, 0.06, 0.44, [0, -0.22, 0], SKIN);
    rig.cyl(`elbow${side}`, 0.045, 0.05, 0.48, [0, -0.24, 0], DARK_SKIN);
    for (const dx of [-0.03, 0, 0.03]) rig.cone(`hand${side}`, 0.016, 0.14, [dx, -0.07, 0.01], 0xc9d6d0, { rot: [Math.PI, 0, 0] });
    rig.box(`hand${side}`, [0.09, 0.08, 0.04], [0, 0, 0], SKIN);
    rig.cyl(`hip${side}`, 0.065, 0.055, 0.39, [0, -0.19, 0], CLOTH);
    rig.cyl(`knee${side}`, 0.05, 0.045, 0.39, [0, -0.19, 0], DARK_SKIN);
    rig.box(`foot${side}`, [0.1, 0.06, 0.22], [0, 0.03, 0.05], DARK_SKIN);
  }
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.0) * 0.01, 0],
  spine: [Math.sin(t * 1.0) * 0.03, Math.sin(t * 0.5) * 0.06, 0],
  head: [0.15 + Math.sin(t * 0.8) * 0.06, Math.sin(t * 0.4) * 0.25, Math.sin(t * 0.7) * 0.1],
  shoulderL: [Math.sin(t * 0.9) * 0.06, 0, 0.1],
  shoulderR: [Math.sin(t * 0.9 + 1) * 0.06, 0, -0.1],
  elbowL: [-0.2, 0, 0],
  elbowR: [-0.2, 0, 0]
});

const walk = (phase: number, amp: number): Pose => ({
  ...bipedGait(phase, amp * 0.8, 0.3),
  head: [0.15, Math.sin(phase) * 0.2 * amp, Math.sin(phase * 2) * 0.12 * amp]
});

const grab: Choreo = {
  total: 72,
  keys: (total) =>
    attack(24, 28, total, {
      // Both arms drawn out wide and back: the reach is coming.
      wind: {
        shoulderL: [-0.4, 0, 1.7],
        shoulderR: [-0.4, 0, -1.7],
        elbowL: [-0.4],
        elbowR: [-0.4],
        spine: [-0.15, 0, 0],
        head: [-0.2, 0, 0]
      },
      strike: {
        shoulderL: [-1.55, 0, 0.35],
        shoulderR: [-1.55, 0, -0.35],
        elbowL: [-0.05],
        elbowR: [-0.05],
        spine: [0.35, 0, 0],
        hips: [0, 0, 0, 0, -0.06, 0.25],
        head: [0.2, 0, 0]
      }
    })
};

const swipe: Choreo = {
  total: 58,
  keys: (total) =>
    attack(20, 23, total, {
      wind: { shoulderR: [-1.4, 0, -1.6], elbowR: [-0.4], spine: [0, -0.7, 0], head: [0, 0.3, 0] },
      strike: { shoulderR: [-1.4, 0, 0.6], elbowR: [-0.1], spine: [0.3, 0.8, 0], hips: [0, 0.25, 0, 0, 0, 0.1] }
    })
};

export const drownedDef: ModelDef = {
  id: 'drowned',
  height: 1.7,
  build,
  idle,
  walk,
  stride: 0.9,
  moves: { dr_grab: grab, dr_swipe: swipe },
  fallback: swipe,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.6 * u], head: [0.4 * u], kneeL: [1.0 * u], kneeR: [0.9 * u], shoulderL: [-0.3 * u], shoulderR: [-0.3 * u] })
};
