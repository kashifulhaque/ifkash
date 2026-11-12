// The Glasswing Stalker (7057): a tall, thin, insect-like ambusher whose glass wings and wing
// blade bend the light. Drawn faint while it stands still (the snapshot's `CLOAKED` flag), and
// fully visible, shimmering, while it moves. Moves: gw_lunge (a lunge out of stillness) and
// gw_slash (a close wing-blade slash). Strike frames come from
// game/data/moves/enemies/glasswing_stalker.ron.

import { biped, bipedGait } from '../biped';
import type { ModelDef } from '../model';
import { attack, type Choreo } from '../pose';
import type { Pose, Rig } from '../rig';

const CHITIN = 0x6a8a92;
const DARK = 0x2c3d44;
const GLASS = 0xbfeaf2;
const EYE = 0xfff1a0;

function build(): Rig {
  const rig = biped({ leg: 0.95, torso: 0.6, neck: 0.12, shoulder: 0.17, hip: 0.09, upperArm: 0.46, foreArm: 0.5 });
  rig.restRot('spine', [0.15, 0, 0]);
  rig.box('hips', [0.22, 0.14, 0.16], [0, 0, 0], DARK);
  rig.box('spine', [0.26, 0.58, 0.16], [0, 0.3, 0], CHITIN);
  rig.box('spine', [0.3, 0.05, 0.18], [0, 0.18, 0], DARK);
  rig.box('spine', [0.3, 0.05, 0.18], [0, 0.34, 0], DARK);
  // An elongated head with big compound eyes.
  rig.sphere('head', 0.12, [0, 0.12, 0.03], CHITIN, { squash: [0.85, 1.25, 1.1] });
  for (const s of [1, -1]) rig.sphere('head', 0.05, [s * 0.075, 0.15, 0.08], EYE, { glow: 1.0 });
  // Glass wings folded high on the back.
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.box('spine', [0.04, 0.9, 0.34], [s * 0.1, 0.62, -0.17], GLASS, { rot: [-0.2, 0, -s * 0.25], alpha: 0.5 });
    rig.cyl(`shoulder${side}`, 0.035, 0.04, 0.46, [0, -0.23, 0], CHITIN);
    rig.cyl(`elbow${side}`, 0.03, 0.035, 0.5, [0, -0.25, 0], DARK);
    rig.box(`hand${side}`, [0.06, 0.08, 0.05], [0, 0, 0], CHITIN);
    rig.cyl(`hip${side}`, 0.05, 0.04, 0.47, [0, -0.23, 0], CHITIN);
    rig.cyl(`knee${side}`, 0.04, 0.03, 0.47, [0, -0.23, 0], DARK);
    rig.box(`foot${side}`, [0.07, 0.05, 0.2], [0, 0.025, 0.05], DARK);
  }
  // The wing blade: a long glass edge along the right forearm.
  rig.bone('blade', 'handR', [0, 0, 0.02]);
  rig.restRot('blade', [-0.4, 0, 0]);
  rig.box('blade', [0.02, 0.9, 0.12], [0, -0.45, 0], GLASS, { glow: 0.4, alpha: 0.8 });
  rig.cone('blade', 0.06, 0.2, [0, -0.98, 0], GLASS, { rot: [Math.PI, 0, 0], alpha: 0.8 });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.1) * 0.004, 0],
  spine: [Math.sin(t * 1.1) * 0.01, Math.sin(t * 0.4) * 0.05, 0],
  head: [0.05, Math.sin(t * 0.5) * 0.4, Math.sin(t * 0.6) * 0.08],
  shoulderL: [0, 0, 0.1],
  shoulderR: [-0.1, 0, -0.1],
  elbowL: [-0.15],
  elbowR: [-0.3]
});

const walk = (phase: number, amp: number): Pose => ({
  ...bipedGait(phase, amp, 0.4),
  head: [0.1, Math.sin(phase) * 0.12 * amp, 0]
});

const lunge: Choreo = {
  total: 52,
  keys: (total) =>
    attack(16, 20, total, {
      wind: {
        hips: [0.1, 0, 0, 0, -0.18, -0.08],
        spine: [0.15],
        head: [-0.3],
        hipL: [-0.5],
        hipR: [-0.6],
        kneeL: [1.0],
        kneeR: [1.1],
        shoulderR: [-0.4, 0, -0.9],
        elbowR: [-1.3],
        shoulderL: [0.3, 0, 0.6]
      },
      strike: {
        hips: [-0.15, 0, 0, 0, 0.04, 0.3],
        spine: [-0.1],
        head: [0.1],
        hipL: [0.5],
        hipR: [0.35],
        kneeL: [0.1],
        kneeR: [0.3],
        shoulderR: [-1.55, 0, -0.05],
        elbowR: [-0.05],
        shoulderL: [-0.5, 0, 0.5]
      }
    })
};

const slash: Choreo = {
  total: 46,
  keys: (total) =>
    attack(14, 17, total, {
      wind: { shoulderR: [-0.8, 0, -1.7], elbowR: [-0.6], spine: [0, -0.8, 0], head: [0, 0.3, 0] },
      strike: { shoulderR: [-1.3, 0, 0.8], elbowR: [-0.05], spine: [0.15, 0.8, 0], hips: [0, 0.3, 0, 0, 0, 0.1] }
    })
};

export const glasswingStalkerDef: ModelDef = {
  id: 'glasswing_stalker',
  height: 2.0,
  build,
  idle,
  walk,
  stride: 1.4,
  moves: { gw_lunge: lunge, gw_slash: slash },
  fallback: slash,
  fallAxis: 'x',
  death: (u) => ({ spine: [0.35 * u], head: [0.5 * u], kneeL: [0.7 * u], kneeR: [0.9 * u], shoulderL: [-0.3 * u], shoulderR: [0.2 * u] })
};
