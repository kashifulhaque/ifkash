// The Husk (7000) and its variants: Servant (7001), Miner (7002), Frost Husk (7003). A
// hollow, hunched person with arms that hang to the knees. Moves: husk_swipe_1 (right claw
// swipe), husk_swipe_2 (left backhand), husk_miner_pick (two-hand overhead pick), and the
// Frost swipes. Strike frames come from game/data/moves/enemies/husk.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

export type HuskVariant = 'husk' | 'servant' | 'miner' | 'frost';

const SKIN = 0xb7b09f;
const CLOTH = 0x5d584a;
const DARK = 0x16140f;

function build(v: HuskVariant): Rig {
  const rig = biped({ leg: 0.82, torso: 0.58, neck: 0.1, shoulder: 0.22, hip: 0.1, upperArm: 0.4, foreArm: 0.46 });
  const skin = v === 'frost' ? 0xa9c3d1 : SKIN;
  const cloth = v === 'servant' ? 0x46505f : v === 'miner' ? 0x6e5a3e : v === 'frost' ? 0x6f8794 : CLOTH;
  rig.restRot('spine', [0.3, 0, 0]);
  rig.box('hips', [0.3, 0.18, 0.2], [0, 0.0, 0], cloth);
  rig.box('spine', [0.38, 0.58, 0.22], [0, 0.31, 0], skin);
  rig.box('spine', [0.4, 0.3, 0.24], [0, 0.18, 0.005], cloth);
  rig.sphere('head', 0.17, [0, 0.12, 0.03], skin, { squash: [0.95, 1.1, 1] });
  // Hollow eyes and mouth.
  rig.box('head', [0.06, 0.07, 0.04], [0.065, 0.14, 0.15], DARK);
  rig.box('head', [0.06, 0.07, 0.04], [-0.065, 0.14, 0.15], DARK);
  rig.box('head', [0.1, 0.05, 0.04], [0, 0.03, 0.15], DARK);
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.05, 0.055, 0.4, [0, -0.2, 0], skin);
    rig.cyl(`elbow${side}`, 0.045, 0.05, 0.46, [0, -0.23, 0], skin);
    // Long claws.
    for (const dx of [-0.03, 0, 0.03]) {
      rig.cone(`hand${side}`, 0.014, 0.12, [dx, -0.06, 0.02], 0xd8d2c0, { rot: [Math.PI, 0, 0] });
    }
    rig.box(`hand${side}`, [0.09, 0.08, 0.04], [0, -0.0, 0], skin);
    rig.cyl(`hip${side}`, 0.065, 0.055, 0.41, [0, -0.2, 0], cloth);
    rig.cyl(`knee${side}`, 0.05, 0.045, 0.41, [0, -0.2, 0], skin);
    rig.box(`foot${side}`, [0.1, 0.06, 0.22], [0, 0.03, 0.05], skin);
  }
  if (v === 'servant') {
    // A pale apron and a collar: a castle servant's livery.
    rig.box('spine', [0.3, 0.5, 0.03], [0, 0.18, 0.14], 0xd9d4c4);
    rig.box('head', [0.26, 0.05, 0.22], [0, -0.06, 0], 0xd9d4c4);
  }
  if (v === 'miner') {
    // A lit lantern on the belt, a cap, and a pickaxe in both hands.
    rig.cyl('hips', 0.02, 0.02, 0.1, [0.2, 0.02, 0.04], 0x2a2620);
    rig.sphere('hips', 0.08, [0.2, -0.1, 0.04], 0xffb347, { glow: 1.6 });
    rig.cyl('head', 0.19, 0.21, 0.08, [0, 0.27, 0.02], 0x8a7a2e);
    rig.sphere('head', 0.03, [0, 0.3, 0.2], 0xffb347, { glow: 1.6 });
    rig.bone('pick', 'handR', [0, 0, 0.02]);
    rig.restRot('pick', [-0.5, 0, 0]);
    rig.cyl('pick', 0.025, 0.025, 1.1, [0, -0.4, 0], 0x5a4128);
    rig.box('pick', [0.06, 0.07, 0.5], [0, -0.92, 0], 0x6c7077);
    rig.cone('pick', 0.05, 0.2, [0, -0.92, 0.32], 0x6c7077, { rot: [Math.PI / 2, 0, 0] });
  }
  if (v === 'frost') {
    // Rime: pale ice shards on the shoulders and back.
    for (const [bone, pos, rot, h] of [
      ['shoulderL', [0.05, 0.08, 0], [0, 0, -0.5], 0.3],
      ['shoulderR', [-0.05, 0.08, 0], [0, 0, 0.5], 0.26],
      ['spine', [0.1, 0.5, -0.1], [-0.3, 0, -0.2], 0.34],
      ['spine', [-0.08, 0.46, -0.12], [-0.3, 0, 0.3], 0.28]
    ] as const) {
      rig.cone(bone, 0.05, h, [pos[0], pos[1] + h / 2, pos[2]], 0xcdeaf5, { rot: [rot[0], rot[1], rot[2]], glow: 0.5 });
    }
  }
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.5) * 0.008, 0],
  spine: [Math.sin(t * 1.5) * 0.03, Math.sin(t * 0.7) * 0.04, 0],
  head: [0.12 + Math.sin(t * 1.1) * 0.05, Math.sin(t * 0.6) * 0.18, Math.sin(t * 0.9) * 0.06],
  shoulderL: [Math.sin(t * 1.3) * 0.05, 0, 0.08],
  shoulderR: [Math.sin(t * 1.3 + 1) * 0.05, 0, -0.08],
  elbowL: [-0.12, 0, 0],
  elbowR: [-0.12, 0, 0]
});

