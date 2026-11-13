// B10, the Bellwright (boss id `b10_the_bellwright`, body registry 39011): a giant smith who hauls
// a frozen bell on a chain and rings it in rhythm. Spec: docs/game/05-enemies-and-bosses.md, "B10
// The Bellwright". One actor; the bell is part of the model, on a chain from his left hand. In
// phase 2 the chain snaps and the bell is a flail: the model switches the first time it plays the
// `snap` animation (the renderer doesn't get the boss phase), hides the long chain, and shortens
// the bell's drop. Render-only: nothing here touches simulation state.
//
// His moves are authored in `game/data/moves/bosses/b10.ron`, so choreography is matched by words
// in the move's `anim` name (hammer, combo, sweep, drag, snap, flail_slam, flail_swing,
// flail_spin). The frame numbers follow the data: every hit lands on a beat, so the first strike
// is on frame 32 (phase 1, a 40-tick beat) or 24 (phase 2, a 32-tick beat), later strikes one
// beat on. The idle sways the bell on its chain at the phase 1 beat.

import { biped, bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, lerpPose, type Choreo, type Keyframes } from '../../procedural/pose';
import type { Mesh } from 'three';
import type { Pose, Rig } from '../../procedural/rig';

const IRON = 0x34373d;
const PLATE = 0x575c66;
const LEATHER = 0x4a3a2e;
const FROST = 0x9fd2e6;
const BRONZE = 0x8a6a3a;
const EMBER = 0xff8a30;

/** Seconds per beat in phase 1 (40 ticks at 30 a second), for the idle bell sway. */
const BEAT_S = 40 / 30;
/** How far the bell hangs below his hand on the long chain, and on the stub after it snaps. */
const CHAIN_LONG = 2.6;
const CHAIN_STUB = 0.9;

/** The meshes that change when the chain snaps, per model instance. */
const chains = new WeakMap<Rig, { long: Mesh[]; stub: Mesh[] }>();
/** Phase 2 state, per model instance: the first `snap` animation seen. */
const snapped = new WeakSet<Rig>();

function build(): Rig {
  const rig = biped({ leg: 2.5, torso: 2.1, neck: 0.3, shoulder: 1.25, hip: 0.55, upperArm: 1.45, foreArm: 1.4 });
  // Belt, apron, and a broad leather smith's chest.
  rig.box('hips', [1.6, 0.5, 0.95], [0, 0, 0], LEATHER);
  rig.box('hips', [1.1, 1.3, 0.1], [0, -0.7, 0.5], LEATHER);
  rig.box('spine', [2.1, 1.1, 1.2], [0, 0.55, 0], PLATE);
  rig.box('spine', [2.7, 1.2, 1.4], [0, 1.55, 0], IRON);
  rig.box('spine', [2.2, 0.16, 1.45], [0, 2.1, 0], FROST, { glow: 0.3 });
  // Rime on the shoulders and a bronze bell-pull at the hip.
  rig.box('spine', [2.6, 0.18, 1.0], [0, 2.2, -0.1], FROST, { glow: 0.2 });
  rig.sphere('spine', 0.22, [-0.95, 0.4, 0.62], BRONZE);
  // Head: a squat hooded helm with a glowing slit and a bell-shaped crest.
  rig.box('head', [0.95, 0.95, 0.95], [0, 0.35, 0], IRON);
  rig.box('head', [0.7, 0.1, 0.06], [0, 0.4, 0.49], EMBER, { glow: 1.5 });
  rig.cone('head', 0.4, 0.8, [0, 1.1, 0], BRONZE);
  for (const side of ['L', 'R'] as const) {
    const s = side === 'L' ? 1 : -1;
    // Pauldrons, rimed.
    rig.box(`shoulder${side}`, [1.1, 0.8, 1.2], [s * 0.1, 0.1, 0], PLATE);
    rig.box(`shoulder${side}`, [1.0, 0.1, 1.0], [s * 0.1, 0.52, 0], FROST, { glow: 0.2 });
    rig.cyl(`shoulder${side}`, 0.4, 0.34, 1.45, [0, -0.72, 0], IRON);
    rig.sphere(`elbow${side}`, 0.4, [0, 0, 0], PLATE);
    rig.cyl(`elbow${side}`, 0.36, 0.42, 1.4, [0, -0.7, 0], IRON);
    rig.box(`hand${side}`, [0.7, 0.7, 0.7], [0, -0.1, 0], LEATHER);
    rig.cyl(`hip${side}`, 0.45, 0.36, 1.25, [0, -0.62, 0], IRON);
    rig.sphere(`knee${side}`, 0.36, [0, 0, 0], PLATE);
    rig.cyl(`knee${side}`, 0.36, 0.32, 1.25, [0, -0.62, 0], PLATE);
    rig.box(`foot${side}`, [0.7, 0.4, 1.2], [0, 0.2, 0.25], LEATHER);
  }
  // The hammer: a long shaft in the right hand and a great square head with a bell-metal face.
  rig.bone('weapon', 'handR', [0, -0.1, 0.05]);
  rig.restRot('weapon', [-0.5, 0, 0]);
  rig.cyl('weapon', 0.13, 0.13, 3.2, [0, -1.2, 0], LEATHER);
  rig.box('weapon', [1.1, 1.0, 1.0], [0, -2.9, 0], IRON);
  rig.box('weapon', [1.15, 0.2, 1.05], [0, -2.4, 0], FROST, { glow: 0.3 });
  rig.box('weapon', [0.2, 1.0, 1.1], [0.62, -2.9, 0], BRONZE);
  // The bell: on a chain from his left hand. The chain is a stack of links down the bone's -Y;
  // the long one is hidden when the chain snaps, and the stub shows.
  rig.bone('chainA', 'handL', [0, -0.15, 0]);
  rig.bone('bell', 'chainA', [0, -CHAIN_LONG, 0]);
  const long: Mesh[] = [];
  const stub: Mesh[] = [];
  for (let y = 0.15; y < CHAIN_LONG - 0.35; y += 0.3) {
    long.push(rig.box('chainA', [0.12, 0.34, 0.12], [0, -y, 0], IRON, { rot: [0, y * 2, 0] }));
  }
  for (let y = 0.15; y < CHAIN_STUB - 0.2; y += 0.3) {
    stub.push(rig.box('chainA', [0.12, 0.34, 0.12], [0, -y, 0], IRON, { rot: [0, y * 2, 0] }));
  }
  stub.forEach((m) => (m.visible = false));
  chains.set(rig, { long, stub });
  // The great frozen bell: a bronze body under a skin of ice, a dark mouth, and a clapper.
  rig.cyl('bell', 0.35, 0.28, 0.5, [0, 0.1, 0], BRONZE);
  rig.cyl('bell', 0.45, 0.9, 1.3, [0, -0.7, 0], BRONZE);
  rig.cyl('bell', 0.5, 0.95, 1.3, [0, -0.7, 0], FROST, { alpha: 0.45, glow: 0.35 });
  rig.cyl('bell', 0.8, 0.82, 0.12, [0, -1.4, 0], 0x0a0c10);
  rig.sphere('bell', 0.2, [0, -1.25, 0], IRON);
  rig.sphere('bell', 0.12, [0, -0.1, 0], FROST, { glow: 1.0 });
  return rig;
}

