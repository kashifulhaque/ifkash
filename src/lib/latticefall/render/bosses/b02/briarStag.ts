// B02, the Briar Stag (boss `b02_briar_stag` 31000, body `b02_briar_stag_body` 31001): a giant
// stag grown over with thorns and briar. Spec: docs/game/05-enemies-and-bosses.md, "B02 The
// Briar Stag". Moves are authored in game/data/moves/bosses/b02.ron; each choreography below
// is named after that file's `anim` field and strikes on the same frames as the move's
// hitboxes, with the tell the fairness rules ask for (docs "Fairness rules"):
//
//   stag_charge        scrapes twice (frames 4-14, 16-28), coils, runs frames 30-62, skids
//   stag_charge_2      phase 2: one shorter scrape, runs frames 26-54, skids
//   stag_charge_again  phase 2: swings round, scrapes once, runs frames 26-54, long recovery
//   stag_toss          head dips to frame 20, then heaves up
//   stag_leap          rears, leaps frames 10-30 (the body arcs 2.4 m up), lands on the marker
//   stag_thorn_burst   thorns bristle for 36 frames, then burst
//   stag_thorn_burst_2 the same with longer thorns
//   stag_roar          the phase 2 transition: it rears and its thorns grow
//
// The thorns are bones of their own (`thorn0`...), scaled along their length by `tick`: they
// bristle during a thorn burst and stay long once the Stag has roared (phase 2). A phase 1
// charge or burst means a fresh fight, so the thorns shrink back.

import type { ModelDef } from '../../procedural/model';
import { attack, type Choreo, type Keyframes } from '../../procedural/pose';
import { Rig, type Pose } from '../../procedural/rig';

/** Registry number of B02's body: `b02_briar_stag_body`. The boss itself (31000) isn't an actor. */
export const B02_KIND = 31001;

const HIDE = 0x6b5640;
const BELLY = 0x8a7456;
const DARK = 0x3c3126;
const THORN = 0x8a3a22;
const BRIAR = 0x3e5b33;
const BONE = 0xd2c6a4;
const EYE = 0xf0a63c;

/** How much longer the thorns are after the roar. */
const GROWN = 1.7;
/** How much longer they bristle during a burst, on top of their length. */
const BRISTLE = 0.9;

const THORNS: Array<{ parent: string; pos: [number, number, number]; rot: [number, number, number]; h: number }> = [
  { parent: 'body', pos: [0, 0.48, -0.9], rot: [-0.5, 0, 0], h: 0.5 },
  { parent: 'body', pos: [0.2, 0.46, -0.45], rot: [-0.3, 0, -0.35], h: 0.6 },
  { parent: 'body', pos: [-0.2, 0.46, -0.45], rot: [-0.3, 0, 0.35], h: 0.6 },
  { parent: 'body', pos: [0, 0.48, 0.0], rot: [-0.15, 0, 0], h: 0.7 },
  { parent: 'body', pos: [0.24, 0.42, 0.45], rot: [-0.1, 0, -0.5], h: 0.65 },
  { parent: 'body', pos: [-0.24, 0.42, 0.45], rot: [-0.1, 0, 0.5], h: 0.65 },
  { parent: 'chest', pos: [0, 0.56, 0.1], rot: [0.1, 0, 0], h: 0.75 },
  { parent: 'chest', pos: [0.3, 0.45, 0.0], rot: [0.1, 0, -0.6], h: 0.6 },
  { parent: 'chest', pos: [-0.3, 0.45, 0.0], rot: [0.1, 0, 0.6], h: 0.6 },
  { parent: 'neck', pos: [0, 0.2, -0.22], rot: [-0.5, 0, 0], h: 0.55 },
  { parent: 'neck', pos: [0, 0.55, -0.2], rot: [-0.4, 0, 0], h: 0.5 },
  { parent: 'hlegL', pos: [0.18, 0.05, -0.05], rot: [0, 0, -0.9], h: 0.45 },
  { parent: 'hlegR', pos: [-0.18, 0.05, -0.05], rot: [0, 0, 0.9], h: 0.45 }
];

