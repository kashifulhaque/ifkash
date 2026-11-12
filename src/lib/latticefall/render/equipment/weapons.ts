// Weapon meshes, one builder per weapon class, built from primitives (no assets). A weapon is
// drawn with its grip at the origin, the blade or head along +Y and the flat of the blade
// facing +Z. Individual weapons tint the class shape from a per-id style table, with a
// deterministic fallback so every id (test weapons too) looks distinct. Spec:
// docs/game/07-presentation.md, "Characters and animation". Render-only.

import type { Group } from 'three';
import type { WeaponClass } from '../../gen/WeaponClass';
import { Builder, hash01, mix, type V3 } from './build';

export interface WeaponSpec {
  id: string;
  name: string;
  class: WeaponClass;
  unique: boolean;
  /** `Aether`, `Fire`, `Storm` or `Radiance`, from the weapon's elemental part. */
  element: string | null;
  /** `Formula` (stylus) or `Litany` (reliquary); catalysts only. */
  school: string | null;
  weightTenths: number;
}

export interface WeaponModel {
  group: Group;
  /** Overall length along +Y from the grip, meters, for framing and labels. */
  length: number;
  /** The glow color (0xRRGGBB) the weapon's effects use, or null for a plain weapon. */
  glow: number | null;
  /** Where a spell's effect starts, in the weapon's own frame; null off a catalyst. */
  castPoint: V3 | null;
  dispose(): void;
}

export const ELEMENT_COLOR: Record<string, number> = {
  Aether: 0x86a8ff,
  Fire: 0xff7f2a,
  Storm: 0x9fe6ff,
  Radiance: 0xffe7a0
};
export const SCHOOL_COLOR: Record<string, number> = { Formula: 0x93a6ff, Litany: 0xffc766 };

const STEEL = [0xb9bec4, 0xa7aeb6, 0xc4c8cc, 0x9aa3ab];
const DARK_STEEL = [0x5d646b, 0x4d545b, 0x6a6258];
const LEATHER = 0x4a3426;
const WOOD = 0x7a5836;
const GOLD = 0xd0a64a;
const BRASS = 0xa88445;

interface Style {
  blade?: number;
  trim?: number;
  grip?: number;
  /** Uniform size multiplier. */
  scale?: number;
  /** Which shape variant of the class to draw. */
  variant?: string;
}

