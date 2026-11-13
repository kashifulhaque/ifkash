// B09, the Lamplight Matriarch (boss id `b09_lamplight_matriarch`, registry 38010; the body is
// 38011): a vast moth, mother of every light in Hollowdeep. Spec: docs/game/05-enemies-and-bosses.md,
// "B09 The Lamplight Matriarch". Render-only: nothing here touches simulation state.
//
// The model's origin is her feet. In the air the simulation keeps her feet 3.2 m above the floor,
// so the body hangs 1.4 m above the origin with the legs dangling; on the ground her feet are
// 0.3 m up and the legs reach the floor. Her wings span 12 m.
//
// Her abdomen is the one lamp she carries: it glows through the drained cavern of phase 2, so its
// material ignores fog (`fog = false`) and stays visible beyond the player's few meters of sight.
//
// Her moves are authored in `game/data/moves/bosses/b09.ron`, so choreography is matched by words
// in the move's `anim` name (`matriarch_dive`, `_dust`, `_beam`, `_beam_b`, `_pass`, `_buffet`,
// `_buffet_b`, `_lunge`, `_sweep`, `_flare`, `_drain`, `_stunned`). Frame numbers follow the data:
// the dive's shadow marks the floor for 24 frames before she plunges, the dust falls in three
// clouds 20 frames apart from frame 32, the beam fires on frame 30, the pass flies from frame 30,
// and the stun lasts the 60-tick stance break.

import type { MeshStandardMaterial } from 'three';
import type { ModelDef } from '../../procedural/model';
import { attack, lerpPose, type Choreo, type Keyframes } from '../../procedural/pose';
import { Rig, type Pose } from '../../procedural/rig';

const FUR = 0xd8cdb0;
const FUR_DARK = 0x7d6e55;
const WING = 0xe8dcc0;
const WING_HIND = 0xb9a98a;
const WING_RIM = 0x6e5a74;
const LAMP = 0xffcf70;
const EYE = 0xffe9a8;

/** Height of the body above her feet, meters. */
const BODY_HEIGHT = 1.4;

function build(): Rig {
  const rig = new Rig();
  rig.bone('body', null, [0, BODY_HEIGHT, 0]);
  rig.bone('head', 'body', [0, 0.25, 1.3]);
  rig.bone('abdomen', 'body', [0, -0.1, -1.35]);
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.bone(`wing${side}`, 'body', [s * 0.8, 0.55, 0.1]);
    rig.bone(`hind${side}`, 'body', [s * 0.7, 0.45, -0.7]);
    rig.bone(`antenna${side}`, 'head', [s * 0.25, 0.55, 0.3]);
    // The forelegs are heavy (they lunge); the other two pairs just dangle.
    rig.bone(`fore${side}`, 'body', [s * 0.55, -0.7, 0.8]);
    rig.bone(`foreShin${side}`, `fore${side}`, [0, -0.65, 0]);
    rig.bone(`mid${side}`, 'body', [s * 0.6, -0.8, 0.0]);
    rig.bone(`midShin${side}`, `mid${side}`, [0, -0.6, 0]);
    rig.bone(`back${side}`, 'body', [s * 0.5, -0.8, -0.6]);
    rig.bone(`backShin${side}`, `back${side}`, [0, -0.6, 0]);
  }
  // Furry thorax, a small head with great eyes, and feathered antennae.
  rig.sphere('body', 1.15, [0, 0, 0], FUR, { squash: [1.0, 0.95, 1.35] });
  rig.sphere('body', 0.8, [0, 0.35, 0.65], FUR_DARK, { squash: [1.1, 0.5, 0.9] });
  rig.sphere('head', 0.62, [0, 0, 0], FUR);
  for (const s of [1, -1]) rig.sphere('head', 0.26, [s * 0.4, 0.1, 0.38], EYE, { glow: 0.9 });
  // The abdomen: a long glowing lamp ringed in dark fur. Visible through the dark.
  const lamp = rig.sphere('abdomen', 0.95, [0, 0, -0.55], LAMP, { glow: 1.5, squash: [0.9, 0.85, 1.9] });
  (lamp.material as MeshStandardMaterial).fog = false;
  for (const z of [-0.1, -0.7, -1.3]) rig.box('abdomen', [1.8, 0.12, 0.16], [0, 0.05, z], FUR_DARK);
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    // Broad forewings with an eyespot, smaller hindwings, and a dark rim.
    rig.box(`wing${side}`, [6.0, 0.08, 3.4], [s * 3.1, 0, -0.2], WING, { alpha: 0.93 });
    rig.box(`wing${side}`, [6.0, 0.1, 0.2], [s * 3.1, 0, 1.5], WING_RIM);
    const spot = rig.sphere(`wing${side}`, 0.55, [s * 4.0, 0.06, -0.3], LAMP, { glow: 0.8, squash: [1, 0.15, 1] });
    (spot.material as MeshStandardMaterial).fog = false;
    rig.box(`hind${side}`, [4.2, 0.07, 2.4], [s * 2.3, 0, -1.3], WING_HIND, { alpha: 0.93 });
    rig.cone(`antenna${side}`, 0.07, 1.1, [s * 0.1, 0.3, 0.05], FUR_DARK, { rot: [0.5, 0, -s * 0.5] });
    rig.cyl(`fore${side}`, 0.16, 0.12, 0.7, [0, -0.33, 0], FUR_DARK);
    rig.cyl(`foreShin${side}`, 0.12, 0.07, 0.7, [0, -0.33, 0], FUR_DARK);
    for (const leg of ['mid', 'back']) {
      rig.cyl(`${leg}${side}`, 0.1, 0.08, 0.65, [0, -0.3, 0], FUR_DARK);
      rig.cyl(`${leg}Shin${side}`, 0.08, 0.04, 0.65, [0, -0.3, 0], FUR_DARK);
    }
  }
  return rig;
}

