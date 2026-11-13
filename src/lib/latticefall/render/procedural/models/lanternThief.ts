// The Lantern Thief (7080): a small, hunched, hooded ambusher in a patched cloak, a stolen
// lantern swinging from its belt. Moves: lt_bomb (a 24-frame wind-up, then it lobs a pitch bomb
// on frame 24) and lt_stab (a quick dagger jab). Strike frames come from
// game/data/moves/enemies/lantern_thief.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const CLOAK = 0x3a3340;
const CLOTH = 0x5b4a3a;
const SKIN = 0xb59a82;
const PITCH = 0x15120f;
const LANTERN = 0xffb347;

/** Frame on which the bomb leaves the hand (the move's projectile frame). */
const LOB_FRAME = 24;

function build(): Rig {
  const rig = biped({ leg: 0.68, torso: 0.5, neck: 0.06, shoulder: 0.19, hip: 0.09, upperArm: 0.32, foreArm: 0.34 });
  rig.restRot('spine', [0.3, 0, 0]);
  rig.box('hips', [0.26, 0.16, 0.18], [0, 0, 0], CLOTH);
  rig.box('spine', [0.34, 0.5, 0.2], [0, 0.27, 0], CLOTH);
  // A hooded cloak draping over the shoulders and back.
  rig.cone('spine', 0.28, 0.75, [0, 0.12, -0.08], CLOAK, { rot: [Math.PI, 0, 0] });
  rig.sphere('head', 0.17, [0, 0.1, 0.0], CLOAK, { squash: [0.95, 1.05, 1.1] });
  rig.box('head', [0.16, 0.14, 0.06], [0, 0.07, 0.13], SKIN);
  for (const s of [1, -1]) rig.sphere('head', 0.018, [s * 0.04, 0.09, 0.165], 0xffd77a, { glow: 1.3 });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.05, 0.055, 0.32, [0, -0.16, 0], CLOAK);
    rig.cyl(`elbow${side}`, 0.045, 0.05, 0.34, [0, -0.17, 0], CLOTH);
    rig.sphere(`hand${side}`, 0.045, [0, 0, 0], SKIN);
    rig.cyl(`hip${side}`, 0.06, 0.05, 0.34, [0, -0.17, 0], CLOTH);
    rig.cyl(`knee${side}`, 0.05, 0.04, 0.34, [0, -0.17, 0], CLOAK);
    rig.box(`foot${side}`, [0.09, 0.06, 0.2], [0, 0.03, 0.05], 0x2a2620);
  }
  // The lantern on the belt.
  rig.bone('lantern', 'hips', [0.22, -0.05, 0.04]);
  rig.cyl('lantern', 0.012, 0.012, 0.12, [0, 0.0, 0], 0x2a2620);
  rig.box('lantern', [0.1, 0.14, 0.1], [0, -0.14, 0], LANTERN, { glow: 1.4 });
  rig.cyl('lantern', 0.07, 0.07, 0.025, [0, -0.07, 0], 0x2a2620);
  // A dagger in the right hand and a pitch bomb in the left.
  rig.bone('dagger', 'handR', [0, -0.02, 0.02]);
  rig.restRot('dagger', [-0.9, 0, 0]);
  rig.box('dagger', [0.03, 0.4, 0.015], [0, -0.22, 0], 0xb9bfc7);
  rig.box('dagger', [0.1, 0.03, 0.03], [0, -0.03, 0], 0x2a2620);
  rig.bone('bomb', 'handL', [0, -0.04, 0.03]);
  rig.sphere('bomb', 0.11, [0, -0.05, 0], PITCH);
  rig.cone('bomb', 0.018, 0.1, [0, 0.07, 0], 0xd9cfa8, { rot: [0, 0, 0.3] });
  rig.sphere('bomb', 0.028, [0.02, 0.13, 0], 0xff7a2a, { glow: 1.6 });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.7) * 0.007, 0],
  spine: [Math.sin(t * 1.7) * 0.03, Math.sin(t * 0.6) * 0.06, 0],
  head: [0.1, Math.sin(t * 0.9) * 0.4, 0],
  lantern: [Math.sin(t * 1.9) * 0.12, 0, Math.sin(t * 1.4) * 0.1],
  shoulderL: [-0.6, 0, 0.15],
  elbowL: [-1.0],
  shoulderR: [-0.4, 0, -0.15],
  elbowR: [-0.7],
  hipL: [-0.15],
  hipR: [-0.1],
  kneeL: [0.3],
  kneeR: [0.25]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.2);
  delete g.shoulderL;
  delete g.shoulderR;
  return { ...g, lantern: [Math.sin(phase * 2) * 0.3 * amp, 0, 0] };
};

const bomb: Choreo = {
  total: 60,
  // A wind-up over 24 frames: the arm draws all the way back, then lobs underhand-over.
  keys: (total) =>
    attack(LOB_FRAME, LOB_FRAME + 2, total, {
      wind: { shoulderL: [-0.8, 0, 1.4], elbowL: [-1.4], spine: [-0.2, 0.6, 0], head: [-0.1, -0.3, 0], hipL: [-0.3], kneeL: [0.7] },
      strike: { shoulderL: [-2.2, 0, 0.3], elbowL: [-0.1], spine: [0.4, -0.5, 0], hips: [0, -0.2, 0, 0, 0, 0.08], head: [0.15, 0.2, 0] }
    }),
  glow: (f) => (f < LOB_FRAME ? 1 + (f / LOB_FRAME) * 0.8 : 1)
};

const stab: Choreo = {
  total: 36,
  keys: (total) =>
    attack(14, 16, total, {
      wind: { shoulderR: [-0.5, 0, -0.9], elbowR: [-2.0], spine: [0, -0.5, 0] },
      strike: { shoulderR: [-1.5, 0, -0.1], elbowR: [-0.1], spine: [0.3, 0.4, 0], hips: [0, 0, 0, 0, -0.06, 0.3], hipL: [-0.5], hipR: [0.4] }
    })
};

export const lanternThiefDef: ModelDef = {
  id: 'lantern_thief',
  height: 1.4,
  build,
  idle,
  walk,
  stride: 1.2,
  moves: { lt_bomb: bomb, lt_stab: stab },
  fallback: stab,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.4 * u], head: [0.4 * u], kneeL: [0.8 * u], kneeR: [0.8 * u] }),
  // The bomb is in hand until it leaves on the lob frame; a new one is ready after the move.
  animatedBones: ['bomb'],
  tick: (rig, ctx) => {
    rig.bones.get('bomb')!.visible = !(ctx.inMove && ctx.anim === 'lt_bomb' && ctx.frame >= LOB_FRAME);
  }
};
