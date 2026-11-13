// B08, the Forgeheart Colossus (boss id `b08_forgeheart_colossus`, registry 37010): a forge
// golem as tall as the cavern, its heart a furnace. Spec: docs/game/05-enemies-and-bosses.md,
// "B08 Forgeheart Colossus". This is the body: the torso, arms, and head, floating at hip height
// above two legs that are separate actors (`legs.ts`), because each leg has a stance meter of its
// own in the simulation. Render-only: nothing here touches simulation state.
//
// Its moves are authored in `game/data/moves/bosses/b08.ron`, so choreography is matched by words
// in the move's `anim` name (stomp_l, stomp_r, sweep, breath_l, breath_r, breath, slam, ignite,
// flare, kneel). The frame numbers follow the data: the stomp's marker glows for 30 frames, the
// breath winds up for 36 with the chest doors opening, and the kneel lasts the 60-tick stance
// break with the furnace open for the riposte.

import { bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, lerpPose, type Choreo, type Keyframes } from '../../procedural/pose';
import { Rig, type Pose } from '../../procedural/rig';

export const IRON = 0x2b2723;
export const PLATE = 0x4a443e;
export const SOOT = 0x15110f;
export const EMBER = 0xff7a22;
export const LAVA = 0xff4a0a;

/** Height of the hips above the ground, meters. The legs (`legs.ts`) are this tall. */
export const HIP_HEIGHT = 4.6;
/** How far the hips drop while the Colossus kneels, meters (the legs fold by the same). */
export const KNEEL_DROP = 1.7;

