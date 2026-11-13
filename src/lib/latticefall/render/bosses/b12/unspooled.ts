// B12, the Unspooled (`b12_the_unspooled_body`, registry 41001): the thing that pulled the
// thread. Spec: docs/game/05-enemies-and-bosses.md, "B12 The Unspooled". One body with two
// shapes:
//
//   - Phase 1, the Weaver: a tall, thin figure wound out of pale threads, with a loom-light in its
//     chest and long thread lashes trailing from its hands.
//   - Phase 2, the Unspooled: the Weaver comes apart during `b12_unravel` (the 45-tick rebirth)
//     and reforms larger, with six extra thread limbs fanned out from its back.
//
// The renderer never reads the boss's phase from the simulation. The shape follows the moves
// it plays (`b12_unravel` and every phase-2 move switch it to the Unspooled; `b12_stitch` and
// `b12_loom`, phase-1 only, switch it back), and the loom effects call `resetUnspooled` when the
// core says the fight ended or reset (the Loom event `CLEARED`). Moves (game/data/moves/bosses/b12.ron): lash (hit on 20), lash_2 (26), loom (lines
// on 6, burst on 30), stitch (white flash from 4, grab on 28), spin (28 to 43), unravel, flurry
// (22), flurry_2 (14), flurry_3 (24), loom_storm (6 and 30); the borrowed moves (b06_cantor_beam,
// b07_leap_slam, b09_gnd_flare, b10_flail_slam) use the matching generic strike below.

import { biped, bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, type Choreo } from '../../procedural/pose';
import type { Pose, Rig } from '../../procedural/rig';
import type { Object3D } from 'three';

/** The body's registry number (`b12_the_unspooled_body`). */
export const B12_KIND = 41001;

const C = {
  thread: 0xe9e1cf,
  threadDim: 0xb8ad98,
  core: 0x1d1a2b,
  loom: 0xffd98a,
  white: 0xffffff,
  limb: 0xcfc4e8
};

/** How much larger the Unspooled is than the Weaver. */
const P2_SCALE = 1.35;
/** Extra limbs of the Unspooled. */
const LIMBS = 6;

/**
 * Which shape each body shows, by rig. A module-level switch the engine's Boss events set too: on a
 * reset every B12 rig goes back to the Weaver.
 */
const unspooled = new WeakMap<Rig, boolean>();
let resetEpoch = 0;
const seenEpoch = new WeakMap<Rig, number>();

/** Called by the loom effects when the fight ends or resets: every B12 body is the Weaver again. */
export function resetUnspooled(): void {
  resetEpoch++;
}

function build(): Rig {
  const rig = biped({ leg: 1.15, torso: 0.85, neck: 0.14, shoulder: 0.24, hip: 0.1, upperArm: 0.5, foreArm: 0.55 });
  // The body is wound thread: thin wrapped cylinders over a dark core.
  rig.box('spine', [0.26, 0.82, 0.18], [0, 0.4, 0], C.core);
  for (let i = 0; i < 7; i++) {
    rig.arc('spine', 0.2 - Math.abs(i - 3) * 0.012, 0.014, Math.PI * 2, [0, 0.08 + i * 0.11, 0], C.thread, { rot: [Math.PI / 2 + (i % 2 ? 0.2 : -0.2), 0, 0] });
  }
  // The loom-light in the chest.
  rig.sphere('spine', 0.1, [0, 0.55, 0.1], C.loom, { glow: 2.2 });
  rig.cyl('hips', 0.12, 0.22, 0.4, [0, -0.18, 0], C.threadDim);
  // A head like a spindle, with a slit of light.
  rig.cone('head', 0.13, 0.42, [0, 0.2, 0], C.thread);
  rig.box('head', [0.18, 0.025, 0.05], [0, 0.16, 0.1], C.loom, { glow: 2 });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.05, 0.06, 0.5, [0, -0.25, 0], C.thread);
    rig.cyl(`elbow${side}`, 0.035, 0.05, 0.55, [0, -0.27, 0], C.thread);
    rig.cyl(`hip${side}`, 0.06, 0.08, 1.15, [0, -0.57, 0], C.threadDim);
    rig.cyl(`knee${side}`, 0.04, 0.06, 1.15, [0, -0.57, 0], C.threadDim);
    // The lash: a long glowing thread from each hand.
    rig.cyl(`hand${side}`, 0.012, 0.012, 1.6, [0, -0.8, 0], C.loom, { glow: 1.4 });
  }
  // The Unspooled's extra limbs, hidden until the rebirth: thread arms fanned from the back.
  rig.bone('limbs', 'spine', [0, 0.6, -0.12]);
  for (let i = 0; i < LIMBS; i++) {
    const a = ((i + 0.5) / LIMBS) * Math.PI - Math.PI / 2;
    rig.cyl('limbs', 0.03, 0.06, 1.9, [Math.sin(a) * 0.95, 0.15 + Math.cos(a) * 0.55, -0.15], C.limb, { rot: [0.3, 0, -a * 1.1], glow: 0.9 });
  }
  // The loose threads of the unravel: a cloud of strands around the body, hidden but for it.
  rig.bone('strands', null, [0, 1.6, 0]);
  for (let i = 0; i < 16; i++) {
    const a = (i / 16) * Math.PI * 2;
    rig.cyl('strands', 0.01, 0.01, 1.4, [Math.sin(a) * 0.6, (i % 4) * 0.25 - 0.4, Math.cos(a) * 0.6], C.thread, { rot: [a, 0, 0.7], glow: 1.2 });
  }
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 0.8) * 0.03, 0],
  spine: [0.06 + Math.sin(t * 0.8) * 0.02, Math.sin(t * 0.3) * 0.08, 0],
  head: [0.1, Math.sin(t * 0.45) * 0.2, Math.sin(t * 0.7) * 0.05],
  shoulderL: [-0.2, 0, 0.35],
  elbowL: [-0.4, 0, 0],
  shoulderR: [-0.2, 0, -0.35],
  elbowR: [-0.4, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.15);
  return { ...g, shoulderL: [-0.2, 0, 0.35], shoulderR: [-0.2, 0, -0.35] };
};

