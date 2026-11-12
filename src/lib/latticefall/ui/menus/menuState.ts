// The menus' view of the character: a plain object parsed from the floats `LfWorld.menu_state`
// returns (game/crates/wasm/src/menu.rs), plus the previews the core computes. Pure and
// read-only, with no Three.js or WASM imports. Every number here comes from the core; the
// menus format them and never recompute them.

/** Floats in the fixed part of a `menu_state` record. Must equal `menu_words()`. */
export const MENU_WORDS = 45;
/** Floats in a `preview_equip` record. Must equal `loadout_words()`. */
export const LOADOUT_WORDS = 8;

/** Equipment slot numbers, as in `lf_core::items::slot`. */
export const SLOT = {
  RIGHT_1: 0,
  RIGHT_2: 1,
  LEFT_1: 2,
  LEFT_2: 3,
  HEAD: 4,
  BODY: 5,
  CHARM_1: 6,
  QUICK_1: 10,
  ARROWS: 15
} as const;

export interface SlotDef {
  slot: number;
  label: string;
  group: 'Hands' | 'Armor' | 'Charms' | 'Quick items' | 'Arrows';
}

export const SLOT_DEFS: SlotDef[] = [
  { slot: 0, label: 'Right hand 1', group: 'Hands' },
  { slot: 1, label: 'Right hand 2', group: 'Hands' },
  { slot: 2, label: 'Left hand 1', group: 'Hands' },
  { slot: 3, label: 'Left hand 2', group: 'Hands' },
  { slot: 4, label: 'Head', group: 'Armor' },
  { slot: 5, label: 'Body', group: 'Armor' },
  { slot: 6, label: 'Charm 1', group: 'Charms' },
  { slot: 7, label: 'Charm 2', group: 'Charms' },
  { slot: 8, label: 'Charm 3', group: 'Charms' },
  { slot: 9, label: 'Charm 4', group: 'Charms' },
  { slot: 10, label: 'Quick item 1', group: 'Quick items' },
  { slot: 11, label: 'Quick item 2', group: 'Quick items' },
  { slot: 12, label: 'Quick item 3', group: 'Quick items' },
  { slot: 13, label: 'Quick item 4', group: 'Quick items' },
  { slot: 14, label: 'Quick item 5', group: 'Quick items' },
  { slot: 15, label: 'Arrows', group: 'Arrows' }
];

/** The attributes, in the order of `Command::LevelUp`'s operand. */
export const ATTRIBUTES = ['Vitality', 'Focus', 'Endurance', 'Might', 'Finesse', 'Insight', 'Devotion'] as const;

export const TIER_NAMES = ['Light', 'Medium', 'Heavy', 'Overloaded'] as const;

export interface Flasks {
  maxCharges: number;
  vigilMax: number;
  vigil: number;
  lucid: number;
  level: number;
  /** The most charges the flasks can ever hold. */
  charLimit: number;
}

export interface LanternEntry {
  index: number;
  lit: boolean;
}

export interface MenuState {
  /** The registry number in each slot, by slot number; 0 for empty. */
  slots: number[];
  charmSlots: number;
  glimmer: number;
  attributes: number[];
  level: number;
  /** Glimmer the next level costs. */
  levelCost: number;
  attributeMax: number;
  flasks: Flasks;
  /** The Lantern menu is open: the player is resting. */
  resting: boolean;
  /** A boss fight is running. */
  bossFight: boolean;
  dead: boolean;
  /** Registry numbers of the Vigil and Lucid flasks; 0 if the content has none. */
  flaskItems: [number, number];
  /** Index of the Lantern rested at, or -1. */
  lantern: number;
  lanterns: LanternEntry[];
  /** Owned weapons, armor, and charms by registry number. */
  gear: number[];
  /** Consumable stacks. */
  stacks: { item: number; count: number }[];
}

