// The Gray Knight Captain (7085): a tall duelist in dark gray plate with a crimson cape and
// plume, a two-handed greatsword held low. Its swings are held back for a long beat before they
// fall (parryable, delayed). Moves: gc_slash, gc_overhead, gc_thrust, gc_flurry and gc_cut.
// Strike frames come from game/data/moves/enemies/gray_captain.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const STEEL = 0x6f757f;
const DARK = 0x363b44;
const CRIMSON = 0x8a2a30;
const GOLD = 0xc9a24a;
const LEATHER = 0x4a3b2c;

function build(): Rig {
  const rig = biped({ leg: 1.0, torso: 0.72, neck: 0.08, shoulder: 0.29, hip: 0.13, upperArm: 0.4, foreArm: 0.4 });
  rig.box('hips', [0.38, 0.2, 0.26], [0, 0, 0], DARK);
  rig.box('spine', [0.48, 0.68, 0.3], [0, 0.35, 0], STEEL);
  rig.box('spine', [0.4, 0.06, 0.32], [0, 0.12, 0], GOLD);
  rig.box('hips', [0.3, 0.42, 0.04], [0, -0.22, 0.14], CRIMSON);
  // The cape hangs behind from the shoulders.
  rig.box('spine', [0.52, 0.95, 0.03], [0, 0.2, -0.2], CRIMSON, { rot: [0.08, 0, 0] });
  // A helm with a visor slit and a swept crimson plume.
  rig.box('head', [0.3, 0.36, 0.32], [0, 0.14, 0], STEEL);
  rig.box('head', [0.24, 0.035, 0.04], [0, 0.17, 0.165], 0x0c0d10);
  rig.box('head', [0.03, 0.2, 0.04], [0, 0.1, 0.165], 0x0c0d10);
  rig.cone('head', 0.07, 0.5, [0, 0.45, -0.18], CRIMSON, { rot: [-1.2, 0, 0] });
  for (const side of ['L', 'R'] as const) {
    const s = side === 'L' ? 1 : -1;
    rig.sphere(`shoulder${side}`, 0.15, [s * 0.02, 0.02, 0], STEEL, { squash: [1.2, 0.85, 1] });
    rig.cyl(`shoulder${side}`, 0.065, 0.06, 0.4, [0, -0.2, 0], DARK);
    rig.cyl(`elbow${side}`, 0.06, 0.07, 0.4, [0, -0.2, 0], STEEL);
    rig.box(`hand${side}`, [0.1, 0.1, 0.1], [0, 0, 0], LEATHER);
    rig.cyl(`hip${side}`, 0.095, 0.075, 0.5, [0, -0.25, 0], STEEL);
    rig.cyl(`knee${side}`, 0.075, 0.07, 0.5, [0, -0.25, 0], DARK);
    rig.box(`foot${side}`, [0.14, 0.1, 0.3], [0, 0.05, 0.07], STEEL);
  }
  // The greatsword: a long blade, a gold cross-guard and a pommel, carried tip forward and down.
  rig.bone('sword', 'handR', [0, -0.02, 0.02]);
  rig.restRot('sword', [-0.15, 0, 0]);
  rig.box('sword', [0.05, 0.28, 0.05], [0, 0.1, 0], LEATHER);
  rig.sphere('sword', 0.04, [0, 0.26, 0], GOLD);
  rig.box('sword', [0.4, 0.05, 0.07], [0, -0.06, 0], GOLD);
  rig.box('sword', [0.1, 1.5, 0.025], [0, -0.85, 0], 0xc4c9d1);
  rig.cone('sword', 0.05, 0.2, [0, -1.7, 0], 0xc4c9d1, { rot: [Math.PI, 0, 0] });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.3) * 0.006, 0],
  spine: [Math.sin(t * 1.3) * 0.012, -0.35, 0],
  head: [0, 0.3 + Math.sin(t * 0.5) * 0.04, 0],
  // A duelist's stance: weight forward, sword low and ahead, the left hand on the grip.
  shoulderR: [-0.7, 0, -0.1],
  elbowR: [-0.8],
  shoulderL: [-0.9, 0, -0.5],
  elbowL: [-0.9],
  hipL: [0.12],
  hipR: [-0.22],
  kneeL: [0.1],
  kneeR: [0.3]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.2);
  delete g.shoulderL;
  delete g.shoulderR;
  return { ...g, spine: [0, -0.35, 0] };
};