/** Looks that follow the weapon's name and lore, not its class. */
const STYLE: Record<string, Style> = {
  wpn_wayfarer_blade: { blade: 0xb4b9be, trim: BRASS },
  wpn_chapel_longsword: { blade: 0xefe9d8, trim: GOLD },
  wpn_lantern_sword: { blade: 0xc7b79a, trim: 0xd9822b },
  wpn_banana_cpp: { blade: 0xf0d34a, trim: 0x5b7f2b, variant: 'banana' },
  wpn_reedcutter: { blade: 0xa9bfa0, grip: 0x556b3c },
  wpn_stillwater_rapier: { blade: 0xc9d4dc, trim: 0x6f8fa8 },
  wpn_glassneedle_estoc: { blade: 0xbfe0ee, trim: 0x86a8ff, variant: 'glass' },
  wpn_gray_greatsword: { blade: 0x8b9199, trim: 0x6e1f26 },
  wpn_rimeforged_claymore: { blade: 0xaed5ec, trim: 0xd9822b },
  wpn_fungal_knife: { blade: 0xb9b0d6, grip: 0x5d3f5f },
  wpn_parrying_dirk: { blade: 0xc4c8cc, trim: BRASS },
  wpn_whisperfang: { blade: 0x8e86a8, trim: 0x2d2840 },
  wpn_woodcutter_axe: { blade: 0x9aa3ab, grip: WOOD },
  wpn_briar_axe: { blade: 0x7e8f6d, grip: 0x5b4a2f },
  wpn_miners_pick: { blade: 0x7d848b, variant: 'pick' },
  wpn_morning_star: { blade: 0x80868c, variant: 'morningstar' },
  wpn_bellwright_mallet: { blade: 0xb08a4a, variant: 'bell' },
  wpn_forgeheart_maul: { blade: 0x4a4038, trim: 0xff7f2a, variant: 'maul' },
  wpn_greatanchor: { blade: 0x5b6670, variant: 'anchor' },
  wpn_ferray: { blade: 0xd8e4ec, trim: 0x9fe6ff },
  wpn_shardglass_spear: { blade: 0xbfe0ee, trim: 0x86a8ff, variant: 'glass' },
  wpn_gray_halberd: { blade: 0x8b9199, trim: 0x6e1f26 },
  wpn_choir_glaive: { blade: 0xefe9d8, trim: GOLD, variant: 'glaive' },
  wpn_bramble_claws: { blade: 0x5f7a46, variant: 'claws' },
  wpn_iron_knuckles: { blade: 0x7d848b },
  wpn_hunter_shortbow: { grip: WOOD },
  wpn_spire_longbow: { grip: 0x9a8a62, trim: GOLD, scale: 1.2 },
  wpn_plank_buckler: { blade: 0x8a6a45, variant: 'buckler' },
  wpn_nopokedb: { blade: 0x5d7fa3, trim: 0xe2c25a, variant: 'heater' },
  wpn_castle_kite: { blade: 0x8b9199, trim: 0x6e1f26, variant: 'kite' },
  wpn_graymantle_greatshield: { blade: 0x6c737b, trim: 0x6e1f26, variant: 'tower' },
  wpn_scholars_stylus: { trim: 0x6d76d6 },
  wpn_exile_stylus: { blade: 0x3d3a52, trim: 0x93a6ff, variant: 'crooked' },
  wpn_gradient_rod: { blade: 0xe8e8f0, trim: 0x93a6ff, variant: 'rod' },
  wpn_pilgrims_reliquary: { trim: BRASS },
  wpn_chapel_reliquary: { blade: 0xefe9d8, trim: GOLD, variant: 'censer' },
  wpn_whitewarden_seal: { blade: 0xe3dcc6, trim: 0x8a5a38, variant: 'seal' }
};

/** Builds the mesh for one weapon. */
export function buildWeapon(spec: WeaponSpec): WeaponModel {
  const b = new Builder();
  const st = STYLE[spec.id] ?? {};
  const h = hash01(spec.id);
  const steel = STEEL[Math.floor(h * STEEL.length) % STEEL.length];
  const blade = st.blade ?? steel;
  const trim = st.trim ?? (spec.unique ? GOLD : BRASS);
  const grip = st.grip ?? LEATHER;
  const glow = spec.element ? ELEMENT_COLOR[spec.element] : spec.school ? SCHOOL_COLOR[spec.school] : null;
  const c: Ctx = { b, blade, trim, grip, glow: spec.element ? ELEMENT_COLOR[spec.element] : null, variant: st.variant ?? '', h };
  let length = 1;
  let cast: V3 | null = null;
  switch (spec.class) {
    case 'Dagger':
      length = dagger(c);
      break;
    case 'StraightSword':
      length = straightSword(c, 0.82, 0.062, 0.22);
      break;
    case 'Greatsword':
      length = straightSword(c, 1.2, 0.1, 0.32);
      break;
    case 'CurvedSword':
      length = curvedSword(c);
      break;
    case 'ThrustingSword':
      length = thrustingSword(c);
      break;
    case 'Axe':
      length = axe(c);
      break;
    case 'Hammer':
      length = hammer(c);
      break;
    case 'Colossal':
      length = colossal(c);
      break;
    case 'Spear':
      length = spear(c);
      break;
    case 'Halberd':
      length = halberd(c);
      break;
    case 'Fist':
      length = fist(c);
      break;
    case 'Bow':
      length = bow(c);
      break;
    case 'Shield':
      length = shield(c);
      break;
    case 'Catalyst': {
      const r = catalyst(c, spec.school ?? 'Formula');
      length = r.length;
      cast = r.cast;
      break;
    }
  }
  if (st.scale) b.group.scale.setScalar(st.scale);
  // Heavy weapons read a touch larger; light ones a touch smaller.
  b.group.name = spec.id;
  return {
    group: b.group,
    length: length * (st.scale ?? 1),
    glow,
    castPoint: cast ? (cast.map((v) => v * (st.scale ?? 1)) as V3) : null,
    dispose: () => b.dispose()
  };
}

