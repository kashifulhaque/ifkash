// Armor sets as tints plus attachments, built from primitives. Each piece is a few parts in the
// "body frame": meters, feet at y = 0, facing +Z, the character's left at +X. A part names the
// bone it rides on (the Wayfarer model's bone names), so the player attaches it to that bone
// and the armory's mannequin just places it. Source: game/data/armor.ron (set names and
// piece ids); the look of each set follows its lore in docs/game/04-arsenal-and-builds.md.
// Render-only.

import type { Group } from 'three';
import { Builder, mix, type V3 } from './build';

export type ArmorSlotName = 'Head' | 'Body';

export interface ArmorPart {
  /** A bone of the Wayfarer model: Head, Chest, Abdomen, UpperArm.L, UpperArm.R. */
  bone: string;
  object: Group;
}

export interface ArmorPieceModel {
  id: string;
  slot: ArmorSlotName;
  parts: ArmorPart[];
  /** Recolors the body's cloth: the player model's `Green` and `LightGreen` materials. */
  tint: { primary: number; secondary: number } | null;
  dispose(): void;
}

interface SetLook {
  /** Cloth or leather. */
  primary: number;
  /** A second cloth tone, for the player's tint. */
  secondary: number;
  /** Metal, fur or lace. */
  trim: number;
}

export const SET_LOOK: Record<string, SetLook> = {
  wayfarer: { primary: 0x9a7f55, secondary: 0x6e5a3c, trim: 0x4a3426 },
  chapel: { primary: 0xe6dfcc, secondary: 0xc9c0a8, trim: 0xd0a64a },
  reedwalker: { primary: 0x5e7a46, secondary: 0x45593a, trim: 0xb6a46a },
  miner: { primary: 0x7b5636, secondary: 0x57402b, trim: 0xc29a3a },
  exile: { primary: 0x3b3449, secondary: 0x2a2535, trim: 0x8f86a8 },
  graymantle: { primary: 0x8a9199, secondary: 0x5d646b, trim: 0x8b2a33 },
  rimeforged: { primary: 0x9fc2d8, secondary: 0x6f93ad, trim: 0xefeff0 },
  boo: { primary: 0xf3f1ea, secondary: 0xf3f1ea, trim: 0x15131c },
  endark: { primary: 0x17141f, secondary: 0x100e16, trim: 0x8d8aa8 }
};

const METAL = { metal: 0.3, rough: 0.55 };

/** The set key for a piece id: `arm_miner_body` gives `miner`; `arm_boo` gives `boo`. */
export function setKey(id: string): string {
  return id.replace(/^arm_/, '').replace(/_(head|body)$/, '');
}

export function armorSlot(id: string): ArmorSlotName {
  if (id.endsWith('_head') || id === 'arm_boo') return 'Head';
  return 'Body';
}

/** Builds one piece, or null for an id the renderer has no look for. */
export function buildArmorPiece(id: string): ArmorPieceModel | null {
  const key = setKey(id);
  const look = SET_LOOK[key];
  if (!look) return null;
  const slot = armorSlot(id);
  const b = new Builder();
  const parts: ArmorPart[] = [];
  const part = (bone: string, pos: V3 = [0, 0, 0]): Group => {
    const g = b.sub(pos, [0, 0, 0], b.group);
    b.group.remove(g);
    parts.push({ bone, object: g });
    return g;
  };
  const fn = (slot === 'Head' ? HEADS : BODIES)[key];
  if (!fn) return null;
  fn(b, look, part);
  return {
    id,
    slot,
    parts,
    tint: slot === 'Body' ? { primary: look.primary, secondary: look.secondary } : null,
    dispose: () => b.dispose()
  };
}

type Maker = (b: Builder, l: SetLook, part: (bone: string, pos?: V3) => Group) => void;

const HEAD_Y = 1.62;
const CHEST_Y = 1.3;

/** A hood or cowl: a squashed sphere shell over the head with a point behind. */
function hood(b: Builder, g: Group, color: number, r = 0.165, back = 0.14): void {
  b.sphere(r, [0, HEAD_Y + 0.02, -0.015], color, { scale: [1, 1.05, 1.1] }, g);
  b.cone(0.1, 0.22, [0, HEAD_Y - 0.05, -r - 0.02], color, { rot: [-1.9, 0, 0] }, g);
  b.box([0.34, 0.05, 0.2], [0, HEAD_Y - 0.14, -0.01], color, {}, g);
  void back;
}

