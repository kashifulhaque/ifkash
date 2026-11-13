// B05, the Ossuary Keeper (boss id `b05_ossuary_keeper`, registry 34000; its body is 34001): a
// tall skeletal warden in a rotted shroud, a bell-lantern swinging from its left hand. Spec:
// docs/game/05-enemies-and-bosses.md, "B05 Ossuary Keeper". Strike frames come from
// game/data/moves/bosses/b05.ron; each choreography keys off the move's `anim` name:
//   keeper_bell_1..3    the bell swing combo (the third is the overhead slam)
//   keeper_beam         the lantern beam: a 24-frame charge, then a 33-frame sweep
//   keeper_raise        Raise: the toll that stands the thralls up
//   keeper_surge        the phase 2 transition (the water rises)
//   keeper_scavenge     stoops and takes a thrall's weapon
//   keeper_spear_thrust, keeper_shield_bash   the borrowed moves
// The beam is drawn by the model itself (a glowing bar that sweeps with the hitboxes).

import { biped, bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, lerpPose, type Choreo, type Keyframes } from '../../procedural/pose';
import type { Pose, Rig } from '../../procedural/rig';

const BONE = 0xd9d2bb;
const BONE_DARK = 0xa59d86;
const SHROUD = 0x2c3a3c;
const SHROUD_LIGHT = 0x40504f;
const BRONZE = 0x8a6a3c;
const FLAME = 0xffd27a;
const BEAM = 0xbff6e0;
const VOID = 0x07090a;
const SPEAR_WOOD = 0x4a3826;
const SPEAR_STEEL = 0x9aa09c;

/** The beam's frames in `keeper_beam`: charge until `BEAM_FROM`, sweep to `BEAM_TO`. */
const BEAM_FROM = 28;
const BEAM_TO = 61;
/** Half the sweep's arc in radians (the hitboxes run from -50 to +50 degrees). */
const BEAM_HALF = (50 * Math.PI) / 180;