const idle = (t: number): Pose => {
  // The bell sways on its chain at the beat; the hammer rests on his shoulder.
  const beat = Math.sin((t * Math.PI * 2) / BEAT_S);
  return {
    hips: [0, 0, 0, 0, Math.sin(t * 0.9) * 0.03, 0],
    spine: [Math.sin(t * 0.9) * 0.015, 0, 0],
    head: [0.05, Math.sin(t * 0.3) * 0.08, 0],
    shoulderR: [-0.9, 0, -0.25],
    elbowR: [-1.1],
    shoulderL: [0.2, 0, 0.2],
    elbowL: [-0.15],
    chainA: [beat * 0.12, 0, beat * 0.05]
  };
};

/** The Bellwright lumbers: the gait, with the bell dragging behind. */
const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.3);
  return {
    ...g,
    shoulderR: [-0.9, 0, -0.25],
    elbowR: [-1.1],
    shoulderL: [0.2 + Math.sin(phase) * 0.15 * amp, 0, 0.2],
    chainA: [-0.35 * amp + Math.sin(phase * 2) * 0.1 * amp, 0, 0]
  };
};

/** The glow rises through a wind-up and peaks as the strike lands. */
const flare = (windEnd: number, peak = 2.4) => (f: number) => {
  if (f < windEnd) return 1 + (peak - 1) * 0.7 * Math.min(f / windEnd, 1) ** 2;
  return Math.max(1, peak - (f - windEnd) * 0.12);
};