/** Legs dangling, wings beating slowly: the idle both in the air and on the ground. */
const idle = (t: number): Pose => {
  const flap = Math.sin(t * 7);
  return {
    body: [0.04 * Math.sin(t * 1.4), 0, 0, 0, Math.sin(t * 2.0) * 0.08, 0],
    wingL: [0, 0, 0.18 + flap * 0.5],
    wingR: [0, 0, -0.18 - flap * 0.5],
    hindL: [0, 0, 0.1 + flap * 0.4],
    hindR: [0, 0, -0.1 - flap * 0.4],
    head: [Math.sin(t * 0.9) * 0.08, Math.sin(t * 0.6) * 0.15, 0],
    abdomen: [Math.sin(t * 1.4 + 1) * 0.1],
    foreL: [0.25 + Math.sin(t * 1.7) * 0.08],
    foreR: [0.25 + Math.sin(t * 1.7 + 1) * 0.08],
    foreShinL: [0.4],
    foreShinR: [0.4],
    midL: [0.15],
    midR: [0.15],
    backL: [-0.1],
    backR: [-0.1]
  };
};

/** Walking: the legs step, the wings only flutter. */
const walk = (phase: number, amp: number): Pose => ({
  body: [0.05 * amp, 0, 0, 0, Math.abs(Math.sin(phase)) * 0.08 * amp, 0],
  wingL: [0, 0, Math.sin(phase * 3) * 0.15 * amp],
  wingR: [0, 0, -Math.sin(phase * 3) * 0.15 * amp],
  foreL: [Math.sin(phase) * 0.5 * amp],
  foreR: [-Math.sin(phase) * 0.5 * amp],
  midL: [-Math.sin(phase) * 0.45 * amp],
  midR: [Math.sin(phase) * 0.45 * amp],
  backL: [Math.sin(phase) * 0.45 * amp],
  backR: [-Math.sin(phase) * 0.45 * amp]
});

const wingsUp: Pose = { wingL: [0, 0, 1.2], wingR: [0, 0, -1.2], hindL: [0, 0, 0.9], hindR: [0, 0, -0.9] };
const wingsBack: Pose = { wingL: [0, 0.7, 0.25], wingR: [0, -0.7, -0.25], hindL: [0, 0.5, 0.2], hindR: [0, -0.5, -0.2] };
const wingsSpread: Pose = { wingL: [0, 0, 0.1], wingR: [0, 0, -0.1], hindL: [0, 0, 0.05], hindR: [0, 0, -0.05] };

/** A glow curve that rises to a peak at `peak` and falls back after `hold` frames. */
const surge =
  (peak: number, top: number, hold = 8) =>
  (f: number): number =>
    f < peak ? 1 + (top - 1) * (f / peak) : f < peak + hold ? top : Math.max(1, top - (f - peak - hold) * 0.12);