interface Ctx {
  b: Builder;
  blade: number;
  trim: number;
  grip: number;
  /** The element glow, set only for an elemental weapon. */
  glow: number | null;
  variant: string;
  h: number;
}

const METAL = { metal: 0.3, rough: 0.6 };

/** The glow strength a blade gets from its element. */
const edgeGlow = (c: Ctx) => (c.glow ? { glow: 0.55 } : {});

/** Grip, pommel and crossguard shared by the swords. Returns the y where the blade starts. */
function hilt(c: Ctx, gripLen: number, guardW: number): number {
  const { b } = c;
  b.cyl(0.017, 0.017, gripLen, [0, -gripLen / 2 + 0.02, 0], c.grip);
  b.sphere(0.03, [0, -gripLen + 0.015, 0], c.trim, METAL);
  b.box([guardW, 0.026, 0.045], [0, 0.03, 0], c.trim, METAL);
  return 0.043;
}

function bladeBox(c: Ctx, y0: number, len: number, w: number, thick = 0.012): void {
  const { b } = c;
  const body = len * 0.88;
  b.box([w, body, thick], [0, y0 + body / 2, 0], c.blade, { ...METAL, ...edgeGlow(c) });
  // The point: a four-sided cone squashed thin, so it reads as an edge from the front.
  const tip = b.cone(w / 2, len * 0.12 + 0.02, [0, y0 + body + (len * 0.12 + 0.02) / 2, 0], c.blade, { ...METAL, ...edgeGlow(c) });
  tip.scale.z = (thick / w) * 2;
  // A fuller down the middle.
  b.box([w * 0.22, body * 0.8, thick * 1.5], [0, y0 + body / 2, 0], mix(c.blade, 0x404850, 0.35), METAL);
}

function dagger(c: Ctx): number {
  const y0 = hilt(c, 0.12, 0.08);
  if (c.variant === '') bladeBox(c, y0, 0.24, 0.036, 0.01);
  return y0 + 0.24;
}

function straightSword(c: Ctx, len: number, w: number, guard: number): number {
  const y0 = hilt(c, len > 1 ? 0.2 : 0.14, guard);
  if (c.variant === 'banana') {
    // banana.cpp: a joke blade in the shape of a banana, stem and all.
    bladeBox(c, y0, len, w, 0.014);
    c.b.cone(0.02, 0.05, [0, y0 + len + 0.06, 0], 0x5b7f2b);
  } else bladeBox(c, y0, len, w);
  if (len > 1) c.b.box([0.05, 0.05, 0.05], [0, 0.07, 0], c.trim, METAL);
  return y0 + len;
}

function curvedSword(c: Ctx): number {
  const { b } = c;
  const y0 = hilt(c, 0.14, 0.1);
  const segs = 6;
  const len = 0.84;
  let x = 0;
  let y = y0;
  for (let i = 0; i < segs; i++) {
    const a = (i / segs) * 0.7;
    const l = len / segs;
    const w = 0.07 * (1 - i / segs / 1.5);
    b.box([0.012, l * 1.08, w], [x, y + (l * Math.cos(a)) / 2, 0], c.blade, { rot: [0, 0, -a], ...METAL, ...edgeGlow(c) });
    // The curve bends toward +X; the box is thin in X, so the flat faces the side.
    y += l * Math.cos(a);
    x += l * Math.sin(a);
  }
  return y;
}

function thrustingSword(c: Ctx): number {
  const { b } = c;
  b.cyl(0.015, 0.015, 0.14, [0, -0.05, 0], c.grip);
  b.sphere(0.026, [0, -0.13, 0], c.trim, METAL);
  // A swept cup guard.
  b.sphere(0.06, [0, 0.03, 0], c.trim, { ...METAL, scale: [1, 0.55, 1] });
  b.box([0.17, 0.014, 0.014], [0, 0.04, 0], c.trim, METAL);
  const len = 0.96;
  b.cyl(0.004, 0.011, len, [0, 0.06 + len / 2, 0], c.blade, { ...METAL, ...edgeGlow(c) });
  return 0.06 + len;
}