const GRIP_L = { shoulderL: [-1.4, 0, -0.35], elbowL: [-0.9] };

const slash: Choreo = {
  total: 74,
  // Held back for 34 frames: the sword draws across and waits, shoulders turned.
  keys: (total) =>
    attack(34, 38, total, {
      ready: { shoulderR: [-1.0, 0, -0.8], elbowR: [-0.8], ...GRIP_L, spine: [0, -0.7, 0] },
      wind: { shoulderR: [-1.3, 0, -1.7], elbowR: [-0.6], ...GRIP_L, spine: [0, -1.0, 0], head: [0, 0.45, 0] },
      strike: { shoulderR: [-1.5, 0, 0.8], elbowR: [-0.1], shoulderL: [-1.4, 0, 0.3], elbowL: [-0.4], spine: [0.1, 0.8, 0], hips: [0, 0.35, 0, 0, 0, 0.12] }
    })
};

const overhead: Choreo = {
  total: 84,
  keys: (total) =>
    attack(44, 48, total, {
      ready: { shoulderR: [-1.8, 0, -0.2], elbowR: [-0.9], ...GRIP_L, spine: [-0.1, -0.2, 0] },
      wind: { shoulderR: [-3.0, 0, -0.3], elbowR: [-0.4], shoulderL: [-3.0, 0, 0.3], elbowL: [-0.4], spine: [-0.4, 0, 0], head: [-0.2, 0, 0] },
      strike: { shoulderR: [-1.0, 0, -0.1], elbowR: [-0.1], shoulderL: [-1.1, 0, 0.1], elbowL: [-0.1], spine: [0.55, 0, 0], hips: [0, 0, 0, 0, -0.1, 0.25] }
    })
};

const thrust: Choreo = {
  total: 54,
  keys: (total) =>
    attack(20, 23, total, {
      wind: { shoulderR: [-0.8, 0.2, -0.5], elbowR: [-2.1], ...GRIP_L, spine: [0, -0.7, 0] },
      strike: { shoulderR: [-1.55, 0, -0.05], elbowR: [-0.05], shoulderL: [-1.5, 0, 0.1], elbowL: [-0.2], spine: [0.1, 0.4, 0], hips: [0, 0, 0, 0, -0.14, 0.45], hipL: [-0.6], hipR: [0.5] }
    })
};

const flurry: Choreo = {
  total: 52,
  keys: (total) =>
    attack(18, 22, total, {
      wind: { shoulderR: [-1.3, 0, -1.6], elbowR: [-0.6], ...GRIP_L, spine: [0, -0.9, 0] },
      strike: { shoulderR: [-1.4, 0, 0.7], elbowR: [-0.1], shoulderL: [-1.3, 0, 0.3], elbowL: [-0.4], spine: [0.1, 0.8, 0], hips: [0, 0.3, 0, 0, 0, 0.1] }
    })
};

/** The fast return cut of the flurry, from the follow-through side back across. */
const cut: Choreo = {
  total: 50,
  keys: (total) =>
    attack(18, 22, total, {
      wind: { shoulderR: [-1.3, 0, 0.9], elbowR: [-0.4], shoulderL: [-1.3, 0, 0.4], elbowL: [-0.6], spine: [0.1, 0.8, 0] },
      strike: { shoulderR: [-1.4, 0, -1.3], elbowR: [-0.1], ...GRIP_L, spine: [0.1, -0.8, 0], hips: [0, -0.3, 0, 0, 0, 0.1] }
    })
};

export const grayCaptainDef: ModelDef = {
  id: 'gray_captain',
  height: 2.2,
  build,
  idle,
  walk,
  stride: 1.7,
  moves: { gc_slash: slash, gc_overhead: overhead, gc_thrust: thrust, gc_flurry: flurry, gc_cut: cut },
  fallback: slash,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.3 * u], head: [0.5 * u], kneeL: [0.6 * u], kneeR: [0.9 * u] })
};