const dive: Choreo = {
  total: 84,
  keys: (): Keyframes => {
    // Rears back with the wings raised while the shadow marks the floor, plunges nose-down, slams, and
    // climbs away with a stroke of the wings.
    const rear: Pose = { ...wingsUp, body: [-0.65, 0, 0], abdomen: [0.5], head: [0.3], foreL: [0.9], foreR: [0.9] };
    const plunge: Pose = { ...wingsBack, body: [1.25, 0, 0], abdomen: [-0.6], head: [-0.3], foreL: [-1.3], foreR: [-1.3] };
    const slam: Pose = { ...wingsBack, body: [0.7, 0, 0, 0, -0.2, 0], foreL: [-0.9], foreR: [-0.9] };
    return [
      [0, {}],
      [20, rear],
      [24, rear],
      [34, plunge],
      [40, slam],
      [50, slam],
      [62, { ...wingsUp, body: [-0.2, 0, 0] }],
      [74, lerpPose({}, wingsSpread, 1)],
      [84, {}]
    ];
  },
  glow: surge(24, 2.2, 14)
};

const dust: Choreo = {
  total: 110,
  keys: (): Keyframes => {
    const keys: Keyframes = [[0, {}]];
    // Quick alternating beats for 32 frames, then the wings snap wide at each cloud.
    for (let f = 3, up = true; f < 32; f += 3, up = !up) {
      const a = up ? 1.0 : -0.3;
      keys.push([f, { wingL: [0, 0, a], wingR: [0, 0, -a], body: [-0.12, 0, 0, 0, up ? 0.1 : -0.02, 0] }]);
    }
    for (const f of [32, 52, 72]) {
      keys.push([f, { wingL: [0, 0, 1.3], wingR: [0, 0, -1.3], body: [-0.3, 0, 0], abdomen: [0.6] }]);
      keys.push([f + 6, { wingL: [0, 0, -0.5], wingR: [0, 0, 0.5], body: [0.1, 0, 0], abdomen: [-0.4] }]);
      keys.push([f + 16, { wingL: [0, 0, 0.5], wingR: [0, 0, -0.5] }]);
    }
    keys.push([110, {}]);
    return keys;
  }
};

/** The beam: the abdomen swings forward and down while it glows, then flares on the strike frame. */
const beam = (windEnd: number, strikeEnd: number, total: number, yaw: number): Choreo => ({
  total,
  keys: (): Keyframes =>
    attack(windEnd, strikeEnd, total, {
      wind: { ...wingsUp, body: [-0.2, 0, 0], abdomen: [0.9], head: [0.2] },
      strike: { ...wingsUp, body: [-0.35, yaw, 0], abdomen: [-1.0, 0, 0], head: [-0.2] },
      follow: { ...wingsSpread, body: [-0.1, yaw * 0.5, 0], abdomen: [-0.5] }
    }),
  glow: surge(windEnd, 3.0, strikeEnd - windEnd + 2)
});

const pass: Choreo = {
  total: 96,
  keys: (): Keyframes => {
    const rear: Pose = { ...wingsUp, body: [-0.5, 0, 0], abdomen: [0.6], head: [0.3] };
    const fly: Pose = { ...wingsBack, body: [0.45, 0, 0], abdomen: [-0.3] };
    return [
      [0, {}],
      [24, rear],
      [30, rear],
      [38, fly],
      [54, fly],
      [74, { ...wingsSpread, body: [0.1, 0, 0] }],
      [96, {}]
    ];
  },
  glow: surge(30, 3.4, 26)
};

/** A wing buffet: the near wing cocks back and sweeps across the front. */
const buffet = (side: 1 | -1, windEnd: number, strikeEnd: number, total: number): Choreo => ({
  total,
  keys: (): Keyframes => {
    const wing = side === 1 ? 'wingL' : 'wingR';
    return attack(windEnd, strikeEnd, total, {
      wind: { body: [0, -0.5 * side, 0], [wing]: [0, 0.9 * side, 1.0 * side] },
      strike: { body: [0.1, 0.7 * side, 0], [wing]: [0, -1.1 * side, 0.2 * side] },
      follow: { body: [0.05, 0.3 * side, 0], [wing]: [0, -0.5 * side, 0.15 * side] }
    });
  },
  glow: (f) => (f < windEnd ? 1 + f * 0.04 : 1.6)
});

