// The Bellbearer (`b06_bellbearer`, registry 35002), the Drowned Choir's tank: a broad drowned
// man in a bronze-stained surplice who drags a cathedral handbell the size of a barrel on a chain.
// Spec: docs/game/05-enemies-and-bosses.md, "B06 The Drowned Choir". Moves
// (game/data/moves/bosses/b06.ron): bell_slam (hit on frame 32), bell_swing_1 and bell_swing_2
// (hits on 18 and 22), bell_ring (34-frame raise, then the stun wave), bell_charge (the verse
// move: aims for 24 frames, runs from frame 28 to 46).

import { biped, bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, lerpPose, type Choreo, type Keyframes } from '../../procedural/pose';
import type { Pose, Rig } from '../../procedural/rig';
import { CHOIR } from './palette';

function build(): Rig {
  const rig = biped({ leg: 1.0, torso: 0.85, neck: 0.1, shoulder: 0.42, hip: 0.2, upperArm: 0.46, foreArm: 0.46 });
  rig.box('hips', [0.62, 0.3, 0.4], [0, 0, 0], CHOIR.robe);
  rig.box('spine', [0.86, 0.9, 0.46], [0, 0.42, 0], CHOIR.robe);
  rig.box('spine', [0.9, 0.12, 0.5], [0, 0.78, 0], CHOIR.trim);
  rig.cyl('hips', 0.3, 0.5, 0.9, [0, -0.38, 0], CHOIR.robe);
  rig.cyl('hips', 0.505, 0.51, 0.05, [0, -0.82, 0], CHOIR.weed);
  rig.sphere('head', 0.2, [0, 0.14, 0], CHOIR.skin, { squash: [1, 1.05, 1] });
  rig.box('head', [0.26, 0.05, 0.05], [0, 0.18, 0.18], CHOIR.dark);
  rig.sphere('head', 0.022, [0.07, 0.17, 0.18], CHOIR.light, { glow: 1.3 });
  rig.sphere('head', 0.022, [-0.07, 0.17, 0.18], CHOIR.light, { glow: 1.3 });
  for (const side of ['L', 'R'] as const) {
    rig.sphere(`shoulder${side}`, 0.22, [0, 0, 0], CHOIR.trim, { squash: [1.1, 0.8, 1] });
    rig.cyl(`shoulder${side}`, 0.12, 0.14, 0.46, [0, -0.23, 0], CHOIR.robe);
    rig.cyl(`elbow${side}`, 0.11, 0.13, 0.46, [0, -0.23, 0], CHOIR.robe);
    rig.sphere(`hand${side}`, 0.1, [0, 0, 0], CHOIR.skin);
  }
  // The bell: a bronze cup on a short chain from the right hand. It glows while it rings.
  rig.bone('bell', 'handR', [0, -0.05, 0.05]);
  rig.restRot('bell', [-0.2, 0, 0]);
  rig.cyl('bell', 0.015, 0.015, 0.45, [0, -0.2, 0], CHOIR.bronzeDark);
  rig.cyl('bell', 0.18, 0.46, 0.62, [0, -0.78, 0], CHOIR.bronze);
  rig.cyl('bell', 0.465, 0.47, 0.06, [0, -1.1, 0], CHOIR.bronzeDark);
  rig.sphere('bell', 0.11, [0, -1.1, 0], CHOIR.light, { glow: 0.8 });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 0.8) * 0.014, 0],
  spine: [0.1 + Math.sin(t * 0.8) * 0.02, Math.sin(t * 0.35) * 0.05, 0],
  head: [0.1, Math.sin(t * 0.4) * 0.1, 0],
  shoulderR: [-0.15, 0, -0.2],
  elbowR: [-0.3, 0, 0],
  shoulderL: [-0.2, 0, 0.2],
  elbowL: [-0.4, 0, 0],
  bell: [Math.sin(t * 0.9) * 0.08, 0, Math.sin(t * 0.7) * 0.05]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.25);
  delete g.shoulderR;
  return { ...g, shoulderR: [-0.15, 0, -0.2], bell: [Math.sin(phase) * 0.2 * amp] };
};

const glowTo = (at: number) => (f: number) => (f < at ? 1 + (f / at) * 1.6 : Math.max(1, 2.6 - (f - at) * 0.1));

