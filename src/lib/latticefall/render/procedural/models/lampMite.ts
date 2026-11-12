// The Lamp Mite (7055): a fist-sized swarm bug carrying a lit lamp bulb on its back, which it
// swells and flares before it bursts. Move: lm_burst (12 frames of flare, then the Fire burst).
// Strike frames come from game/data/moves/enemies/lamp_mite.ron.

import type { ModelDef } from '../model';
import type { Choreo } from '../pose';
import { Rig, type Pose } from '../rig';

const SHELL = 0x5a4b3a;
const DARK = 0x2f261c;
const BULB = 0xffa23a;

const LEG_Z = [0.08, 0.0, -0.08] as const;

function build(): Rig {
  const rig = new Rig();
  rig.bone('body', null, [0, 0.14, 0]);
  rig.bone('head', 'body', [0, 0.0, 0.12]);
  rig.bone('bulb', 'body', [0, 0.1, -0.06]);
  LEG_Z.forEach((z, i) => {
    for (const [s, side] of [
      [1, 'L'],
      [-1, 'R']
    ] as const) {
      rig.bone(`leg${i}${side}`, 'body', [s * 0.07, -0.04, z]);
      rig.restRot(`leg${i}${side}`, [0, 0, s * 0.8]);
      rig.cyl(`leg${i}${side}`, 0.012, 0.008, 0.11, [0, -0.05, 0], DARK);
    }
  });
  rig.sphere('body', 0.09, [0, 0, 0], SHELL, { squash: [1, 0.8, 1.3] });
  rig.sphere('head', 0.055, [0, 0, 0.02], SHELL);
  for (const s of [1, -1]) rig.sphere('head', 0.014, [s * 0.03, 0.02, 0.05], 0xffe9b0, { glow: 1.2 });
  rig.cyl('bulb', 0.03, 0.04, 0.05, [0, 0, 0], DARK);
  rig.sphere('bulb', 0.075, [0, 0.07, 0], BULB, { glow: 1.0 });
  return rig;
}

const idle = (t: number): Pose => ({
  body: [0, 0, 0, 0, Math.sin(t * 3.0) * 0.006, 0],
  head: [Math.sin(t * 1.7) * 0.15, Math.sin(t * 1.1) * 0.4, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const out: Pose = { body: [0, 0, 0, 0, Math.abs(Math.cos(phase * 2)) * 0.01 * amp, 0] };
  for (let i = 0; i < 3; i++) {
    const a = i === 1 ? phase + Math.PI : phase;
    out[`leg${i}L`] = [Math.sin(a) * 0.5 * amp, 0, 0];
    out[`leg${i}R`] = [Math.sin(a + Math.PI) * 0.5 * amp, 0, 0];
  }
  return out;
};

// The burst: crouch and tremble through the 12-frame flare, then spring open on the blast.
const burst: Choreo = {
  total: 36,
  keys: () => [
    [0, {}],
    [4, { body: [0, 0, 0, 0, -0.03, 0], head: [-0.2] }],
    [12, { body: [0, 0, 0, 0, -0.05, 0], head: [-0.4], leg0L: [0, 0, 0.3], leg0R: [0, 0, -0.3], leg2L: [0, 0, 0.3], leg2R: [0, 0, -0.3] }],
    [16, { body: [-0.3, 0, 0, 0, 0.12, 0], head: [0.3], leg0L: [0, 0, -0.4], leg0R: [0, 0, 0.4], leg2L: [0, 0, -0.4], leg2R: [0, 0, 0.4] }],
    [36, { body: [-0.1, 0, 0, 0, 0.04, 0] }]
  ],
  // The bulb flares white-hot through the telegraph and stays bright through the blast.
  glow: (f) => (f < 12 ? 1 + (f / 12) * 3.5 : f < 16 ? 5 : Math.max(1, 5 - (f - 16) * 0.3))
};

export const lampMiteDef: ModelDef = {
  id: 'lamp_mite',
  height: 0.3,
  build,
  idle,
  walk,
  stride: 0.3,
  moves: { lm_burst: burst },
  fallback: burst,
  fallAxis: 'z',
  death: (u) => ({ body: [0, 0, 0, 0, -0.06 * u, 0], head: [0.3 * u] }),
  // The bulb swells over the flare and bursts past full size at the blast.
  tick: (rig, ctx) => {
    const bulb = rig.bones.get('bulb')!;
    const k = !ctx.inMove ? 1 : ctx.frame < 12 ? 1 + (ctx.frame / 12) * 0.9 : ctx.frame < 16 ? 2.6 : Math.max(1, 2.6 - (ctx.frame - 16) * 0.12);
    bulb.scale.setScalar(k);
  }
};
