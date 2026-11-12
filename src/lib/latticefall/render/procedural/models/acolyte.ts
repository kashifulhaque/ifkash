// The Acolyte (7050) and its Bell variant (7058): a hooded caster in a long robe with a staff
// topped by a lamp. Moves: ac_cast and ac_bell_cast (a 30-frame glow, then the bolt leaves on
// frame 30) and ac_smack (a staff swat). Strike frames come from
// game/data/moves/enemies/acolyte.ron and acolyte_bell.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

export type AcolyteVariant = 'acolyte' | 'bell';

// Idle holds the staff upright in the right hand; the staff bone's rest rotation cancels the
// arm's idle pitch so it stands vertical.
const IDLE_SHOULDER = -0.3;
const IDLE_ELBOW = -0.5;

function build(v: AcolyteVariant): Rig {
  const bell = v === 'bell';
  const robe = bell ? 0x4a6a80 : 0xd8d2bf;
  const trim = bell ? 0xcfe8f2 : 0xd0a23c;
  const hood = bell ? 0x35505f : 0xb8b09a;
  const light = bell ? 0x9fdcf5 : 0xffd77a;
  const rig = biped({ leg: 0.8, torso: 0.6, neck: 0.08, shoulder: 0.22, hip: 0.1, upperArm: 0.34, foreArm: 0.34 });
  rig.box('spine', [0.4, 0.62, 0.24], [0, 0.31, 0], robe);
  rig.box('spine', [0.42, 0.06, 0.26], [0, 0.1, 0], trim);
  // The skirt of the robe hangs from the hips and covers the legs.
  rig.cyl('hips', 0.2, 0.36, 0.95, [0, -0.38, 0], robe);
  rig.cyl('hips', 0.365, 0.37, 0.05, [0, -0.85, 0], trim);
  // A deep hood with a dark face and two pinpoint eyes.
  rig.sphere('head', 0.2, [0, 0.12, -0.01], hood, { squash: [0.95, 1.1, 1.05] });
  rig.box('head', [0.2, 0.2, 0.06], [0, 0.1, 0.15], 0x0d0c10);
  for (const s of [1, -1]) rig.sphere('head', 0.016, [s * 0.045, 0.12, 0.185], light, { glow: 1.5 });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.075, 0.09, 0.34, [0, -0.17, 0], robe);
    rig.cyl(`elbow${side}`, 0.06, 0.07, 0.34, [0, -0.17, 0], robe);
    rig.sphere(`hand${side}`, 0.05, [0, 0, 0], 0xc9b9a4);
  }
  // The staff: a pole with a lamp cage on top. The lamp glows brighter while casting.
  rig.bone('staff', 'handR', [0, 0, 0.02]);
  rig.restRot('staff', [-(IDLE_SHOULDER + IDLE_ELBOW), 0, 0]);
  rig.cyl('staff', 0.022, 0.026, 1.9, [0, 0.35, 0], 0x5a4128);
  rig.cyl('staff', 0.06, 0.04, 0.1, [0, 1.3, 0], trim);
  rig.sphere('staff', 0.09, [0, 1.42, 0], light, { glow: 1.1 });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.3) * 0.006, 0],
  spine: [Math.sin(t * 1.3) * 0.015, Math.sin(t * 0.5) * 0.05, 0],
  head: [0.05, Math.sin(t * 0.6) * 0.15, 0],
  shoulderR: [IDLE_SHOULDER, 0, -0.1],
  elbowR: [IDLE_ELBOW, 0, 0],
  shoulderL: [Math.sin(t * 1.1) * 0.04, 0, 0.12],
  elbowL: [-0.25, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.3);
  delete g.shoulderR;
  return { ...g, shoulderR: [IDLE_SHOULDER, 0, -0.1] };
};

const cast: Choreo = {
  total: 80,
  // The bolt leaves on frame 30: raise the staff and the free hand through the glow, then
  // thrust the staff forward as it goes.
  keys: (total) =>
    attack(30, 32, total, {
      ready: { shoulderR: [-0.9, 0, -0.1], elbowR: [-0.7], shoulderL: [-0.5, 0, 0.6], spine: [-0.05, 0, 0] },
      wind: {
        shoulderR: [-1.9, 0, -0.15],
        elbowR: [-0.5],
        staff: [1.6],
        shoulderL: [-1.3, 0, 0.9],
        elbowL: [-0.5],
        spine: [-0.12, 0, 0],
        head: [-0.35, 0, 0]
      },
      strike: {
        shoulderR: [-1.35, 0, -0.05],
        elbowR: [-0.15],
        staff: [2.2],
        shoulderL: [-1.5, 0, 0.2],
        elbowL: [-0.2],
        spine: [0.2, 0, 0],
        head: [0.1, 0, 0]
      }
    }),
  // The lamp swells through the 30-frame glow (the telegraph) and flashes as the bolt leaves.
  glow: (f) => (f < 30 ? 1 + (f / 30) * 2.4 : Math.max(1, 3.6 - (f - 30) * 0.13))
};

const smack: Choreo = {
  total: 44,
  keys: (total) =>
    attack(14, 16, total, {
      wind: { shoulderR: [-2.3, 0, -0.5], elbowR: [-0.8], staff: [1.4], spine: [-0.1, -0.5, 0] },
      strike: { shoulderR: [-1.2, 0, 0.4], elbowR: [-0.2], staff: [1.9], spine: [0.25, 0.6, 0], hips: [0, 0.2, 0, 0, 0, 0.08] }
    })
};

export function acolyteDef(v: AcolyteVariant): ModelDef {
  return {
    id: v === 'bell' ? 'acolyte_bell' : 'acolyte',
    height: 1.9,
    build: () => build(v),
    idle,
    walk,
    stride: 1.2,
    moves: { ac_cast: cast, ac_bell_cast: cast, ac_smack: smack },
    fallback: smack,
    fallAxis: 'x',
    death: (u) => ({ spine: [0.35 * u], head: [0.5 * u], kneeL: [0.9 * u], kneeR: [0.7 * u], shoulderR: [0.4 * u], shoulderL: [-0.3 * u] })
  };
}
