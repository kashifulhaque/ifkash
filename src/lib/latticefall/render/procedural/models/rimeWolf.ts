// The Rime Wolf (7081): a big, shaggy, frost-grey wolf with ice crusted along its spine and a
// pale glow in its eyes. Same quadruped skeleton and gait as the Thorn Hound. Moves: rw_lunge
// (a leaping bite) and rw_bite (a close snap). Strike frames come from
// game/data/moves/enemies/rime_wolf.ron.

import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import { Rig, type Pose } from '../rig';
import { houndWalk } from './thornHound';

const FUR = 0xaab4bd;
const DARK_FUR = 0x6d7a86;
const ICE = 0xcdeef9;
const BONE = 0xf0f3f5;

function build(): Rig {
  const rig = new Rig();
  rig.bone('body', null, [0, 0.72, 0]);
  rig.bone('chest', 'body', [0, 0.03, 0.32]);
  rig.bone('head', 'chest', [0, 0.12, 0.34]);
  rig.bone('jaw', 'head', [0, -0.03, 0.12]);
  rig.bone('tail', 'body', [0, 0.08, -0.55]);
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.bone(`fleg${side}`, 'chest', [s * 0.13, -0.1, 0.0]);
    rig.bone(`fshin${side}`, `fleg${side}`, [0, -0.32, 0]);
    rig.bone(`hleg${side}`, 'body', [s * 0.13, -0.06, -0.38]);
    rig.bone(`hshin${side}`, `hleg${side}`, [0, -0.34, 0.0]);
  }
  rig.box('body', [0.36, 0.34, 0.8], [0, 0, -0.05], FUR);
  rig.box('chest', [0.42, 0.42, 0.36], [0, 0.01, 0.02], FUR);
  // A heavy dark mane over the shoulders.
  rig.box('chest', [0.3, 0.12, 0.5], [0, 0.26, -0.08], DARK_FUR);
  // Ice crusted along the back.
  for (const [bone, z, h] of [
    ['chest', 0.08, 0.2],
    ['chest', -0.08, 0.22],
    ['body', -0.1, 0.18],
    ['body', -0.26, 0.15],
    ['body', -0.42, 0.12]
  ] as const) {
    rig.cone(bone, 0.04, h, [0, 0.2 + h / 2, z], ICE, { rot: [-0.25, 0, 0], glow: 0.5 });
  }
  rig.box('head', [0.24, 0.24, 0.26], [0, 0, 0.05], FUR);
  rig.box('head', [0.15, 0.14, 0.24], [0, -0.04, 0.25], DARK_FUR);
  rig.box('jaw', [0.13, 0.04, 0.2], [0, -0.04, 0.07], DARK_FUR);
  for (const s of [1, -1]) {
    rig.cone('head', 0.045, 0.14, [s * 0.09, 0.2, -0.02], FUR, { rot: [-0.2, 0, -s * 0.2] });
    rig.sphere('head', 0.028, [s * 0.11, 0.06, 0.14], 0xbfeaff, { glow: 1.2 });
    rig.cone('head', 0.024, 0.1, [s * 0.05, -0.13, 0.32], BONE, { rot: [Math.PI, 0, 0] });
  }
  rig.cone('tail', 0.055, 0.5, [0, 0.0, -0.25], FUR, { rot: [-Math.PI / 2 - 0.3, 0, 0] });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`fleg${side}`, 0.06, 0.05, 0.32, [0, -0.16, 0], FUR);
    rig.cyl(`fshin${side}`, 0.045, 0.035, 0.34, [0, -0.17, 0], DARK_FUR);
    rig.box(`fshin${side}`, [0.09, 0.05, 0.15], [0, -0.34, 0.04], DARK_FUR);
    rig.cyl(`hleg${side}`, 0.075, 0.05, 0.34, [0, -0.17, 0], FUR);
    rig.cyl(`hshin${side}`, 0.05, 0.035, 0.34, [0, -0.17, 0.0], DARK_FUR);
    rig.box(`hshin${side}`, [0.09, 0.05, 0.15], [0, -0.34, 0.04], DARK_FUR);
  }
  return rig;
}

const idle = (t: number): Pose => ({
  body: [0, 0, 0, 0, Math.sin(t * 2.0) * 0.007, 0],
  chest: [Math.sin(t * 2.0) * 0.02, 0, 0],
  head: [-0.05 + Math.sin(t * 0.8) * 0.08, Math.sin(t * 0.5) * 0.35, 0],
  tail: [Math.sin(t * 1.5) * 0.12, Math.sin(t * 1.1) * 0.3, 0],
  jaw: [0.05 + Math.max(0, Math.sin(t * 0.45)) * 0.1]
});

const lunge: Choreo = {
  total: 48,
  keys: (total) =>
    attack(14, 19, total, {
      wind: {
        body: [0.24, 0, 0, 0, -0.22, -0.1],
        chest: [0.1],
        head: [-0.45],
        jaw: [0.2],
        flegL: [0.5],
        flegR: [0.5],
        fshinL: [-0.5],
        fshinR: [-0.5],
        hlegL: [-0.95],
        hlegR: [-0.95],
        hshinL: [1.25],
        hshinR: [1.25],
        tail: [-0.5]
      },
      strike: {
        body: [-0.3, 0, 0, 0, 0.14, 0.28],
        chest: [-0.1],
        head: [0.15],
        jaw: [0.8],
        flegL: [-1.3],
        flegR: [-1.3],
        fshinL: [0.2],
        fshinR: [0.2],
        hlegL: [1.1],
        hlegR: [1.1],
        hshinL: [-0.2],
        hshinR: [-0.2],
        tail: [0.3]
      },
      follow: { body: [0.05, 0, 0, 0, -0.05, 0.1], jaw: [0.1], flegL: [-0.3], flegR: [-0.3], hlegL: [0.3], hlegR: [0.3] }
    })
};

const bite: Choreo = {
  total: 38,
  keys: (total) =>
    attack(13, 15, total, {
      wind: { body: [0.1, 0, 0, 0, -0.08, -0.12], head: [-0.35], jaw: [0.55], chest: [-0.15], tail: [0, 0.5, 0] },
      strike: { body: [0.12, 0, 0, 0, -0.04, 0.24], head: [0.35], jaw: [-0.02], chest: [0.2], flegL: [-0.4], flegR: [-0.4] }
    })
};

export const rimeWolfDef: ModelDef = {
  id: 'rime_wolf',
  height: 1.1,
  build,
  idle,
  walk: houndWalk,
  stride: 1.7,
  moves: { rw_lunge: lunge, rw_bite: bite },
  fallback: bite,
  fallAxis: 'z',
  death: (u) => ({ head: [0.3 * u], jaw: [0.5 * u], flegL: [-0.6 * u], flegR: [0.6 * u], hlegL: [0.6 * u], hlegR: [-0.6 * u] })
};