const lunge: Choreo = {
  total: 78,
  keys: (total): Keyframes =>
    attack(28, 36, total, {
      wind: { ...wingsUp, body: [-0.55, 0, 0], foreL: [1.0], foreR: [1.0], foreShinL: [0.6], foreShinR: [0.6] },
      strike: { ...wingsBack, body: [0.6, 0, 0], foreL: [-1.6], foreR: [-1.6], foreShinL: [-0.4], foreShinR: [-0.4] },
      follow: { body: [0.3, 0, 0], foreL: [-0.6], foreR: [-0.6] }
    })
};

/** The spinning sweep: wings spread for 30 frames, one full turn over frames 30 to 46. */
const sweep: Choreo = {
  total: 104,
  keys: (): Keyframes => [
    [0, {}],
    [10, { ...wingsSpread, body: [-0.1, 0, 0] }],
    [30, { ...wingsSpread, body: [-0.2, 0, 0], abdomen: [0.3] }],
    [38, { ...wingsSpread, body: [0.1, Math.PI, 0] }],
    [46, { ...wingsSpread, body: [0.1, Math.PI * 2, 0] }],
    [60, { ...wingsSpread, body: [0, Math.PI * 2, 0] }],
    [104, { body: [0, Math.PI * 2, 0] }]
  ],
  glow: (f) => (f < 30 ? 1 + f * 0.03 : 1.9)
};

/** The ground flare: wings high, the abdomen swelling until it bursts on frame 36. */
const flare: Choreo = {
  total: 100,
  keys: (): Keyframes => {
    const swell: Pose = { ...wingsUp, body: [-0.3, 0, 0, 0, 0.35, 0], abdomen: [0.7], head: [0.3] };
    return [
      [0, {}],
      [30, swell],
      [36, swell],
      [44, { ...wingsSpread, body: [0.15, 0, 0] }],
      [100, {}]
    ];
  },
  glow: surge(36, 4.0, 10)
};

/** Phase 2: the wings rise and the abdomen blazes as the cavern's light is drawn into it. */
const drain: Choreo = {
  total: 45,
  keys: (): Keyframes => {
    const up: Pose = { ...wingsUp, body: [-0.35, 0, 0, 0, 0.4, 0], abdomen: [0.6], head: [0.3] };
    return [
      [0, {}],
      [14, up],
      [40, up],
      [45, lerpPose({}, up, 0.5)]
    ];
  },
  glow: (f) => 1 + (f / 45) * 2.6
};

/** Down on the floor for the 60-tick stance break: the wings droop, the legs splay, the lamp dims. */
const stunned: Choreo = {
  total: 60,
  keys: (): Keyframes => {
    const down: Pose = {
      wingL: [0, 0, -0.5],
      wingR: [0, 0, 0.5],
      hindL: [0, 0, -0.4],
      hindR: [0, 0, 0.4],
      body: [0.5, 0, 0, 0, -0.7, 0],
      head: [0.5],
      abdomen: [-0.2],
      foreL: [-0.4, 0, 0.5],
      foreR: [-0.4, 0, -0.5]
    };
    return [
      [0, {}],
      [8, down],
      [50, down],
      [60, lerpPose({}, down, 0.2)]
    ];
  },
  glow: (f) => (f < 8 ? 1 - f * 0.08 : f < 50 ? 0.35 : 0.35 + (f - 50) * 0.06)
};

export const matriarchDef: ModelDef = {
  id: 'b09_lamplight_matriarch',
  height: 4.6,
  build,
  idle,
  walk,
  stride: 3,
  // Substring keys, tried in order: `beam_b` before `beam`, `buffet_b` before `buffet`.
  moves: {
    dive,
    dust,
    beam_b: beam(16, 22, 70, 0.5),
    beam: beam(30, 34, 46, 0),
    pass,
    buffet_b: buffet(-1, 18, 22, 60),
    buffet: buffet(1, 20, 24, 66),
    lunge,
    sweep,
    flare,
    drain,
    stunned
  },
  fallback: dive,
  fallAxis: 'z',
  // She folds her wings and tips onto her side.
  death: (u) => ({
    wingL: [0, 0, -0.9 * u],
    wingR: [0, 0, 0.9 * u],
    hindL: [0, 0, -0.7 * u],
    hindR: [0, 0, 0.7 * u],
    head: [0.5 * u],
    abdomen: [-0.3 * u],
    foreL: [0.9 * u],
    foreR: [0.9 * u]
  })
};