/** Thread lash: the right arm draws back, then cracks forward like a whip. */
const lash: Choreo = {
  total: 34,
  keys: (total) =>
    attack(20, 23, total, {
      wind: { shoulderR: [-2.6, 0, -0.6], elbowR: [-1.2], spine: [-0.15, 0.4, 0], head: [0, 0.3, 0] },
      strike: { shoulderR: [-1.4, 0, 0.3], elbowR: [-0.05], spine: [0.15, -0.3, 0] }
    }),
  glow: (f) => (f < 20 ? 1 + (f / 20) * 1.6 : 1.2)
};

/** The second lash, delayed: the left arm holds high for 26 frames, then lashes across. */
const lash2: Choreo = {
  total: 64,
  keys: (total) =>
    attack(26, 29, total, {
      wind: { shoulderL: [-2.9, 0, 0.5], elbowL: [-1.0], spine: [-0.2, -0.4, 0], head: [-0.1, -0.3, 0] },
      strike: { shoulderL: [-1.3, 0, -0.3], elbowL: [-0.05], spine: [0.2, 0.35, 0] }
    }),
  glow: (f) => (f < 26 ? 1 + (f / 26) * 1.8 : 1.2)
};

/** Loom lines: both arms rise and spread, the chest-light flares, a burst on frame 30. */
const loom: Choreo = {
  total: 74,
  keys: (total) =>
    attack(30, 33, total, {
      ready: { shoulderL: [-1.0, 0, 1.0], shoulderR: [-1.0, 0, -1.0] },
      wind: { shoulderL: [-2.8, 0, 0.9], shoulderR: [-2.8, 0, -0.9], spine: [-0.25, 0, 0], head: [-0.4, 0, 0] },
      strike: { shoulderL: [-0.6, 0, 1.4], shoulderR: [-0.6, 0, -1.4], spine: [0.2, 0, 0] }
    }),
  glow: (f) => (f < 30 ? 1.4 + (f / 30) * 2 : Math.max(1, 3.4 - (f - 30) * 0.06))
};

/** Stitching grab: a white flash for 24 frames, then both hands snap shut in front. */
const stitch: Choreo = {
  total: 74,
  keys: (total) =>
    attack(28, 33, total, {
      wind: { shoulderL: [-1.6, 0, 1.2], shoulderR: [-1.6, 0, -1.2], elbowL: [-0.3], elbowR: [-0.3], spine: [-0.2, 0, 0] },
      strike: { shoulderL: [-1.6, 0, -0.1], shoulderR: [-1.6, 0, 0.1], elbowL: [-0.1], elbowR: [-0.1], spine: [0.3, 0, 0], head: [0.2, 0, 0] }
    }),
  glow: (f) => (f >= 4 && f < 28 ? 3.2 : 1)
};

/** Spinning cut: wound up for 28 frames, then one full turn with both lashes out. */
const spin: Choreo = {
  total: 80,
  keys: (total) =>
    attack(28, 43, total, {
      wind: { shoulderL: [-0.3, 0, 1.0], shoulderR: [-0.3, 0, -1.0], spine: [0, 1.2, 0], head: [0, -0.6, 0] },
      strike: { shoulderL: [-0.2, 0, 1.55], shoulderR: [-0.2, 0, -1.55], spine: [0, -1.0, 0] }
    }),
  glow: (f) => (f < 28 ? 1 + (f / 28) * 1.2 : 2)
};

/** The rebirth: the figure sags and comes apart, then rises reformed. */
const unravel: Choreo = {
  total: 45,
  keys: () => [
    [0, {}],
    [14, { spine: [0.7, 0, 0], head: [0.6, 0, 0], shoulderL: [0.3, 0, 0.2], shoulderR: [0.3, 0, -0.2], kneeL: [0.6], kneeR: [0.6] }],
    [30, { spine: [0.4, 0, 0], head: [0.2, 0, 0], shoulderL: [-1.5, 0, 1.3], shoulderR: [-1.5, 0, -1.3] }],
    [45, { spine: [-0.1, 0, 0], shoulderL: [-0.5, 0, 0.6], shoulderR: [-0.5, 0, -0.6] }]
  ],
  glow: (f) => (f < 30 ? 0.6 + f * 0.05 : 3)
};

