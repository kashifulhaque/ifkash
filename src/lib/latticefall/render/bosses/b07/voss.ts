// B07, Castellan Voss, the Overfit Warden (boss id `b07_castellan_voss`, registry 36010; body
// 36011): the castle's lord, who memorized every duel he ever fought. A gaunt figure in an
// ink-violet long coat with bone-white facings, a high collar, a thin iron circlet, a greatsword
// in the right hand, and a dagger reversed in the left. His coat's lining is stitched with rows
// of tiny tally marks (the counts), and a ring of pale rune plates floats at his back; in phase 2
// it brightens (the `overfit` pose), the way the core's counters light up. Spec:
// docs/game/05-enemies-and-bosses.md, "B07 Castellan Voss, the Overfit Warden".
//
// Choreography is matched on words in the move's `anim` (see game/data/moves/bosses/b07.ron):
// voss_gs_1 to _3 (the greatsword combo, the third cut held overhead for 30 frames), voss_stab,
// voss_guard (the greatsword up across his body, then the shove), voss_counter, voss_leap_slam,
// voss_overfit, voss_sweep_pl and _pr, voss_lunge, voss_delay_1 and _2, voss_kick, voss_dash.
// Strike frames follow the move data: gs_1 on 20, gs_2 on 16, gs_3 on 46, the stab on 16, the
// guard's shove on 64, the counter on 16, the slam on 32, the sweeps on 22, the lunge on 22, the
// feint on 14 and its delayed cut on 40, the kick on 22, the dash on 30. Render-only.

import { biped, bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, lerpPose, type Choreo, type Keyframes } from '../../procedural/pose';
import type { Pose, Rig } from '../../procedural/rig';

const COAT = 0x2b2433;
const COAT_DARK = 0x1b1722;
const BONE = 0xc9b99a;
const LINING = 0x6a2f3b;
const STEEL = 0x8a93a3;
const IRON = 0x4a4d57;
const SKIN = 0xb9a58f;
const EYE = 0xe8c46b;
const RUNE = 0x9fd6ff;