// Poses. The hammer is a two-bone swing: the right arm and the spine's turn.
const hammerUp: Pose = {
  shoulderR: [-3.0, 0, -0.2],
  elbowR: [-0.5],
  shoulderL: [-0.4, 0, 0.5],
  spine: [-0.35, 0, 0],
  head: [-0.2, 0, 0],
  hips: [0, 0, 0, 0, 0.08, -0.2],
  chainA: [0.5, 0, 0]
};
const hammerDown: Pose = {
  shoulderR: [-0.6, 0, -0.1],
  elbowR: [-0.15],
  shoulderL: [-0.2, 0, 0.3],
  spine: [0.65, 0, 0],
  head: [0.35, 0, 0],
  hips: [0, 0, 0, 0, -0.5, 0.45],
  chainA: [-0.4, 0, 0]
};
const swingRight: Pose = {
  shoulderR: [-1.5, 0, -1.3],
  elbowR: [-0.3],
  spine: [0, -0.9, 0],
  head: [0, 0.4, 0],
  hips: [0, -0.2, 0, 0, -0.05, 0],
  chainA: [0.2, 0, 0.4]
};
const swingLeft: Pose = {
  shoulderR: [-1.5, 0, 1.0],
  elbowR: [-0.1],
  spine: [0.05, 0.9, 0],
  head: [0, -0.4, 0],
  hips: [0, 0.3, 0, 0, -0.05, 0],
  chainA: [0.2, 0, -0.4]
};

const hammer: Choreo = {
  total: 80,
  keys: (n) => attack(32, 35, n, { wind: hammerUp, strike: hammerDown }),
  glow: flare(32, 2.2)
};

const combo: Choreo = {
  total: 160,
  keys: (): Keyframes => [
    [0, {}],
    [20, swingRight],
    [32, swingLeft],
    [44, lerpPose(swingLeft, {}, 0.3)],
    [60, swingLeft],
    [72, swingRight],
    [84, lerpPose(swingRight, {}, 0.3)],
    [100, hammerUp],
    [112, hammerDown],
    [118, hammerDown],
    [160, {}]
  ],
  glow: (f) => (f < 112 ? 1 + 0.4 * (1 + Math.sin(f * 0.4)) : Math.max(1, 2.4 - (f - 112) * 0.05))
};

const sweep: Choreo = {
  total: 120,
  keys: (): Keyframes => {
    const drag: Pose = {
      shoulderR: [-1.0, 0, -1.4],
      elbowR: [-0.1],
      spine: [0, -1.0, 0],
      head: [0, 0.5, 0],
      shoulderL: [-0.4, 0, 0.6],
      hips: [0, -0.3, 0, 0, -0.3, 0]
    };
    const across: Pose = {
      shoulderR: [-1.2, 0, 1.2],
      elbowR: [-0.05],
      spine: [0.1, 1.0, 0],
      head: [0, -0.5, 0],
      shoulderL: [-0.6, 0, 1.0],
      hips: [0, 0.3, 0, 0, -0.3, 0]
    };
    const back: Pose = {
      shoulderL: [-1.8, 0, -0.6],
      spine: [0, -1.1, 0],
      shoulderR: [-0.9, 0, -0.5],
      head: [0, 0.6, 0],
      chainA: [0.9, 0, 0]
    };
    return [
      [0, {}],
      [14, lerpPose({}, drag, 0.6)],
      [30, drag],
      [32, drag],
      [37, across],
      [52, lerpPose(across, {}, 0.2)],
      [66, back],
      [72, back],
      [75, lerpPose(across, back, 0.2)],
      [100, lerpPose({}, across, 0.15)],
      [120, {}]
    ];
  },
  glow: flare(32, 2.4)
};

const drag: Choreo = {
  total: 120,
  keys: (): Keyframes => {
    const haul: Pose = {
      shoulderL: [-1.6, 0, 1.3],
      elbowL: [-0.4],
      spine: [0.1, 1.1, 0],
      head: [0, -0.6, 0],
      shoulderR: [-0.9, 0, -0.4],
      hips: [0, 0.3, 0, 0, -0.4, 0],
      chainA: [-0.6, 0, 0]
    };
    const slam: Pose = {
      shoulderL: [-0.9, 0, -0.5],
      elbowL: [-0.1],
      spine: [0.3, -1.0, 0],
      head: [0, 0.5, 0],
      shoulderR: [-0.9, 0, -0.4],
      hips: [0, -0.3, 0, 0, -0.6, 0.2],
      chainA: [0.6, 0, 0]
    };
    return [
      [0, {}],
      [12, lerpPose({}, haul, 0.5)],
      [30, haul],
      [32, haul],
      [37, slam],
      [60, lerpPose(slam, {}, 0.4)],
      [120, {}]
    ];
  },
  glow: flare(32, 2.6)
};

const snap: Choreo = {
  total: 45,
  keys: (): Keyframes => {
    const crouch: Pose = {
      hips: [0, 0, 0, 0, -0.7, 0],
      spine: [0.3, 0, 0],
      shoulderL: [-1.0, 0, 0.6],
      shoulderR: [-1.0, 0, -0.6],
      head: [0.3, 0, 0],
      chainA: [0.9, 0, 0]
    };
    const roar: Pose = {
      hips: [0, 0, 0, 0, 0.1, 0],
      spine: [-0.4, 0, 0],
      head: [-0.6, 0, 0],
      shoulderL: [-2.2, 0, 0.8],
      shoulderR: [-2.2, 0, -0.8],
      elbowL: [-0.3],
      elbowR: [-0.3]
    };
    return [
      [0, {}],
      [10, crouch],
      [22, roar],
      [40, roar],
      [45, {}]
    ];
  },
  glow: (f) => (f < 10 ? 1 + f * 0.1 : f < 40 ? 2.4 + Math.sin(f) * 0.3 : 1.5)
};

