// The Bound Exile (`b11_exile`, registry 40001) and its image (`b11_exile_image`, 40002): a
// sorcerer in a hooded grey-violet robe with iron shackles and a broken chain at each wrist.
// Spec: docs/game/05-enemies-and-bosses.md, "B11 The Bound Exile". Moves
// (game/data/moves/bosses/b11.ron): beam (a glyph ring lights over frames 6 to 65, the beam lands on
// 66), blink (fades from frame 6, reappears on 22, bursts on 40), orbs (three orbs circle, then leave
// on frames 36, 48, and 60), shuffle (the blink, swapping with the image), split (the 45-frame
// phase change).
//
// The glyph ring is the cast bar. It is drawn on the floor round the model and fills one glyph at a
// time over the cast. The spec makes the REAL Exile's ring the odd one out: it is pale blue with ROUND
// glyphs and no outer line, while the image's ring is the usual gold with an outer line and DIAMOND
// glyphs, so the real one can be told apart by shape as well as by color (the spec's colorblind
// rule). The `tick` hook drives the ring, the orbs, and the fade
// from the move's frame alone: the renderer never writes the simulation.

import { biped, bipedGait } from '../../procedural/biped';
import type { ModelDef } from '../../procedural/model';
import { attack, type Choreo } from '../../procedural/pose';
import type { Pose, Rig } from '../../procedural/rig';
import type { Object3D } from 'three';
import { EXILE } from './palette';

/** Frames of the move `b11_beam` the ring is drawn in, and how many glyphs it holds. */
const GLYPH_FROM = 6;
const GLYPH_TO = 65;
const BEAM_FRAME = 66;
const BEAM_END = 70;
const GLYPHS = 12;
const RING_RADIUS = 1.6;
/** Frames on which `b11_orbs` launches its three orbs, and the circle they travel. */
const ORB_LAUNCH = [36, 48, 60];
const ORB_RADIUS = 1.6;
/** The frame of the blink moves on which the core moves the actor, and the fade round it. */
const TELEPORT = 22;

function build(real: boolean): Rig {
  const robe = real ? EXILE.robe : EXILE.imageRobe;
  const trim = real ? EXILE.trim : EXILE.imageTrim;
  const skin = real ? EXILE.skin : EXILE.imageSkin;
  const glyph = real ? EXILE.glyph : EXILE.imageGlyph;
  // The glyph ring is the other way round: the real Exile's is the odd one out (see above).
  const ring = real ? EXILE.imageGlyph : EXILE.glyph;
  // The image is drawn a little transparent so it never looks solid.
  const alpha = real ? 1 : 0.78;
  const rig = biped({ leg: 0.95, torso: 0.7, neck: 0.1, shoulder: 0.23, hip: 0.1, upperArm: 0.37, foreArm: 0.37 });
  const o = { alpha };
  rig.box('spine', [0.42, 0.7, 0.25], [0, 0.34, 0], robe, o);
  rig.box('spine', [0.44, 0.06, 0.27], [0, 0.66, 0], trim, o);
  // A long robe, torn at the hem, and a hood that hides all but the chin.
  rig.cyl('hips', 0.2, 0.42, 1.0, [0, -0.42, 0], robe, o);
  rig.box('hips', [0.5, 0.12, 0.5], [0, -0.92, 0], real ? EXILE.robeDark : trim, o);
  rig.sphere('head', 0.19, [0, 0.12, 0], skin, { ...o, squash: [0.9, 1.1, 1] });
  rig.cone('head', 0.24, 0.42, [0, 0.2, -0.03], robe, { ...o, rot: [-0.15, 0, 0] });
  rig.box('head', [0.3, 0.26, 0.08], [0, 0.14, 0.13], real ? EXILE.dark : EXILE.imageTrim, o);
  for (const s of [1, -1]) rig.sphere('head', 0.02, [s * 0.06, 0.15, 0.17], glyph, { ...o, glow: 1.8 });
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`shoulder${side}`, 0.075, 0.095, 0.37, [0, -0.18, 0], robe, o);
    rig.cyl(`elbow${side}`, 0.06, 0.08, 0.37, [0, -0.18, 0], robe, o);
    rig.sphere(`hand${side}`, 0.055, [0, 0, 0], skin, o);
    // An iron shackle at each wrist, and a few links of chain hanging from it.
    rig.cyl(`hand${side}`, 0.075, 0.075, 0.07, [0, 0.07, 0], real ? EXILE.iron : EXILE.imageTrim, o);
    for (let i = 0; i < 3; i++) rig.box(`hand${side}`, [0.03, 0.07, 0.03], [0.02, -0.04 - i * 0.07, 0], real ? EXILE.ironBright : EXILE.imageTrim, o);
    rig.sphere(`hand${side}`, 0.07, [0, -0.04, 0.04], glyph, { ...o, glow: 0.9 });
  }
  // The glyph ring on the floor, in the model's own frame (it turns with the Exile).
  rig.bone('glyph', null, [0, 0.04, 0]);
  for (let i = 0; i < GLYPHS; i++) {
    const a = (i / GLYPHS) * Math.PI * 2;
    const pos: [number, number, number] = [Math.sin(a) * RING_RADIUS, 0, Math.cos(a) * RING_RADIUS];
    if (real) rig.cyl('glyph', 0.17, 0.17, 0.04, pos, ring, { glow: 1.3, alpha: 0.85 });
    else rig.box('glyph', [0.34, 0.04, 0.34], pos, ring, { glow: 1.6, rot: [0, a + Math.PI / 4, 0] });
  }
  if (!real) rig.arc('glyph', RING_RADIUS + 0.26, 0.025, Math.PI * 2, [0, 0, 0], ring, { glow: 1.2, rot: [Math.PI / 2, 0, 0] });
  // Three orbs of Aether that circle the Exile during `orbs`.
  rig.bone('orbs', null, [0, 1.55, 0]);
  for (let i = 0; i < ORB_LAUNCH.length; i++) rig.sphere('orbs', 0.2, [0, 0, 0], real ? EXILE.aether : EXILE.imageGlyph, { glow: 2.2, alpha: real ? 1 : 0.85 });
  return rig;
}

