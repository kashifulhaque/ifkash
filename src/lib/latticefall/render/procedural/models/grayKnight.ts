// The Gray Knight (7020): a shield-bearer in dull steel. Guards high, so its shield arm is
// raised in the idle pose. Moves: gk_shield_advance, gk_slash_1, gk_slash_2 (overhead),
// gk_thrust, gk_shield_bash. Strike frames come from game/data/moves/enemies/gray_knight.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const STEEL = 0x8b919b;
const DARK = 0x464c57;
const TABARD = 0x3b4558;
const LEATHER = 0x4a3b2c;

// The shield arm's guard pose, which the shield bone's rest rotation cancels so the shield
// faces forward while the knight stands guarded.
const GUARD_SHOULDER = -0.35;
const GUARD_ELBOW = -1.55;

function build(): Rig {
  const rig = biped({ leg: 0.95, torso: 0.7, neck: 0.08, shoulder: 0.3, hip: 0.13, upperArm: 0.4, foreArm: 0.4 });
  rig.box('hips', [0.38, 0.2, 0.26], [0, 0, 0], DARK);
  rig.box('spine', [0.5, 0.66, 0.3], [0, 0.34, 0], STEEL);
  rig.box('spine', [0.44, 0.34, 0.04], [0, 0.18, 0.17], TABARD);
  rig.box('spine', [0.1, 0.5, 0.02], [0, 0.36, 0.16], DARK);
  rig.box('hips', [0.3, 0.42, 0.04], [0, -0.22, 0.14], TABARD);
  // Helm: a closed bucket with a visor slit and a crest.
  rig.box('head', [0.3, 0.36, 0.32], [0, 0.14, 0], STEEL);
  rig.box('head', [0.24, 0.035, 0.04], [0, 0.17, 0.165], 0x0c0d10);
  rig.box('head', [0.03, 0.2, 0.04], [0, 0.1, 0.165], 0x0c0d10);
  rig.box('head', [0.05, 0.1, 0.34], [0, 0.35, 0], DARK);
  for (const side of ['L', 'R'] as const) {
    const s = side === 'L' ? 1 : -1;
    rig.sphere(`shoulder${side}`, 0.15, [s * 0.02, 0.02, 0], STEEL, { squash: [1.15, 0.85, 1] });
    rig.cyl(`shoulder${side}`, 0.065, 0.06, 0.4, [0, -0.2, 0], DARK);
    rig.cyl(`elbow${side}`, 0.06, 0.07, 0.4, [0, -0.2, 0], STEEL);
    rig.box(`hand${side}`, [0.1, 0.1, 0.1], [0, 0.0, 0], LEATHER);
    rig.cyl(`hip${side}`, 0.095, 0.075, 0.48, [0, -0.24, 0], STEEL);
    rig.cyl(`knee${side}`, 0.075, 0.07, 0.48, [0, -0.24, 0], DARK);
    rig.box(`foot${side}`, [0.14, 0.1, 0.3], [0, 0.05, 0.07], STEEL);
  }
  // Shield on the left forearm.
  rig.bone('shield', 'elbowL', [0.0, -0.22, 0.0]);
  rig.restRot('shield', [-(GUARD_SHOULDER + GUARD_ELBOW), 0, 0]);
  rig.box('shield', [0.56, 0.78, 0.06], [0, 0.0, 0.14], STEEL);
  rig.box('shield', [0.5, 0.72, 0.03], [0, 0.0, 0.18], DARK);
  rig.box('shield', [0.1, 0.14, 0.06], [0, 0.0, 0.2], STEEL);
  // Sword in the right hand, angled forward.
  rig.bone('weapon', 'handR', [0, -0.02, 0.02]);
  rig.restRot('weapon', [-0.55, 0, 0]);
  rig.box('weapon', [0.05, 0.2, 0.05], [0, 0.0, 0], LEATHER);
  rig.box('weapon', [0.3, 0.05, 0.07], [0, -0.12, 0], DARK);
  rig.box('weapon', [0.08, 0.95, 0.025], [0, -0.62, 0], 0xc4c9d1);
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.4) * 0.006, 0],
  spine: [Math.sin(t * 1.4) * 0.015, -0.25, 0],
  head: [0, 0.25 + Math.sin(t * 0.5) * 0.05, 0],
  shoulderL: [GUARD_SHOULDER, 0, 0.18],
  elbowL: [GUARD_ELBOW, 0, 0],
  shoulderR: [-0.15, 0, -0.18],
  elbowR: [-0.5, 0, 0],
  hipL: [0, 0, 0.04],
  hipR: [0.1, 0, -0.04]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.3);
  // Keep the shield arm up and the sword arm relaxed.
  delete g.shoulderL;
  return { ...g, shoulderR: [-0.15 - Math.sin(phase) * 0.15 * amp, 0, -0.18] };
};

