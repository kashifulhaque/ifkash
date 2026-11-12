// The Bell Warden (7087): a gaunt, hooded warden in a long frost-grey habit, a heavy hand bell
// in the right hand. Strikes the bell twice, then rings it out as a stun wave. Moves:
// bw_strike_1 and bw_strike_2 (bell swings) and bw_wave (the bell raised overhead and rung).
// Strike frames come from game/data/moves/enemies/bell_warden.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo, type Keyframes } from '../pose';
import type { Pose, Rig } from '../rig';

const HABIT = 0x7a8794;
const TRIM = 0xdfe8ee;
const BRONZE = 0xb98a3c;
const DARK = 0x242a31;
const RIME = 0xbfe8f5;

function build(): Rig {
  const rig = biped({ leg: 0.95, torso: 0.7, neck: 0.1, shoulder: 0.26, hip: 0.11, upperArm: 0.42, foreArm: 0.42 });
  rig.restRot('spine', [0.12, 0, 0]);
  rig.box('spine', [0.42, 0.7, 0.26], [0, 0.35, 0], HABIT);
  rig.box('spine', [0.44, 0.06, 0.28], [0, 0.12, 0], TRIM);
  // The habit's skirt covers the legs; frost crusts its hem and shoulders.
  rig.cyl('hips', 0.2, 0.4, 1.0, [0, -0.42, 0], HABIT);
  rig.cyl('hips', 0.405, 0.41, 0.06, [0, -0.92, 0], RIME, { glow: 0.3 });
  rig.cone('spine', 0.06, 0.3, [0.2, 0.74, 0], RIME, { rot: [0, 0, -0.5], glow: 0.4 });
  rig.cone('spine', 0.05, 0.26, [-0.2, 0.74, 0], RIME, { rot: [0, 0, 0.5], glow: 0.4 });
  // A tall pointed hood over a pale, narrow face.
  rig.cone('head', 0.2, 0.7, [0, 0.35, -0.01], HABIT);
  rig.sphere('head', 0.17, [0, 0.12, 0.0], HABIT, { squash: [0.95, 1.1, 1.05] });
  rig.box('head', [0.17, 0.2, 0.06], [0, 0.1, 0.13], DARK);
  for (const s of [1, -1]) rig.sphere('head', 0.016, [s * 0.04, 0.12, 0.165], RIME, { glow: 1.6 });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.07, 0.085, 0.42, [0, -0.21, 0], HABIT);
    rig.cyl(`elbow${side}`, 0.06, 0.07, 0.42, [0, -0.21, 0], HABIT);
    rig.box(`hand${side}`, [0.09, 0.09, 0.09], [0, 0, 0], 0xc8cdd2);
  }
  // The bell: a bronze cup on a short handle, its clapper swaying below.
  rig.bone('bell', 'handR', [0, 0, 0.02]);
  rig.restRot('bell', [1.0, 0, 0]);
  rig.cyl('bell', 0.02, 0.02, 0.2, [0, 0.0, 0], 0x5a4128);
  rig.cyl('bell', 0.07, 0.22, 0.34, [0, -0.28, 0], BRONZE, { glow: 0.3 });
  rig.cyl('bell', 0.225, 0.23, 0.04, [0, -0.46, 0], TRIM);
  rig.sphere('bell', 0.05, [0, -0.5, 0], DARK);
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.2) * 0.006, 0],
  spine: [Math.sin(t * 1.2) * 0.015, Math.sin(t * 0.5) * 0.06, 0],
  head: [0.1, Math.sin(t * 0.6) * 0.2, 0],
  shoulderR: [-0.3, 0, -0.18],
  elbowR: [-0.8],
  shoulderL: [Math.sin(t * 1.0) * 0.04, 0, 0.14],
  elbowL: [-0.2],
  bell: [Math.sin(t * 1.4) * 0.1]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.25);
  delete g.shoulderR;
  return { ...g, shoulderR: [-0.3, 0, -0.18], bell: [Math.sin(phase) * 0.15 * amp] };
};

const strike1: Choreo = {
  total: 44,
  keys: (total) =>
    attack(16, 19, total, {
      wind: { shoulderR: [-2.4, 0, -0.6], elbowR: [-0.7], spine: [-0.1, -0.5, 0], head: [-0.1, 0.2, 0] },
      strike: { shoulderR: [-1.3, 0, 0.5], elbowR: [-0.15], spine: [0.3, 0.6, 0], hips: [0, 0.2, 0, 0, -0.04, 0.15], bell: [0.5] }
    })
};

const strike2: Choreo = {
  total: 44,
  keys: (total) =>
    attack(16, 19, total, {
      wind: { shoulderR: [-1.5, 0, 1.2], elbowR: [-0.5], spine: [0, 0.6, 0], head: [0, -0.2, 0] },
      strike: { shoulderR: [-1.4, 0, -0.9], elbowR: [-0.1], spine: [0.25, -0.6, 0], hips: [0, -0.2, 0, 0, -0.04, 0.15], bell: [-0.5] }
    })
};

/** The stun wave: the bell lifted high over 36 frames, then shaken as it rings out. */
const waveKeys = (total: number): Keyframes => {
  const lift = { shoulderR: [-3.0, 0, -0.3], elbowR: [-0.35], spine: [-0.3, 0, 0], head: [-0.35, 0, 0], shoulderL: [-0.6, 0, 0.5], bell: [0.0] };
  const keys: Keyframes = [
    [0, {}],
    [14, { shoulderR: [-1.7, 0, -0.2], elbowR: [-0.7], spine: [-0.1, 0, 0] }],
    [33, lift],
    [36, lift]
  ];
  [37, 38, 39, 40].forEach((f, i) => keys.push([f, { ...lift, bell: [i % 2 ? -1.0 : 1.0], head: [-0.15, 0, 0], spine: [-0.05, 0, 0] }]));
  keys.push([56, { shoulderR: [-2.4, 0, -0.3], elbowR: [-0.5], bell: [0.2] }]);
  keys.push([Math.max(total, 62), {}]);
  return keys;
};

const wave: Choreo = {
  total: 80,
  keys: waveKeys,
  // The bell and the warden's eyes brighten through the lift and flash as the wave rings out.
  glow: (f) => (f < 36 ? 1 + (f / 36) * 1.8 : f < 44 ? 4 : Math.max(1, 4 - (f - 44) * 0.2))
};

export const bellWardenDef: ModelDef = {
  id: 'bell_warden',
  height: 2.1,
  build,
  idle,
  walk,
  stride: 1.5,
  moves: { bw_strike_1: strike1, bw_strike_2: strike2, bw_wave: wave },
  fallback: strike1,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.35 * u], head: [0.5 * u], kneeL: [0.7 * u], kneeR: [0.8 * u], shoulderR: [0.3 * u] })
};