function build(): Rig {
  const rig = biped({ leg: 1.12, torso: 0.86, neck: 0.18, shoulder: 0.42, hip: 0.17, upperArm: 0.5, foreArm: 0.5 });
  rig.box('hips', [0.46, 0.28, 0.32], [0, 0.0, 0], COAT_DARK);
  // The long coat: a front panel and a back panel from the hips to the shins, split at the front.
  rig.box('hips', [0.22, 0.95, 0.05], [-0.13, -0.5, 0.19], COAT);
  rig.box('hips', [0.22, 0.95, 0.05], [0.13, -0.5, 0.19], COAT);
  rig.box('hips', [0.5, 1.0, 0.05], [0, -0.52, -0.2], COAT);
  rig.box('hips', [0.52, 0.06, 0.36], [0, -0.02, 0], BONE);
  // The coat's lining flashes crimson at the hem.
  rig.box('hips', [0.52, 0.07, 0.07], [0, -1.0, -0.2], LINING);
  // The torso: a fitted doublet, a bone-white facing down the chest (the tally rows), a high collar.
  rig.box('spine', [0.62, 0.82, 0.38], [0, 0.44, 0], COAT);
  rig.box('spine', [0.16, 0.74, 0.05], [0, 0.44, 0.2], BONE);
  for (const y of [0.18, 0.3, 0.42, 0.54, 0.66]) rig.box('spine', [0.1, 0.012, 0.055], [0, y, 0.215], LINING);
  rig.box('spine', [0.66, 0.1, 0.42], [0, 0.04, 0], COAT_DARK);
  rig.box('spine', [0.5, 0.2, 0.3], [0, 0.88, -0.04], COAT_DARK);
  rig.box('spine', [0.46, 0.22, 0.06], [0, 0.94, -0.16], BONE);
  // A lean face under a thin iron circlet, with a glint in each eye.
  rig.box('head', [0.3, 0.38, 0.32], [0, 0.2, 0], SKIN);
  rig.box('head', [0.34, 0.07, 0.36], [0, 0.4, 0], IRON);
  rig.cone('head', 0.04, 0.12, [0, 0.5, 0.14], IRON);
  rig.cone('head', 0.04, 0.16, [-0.12, 0.5, 0.04], IRON);
  rig.cone('head', 0.04, 0.16, [0.12, 0.5, 0.04], IRON);
  rig.box('head', [0.32, 0.12, 0.34], [0, 0.0, 0], COAT_DARK);
  rig.sphere('head', 0.028, [-0.075, 0.26, 0.165], EYE, { glow: 1.2 });
  rig.sphere('head', 0.028, [0.075, 0.26, 0.165], EYE, { glow: 1.2 });
  // The ring of rune plates behind his shoulders: six small plates, one per counted action.
  for (let i = 0; i < 6; i++) {
    const a = (i / 6) * Math.PI * 2;
    rig.box('spine', [0.12, 0.16, 0.025], [Math.sin(a) * 0.5, 0.62 + Math.cos(a) * 0.5, -0.36], RUNE, { glow: 0.8, alpha: 0.85 });
  }
  for (const side of ['L', 'R'] as const) {
    const s = side === 'L' ? 1 : -1;
    rig.sphere(`shoulder${side}`, 0.2, [s * 0.03, 0.0, 0], COAT, { squash: [1.1, 0.9, 1] });
    rig.cyl(`shoulder${side}`, 0.095, 0.085, 0.5, [0, -0.25, 0], COAT);
    rig.cyl(`elbow${side}`, 0.085, 0.08, 0.5, [0, -0.25, 0], COAT_DARK);
    rig.box(`elbow${side}`, [0.11, 0.18, 0.11], [0, -0.1, 0], BONE);
    rig.box(`hand${side}`, [0.15, 0.15, 0.15], [0, 0, 0], IRON);
    rig.cyl(`hip${side}`, 0.12, 0.09, 0.55, [0, -0.27, 0], COAT_DARK);
    rig.sphere(`knee${side}`, 0.09, [0, 0, 0], IRON);
    rig.cyl(`knee${side}`, 0.085, 0.075, 0.55, [0, -0.27, 0], COAT_DARK);
    rig.box(`foot${side}`, [0.19, 0.13, 0.4], [0, 0.065, 0.09], COAT_DARK);
  }
  // The greatsword: a long, straight blade, a crossguard, a leather grip, and a pommel.
  rig.bone('weapon', 'handR', [0, -0.05, 0.03]);
  rig.restRot('weapon', [-0.5, 0, 0]);
  rig.cyl('weapon', 0.03, 0.03, 0.3, [0, -0.02, 0], LINING);
  rig.box('weapon', [0.5, 0.06, 0.08], [0, -0.2, 0], BONE);
  rig.box('weapon', [0.1, 1.5, 0.025], [0, -1.0, 0], STEEL);
  rig.cone('weapon', 0.05, 0.2, [0, -1.85, 0], STEEL, { rot: [Math.PI, 0, 0] });
  rig.box('weapon', [0.02, 1.3, 0.03], [0, -1.0, 0.005], 0xd9deea);
  rig.sphere('weapon', 0.05, [0, 0.16, 0], BONE);
  // The off-hand dagger, held reversed.
  rig.bone('dagger', 'handL', [0, -0.04, 0.02]);
  rig.restRot('dagger', [0.4, 0, 0]);
  rig.box('dagger', [0.04, 0.34, 0.015], [0, 0.22, 0], STEEL);
  rig.cone('dagger', 0.025, 0.1, [0, 0.44, 0], STEEL);
  rig.box('dagger', [0.1, 0.03, 0.04], [0, 0.03, 0], BONE);
  return rig;
}

// The duelist's stance: the greatsword held low and across, the dagger hand back.
const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.1) * 0.012, 0],
  spine: [Math.sin(t * 1.1) * 0.012, -0.25, 0],
  head: [0.02, 0.25 + Math.sin(t * 0.5) * 0.04, 0],
  shoulderR: [-0.55, 0, -0.3],
  elbowR: [-0.9, 0, 0],
  shoulderL: [-0.2, 0, 0.55],
  elbowL: [-1.1, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.22);
  delete g.shoulderR;
  return g;
};

/** The eye glints brighten through a wind-up and peak at the strike. */
const glint = (windEnd: number) => (f: number) => {
  if (f < windEnd) return 1 + 1.5 * Math.min(f / windEnd, 1) ** 2;
  return Math.max(1, 2.5 - (f - windEnd) * 0.1);
};

// The combo: a rising diagonal (frame 20), the backhand back across it (frame 16).
const GS_1: Choreo = {
  total: 46,
  keys: (total) =>
    attack(20, 23, total, {
      wind: { shoulderR: [-2.2, 0, -0.9], elbowR: [-0.6], spine: [-0.1, -0.8, 0], shoulderL: [-0.7, 0, 0.8] },
      strike: { shoulderR: [-1.4, 0, 0.9], elbowR: [-0.1], spine: [0.12, 0.85, 0], hips: [0, 0.3, 0, 0, -0.04, 0.2] }
    }),
  glow: glint(20)
};

