// The HUD's view of the simulation: a plain object parsed from the floats `LfWorld.hud_state`
// returns (game/crates/wasm/src/lib.rs). Pure and read-only, with no Three.js or WASM
// imports, so it can be drawn without the engine. The HUD never writes simulation state.

/** Floats in a `hud_state` record. Must equal `hud_words()` from the module. */
export const HUD_WORDS = 30;

/** The statuses, in the order of the core's `StatusProc` codes. */
export const STATUS_NAMES = ['Poison', 'Blight', 'Drowse', 'Delirium', 'Rupture', 'Rime'] as const;

export interface QuickSlot {
  /** Registry number; 0 for an empty slot. */
  num: number;
  name: string;
  /** Flask charges or stack size; null where a count makes no sense (weapons). */
  count: number | null;
  /** True for the Vigil and Lucid flasks, whose count refills at a Lantern. */
  flask: boolean;
}

export interface Bar {
  value: number;
  max: number;
}

export interface BossBar {
  name: string;
  /** 0 to 1. */
  frac: number;
  phase: number;
}

export interface HudState {
  hp: Bar;
  fp: Bar;
  stamina: Bar;
  glimmer: number;
  vigil: Bar;
  lucid: Bar;
  right: QuickSlot;
  left: QuickSlot;
  spell: QuickSlot;
  item: QuickSlot;
  inCombat: boolean;
  /** 0 none, 1 a Lantern is in reach. */
  prompt: number;
  boss: BossBar | null;
  /** Buildup 0 to 1 per status, in `STATUS_NAMES` order. */
  status: number[];
  dead: boolean;
  /** Two-handing the right weapon. */
  twoHanding: boolean;
}

/** Resolves registry numbers and boss indexes to display names. */
export interface NameLookup {
  item(num: number): string;
  boss(index: number): string;
}

const EMPTY: QuickSlot = { num: 0, name: '', count: null, flask: false };

export function emptyHud(): HudState {
  return {
    hp: { value: 0, max: 0 },
    fp: { value: 0, max: 0 },
    stamina: { value: 0, max: 0 },
    glimmer: 0,
    vigil: { value: 0, max: 0 },
    lucid: { value: 0, max: 0 },
    right: EMPTY,
    left: EMPTY,
    spell: EMPTY,
    item: EMPTY,
    inCombat: false,
    prompt: 0,
    boss: null,
    status: STATUS_NAMES.map(() => 0),
    dead: false,
    twoHanding: false
  };
}

function clamp01(x: number): number {
  return x < 0 ? 0 : x > 1 ? 1 : x;
}

export function parseHud(w: ArrayLike<number>, names: NameLookup): HudState {
  if (w.length < HUD_WORDS) return emptyHud();
  const slot = (num: number, count: number | null, flask = false): QuickSlot =>
    num > 0 ? { num, name: names.item(num), count, flask } : EMPTY;
  const boss = w[18] > 0 ? { name: names.boss(w[18] - 1), frac: clamp01(w[19]), phase: w[20] } : null;
  return {
    hp: { value: w[0], max: w[1] },
    fp: { value: w[2], max: w[3] },
    stamina: { value: w[4], max: w[5] },
    glimmer: w[6],
    vigil: { value: w[7], max: w[8] },
    lucid: { value: w[9], max: w[10] },
    right: slot(w[11], null),
    left: slot(w[12], null),
    spell: slot(w[13], null),
    item: slot(w[14], w[15], w[28] > 0),
    inCombat: w[16] > 0,
    prompt: w[17],
    boss,
    status: Array.from({ length: STATUS_NAMES.length }, (_, i) => clamp01(w[21 + i])),
    dead: w[27] > 0,
    twoHanding: w[29] > 0
  };
}

/** Below this maximum a bar keeps its base width; above it, it grows. */
const BAR_BASE_POINTS = 100;
const BAR_BASE_PX = 160;
const BAR_PX_PER_POINT = 0.9;
const BAR_MAX_PX = 440;

/** Bar length in CSS pixels: grows with the stat's maximum (HUD spec), within limits. */
export function barLength(max: number): number {
  const px = BAR_BASE_PX + Math.max(0, max - BAR_BASE_POINTS) * BAR_PX_PER_POINT;
  return Math.round(Math.min(BAR_MAX_PX, px));
}

/** True when the HUD has nothing to say: out of combat, full bars, no status, boss, or prompt. */
export function isQuiet(s: HudState): boolean {
  const full = (b: Bar) => b.value >= b.max;
  return (
    !s.inCombat &&
    !s.dead &&
    s.boss === null &&
    s.prompt === 0 &&
    full(s.hp) &&
    full(s.fp) &&
    full(s.stamina) &&
    s.status.every((v) => v <= 0)
  );
}