const idle = (t: number): Pose => ({
  hips: [0, 0, 0, 0, Math.sin(t * 1.0) * 0.015, 0],
  spine: [0.05 + Math.sin(t * 1.0) * 0.02, Math.sin(t * 0.4) * 0.06, 0],
  head: [0.12, Math.sin(t * 0.5) * 0.15, 0],
  shoulderL: [-0.35, 0, 0.3],
  elbowL: [-0.9, 0, 0],
  shoulderR: [-0.35, 0, -0.3],
  elbowR: [-0.9, 0, 0]
});

const walk = (phase: number, amp: number): Pose => {
  const g = bipedGait(phase, amp, 0.12);
  return { ...g, shoulderL: [-0.35, 0, 0.3], shoulderR: [-0.35, 0, -0.3] };
};

/** Glyph beam: both hands rise as the ring fills, then the right arm throws the line out. */
const beam: Choreo = {
  total: 140,
  keys: (total) =>
    attack(BEAM_FRAME, BEAM_END, total, {
      ready: { shoulderL: [-1.3, 0, 0.9], shoulderR: [-1.3, 0, -0.9], head: [-0.2, 0, 0] },
      wind: { shoulderL: [-2.5, 0, 0.7], shoulderR: [-2.8, 0, -0.4], elbowR: [-0.3], elbowL: [-0.3], spine: [-0.2, 0, 0], head: [-0.5, 0, 0] },
      strike: { shoulderR: [-1.55, 0, -0.05], elbowR: [-0.05], shoulderL: [-1.0, 0, 0.8], spine: [0.15, 0, 0], head: [0.05, 0, 0] }
    }),
  glow: (f) => (f < BEAM_FRAME ? 1 + (f / BEAM_FRAME) * 2.4 : Math.max(1, 3.4 - (f - BEAM_FRAME) * 0.1))
};

/** Blink: crouch and vanish, rise where he lands, and burst on frame 40. */
const blink: Choreo = {
  total: 84,
  keys: (total) =>
    attack(40, 43, total, {
      ready: { spine: [0.25, 0, 0], head: [0.3, 0, 0], shoulderL: [-0.8, 0, 0.5], shoulderR: [-0.8, 0, -0.5] },
      wind: { spine: [-0.1, 0, 0], shoulderL: [-1.3, 0, 1.0], shoulderR: [-1.3, 0, -1.0], elbowL: [-0.4], elbowR: [-0.4], head: [-0.3, 0, 0] },
      strike: { shoulderL: [-0.4, 0, 1.5], shoulderR: [-0.4, 0, -1.5], spine: [0.15, 0, 0] }
    }),
  glow: (f) => (f >= TELEPORT && f < 40 ? 2.4 : f >= 40 && f < 46 ? 3.2 : 1)
};