const SHIELD_FWD = { shoulderL: [-1.55, 0, 0.05], elbowL: [-0.35, 0, 0] };

const advance: Choreo = {
  total: 56,
  keys: (total) =>
    attack(24, 28, total, {
      wind: { ...SHIELD_FWD, spine: [0.05, -0.1, 0], hips: [0, 0, 0, 0, -0.1, -0.05], kneeL: [0.5], kneeR: [0.5] },
      strike: { ...SHIELD_FWD, spine: [0.4, 0.0, 0], hips: [0, 0, 0, 0, -0.12, 0.3], hipL: [-0.5], hipR: [0.4], kneeR: [0.5] }
    })
};

const slash1: Choreo = {
  total: 52,
  keys: (total) =>
    attack(17, 20, total, {
      wind: { shoulderR: [-1.3, 0, -1.4], elbowR: [-0.6], spine: [0, -0.9, 0], head: [0, 0.4, 0] },
      strike: { shoulderR: [-1.5, 0, 0.6], elbowR: [-0.1], spine: [0.1, 0.6, 0], hips: [0, 0.3, 0, 0, 0, 0.1] }
    })
};

const slash2: Choreo = {
  total: 60,
  keys: (total) =>
    attack(18, 22, total, {
      wind: { shoulderR: [-2.9, 0, -0.35], elbowR: [-0.5], spine: [-0.3, 0.1, 0], head: [-0.2, 0, 0] },
      strike: { shoulderR: [-1.0, 0, -0.1], elbowR: [-0.1], spine: [0.5, 0, 0], hips: [0, 0, 0, 0, -0.08, 0.2] }
    })
};

const thrust: Choreo = {
  total: 54,
  keys: (total) =>
    attack(18, 21, total, {
      wind: { shoulderR: [-0.9, 0.3, -0.5], elbowR: [-2.1], spine: [0, -0.6, 0] },
      strike: { shoulderR: [-1.55, 0, -0.05], elbowR: [-0.05], spine: [0.1, 0.4, 0], hips: [0, 0, 0, 0, -0.14, 0.4], hipL: [-0.6], hipR: [0.5] }
    })
};

const bash: Choreo = {
  total: 50,
  keys: (total) =>
    attack(15, 18, total, {
      wind: { shoulderL: [-0.9, 0, 0.6], elbowL: [-1.6], spine: [-0.1, 0.5, 0] },
      strike: { ...SHIELD_FWD, shoulderL: [-1.7, 0, 0.0], spine: [0.3, -0.3, 0], hips: [0, 0, 0, 0, -0.06, 0.25] }
    })
};

export const grayKnightDef: ModelDef = {
  id: 'gray_knight',
  height: 2.0,
  build,
  idle,
  walk,
  stride: 1.6,
  moves: { gk_shield_advance: advance, gk_slash_1: slash1, gk_slash_2: slash2, gk_thrust: thrust, gk_shield_bash: bash },
  fallback: slash1,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.3 * u], head: [0.5 * u], kneeL: [0.6 * u], kneeR: [0.9 * u] })
};