function build(): Rig {
  const rig = biped({ leg: 1.3, torso: 0.95, neck: 0.12, shoulder: 0.45, hip: 0.18, upperArm: 0.62, foreArm: 0.62 });
  rig.box('hips', [0.5, 0.2, 0.3], [0, 0, 0], BONE_DARK);
  // The spine and ribs: a column and stacked ribs that narrow upward, with a shroud hanging
  // from the shoulders.
  rig.cyl('spine', 0.045, 0.05, 1.0, [0, 0.5, -0.1], BONE);
  for (let i = 0; i < 7; i++) {
    const w = 0.62 - i * 0.04;
    rig.box('spine', [w, 0.045, 0.34], [0, 0.18 + i * 0.12, 0.04], BONE);
  }
  rig.box('spine', [0.7, 0.07, 0.24], [0, 0.98, 0], BONE_DARK);
  rig.box('spine', [0.8, 0.9, 0.04], [0, 0.55, -0.26], SHROUD);
  rig.box('spine', [0.5, 0.5, 0.04], [0, 0.0, 0.2], SHROUD_LIGHT);
  rig.cone('spine', 0.4, 0.8, [0, -0.35, 0.0], SHROUD, { rot: [Math.PI, 0, 0] });
  // A skull under a tall hood, with two cold sparks for eyes.
  rig.sphere('head', 0.19, [0, 0.15, 0], BONE, { squash: [0.95, 1.1, 1.05] });
  rig.box('head', [0.2, 0.08, 0.14], [0, -0.03, 0.1], BONE);
  rig.sphere('head', 0.05, [0.07, 0.19, 0.15], VOID);
  rig.sphere('head', 0.05, [-0.07, 0.19, 0.15], VOID);
  rig.sphere('head', 0.018, [0.07, 0.19, 0.185], BEAM, { glow: 1.6 });
  rig.sphere('head', 0.018, [-0.07, 0.19, 0.185], BEAM, { glow: 1.6 });
  rig.cone('head', 0.27, 0.9, [0, 0.55, -0.05], SHROUD, { rot: [-0.18, 0, 0] });
  rig.cyl('head', 0.25, 0.27, 0.12, [0, 0.12, -0.02], SHROUD_LIGHT);
  for (const side of ['L', 'R'] as const) {
    const s = side === 'L' ? 1 : -1;
    rig.sphere(`shoulder${side}`, 0.14, [0, 0, 0], BONE_DARK);
    rig.cyl(`shoulder${side}`, 0.04, 0.035, 0.62, [0, -0.31, 0], BONE);
    rig.cyl(`elbow${side}`, 0.034, 0.03, 0.62, [0, -0.31, 0], BONE);
    rig.sphere(`elbow${side}`, 0.06, [0, 0, 0], BONE_DARK);
    rig.box(`hand${side}`, [0.1, 0.12, 0.07], [0, -0.02, 0], BONE_DARK);
    // Rags hanging from the forearms.
    rig.box(`elbow${side}`, [0.16, 0.5, 0.02], [s * 0.04, -0.34, -0.04], SHROUD);
    rig.cyl(`hip${side}`, 0.06, 0.045, 0.65, [0, -0.325, 0], BONE);
    rig.sphere(`knee${side}`, 0.06, [0, 0, 0], BONE_DARK);
    rig.cyl(`knee${side}`, 0.045, 0.038, 0.65, [0, -0.325, 0], BONE);
    rig.box(`foot${side}`, [0.12, 0.07, 0.3], [0, 0.035, 0.08], BONE_DARK);
  }
  // The bell-lantern: a bronze bell on a chain from the left hand, a flame inside it.
  rig.bone('bell', 'handL', [0, -0.12, 0]);
  rig.cyl('bell', 0.012, 0.012, 0.5, [0, -0.25, 0], BRONZE);
  rig.cone('bell', 0.3, 0.42, [0, -0.72, 0], BRONZE, { rot: [Math.PI, 0, 0] });
  rig.cyl('bell', 0.3, 0.3, 0.05, [0, -0.93, 0], 0x5e4727);
  rig.sphere('bell', 0.12, [0, -0.7, 0], FLAME, { glow: 1.8 });
  rig.sphere('bell', 0.05, [0, -0.96, 0], BRONZE);
  // The beam: a bar from the lantern's flame, hidden until the lantern beam move. It hangs off
  // the chest so that the body's own turn doesn't have to carry it.
  rig.bone('beam', 'spine', [0, 1.1, 0.35]);
  rig.box('beam', [0.34, 0.34, 10.4], [0, 0, 5.2], BEAM, { glow: 2.2, alpha: 0.62 });
  rig.box('beam', [0.12, 0.12, 10.4], [0, 0, 5.2], 0xffffff, { glow: 3 });
  // A borrowed spear, hidden until the Keeper takes a thrall's weapon.
  rig.bone('spear', 'handR', [0, -0.05, 0.03]);
  rig.restRot('spear', [-0.3, 0, 0]);
  rig.cyl('spear', 0.025, 0.025, 2.4, [0, -0.7, 0], SPEAR_WOOD);
  rig.cone('spear', 0.07, 0.3, [0, -1.95, 0], SPEAR_STEEL, { rot: [Math.PI, 0, 0] });
  // And its shield, on the left forearm.
  rig.bone('shield', 'elbowL', [0, -0.28, 0.06]);
  rig.restRot('shield', [0.9, 0, 0]);
  rig.box('shield', [0.6, 0.85, 0.07], [0, 0, 0.2], SPEAR_WOOD);
  rig.cyl('shield', 0.1, 0.1, 0.08, [0, 0, 0.26], BRONZE, { rot: [Math.PI / 2, 0, 0] });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 0.8) * 0.012, 0],
  spine: [Math.sin(t * 0.8) * 0.015, Math.sin(t * 0.3) * 0.05, 0],
  head: [0.08, Math.sin(t * 0.35) * 0.25, Math.sin(t * 0.6) * 0.04],
  shoulderL: [-0.25, 0, 0.18],
  elbowL: [-0.5, 0, 0],
  shoulderR: [-0.1, 0, -0.18],
  elbowR: [-0.35, 0, 0],
  // The bell sways on its chain.
  bell: [Math.sin(t * 1.3) * 0.12, 0, Math.sin(t * 0.9) * 0.1]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.25);
  return { ...g, bell: [Math.sin(phase) * 0.25 * amp, 0, Math.cos(phase) * 0.15 * amp] };
};