function axe(c: Ctx): number {
  const { b } = c;
  const len = 0.78;
  b.cyl(0.02, 0.022, len, [0, len / 2 - 0.14, 0], c.grip === LEATHER ? WOOD : c.grip);
  b.sphere(0.028, [0, -0.15, 0], c.trim, METAL);
  const top = len - 0.14;
  // A wedge head: a thick shoulder and a wide bit.
  b.box([0.06, 0.2, 0.05], [0.03, top - 0.08, 0], c.blade, METAL);
  c.b.shape(
    [
      [0, 0.1],
      [0.2, 0.17],
      [0.2, -0.17],
      [0, -0.1]
    ],
    0.022,
    [0.04, top - 0.08, 0],
    c.blade,
    { ...METAL, ...edgeGlow(c) }
  );
  if (c.variant === '' && c.h > 0.5) b.cone(0.025, 0.1, [-0.07, top - 0.08, 0], c.blade, { rot: [0, 0, Math.PI / 2], ...METAL });
  return top + 0.1;
}

function hammer(c: Ctx): number {
  const { b } = c;
  const len = c.variant === 'morningstar' ? 0.75 : 0.85;
  b.cyl(0.022, 0.024, len, [0, len / 2 - 0.15, 0], WOOD);
  b.sphere(0.03, [0, -0.16, 0], c.trim, METAL);
  const top = len - 0.15;
  if (c.variant === 'morningstar') {
    b.sphere(0.085, [0, top + 0.05, 0], c.blade, METAL);
    for (let i = 0; i < 8; i++) {
      const a = (i / 8) * Math.PI * 2;
      b.cone(0.017, 0.07, [Math.cos(a) * 0.09, top + 0.05, Math.sin(a) * 0.09], c.blade, { rot: [Math.sin(a) * 1.57, 0, -Math.cos(a) * 1.57], ...METAL });
    }
    b.cone(0.017, 0.07, [0, top + 0.14, 0], c.blade, METAL);
    b.cone(0.017, 0.07, [0, top - 0.04, 0], c.blade, { rot: [Math.PI, 0, 0], ...METAL });
  } else if (c.variant === 'pick') {
    b.box([0.06, 0.06, 0.06], [0, top, 0], c.blade, METAL);
    b.cone(0.03, 0.26, [0.15, top, 0], c.blade, { rot: [0, 0, -Math.PI / 2], ...METAL });
    b.cone(0.03, 0.26, [-0.15, top, 0], c.blade, { rot: [0, 0, Math.PI / 2], ...METAL });
  } else if (c.variant === 'bell') {
    // The Bellwright's Mallet: a bell-shaped head.
    b.cyl(0.07, 0.13, 0.17, [0, top + 0.02, 0], c.blade, { ...METAL, ...edgeGlow(c) });
    b.sphere(0.07, [0, top + 0.1, 0], c.blade, METAL);
    b.cyl(0.04, 0.04, 0.03, [0, top - 0.08, 0], c.trim, METAL);
  } else {
    b.box([0.25, 0.14, 0.14], [0, top + 0.03, 0], c.blade, { ...METAL, ...edgeGlow(c) });
    b.box([0.27, 0.03, 0.16], [0, top + 0.1, 0], c.trim, METAL);
    b.box([0.27, 0.03, 0.16], [0, top - 0.04, 0], c.trim, METAL);
  }
  return top + 0.15;
}

