// The Cantor (`b06_cantor`, registry 35001), the Drowned Choir's caster: a tall drowned singer in
// a waterlogged choir robe with a halo of pale light. Spec: docs/game/05-enemies-and-bosses.md,
// "B06 The Drowned Choir". Moves (game/data/moves/bosses/b06.ron): cantor_beam (30-frame gather,
// then a Radiance line), cantor_rebuke (burst on frame 16), cantor_mend (arms up for 34 frames,
// then the heal), cantor_orbs (the verse move: three orbs on frame 30).

import { biped, bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, type Choreo } from '../../procedural/pose';
import type { Pose, Rig } from '../../procedural/rig';
import { CHOIR } from './palette';

function build(): Rig {
  const rig = biped({ leg: 0.9, torso: 0.7, neck: 0.1, shoulder: 0.24, hip: 0.11, upperArm: 0.36, foreArm: 0.36 });
  rig.box('spine', [0.44, 0.72, 0.26], [0, 0.34, 0], CHOIR.robe);
  rig.box('spine', [0.46, 0.07, 0.28], [0, 0.66, 0], CHOIR.trim);
  // The robe falls to the water, wider at the hem and streaked with weed.
  rig.cyl('hips', 0.22, 0.46, 1.0, [0, -0.42, 0], CHOIR.robe);
  rig.cyl('hips', 0.465, 0.47, 0.06, [0, -0.9, 0], CHOIR.weed);
  rig.sphere('head', 0.19, [0, 0.12, 0], CHOIR.skin, { squash: [0.9, 1.15, 1] });
  // An open mouth, mid-note, and two pale eyes.
  rig.box('head', [0.09, 0.1, 0.04], [0, 0.04, 0.17], 0x0a0e12);
  for (const s of [1, -1]) rig.sphere('head', 0.018, [s * 0.065, 0.16, 0.17], CHOIR.light, { glow: 1.6 });
  // A halo that glows brighter as the Cantor gathers light.
  rig.arc('head', 0.34, 0.018, Math.PI * 2, [0, 0.46, -0.04], CHOIR.light, { glow: 1.2, rot: [Math.PI / 2, 0, 0] });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.08, 0.1, 0.36, [0, -0.18, 0], CHOIR.robe);
    rig.cyl(`elbow${side}`, 0.065, 0.085, 0.36, [0, -0.18, 0], CHOIR.robe);
    rig.sphere(`hand${side}`, 0.055, [0, 0, 0], CHOIR.skin);
    rig.sphere(`hand${side}`, 0.07, [0, -0.05, 0.03], CHOIR.light, { glow: 0.9 });
  }
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.1) * 0.012, 0],
  spine: [Math.sin(t * 1.1) * 0.02, Math.sin(t * 0.4) * 0.05, 0],
  head: [-0.08, Math.sin(t * 0.5) * 0.12, 0],
  shoulderL: [-0.5, 0, 0.45],
  elbowL: [-0.9, 0, 0],
  shoulderR: [-0.5, 0, -0.45],
  elbowR: [-0.9, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.15);
  return { ...g, shoulderL: [-0.5, 0, 0.45], shoulderR: [-0.5, 0, -0.45] };
};

const beam: Choreo = {
  total: 102,
  // One hand lifts through the 30-frame gather, then points down the line as the light lashes.
  keys: (total) =>
    attack(30, 34, total, {
      wind: { shoulderR: [-2.6, 0, -0.3], elbowR: [-0.3], shoulderL: [-1.0, 0, 0.8], spine: [-0.15, 0, 0], head: [-0.5, 0, 0] },
      strike: { shoulderR: [-1.55, 0, -0.05], elbowR: [-0.05], shoulderL: [-1.1, 0, 0.7], spine: [0.1, 0, 0], head: [0.05, 0, 0] }
    }),
  glow: (f) => (f < 30 ? 1 + (f / 30) * 2.2 : Math.max(1, 3.4 - (f - 30) * 0.12))
};

const rebuke: Choreo = {
  total: 76,
  keys: (total) =>
    attack(16, 19, total, {
      wind: { shoulderR: [-1.6, 0, -0.6], elbowR: [-1.1], shoulderL: [-1.6, 0, 0.6], elbowL: [-1.1], spine: [-0.2, 0, 0] },
      strike: { shoulderR: [-1.5, 0, -0.1], elbowR: [-0.05], shoulderL: [-1.5, 0, 0.1], elbowL: [-0.05], spine: [0.2, 0, 0] }
    }),
  glow: (f) => (f < 16 ? 1 + f / 8 : Math.max(1, 3 - (f - 16) * 0.15))
};

/** Mending hymn: both arms rise slowly over 34 frames, the halo swells, and the light falls. */
const mend: Choreo = {
  total: 104,
  keys: (total) =>
    attack(34, 37, total, {
      ready: { shoulderL: [-1.4, 0, 0.9], shoulderR: [-1.4, 0, -0.9], head: [-0.3, 0, 0] },
      wind: { shoulderL: [-2.9, 0, 0.5], shoulderR: [-2.9, 0, -0.5], elbowL: [-0.2], elbowR: [-0.2], spine: [-0.2, 0, 0], head: [-0.6, 0, 0] },
      strike: { shoulderL: [-1.3, 0, 1.2], shoulderR: [-1.3, 0, -1.2], spine: [0.1, 0, 0], head: [0.1, 0, 0] }
    }),
  glow: (f) => (f < 34 ? 1 + (f / 34) * 2.6 : Math.max(1, 3.6 - (f - 34) * 0.1))
};

/** The verse move: arms spread wide, three orbs on frame 30. */
const orbs: Choreo = {
  total: 110,
  keys: (total) =>
    attack(30, 33, total, {
      wind: { shoulderL: [-1.2, 0, 1.4], shoulderR: [-1.2, 0, -1.4], elbowL: [-0.4], elbowR: [-0.4], spine: [-0.25, 0, 0], head: [-0.5, 0, 0] },
      strike: { shoulderL: [-1.5, 0, 0.5], shoulderR: [-1.5, 0, -0.5], spine: [0.15, 0, 0] }
    }),
  glow: (f) => (f < 30 ? 1 + (f / 30) * 2.2 : Math.max(1, 3.2 - (f - 30) * 0.1))
};

export const cantorDef: ModelDef = {
  id: 'b06_cantor',
  height: 2.2,
  build,
  idle,
  walk,
  stride: 1.5,
  moves: { beam, rebuke, mend, orbs, verse: orbs },
  fallback: rebuke,
  fallAxis: 'x',
  death: (u) => ({ spine: [-0.4 * u], head: [-0.6 * u], shoulderL: [-0.4 * u, 0, 0.8 * u], shoulderR: [-0.4 * u, 0, -0.8 * u] })
};