export function emptyMenuState(): MenuState {
  return {
    slots: Array.from({ length: 16 }, () => 0),
    charmSlots: 2,
    glimmer: 0,
    attributes: ATTRIBUTES.map(() => 0),
    level: 0,
    levelCost: 0,
    attributeMax: 60,
    flasks: { maxCharges: 0, vigilMax: 0, vigil: 0, lucid: 0, level: 0, charLimit: 0 },
    resting: false,
    bossFight: false,
    dead: false,
    flaskItems: [0, 0],
    lantern: -1,
    lanterns: [],
    gear: [],
    stacks: []
  };
}

export function parseMenuState(w: ArrayLike<number>): MenuState {
  if (w.length < MENU_WORDS) return emptyMenuState();
  const nLanterns = w[41];
  const nGear = w[42];
  let at = MENU_WORDS;
  const lanterns: LanternEntry[] = [];
  for (let i = 0; i < nLanterns; i++) lanterns.push({ index: i, lit: w[at++] > 0 });
  const gear: number[] = [];
  for (let i = 0; i < nGear; i++) gear.push(w[at++]);
  const stacks: { item: number; count: number }[] = [];
  while (at + 1 < w.length) {
    stacks.push({ item: w[at], count: w[at + 1] });
    at += 2;
  }
  return {
    slots: Array.from({ length: 16 }, (_, i) => w[i]),
    charmSlots: w[19],
    glimmer: w[20],
    attributes: Array.from({ length: ATTRIBUTES.length }, (_, i) => w[21 + i]),
    level: w[28],
    levelCost: w[29],
    attributeMax: w[30],
    flasks: {
      maxCharges: w[31],
      vigilMax: w[32],
      vigil: w[33],
      lucid: w[34],
      level: w[35],
      charLimit: w[36]
    },
    resting: w[37] > 0,
    bossFight: w[38] > 0,
    dead: w[39] > 0,
    flaskItems: [w[43], w[44]],
    lantern: w[40] - 1,
    lanterns,
    gear,
    stacks
  };
}

/** What a loadout comes to: the core's `LoadoutStats`. */
export interface Loadout {
  /** For a preview: the core would accept the change. */
  allowed: boolean;
  /** Physical attack rating of the selected right-hand weapon. */
  attackRating: number;
  unwieldy: boolean;
  defense: number;
  /** Equip load and its maximum, in units (the core keeps tenths). */
  load: number;
  maxLoad: number;
  tier: (typeof TIER_NAMES)[number];
}

export function parseLoadout(w: ArrayLike<number>): Loadout | null {
  if (w.length < LOADOUT_WORDS) return null;
  return {
    allowed: w[0] > 0,
    attackRating: w[1],
    unwieldy: w[2] > 0,
    defense: w[3],
    load: w[4] / 10,
    maxLoad: w[5] / 10,
    tier: TIER_NAMES[Math.min(3, Math.max(0, w[6]))]
  };
}

/** Maximum HP, FP, stamina, and equip load now and after one more point of an attribute. */
export interface LevelPreview {
  hp: [number, number];
  fp: [number, number];
  stamina: [number, number];
  load: [number, number];
}

export function parseLevelPreview(w: ArrayLike<number>): LevelPreview | null {
  if (w.length < 8) return null;
  return {
    hp: [w[0], w[1]],
    fp: [w[2], w[3]],
    stamina: [w[4], w[5]],
    load: [w[6] / 10, w[7] / 10]
  };
}

/** "lantern_field" becomes "Field"; "lantern_dev_1" becomes "Dev 1". */
export function lanternLabel(id: string): string {
  const words = id.replace(/^lantern_/, '').split('_').filter(Boolean);
  if (words.length === 0) return 'Lantern';
  return words.map((w) => w[0].toUpperCase() + w.slice(1)).join(' ');
}

/** Signed difference for a stat preview: "+3", "-2", or "" when equal. */
export function delta(from: number, to: number, digits = 0): string {
  const d = Number((to - from).toFixed(digits));
  if (d === 0) return '';
  return d > 0 ? `+${d}` : `${d}`;
}
