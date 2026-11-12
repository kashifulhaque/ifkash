// The Choir Knight (7084): a tall knight in pale blue plate and a white surplice, a halberd in
// the right hand and a hand bell in the left. Moves: ck_halberd_1 and ck_halberd_2 (the
// sweeping cut and its return), ck_thrust, and ck_bell (lifts the bell for 36 frames, then
// rings it: the stun). Strike frames come from game/data/moves/enemies/choir_knight.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo, type Keyframes } from '../pose';
import type { Pose, Rig } from '../rig';

const PLATE = 0xa9b9c9;
const DARK = 0x4d5a6b;
const SURPLICE = 0xe6e9ee;
const GOLD = 0xd0a23c;

function build(): Rig {
  const rig = biped({ leg: 1.0, torso: 0.72, neck: 0.08, shoulder: 0.3, hip: 0.13, upperArm: 0.4, foreArm: 0.4 });
  rig.box('hips', [0.38, 0.2, 0.26], [0, 0, 0], DARK);
  rig.box('spine', [0.5, 0.68, 0.3], [0, 0.35, 0], PLATE);
  // A long surplice over the plate, front and back, with a gold collar.
  rig.box('spine', [0.46, 0.5, 0.03], [0, 0.2, 0.17], SURPLICE);
  rig.box('hips', [0.4, 0.62, 0.03], [0, -0.3, 0.14], SURPLICE);
  rig.box('hips', [0.4, 0.62, 0.03], [0, -0.3, -0.14], SURPLICE);
  rig.box('spine', [0.34, 0.06, 0.34], [0, 0.66, 0], GOLD);
  // A helm with a high crest and a visor slit.
  rig.box('head', [0.3, 0.36, 0.32], [0, 0.14, 0], PLATE);
  rig.box('head', [0.24, 0.035, 0.04], [0, 0.17, 0.165], 0x0c0d10);
  rig.box('head', [0.04, 0.24, 0.3], [0, 0.42, 0], GOLD);
  for (const side of ['L', 'R'] as const) {
    const s = side === 'L' ? 1 : -1;
    rig.sphere(`shoulder${side}`, 0.16, [s * 0.02, 0.02, 0], PLATE, { squash: [1.15, 0.85, 1] });
    rig.cyl(`shoulder${side}`, 0.065, 0.06, 0.4, [0, -0.2, 0], DARK);
    rig.cyl(`elbow${side}`, 0.06, 0.07, 0.4, [0, -0.2, 0], PLATE);
    rig.box(`hand${side}`, [0.1, 0.1, 0.1], [0, 0, 0], DARK);
    rig.cyl(`hip${side}`, 0.095, 0.075, 0.5, [0, -0.25, 0], PLATE);
    rig.cyl(`knee${side}`, 0.075, 0.07, 0.5, [0, -0.25, 0], DARK);
    rig.box(`foot${side}`, [0.14, 0.1, 0.3], [0, 0.05, 0.07], PLATE);
  }
  // Halberd: a long pole, an axe blade to one side and a spike on top. Held at the grip, it
  // angles forward and up.
  rig.bone('halberd', 'handR', [0, 0, 0.02]);
  rig.restRot('halberd', [1.15, 0, 0]);
  rig.cyl('halberd', 0.025, 0.025, 2.6, [0, 0.2, 0], 0x5a4128);
  rig.box('halberd', [0.34, 0.3, 0.025], [0.18, 1.35, 0], 0xcfd4dc);
  rig.cone('halberd', 0.04, 0.3, [0, 1.62, 0], 0xcfd4dc);
  // The hand bell: swings below the left hand and glows when it rings.
  rig.bone('bell', 'handL', [0, -0.05, 0.02]);
  rig.cyl('bell', 0.015, 0.015, 0.1, [0, 0.02, 0], 0x5a4128);
  rig.cyl('bell', 0.05, 0.12, 0.2, [0, -0.1, 0], GOLD, { glow: 0.3 });
  rig.sphere('bell', 0.03, [0, -0.22, 0], DARK);
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.3) * 0.006, 0],
  spine: [Math.sin(t * 1.3) * 0.012, -0.1, 0],
  head: [0, 0.1 + Math.sin(t * 0.5) * 0.05, 0],
  shoulderR: [-0.35, 0, -0.18],
  elbowR: [-0.7],
  shoulderL: [-0.2, 0, 0.2],
  elbowL: [-0.5],
  bell: [Math.sin(t * 1.5) * 0.08]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.25);
  delete g.shoulderR;
  return { ...g, shoulderR: [-0.35, 0, -0.18] };
};