const GS_2: Choreo = {
  total: 40,
  keys: (total) =>
    attack(16, 19, total, {
      wind: { shoulderR: [-1.5, 0, 0.95], elbowR: [-0.3], spine: [0, 0.85, 0], shoulderL: [-0.7, 0, 0.8] },
      strike: { shoulderR: [-1.5, 0, -1.0], elbowR: [-0.1], spine: [0.1, -0.85, 0], hips: [0, -0.3, 0, 0, -0.04, 0.2] }
    }),
  glow: glint(16)
};

/** The delayed third cut: the greatsword is raised overhead (frames 14 to 45) and held. */
const GS_3: Choreo = {
  total: 88,
  keys: (total): Keyframes => {
    const lift: Pose = { shoulderR: [-3.0, 0, -0.2], elbowR: [-0.5], shoulderL: [-0.8, 0, 0.7], spine: [-0.45, -0.1, 0], head: [-0.2, 0, 0] };
    const held: Pose = { ...lift, spine: [-0.52, -0.1, 0], shoulderR: [-3.1, 0, -0.15] };
    const hit: Pose = {
      shoulderR: [-1.15, 0, -0.1],
      elbowR: [-0.1],
      spine: [0.7, 0.05, 0],
      head: [0.35, 0, 0],
      hips: [0, 0, 0, 0, -0.16, 0.35],
      hipL: [-0.6],
      hipR: [0.5],
      kneeR: [0.6]
    };
    return [
      [0, {}],
      [10, lerpPose({}, lift, 0.6)],
      [16, lift],
      [44, held],
      [46, held],
      [49, hit],
      // The 38-tick punish window: he stays bent over the blade.
      [Math.round(total * 0.75), { ...hit, spine: [0.55, 0, 0] }],
      [total, {}]
    ];
  },
  // The glints build slowly while the blade is held, so the delay reads as a tell.
  glow: (f) => (f < 46 ? 1 + (f / 46) * 1.8 : Math.max(1, 2.8 - (f - 46) * 0.12))
};

// The dagger stab: the dagger hand drops, the body turns side-on (frames 0 to 14), then a step
// and a straight thrust (frame 16).
const STAB: Choreo = {
  total: 58,
  keys: (total) =>
    attack(16, 18, total, {
      wind: { shoulderL: [0.3, 0, 0.7], elbowL: [-1.6], spine: [0, -0.7, 0], hips: [0, -0.3, 0, 0, -0.06, -0.1], shoulderR: [-0.7, 0, -0.5] },
      strike: { shoulderL: [-1.5, 0, 0.1], elbowL: [-0.2], spine: [0.3, 0.45, 0], hips: [0, 0.3, 0, 0, -0.1, 0.3], hipL: [-0.8], hipR: [0.6] }
    }),
  glow: glint(16)
};

// The guard stance: the greatsword stands up in front of him (frames 0 to 14) and is held up
// to frame 60; then, with nothing struck, a shoulder shove (frame 64).
const GUARD: Choreo = {
  total: 98,
  keys: (total): Keyframes => {
    const up: Pose = { shoulderR: [-1.9, 0, 0.1], elbowR: [-1.1], shoulderL: [-1.3, 0, 0.35], elbowL: [-1.0], spine: [-0.08, -0.15, 0], head: [-0.05, 0, 0], hipL: [-0.3], hipR: [0.2], kneeR: [0.25], hips: [0, 0, 0, 0, -0.08, 0] };
    const shove: Pose = { shoulderR: [-1.5, 0, 0.1], elbowR: [-0.8], spine: [0.4, 0.3, 0], hips: [0, 0.2, 0, 0, -0.12, 0.25], hipL: [-0.5], hipR: [0.4] };
    return [
      [0, {}],
      [8, lerpPose({}, up, 0.7)],
      [14, up],
      [60, up],
      [64, shove],
      [Math.round(total * 0.8), { ...shove, spine: [0.3, 0.2, 0] }],
      [total, {}]
    ];
  },
  // A steady shimmer in the stance.
  glow: (f) => (f >= 14 && f <= 60 ? 1.8 + Math.sin(f * 0.5) * 0.3 : glint(64)(f))
};

// The counter: the blade whips from the guard through a backhand (frame 16).
const COUNTER: Choreo = {
  total: 72,
  keys: (total) =>
    attack(16, 20, total, {
      wind: { shoulderR: [-1.6, 0, 1.0], elbowR: [-0.3], spine: [0, 0.9, 0], head: [0, 0.3, 0] },
      strike: { shoulderR: [-1.5, 0, -1.1], elbowR: [-0.1], spine: [0.15, -0.9, 0], hips: [0, -0.4, 0, 0, -0.06, 0.2] }
    }),
  glow: glint(16)
};

