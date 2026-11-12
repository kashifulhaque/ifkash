// The Marsh Crawler (7053): a low, bloated, six-legged swamp beetle the size of a dog, with a
// pale venom sac. Move: mc_bite (a quick bite; Poison buildup). Strike frames come from
// game/data/moves/enemies/marsh_crawler.ron.

import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import { Rig, type Pose } from '../rig';

const SHELL = 0x4b5a32;
const DARK = 0x2b331c;
const SAC = 0x9fc25a;
const BONE = 0xd9d3b4;

const LEG_Z = [0.2, 0.0, -0.2] as const;
const LEG_SPLAY = [0.6, 0.9, 0.6] as const;

function build(): Rig {
  const rig = new Rig();
  rig.bone('body', null, [0, 0.3, 0]);
  rig.bone('head', 'body', [0, 0.0, 0.32]);
  rig.bone('jaw', 'head', [0, -0.05, 0.1]);
  LEG_Z.forEach((z, i) => {
    for (const [s, side] of [
      [1, 'L'],
      [-1, 'R']
    ] as const) {
      rig.bone(`leg${i}${side}`, 'body', [s * 0.17, -0.04, z]);
      rig.restRot(`leg${i}${side}`, [0, 0, s * LEG_SPLAY[i]]);
      rig.bone(`shin${i}${side}`, `leg${i}${side}`, [0, -0.18, 0]);
      rig.restRot(`shin${i}${side}`, [0, 0, -s * 0.9]);
    }
  });
  rig.sphere('body', 0.24, [0, 0.03, -0.05], SHELL, { squash: [1.0, 0.7, 1.35] });
  rig.sphere('body', 0.17, [0, 0.1, -0.3], SAC, { glow: 0.5, squash: [1, 0.9, 1.1] });
  rig.box('body', [0.05, 0.03, 0.5], [0, 0.2, 0.0], DARK);
  rig.sphere('head', 0.12, [0, 0, 0.02], SHELL, { squash: [1, 0.8, 1] });
  rig.box('jaw', [0.1, 0.03, 0.12], [0, -0.01, 0.04], DARK);
  for (const s of [1, -1]) {
    rig.cone('head', 0.025, 0.12, [s * 0.05, -0.07, 0.14], BONE, { rot: [Math.PI / 2 + 0.3, 0, 0] });
    rig.sphere('head', 0.025, [s * 0.07, 0.05, 0.08], 0xd8e85a, { glow: 1.0 });
  }
  for (let i = 0; i < 3; i++) {
    for (const side of ['L', 'R'] as const) {
      rig.cyl(`leg${i}${side}`, 0.025, 0.02, 0.2, [0, -0.09, 0], DARK);
      rig.cyl(`shin${i}${side}`, 0.02, 0.012, 0.22, [0, -0.1, 0], SHELL);
    }
  }
  return rig;
}

const idle = (t: number): Pose => ({
  body: [0, 0, 0, 0, Math.sin(t * 2.2) * 0.008, 0],
  head: [Math.sin(t * 1.2) * 0.1, Math.sin(t * 0.8) * 0.3, 0],
  jaw: [0.05 + Math.max(0, Math.sin(t * 1.9)) * 0.12]
});

/** A tripod gait: legs 0 and 2 on one side move with leg 1 on the other. */
const walk = (phase: number, amp: number): Pose => {
  const out: Pose = { body: [0, 0, 0, 0, Math.abs(Math.cos(phase)) * 0.015 * amp, 0] };
  for (let i = 0; i < 3; i++) {
    const a = i === 1 ? phase + Math.PI : phase;
    out[`leg${i}L`] = [Math.sin(a) * 0.5 * amp, 0, Math.max(0, Math.cos(a)) * 0.25 * amp];
    out[`leg${i}R`] = [Math.sin(a + Math.PI) * 0.5 * amp, 0, -Math.max(0, -Math.cos(a)) * 0.25 * amp];
  }
  return out;
};

const bite: Choreo = {
  total: 36,
  keys: (total) =>
    attack(12, 14, total, {
      wind: { body: [-0.35, 0, 0, 0, 0.04, -0.1], head: [-0.4], jaw: [0.7], leg0L: [-0.4], leg0R: [-0.4] },
      strike: { body: [0.3, 0, 0, 0, -0.02, 0.22], head: [0.3], jaw: [-0.05], leg1L: [-0.4], leg1R: [-0.4] }
    })
};

export const marshCrawlerDef: ModelDef = {
  id: 'marsh_crawler',
  height: 0.45,
  build,
  idle,
  walk,
  stride: 0.5,
  moves: { mc_bite: bite },
  fallback: bite,
  fallAxis: 'z',
  death: (u) => ({ head: [0.2 * u], jaw: [0.6 * u], body: [0, 0, 0, 0, -0.08 * u, 0] })
};
