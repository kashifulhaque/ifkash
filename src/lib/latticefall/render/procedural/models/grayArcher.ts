// The Gray Archer (7030): a hooded, lightly armored bowman. Moves: ga_shoot (draw, loose at
// frame 26) and ga_shove (a close-range shove). Strike frames come from
// game/data/moves/enemies/gray_archer.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const CLOAK = 0x59605a;
const LEATHER = 0x5b4733;
const STEEL = 0x8b919b;
const WOOD = 0x7c5a36;

function build(): Rig {
  const rig = biped({ leg: 0.9, torso: 0.62, neck: 0.07, shoulder: 0.24, hip: 0.11, upperArm: 0.36, foreArm: 0.36 });
  rig.box('hips', [0.32, 0.18, 0.22], [0, 0, 0], LEATHER);
  rig.box('spine', [0.4, 0.6, 0.24], [0, 0.31, 0], CLOAK);
  rig.box('spine', [0.42, 0.08, 0.26], [0, 0.04, 0], LEATHER);
  rig.cone('spine', 0.26, 0.62, [0, 0.12, -0.14], CLOAK, { rot: [-0.18, 0, 0] });
  // Hood, with a shadowed face.
  rig.cone('head', 0.22, 0.34, [0, 0.2, -0.02], CLOAK, { rot: [-0.1, 0, 0] });
  rig.sphere('head', 0.15, [0, 0.12, 0.01], 0x2d2b28);
  rig.box('head', [0.2, 0.06, 0.08], [0, 0.13, 0.12], 0x0d0d0c);
  rig.sphere('head', 0.02, [0.04, 0.13, 0.15], 0xdcd7c3, { glow: 0.6 });
  rig.sphere('head', 0.02, [-0.04, 0.13, 0.15], 0xdcd7c3, { glow: 0.6 });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.055, 0.05, 0.36, [0, -0.18, 0], CLOAK);
    rig.cyl(`elbow${side}`, 0.05, 0.05, 0.36, [0, -0.18, 0], LEATHER);
    rig.box(`hand${side}`, [0.08, 0.08, 0.08], [0, 0, 0], LEATHER);
    rig.cyl(`hip${side}`, 0.075, 0.06, 0.45, [0, -0.225, 0], CLOAK);
    rig.cyl(`knee${side}`, 0.06, 0.055, 0.45, [0, -0.225, 0], LEATHER);
    rig.box(`foot${side}`, [0.11, 0.08, 0.26], [0, 0.04, 0.06], LEATHER);
  }
  // Quiver on the back.
  rig.cyl('spine', 0.07, 0.06, 0.5, [0.1, 0.4, -0.2], LEATHER, { rot: [0.2, 0, 0.2] });
  for (const dx of [-0.02, 0.02, 0.0]) rig.cyl('spine', 0.008, 0.008, 0.12, [0.12 + dx, 0.7, -0.26 + dx], 0xd9d0b8);
  // The bow in the left hand: a limb arc bulging forward, and a string.
  rig.bone('bow', 'handL', [0, -0.02, 0]);
  rig.restRot('bow', [0, 0, 0]);
  rig.arc('bow', 0.55, 0.022, 2.2, [0, 0.0, -0.55], WOOD, { rot: [0, -Math.PI / 2, -1.1] });
  rig.box('bow', [0.008, 0.9, 0.008], [0, 0.0, -0.2], 0xe6e1d0);
  // An arrow nocked on the string, hidden once loosed.
  rig.bone('arrow', 'bow', [0, 0, -0.2]);
  rig.cyl('arrow', 0.01, 0.01, 0.8, [0, 0, 0.35], 0xd9d0b8, { rot: [Math.PI / 2, 0, 0] });
  rig.cone('arrow', 0.025, 0.07, [0, 0, 0.78], STEEL, { rot: [Math.PI / 2, 0, 0] });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.3) * 0.006, 0],
  spine: [Math.sin(t * 1.3) * 0.012, 0.3, 0],
  head: [0, -0.2 + Math.sin(t * 0.6) * 0.1, 0],
  shoulderL: [-0.2, 0, 0.12],
  elbowL: [-0.6, 0, 0],
  shoulderR: [-0.1, 0, -0.14],
  elbowR: [-0.35, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.35);
  delete g.shoulderL;
  return g;
};

const shoot: Choreo = {
  total: 70,
  keys: (total) => {
    // Draw from frame 6, hold, loose on frame 26 (the arrow leaves then).
    const draw: Pose = {
      shoulderL: [-1.5, 0, 0.12],
      elbowL: [-0.05],
      shoulderR: [-1.45, 0, -1.0],
      elbowR: [-2.3],
      spine: [0, 0.65, 0],
      head: [0, -0.7, 0]
    };
    const loose: Pose = { ...draw, shoulderR: [-1.3, 0, -1.4], elbowR: [-0.5], spine: [0, 0.6, 0] };
    return [
      [0, {}],
      [8, { shoulderL: [-1.0, 0, 0.12], shoulderR: [-0.9, 0, -0.5], elbowR: [-1.4], spine: [0, 0.4, 0], head: [0, -0.4, 0] }],
      [22, draw],
      [25, draw],
      [27, loose],
      [40, { ...loose, shoulderL: [-1.35, 0, 0.12] }],
      [Math.max(total, 45), {}]
    ];
  }
};

const shove: Choreo = {
  total: 46,
  keys: (total) =>
    attack(14, 16, total, {
      wind: { shoulderR: [-0.8, 0, -0.9], elbowR: [-1.9], spine: [0, -0.4, 0] },
      strike: { shoulderR: [-1.5, 0, -0.1], elbowR: [-0.1], shoulderL: [-0.9, 0, 0.3], spine: [0.2, 0.3, 0], hips: [0, 0, 0, 0, 0, 0.25] }
    })
};

export const grayArcherDef: ModelDef = {
  id: 'gray_archer',
  height: 1.85,
  build,
  idle,
  walk,
  stride: 1.5,
  moves: { ga_shoot: shoot, ga_shove: shove },
  fallback: shoot,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.3 * u], head: [0.4 * u], kneeL: [0.7 * u], kneeR: [0.5 * u] }),
  tick(rig, ctx) {
    const arrow = rig.bones.get('arrow');
    if (arrow) arrow.visible = !ctx.inMove || ctx.frame < 27 || ctx.total < 50;
  }
};
