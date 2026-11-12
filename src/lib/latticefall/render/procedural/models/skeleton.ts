// The Whitewarden Skeleton (7040): a thin, bone-white warden in a rusted helm with a notched
// blade. Moves: ws_slash and ws_lunge. Strike frames come from
// game/data/moves/enemies/whitewarden_skeleton.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const BONE = 0xe3dcc6;
const BONE_DARK = 0xb4ac96;
const RUST = 0x8a5a38;
const VOID = 0x0e0d0b;

function build(): Rig {
  const rig = biped({ leg: 0.9, torso: 0.58, neck: 0.1, shoulder: 0.2, hip: 0.1, upperArm: 0.34, foreArm: 0.34 });
  rig.box('hips', [0.28, 0.12, 0.16], [0, 0, 0], BONE_DARK);
  rig.cyl('spine', 0.025, 0.025, 0.6, [0, 0.3, -0.04], BONE);
  // Ribs: stacked flat boxes, shrinking upward.
  for (let i = 0; i < 5; i++) {
    const w = 0.3 - i * 0.025;
    rig.box('spine', [w, 0.03, 0.2], [0, 0.18 + i * 0.075, 0.03], BONE);
  }
  rig.box('spine', [0.34, 0.04, 0.12], [0, 0.56, 0], BONE_DARK);
  rig.sphere('head', 0.12, [0, 0.1, 0], BONE, { squash: [0.95, 1.1, 1.05] });
  rig.box('head', [0.14, 0.05, 0.1], [0, -0.02, 0.06], BONE);
  rig.sphere('head', 0.035, [0.05, 0.12, 0.1], VOID);
  rig.sphere('head', 0.035, [-0.05, 0.12, 0.1], VOID);
  rig.sphere('head', 0.012, [0.05, 0.12, 0.125], 0x9ad4ff, { glow: 1.2 });
  rig.sphere('head', 0.012, [-0.05, 0.12, 0.125], 0x9ad4ff, { glow: 1.2 });
  // A rusted helm over the skull.
  rig.cyl('head', 0.125, 0.135, 0.12, [0, 0.19, 0], RUST);
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.03, 0.028, 0.34, [0, -0.17, 0], BONE);
    rig.cyl(`elbow${side}`, 0.026, 0.024, 0.34, [0, -0.17, 0], BONE);
    rig.box(`hand${side}`, [0.06, 0.07, 0.04], [0, 0, 0], BONE_DARK);
    rig.cyl(`hip${side}`, 0.04, 0.032, 0.45, [0, -0.225, 0], BONE);
    rig.cyl(`knee${side}`, 0.032, 0.028, 0.45, [0, -0.225, 0], BONE);
    rig.box(`foot${side}`, [0.07, 0.05, 0.2], [0, 0.025, 0.05], BONE_DARK);
    rig.sphere(`elbow${side === 'L' ? 'L' : 'R'}`, 0.035, [0, 0, 0], BONE_DARK);
    rig.sphere(`knee${side}`, 0.04, [0, 0, 0], BONE_DARK);
  }
  rig.bone('weapon', 'handR', [0, -0.02, 0.02]);
  rig.restRot('weapon', [-0.5, 0, 0]);
  rig.box('weapon', [0.04, 0.16, 0.04], [0, 0, 0], 0x3a2c20);
  rig.box('weapon', [0.2, 0.04, 0.05], [0, -0.1, 0], RUST);
  rig.box('weapon', [0.07, 0.85, 0.02], [0, -0.55, 0], 0x9a9d9a);
  rig.box('weapon', [0.03, 0.06, 0.03], [0.05, -0.45, 0], 0x9a9d9a);
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, Math.sin(t * 0.8) * 0.05, 0, 0, Math.sin(t * 1.6) * 0.008, 0],
  spine: [0.06 + Math.sin(t * 1.6) * 0.02, Math.sin(t * 0.7) * 0.08, Math.sin(t * 0.9) * 0.04],
  head: [0.1, Math.sin(t * 0.5) * 0.3, Math.sin(t * 0.8) * 0.08],
  shoulderL: [Math.sin(t * 1.2) * 0.06, 0, 0.1],
  shoulderR: [-0.1, 0, -0.1],
  elbowR: [-0.5, 0, 0],
  elbowL: [-0.15, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.5);
  return { ...g, head: [0.1, 0, Math.sin(phase) * 0.12 * amp] };
};

const slash: Choreo = {
  total: 54,
  keys: (total) =>
    attack(17, 20, total, {
      wind: { shoulderR: [-1.4, 0, -1.5], elbowR: [-0.7], spine: [0, -0.8, 0], head: [0, 0.3, 0] },
      strike: { shoulderR: [-1.5, 0, 0.5], elbowR: [-0.1], spine: [0.15, 0.7, 0], hips: [0, 0.3, 0, 0, 0, 0.08] }
    })
};

const lunge: Choreo = {
  total: 60,
  keys: (total) =>
    attack(20, 23, total, {
      wind: { shoulderR: [-0.9, 0.2, -0.6], elbowR: [-2.2], spine: [-0.05, -0.7, 0], hipL: [0.4], hipR: [-0.3], kneeR: [0.5] },
      strike: {
        shoulderR: [-1.55, 0, -0.05],
        elbowR: [-0.05],
        spine: [0.3, 0.5, 0],
        hips: [0, 0, 0, 0, -0.14, 0.4],
        hipL: [-0.8],
        hipR: [0.6],
        kneeR: [0.5]
      }
    })
};

export const skeletonDef: ModelDef = {
  id: 'whitewarden_skeleton',
  height: 1.85,
  build,
  idle,
  walk,
  stride: 1.5,
  moves: { ws_slash: slash, ws_lunge: lunge },
  fallback: slash,
  fallAxis: 'x',
  // A whitewarden doesn't stay down (it revives), so it collapses in a heap.
  death: (u) => ({ spine: [0.7 * u], head: [0.6 * u], kneeL: [1.1 * u], kneeR: [1.3 * u], hipL: [-0.4 * u], hipR: [-0.5 * u], shoulderL: [0.5 * u], shoulderR: [0.8 * u] })
};
