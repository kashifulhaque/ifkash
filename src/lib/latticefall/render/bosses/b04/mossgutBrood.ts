// B04, the Mossgut Brood (boss id `b04_mossgut_brood`, registry 33000; body 33001, egg sacs
// 33002-33005): a bloated fungal mother grown into the cave wall, ringed by egg sacs. Spec:
// docs/game/05-enemies-and-bosses.md, "B04 Mossgut Brood". Moves and frames are authored in
// game/data/moves/bosses/b04.ron, and the choreography below lands each strike on the same
// frames: spit_near (the glob leaves on 30), spit_far (34), tendril (hit 26-30), tendril_back
// (14-17), and the phase 2 spray (30-42) and tear_free (45 invulnerable frames).
//
// She is low and wide because the chamber's roof is 2.0 m: about 1.9 m to the top of her cap, 3 m
// across her tendrils. Local +Z is forward (toward the player). The pale bulb on her back is her
// weak point; it glows all the time (the renderer isn't told when the four sacs are down).

import type { ModelDef } from '../../procedural/model';
import { attack, type Choreo } from '../../procedural/pose';
import { Rig, type Pose } from '../../procedural/rig';

const FLESH = 0x56653a;
const DARK = 0x2a3320;
const CAP = 0x7a5d3d;
const CAP_SPOT = 0xc9b27a;
const SPORE = 0xb7d75a;
const CORE = 0xe8f08a;
const VEIN = 0x8a9a46;

/** Tendril segments hang along +Z: a bone chain whose rest rotation lays +Y down onto +Z. */
function tendril(rig: Rig, name: string, parent: string, pos: [number, number, number], yaw: number, len: number, r: number) {
  rig.bone(name, parent, pos);
  rig.restRot(name, [Math.PI / 2, yaw, 0]);
  rig.bone(`${name}2`, name, [0, len, 0]);
  rig.restRot(`${name}2`, [-0.25, 0, 0]);
  rig.bone(`${name}3`, `${name}2`, [0, len * 0.9, 0]);
  rig.restRot(`${name}3`, [-0.3, 0, 0]);
  rig.cyl(name, r * 0.8, r, len, [0, len / 2, 0], FLESH);
  rig.cyl(`${name}2`, r * 0.55, r * 0.8, len * 0.9, [0, len * 0.45, 0], FLESH);
  rig.cone(`${name}3`, r * 0.55, len * 0.7, [0, len * 0.35, 0], VEIN);
  rig.sphere(name, r * 1.05, [0, 0, 0], DARK);
}