function colossal(c: Ctx): number {
  const { b } = c;
  if (c.variant === 'maul') {
    // The Forgeheart Maul: a furnace-block head with a glowing seam.
    b.cyl(0.03, 0.034, 1.25, [0, 0.4, 0], 0x3c3027);
    b.sphere(0.045, [0, -0.25, 0], c.trim, METAL);
    b.box([0.46, 0.3, 0.3], [0, 1.1, 0], c.blade, METAL);
    b.box([0.48, 0.05, 0.31], [0, 1.1, 0], c.trim, { glow: 0.9 });
    b.box([0.5, 0.05, 0.32], [0, 1.26, 0], 0x2a241f, METAL);
    b.box([0.5, 0.05, 0.32], [0, 0.94, 0], 0x2a241f, METAL);
    return 1.3;
  }
  if (c.variant === 'anchor') {
    // The Greatanchor: an anchor wielded by its shank.
    b.cyl(0.035, 0.035, 1.35, [0, 0.45, 0], c.blade, METAL);
    b.sphere(0.07, [0, -0.25, 0], c.blade, METAL);
    b.box([0.5, 0.05, 0.05], [0, 1.0, 0], c.blade, METAL);
    b.arc(0.28, 0.035, Math.PI, [0, 1.32, 0], c.blade, { rot: [0, 0, Math.PI], ...METAL });
    b.cone(0.05, 0.14, [0.28, 1.2, 0], c.blade, { rot: [0, 0, 0.2], ...METAL });
    b.cone(0.05, 0.14, [-0.28, 1.2, 0], c.blade, { rot: [0, 0, -0.2], ...METAL });
    return 1.5;
  }
  // A slab of a sword.
  b.cyl(0.024, 0.024, 0.34, [0, -0.05, 0], c.grip);
  b.box([0.4, 0.05, 0.08], [0, 0.14, 0], c.trim, METAL);
  bladeBox(c, 0.17, 1.35, 0.19, 0.03);
  return 1.52;
}

function spear(c: Ctx): number {
  const { b } = c;
  const len = 1.75;
  b.cyl(0.017, 0.019, len, [0, len / 2 - 0.55, 0], WOOD);
  b.cone(0.022, 0.06, [0, -0.58, 0], c.trim, { rot: [Math.PI, 0, 0], ...METAL });
  b.cyl(0.026, 0.02, 0.07, [0, len - 0.6, 0], c.trim, METAL);
  // A leaf-shaped head.
  b.shape(
    [
      [0, 0],
      [0.055, 0.07],
      [0.04, 0.17],
      [0, 0.3],
      [-0.04, 0.17],
      [-0.055, 0.07]
    ],
    0.012,
    [0, len - 0.57, 0],
    c.blade,
    { ...METAL, ...edgeGlow(c) }
  );
  if (c.glow) b.box([0.12, 0.012, 0.012], [0, len - 0.6, 0], c.glow, { glow: 0.8 });
  return len - 0.27;
}

function halberd(c: Ctx): number {
  const { b } = c;
  const len = 1.6;
  b.cyl(0.02, 0.022, len, [0, len / 2 - 0.5, 0], WOOD);
  b.cone(0.023, 0.06, [0, -0.53, 0], c.trim, { rot: [Math.PI, 0, 0], ...METAL });
  const top = len - 0.5;
  b.cone(0.03, 0.24, [0, top + 0.12, 0], c.blade, { ...METAL, ...edgeGlow(c) });
  if (c.variant === 'glaive') {
    // A single curved blade, like a long cleaver.
    b.shape(
      [
        [0, -0.2],
        [0.02, 0.35],
        [0.1, 0.5],
        [-0.04, 0.46],
        [-0.04, -0.2]
      ],
      0.014,
      [0.0, top - 0.05, 0],
      c.blade,
      { ...METAL, ...edgeGlow(c) }
    );
  } else {
    // Axe blade on one side, a hook on the other.
    b.shape(
      [
        [0, 0.12],
        [0.22, 0.2],
        [0.22, -0.12],
        [0, -0.06]
      ],
      0.016,
      [0.02, top - 0.05, 0],
      c.blade,
      { ...METAL, ...edgeGlow(c) }
    );
    b.cone(0.02, 0.15, [-0.08, top - 0.03, 0], c.blade, { rot: [0, 0, Math.PI / 2 + 0.3], ...METAL });
  }
  b.cyl(0.03, 0.03, 0.05, [0, top - 0.15, 0], c.trim, METAL);
  return top + 0.24;
}