/** The lantern brightens through a wind-up, peaks at the strike, and settles. */
const flare = (windEnd: number) => (f: number) => {
  if (f < windEnd) return 1 + 1.6 * Math.min(f / windEnd, 1) ** 2;
  return Math.max(1, 2.6 - (f - windEnd) * 0.1);
};

const BELL_SWING = (s: number): Pose => ({ bell: [0, 0, s * 0.9] });

// First swing: the bell arcs from the Keeper's right side across to its left.
const bell1: Choreo = {
  total: 46,
  keys: (total) =>
    attack(20, 23, total, {
      wind: { shoulderL: [-1.3, 0, -0.7], elbowL: [-0.5], spine: [0, -0.9, 0], head: [0, 0.4, 0], ...BELL_SWING(-1) },
      strike: { shoulderL: [-1.4, 0, 1.0], elbowL: [-0.1], spine: [0.1, 0.9, 0], hips: [0, 0.3, 0, 0, -0.04, 0.12], ...BELL_SWING(1) }
    }),
  glow: flare(20)
};

// Second swing, back the other way.
const bell2: Choreo = {
  total: 40,
  keys: (total) =>
    attack(16, 19, total, {
      wind: { shoulderL: [-1.4, 0, 1.0], elbowL: [-0.3], spine: [0, 0.9, 0], head: [0, -0.4, 0], ...BELL_SWING(1) },
      strike: { shoulderL: [-1.4, 0, -0.7], elbowL: [-0.1], spine: [0.1, -0.9, 0], hips: [0, -0.3, 0, 0, -0.04, 0.12], ...BELL_SWING(-1) }
    }),
  glow: flare(16)
};

// The finisher: the bell raised high, then slammed into the floor.
const bell3: Choreo = {
  total: 80,
  keys: (total) =>
    attack(26, 30, total, {
      wind: { shoulderL: [-3.1, 0, 0.3], elbowL: [-0.3], shoulderR: [-2.0, 0, -0.5], spine: [-0.45, 0, 0], head: [-0.3, 0, 0], hips: [0, 0, 0, 0, 0.02, -0.1] },
      strike: { shoulderL: [-1.0, 0, 0.1], elbowL: [-0.05], shoulderR: [-1.0, 0, -0.3], spine: [0.85, 0, 0], head: [0.4, 0, 0], hips: [0, 0, 0, 0, -0.2, 0.4], hipL: [-0.7], hipR: [0.6], kneeR: [0.7] },
      // The long punish window: the Keeper stays bent over the bell.
      follow: { shoulderL: [-1.0, 0, 0.1], shoulderR: [-1.0, 0, -0.3], spine: [0.7, 0, 0], head: [0.4, 0, 0], hips: [0, 0, 0, 0, -0.16, 0.3], hipL: [-0.6], hipR: [0.5], kneeR: [0.6] }
    }),
  glow: flare(26)
};

/** The lantern beam: lift and charge (the lantern brightens), flash, then sweep. */
const beam: Choreo = {
  total: 110,
  keys: (total): Keyframes => {
    const charge: Pose = { shoulderL: [-2.5, 0, 0.35], elbowL: [-0.4], shoulderR: [-0.9, 0, -0.7], spine: [-0.3, -0.9, 0], head: [-0.2, -0.3, 0], hips: [0, 0, 0, 0, 0, -0.05] };
    const start: Pose = { ...charge, shoulderL: [-1.55, 0, 0.1], spine: [0.0, -0.9, 0] };
    const end: Pose = { shoulderL: [-1.55, 0, 0.1], elbowL: [-0.3], shoulderR: [-0.9, 0, -0.7], spine: [0.0, 0.9, 0], head: [0, 0.3, 0] };
    return [
      [0, {}],
      [10, lerpPose({}, charge, 0.7)],
      [20, charge],
      [BEAM_FROM - 2, start],
      [BEAM_FROM, start],
      [BEAM_TO, end],
      [BEAM_TO + 14, end],
      [total, {}]
    ];
  },
  glow: (f) => {
    if (f < 24) return 1 + (f / 24) ** 2 * 3.2;
    if (f < 28) return 5;
    if (f < BEAM_TO + 6) return 4;
    return Math.max(1, 4 - (f - BEAM_TO - 6) * 0.08);
  }
};