function build(): Rig {
  const rig = new Rig();
  rig.bone('body', null, [0, 0.8, -0.4]);
  rig.bone('cap', 'body', [0, 0.65, 0.1]);
  rig.bone('maw', 'body', [0, 0.05, 1.05]);
  rig.bone('jaw', 'maw', [0, -0.12, 0.05]);
  // A swollen, lumpy trunk, wider than it is tall, rooted into the wall behind it.
  rig.sphere('body', 1.05, [0, 0, 0], FLESH, { squash: [1.35, 0.8, 1.0] });
  rig.sphere('body', 0.7, [-0.95, -0.15, 0.25], FLESH, { squash: [1, 0.8, 1] });
  rig.sphere('body', 0.7, [0.95, -0.15, 0.25], FLESH, { squash: [1, 0.8, 1] });
  rig.sphere('body', 0.55, [0, -0.35, 0.55], VEIN, { squash: [1.5, 0.7, 1] });
  // Roots and mycelium running back into the stone.
  for (const x of [-1.4, -0.7, 0, 0.7, 1.4]) {
    rig.cone('body', 0.22, 1.1, [x, -0.1, -0.95], DARK, { rot: [-Math.PI / 2, 0, 0] });
  }
  // The cap: a wide brown dome with pale spots, tilted slightly forward.
  rig.sphere('cap', 1.15, [0, 0.0, 0.0], CAP, { squash: [1.25, 0.45, 1.05] });
  for (const [x, z] of [
    [-0.5, 0.3],
    [0.45, 0.35],
    [0.0, -0.3],
    [-0.8, -0.1],
    [0.85, 0.0]
  ] as const) {
    rig.sphere('cap', 0.14, [x, 0.4, z], CAP_SPOT, { squash: [1, 0.5, 1] });
  }
  // The weak point: a pale bulb in a cleft on her back, under the cap's rim.
  rig.sphere('cap', 0.28, [0, 0.15, -0.95], CORE, { glow: 1.6 });
  rig.sphere('cap', 0.12, [0.0, 0.2, -1.15], CORE, { glow: 2.2 });
  // The maw: a dark throat ringed by a wet lip, and spore-glow in it.
  rig.sphere('maw', 0.38, [0, 0.0, 0.0], DARK, { squash: [1.2, 0.9, 0.8] });
  rig.sphere('maw', 0.18, [0, 0.0, 0.1], SPORE, { glow: 1.2 });
  rig.box('jaw', [0.55, 0.1, 0.35], [0, -0.05, 0.1], VEIN);
  // Eyes: three small glowing sores above the maw.
  for (const x of [-0.3, 0, 0.3]) rig.sphere('body', 0.07, [x, 0.3, 0.95], SPORE, { glow: 1.4 });
  // Two long tendrils (the sweep) and four short ones that writhe.
  tendril(rig, 'tendA', 'body', [1.0, -0.05, 0.6], 0.5, 1.15, 0.13);
  tendril(rig, 'tendB', 'body', [-1.0, -0.05, 0.6], -0.5, 1.15, 0.13);
  tendril(rig, 'tendC', 'body', [1.25, -0.25, 0.1], 1.0, 0.7, 0.1);
  tendril(rig, 'tendD', 'body', [-1.25, -0.25, 0.1], -1.0, 0.7, 0.1);
  return rig;
}

const idle = (t: number): Pose => ({
  body: [Math.sin(t * 0.8) * 0.015, Math.sin(t * 0.37) * 0.04, 0, 0, Math.sin(t * 1.1) * 0.025, 0],
  cap: [Math.sin(t * 0.6) * 0.03, 0, Math.sin(t * 0.9) * 0.02],
  maw: [0.04 + Math.max(0, Math.sin(t * 1.3)) * 0.05],
  jaw: [0.05 + Math.max(0, Math.sin(t * 1.3)) * 0.12],
  tendA: [0, Math.sin(t * 1.2) * 0.2, 0],
  tendB: [0, -Math.sin(t * 1.2 + 1) * 0.2, 0],
  tendC: [0, Math.sin(t * 1.7 + 2) * 0.35, 0],
  tendD: [0, -Math.sin(t * 1.7) * 0.35, 0],
  tendA2: [Math.sin(t * 1.5) * 0.15],
  tendB2: [Math.sin(t * 1.5 + 1) * 0.15]
});

/** Phase 2: she hauls herself along, the whole trunk heaving and the short tendrils paddling. */
const crawl = (phase: number, amp: number): Pose => ({
  body: [Math.sin(phase) * 0.06 * amp, Math.sin(phase * 0.5) * 0.08 * amp, 0, 0, Math.abs(Math.sin(phase)) * 0.12 * amp, 0],
  cap: [Math.sin(phase + 1) * 0.05 * amp],
  tendC: [Math.sin(phase) * 0.9 * amp, Math.sin(phase) * 0.6 * amp, 0],
  tendD: [Math.sin(phase + Math.PI) * 0.9 * amp, -Math.sin(phase + Math.PI) * 0.6 * amp, 0],
  tendA: [Math.sin(phase + 1) * 0.3 * amp, 0.3 * amp, 0],
  tendB: [Math.sin(phase + 1 + Math.PI) * 0.3 * amp, -0.3 * amp, 0]
});

/** The spores pulse through a wind-up, peak at the strike, and settle. */
const pulse = (windEnd: number) => (f: number) => {
  if (f < windEnd) return 1 + 1.6 * Math.min(f / windEnd, 1) ** 2;
  return Math.max(1, 2.6 - (f - windEnd) * 0.1);
};

