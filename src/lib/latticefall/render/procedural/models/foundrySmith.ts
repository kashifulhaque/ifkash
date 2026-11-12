// The Foundry Smith (7086): a broad, heavy smith in a scorched leather apron and a soot-black
// hood, a glowing-hot hammer in the right hand and long tongs in the left. Moves: fs_hammer (a
// long wind-up, then the slam that leaves burning metal), fs_sweep (a wide hammer sweep) and
// fs_tongs (a quick poke). Strike frames come from game/data/moves/enemies/foundry_smith.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const SKIN = 0x8a6a52;
const APRON = 0x4a3526;
const SOOT = 0x25221f;
const IRON = 0x585d66;
const HOT = 0xff6a1f;

function build(): Rig {
  const rig = biped({ leg: 0.95, torso: 0.8, neck: 0.06, shoulder: 0.4, hip: 0.16, upperArm: 0.46, foreArm: 0.46 });
  rig.restRot('spine', [0.12, 0, 0]);
  rig.box('hips', [0.5, 0.24, 0.32], [0, 0, 0], SOOT);
  rig.box('spine', [0.64, 0.78, 0.38], [0, 0.4, 0], SKIN);
  // The leather apron, front, with a scorched hem.
  rig.box('spine', [0.58, 0.8, 0.04], [0, 0.2, 0.2], APRON);
  rig.box('hips', [0.5, 0.6, 0.04], [0, -0.26, 0.17], APRON);
  rig.box('hips', [0.52, 0.04, 0.05], [0, -0.55, 0.17], 0x1a1511);
  // A soot-black hood and a mask with two furnace-glow eyes.
  rig.sphere('head', 0.2, [0, 0.12, 0], SOOT, { squash: [1, 1.1, 1.05] });
  rig.box('head', [0.2, 0.14, 0.05], [0, 0.1, 0.16], 0x111010);
  for (const s of [1, -1]) rig.sphere('head', 0.022, [s * 0.05, 0.12, 0.19], HOT, { glow: 1.6 });
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.sphere(`shoulder${side}`, 0.17, [s * 0.02, 0.02, 0], SKIN, { squash: [1.1, 0.9, 1] });
    rig.cyl(`shoulder${side}`, 0.1, 0.11, 0.46, [0, -0.23, 0], SKIN);
    rig.cyl(`elbow${side}`, 0.095, 0.1, 0.46, [0, -0.23, 0], IRON);
    rig.box(`hand${side}`, [0.14, 0.14, 0.14], [0, 0, 0], SOOT);
    rig.cyl(`hip${side}`, 0.12, 0.1, 0.48, [0, -0.24, 0], SOOT);
    rig.cyl(`knee${side}`, 0.1, 0.09, 0.48, [0, -0.24, 0], APRON);
    rig.box(`foot${side}`, [0.18, 0.12, 0.34], [0, 0.06, 0.07], SOOT);
  }
  // The hammer: a long haft with a heavy head that glows with the forge's heat.
  rig.bone('hammer', 'handR', [0, 0, 0.02]);
  rig.restRot('hammer', [1.6, 0, 0]);
  rig.cyl('hammer', 0.032, 0.032, 1.5, [0, 0.1, 0], 0x5a4128);
  rig.box('hammer', [0.3, 0.3, 0.55], [0, 0.9, 0], IRON);
  rig.box('hammer', [0.32, 0.32, 0.12], [0, 0.9, 0.32], HOT, { glow: 0.9 });
  rig.box('hammer', [0.32, 0.32, 0.12], [0, 0.9, -0.32], HOT, { glow: 0.9 });
  // Tongs: two long iron arms gripping a glowing ingot.
  rig.bone('tongs', 'handL', [0, 0, 0.02]);
  rig.restRot('tongs', [-0.6, 0, 0]);
  for (const s of [1, -1]) rig.box('tongs', [0.025, 1.0, 0.03], [s * 0.025, -0.4, 0], IRON, { rot: [0, 0, s * 0.03] });
  rig.box('tongs', [0.1, 0.1, 0.1], [0, -0.92, 0.02], HOT, { glow: 1.1 });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.0) * 0.008, 0],
  spine: [Math.sin(t * 1.0) * 0.02, Math.sin(t * 0.4) * 0.05, 0],
  head: [0.05, Math.sin(t * 0.5) * 0.15, 0],
  shoulderR: [-0.5, 0, -0.2],
  elbowR: [-0.9],
  shoulderL: [-0.4, 0, 0.25],
  elbowL: [-0.7]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp * 0.8, 0.25);
  delete g.shoulderR;
  delete g.shoulderL;
  return g;
};