const HEADS: Record<string, Maker> = {
  wayfarer(b, l, part) {
    const g = part('Head');
    hood(b, g, l.primary);
    b.box([0.2, 0.025, 0.02], [0, HEAD_Y + 0.07, 0.15], l.trim, {}, g);
  },
  chapel(b, l, part) {
    const g = part('Head');
    hood(b, g, l.primary, 0.168);
    // A gold circlet over the brow.
    b.arc(0.165, 0.011, Math.PI * 1.9, [0, HEAD_Y + 0.06, 0], l.trim, { rot: [Math.PI / 2, 0, 0], ...METAL }, g);
    b.gem(0.022, [0, HEAD_Y + 0.07, 0.17], 0xfff0c0, { glow: 0.8 }, g);
  },
  reedwalker(b, l, part) {
    const g = part('Head');
    hood(b, g, l.primary);
    // Reed tufts sewn along the hood.
    for (let i = 0; i < 7; i++) {
      const a = -0.9 + i * 0.3;
      b.cone(0.012, 0.2 + (i % 3) * 0.04, [Math.sin(a) * 0.14, HEAD_Y + 0.17, Math.cos(a) * 0.05 - 0.02], l.trim, { rot: [0.2, 0, -Math.sin(a) * 0.6] }, g);
    }
  },
  miner(b, l, part) {
    const g = part('Head');
    // A hard hat with a brim and a lamp.
    b.sphere(0.17, [0, HEAD_Y + 0.05, 0], l.trim, { scale: [1, 0.75, 1.05], ...METAL }, g);
    b.cyl(0.2, 0.2, 0.02, [0, HEAD_Y + 0.04, 0.01], l.trim, METAL, g);
    b.cyl(0.045, 0.05, 0.06, [0, HEAD_Y + 0.1, 0.17], 0x40382c, METAL, g);
    b.sphere(0.034, [0, HEAD_Y + 0.1, 0.205], 0xffd98a, { glow: 1.6 }, g);
  },
  exile(b, l, part) {
    const g = part('Head');
    hood(b, g, l.primary, 0.175);
    // The hood is drawn low, leaving a shadowed face.
    b.box([0.24, 0.12, 0.05], [0, HEAD_Y - 0.01, 0.14], 0x0c0a10, {}, g);
    b.sphere(0.012, [0.045, HEAD_Y - 0.01, 0.17], l.trim, { glow: 1.3 }, g);
    b.sphere(0.012, [-0.045, HEAD_Y - 0.01, 0.17], l.trim, { glow: 1.3 }, g);
  },
  graymantle(b, l, part) {
    const g = part('Head');
    // A closed helm with a visor slit and a crimson plume.
    b.cyl(0.17, 0.165, 0.3, [0, HEAD_Y + 0.02, 0], l.primary, METAL, g);
    b.sphere(0.17, [0, HEAD_Y + 0.17, 0], l.primary, METAL, g);
    b.box([0.22, 0.03, 0.03], [0, HEAD_Y + 0.05, 0.165], 0x14161a, {}, g);
    b.box([0.025, 0.14, 0.03], [0, HEAD_Y - 0.02, 0.17], l.secondary, METAL, g);
    b.box([0.03, 0.12, 0.3], [0, HEAD_Y + 0.3, -0.08], l.trim, {}, g);
    b.cone(0.045, 0.2, [0, HEAD_Y + 0.28, -0.26], l.trim, { rot: [-1.9, 0, 0] }, g);
  },
  rimeforged(b, l, part) {
    const g = part('Head');
    // A steel cap with a nasal bar, a fur rim and forge-orange studs.
    b.sphere(0.17, [0, HEAD_Y + 0.06, 0], l.primary, { scale: [1, 0.85, 1.05], ...METAL }, g);
    b.box([0.025, 0.14, 0.03], [0, HEAD_Y + 0.0, 0.17], l.secondary, METAL, g);
    for (let i = 0; i < 12; i++) {
      const a = (i / 12) * Math.PI * 2;
      b.sphere(0.034, [Math.cos(a) * 0.165, HEAD_Y + 0.075, Math.sin(a) * 0.165], l.trim, {}, g);
    }
    b.sphere(0.014, [0.1, HEAD_Y + 0.15, 0.1], 0xff8a2a, { glow: 1.0 }, g);
    b.sphere(0.014, [-0.1, HEAD_Y + 0.15, 0.1], 0xff8a2a, { glow: 1.0 }, g);
  },
  boo(b, l, part) {
    // A white sheet with two round eye holes: a ghost.
    const g = part('Head');
    b.sphere(0.2, [0, HEAD_Y + 0.02, -0.01], l.primary, { scale: [1, 1.1, 1.05], rough: 0.95 }, g);
    b.cone(0.26, 0.4, [0, HEAD_Y - 0.2, -0.04], l.primary, { rot: [0, 0, 0], rough: 0.95 }, g);
    b.sphere(0.035, [0.065, HEAD_Y + 0.03, 0.19], l.trim, { scale: [1, 1.3, 0.5] }, g);
    b.sphere(0.035, [-0.065, HEAD_Y + 0.03, 0.19], l.trim, { scale: [1, 1.3, 0.5] }, g);
    b.sphere(0.025, [0, HEAD_Y - 0.06, 0.2], l.trim, { scale: [1, 1.4, 0.5] }, g);
  }
};