/** A glob: the trunk bloats and rears back, then the throat heaves forward on the launch frame. */
function spit(windEnd: number, total: number): Choreo {
  return {
    total,
    keys: (t) =>
      attack(windEnd, windEnd + 3, t, {
        wind: {
          body: [-0.28, 0, 0, 0, 0.08, -0.25],
          cap: [-0.2, 0, 0],
          maw: [-0.4, 0, 0, 0, 0.1, -0.1],
          jaw: [0.9],
          tendA: [0, 0.6, 0],
          tendB: [0, -0.6, 0]
        },
        strike: {
          body: [0.3, 0, 0, 0, -0.04, 0.3],
          cap: [0.15, 0, 0],
          maw: [0.5, 0, 0, 0, 0, 0.35],
          jaw: [0.1],
          tendA: [0, -0.2, 0],
          tendB: [0, 0.2, 0]
        }
      }),
    glow: pulse(windEnd)
  };
}

/** The sweep: the right tendril draws back across her, then whips left to right (and back). */
const tendrilSweep: Choreo = {
  total: 54,
  keys: (t) =>
    attack(26, 30, t, {
      wind: {
        body: [0, 0.3, 0, 0, 0, -0.1],
        tendA: [-0.2, 1.7, 0],
        tendA2: [-0.2],
        tendB: [0, 0.9, 0],
        tendC: [0, 0.8, 0]
      },
      strike: {
        body: [0, -0.25, 0, 0, 0, 0.2],
        tendA: [-0.1, -1.7, 0],
        tendA2: [0.3],
        tendB: [0, -0.6, 0],
        tendC: [0, -0.4, 0]
      }
    }),
  glow: pulse(26)
};

const tendrilBack: Choreo = {
  total: 60,
  keys: (t) =>
    attack(14, 17, t, {
      wind: {
        body: [0, -0.25, 0, 0, 0, 0.1],
        tendB: [-0.2, -1.7, 0],
        tendB2: [-0.2],
        tendA: [0, -0.9, 0]
      },
      strike: {
        body: [0, 0.3, 0, 0, 0, 0.2],
        tendB: [-0.1, 1.7, 0],
        tendB2: [0.3],
        tendA: [0, 0.6, 0]
      }
    }),
  glow: pulse(14)
};

/** Phase 2's spray: she rears up, the cap flares open, and the maw gapes through the cloud. */
const spray: Choreo = {
  total: 80,
  keys: (t) =>
    attack(30, 42, t, {
      wind: {
        body: [-0.4, 0, 0, 0, 0.18, -0.35],
        cap: [-0.55, 0, 0],
        maw: [-0.5, 0, 0, 0, 0.15, -0.1],
        jaw: [1.0],
        tendA: [0, 0.9, 0.2],
        tendB: [0, -0.9, -0.2],
        tendC: [0, 1.2, 0],
        tendD: [0, -1.2, 0]
      },
      strike: {
        body: [0.1, 0, 0, 0, 0.1, 0.2],
        cap: [0.35, 0, 0],
        maw: [0.15, 0, 0, 0, 0, 0.3],
        jaw: [0.7],
        tendA: [0, 0.9, 0.2],
        tendB: [0, -0.9, -0.2],
        tendC: [0, 1.0, 0],
        tendD: [0, -1.0, 0]
      }
    }),
  glow: pulse(30)
};

/** The transition: she wrenches loose of the wall, shuddering, and drops forward onto her tendrils. */
const tearFree: Choreo = {
  total: 45,
  keys: () => [
    [0, {}],
    [8, { body: [-0.15, 0.1, 0.06, 0.05, 0.05, -0.1], cap: [-0.2, 0, 0.1], jaw: [0.8], tendA: [0, 0.9, 0], tendB: [0, -0.9, 0] }],
    [14, { body: [-0.1, -0.1, -0.06, -0.05, 0.1, -0.2], cap: [-0.3, 0, -0.1], jaw: [1.0], tendC: [0, 1.2, 0], tendD: [0, -1.2, 0] }],
    [22, { body: [-0.2, 0.1, 0.05, 0.04, 0.2, -0.3], cap: [-0.35], jaw: [1.1], tendA: [0, 1.3, 0], tendB: [0, -1.3, 0] }],
    [34, { body: [0.15, 0, 0, 0, -0.05, 0.25], cap: [0.15], jaw: [0.5], tendA: [0.4, 0.3, 0], tendB: [0.4, -0.3, 0], tendC: [0.5, 0.4, 0], tendD: [0.5, -0.4, 0] }],
    [45, { body: [0.05, 0, 0, 0, -0.02, 0.3], jaw: [0.2], tendA: [0.2, 0.2, 0], tendB: [0.2, -0.2, 0] }]
  ],
  glow: (f) => 1 + 2.4 * Math.max(0, Math.sin((f / 45) * Math.PI))
};

