// B01, the Hollow Sentry (boss id `b01_hollow_sentry`, registry 9010): an empty suit of
// armor animated by a lantern in its chest. Spec: docs/game/05-enemies-and-bosses.md, "B01
// Hollow Sentry". Its moves are authored in another work package, so choreography is matched
// by words in the move's `anim` name (overhead/slam, sweep/horizontal, bash/shield,
// combo/chain); anything else plays the slam. Frame counts follow the spec: the slam winds
// up for 30 ticks and the bash for 14, and the lantern flares during a wind-up.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, lerpPose, type Choreo, type Keyframes } from '../pose';
import type { Pose, Rig } from '../rig';

const IRON = 0x3d4048;
const TRIM = 0x666a75;
const RUSTY = 0x59473a;
const LANTERN = 0xffa23a;

function build(): Rig {
  const rig = biped({ leg: 1.2, torso: 0.95, neck: 0.16, shoulder: 0.5, hip: 0.2, upperArm: 0.55, foreArm: 0.55 });
  rig.box('hips', [0.62, 0.3, 0.4], [0, 0.0, 0], IRON);
  rig.box('hips', [0.5, 0.5, 0.05], [0, -0.28, 0.2], RUSTY);
  // The cuirass is open at the center: a cage of bars around the lantern flame.
  rig.box('spine', [0.86, 0.22, 0.5], [0, 0.12, 0], IRON);
  rig.box('spine', [0.9, 0.2, 0.52], [0, 0.88, 0], IRON);
  rig.box('spine', [0.1, 0.6, 0.5], [-0.38, 0.5, 0], IRON);
  rig.box('spine', [0.1, 0.6, 0.5], [0.38, 0.5, 0], IRON);
  for (const x of [-0.17, 0, 0.17]) rig.box('spine', [0.045, 0.62, 0.045], [x, 0.5, 0.25], TRIM);
  rig.box('spine', [0.7, 0.62, 0.04], [0, 0.5, -0.24], IRON);
  rig.sphere('spine', 0.16, [0, 0.5, 0.02], LANTERN, { glow: 1.6 });
  rig.cyl('spine', 0.04, 0.04, 0.1, [0, 0.74, 0.02], TRIM);
  // A bucket helm floating above the gorget, empty behind the visor but for a glimmer.
  rig.box('head', [0.44, 0.5, 0.44], [0, 0.22, 0], IRON);
  rig.box('head', [0.34, 0.05, 0.05], [0, 0.26, 0.225], 0x050506);
  rig.box('head', [0.04, 0.2, 0.05], [0, 0.18, 0.225], 0x050506);
  rig.sphere('head', 0.03, [0, 0.26, 0.215], LANTERN, { glow: 1.4 });
  rig.cone('head', 0.07, 0.34, [0, 0.62, -0.08], RUSTY, { rot: [-0.5, 0, 0] });
  for (const side of ['L', 'R'] as const) {
    const s = side === 'L' ? 1 : -1;
    rig.sphere(`shoulder${side}`, 0.27, [s * 0.04, 0.0, 0], IRON, { squash: [1.2, 0.9, 1.1] });
    rig.cone(`shoulder${side}`, 0.06, 0.2, [s * 0.14, 0.2, 0], TRIM, { rot: [0, 0, -s * 0.5] });
    rig.cyl(`shoulder${side}`, 0.12, 0.11, 0.55, [0, -0.27, 0], IRON);
    rig.cyl(`elbow${side}`, 0.12, 0.13, 0.55, [0, -0.27, 0], TRIM);
    rig.box(`hand${side}`, [0.2, 0.2, 0.2], [0, 0, 0], IRON);
    rig.cyl(`hip${side}`, 0.16, 0.12, 0.6, [0, -0.3, 0], IRON);
    rig.sphere(`knee${side}`, 0.13, [0, 0, 0], TRIM);
    rig.cyl(`knee${side}`, 0.12, 0.11, 0.6, [0, -0.3, 0], TRIM);
    rig.box(`foot${side}`, [0.24, 0.16, 0.46], [0, 0.08, 0.1], IRON);
  }
  // Tower shield on the left forearm, facing forward in the idle guard.
  rig.bone('shield', 'elbowL', [0, -0.3, 0]);
  rig.restRot('shield', [0.9, 0, 0]);
  rig.box('shield', [0.8, 1.15, 0.09], [0, 0.0, 0.2], IRON);
  rig.box('shield', [0.7, 1.05, 0.04], [0, 0.0, 0.26], RUSTY);
  rig.cyl('shield', 0.12, 0.12, 0.08, [0, 0.0, 0.3], TRIM, { rot: [Math.PI / 2, 0, 0] });
  // A great blade in the right hand.
  rig.bone('weapon', 'handR', [0, -0.05, 0.03]);
  rig.restRot('weapon', [-0.4, 0, 0]);
  rig.box('weapon', [0.08, 0.34, 0.08], [0, 0.0, 0], RUSTY);
  rig.box('weapon', [0.5, 0.08, 0.1], [0, -0.2, 0], TRIM);
  rig.box('weapon', [0.2, 1.6, 0.045], [0, -1.05, 0], 0x7c8088);
  rig.cone('weapon', 0.1, 0.22, [0, -1.95, 0], 0x7c8088, { rot: [Math.PI, 0, 0] });
  return rig;
}