/** A torso shell: a box over the chest, and a belt. */
function torso(b: Builder, g: Group, color: number, w = 0.4, h = 0.44, d = 0.25): void {
  b.box([w, h, d], [0, CHEST_Y, 0], color, {}, g);
}

const BODIES: Record<string, Maker> = {
  wayfarer(b, l, part) {
    const c = part('Chest');
    torso(b, c, l.primary);
    b.box([0.42, 0.04, 0.27], [0, CHEST_Y - 0.2, 0], l.trim, {}, c);
    // The coat's collar and a short skirt.
    b.box([0.3, 0.06, 0.2], [0, CHEST_Y + 0.23, 0], l.secondary, {}, c);
    const a = part('Abdomen');
    b.box([0.42, 0.3, 0.26], [0, 1.0, 0], l.primary, {}, a);
    b.box([0.1, 0.1, 0.04], [0.14, 1.14, 0.14], l.trim, {}, a);
  },
  chapel(b, l, part) {
    const c = part('Chest');
    torso(b, c, l.primary);
    b.box([0.42, 0.03, 0.27], [0, CHEST_Y + 0.17, 0], l.trim, {}, c);
    b.arc(0.17, 0.014, Math.PI * 1.9, [0, CHEST_Y + 0.23, 0], l.trim, { rot: [Math.PI / 2, 0, 0], ...METAL }, c);
    const a = part('Abdomen');
    // A long tabard hanging to the knees, gold at the hem.
    b.box([0.3, 0.8, 0.03], [0, 0.78, 0.15], l.primary, {}, a);
    b.box([0.3, 0.03, 0.035], [0, 0.4, 0.15], l.trim, {}, a);
    b.box([0.3, 0.8, 0.03], [0, 0.78, -0.15], l.secondary, {}, a);
    b.box([0.025, 0.7, 0.04], [0, 0.78, 0.155], l.trim, {}, a);
  },
  reedwalker(b, l, part) {
    const c = part('Chest');
    torso(b, c, l.primary);
    b.box([0.42, 0.05, 0.27], [0, CHEST_Y - 0.12, 0], l.secondary, {}, c);
    const a = part('Abdomen');
    b.box([0.42, 0.34, 0.26], [0, 0.98, 0], l.secondary, {}, a);
    // Woven reeds layered on each shoulder.
    for (const [bone, s] of [
      ['UpperArm.L', 1],
      ['UpperArm.R', -1]
    ] as const) {
      const sh = part(bone);
      for (let i = 0; i < 5; i++) b.cone(0.016, 0.26, [s * (0.24 + i * 0.012), 1.42 - i * 0.03, (i - 2) * 0.04], l.trim, { rot: [0, 0, -s * (1.25 + i * 0.12)] }, sh);
    }
  },
  miner(b, l, part) {
    const c = part('Chest');
    torso(b, c, l.primary, 0.38, 0.42);
    // Crossed harness straps and a buckle plate.
    b.box([0.05, 0.56, 0.27], [0.06, CHEST_Y, 0], l.secondary, { rot: [0, 0, 0.62] }, c);
    b.box([0.05, 0.56, 0.27], [-0.06, CHEST_Y, 0], l.secondary, { rot: [0, 0, -0.62] }, c);
    b.box([0.08, 0.08, 0.03], [0, CHEST_Y, 0.14], l.trim, METAL, c);
    const a = part('Abdomen');
    b.box([0.42, 0.07, 0.27], [0, 1.08, 0], l.secondary, {}, a);
    // A tool loop with a small pick hanging on the hip.
    b.cyl(0.012, 0.012, 0.26, [0.22, 0.96, 0.02], 0x5b4a2f, {}, a);
    b.box([0.1, 0.03, 0.03], [0.22, 1.1, 0.02], 0x7d848b, METAL, a);
  },
  exile(b, l, part) {
    const c = part('Chest');
    torso(b, c, l.primary, 0.4, 0.44, 0.26);
    b.box([0.36, 0.025, 0.28], [0, CHEST_Y + 0.05, 0], l.trim, {}, c);
    const a = part('Abdomen');
    // A long, ragged robe skirt.
    b.cyl(0.21, 0.34, 0.85, [0, 0.62, 0], l.primary, {}, a);
    for (let i = 0; i < 8; i++) {
      const t = (i / 8) * Math.PI * 2;
      b.cone(0.05, 0.14, [Math.cos(t) * 0.33, 0.14 - (i % 2) * 0.03, Math.sin(t) * 0.33], l.primary, { rot: [Math.PI, 0, 0] }, a);
    }
    b.box([0.18, 0.5, 0.02], [0, 0.7, 0.28], l.secondary, {}, a);
  },
  graymantle(b, l, part) {
    const c = part('Chest');
    // A breastplate with a raised ridge, a gorget, and a crimson tabard strip.
    torso(b, c, l.primary, 0.44, 0.46, 0.28);
    b.box([0.06, 0.44, 0.06], [0, CHEST_Y, 0.15], l.secondary, METAL, c);
    b.cyl(0.14, 0.17, 0.08, [0, CHEST_Y + 0.26, 0], l.primary, METAL, c);
    const a = part('Abdomen');
    b.box([0.2, 0.55, 0.02], [0, 0.82, 0.17], l.trim, {}, a);
    b.box([0.44, 0.1, 0.28], [0, 1.08, 0], l.secondary, METAL, a);
    for (const [bone, s] of [
      ['UpperArm.L', 1],
      ['UpperArm.R', -1]
    ] as const) {
      const sh = part(bone);
      b.sphere(0.13, [s * 0.27, 1.42, 0], l.primary, { scale: [1, 0.7, 1.05], ...METAL }, sh);
      b.box([0.1, 0.03, 0.2], [s * 0.3, 1.34, 0], l.secondary, METAL, sh);
    }
  },
  rimeforged(b, l, part) {
    const c = part('Chest');
    torso(b, c, l.primary, 0.42, 0.46, 0.27);
    // Mail rings suggested by banded ridges, and a heavy fur collar.
    for (let i = 0; i < 4; i++) b.box([0.43, 0.012, 0.275], [0, CHEST_Y - 0.15 + i * 0.1, 0], l.secondary, METAL, c);
    for (let i = 0; i < 10; i++) {
      const t = (i / 10) * Math.PI * 2;
      b.sphere(0.05, [Math.cos(t) * 0.17, CHEST_Y + 0.25, Math.sin(t) * 0.15], l.trim, {}, c);
    }
    b.sphere(0.02, [0, CHEST_Y + 0.05, 0.14], 0xff8a2a, { glow: 1.0 }, c);
    const a = part('Abdomen');
    b.cyl(0.21, 0.28, 0.3, [0, 0.95, 0], l.secondary, METAL, a);
    for (const [bone, s] of [
      ['UpperArm.L', 1],
      ['UpperArm.R', -1]
    ] as const) {
      const sh = part(bone);
      b.sphere(0.11, [s * 0.26, 1.42, 0], l.primary, { scale: [1, 0.65, 1.05], ...METAL }, sh);
      b.sphere(0.07, [s * 0.28, 1.46, 0], l.trim, {}, sh);
    }
  },
  endark(b, l, part) {
    const c = part('Chest');
    torso(b, c, l.primary, 0.42, 0.46, 0.27);
    // A long dark cloak down the back, flecked with faint stars.
    const back = part('Abdomen');
    b.box([0.46, 1.1, 0.03], [0, 1.0, -0.17], l.secondary, { rough: 0.95 }, back);
    b.box([0.5, 0.1, 0.1], [0, 1.52, -0.13], l.primary, {}, back);
    for (let i = 0; i < 9; i++) b.gem(0.012, [-0.18 + ((i * 37) % 9) * 0.045, 0.6 + ((i * 53) % 9) * 0.07, -0.19], l.trim, { glow: 1.0 }, back);
    b.box([0.2, 0.03, 0.04], [0, CHEST_Y + 0.2, 0.14], mix(l.trim, 0x000000, 0.4), {}, c);
  }
};

/** Piece ids that have a look, for the armory's coverage check. */
export const ARMOR_LOOK_KEYS = Object.keys(SET_LOOK);