export const mossgutBroodDef: ModelDef = {
  id: 'b04_mossgut_brood',
  height: 1.9,
  build,
  idle,
  walk: crawl,
  stride: 1.4,
  // Substring keys, tried in order; the first is also the fallback.
  moves: {
    b04_tendril_back: tendrilBack,
    b04_tendril: tendrilSweep,
    b04_spit_near: spit(30, 78),
    b04_spit_far: spit(34, 82),
    b04_spray: spray,
    b04_tear_free: tearFree
  },
  fallback: tendrilSweep,
  fallAxis: 'x',
  // She slumps and the cap sags; the tendrils go slack.
  death: (u) => ({
    body: [0.25 * u, 0, 0, 0, -0.35 * u, 0.2 * u],
    cap: [0.5 * u, 0, 0.15 * u],
    maw: [0.3 * u],
    jaw: [1.2 * u],
    tendA: [0.6 * u, 0.4 * u, 0],
    tendB: [0.6 * u, -0.4 * u, 0],
    tendC: [0.6 * u, 0.5 * u, 0],
    tendD: [0.6 * u, -0.5 * u, 0]
  })
};

// --- The egg sacs ---------------------------------------------------------------------------

function buildSac(): Rig {
  const rig = new Rig();
  rig.bone('sac', null, [0, 0.5, 0]);
  rig.bone('eggs', 'sac', [0, 0, 0]);
  // A glistening, translucent membrane over a cluster of dark eggs with a sickly glow.
  rig.sphere('sac', 0.62, [0, 0, 0], 0xc9d89a, { alpha: 0.78, glow: 0.35, squash: [1, 1.1, 1] });
  for (const [x, y, z] of [
    [0.15, -0.05, 0.1],
    [-0.2, 0.1, -0.05],
    [0.05, 0.25, -0.15],
    [-0.05, -0.2, 0.2]
  ] as const) {
    rig.sphere('eggs', 0.2, [x, y, z], SPORE, { glow: 0.9 });
    rig.sphere('eggs', 0.07, [x + 0.04, y + 0.04, z + 0.12], DARK);
  }
  // Roots that tie it to the floor.
  for (let i = 0; i < 5; i++) {
    const a = (i / 5) * Math.PI * 2;
    rig.cone('sac', 0.1, 0.55, [Math.cos(a) * 0.5, -0.35, Math.sin(a) * 0.5], DARK, { rot: [Math.sin(a) * 1.1, 0, -Math.cos(a) * 1.1] });
  }
  return rig;
}

const sacIdle = (t: number): Pose => ({
  sac: [Math.sin(t * 0.9) * 0.025, 0, Math.sin(t * 1.3) * 0.02, 0, Math.sin(t * 2.0) * 0.03, 0],
  eggs: [0, t * 0.25, 0]
});

/** A sac never attacks; a move that names one would only swell. */
const swell: Choreo = {
  total: 30,
  keys: (t) => [
    [0, {}],
    [t / 2, { sac: [0, 0, 0, 0, 0.06, 0] }],
    [t, {}]
  ]
};

export const eggSacDef: ModelDef = {
  id: 'b04_egg_sac',
  height: 1.1,
  build: buildSac,
  idle: sacIdle,
  walk: () => ({}),
  stride: 1,
  moves: {},
  fallback: swell,
  fallAxis: 'z',
  // It bursts: it slumps and sinks into the floor.
  death: (u) => ({ sac: [0.3 * u, 0, 0, 0, -0.45 * u, 0], eggs: [0, 0, 0, 0, -0.1 * u, 0] })
};