const halberd1: Choreo = {
  total: 60,
  keys: (total) =>
    attack(20, 24, total, {
      wind: { shoulderR: [-1.4, 0, -1.6], elbowR: [-0.6], shoulderL: [-1.0, 0, -0.5], elbowL: [-0.8], spine: [0, -0.9, 0], head: [0, 0.4, 0] },
      strike: { halberd: [1.4], shoulderR: [-1.5, 0, 0.7], elbowR: [-0.1], shoulderL: [-1.2, 0, 0.4], elbowL: [-0.4], spine: [0.1, 0.8, 0], hips: [0, 0.3, 0, 0, 0, 0.1] }
    })
};

const halberd2: Choreo = {
  total: 68,
  keys: (total) =>
    attack(24, 28, total, {
      wind: { shoulderR: [-1.5, 0, 0.9], elbowR: [-0.3], shoulderL: [-1.2, 0, 0.5], elbowL: [-0.6], spine: [0, 0.8, 0] },
      strike: { halberd: [1.4], shoulderR: [-1.4, 0, -1.3], elbowR: [-0.1], shoulderL: [-1.0, 0, -0.5], elbowL: [-0.4], spine: [0.1, -0.8, 0], hips: [0, -0.3, 0, 0, 0, 0.1] }
    })
};

const thrust: Choreo = {
  total: 56,
  keys: (total) =>
    attack(22, 25, total, {
      wind: { shoulderR: [-0.9, 0.2, -0.4], elbowR: [-1.9], shoulderL: [-1.2, 0, 0.1], elbowL: [-1.0], spine: [0, -0.5, 0] },
      strike: { halberd: [1.7], shoulderR: [-1.55, 0, -0.05], elbowR: [-0.05], shoulderL: [-1.5, 0, 0.2], elbowL: [-0.1], spine: [0.15, 0.4, 0], hips: [0, 0, 0, 0, -0.14, 0.4], hipL: [-0.6], hipR: [0.5] }
    })
};

/** The bell: lifted slowly over 36 frames, then shaken hard through the ring. */
const bellKeys = (total: number): Keyframes => {
  const lift = { shoulderL: [-3.0, 0, 0.35], elbowL: [-0.4], spine: [-0.15, 0, 0], head: [-0.3, 0, 0], shoulderR: [-0.5, 0, -0.35], elbowR: [-0.5] };
  const keys: Keyframes = [
    [0, {}],
    [14, { shoulderL: [-1.6, 0, 0.3], elbowL: [-0.9], spine: [-0.05, 0, 0] }],
    [34, lift],
    [38, lift]
  ];
  // Shake: the wrist whips the bell back and forth through the active frames.
  [40, 42].forEach((f, i) => keys.push([f, { ...lift, bell: [i % 2 ? -0.9 : 0.9], head: [-0.15, 0, 0], spine: [0.0, 0, 0] }]));
  keys.push([56, { shoulderL: [-2.2, 0, 0.3], elbowL: [-0.5], bell: [0.2] }]);
  keys.push([Math.max(total, 60), {}]);
  return keys;
};

const bell: Choreo = {
  total: 90,
  keys: bellKeys,
  // The bell and its carrier brighten through the lift and flash as it rings (frame 38).
  glow: (f) => (f < 38 ? 1 + (f / 38) * 1.8 : f < 44 ? 3.6 : Math.max(1, 3.6 - (f - 44) * 0.2))
};

export const choirKnightDef: ModelDef = {
  id: 'choir_knight',
  height: 2.2,
  build,
  idle,
  walk,
  stride: 1.7,
  moves: { ck_halberd_1: halberd1, ck_halberd_2: halberd2, ck_thrust: thrust, ck_bell: bell },
  fallback: halberd1,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.3 * u], head: [0.5 * u], kneeL: [0.6 * u], kneeR: [0.9 * u] })
};
