// The Lamenter (`b06_lamenter`, registry 35003), the Drowned Choir's flanker: a thin, quick
// drowned girl in a shroud of wet veils, trailing frost, who weeps knives of ice. Spec:
// docs/game/05-enemies-and-bosses.md, "B06 The Drowned Choir". Moves
// (game/data/moves/bosses/b06.ron): lamenter_knives (a fan of three on frame 20), lamenter_slash_1/2/3
// (hits on 12, 12, and 16), lamenter_veilstab (the verse move: she fades from frame 4, the core
// moves her on frame 18, and the stab lands on frame 34). The `tick` hook fades the whole model
// out and back in for the veil.

import { biped, bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, type Choreo } from '../../procedural/pose';
import type { Pose, Rig } from '../../procedural/rig';
import { CHOIR } from './palette';

function build(): Rig {
  const rig = biped({ leg: 0.95, torso: 0.6, neck: 0.09, shoulder: 0.19, hip: 0.1, upperArm: 0.4, foreArm: 0.4 });
  rig.box('spine', [0.34, 0.62, 0.2], [0, 0.3, 0], CHOIR.robe);
  rig.cyl('hips', 0.16, 0.3, 1.0, [0, -0.4, 0], CHOIR.robe, { alpha: 0.9 });
  // Veils: thin panels streaming from the shoulders and the hem.
  rig.box('spine', [0.4, 0.8, 0.02], [0, 0.1, -0.16], CHOIR.trim, { alpha: 0.7 });
  rig.box('hips', [0.5, 0.5, 0.02], [0, -0.9, 0.18], CHOIR.rime, { alpha: 0.6 });
  rig.sphere('head', 0.17, [0, 0.12, 0], CHOIR.skin, { squash: [0.9, 1.15, 1] });
  rig.box('head', [0.3, 0.32, 0.05], [0, 0.1, 0.12], CHOIR.trim, { alpha: 0.75 });
  for (const s of [1, -1]) rig.sphere('head', 0.016, [s * 0.05, 0.14, 0.16], CHOIR.rime, { glow: 1.8 });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.055, 0.07, 0.4, [0, -0.2, 0], CHOIR.robe);
    rig.cyl(`elbow${side}`, 0.045, 0.06, 0.4, [0, -0.2, 0], CHOIR.robe);
    rig.sphere(`hand${side}`, 0.05, [0, 0, 0], CHOIR.skin);
    // A shard of ice held in each hand.
    rig.cone(`hand${side}`, 0.04, 0.34, [0, -0.2, 0.04], CHOIR.rime, { rot: [Math.PI, 0, 0], glow: 0.7, alpha: 0.85 });
  }
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.6) * 0.02, 0],
  spine: [0.12 + Math.sin(t * 1.6) * 0.03, Math.sin(t * 0.7) * 0.1, 0],
  head: [0.25, Math.sin(t * 0.9) * 0.2, 0],
  shoulderL: [-0.4, 0, 0.3],
  elbowL: [-0.7, 0, 0],
  shoulderR: [-0.4, 0, -0.3],
  elbowR: [-0.7, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.3);
  return { ...g, spine: [0.2 * amp + 0.1, 0, 0] };
};

const knives: Choreo = {
  total: 86,
  keys: (total) =>
    attack(20, 23, total, {
      wind: { shoulderR: [-2.4, 0, -0.9], elbowR: [-0.5], shoulderL: [-2.4, 0, 0.9], elbowL: [-0.5], spine: [-0.3, 0, 0], head: [-0.5, 0, 0] },
      strike: { shoulderR: [-1.5, 0, -0.35], elbowR: [-0.05], shoulderL: [-1.5, 0, 0.35], elbowL: [-0.05], spine: [0.25, 0, 0] }
    }),
  glow: (f) => (f < 20 ? 1 + (f / 20) * 1.8 : Math.max(1, 2.8 - (f - 20) * 0.12))
};

const slash1: Choreo = {
  total: 28,
  keys: (total) =>
    attack(12, 14, total, {
      wind: { shoulderR: [-1.4, 0, -1.3], elbowR: [-0.3], spine: [0.1, -0.7, 0] },
      strike: { shoulderR: [-1.5, 0, 0.6], spine: [0.15, 0.7, 0], hips: [0, 0.3, 0, 0, -0.05, 0.15] }
    })
};

const slash2: Choreo = {
  total: 28,
  keys: (total) =>
    attack(12, 14, total, {
      wind: { shoulderL: [-1.4, 0, 1.3], elbowL: [-0.3], spine: [0.1, 0.7, 0] },
      strike: { shoulderL: [-1.5, 0, -0.6], spine: [0.15, -0.7, 0], hips: [0, -0.3, 0, 0, -0.05, 0.15] }
    })
};

/** The finisher: both shards drawn back, then a lunging thrust, then 34 frames of recovery. */
const slash3: Choreo = {
  total: 74,
  keys: (total) =>
    attack(16, 19, total, {
      wind: { shoulderR: [-1.0, 0, -0.4], elbowR: [-1.9], shoulderL: [-1.0, 0, 0.4], elbowL: [-1.9], spine: [-0.1, 0, 0] },
      strike: { shoulderR: [-1.55, 0, -0.1], elbowR: [-0.05], shoulderL: [-1.55, 0, 0.1], elbowL: [-0.05], spine: [0.5, 0, 0], hipL: [-0.7], hipR: [0.5] }
    })
};

/** The verse move: crouch and fade, reappear coiled for 16 frames, then the thrust on 34. */
const veilstab: Choreo = {
  total: 100,
  keys: (total) =>
    attack(34, 37, total, {
      ready: { spine: [0.3, 0, 0], head: [0.3, 0, 0], shoulderR: [-0.6, 0, -0.2], shoulderL: [-0.6, 0, 0.2] },
      wind: { shoulderR: [-1.2, 0, -0.5], elbowR: [-2.0], shoulderL: [-0.6, 0, 0.3], spine: [-0.2, -0.5, 0], head: [-0.2, 0, 0] },
      strike: { shoulderR: [-1.55, 0, -0.05], elbowR: [-0.05], shoulderL: [-0.4, 0, 0.5], spine: [0.5, 0.2, 0], hipL: [-0.8], hipR: [0.6] }
    }),
  glow: (f) => (f >= 18 && f < 34 ? 2.2 : 1)
};

/** The veil: invisible from frame 6 to 16 of the stab move, fading in and out around that. */
function veil(rig: Rig, ctx: { frame: number; inMove: boolean; anim: string }): void {
  if (!ctx.inMove || !ctx.anim.includes('veilstab')) {
    rig.setOpacity(1);
    return;
  }
  const f = ctx.frame;
  if (f < 4) rig.setOpacity(1);
  else if (f < 7) rig.setOpacity(1 - (f - 3) / 4);
  else if (f < 16) rig.setOpacity(0.04);
  else if (f < 20) rig.setOpacity(0.04 + ((f - 15) / 5) * 0.96);
  else rig.setOpacity(1);
}

export const lamenterDef: ModelDef = {
  id: 'b06_lamenter',
  height: 2.0,
  build,
  idle,
  walk,
  stride: 1.3,
  moves: { knives, slash_1: slash1, slash_2: slash2, slash_3: slash3, veilstab },
  fallback: slash1,
  fallAxis: 'z',
  tick: veil,
  death: (u) => ({ spine: [0.3 * u], head: [0.5 * u], shoulderL: [-0.3 * u, 0, 0.6 * u], shoulderR: [-0.3 * u, 0, -0.6 * u], kneeL: [0.8 * u], kneeR: [0.9 * u] })
};
