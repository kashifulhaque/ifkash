// The Mimic Chest (7082): a brass-bound wooden chest on stubby legs, with a lid that opens on
// rows of teeth and a long tongue. Its latch is on the wrong side (off-center to the right), the
// hint the roster mentions. Moves: mc_chomp (lunges open, snaps shut), mc_slam (rears up and
// slams the lid down) and mc_hop (hops on its hinges and lands). Strike frames come from
// game/data/moves/enemies/mimic_chest.ron.
//
// The lid pivots at the back edge: a negative x rotation lifts its front.

import type { ModelDef } from '../model';
import { attack, type Choreo, type Keyframes } from '../pose';
import { Rig, type Pose } from '../rig';

const WOOD = 0x6b4a2c;
const DARK_WOOD = 0x3f2b19;
const BRASS = 0xc59a3a;
const TOOTH = 0xefe6cf;
const TONGUE = 0xa8323a;
const MAW = 0x2a0d12;

function build(): Rig {
  const rig = new Rig();
  rig.bone('body', null, [0, 0.3, 0]);
  rig.bone('lid', 'body', [0, 0.3, -0.34]);
  rig.bone('tongue', 'body', [0, 0.27, 0.05]);
  for (const [sx, sz, name] of [
    [1, 1, 'FL'],
    [-1, 1, 'FR'],
    [1, -1, 'BL'],
    [-1, -1, 'BR']
  ] as const) {
    rig.bone(`leg${name}`, 'body', [sx * 0.38, -0.22, sz * 0.22]);
  }
  // The chest body, its inside and brass bands.
  rig.box('body', [1.0, 0.56, 0.68], [0, 0, 0], WOOD);
  rig.box('body', [0.9, 0.04, 0.58], [0, 0.29, 0], MAW);
  for (const x of [-0.36, 0.36]) rig.box('body', [0.07, 0.58, 0.7], [x, 0, 0], BRASS);
  // A lower row of teeth along the front lip, set back so they show when the lid is open.
  for (let i = 0; i < 7; i++) rig.cone('body', 0.04, 0.14, [-0.36 + i * 0.12, 0.34, 0.3], TOOTH);
  // The lid: a barrel-topped slab with brass bands and a row of upper teeth.
  rig.box('lid', [1.0, 0.1, 0.68], [0, 0.05, 0.34], WOOD);
  rig.cyl('lid', 0.18, 0.18, 1.0, [0, 0.1, 0.34], WOOD, { rot: [0, 0, Math.PI / 2] });
  for (const x of [-0.36, 0.36]) rig.box('lid', [0.07, 0.3, 0.7], [x, 0.14, 0.34], BRASS);
  for (let i = 0; i < 7; i++) rig.cone('lid', 0.04, 0.14, [-0.36 + i * 0.12, -0.02, 0.64], TOOTH, { rot: [Math.PI, 0, 0] });
  // The latch, on the wrong side.
  rig.box('lid', [0.1, 0.14, 0.04], [0.3, 0.02, 0.69], BRASS);
  rig.sphere('lid', 0.035, [0.3, -0.02, 0.72], DARK_WOOD);
  // Tongue and eyes inside.
  rig.box('tongue', [0.2, 0.04, 0.5], [0, 0, 0.2], TONGUE);
  for (const s of [1, -1]) rig.sphere('lid', 0.045, [s * 0.2, 0.04, 0.5], 0xffd23a, { glow: 1.2 });
  for (const name of ['FL', 'FR', 'BL', 'BR']) {
    rig.box(`leg${name}`, [0.14, 0.2, 0.14], [0, -0.08, 0], DARK_WOOD);
    rig.box(`leg${name}`, [0.2, 0.05, 0.26], [0, -0.2, 0.04], BRASS);
  }
  return rig;
}

const idle = (t: number): Pose => ({
  body: [0, 0, 0, 0, Math.sin(t * 1.4) * 0.006, 0],
  // Not quite shut: the lid breathes open a crack and the tongue lolls.
  lid: [-0.05 - Math.max(0, Math.sin(t * 0.9)) * 0.12],
  tongue: [Math.sin(t * 1.6) * 0.05]
});

/** Waddles: the body rocks side to side and the legs shuffle in diagonal pairs. */
const walk = (phase: number, amp: number): Pose => {
  const s = Math.sin(phase);
  return {
    body: [0, 0, s * 0.06 * amp, 0, Math.abs(Math.cos(phase)) * 0.04 * amp, 0],
    legFL: [s * 0.5 * amp],
    legBR: [s * 0.5 * amp],
    legFR: [-s * 0.5 * amp],
    legBL: [-s * 0.5 * amp],
    lid: [-0.1 - Math.abs(s) * 0.1 * amp]
  };
};

const chomp: Choreo = {
  total: 62,
  keys: (total) =>
    attack(20, 24, total, {
      // Rears back with the lid flung wide, tongue out.
      wind: { body: [-0.2, 0, 0, 0, 0.05, -0.08], lid: [-1.25], tongue: [0.2], legFL: [-0.2], legFR: [-0.2] },
      // Lunges forward and the lid slams shut on the target.
      strike: { body: [0.12, 0, 0, 0, -0.03, 0.3], lid: [0.05], tongue: [-0.1], legBL: [0.3], legBR: [0.3] }
    })
};

const slam: Choreo = {
  total: 74,
  keys: (total) =>
    attack(26, 30, total, {
      // Reared up on its back legs with the lid held high over its head.
      wind: { body: [-0.7, 0, 0, 0, 0.18, -0.12], lid: [-1.7], legFL: [0.5], legFR: [0.5], legBL: [-0.2], legBR: [-0.2] },
      strike: { body: [0.25, 0, 0, 0, -0.06, 0.28], lid: [0.1], legFL: [-0.4], legFR: [-0.4] }
    })
};

/** Crouches low, springs into the air, comes down on the target. */
const hopKeys = (total: number): Keyframes => [
  [0, {}],
  [10, { body: [0.1, 0, 0, 0, -0.1, 0], lid: [-0.5], legFL: [0.4], legFR: [0.4], legBL: [-0.4], legBR: [-0.4] }],
  [18, { body: [0.15, 0, 0, 0, -0.16, -0.05], lid: [-0.9], legFL: [0.6], legFR: [0.6], legBL: [-0.5], legBR: [-0.5] }],
  [22, { body: [-0.35, 0, 0, 0, 0.7, 0.4], lid: [-1.3], tongue: [0.3], legFL: [-0.7], legFR: [-0.7], legBL: [0.7], legBR: [0.7] }],
  [25, { body: [-0.1, 0, 0, 0, 0.75, 1.2], lid: [-1.2], legFL: [-0.5], legFR: [-0.5] }],
  [28, { body: [0.15, 0, 0, 0, -0.04, 1.4], lid: [0.05], legFL: [0.2], legFR: [0.2] }],
  [Math.max(total * 0.75, 40), { body: [0.05, 0, 0, 0, -0.06, 1.4], lid: [-0.1] }],
  [Math.max(total, 44), {}]
];

const hop: Choreo = { total: 70, keys: hopKeys };

export const mimicChestDef: ModelDef = {
  id: 'mimic_chest',
  height: 0.9,
  build,
  idle,
  walk,
  stride: 0.9,
  moves: { mc_chomp: chomp, mc_slam: slam, mc_hop: hop },
  fallback: chomp,
  fallAxis: 'x',
  death: (u) => ({ lid: [-1.0 * u], tongue: [0.3 * u], legFL: [0.5 * u], legFR: [0.5 * u], legBL: [-0.5 * u], legBR: [-0.5 * u] })
};