// The leaping slam: a deep crouch (frames 0 to 24), the leap (24 to 32), and the blade driven
// into the floor (frame 32).
const LEAP: Choreo = {
  total: 80,
  keys: (total): Keyframes => {
    const crouch: Pose = { hipL: [-0.9], hipR: [-0.9], kneeL: [1.5], kneeR: [1.5], spine: [0.35, 0, 0], hips: [0, 0, 0, 0, -0.4, 0], shoulderR: [-2.4, 0, -0.2], elbowR: [-0.4], shoulderL: [-1.0, 0, 0.6] };
    const air: Pose = { hipL: [-0.4], hipR: [0.2], kneeL: [0.8], kneeR: [0.4], spine: [-0.1, 0, 0], hips: [0, 0, 0, 0, 0.3, 0], shoulderR: [-3.0, 0, -0.15], elbowR: [-0.3], shoulderL: [-1.2, 0, 0.6] };
    const slam: Pose = { hipL: [-0.7], hipR: [-0.5], kneeL: [0.9], kneeR: [0.7], spine: [0.75, 0, 0], head: [0.3, 0, 0], hips: [0, 0, 0, 0, -0.3, 0.1], shoulderR: [-1.0, 0, -0.05], elbowR: [-0.05], shoulderL: [-0.9, 0, 0.5] };
    return [
      [0, {}],
      [10, lerpPose({}, crouch, 0.7)],
      [22, crouch],
      [24, crouch],
      [28, air],
      [32, slam],
      [36, slam],
      [Math.round(total * 0.75), { ...slam, spine: [0.5, 0, 0] }],
      [total, {}]
    ];
  },
  glow: glint(32)
};

// The phase change: the blade's tip to the floor, both hands on the pommel, the head bowed while
// the rune ring brightens.
const OVERFIT: Choreo = {
  total: 45,
  keys: (): Keyframes => {
    const bow: Pose = { shoulderR: [-0.9, 0, -0.2], elbowR: [-0.4], shoulderL: [-0.9, 0, 0.1], elbowL: [-0.5], spine: [0.3, 0, 0], head: [0.5, 0, 0] };
    const rise: Pose = { ...bow, spine: [-0.15, 0, 0], head: [-0.35, 0, 0], shoulderR: [-1.0, 0, -0.2] };
    return [
      [0, {}],
      [14, bow],
      [30, bow],
      [40, rise],
      [45, rise]
    ];
  },
  glow: (f) => 1 + 2.4 * Math.min(f / 30, 1)
};

// The wide sweeps: the blade drawn out to one side (frames 0 to 20), then a long flat cut
// across the whole arc (frame 22). `side` is which way the blade reaches, in his own frame.
const sweep = (side: 1 | -1): Choreo => ({
  total: 74,
  keys: (total) =>
    attack(22, 28, total, {
      wind: { shoulderR: [-1.5, 0, side * 1.6], elbowR: [-0.2], spine: [0, side * 1.0, 0], head: [0, side * 0.4, 0], shoulderL: [-0.8, 0, 0.7] },
      strike: { shoulderR: [-1.5, 0, -side * 1.0], elbowR: [-0.05], spine: [0.1, -side * 1.1, 0], head: [0, -side * 0.3, 0], hips: [0, -side * 0.4, 0, 0, -0.06, 0.1] }
    }),
  glow: glint(22)
});

// The lunge: the blade levels and he sinks (frames 0 to 20), then a long thrust (frame 22).
const LUNGE: Choreo = {
  total: 66,
  keys: (total) =>
    attack(22, 28, total, {
      wind: { shoulderR: [-1.45, 0, -0.2], elbowR: [-1.2], spine: [0.1, -0.5, 0], hipL: [0.5], hipR: [-0.4], kneeL: [0.7], hips: [0, 0, 0, 0, -0.25, -0.1], shoulderL: [-0.8, 0, 0.7] },
      strike: { shoulderR: [-1.55, 0, 0.0], elbowR: [-0.05], spine: [0.55, 0.15, 0], hipL: [-1.0], hipR: [0.8], kneeR: [0.5], hips: [0, 0, 0, 0, -0.22, 0.55] }
    }),
  glow: glint(22)
};

// The parry counter: a quick feint-cut (frame 14), then the blade held back (frames 17 to 39)
// and a late cut (frame 40).
const DELAY_1: Choreo = {
  total: 34,
  keys: (total) =>
    attack(14, 17, total, {
      wind: { shoulderR: [-1.8, 0, -0.7], elbowR: [-0.5], spine: [0, -0.6, 0], shoulderL: [-0.7, 0, 0.8] },
      strike: { shoulderR: [-1.4, 0, 0.6], elbowR: [-0.1], spine: [0.1, 0.6, 0] }
    }),
  glow: glint(14)
};