const walk = (phase: number, amp: number): Pose => ({
  ...bipedGait(phase, amp, 0.5),
  // A shamble: the head lags and lolls.
  head: [0.1, Math.sin(phase) * 0.25 * amp, Math.sin(phase * 2) * 0.1 * amp]
});

const swipe1: Choreo = {
  total: 58,
  keys: (total) =>
    attack(18, 21, total, {
      wind: { shoulderR: [-2.5, 0, -0.5], elbowR: [-0.7], spine: [-0.1, -0.6, 0], head: [0.05, 0.3, 0] },
      strike: { shoulderR: [-1.1, 0, 0.5], elbowR: [-0.15], spine: [0.35, 0.7, 0], hips: [0, 0.25, 0, 0, 0, 0.1] }
    })
};

const swipe2: Choreo = {
  total: 66,
  keys: (total) =>
    attack(20, 24, total, {
      wind: { shoulderL: [-1.5, 0, 1.5], elbowL: [-0.3], spine: [0.0, 0.6, 0], head: [0.05, -0.3, 0] },
      strike: { shoulderL: [-1.4, 0, -0.5], elbowL: [-0.1], spine: [0.3, -0.7, 0], hips: [0, -0.25, 0, 0, 0, 0.1] }
    })
};

const pick: Choreo = {
  total: 70,
  keys: (total) =>
    attack(22, 26, total, {
      wind: {
        shoulderR: [-3.0, 0, -0.1],
        shoulderL: [-3.0, 0, 0.1],
        elbowR: [-0.2],
        elbowL: [-0.2],
        spine: [-0.45, 0, 0],
        head: [-0.2, 0, 0]
      },
      strike: {
        shoulderR: [-1.1, 0, -0.1],
        shoulderL: [-1.1, 0, 0.1],
        spine: [0.7, 0, 0],
        hips: [0, 0, 0, 0, -0.06, 0.15]
      }
    })
};

export function huskDef(v: HuskVariant): ModelDef {
  const id = v === 'husk' ? 'husk' : `husk_${v}`;
  const moves: Record<string, Choreo> = {
    husk_swipe_1: swipe1,
    husk_swipe_2: swipe2,
    husk_frost_swipe_1: swipe1,
    husk_frost_swipe_2: swipe2,
    husk_miner_pick: pick
  };
  return {
    id,
    height: 1.7,
    build: () => build(v),
    idle,
    walk,
    stride: 1.1,
    moves,
    fallback: swipe1,
    fallAxis: 'x',
    death: (u) => ({ spine: [0.5 * u], head: [0.4 * u], shoulderL: [-0.4 * u], shoulderR: [-0.4 * u], kneeL: [0.8 * u], kneeR: [0.8 * u] })
  };
}