const hammer: Choreo = {
  total: 150,
  // The slam at frame 36 leaves burning metal; the long tail keeps the smith bent over it.
  keys: (total) => {
    const keys = attack(36, 40, total, {
      ready: { shoulderR: [-1.5, 0, -0.1], shoulderL: [-1.0, 0, 0.5], elbowR: [-0.6], spine: [-0.1] },
      wind: {
        shoulderR: [-3.0, 0, -0.2],
        elbowR: [-0.3],
        shoulderL: [-2.4, 0, 0.5],
        elbowL: [-0.4],
        hammer: [0.5],
        spine: [-0.5, 0, 0],
        head: [-0.2],
        kneeL: [0.3],
        kneeR: [0.3]
      },
      strike: {
        shoulderR: [-1.0, 0, -0.15],
        elbowR: [-0.05],
        shoulderL: [-1.2, 0, 0.3],
        elbowL: [-0.2],
        hammer: [1.1],
        spine: [0.65, 0, 0],
        head: [0.25],
        hips: [0, 0, 0, 0, -0.16, 0.2],
        kneeL: [0.5],
        kneeR: [0.5]
      }
    });
    return keys;
  },
  // The hammer head brightens through the wind-up and flares on the strike.
  glow: (f) => (f < 36 ? 1 + (f / 36) * 1.6 : f < 44 ? 3.2 : Math.max(1, 3.2 - (f - 44) * 0.05))
};

const sweep: Choreo = {
  total: 76,
  keys: (total) =>
    attack(28, 33, total, {
      wind: { shoulderR: [-0.8, 0.5, -2.1], elbowR: [-0.3], shoulderL: [-0.8, 0, 1.0], spine: [0, -1.0, 0], hips: [0, -0.2, 0], head: [0, 0.4, 0] },
      strike: { shoulderR: [-1.4, -0.4, 1.0], elbowR: [-0.05], shoulderL: [-0.8, 0, -0.1], spine: [0.1, 1.0, 0], hips: [0, 0.4, 0, 0, -0.05, 0.1], head: [0, -0.3, 0] }
    }),
  glow: (f) => (f < 28 ? 1 + (f / 28) * 1.2 : 2.2)
};

const tongs: Choreo = {
  total: 52,
  keys: (total) =>
    attack(18, 21, total, {
      wind: { shoulderL: [-0.6, 0.2, 0.5], elbowL: [-2.0], spine: [0, 0.5, 0] },
      strike: { shoulderL: [-1.5, 0, 0.05], elbowL: [-0.05], spine: [0.15, -0.4, 0], hips: [0, 0, 0, 0, -0.08, 0.3], hipL: [-0.5], hipR: [0.4] }
    })
};

export const foundrySmithDef: ModelDef = {
  id: 'foundry_smith',
  height: 2.2,
  build,
  idle,
  walk,
  stride: 1.5,
  moves: { fs_hammer: hammer, fs_sweep: sweep, fs_tongs: tongs },
  fallback: hammer,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.4 * u], head: [0.5 * u], kneeL: [0.7 * u], kneeR: [0.9 * u], shoulderR: [0.3 * u] })
};