function build(): Rig {
  const rig = new Rig();
  rig.bone('body', null, [0, 1.55, 0]);
  rig.bone('chest', 'body', [0, 0.05, 0.85]);
  rig.bone('neck', 'chest', [0, 0.25, 0.4]);
  rig.restRot('neck', [0.85, 0, 0]);
  rig.bone('head', 'neck', [0, 0.78, 0]);
  rig.restRot('head', [-0.7, 0, 0]);
  rig.bone('jaw', 'head', [0, -0.12, 0.32]);
  rig.bone('tail', 'body', [0, 0.3, -1.05]);
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    rig.bone(`antler${side}`, 'head', [s * 0.17, 0.22, -0.02]);
    rig.bone(`fleg${side}`, 'chest', [s * 0.34, -0.3, 0.05]);
    rig.bone(`fshin${side}`, `fleg${side}`, [0, -0.78, 0]);
    rig.bone(`hleg${side}`, 'body', [s * 0.34, -0.1, -0.85]);
    rig.bone(`hshin${side}`, `hleg${side}`, [0, -0.72, 0]);
  }
  THORNS.forEach((t, i) => {
    rig.bone(`thorn${i}`, t.parent, t.pos);
    rig.restRot(`thorn${i}`, t.rot);
    rig.cone(`thorn${i}`, 0.07, t.h, [0, t.h / 2, 0], THORN);
  });

  // The body: a deep barrel of a chest, a lean belly, a short tail.
  rig.box('body', [0.95, 0.9, 2.0], [0, 0, 0], HIDE);
  rig.box('body', [0.8, 0.3, 1.9], [0, -0.4, 0.0], BELLY);
  rig.box('chest', [1.05, 1.05, 0.95], [0, 0.0, 0.0], HIDE);
  rig.box('chest', [0.5, 0.5, 0.5], [0, 0.3, 0.25], DARK);
  rig.cone('tail', 0.12, 0.55, [0, 0.05, -0.2], BELLY, { rot: [-Math.PI / 2 - 0.5, 0, 0] });
  // Briar wrapped round the barrel and the neck: green bands with thorny knots.
  for (const z of [-0.7, -0.15, 0.4]) rig.box('body', [1.0, 0.09, 0.14], [0, 0.12 + z * 0.1, z], BRIAR, { rot: [0.0, 0.1, 0.05] });
  rig.box('chest', [1.1, 0.1, 0.15], [0, -0.1, 0.1], BRIAR, { rot: [0, -0.15, 0.1] });
  rig.box('neck', [0.46, 0.1, 0.16], [0, 0.45, 0], BRIAR, { rot: [0, 0.2, 0] });
  rig.cyl('neck', 0.3, 0.38, 0.9, [0, 0.38, 0], HIDE);
  // The head: a long skull, a broad snout, burning eyes.
  rig.box('head', [0.42, 0.42, 0.7], [0, 0, 0.25], HIDE);
  rig.box('head', [0.3, 0.28, 0.42], [0, -0.06, 0.65], BELLY);
  rig.box('head', [0.12, 0.1, 0.1], [0, -0.04, 0.88], DARK);
  rig.box('jaw', [0.26, 0.08, 0.4], [0, -0.05, 0.2], DARK);
  for (const s of [1, -1]) {
    rig.sphere('head', 0.055, [s * 0.2, 0.1, 0.36], EYE, { glow: 1.2 });
    rig.cone('head', 0.07, 0.28, [s * 0.28, 0.12, 0.05], HIDE, { rot: [0.0, 0, -s * 1.2] });
  }
  // Antlers: a main beam sweeping up and back, and three tines forward. Briar is caught in them.
  for (const [s, side] of [
    [1, 'L'],
    [-1, 'R']
  ] as const) {
    const bone = `antler${side}`;
    rig.cyl(bone, 0.045, 0.07, 1.1, [s * 0.18, 0.5, -0.12], BONE, { rot: [-0.35, 0, -s * 0.45] });
    for (const [h, up, fwd, len] of [
      [0.35, 0.35, 0.42, 0.55],
      [0.62, 0.62, 0.34, 0.5],
      [0.88, 0.88, 0.2, 0.42]
    ]) {
      rig.cone(bone, 0.04, len, [s * (0.1 + h * 0.34), up, 0.1 + fwd * 0.2], BONE, { rot: [1.0, 0, -s * 0.5] });
    }
    rig.box(bone, [0.4, 0.06, 0.08], [s * 0.3, 0.55, -0.1], BRIAR, { rot: [0, 0, -s * 0.5] });
  }
  // Legs: thick upper, slim lower, dark hooves. Vines at the knees.
  for (const side of ['L', 'R'] as const) {
    rig.cyl(`fleg${side}`, 0.15, 0.12, 0.78, [0, -0.39, 0], HIDE);
    rig.cyl(`fshin${side}`, 0.1, 0.07, 0.78, [0, -0.39, 0], DARK);
    rig.box(`fshin${side}`, [0.14, 0.1, 0.22], [0, -0.78, 0.05], DARK);
    rig.box(`fleg${side}`, [0.3, 0.07, 0.2], [0, -0.7, 0], BRIAR);
    rig.cyl(`hleg${side}`, 0.19, 0.12, 0.72, [0, -0.36, 0], HIDE);
    rig.cyl(`hshin${side}`, 0.1, 0.07, 0.72, [0, -0.36, 0.0], DARK);
    rig.box(`hshin${side}`, [0.14, 0.1, 0.22], [0, -0.72, 0.05], DARK);
  }
  return rig;
}