/** Raise: the bell lifted, struck down at its toll (frame 24 to 34), the arms thrown wide. */
const raise: Choreo = {
  total: 120,
  keys: (total): Keyframes => {
    const up: Pose = { shoulderL: [-3.0, 0, 0.3], elbowL: [-0.3], shoulderR: [-2.6, 0, -0.4], spine: [-0.4, 0, 0], head: [-0.5, 0, 0], bell: [0.4, 0, 0] };
    const toll: Pose = { shoulderL: [-1.3, 0, 0.2], elbowL: [-0.15], shoulderR: [-1.6, 0, -0.9], spine: [0.35, 0, 0], head: [0.25, 0, 0], bell: [-0.5, 0, 0] };
    const wide: Pose = { shoulderL: [-0.6, 0, 1.6], elbowL: [-0.2], shoulderR: [-0.6, 0, -1.6], elbowR: [-0.2], spine: [-0.2, 0, 0], head: [-0.4, 0, 0] };
    return [
      [0, {}],
      [18, up],
      [24, up],
      [30, toll],
      [34, toll],
      [48, wide],
      [90, wide],
      [total, {}]
    ];
  },
  glow: (f) => (f < 24 ? 1 + (f / 24) * 1.4 : f < 60 ? 3.2 : Math.max(1, 3.2 - (f - 60) * 0.05))
};

/** The phase 2 transition: both arms lifted, the head thrown back, the lantern blazing. */
const surge: Choreo = {
  total: 45,
  keys: (): Keyframes => {
    const pose: Pose = { shoulderL: [-2.7, 0, 0.6], shoulderR: [-2.7, 0, -0.6], elbowL: [-0.2], elbowR: [-0.2], spine: [-0.45, 0, 0], head: [-0.7, 0, 0], bell: [0.5, 0, 0] };
    return [
      [0, {}],
      [14, pose],
      [34, pose],
      [45, {}]
    ];
  },
  glow: (f) => 1 + Math.sin((Math.min(f, 45) / 45) * Math.PI) * 4
};

/** The scavenge: stoops to the floor (the weapon is taken at frame 24), rises with a shove. */
const scavenge: Choreo = {
  total: 84,
  keys: (): Keyframes => {
    const stoop: Pose = { spine: [1.15, 0, 0], head: [0.5, 0, 0], shoulderR: [-1.7, 0, -0.2], elbowR: [-0.2], shoulderL: [-0.4, 0, 0.5], hips: [0, 0, 0, 0, -0.35, 0.3], hipL: [-0.7], hipR: [-0.7], kneeL: [0.9], kneeR: [0.9] };
    const rise: Pose = { shoulderR: [-1.0, 0, -0.5], elbowR: [-0.9], spine: [0.1, 0, 0] };
    const wind: Pose = { shoulderL: [-1.3, 0, -0.8], spine: [0, -0.7, 0], shoulderR: [-1.0, 0, -0.5], elbowR: [-0.9] };
    const shove: Pose = { shoulderL: [-1.4, 0, 0.6], spine: [0.2, 0.7, 0], shoulderR: [-1.0, 0, -0.5], elbowR: [-0.9], hips: [0, 0.2, 0, 0, -0.05, 0.12] };
    return [
      [0, {}],
      [16, stoop],
      [30, stoop],
      [42, rise],
      [45, wind],
      [49, shove],
      [60, shove],
      [84, {}]
    ];
  }
};