function build(): Rig {
  const rig = new Rig();
  rig.bone('hips', null, [0, HIP_HEIGHT, 0]);
  rig.bone('spine', 'hips', [0, 0.02, 0]);
  rig.bone('head', 'spine', [0, 4.35, 0.15]);
  // Furnace doors, hinged at the edges of the chest opening. L is +x, as in every biped.
  rig.bone('doorL', 'spine', [1.05, 2.4, 1.38]);
  rig.bone('doorR', 'spine', [-1.05, 2.4, 1.38]);
  // The ember veins that show once the heart ignites (phase 2).
  rig.bone('veins', 'spine', [0, 0, 0]);
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.bone(`shoulder${side}`, 'spine', [s * 2.9, 3.5, 0]);
    rig.bone(`elbow${side}`, `shoulder${side}`, [0, -2.2, 0]);
    rig.bone(`hand${side}`, `elbow${side}`, [0, -2.3, 0]);
  }

  // Pelvis and a belt of banked embers.
  rig.box('hips', [3.2, 1.0, 2.1], [0, 0, 0], IRON);
  rig.box('hips', [3.3, 0.16, 2.2], [0, 0.18, 0], EMBER, { glow: 0.8 });
  // Waist and the broad chest.
  rig.box('spine', [2.6, 1.3, 1.8], [0, 0.9, 0], PLATE);
  rig.box('spine', [4.5, 2.9, 2.7], [0, 2.55, 0], IRON);
  rig.box('spine', [4.7, 0.45, 2.9], [0, 4.05, 0], PLATE);
  rig.box('spine', [4.7, 0.3, 2.9], [0, 1.15, 0], PLATE);
  // The furnace: a dark recess, the heart in it, and a grate of glowing bars.
  rig.box('spine', [2.2, 2.2, 0.3], [0, 2.4, 1.25], SOOT);
  rig.sphere('spine', 0.68, [0, 2.4, 1.0], EMBER, { glow: 1.4 });
  rig.sphere('spine', 0.4, [0, 2.4, 1.0], 0xfff0c0, { glow: 1.8 });
  for (const x of [-0.55, 0, 0.55]) rig.box('spine', [0.07, 1.9, 0.07], [x, 2.4, 1.42], PLATE);
  // Doors: closed they meet in the middle; open they swing out to either side of the chest.
  rig.box('doorL', [1.05, 2.2, 0.16], [-0.52, 0, 0], PLATE);
  rig.box('doorR', [1.05, 2.2, 0.16], [0.52, 0, 0], PLATE);
  rig.box('doorL', [0.8, 0.12, 0.2], [-0.52, 0.0, 0.02], EMBER, { glow: 0.6 });
  rig.box('doorR', [0.8, 0.12, 0.2], [0.52, 0.0, 0.02], EMBER, { glow: 0.6 });
  // The furnace pack on its back: a stack and a venting grill.
  rig.box('spine', [3.0, 2.4, 1.0], [0, 2.6, -1.85], PLATE);
  rig.box('spine', [2.2, 0.16, 0.1], [0, 2.9, -2.4], EMBER, { glow: 0.9 });
  rig.box('spine', [2.2, 0.16, 0.1], [0, 2.4, -2.4], EMBER, { glow: 0.9 });
  rig.cyl('spine', 0.5, 0.65, 2.2, [-1.0, 4.3, -1.9], IRON);
  rig.cyl('spine', 0.5, 0.65, 2.2, [1.0, 4.3, -1.9], IRON);
  rig.sphere('spine', 0.4, [-1.0, 5.45, -1.9], EMBER, { glow: 1.0 });
  rig.sphere('spine', 0.4, [1.0, 5.45, -1.9], EMBER, { glow: 1.0 });
  // Veins: bars of lava across the chest and belly.
  for (const [x, y, h] of [
    [-1.7, 2.6, 2.2],
    [1.7, 2.6, 2.2],
    [-0.9, 0.9, 1.0],
    [0.9, 0.9, 1.0]
  ] as const) {
    rig.box('veins', [0.14, h, 0.06], [x, y, 1.4], LAVA, { glow: 1.6 });
  }
  rig.box('veins', [3.4, 0.12, 0.06], [0, 3.85, 1.46], LAVA, { glow: 1.6 });
  // Head: a squat helm with a glowing slit and two chimney horns.
  rig.cyl('head', 0.6, 0.8, 0.6, [0, -0.2, 0], SOOT);
  rig.box('head', [1.5, 1.25, 1.45], [0, 0.45, 0], IRON);
  rig.box('head', [1.1, 0.16, 0.08], [0, 0.55, 0.74], EMBER, { glow: 1.7 });
  rig.cone('head', 0.22, 0.9, [-0.75, 1.2, 0], PLATE, { rot: [0, 0, 0.35] });
  rig.cone('head', 0.22, 0.9, [0.75, 1.2, 0], PLATE, { rot: [0, 0, -0.35] });
  for (const side of ['L', 'R'] as const) {
    const s = side === 'L' ? 1 : -1;
    // Pauldron with a vent.
    rig.box(`shoulder${side}`, [2.0, 1.3, 2.2], [s * 0.2, 0.35, 0], PLATE);
    rig.cyl(`shoulder${side}`, 0.25, 0.3, 1.0, [s * 0.5, 1.4, -0.5], IRON);
    rig.sphere(`shoulder${side}`, 0.28, [s * 0.5, 1.95, -0.5], EMBER, { glow: 1.0 });
    // Upper arm, then a forearm that flares into the fist.
    rig.cyl(`shoulder${side}`, 0.8, 0.65, 2.2, [0, -1.1, 0], IRON);
    rig.sphere(`elbow${side}`, 0.75, [0, 0, 0], PLATE);
    rig.cyl(`elbow${side}`, 0.75, 0.95, 2.3, [0, -1.15, 0], IRON);
    rig.box(`hand${side}`, [1.7, 1.6, 1.7], [0, -0.45, 0.15], PLATE);
    rig.box(`hand${side}`, [1.2, 0.14, 1.75], [0, -0.1, 0.15], EMBER, { glow: 0.9 });
  }
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, Math.sin(t * 0.3) * 0.02, 0, 0, Math.sin(t * 0.8) * 0.05, 0],
  spine: [Math.sin(t * 0.8) * 0.015, 0, 0],
  head: [0.03, Math.sin(t * 0.25) * 0.08, 0],
  shoulderL: [0.06, 0, 0.14],
  elbowL: [-0.28],
  shoulderR: [0.06, 0, -0.14],
  elbowR: [-0.28]
});

/** The body lumbers: it bobs and sways with the leg actors' own gait, and the arms swing. */
const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.3);
  return {
    hips: [0, g.hips?.[1] ?? 0, 0.02 * Math.sin(phase) * amp, 0, 0.16 * Math.abs(Math.cos(phase)) * amp, 0],
    shoulderL: g.shoulderL,
    shoulderR: g.shoulderR
  };
};