function fist(c: Ctx): number {
  const { b } = c;
  if (c.variant === 'claws') {
    // Bramble Claws: a leather cuff with three thorn claws along the fingers.
    b.cyl(0.05, 0.05, 0.1, [0, 0, 0], c.grip);
    for (let i = -1; i <= 1; i++) b.cone(0.012, 0.22, [i * 0.032, 0.17, 0.02], c.blade, { rot: [0.25, 0, 0], ...METAL });
    return 0.28;
  }
  // Knuckles: a bar across the fist with four rings.
  b.box([0.11, 0.03, 0.03], [0, 0.03, 0.03], c.blade, METAL);
  for (let i = 0; i < 4; i++) b.arc(0.017, 0.006, Math.PI * 1.7, [-0.04 + i * 0.027, 0.0, 0.03], c.blade, { rot: [Math.PI / 2, 0, 0], ...METAL });
  b.box([0.12, 0.05, 0.04], [0, -0.02, -0.02], c.grip);
  for (let i = 0; i < 4; i++) b.cone(0.009, 0.03, [-0.04 + i * 0.027, 0.06, 0.03], c.blade, { ...(c.glow ? { glow: 0.5 } : {}), ...METAL });
  return 0.12;
}

function bow(c: Ctx): number {
  const { b } = c;
  const wood = c.grip === LEATHER ? WOOD : c.grip;
  const r = 0.55;
  const span = Math.PI * 0.62;
  // The limbs: an arc in the XY plane bulging toward +X, turned at the end to bulge toward +Z
  // (the way the wielder faces), with the string across its chord.
  const limb = b.sub();
  b.arc(r, 0.014, span, [-r, 0, 0], wood, { rot: [0, 0, -span / 2] }, limb);
  b.box([0.03, 0.14, 0.03], [0, 0, 0], c.grip, {}, limb);
  const half = r * Math.sin(span / 2);
  const chordX = -r + r * Math.cos(span / 2);
  b.cyl(0.003, 0.003, half * 2, [chordX, 0, 0], 0xe8e0cc, {}, limb);
  b.sphere(0.014, [chordX, half, 0], c.trim, {}, limb);
  b.sphere(0.014, [chordX, -half, 0], c.trim, {}, limb);
  limb.rotation.y = -Math.PI / 2;
  return half * 2;
}

function shield(c: Ctx): number {
  const { b } = c;
  const v = c.variant;
  // Held on the left forearm: the face is toward +Z, the grip behind it. The origin is the
  // grip, so the face center sits at the origin.
  const face = (outline: Array<[number, number]>, depth: number) => b.shape(outline, depth, [0, 0.0, 0.05], c.blade, METAL);
  let h = 0.6;
  if (v === 'buckler') {
    b.cyl(0.21, 0.21, 0.05, [0, 0, 0.05], c.blade, { rot: [Math.PI / 2, 0, 0], ...METAL });
    b.cyl(0.05, 0.05, 0.07, [0, 0, 0.1], c.trim, { rot: [Math.PI / 2, 0, 0], ...METAL });
    for (let i = 0; i < 8; i++) {
      const a = (i / 8) * Math.PI * 2;
      b.sphere(0.014, [Math.cos(a) * 0.17, Math.sin(a) * 0.17, 0.08], c.trim, METAL);
    }
    h = 0.42;
  } else if (v === 'kite') {
    face([
      [-0.22, 0.33],
      [0.22, 0.33],
      [0.2, -0.05],
      [0, -0.48],
      [-0.2, -0.05]
    ], 0.05);
    b.box([0.05, 0.7, 0.012], [0, -0.05, 0.08], c.trim, METAL);
    b.box([0.34, 0.05, 0.012], [0, 0.12, 0.08], c.trim, METAL);
    h = 0.82;
  } else if (v === 'tower') {
    face([
      [-0.3, 0.52],
      [0.3, 0.52],
      [0.3, -0.5],
      [-0.3, -0.5]
    ], 0.07);
    b.box([0.6, 0.04, 0.012], [0, 0.0, 0.1], c.trim, METAL);
    b.box([0.04, 1.0, 0.012], [0, 0.0, 0.1], c.trim, METAL);
    b.cyl(0.07, 0.07, 0.03, [0, 0.0, 0.11], c.trim, { rot: [Math.PI / 2, 0, 0], ...METAL });
    h = 1.04;
  } else if (v === 'heater') {
    face([
      [-0.22, 0.26],
      [0.22, 0.26],
      [0.22, 0.0],
      [0, -0.4],
      [-0.22, 0.0]
    ], 0.04);
    b.box([0.34, 0.05, 0.012], [0, 0.12, 0.06], c.trim, METAL);
    h = 0.66;
  } else {
    // A plain round shield.
    b.cyl(0.26, 0.26, 0.05, [0, 0, 0.05], c.blade, { rot: [Math.PI / 2, 0, 0], ...METAL });
    b.cyl(0.07, 0.07, 0.07, [0, 0, 0.1], c.trim, { rot: [Math.PI / 2, 0, 0], ...METAL });
    h = 0.52;
  }
  // The strap and handle behind the face.
  b.box([0.04, 0.16, 0.03], [0, 0, -0.02], c.grip);
  return h;
}