const GUARD_S = -0.5;
const GUARD_E = -1.4;

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 0.9) * 0.012, 0],
  spine: [Math.sin(t * 0.9) * 0.015, -0.2, 0],
  head: [0.02, 0.2 + Math.sin(t * 0.4) * 0.05, 0, 0, Math.sin(t * 1.3) * 0.02, 0],
  shoulderL: [GUARD_S, 0, 0.2],
  elbowL: [GUARD_E, 0, 0],
  shoulderR: [-0.2, 0, -0.22],
  elbowR: [-0.45, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.25);
  delete g.shoulderL;
  return g;
};

/** The lantern flares through the wind-up, peaks at the strike, and settles. */
const flare = (windEnd: number) => (f: number) => {
  if (f < windEnd) return 1 + 2.2 * Math.min(f / windEnd, 1) ** 2;
  return Math.max(1, 3.2 - (f - windEnd) * 0.12);
};

const SLAM_WIND: Pose = {
  shoulderR: [-3.1, 0, -0.2],
  elbowR: [-0.5],
  shoulderL: [-1.0, 0, 0.5],
  spine: [-0.5, -0.1, 0],
  head: [-0.2, 0, 0],
  hips: [0, 0, 0, 0, 0.0, -0.1]
};
const SLAM_HIT: Pose = {
  shoulderR: [-1.15, 0, -0.1],
  elbowR: [-0.1],
  shoulderL: [-0.9, 0, 0.5],
  spine: [0.7, 0.05, 0],
  head: [0.35, 0, 0],
  hips: [0, 0, 0, 0, -0.16, 0.35],
  hipL: [-0.6],
  hipR: [0.5],
  kneeR: [0.6]
};

const slam: Choreo = {
  total: 70,
  keys: (total) => attack(30, 34, total, { wind: SLAM_WIND, strike: SLAM_HIT }),
  glow: flare(30)
};

const sweep: Choreo = {
  total: 64,
  keys: (total) =>
    attack(26, 31, total, {
      wind: { shoulderR: [-1.4, 0, -1.55], elbowR: [-0.4], spine: [0, -1.0, 0], head: [0, 0.5, 0], shoulderL: [-1.2, 0, 0.5] },
      strike: { shoulderR: [-1.5, 0, 0.9], elbowR: [-0.05], spine: [0.1, 0.9, 0], head: [0, -0.3, 0], hips: [0, 0.35, 0, 0, -0.06, 0.1] }
    }),
  glow: flare(26)
};

const bash: Choreo = {
  total: 46,
  keys: (total) =>
    attack(14, 17, total, {
      wind: { shoulderL: [-0.9, 0, 0.7], elbowL: [-1.7], spine: [-0.1, 0.5, 0], hips: [0, 0, 0, 0, -0.06, -0.05] },
      strike: { shoulderL: [-1.75, 0, 0.0], elbowL: [-0.4], spine: [0.35, -0.35, 0], hips: [0, 0, 0, 0, -0.1, 0.3], hipL: [-0.5], hipR: [0.4] }
    }),
  glow: flare(14)
};

/** Three chained swings (diagonal, backhand, overhead), by fractions of the move. */
const combo: Choreo = {
  total: 150,
  keys: (total): Keyframes => {
    const f = (u: number) => Math.round(u * total);
    const hit1: Pose = {
      shoulderR: [-1.5, 0, 0.7],
      elbowR: [-0.1],
      spine: [0.1, 0.8, 0],
      hips: [0, 0.3, 0, 0, -0.04, 0.2]
    };
    const wind1: Pose = { shoulderR: [-2.0, 0, -1.1], elbowR: [-0.5], spine: [-0.1, -0.8, 0], shoulderL: [-1.0, 0, 0.5] };
    const wind2: Pose = { shoulderR: [-1.5, 0, 0.8], elbowR: [-0.3], spine: [0, 0.8, 0] };
    const hit2: Pose = { shoulderR: [-1.5, 0, -0.9], elbowR: [-0.1], spine: [0.1, -0.8, 0], hips: [0, -0.3, 0, 0, -0.04, 0.2] };
    return [
      [0, {}],
      [f(0.1), lerpPose({}, wind1, 0.5)],
      [f(0.2), wind1],
      [f(0.25), hit1],
      [f(0.3), hit1],
      [f(0.4), wind2],
      [f(0.47), hit2],
      [f(0.52), hit2],
      [f(0.6), SLAM_WIND],
      [f(0.66), SLAM_HIT],
      [f(0.72), SLAM_HIT],
      // The 40-tick punish window: the sentry stays bent over its blade.
      [f(0.95), { ...SLAM_HIT, spine: [0.55, 0, 0] }],
      [total, {}]
    ];
  },
  glow: (frame, total) => {
    const u = frame / Math.max(total, 1);
    const near = Math.min(Math.abs(u - 0.2), Math.abs(u - 0.4), Math.abs(u - 0.6));
    return 1 + 1.8 * Math.max(0, 1 - near * 8);
  }
};

export const hollowSentryDef: ModelDef = {
  id: 'b01_hollow_sentry',
  height: 2.8,
  build,
  idle,
  walk,
  stride: 2.1,
  // Substring keys, tried in order; the first is also the fallback.
  moves: { overhead: slam, slam, sweep, horizontal: sweep, bash, shield: bash, combo, chain: combo },
  fallback: slam,
  fallAxis: 'x',
  // Armor clatters apart: it folds at the knees and the helm drops.
  death: (u) => ({ spine: [0.5 * u], head: [0.7 * u, 0, 0, 0, -0.1 * u, 0.08 * u], kneeL: [1.0 * u], kneeR: [1.3 * u], shoulderL: [0.4 * u], shoulderR: [0.6 * u] })
};