const DOOR_OPEN = 1.5;
const doors = (u: number): Pose => ({ doorL: [0, DOOR_OPEN * u, 0], doorR: [0, -DOOR_OPEN * u, 0] });
const withDoors = (p: Pose, u: number): Pose => ({ ...p, ...doors(u) });

/** The glow rises through a wind-up and peaks as the strike lands. */
const flare = (windEnd: number, peak = 3.2) => (f: number) => {
  if (f < windEnd) return 1 + (peak - 1) * 0.7 * Math.min(f / windEnd, 1) ** 2;
  return Math.max(1, peak - (f - windEnd) * 0.1);
};

const stomp = (side: 1 | -1, windEnd: number, strikeEnd: number, total: number): Choreo => ({
  total,
  keys: (n) =>
    attack(windEnd, strikeEnd, n, {
      wind: {
        hips: [0, 0, side * 0.1, side * -0.35, 0.1, 0],
        spine: [-0.2, side * 0.2, side * 0.1],
        shoulderL: [-0.8, 0, 0.7],
        shoulderR: [-0.8, 0, -0.7],
        head: [-0.15, 0, 0]
      },
      strike: {
        hips: [0, 0, side * -0.06, side * 0.25, -0.75, 0.35],
        spine: [0.5, 0, side * -0.1],
        shoulderL: [-1.3, 0, 0.4],
        shoulderR: [-1.3, 0, -0.4],
        elbowL: [-0.5],
        elbowR: [-0.5],
        head: [0.3, 0, 0]
      }
    }),
  glow: flare(windEnd, 2.4)
});

const sweep: Choreo = {
  total: 70,
  keys: (n) =>
    attack(28, 33, n, {
      wind: {
        spine: [0, -0.9, 0],
        shoulderR: [-1.4, 0, -1.2],
        elbowR: [-0.2],
        shoulderL: [-0.7, 0, 0.7],
        hips: [0, -0.2, 0, 0, -0.1, 0],
        head: [0, 0.4, 0]
      },
      strike: {
        spine: [0.1, 0.9, 0],
        shoulderR: [-1.5, 0, 0.9],
        elbowR: [-0.05],
        shoulderL: [-1.4, 0, 1.3],
        hips: [0, 0.3, 0, 0, -0.15, 0.2],
        head: [0, -0.4, 0]
      }
    }),
  glow: flare(28, 2.6)
};

const breath = (windEnd: number, strikeEnd: number, total: number, yaw: number): Choreo => ({
  total,
  keys: (n): Keyframes => {
    const wind = withDoors(
      {
        spine: [-0.35, yaw * -0.5, 0],
        head: [-0.55, 0, 0],
        shoulderL: [-0.3, 0, 0.9],
        shoulderR: [-0.3, 0, -0.9],
        hips: [0, 0, 0, 0, 0.1, -0.2]
      },
      1
    );
    const strike = withDoors(
      {
        spine: [0.3, yaw * 0.55, 0],
        head: [0.4, 0, 0],
        shoulderL: [-0.2, 0, 1.0],
        shoulderR: [-0.2, 0, -1.0],
        hips: [0, 0, 0, 0, -0.1, 0.25]
      },
      1
    );
    const keys = attack(windEnd, strikeEnd, n, { wind, strike });
    // The chest opens early in the wind-up and swings shut at the end of the move.
    return keys.map(([f, p], i) => (i === 0 || i === keys.length - 1 ? [f, { ...p, ...doors(0) }] : [f, p]));
  },
  glow: flare(windEnd, 3.4)
});

const slam: Choreo = {
  total: 96,
  keys: (n) =>
    attack(34, 52, n, {
      wind: {
        shoulderL: [-3.0, 0, 0.25],
        shoulderR: [-3.0, 0, -0.25],
        elbowL: [-0.4],
        elbowR: [-0.4],
        spine: [-0.5, 0, 0],
        head: [-0.35, 0, 0],
        hips: [0, 0, 0, 0, 0.15, -0.3]
      },
      strike: {
        shoulderL: [-1.1, 0, 0.2],
        shoulderR: [-1.1, 0, -0.2],
        elbowL: [-0.1],
        elbowR: [-0.1],
        spine: [0.75, 0, 0],
        head: [0.45, 0, 0],
        hips: [0, 0, 0, 0, -0.9, 0.5]
      }
    }),
  glow: flare(34, 2.8)
};

