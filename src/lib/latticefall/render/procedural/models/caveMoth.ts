// The Cave Moth (7054): a dog-sized pale moth that hovers on beating wings. The sim keeps its
// feet 0.9 m up, so the body hangs just above the model's origin with its legs dangling.
// Moves: cm_dive (nose-down strike) and cm_dust (a burst of wing beats that shakes Drowse dust
// down). Strike frames come from game/data/moves/enemies/cave_moth.ron.

import type { ModelDef } from '../model';
import { attack, type Choreo, type Keyframes } from '../pose';
import { Rig, type Pose } from '../rig';

const FUR = 0xcfc4a8;
const DARK = 0x6b5f4a;
const WING = 0xe5dcc3;
const WING_MARK = 0x8a6f9a;
const EYE = 0xf0a0ff;

function build(): Rig {
  const rig = new Rig();
  rig.bone('body', null, [0, 0.5, 0]);
  rig.bone('head', 'body', [0, 0.03, 0.3]);
  rig.bone('abdomen', 'body', [0, -0.02, -0.25]);
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.bone(`wing${side}`, 'body', [s * 0.1, 0.12, 0.0]);
    rig.bone(`antenna${side}`, 'head', [s * 0.05, 0.1, 0.08]);
    rig.bone(`leg${side}`, 'body', [s * 0.07, -0.1, 0.08]);
    rig.bone(`shin${side}`, `leg${side}`, [0, -0.17, 0]);
  }
  rig.sphere('body', 0.17, [0, 0, 0], FUR, { squash: [0.9, 0.9, 1.35] });
  rig.sphere('abdomen', 0.14, [0, 0, -0.1], FUR, { squash: [0.85, 0.85, 1.6] });
  rig.box('abdomen', [0.2, 0.03, 0.05], [0, 0.0, -0.18], DARK);
  rig.box('abdomen', [0.18, 0.03, 0.05], [0, 0.0, -0.08], DARK);
  rig.sphere('head', 0.11, [0, 0, 0.03], FUR);
  for (const s of [1, -1]) rig.sphere('head', 0.045, [s * 0.075, 0.02, 0.07], EYE, { glow: 0.9 });
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    // Broad, thin wings with an eyespot, and feathery antennae.
    rig.box(`wing${side}`, [0.8, 0.015, 0.5], [s * 0.4, 0, -0.05], WING, { alpha: 0.92 });
    rig.box(`wing${side}`, [0.3, 0.02, 0.22], [s * 0.55, 0, -0.1], WING_MARK, { alpha: 0.92 });
    rig.cone(`antenna${side}`, 0.012, 0.22, [s * 0.04, 0.1, 0.02], DARK, { rot: [0.5, 0, -s * 0.5] });
    rig.cyl(`leg${side}`, 0.012, 0.01, 0.18, [0, -0.085, 0], DARK);
    rig.cyl(`shin${side}`, 0.01, 0.006, 0.2, [0, -0.1, 0], DARK);
  }
  return rig;
}

const idle = (t: number): Pose => {
  const flap = Math.sin(t * 26);
  return {
    body: [0.06 * Math.sin(t * 1.6), 0, 0, 0, Math.sin(t * 2.2) * 0.03, 0],
    wingL: [0, 0, 0.1 + flap * 0.65],
    wingR: [0, 0, -0.1 - flap * 0.65],
    head: [Math.sin(t * 1.1) * 0.1, Math.sin(t * 0.7) * 0.2, 0],
    abdomen: [Math.sin(t * 1.6 + 1) * 0.12],
    legL: [0.3 + Math.sin(t * 2.1) * 0.1],
    legR: [0.3 + Math.sin(t * 2.1 + 1) * 0.1],
    shinL: [0.5],
    shinR: [0.5]
  };
};

/** Flying forward: the body tips into it and the wings beat harder. */
const walk = (phase: number, amp: number): Pose => ({
  body: [0.2 * amp],
  wingL: [0, 0, Math.sin(phase * 5) * 0.3 * amp],
  wingR: [0, 0, -Math.sin(phase * 5) * 0.3 * amp],
  legL: [0.5 * amp],
  legR: [0.5 * amp]
});

const dive: Choreo = {
  total: 56,
  keys: (total) =>
    attack(16, 22, total, {
      // Rears back, wings up and cocked, legs drawn in.
      wind: {
        body: [-0.5, 0, 0, 0, 0.1, -0.1],
        wingL: [0, 0, 1.1],
        wingR: [0, 0, -1.1],
        head: [0.3],
        abdomen: [0.4],
        legL: [0.8],
        legR: [0.8],
        shinL: [0.8],
        shinR: [0.8]
      },
      // Nose down, wings swept back, legs reaching forward.
      strike: {
        body: [0.9, 0, 0, 0, -0.05, 0.2],
        wingL: [0, 0, 0.35],
        wingR: [0, 0, -0.35],
        head: [-0.2],
        abdomen: [-0.5],
        legL: [-1.4],
        legR: [-1.4],
        shinL: [-0.5],
        shinR: [-0.5]
      },
      follow: { body: [0.3], wingL: [0, 0, 0.8], wingR: [0, 0, -0.8], legL: [-0.5], legR: [-0.5] }
    })
};

/** Fast alternating beats until the burst, then the wings snap wide. */
const dustKeys = (total: number): Keyframes => {
  const keys: Keyframes = [[0, {}]];
  for (let f = 3, up = true; f < 20; f += 3, up = !up) {
    const a = up ? 1.0 : -0.4;
    keys.push([f, { wingL: [0, 0, a], wingR: [0, 0, -a], body: [-0.1, 0, 0, 0, up ? 0.08 : -0.02, 0] }]);
  }
  keys.push([20, { wingL: [0, 0, 1.3], wingR: [0, 0, -1.3], body: [-0.25, 0, 0, 0, 0.12, 0], abdomen: [0.5], head: [0.2] }]);
  keys.push([24, { wingL: [0, 0, -0.5], wingR: [0, 0, 0.5], body: [0.1, 0, 0, 0, -0.04, 0], abdomen: [-0.3] }]);
  keys.push([Math.max(total, 30), {}]);
  return keys;
};

const dust: Choreo = { total: 64, keys: dustKeys };

export const caveMothDef: ModelDef = {
  id: 'cave_moth',
  height: 0.8,
  build,
  idle,
  walk,
  stride: 2.0,
  moves: { cm_dive: dive, cm_dust: dust },
  fallback: dive,
  fallAxis: 'z',
  death: (u) => ({
    wingL: [0, 0, -0.3 * u],
    wingR: [0, 0, 0.3 * u],
    legL: [1.2 * u],
    legR: [1.2 * u],
    head: [0.4 * u]
  })
};