const OVERHEAD: Pose = { shoulderR: [-3.0, 0, -0.2], elbowR: [-0.4], shoulderL: [-1.2, 0, 0.4], spine: [-0.5, 0, 0], head: [-0.2, 0, 0], bell: [0.3] };

const slam: Choreo = {
  total: 98,
  keys: (total) =>
    attack(32, 35, total, {
      wind: OVERHEAD,
      strike: { shoulderR: [-1.0, 0, -0.1], elbowR: [-0.1], shoulderL: [-0.8, 0, 0.4], spine: [0.7, 0, 0], head: [0.3, 0, 0], hips: [0, 0, 0, 0, -0.14, 0.3], bell: [-0.4] }
    }),
  glow: glowTo(32)
};

const swing1: Choreo = {
  total: 44,
  keys: (total) =>
    attack(18, 21, total, {
      wind: { shoulderR: [-1.3, 0, -1.7], elbowR: [-0.3], spine: [0, -0.9, 0], head: [0, 0.4, 0], bell: [0.2] },
      strike: { shoulderR: [-1.4, 0, 0.9], elbowR: [-0.05], spine: [0.05, 0.8, 0], head: [0, -0.3, 0], bell: [-0.2] }
    }),
  glow: glowTo(18)
};

const swing2: Choreo = {
  total: 90,
  keys: (total) =>
    attack(22, 25, total, {
      wind: { shoulderR: [-2.4, 0, 0.5], elbowR: [-0.4], spine: [-0.3, 0.6, 0], bell: [0.3] },
      strike: { shoulderR: [-1.1, 0, -0.2], elbowR: [-0.1], spine: [0.6, -0.2, 0], hips: [0, 0, 0, 0, -0.12, 0.25], bell: [-0.4] }
    }),
  glow: glowTo(22)
};

/** Bell ring: the bell hauled overhead for 34 frames, then slammed down and the wave rolls out. */
const ring: Choreo = {
  total: 104,
  keys: (total): Keyframes => {
    const up: Pose = { ...OVERHEAD, shoulderL: [-2.8, 0, 0.3], elbowL: [-0.3] };
    const down: Pose = { shoulderR: [-0.7, 0, -0.1], shoulderL: [-0.7, 0, 0.1], spine: [0.9, 0, 0], head: [0.4, 0, 0], hips: [0, 0, 0, 0, -0.22, 0.2], bell: [-0.5] };
    return [
      [0, {}],
      [14, lerpPose({}, up, 0.6)],
      [31, up],
      [34, up],
      [37, down],
      [50, down],
      [total, {}]
    ];
  },
  glow: (f) => (f < 34 ? 1 + (f / 34) * 2.4 : Math.max(1, 3.4 - (f - 34) * 0.1))
};

/** The charge: head and bell dropped, a long lean forward, the run, then the stagger of a stop. */
const charge: Choreo = {
  total: 120,
  keys: (total): Keyframes => {
    const aim: Pose = { spine: [0.6, 0, 0], head: [0.5, 0, 0], shoulderR: [-1.3, 0, -0.1], shoulderL: [-1.3, 0, 0.1], bell: [0.9], hipL: [-0.4], hipR: [0.4] };
    const run: Pose = { spine: [0.9, 0, 0], head: [0.4, 0, 0], shoulderR: [-1.6, 0, -0.05], shoulderL: [-1.6, 0, 0.05], bell: [1.1], hipL: [0.7], hipR: [-0.7], kneeL: [0.6] };
    return [
      [0, {}],
      [14, aim],
      [27, aim],
      [32, run],
      [46, run],
      [60, { ...run, spine: [0.5, 0, 0] }],
      [total, {}]
    ];
  },
  glow: (f) => (f < 28 ? 1 + (f / 28) * 1.8 : 2.8)
};

export const bellbearerDef: ModelDef = {
  id: 'b06_bellbearer',
  height: 2.6,
  build,
  idle,
  walk,
  stride: 1.9,
  moves: { slam, swing_1: swing1, swing_2: swing2, ring, charge },
  fallback: slam,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.5 * u], head: [0.6 * u], kneeL: [1.1 * u], kneeR: [1.2 * u], shoulderR: [0.5 * u], bell: [1.0 * u] })
};