const flailSlam: Choreo = {
  total: 64,
  keys: (n) =>
    attack(24, 27, n, {
      wind: {
        shoulderL: [-3.0, 0, 0.3],
        elbowL: [-0.3],
        shoulderR: [-1.0, 0, -0.5],
        spine: [-0.4, 0, 0],
        head: [-0.3, 0, 0],
        hips: [0, 0, 0, 0, 0.1, -0.2],
        chainA: [0.4, 0, 0]
      },
      strike: {
        shoulderL: [-1.0, 0, 0.2],
        elbowL: [-0.1],
        shoulderR: [-1.0, 0, -0.5],
        spine: [0.7, 0, 0],
        head: [0.4, 0, 0],
        hips: [0, 0, 0, 0, -0.6, 0.4],
        chainA: [-0.5, 0, 0]
      }
    }),
  glow: flare(24, 2.8)
};

const flailSwing: Choreo = {
  total: 96,
  keys: (): Keyframes => {
    const left: Pose = {
      shoulderL: [-1.5, 0, 1.4],
      elbowL: [-0.1],
      spine: [0, 0.9, 0],
      head: [0, -0.4, 0],
      shoulderR: [-1.0, 0, -0.5],
      chainA: [0.6, 0, 0]
    };
    const right: Pose = {
      shoulderL: [-1.5, 0, -0.9],
      elbowL: [-0.1],
      spine: [0, -0.9, 0],
      head: [0, 0.4, 0],
      shoulderR: [-1.0, 0, -0.5],
      chainA: [0.6, 0, 0]
    };
    return [
      [0, {}],
      [16, lerpPose({}, left, 0.8)],
      [23, left],
      [27, right],
      [40, lerpPose(right, {}, 0.3)],
      [52, right],
      [56, left],
      [59, left],
      [64, lerpPose(left, {}, 0.5)],
      [96, {}]
    ];
  },
  glow: flare(24, 2.6)
};

const TAU = Math.PI * 2;
const flailSpin: Choreo = {
  total: 128,
  keys: (): Keyframes => {
    const spin = (turn: number): Pose => ({
      shoulderL: [-1.55, 0, 1.45],
      elbowL: [-0.05],
      shoulderR: [-1.0, 0, -0.6],
      spine: [0, turn, 0],
      hips: [0, 0, 0, 0, -0.2, 0],
      chainA: [0.7, 0, 0]
    });
    // One full turn per beat: the bell passes the circle once on each of frames 24, 56, and 88.
    return [
      [0, {}],
      [14, spin(-0.6)],
      [24, spin(-0.6)],
      [56, spin(-0.6 - TAU)],
      [88, spin(-0.6 - 2 * TAU)],
      [96, spin(-0.6 - 2 * TAU)],
      [112, lerpPose(spin(-0.6 - 2 * TAU), {}, 0.4)],
      [128, {}]
    ];
  },
  glow: (f) => 1.4 + Math.sin(f * 0.5) * 0.4
};

export const bellwrightDef: ModelDef = {
  id: 'b10_the_bellwright',
  height: 6.4,
  build,
  idle,
  walk,
  stride: 4.5,
  // Substring keys, tried in order: the flail moves before `hammer`, which `combo` doesn't match.
  moves: {
    flail_slam: flailSlam,
    flail_swing: flailSwing,
    flail_spin: flailSpin,
    combo,
    hammer,
    sweep,
    drag,
    snap
  },
  fallback: hammer,
  fallAxis: 'x',
  // He staggers back, the hammer drops, and he goes over onto his back with the bell.
  death: (u) => ({
    hips: [0, 0, 0, 0, -0.5 * u, 0],
    spine: [-0.5 * u],
    head: [0.5 * u],
    shoulderL: [0.6 * u, 0, 0.8 * u],
    shoulderR: [0.6 * u, 0, -0.8 * u]
  }),
  tick(rig, ctx) {
    if (ctx.anim.includes('snap') || ctx.anim.includes('flail')) snapped.add(rig);
    const c = chains.get(rig);
    if (!c) return;
    const over = snapped.has(rig);
    c.long.forEach((m) => (m.visible = !over));
    c.stub.forEach((m) => (m.visible = over));
    // After the snap the bell hangs from a short stub.
    rig.bones.get('bell')?.position.set(0, over ? -CHAIN_STUB : -CHAIN_LONG, 0);
  }
};