function catalyst(c: Ctx, school: string): { length: number; cast: V3 } {
  const { b } = c;
  const glow = SCHOOL_COLOR[school] ?? SCHOOL_COLOR.Formula;
  if (school === 'Litany') {
    // A reliquary: a short staff topped with a small glazed shrine around a candle flame.
    const len = c.variant === 'censer' || c.variant === 'seal' ? 0.62 : 0.78;
    b.cyl(0.018, 0.022, len, [0, len / 2 - 0.1, 0], mix(WOOD, c.blade, 0.2));
    b.sphere(0.03, [0, -0.11, 0], c.trim, METAL);
    const top = len - 0.1;
    if (c.variant === 'seal') {
      // The Whitewarden Seal: a bone disc hung on a short haft.
      b.cyl(0.11, 0.11, 0.025, [0, top + 0.12, 0], c.blade, { rot: [Math.PI / 2, 0, 0] });
      b.cyl(0.05, 0.05, 0.03, [0, top + 0.12, 0], glow, { rot: [Math.PI / 2, 0, 0], glow: 0.9 });
      return { length: top + 0.24, cast: [0, top + 0.12, 0.08] };
    }
    b.cyl(0.045, 0.035, 0.05, [0, top, 0], c.trim, METAL);
    b.cone(0.07, 0.07, [0, top + 0.19, 0], c.trim, METAL);
    for (const [sx, sz] of [
      [1, 1],
      [-1, 1],
      [1, -1],
      [-1, -1]
    ])
      b.cyl(0.006, 0.006, 0.13, [sx * 0.048, top + 0.095, sz * 0.048], c.trim, METAL);
    b.cyl(0.011, 0.011, 0.06, [0, top + 0.08, 0], 0xfff0cf);
    b.cone(0.017, 0.05, [0, top + 0.14, 0], glow, { glow: 1.1 });
    return { length: top + 0.23, cast: [0, top + 0.14, 0] };
  }
  // A stylus: a slim rod with a floating crystal above the nib.
  const len = c.variant === 'rod' ? 0.95 : 0.7;
  b.cyl(0.011, 0.016, len, [0, len / 2 - 0.1, 0], c.variant === 'crooked' ? c.blade : mix(0x3a3550, c.trim, 0.2));
  b.cyl(0.02, 0.02, 0.05, [0, 0.0, 0], c.trim, METAL);
  b.cone(0.012, 0.05, [0, -0.13, 0], c.trim, { rot: [Math.PI, 0, 0], ...METAL });
  const top = len - 0.1;
  b.cone(0.022, 0.07, [0, top + 0.02, 0], c.trim, METAL);
  b.gem(0.04, [0, top + 0.12, 0], glow, { glow: 1.0 });
  b.gem(0.018, [0.065, top + 0.1, 0.02], glow, { glow: 1.0 });
  b.gem(0.014, [-0.06, top + 0.15, -0.02], glow, { glow: 1.0 });
  return { length: top + 0.17, cast: [0, top + 0.12, 0] };
}
