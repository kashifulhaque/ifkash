// B03, Bridgewarden Corvane (boss id `b03_bridgewarden_corvane`, registry 32010; body 32011):
// a knight in gray plate who holds the bridge out of habit. A tall halberd in the right hand, a
// kite shield on the left arm, a slate cloak, and a worn crimson plume. Spec:
// docs/game/05-enemies-and-bosses.md, "B03 Bridgewarden Corvane". Choreography is matched on
// words in the move's `anim` (see game/data/moves/bosses/b03.ron): corvane_shield_advance,
// corvane_combo_1 to _3, corvane_sweep_turn, corvane_lunge_shout. Strike frames follow the
// move data: the first cut lands on frame 18, the second on 16, the delayed third on 36 (the
// halberd is lifted and held for 30 frames), the sweep on 26, the shield shove on 20, and the
// lunge on 18 after the shout. Render-only.

import { biped, bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, lerpPose, type Choreo, type Keyframes } from '../../procedural/pose';
import type { Pose, Rig } from '../../procedural/rig';

const STEEL = 0x59606e;
const DARK = 0x2e333d;
const TRIM = 0x8d92a0;
const CLOAK = 0x3a4963;
const PLUME = 0x8c3a3a;
const LEATHER = 0x4a3b30;
const EYE = 0xf0c870;

function build(): Rig {
  const rig = biped({ leg: 1.18, torso: 0.9, neck: 0.15, shoulder: 0.46, hip: 0.19, upperArm: 0.52, foreArm: 0.52 });
  rig.box('hips', [0.58, 0.3, 0.38], [0, 0.0, 0], DARK);
  // A faulds skirt of overlapping plates.
  for (const x of [-0.2, 0, 0.2]) rig.box('hips', [0.19, 0.42, 0.05], [x, -0.3, 0.2], STEEL);
  rig.box('hips', [0.5, 0.42, 0.05], [0, -0.3, -0.2], CLOAK);
  // Cuirass, a gorget, and a belt.
  rig.box('spine', [0.78, 0.82, 0.46], [0, 0.46, 0], STEEL);
  rig.box('spine', [0.84, 0.1, 0.5], [0, 0.04, 0], LEATHER);
  rig.box('spine', [0.5, 0.5, 0.05], [0, 0.5, 0.25], TRIM);
  rig.cyl('spine', 0.2, 0.26, 0.16, [0, 0.93, 0], DARK);
  // The slate cloak hangs from the shoulders down the back.
  rig.box('spine', [0.86, 1.25, 0.05], [0, 0.35, -0.28], CLOAK);
  rig.box('spine', [0.92, 0.12, 0.2], [0, 0.88, -0.17], CLOAK);
  // A close helm with a visor slit, an eye glint, and a plume.
  rig.box('head', [0.4, 0.46, 0.42], [0, 0.2, 0], STEEL);
  rig.box('head', [0.34, 0.05, 0.05], [0, 0.24, 0.215], 0x050506);
  rig.sphere('head', 0.03, [0, 0.24, 0.205], EYE, { glow: 1.2 });
  rig.box('head', [0.06, 0.2, 0.3], [0, 0.5, -0.04], TRIM);
  rig.cone('head', 0.08, 0.5, [0, 0.62, -0.28], PLUME, { rot: [-1.0, 0, 0] });
  for (const side of ['L', 'R'] as const) {
    const s = side === 'L' ? 1 : -1;
    rig.sphere(`shoulder${side}`, 0.25, [s * 0.04, 0.0, 0], STEEL, { squash: [1.15, 0.85, 1.05] });
    rig.cyl(`shoulder${side}`, 0.11, 0.1, 0.52, [0, -0.26, 0], DARK);
    rig.cyl(`elbow${side}`, 0.11, 0.12, 0.52, [0, -0.26, 0], STEEL);
    rig.box(`hand${side}`, [0.19, 0.19, 0.19], [0, 0, 0], LEATHER);
    rig.cyl(`hip${side}`, 0.15, 0.11, 0.58, [0, -0.29, 0], DARK);
    rig.sphere(`knee${side}`, 0.12, [0, 0, 0], TRIM);
    rig.cyl(`knee${side}`, 0.11, 0.1, 0.58, [0, -0.29, 0], STEEL);
    rig.box(`foot${side}`, [0.23, 0.15, 0.44], [0, 0.07, 0.1], DARK);
  }
  // A kite shield on the left forearm, carried forward at the guard.
  rig.bone('shield', 'elbowL', [0, -0.28, 0]);
  rig.restRot('shield', [0.9, 0, 0]);
  rig.box('shield', [0.62, 0.95, 0.07], [0, 0.0, 0.2], STEEL);
  rig.box('shield', [0.5, 0.82, 0.04], [0, 0.0, 0.25], CLOAK);
  rig.box('shield', [0.1, 0.8, 0.05], [0, 0.0, 0.28], TRIM);
  rig.box('shield', [0.5, 0.1, 0.05], [0, 0.18, 0.28], TRIM);
  rig.cone('shield', 0.31, 0.3, [0, -0.62, 0.2], STEEL, { rot: [Math.PI, 0, 0] });
  // A halberd: a long shaft, an axe blade, a spike on top, and a hook behind.
  rig.bone('weapon', 'handR', [0, -0.05, 0.03]);
  rig.restRot('weapon', [-0.5, 0, 0]);
  rig.cyl('weapon', 0.035, 0.035, 2.6, [0, -0.7, 0], LEATHER);
  rig.cyl('weapon', 0.05, 0.05, 0.12, [0, 0.55, 0], TRIM);
  rig.box('weapon', [0.04, 0.6, 0.5], [0, -1.78, 0.22], TRIM);
  rig.box('weapon', [0.05, 0.3, 0.22], [0, -1.7, 0.4], 0x9aa0ac);
  rig.cone('weapon', 0.06, 0.4, [0, -2.2, 0], 0x9aa0ac, { rot: [Math.PI, 0, 0] });
  rig.cone('weapon', 0.04, 0.26, [0, -1.82, -0.2], TRIM, { rot: [-1.9, 0, 0] });
  rig.box('weapon', [0.12, 0.12, 0.12], [0, 0.0, 0], TRIM);
  return rig;
}