// Borrowed spear thrust: the spear drawn back, then driven forward.
const spearThrust: Choreo = {
  total: 74,
  keys: (total) =>
    attack(24, 27, total, {
      wind: { shoulderR: [-0.9, 0.2, -0.6], elbowR: [-2.2], shoulderL: [-0.9, 0, 0.6], spine: [-0.05, -0.7, 0], hipL: [0.4], hipR: [-0.3], kneeR: [0.5] },
      strike: { shoulderR: [-1.55, 0, -0.05], elbowR: [-0.05], shoulderL: [-0.9, 0, 0.6], spine: [0.3, 0.5, 0], hips: [0, 0, 0, 0, -0.16, 0.5], hipL: [-0.8], hipR: [0.6], kneeR: [0.5] }
    }),
  glow: flare(24)
};

// Borrowed shield bash: the shield arm drawn in, then punched out.
const shieldBash: Choreo = {
  total: 60,
  keys: (total) =>
    attack(18, 21, total, {
      wind: { shoulderL: [-0.9, 0, 0.8], elbowL: [-1.7], spine: [-0.1, 0.5, 0], hips: [0, 0, 0, 0, -0.06, -0.05] },
      strike: { shoulderL: [-1.75, 0, 0.0], elbowL: [-0.4], spine: [0.35, -0.35, 0], hips: [0, 0, 0, 0, -0.1, 0.3], hipL: [-0.5], hipR: [0.4] }
    }),
  glow: flare(18)
};

export const ossuaryKeeperDef: ModelDef = {
  id: 'b05_ossuary_keeper',
  height: 3.3,
  build,
  idle,
  walk,
  stride: 2.2,
  // Exact anim names, as in the move data.
  moves: {
    keeper_bell_1: bell1,
    keeper_bell_2: bell2,
    keeper_bell_3: bell3,
    keeper_beam: beam,
    keeper_raise: raise,
    keeper_surge: surge,
    keeper_scavenge: scavenge,
    keeper_spear_thrust: spearThrust,
    keeper_shield_bash: shieldBash
  },
  fallback: bell3,
  fallAxis: 'x',
  // The bones let go in a clatter: the knees give and the hood drops.
  death: (u) => ({ spine: [0.5 * u], head: [0.8 * u, 0, 0, 0, -0.1 * u, 0.1 * u], kneeL: [1.0 * u], kneeR: [1.3 * u], shoulderL: [0.4 * u], shoulderR: [0.6 * u] }),
  tick(rig, ctx) {
    const { anim, frame, inMove } = ctx;
    // The beam shows during the sweep, and a faint aim line in the last frames of the charge.
    const beamBone = rig.bones.get('beam');
    if (beamBone) {
      const sweeping = inMove && anim === 'keeper_beam' && frame >= BEAM_FROM && frame <= BEAM_TO;
      const aiming = inMove && anim === 'keeper_beam' && frame >= 8 && frame < BEAM_FROM;
      beamBone.visible = sweeping || aiming;
      if (sweeping) {
        // Local +x is the Keeper's left: the sweep runs from its right to its left, matching
        // the hitboxes' angles.
        const u = (frame - BEAM_FROM) / (BEAM_TO - BEAM_FROM);
        beamBone.rotation.set(0, -BEAM_HALF + u * 2 * BEAM_HALF, 0);
        beamBone.scale.set(1, 1, 1);
      } else if (aiming) {
        beamBone.rotation.set(0, -BEAM_HALF, 0);
        beamBone.scale.set(0.18, 0.18, 1);
      }
    }
    const spear = rig.bones.get('spear');
    const shield = rig.bones.get('shield');
    const spearOn = inMove && (anim === 'keeper_spear_thrust' || (anim === 'keeper_scavenge' && frame >= 24));
    const shieldOn = inMove && (anim === 'keeper_shield_bash' || anim === 'keeper_spear_thrust');
    if (spear) spear.visible = spearOn;
    if (shield) shield.visible = shieldOn;
  }
};