const ignite: Choreo = {
  total: 45,
  keys: (): Keyframes => {
    const crouch = withDoors(
      {
        hips: [0, 0, 0, 0, -0.9, 0],
        spine: [0.35, 0, 0],
        head: [0.3, 0, 0],
        shoulderL: [-0.6, 0, 0.1],
        shoulderR: [-0.6, 0, -0.1]
      },
      0.3
    );
    const roar = withDoors(
      {
        hips: [0, 0, 0, 0, 0.1, 0],
        spine: [-0.4, 0, 0],
        head: [-0.7, 0, 0],
        shoulderL: [-0.2, 0, 1.45],
        shoulderR: [-0.2, 0, -1.45],
        elbowL: [-0.3],
        elbowR: [-0.3]
      },
      1
    );
    return [
      [0, {}],
      [9, crouch],
      [22, roar],
      [40, roar],
      [45, withDoors({}, 0.4)]
    ];
  },
  glow: (f) => (f < 10 ? 1 + f * 0.2 : f < 40 ? 3.6 + Math.sin(f) * 0.3 : 2.4)
};

const flareMove: Choreo = {
  total: 90,
  keys: (n) =>
    attack(30, 34, n, {
      wind: withDoors(
        {
          shoulderL: [-0.3, 0, 0.4],
          shoulderR: [-0.3, 0, -0.4],
          spine: [-0.2, 0, 0],
          head: [-0.3, 0, 0],
          hips: [0, 0, 0, 0, -0.35, 0]
        },
        1
      ),
      strike: withDoors(
        {
          shoulderL: [-0.2, 0, 1.5],
          shoulderR: [-0.2, 0, -1.5],
          spine: [-0.3, 0, 0],
          head: [-0.65, 0, 0],
          hips: [0, 0, 0, 0, 0.15, 0]
        },
        1
      )
    }),
  glow: flare(30, 4)
};

const kneel: Choreo = {
  total: 60,
  keys: (): Keyframes => {
    const down = withDoors(
      {
        hips: [0, 0, 0, 0, -KNEEL_DROP, 0.2],
        spine: [0.5, 0, 0],
        head: [0.35, 0, 0],
        shoulderL: [-0.5, 0, 0.35],
        shoulderR: [-0.5, 0, -0.35],
        elbowL: [-0.6],
        elbowR: [-0.6]
      },
      1
    );
    return [
      [0, {}],
      [8, down],
      [50, down],
      [60, lerpPose({}, down, 0.15)]
    ];
  },
  glow: (f) => (f < 8 ? 1 + f * 0.25 : f < 50 ? 3 : 3 - (f - 50) * 0.15)
};

const breathL = breath(36, 49, 70, 1);
const breathR = breath(22, 35, 80, -1);
const breathFull = breath(36, 49, 110, 0);

// Phase 2 state, per model instance: the first ignite animation seen. The renderer doesn't
// get the boss phase, so the ember veins show from then on.
const ignited = new WeakSet<Rig>();

export const colossusDef: ModelDef = {
  id: 'b08_forgeheart_colossus',
  height: 10.6,
  build,
  idle,
  walk,
  stride: 6,
  // Substring keys, tried in order: `breath_l` and `breath_r` before `breath`.
  moves: {
    stomp_l: stomp(-1, 30, 33, 56),
    stomp_r: stomp(1, 20, 23, 52),
    sweep,
    breath_l: breathL,
    breath_r: breathR,
    breath: breathFull,
    slam,
    ignite,
    flare: flareMove,
    kneel
  },
  fallback: slam,
  fallAxis: 'x',
  // It sags and the furnace doors fall open.
  death: (u) => ({
    hips: [0, 0, 0, 0, -1.0 * u, 0],
    spine: [0.6 * u],
    head: [0.8 * u],
    shoulderL: [0.5 * u, 0, 0.7 * u],
    shoulderR: [0.5 * u, 0, -0.7 * u],
    doorL: [0, 1.2 * u, 0],
    doorR: [0, -1.2 * u, 0]
  }),
  tick(rig, ctx) {
    if (ctx.anim.includes('ignite')) ignited.add(rig);
    const veins = rig.bones.get('veins');
    if (veins) veins.visible = ignited.has(rig);
  }
};