const GUARD_S = -0.45;
const GUARD_E = -1.3;

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 0.9) * 0.012, 0],
  spine: [Math.sin(t * 0.9) * 0.015, -0.18, 0],
  head: [0.02, 0.18 + Math.sin(t * 0.4) * 0.05, 0],
  shoulderL: [GUARD_S, 0, 0.2],
  elbowL: [GUARD_E, 0, 0],
  shoulderR: [-0.35, 0, -0.25],
  elbowR: [-0.6, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.2);
  delete g.shoulderL;
  return g;
};

/** The eye glint brightens through a wind-up and peaks at the strike. */
const glint = (windEnd: number) => (f: number) => {
  if (f < windEnd) return 1 + 1.6 * Math.min(f / windEnd, 1) ** 2;
  return Math.max(1, 2.6 - (f - windEnd) * 0.1);
};

// Diagonal chop, down and across (frame 18), then the backhand back the other way (frame 16).
const COMBO_1: Choreo = {
  total: 38,
  keys: (total) =>
    attack(18, 21, total, {
      wind: { shoulderR: [-2.1, 0, -1.0], elbowR: [-0.5], spine: [-0.1, -0.75, 0], shoulderL: [-1.0, 0, 0.5] },
      strike: { shoulderR: [-1.4, 0, 0.85], elbowR: [-0.1], spine: [0.1, 0.8, 0], hips: [0, 0.3, 0, 0, -0.04, 0.2] }
    }),
  glow: glint(18)
};

const COMBO_2: Choreo = {
  total: 34,
  keys: (total) =>
    attack(16, 19, total, {
      wind: { shoulderR: [-1.5, 0, 0.9], elbowR: [-0.3], spine: [0, 0.8, 0], shoulderL: [-1.0, 0, 0.5] },
      strike: { shoulderR: [-1.5, 0, -0.95], elbowR: [-0.1], spine: [0.1, -0.8, 0], hips: [0, -0.3, 0, 0, -0.04, 0.2] }
    }),
  glow: glint(16)
};