/** Orbs: arms spread while three orbs circle; each orb leaves on its frame. */
const orbs: Choreo = {
  total: 120,
  keys: (total) =>
    attack(36, 62, total, {
      wind: { shoulderL: [-1.2, 0, 1.2], shoulderR: [-1.2, 0, -1.2], elbowL: [-0.5], elbowR: [-0.5], spine: [-0.2, 0, 0], head: [-0.3, 0, 0] },
      strike: { shoulderL: [-1.5, 0, 0.5], shoulderR: [-1.5, 0, -0.5], spine: [0.12, 0, 0] }
    }),
  glow: (f) => (f < 36 ? 1 + (f / 36) * 1.8 : Math.max(1, 2.8 - (f - 36) * 0.05))
};

/** The split: arms drawn in, then flung wide as the image steps out near the end. */
const split: Choreo = {
  total: 45,
  keys: (total) =>
    attack(30, 36, total, {
      wind: { shoulderL: [-0.6, 0, 0.2], shoulderR: [-0.6, 0, -0.2], spine: [-0.2, 0, 0], head: [-0.4, 0, 0] },
      strike: { shoulderL: [-0.3, 0, 1.6], shoulderR: [-0.3, 0, -1.6], spine: [0.2, 0, 0], head: [0.1, 0, 0] }
    }),
  glow: (f) => (f < 30 ? 1 + (f / 30) * 1.5 : 3.2)
};

/** The fade of a blink: gone from frame 10 to 22, back by 28. */
function fade(f: number): number {
  if (f < 6) return 1;
  if (f < 10) return 1 - ((f - 5) / 4) * 0.96;
  if (f < TELEPORT) return 0.04;
  if (f < 28) return 0.04 + ((f - (TELEPORT - 1)) / 6) * 0.96;
  return 1;
}

function tick(rig: Rig, ctx: { frame: number; inMove: boolean; anim: string }): void {
  const f = ctx.frame;
  const glyph = rig.bones.get('glyph');
  const orbsBone = rig.bones.get('orbs');
  const move = ctx.inMove ? ctx.anim : '';
  // Fade for the blink and the shuffle.
  rig.setOpacity(move.includes('blink') || move.includes('shuffle') ? fade(f) : 1);
  // The cast bar: one more glyph lit every 5 frames from frame 6, all of them by 65, and
  // lit through the beam's frames.
  if (glyph) {
    const casting = move.includes('beam') && f >= GLYPH_FROM && f <= BEAM_END;
    glyph.visible = casting;
    if (casting) {
      const lit = Math.min(GLYPHS, Math.ceil(((f - GLYPH_FROM + 1) / (GLYPH_TO - GLYPH_FROM + 1)) * GLYPHS));
      glyph.rotation.y = f * 0.02;
      glyph.children.forEach((c: Object3D, i: number) => {
        // The outer line (the last child of the image's ring) shows from the first glyph.
        c.visible = i < GLYPHS ? i < lit : lit > 0;
      });
    }
  }
  // The orbs: out from frame 4, circling, each gone on its launch frame.
  if (orbsBone) {
    const circling = move.includes('orbs') && f >= 4 && f < ORB_LAUNCH[ORB_LAUNCH.length - 1];
    orbsBone.visible = circling;
    if (circling) {
      orbsBone.children.forEach((c: Object3D, i: number) => {
        const a = f * 0.13 + (i / ORB_LAUNCH.length) * Math.PI * 2;
        const grow = Math.min(1, (f - 4) / 14);
        c.position.set(Math.sin(a) * ORB_RADIUS * grow, Math.sin(f * 0.07 + i) * 0.12, Math.cos(a) * ORB_RADIUS * grow);
        c.visible = f < ORB_LAUNCH[i];
      });
    }
  }
}

const dims = {
  height: 2.3,
  stride: 1.4,
  idle,
  walk,
  moves: { beam, blink, orbs, shuffle: blink, split },
  fallback: blink,
  fallAxis: 'x' as const,
  tick,
  death: (u: number): Pose => ({ spine: [-0.35 * u], head: [-0.5 * u], shoulderL: [-0.4 * u, 0, 0.9 * u], shoulderR: [-0.4 * u, 0, -0.9 * u], kneeL: [0.4 * u], kneeR: [0.5 * u] })
};

export const exileDef: ModelDef = { id: 'b11_exile', build: () => build(true), ...dims };
export const exileImageDef: ModelDef = { id: 'b11_exile_image', build: () => build(false), ...dims };