/** The flurry: the limbs rear and stab down, side to side; the last strike brings them all. */
const flurry = (hit: number, total: number, side: number): Choreo => ({
  total,
  keys: (t) =>
    attack(hit, hit + 3, t, {
      wind: { shoulderL: [-2.4 * (side >= 0 ? 1 : 0.4), 0, 0.6], shoulderR: [-2.4 * (side <= 0 ? 1 : 0.4), 0, -0.6], spine: [-0.3, 0.2 * side, 0] },
      strike: { shoulderL: [-1.0, 0, 0.2], shoulderR: [-1.0, 0, -0.2], spine: [0.4, -0.2 * side, 0], head: [0.3, 0, 0] }
    }),
  glow: (f) => (f < hit ? 1 + (f / hit) * 1.5 : 1.3)
});

/** A generic heavy strike for the borrowed moves: the frames follow each move's own hit. */
const borrowed = (hit: number, total: number): Choreo => ({
  total,
  keys: (t) =>
    attack(hit, hit + 4, t, {
      wind: { shoulderL: [-2.7, 0, 0.5], shoulderR: [-2.7, 0, -0.5], spine: [-0.3, 0, 0], head: [-0.3, 0, 0] },
      strike: { shoulderL: [-0.9, 0, 0.3], shoulderR: [-0.9, 0, -0.3], spine: [0.45, 0, 0] }
    }),
  glow: (f) => (f < hit ? 1 + (f / hit) * 2 : 1.5)
});

const PHASE2 = ['flurry', 'loom_storm', 'cantor_beam', 'leap_slam', 'gnd_flare', 'flail_slam', 'unravel'];
const PHASE1_ONLY = ['b12_stitch', 'b12_loom'];

function tick(rig: Rig, ctx: { frame: number; inMove: boolean; anim: string }): void {
  const move = ctx.inMove ? ctx.anim : '';
  const f = ctx.frame;
  if ((seenEpoch.get(rig) ?? 0) !== resetEpoch) {
    seenEpoch.set(rig, resetEpoch);
    unspooled.set(rig, false);
  }
  let big = unspooled.get(rig) ?? false;
  if (move === 'b12_unravel') big = f >= 24;
  else if (PHASE2.some((k) => move.includes(k))) big = true;
  else if (PHASE1_ONLY.includes(move)) big = false;
  unspooled.set(rig, big);
  // The rebirth: the strands show as it comes apart, and the body thins out in the middle.
  const strands = rig.bones.get('strands');
  if (strands) {
    const unravelling = move === 'b12_unravel';
    strands.visible = unravelling;
    if (unravelling) {
      strands.rotation.y = f * 0.12;
      const spread = 1 + Math.sin((Math.min(f, 45) / 45) * Math.PI) * 1.6;
      strands.scale.set(spread, 1, spread);
    }
  }
  rig.setOpacity(move === 'b12_unravel' ? 0.35 + Math.abs(f - 22) / 22 * 0.65 : 1);
  const limbs = rig.bones.get('limbs');
  if (limbs) {
    limbs.visible = big;
    if (big) {
      // The limbs sway, and rear with the flurry.
      const rear = move.includes('flurry') ? 0.5 : 0;
      limbs.children.forEach((c: Object3D, i: number) => {
        c.rotation.x = 0.3 + Math.sin(f * 0.15 + i) * 0.12 - rear;
      });
    }
  }
  const s = big ? P2_SCALE : 1;
  rig.root.scale.set(s, s, s);
}

const dims = {
  height: 3.0,
  stride: 1.7,
  idle,
  walk,
  moves: {
    b12_lash_2: lash2,
    b12_lash: lash,
    b12_loom_storm: { ...loom, total: 74 },
    b12_loom: loom,
    b12_stitch: stitch,
    b12_spin: spin,
    b12_unravel: unravel,
    b12_flurry_3: flurry(24, 66, 0),
    b12_flurry_2: flurry(14, 22, -1),
    b12_flurry: flurry(22, 30, 1),
    // The borrowed moves (their anims are their own bosses'); hit frames from those files.
    cantor_beam: borrowed(30, 102),
    leap_slam: borrowed(30, 70),
    gnd_flare: borrowed(36, 100),
    flail_slam: borrowed(24, 64)
  },
  fallback: lash,
  fallAxis: 'x' as const,
  tick,
  death: (u: number): Pose => ({ spine: [0.6 * u], head: [0.5 * u], shoulderL: [0.4 * u, 0, 0.3 * u], shoulderR: [0.4 * u, 0, -0.3 * u], kneeL: [0.8 * u], kneeR: [0.7 * u] })
};

export const unspooledDef: ModelDef = { id: 'b12_the_unspooled_body', build, ...dims };