/** The delayed third hit: the halberd is lifted high and held (frames 14 to 35) before it falls. */
const COMBO_3: Choreo = {
  total: 72,
  keys: (total): Keyframes => {
    const lift: Pose = { shoulderR: [-3.0, 0, -0.25], elbowR: [-0.5], shoulderL: [-1.0, 0, 0.5], spine: [-0.45, -0.1, 0], head: [-0.2, 0, 0] };
    const held: Pose = { ...lift, spine: [-0.5, -0.1, 0], shoulderR: [-3.1, 0, -0.2] };
    const hit: Pose = {
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
    return [
      [0, {}],
      [10, lerpPose({}, lift, 0.6)],
      [16, lift],
      [34, held],
      [36, held],
      [39, hit],
      // The 32-tick punish window: he stays bent over the halberd.
      [Math.round(total * 0.75), { ...hit, spine: [0.55, 0, 0] }],
      [total, {}]
    ];
  },
  // The glint builds slowly while the halberd is held, so the delay reads as a tell.
  glow: (f) => (f < 36 ? 1 + (f / 36) * 1.8 : Math.max(1, 2.8 - (f - 36) * 0.12))
};

// The sweeping turn: the halberd winds round behind him (frame 26), then spins through.
const SWEEP: Choreo = {
  total: 72,
  keys: (total) =>
    attack(26, 31, total, {
      wind: { shoulderR: [-1.4, 0, -1.55], elbowR: [-0.4], spine: [0, -1.1, 0], head: [0, 0.5, 0], shoulderL: [-1.2, 0, 0.5] },
      strike: { shoulderR: [-1.5, 0, 0.9], elbowR: [-0.05], spine: [0.1, 1.0, 0], head: [0, -0.3, 0], hips: [0, 0.4, 0, 0, -0.06, 0.1] }
    }),
  glow: glint(26)
};

// The shield advance: the shield comes up, he steps in behind it, and shoves (frame 20).
const SHIELD: Choreo = {
  total: 66,
  keys: (total) =>
    attack(20, 23, total, {
      wind: { shoulderL: [-1.0, 0, 0.7], elbowL: [-1.6], spine: [-0.1, 0.45, 0], hips: [0, 0, 0, 0, -0.06, -0.05], shoulderR: [-0.5, 0, -0.5] },
      strike: { shoulderL: [-1.75, 0, 0.0], elbowL: [-0.4], spine: [0.35, -0.35, 0], hips: [0, 0, 0, 0, -0.1, 0.3], hipL: [-0.5], hipR: [0.4] }
    }),
  glow: glint(20)
};

// The healing punish: the head tips back for the shout (frames 2 to 14, the eye flares, the
// halberd is drawn back), then the lunge: a long step and a thrust (frame 18).
const LUNGE: Choreo = {
  total: 78,
  keys: (total) =>
    attack(18, 27, total, {
      ready: { head: [-0.6, 0, 0], spine: [-0.3, -0.1, 0], shoulderR: [-0.8, 0, -0.4], elbowR: [-0.7], shoulderL: [-0.9, 0, 0.6] },
      wind: { head: [0.1, 0, 0], spine: [0.1, -0.5, 0], shoulderR: [-1.2, 0, -0.5], elbowR: [-1.2], shoulderL: [-0.9, 0, 0.6], hipL: [0.5], hipR: [-0.4], kneeL: [0.6], hips: [0, 0, 0, 0, -0.22, -0.1] },
      strike: { head: [0.2, 0, 0], spine: [0.55, 0.15, 0], shoulderR: [-1.55, 0, 0.05], elbowR: [-0.1], shoulderL: [-0.6, 0, 0.5], hipL: [-0.9], hipR: [0.7], kneeR: [0.5], hips: [0, 0, 0, 0, -0.2, 0.5] }
    }),
  // The shout: a bright flare in the first 14 frames, then the lunge's glint.
  glow: (f) => (f < 14 ? 1 + 3.2 * Math.sin((Math.min(f, 14) / 14) * Math.PI * 0.5) : Math.max(1, 3 - (f - 14) * 0.15))
};

export const corvaneDef: ModelDef = {
  id: 'b03_bridgewarden_corvane',
  height: 2.7,
  build,
  idle,
  walk,
  stride: 2.0,
  // Substring keys, tried in order; the first is also the fallback.
  moves: {
    combo_1: COMBO_1,
    combo_2: COMBO_2,
    combo_3: COMBO_3,
    sweep: SWEEP,
    turn: SWEEP,
    shield: SHIELD,
    lunge: LUNGE,
    shout: LUNGE
  },
  fallback: COMBO_1,
  fallAxis: 'x',
  // He kneels first, then topples back with the halberd across him.
  death: (u) => ({
    spine: [0.25 * u],
    head: [0.5 * u, 0, 0, 0, -0.08 * u, 0.06 * u],
    kneeL: [1.2 * u],
    kneeR: [1.0 * u],
    hipL: [-0.5 * u],
    shoulderL: [0.3 * u],
    shoulderR: [0.5 * u]
  })
};