const idle = (t: number): Pose => ({
  body: [0, 0, 0, 0, Math.sin(t * 1.6) * 0.012, 0],
  chest: [Math.sin(t * 1.6) * 0.015, 0, 0],
  neck: [Math.sin(t * 0.7) * 0.04, Math.sin(t * 0.45) * 0.2, 0],
  head: [Math.sin(t * 0.9) * 0.05, Math.sin(t * 0.6) * 0.15, 0],
  tail: [Math.sin(t * 2.2) * 0.12, 0, 0],
  jaw: [0.02 + Math.max(0, Math.sin(t * 0.5)) * 0.06]
});

/** A diagonal-pair gait, heavy and long-striding. */
const walk = (phase: number, amp: number): Pose => {
  const s = Math.sin(phase) * 0.7 * amp;
  const c = Math.cos(phase);
  const bend = (v: number) => Math.max(0, v) * 0.7 * amp;
  return {
    body: [0.03 * Math.sin(phase * 2) * amp, 0, 0, 0, Math.abs(c) * 0.05 * amp, 0],
    flegL: [-s],
    flegR: [s],
    fshinL: [bend(-Math.cos(phase))],
    fshinR: [bend(Math.cos(phase))],
    hlegL: [s],
    hlegR: [-s],
    hshinL: [-bend(-Math.cos(phase)) * 1.1],
    hshinR: [-bend(Math.cos(phase)) * 1.1],
    head: [0.1 * Math.sin(phase * 2) * amp],
    tail: [0, Math.sin(phase) * 0.3 * amp, 0]
  };
};

/** Full gallop at `phase`: fore and hind pairs move together, half a stride apart. */
function gallop(phase: number): Pose {
  const f = Math.sin(phase);
  const h = Math.sin(phase + 2.2);
  const bendF = Math.max(0, Math.cos(phase)) * 1.1;
  const bendH = Math.max(0, Math.cos(phase + 2.2)) * 1.1;
  return {
    body: [-0.08 * f, 0, 0, 0, -0.1 + 0.1 * Math.abs(f), 0],
    chest: [0.12],
    neck: [0.35],
    head: [0.4],
    flegL: [-1.0 * f],
    flegR: [-1.0 * f],
    fshinL: [bendF],
    fshinR: [bendF],
    hlegL: [1.0 * h],
    hlegR: [1.0 * h],
    hshinL: [-bendH],
    hshinR: [-bendH],
    tail: [-0.5],
    jaw: [0.08]
  };
}

