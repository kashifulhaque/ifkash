// The Reed Stalker (7052): a lean, long-limbed ambusher in a cloak of dried reeds, crouched
// low. Hidden in the reeds it isn't drawn at all (the snapshot's `HIDDEN` flag). Move:
// rs_lunge (16 frames of coil, then a long forward leap). Strike frames come from
// game/data/moves/enemies/reed_stalker.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const SKIN = 0x8a8f6a;
const REED = 0xa89a58;
const DARK_REED = 0x6b6238;
const CLAW = 0xe4dcc0;

function build(): Rig {
  const rig = biped({ leg: 0.85, torso: 0.55, neck: 0.1, shoulder: 0.2, hip: 0.1, upperArm: 0.45, foreArm: 0.5 });
  rig.restRot('spine', [0.45, 0, 0]);
  rig.box('hips', [0.28, 0.16, 0.18], [0, 0, 0], DARK_REED);
  rig.box('spine', [0.32, 0.56, 0.2], [0, 0.3, 0], SKIN);
  // A cloak of reeds: tall blades splayed from the shoulders and the waist.
  for (const [x, z, h, lean] of [
    [0.13, -0.12, 0.8, 0.25],
    [-0.12, -0.12, 0.7, -0.3],
    [0.0, -0.14, 0.9, 0.0],
    [0.2, -0.04, 0.6, 0.5],
    [-0.2, -0.04, 0.62, -0.5]
  ] as const) {
    rig.cone('spine', 0.045, h, [x, 0.3 + h / 2 - 0.05, z], x > 0 ? REED : DARK_REED, { rot: [-0.25, 0, -lean] });
  }
  rig.cone('hips', 0.2, 0.6, [0, -0.25, -0.02], DARK_REED, { rot: [Math.PI, 0, 0] });
  rig.sphere('head', 0.14, [0, 0.1, 0.04], SKIN, { squash: [0.9, 1.15, 1.1] });
  rig.box('head', [0.16, 0.07, 0.08], [0, 0.02, 0.14], DARK_REED);
  for (const s of [1, -1]) rig.sphere('head', 0.022, [s * 0.05, 0.14, 0.15], 0xe9d85a, { glow: 1.3 });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.04, 0.05, 0.45, [0, -0.22, 0], SKIN);
    rig.cyl(`elbow${side}`, 0.035, 0.04, 0.5, [0, -0.25, 0], SKIN);
    for (const dx of [-0.03, 0, 0.03]) rig.cone(`hand${side}`, 0.012, 0.18, [dx, -0.09, 0.01], CLAW, { rot: [Math.PI, 0, 0] });
    rig.box(`hand${side}`, [0.07, 0.07, 0.04], [0, 0, 0], SKIN);
    rig.cyl(`hip${side}`, 0.055, 0.045, 0.43, [0, -0.21, 0], DARK_REED);
    rig.cyl(`knee${side}`, 0.045, 0.04, 0.43, [0, -0.21, 0], SKIN);
    rig.box(`foot${side}`, [0.08, 0.05, 0.22], [0, 0.03, 0.06], SKIN);
  }
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, -0.08 + Math.sin(t * 1.8) * 0.008, 0],
  spine: [0.05 + Math.sin(t * 1.8) * 0.03, Math.sin(t * 0.6) * 0.08, 0],
  head: [-0.3 + Math.sin(t * 1.1) * 0.05, Math.sin(t * 0.7) * 0.25, 0],
  hipL: [-0.35],
  hipR: [-0.3],
  kneeL: [0.7],
  kneeR: [0.6],
  shoulderL: [-0.3, 0, 0.12],
  shoulderR: [-0.3, 0, -0.12],
  elbowL: [-0.5],
  elbowR: [-0.5]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.4);
  return { ...g, hips: [0, 0, 0, 0, -0.1, 0], head: [-0.3, Math.sin(phase) * 0.15 * amp, 0] };
};

const lunge: Choreo = {
  total: 60,
  keys: (total) =>
    attack(16, 20, total, {
      // Coil: sunk low on bent legs, claws drawn back, head up on the target.
      wind: {
        hips: [0.15, 0, 0, 0, -0.3, -0.12],
        spine: [0.2],
        head: [-0.55],
        hipL: [-0.85],
        hipR: [-0.85],
        kneeL: [1.5],
        kneeR: [1.5],
        shoulderL: [0.5, 0, 0.5],
        shoulderR: [0.5, 0, -0.5],
        elbowL: [-0.4],
        elbowR: [-0.4]
      },
      // Airborne and stretched out, claws first.
      strike: {
        hips: [-0.3, 0, 0, 0, 0.1, 0.35],
        spine: [-0.2],
        head: [0.1],
        hipL: [0.7],
        hipR: [0.4],
        kneeL: [0.1],
        kneeR: [0.3],
        shoulderL: [-2.1, 0, 0.2],
        shoulderR: [-2.1, 0, -0.2],
        elbowL: [-0.05],
        elbowR: [-0.05]
      },
      follow: { hips: [0.1, 0, 0, 0, -0.18, 0.15], hipL: [-0.4], hipR: [-0.4], kneeL: [0.9], kneeR: [0.9], shoulderL: [-0.9], shoulderR: [-0.9] }
    })
};

export const reedStalkerDef: ModelDef = {
  id: 'reed_stalker',
  height: 1.4,
  build,
  idle,
  walk,
  stride: 1.5,
  moves: { rs_lunge: lunge },
  fallback: lunge,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.4 * u], head: [0.5 * u], kneeL: [0.7 * u], kneeR: [0.9 * u], shoulderL: [-0.4 * u], shoulderR: [-0.2 * u] })
};