const DELAY_2: Choreo = {
  total: 80,
  keys: (total): Keyframes => {
    const held: Pose = { shoulderR: [-2.6, 0, 0.5], elbowR: [-0.7], spine: [-0.3, 0.55, 0], shoulderL: [-0.8, 0, 0.7], head: [-0.1, 0, 0] };
    const hit: Pose = { shoulderR: [-1.2, 0, -0.1], elbowR: [-0.1], spine: [0.6, -0.15, 0], head: [0.3, 0, 0], hips: [0, 0, 0, 0, -0.14, 0.3], hipL: [-0.5], hipR: [0.4] };
    return [
      [0, {}],
      [12, held],
      [38, held],
      [40, hit],
      [44, hit],
      [Math.round(total * 0.75), { ...hit, spine: [0.45, 0, 0] }],
      [total, {}]
    ];
  },
  glow: (f) => (f < 40 ? 1 + (f / 40) * 1.6 : Math.max(1, 2.6 - (f - 40) * 0.12))
};

// The guard-break kick: a flash (frame 8), the knee comes up (frames 8 to 20), and the heel
// drives out (frame 22).
const KICK: Choreo = {
  total: 64,
  keys: (total) =>
    attack(22, 25, total, {
      ready: { spine: [-0.1, 0, 0], shoulderR: [-1.0, 0, -0.4], elbowR: [-1.0] },
      wind: { hipL: [-1.5], kneeL: [1.6], spine: [-0.35, 0, 0], hips: [0, 0, 0, 0, -0.05, -0.1], shoulderR: [-1.0, 0, -0.5], shoulderL: [-0.6, 0, 0.9] },
      strike: { hipL: [-1.55], kneeL: [0.05], spine: [-0.5, 0, 0], hips: [0, 0, 0, 0, -0.04, 0.1], shoulderR: [-0.9, 0, -0.6], shoulderL: [-0.5, 0, 0.9] }
    }),
  // The unblockable flash: a bright pulse at frame 8 that decays by the time the heel lands.
  glow: (f) => 1 + Math.max(0, 3.4 - Math.abs(f - 8) * 0.35)
};

// The closing dash: he drops low (frames 0 to 16), covers the distance (16 to 30), and cuts as
// he arrives (frame 30).
const DASH: Choreo = {
  total: 76,
  keys: (total) =>
    attack(30, 34, total, {
      ready: { spine: [0.3, 0, 0], hipL: [-0.5], hipR: [-0.5], kneeL: [1.1], kneeR: [1.1], hips: [0, 0, 0, 0, -0.3, 0] },
      wind: { spine: [0.55, -0.4, 0], hipL: [-0.9], hipR: [0.6], kneeL: [0.9], kneeR: [0.4], shoulderR: [-1.0, 0, -1.2], elbowR: [-0.3], hips: [0, 0, 0, 0, -0.2, 0.5] },
      strike: { spine: [0.2, 0.8, 0], shoulderR: [-1.5, 0, 0.9], elbowR: [-0.1], hipL: [-0.6], hipR: [0.5], hips: [0, 0.3, 0, 0, -0.1, 0.3] }
    }),
  glow: glint(30)
};

export const vossDef: ModelDef = {
  id: 'b07_castellan_voss',
  height: 2.5,
  build,
  idle,
  walk,
  stride: 1.9,
  // Substring keys, tried in order; the first is also the fallback.
  moves: {
    gs_1: GS_1,
    gs_2: GS_2,
    gs_3: GS_3,
    stab: STAB,
    guard: GUARD,
    counter: COUNTER,
    leap_slam: LEAP,
    overfit: OVERFIT,
    sweep_pl: sweep(1),
    sweep_pr: sweep(-1),
    lunge: LUNGE,
    delay_1: DELAY_1,
    delay_2: DELAY_2,
    kick: KICK,
    dash: DASH
  },
  fallback: GS_1,
  fallAxis: 'x',
  // He kneels, the blade clattering from his hand, and topples back.
  death: (u) => ({
    spine: [0.3 * u],
    head: [0.55 * u, 0, 0, 0, -0.08 * u, 0.06 * u],
    kneeL: [1.2 * u],
    kneeR: [1.0 * u],
    hipL: [-0.5 * u],
    shoulderL: [0.3 * u],
    shoulderR: [0.6 * u]
  })
};