/** Front-right hoof raised and dragged back, head low and watching the target. */
const scrapeLift: Pose = {
  body: [0.06, 0, 0, 0, -0.03, 0],
  neck: [0.2],
  head: [0.5],
  flegR: [-0.95],
  fshinR: [1.3],
  flegL: [0.08],
  hlegL: [0.12],
  hlegR: [0.12],
  tail: [0.2]
};
const scrapeDrag: Pose = {
  body: [0.1, 0, 0, 0, -0.06, 0],
  neck: [0.3],
  head: [0.6],
  flegR: [0.45],
  fshinR: [0.1],
  flegL: [0.08],
  hlegL: [0.15],
  hlegR: [0.15]
};
/** The coil before a charge: front low, haunches loaded, antlers aimed. */
const COIL: Pose = {
  body: [0.2, 0, 0, 0, -0.22, 0],
  chest: [0.12],
  neck: [0.55],
  head: [0.75],
  flegL: [0.35],
  flegR: [0.35],
  fshinL: [0.4],
  fshinR: [0.4],
  hlegL: [-0.7],
  hlegR: [-0.7],
  hshinL: [0.9],
  hshinR: [0.9],
  tail: [-0.6],
  jaw: [0.1]
};
/** Braking: forelegs planted forward, haunches down. */
const SKID: Pose = {
  body: [-0.22, 0, 0, 0, -0.14, 0],
  chest: [-0.1],
  neck: [-0.1],
  head: [0.15],
  flegL: [-0.75],
  flegR: [-0.75],
  fshinL: [0.2],
  fshinR: [0.2],
  hlegL: [0.55],
  hlegR: [0.55],
  hshinL: [-0.6],
  hshinR: [-0.6],
  tail: [0.2]
};

/** Keyframes for a charge: the scrapes, the coil, the run, the skid, and the settle. */
function chargeKeys(opts: { scrapes: Array<[lift: number, drag: number]>; coilFrom: number; runFrom: number; runTo: number; skidTo: number }, total: number): Keyframes {
  const keys: Keyframes = [[0, {}]];
  for (const [lift, drag] of opts.scrapes) {
    keys.push([lift, scrapeLift], [drag, scrapeDrag]);
  }
  keys.push([opts.coilFrom, COIL], [opts.runFrom - 1, COIL]);
  for (let f = opts.runFrom; f <= opts.runTo; f += 2) keys.push([f, gallop(f * 0.85)]);
  keys.push([opts.skidTo, SKID], [Math.min(opts.skidTo + 12, total), { ...SKID, body: [-0.1, 0, 0, 0, -0.06, 0] }], [total, {}]);
  return keys;
}

const chargeGlow = (runFrom: number) => (f: number) => (f < runFrom ? 1 + 1.4 * Math.min(f / runFrom, 1) ** 2 : 2.4);

const charge: Choreo = {
  total: 106,
  keys: (total) => chargeKeys({ scrapes: [[4, 10], [16, 22]], coilFrom: 28, runFrom: 30, runTo: 62, skidTo: 70 }, total),
  glow: chargeGlow(30)
};

const charge2: Choreo = {
  total: 72,
  keys: (total) => chargeKeys({ scrapes: [[4, 12]], coilFrom: 24, runFrom: 26, runTo: 54, skidTo: 62 }, total),
  glow: chargeGlow(26)
};

const chargeAgain: Choreo = {
  total: 110,
  keys: (total) => chargeKeys({ scrapes: [[8, 16]], coilFrom: 24, runFrom: 26, runTo: 54, skidTo: 64 }, total),
  glow: chargeGlow(26)
};

