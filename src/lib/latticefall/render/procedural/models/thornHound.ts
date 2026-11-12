// The Thorn Hound (7010): a lean, quick quadruped grown over with thorns. Moves: th_lunge
// (crouch, then a long leaping bite) and th_snap (a short head-forward snap). Strike frames
// come from game/data/moves/enemies/thorn_hound.ron.

import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import { Rig, type Pose } from '../rig';

const FUR = 0x6a5a45;
const DARK_FUR = 0x453a2c;
const THORN = 0x8d3b24;
const BONE = 0xe2d9c3;

function build(): Rig {
  const rig = new Rig();
  rig.bone('body', null, [0, 0.62, 0]);
  rig.bone('chest', 'body', [0, 0.02, 0.28]);
  rig.bone('head', 'chest', [0, 0.1, 0.3]);
  rig.bone('jaw', 'head', [0, -0.02, 0.1]);
  rig.bone('tail', 'body', [0, 0.06, -0.5]);
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.bone(`fleg${side}`, 'chest', [s * 0.11, -0.08, 0.0]);
    rig.bone(`fshin${side}`, `fleg${side}`, [0, -0.28, 0]);
    rig.bone(`hleg${side}`, 'body', [s * 0.11, -0.05, -0.34]);
    rig.bone(`hshin${side}`, `hleg${side}`, [0, -0.3, 0.0]);
  }
  rig.box('body', [0.3, 0.3, 0.7], [0, 0, -0.05], FUR);
  rig.box('chest', [0.34, 0.34, 0.3], [0, 0.01, 0.02], FUR);
  rig.box('chest', [0.2, 0.04, 0.4], [0, 0.22, -0.1], DARK_FUR);
  // Thorns along the spine, longest over the shoulders.
  for (const [bone, z, h] of [
    ['chest', 0.08, 0.17],
    ['chest', -0.06, 0.17],
    ['body', -0.08, 0.14],
    ['body', -0.24, 0.12],
    ['body', -0.4, 0.1]
  ] as const) {
    rig.cone(bone, 0.035, h, [0, 0.2 + h / 2 - 0.02, z], THORN, { rot: [-0.25, 0, 0] });
  }
  rig.box('head', [0.2, 0.2, 0.22], [0, 0, 0.05], FUR);
  rig.box('head', [0.13, 0.12, 0.2], [0, -0.03, 0.22], DARK_FUR);
  rig.box('jaw', [0.11, 0.04, 0.17], [0, -0.04, 0.06], DARK_FUR);
  for (const s of [1, -1]) {
    rig.cone('head', 0.04, 0.12, [s * 0.08, 0.17, -0.02], FUR, { rot: [-0.2, 0, -s * 0.2] });
    rig.sphere('head', 0.025, [s * 0.1, 0.05, 0.12], 0xe9a23a, { glow: 0.9 });
    rig.cone('head', 0.022, 0.08, [s * 0.04, -0.12, 0.28], BONE, { rot: [Math.PI, 0, 0] });
  }
  rig.cone('tail', 0.045, 0.45, [0, 0.0, -0.22], DARK_FUR, { rot: [-Math.PI / 2 - 0.4, 0, 0] });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`fleg${side}`, 0.05, 0.04, 0.28, [0, -0.14, 0], FUR);
    rig.cyl(`fshin${side}`, 0.04, 0.03, 0.3, [0, -0.15, 0], DARK_FUR);
    rig.box(`fshin${side}`, [0.07, 0.04, 0.12], [0, -0.3, 0.03], DARK_FUR);
    rig.cyl(`hleg${side}`, 0.065, 0.045, 0.3, [0, -0.15, 0], FUR);
    rig.cyl(`hshin${side}`, 0.045, 0.03, 0.3, [0, -0.15, 0.0], DARK_FUR);
    rig.box(`hshin${side}`, [0.07, 0.04, 0.12], [0, -0.3, 0.03], DARK_FUR);
  }
  return rig;
}

const idle = (t: number): Pose => ({
  body: [0, 0, 0, 0, Math.sin(t * 2.4) * 0.006, 0],
  chest: [Math.sin(t * 2.4) * 0.02, 0, 0],
  head: [-0.1 + Math.sin(t * 0.9) * 0.08, Math.sin(t * 0.6) * 0.3, 0],
  tail: [Math.sin(t * 1.7) * 0.15, Math.sin(t * 1.2) * 0.3, 0],
  jaw: [0.04 + Math.max(0, Math.sin(t * 0.5)) * 0.1]
});

/** A gallop-trot: diagonal pairs swing together. */
const walk = (phase: number, amp: number): Pose => {
  const s = Math.sin(phase) * 0.8 * amp;
  const c = Math.cos(phase);
  const bend = (v: number) => Math.max(0, v) * 0.7 * amp;
  return {
    body: [0.04 * Math.sin(phase * 2) * amp, 0, 0, 0, Math.abs(c) * 0.03 * amp, 0],
    flegL: [-s],
    flegR: [s],
    fshinL: [bend(-Math.cos(phase))],
    fshinR: [bend(Math.cos(phase))],
    hlegL: [s],
    hlegR: [-s],
    hshinL: [-bend(Math.cos(phase)) * 1.1],
    hshinR: [-bend(-Math.cos(phase)) * 1.1],
    tail: [0, Math.sin(phase) * 0.35 * amp, 0],
    head: [0, 0, 0]
  };
};

const lunge: Choreo = {
  total: 46,
  keys: (total) =>
    attack(14, 18, total, {
      // Coil: front low, haunches loaded, head up and eyes on the target.
      wind: {
        body: [0.22, 0, 0, 0, -0.2, -0.1],
        chest: [0.1],
        head: [-0.4],
        jaw: [0.15],
        flegL: [0.5],
        flegR: [0.5],
        fshinL: [-0.5],
        fshinR: [-0.5],
        hlegL: [-0.9],
        hlegR: [-0.9],
        hshinL: [1.2],
        hshinR: [1.2],
        tail: [-0.5]
      },
      // Airborne, stretched out, jaws wide.
      strike: {
        body: [-0.3, 0, 0, 0, 0.12, 0.25],
        chest: [-0.1],
        head: [0.15],
        jaw: [0.75],
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

const snap: Choreo = {
  total: 40,
  keys: (total) =>
    attack(13, 15, total, {
      wind: { body: [0.1, 0, 0, 0, -0.08, -0.12], head: [-0.35], jaw: [0.5], chest: [-0.15], tail: [0, 0.5, 0] },
      strike: { body: [0.12, 0, 0, 0, -0.04, 0.22], head: [0.35], jaw: [-0.02], chest: [0.2], flegL: [-0.4], flegR: [-0.4] }
    })
};

export const thornHoundDef: ModelDef = {
  id: 'thorn_hound',
  height: 0.95,
  build,
  idle,
  walk,
  stride: 1.5,
  moves: { th_lunge: lunge, th_snap: snap },
  fallback: snap,
  fallAxis: 'z',
  death: (u) => ({ head: [0.3 * u], jaw: [0.5 * u], flegL: [-0.6 * u], flegR: [0.6 * u], hlegL: [0.6 * u], hlegR: [-0.6 * u] })
};