const toss: Choreo = {
  total: 66,
  keys: (total) =>
    attack(20, 24, total, {
      // Head dipped to the ground, antlers forward, forelegs braced.
      wind: { body: [0.12, 0, 0, 0, -0.1, 0], chest: [0.12], neck: [0.8], head: [0.8], flegL: [0.2], flegR: [0.2], hlegL: [-0.3], hlegR: [-0.3], jaw: [0.1] },
      // The heave: neck and head thrown up, the forehand rising.
      strike: { body: [-0.18, 0, 0, 0, 0.05, 0], chest: [-0.2], neck: [-0.8], head: [-0.7], flegL: [-0.6], flegR: [-0.6], fshinL: [0.5], fshinR: [0.5], hlegL: [0.3], hlegR: [0.3] }
    }),
  glow: (f) => (f < 20 ? 1 + (f / 20) ** 2 : 1.6)
};

/** Rear, leap in an arc (up to 2.4 m: the sim keeps it on the ground), land on the marker. */
const leap: Choreo = {
  total: 80,
  keys: (total) => [
    [0, {}],
    [5, { body: [-0.45, 0, 0, 0, 0.1, 0], chest: [-0.2], neck: [-0.3], head: [-0.2], flegL: [-1.0], flegR: [-1.0], fshinL: [0.9], fshinR: [0.9], hlegL: [0.4], hlegR: [0.4] }],
    // Takeoff: haunches driving, body pitched up.
    [10, { body: [-0.65, 0, 0, 0, 0.3, 0], chest: [-0.2], neck: [-0.2], head: [0.1], flegL: [-0.7], flegR: [-0.7], fshinL: [1.0], fshinR: [1.0], hlegL: [1.0], hlegR: [1.0], hshinL: [-0.4], hshinR: [-0.4] }],
    [14, { body: [-0.35, 0, 0, 0, 1.4, 0], neck: [0.1], head: [0.3], flegL: [-0.9], flegR: [-0.9], fshinL: [1.2], fshinR: [1.2], hlegL: [0.8], hlegR: [0.8], hshinL: [-0.9], hshinR: [-0.9] }],
    [20, { body: [0.0, 0, 0, 0, 2.4, 0], neck: [0.3], head: [0.45], flegL: [-0.5], flegR: [-0.5], fshinL: [1.4], fshinR: [1.4], hlegL: [0.4], hlegR: [0.4], hshinL: [-1.0], hshinR: [-1.0] }],
    [26, { body: [0.3, 0, 0, 0, 1.1, 0], neck: [0.4], head: [0.6], flegL: [-0.2], flegR: [-0.2], fshinL: [0.4], fshinR: [0.4], hlegL: [0.3], hlegR: [0.3], hshinL: [-0.5], hshinR: [-0.5] }],
    // Landing on the marker (the hit is on frames 30-33): forelegs slam down.
    [30, { body: [0.3, 0, 0, 0, -0.18, 0], chest: [0.15], neck: [0.5], head: [0.55], flegL: [0.15], flegR: [0.15], hlegL: [-0.2], hlegR: [-0.2] }],
    [34, { body: [0.2, 0, 0, 0, -0.14, 0], neck: [0.4], head: [0.5], flegL: [0.1], flegR: [0.1] }],
    [Math.max(total, 40), {}]
  ],
  glow: (f) => (f < 30 ? 1 + 1.2 * Math.min(f / 10, 1) : 1.4)
};

const burst = (grown: boolean): Choreo => ({
  total: grown ? 96 : 90,
  keys: (total) =>
    attack(36, 40, total, {
      // Crouched, shuddering, the thorns standing up.
      wind: { body: [0.1, 0, 0, 0, -0.2, 0], chest: [0.1], neck: [0.5], head: [0.6], flegL: [-0.25], flegR: [-0.25], fshinL: [0.5], fshinR: [0.5], hlegL: [0.3], hlegR: [0.3], hshinL: [-0.5], hshinR: [-0.5], tail: [-0.3] },
      // The burst: the whole body flung upright, the head thrown back.
      strike: { body: [-0.22, 0, 0, 0, 0.1, 0], chest: [-0.15], neck: [-0.55], head: [-0.5], flegL: [-0.2], flegR: [-0.2], hlegL: [0.15], hlegR: [0.15], jaw: [0.4] }
    }),
  glow: (f) => (f < 36 ? 1 + 1.8 * (f / 36) ** 2 : 3.0)
});

const roar: Choreo = {
  total: 45,
  keys: () => {
    const reared: Pose = { body: [-0.6, 0, 0, 0, 0.5, 0], chest: [-0.25], neck: [-0.7], head: [-0.6], flegL: [-1.1], flegR: [-1.1], fshinL: [1.0], fshinR: [1.0], hlegL: [0.5], hlegR: [0.5], hshinL: [-0.4], hshinR: [-0.4], jaw: [0.9], tail: [-0.5] };
    return [
      [0, {}],
      [8, reared],
      [36, { ...reared, body: [-0.55, 0, 0, 0, 0.45, 0] }],
      [45, {}]
    ];
  },
  glow: (f) => 1 + 2.2 * Math.min(f / 10, 1) * (f > 38 ? Math.max(0, (45 - f) / 7) : 1)
};

/** Kneeling after a stun (a charge into a tree, a stance break): forelegs folded, head low. */
function stagger(t: number): Pose {
  const breathe = Math.sin(t * 5) * 0.015;
  return {
    body: [0.28, 0, 0, 0, -0.62 + breathe, 0],
    chest: [0.1],
    neck: [0.45],
    head: [0.7, Math.sin(t * 3) * 0.12, 0],
    flegL: [-1.1],
    flegR: [-1.1],
    fshinL: [2.0],
    fshinR: [2.0],
    hlegL: [0.25],
    hlegR: [0.25],
    jaw: [0.14 + Math.sin(t * 5) * 0.03],
    tail: [0.2]
  };
}

/** Thorn scale state per rig: grown after the roar, until a fresh fight's phase 1 move. */
const grown = new WeakSet<Rig>();

const smooth = (x: number) => {
  const t = Math.min(Math.max(x, 0), 1);
  return t * t * (3 - 2 * t);
};

function tick(rig: Rig, ctx: { frame: number; total: number; inMove: boolean; anim: string }): void {
  const { anim, frame, inMove } = ctx;
  if (inMove) {
    if (anim === 'stag_charge' || anim === 'stag_thorn_burst') grown.delete(rig);
    else if (anim === 'stag_charge_2' || anim === 'stag_charge_again' || anim === 'stag_thorn_burst_2') grown.add(rig);
    else if (anim === 'stag_roar' && frame >= 40) grown.add(rig);
  }
  let len = grown.has(rig) ? GROWN : 1;
  if (inMove && anim === 'stag_roar') len = 1 + (GROWN - 1) * smooth((frame - 10) / 30);
  let bristle = 0;
  if (inMove && (anim === 'stag_thorn_burst' || anim === 'stag_thorn_burst_2')) {
    // Standing up over the 36-frame wind-up, snapping back after the burst.
    bristle = frame < 36 ? smooth(frame / 36) : Math.max(0, 1 - (frame - 36) / 10);
  }
  const l = len * (1 + BRISTLE * bristle);
  const w = 1 + 0.3 * (len - 1) + 0.25 * bristle;
  for (let i = 0; i < THORNS.length; i++) rig.bones.get(`thorn${i}`)?.scale.set(w, l, w);
}

export const briarStagDef: ModelDef = {
  id: 'b02_briar_stag',
  height: 3.3,
  build,
  idle,
  walk,
  stride: 3.0,
  stagger,
  // Exact `anim` names from game/data/moves/bosses/b02.ron.
  moves: {
    stag_charge: charge,
    stag_charge_2: charge2,
    stag_charge_again: chargeAgain,
    stag_toss: toss,
    stag_leap: leap,
    stag_thorn_burst: burst(false),
    stag_thorn_burst_2: burst(true),
    stag_roar: roar
  },
  fallback: toss,
  fallAxis: 'z',
  // Falls onto its side, legs stiff, head down.
  death: (u) => ({ head: [0.4 * u], neck: [0.3 * u], flegL: [-0.5 * u], flegR: [0.4 * u], hlegL: [0.5 * u], hlegR: [-0.4 * u], jaw: [0.5 * u], body: [0, 0, 0, 0, -0.1 * u, 0] }),
  tick
};
