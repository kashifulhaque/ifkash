// The binding table: which inputs trigger which actions, plus the input options.
// Defaults follow docs/game/07-presentation.md, "Controls". Saved to localStorage.
//
// An input token is one of:
//   - a `KeyboardEvent.code`, such as `KeyW` or `Space`
//   - `Mouse0` (left), `Mouse1` (middle), `Mouse2` (right), `WheelUp`, `WheelDown`
//   - `Pad<N>`, button N of a standard-layout gamepad
//   - `Pad<M>+Pad<N>`, a pad chord: button N pressed while button M is held (Y + RB)
// A token can carry a `Shift+` prefix, which requires Shift to be held when it is pressed.

/** Button bits of the ActionFrame. Mirrors `lf_core::input::buttons`. */
export const BTN = {
  LIGHT: 1 << 0,
  HEAVY: 1 << 1,
  GUARD: 1 << 2,
  DODGE: 1 << 3,
  ART: 1 << 4,
  JUMP: 1 << 5,
  USE_ITEM: 1 << 6,
  INTERACT: 1 << 7,
  LOCK: 1 << 8,
  SWITCH_L: 1 << 9,
  SWITCH_R: 1 << 10,
  CYCLE_ITEM: 1 << 11,
  CYCLE_SPELL: 1 << 12,
  CYCLE_RIGHT: 1 << 13,
  CYCLE_LEFT: 1 << 14,
  TWO_HAND: 1 << 15
} as const;

/** Actions that become a bit of the ActionFrame. */
export const BUTTON_ACTIONS = [
  'light',
  'heavy',
  'guard',
  'dodge',
  'art',
  'jump',
  'useItem',
  'interact',
  'lock',
  'switchL',
  'switchR',
  'cycleItem',
  'cycleSpell',
  'cycleRight',
  'cycleLeft',
  'twoHand'
] as const;
export type ButtonAction = (typeof BUTTON_ACTIONS)[number];

/** The bit of each button action. */
export const ACTION_BIT: Record<ButtonAction, number> = {
  light: BTN.LIGHT,
  heavy: BTN.HEAVY,
  guard: BTN.GUARD,
  dodge: BTN.DODGE,
  art: BTN.ART,
  jump: BTN.JUMP,
  useItem: BTN.USE_ITEM,
  interact: BTN.INTERACT,
  lock: BTN.LOCK,
  switchL: BTN.SWITCH_L,
  switchR: BTN.SWITCH_R,
  cycleItem: BTN.CYCLE_ITEM,
  cycleSpell: BTN.CYCLE_SPELL,
  cycleRight: BTN.CYCLE_RIGHT,
  cycleLeft: BTN.CYCLE_LEFT,
  twoHand: BTN.TWO_HAND
};

export const MOVE_ACTIONS = ['moveForward', 'moveBack', 'moveLeft', 'moveRight'] as const;
/** Actions handled by the UI, not the simulation. They never reach the ActionFrame. */
export const UI_ACTIONS = ['map', 'menu'] as const;
export type UiAction = (typeof UI_ACTIONS)[number];

export type Action = ButtonAction | (typeof MOVE_ACTIONS)[number] | UiAction;
export const ALL_ACTIONS: readonly Action[] = [...MOVE_ACTIONS, ...BUTTON_ACTIONS, ...UI_ACTIONS];

export type Bindings = Record<Action, string[]>;

export const DEFAULT_BINDINGS: Bindings = {
  moveForward: ['KeyW'],
  moveBack: ['KeyS'],
  moveLeft: ['KeyA'],
  moveRight: ['KeyD'],
  light: ['Mouse0', 'Pad5'],
  heavy: ['Shift+Mouse0', 'Pad7'],
  guard: ['Mouse2', 'Pad4'],
  art: ['Shift+Mouse2', 'Pad6'],
  dodge: ['Space', 'Pad1'],
  jump: ['KeyF', 'Pad0'],
  // Elden Ring's layout (issue #23): Y interacts, X uses the quick item, and Y + RB
  // two-hands the right weapon.
  interact: ['KeyE', 'Pad3'],
  useItem: ['KeyR', 'Pad2'],
  lock: ['KeyQ', 'Mouse1', 'Pad11'],
  switchL: ['WheelUp'],
  switchR: ['WheelDown'],
  cycleItem: ['ArrowDown', 'Pad13'],
  cycleSpell: ['ArrowUp', 'Pad12'],
  cycleRight: ['ArrowRight', 'Pad15'],
  cycleLeft: ['ArrowLeft', 'Pad14'],
  twoHand: ['KeyV', 'Pad3+Pad5'],
  map: ['KeyM', 'Pad8', 'Pad17'],
  menu: ['Escape', 'Pad9']
};

/** Hold actions that can be switched to a toggle. */
export const TOGGLEABLE: readonly ButtonAction[] = ['guard', 'dodge', 'lock', 'twoHand'];

export interface InputOptions {
  /** A press flips the action on or off instead of holding it. */
  toggle: Partial<Record<ButtonAction, boolean>>;
  /** Merges light and heavy: a tap is light, a hold is heavy. */
  oneButtonAttack: boolean;
  /** Radians of camera yaw per pixel of mouse movement. */
  mouseSensitivity: number;
  invertY: boolean;
}

export const DEFAULT_OPTIONS: InputOptions = {
  toggle: {},
  oneButtonAttack: false,
  mouseSensitivity: 0.0022,
  invertY: false
};

export interface InputSettings {
  bindings: Bindings;
  options: InputOptions;
}

const STORAGE_KEY = 'ifkash-latticefall-input';
/** Version 2 moved Interact to Y and Two-hand to Y + RB (issue #23). */
const STORAGE_VERSION = 2;

export function defaultSettings(): InputSettings {
  return {
    bindings: Object.fromEntries(ALL_ACTIONS.map((a) => [a, [...DEFAULT_BINDINGS[a]]])) as Bindings,
    options: { ...DEFAULT_OPTIONS, toggle: {} }
  };
}

/** Loads saved settings over the defaults. Unknown or malformed fields are ignored. */
export function loadSettings(): InputSettings {
  const out = defaultSettings();
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return out;
    const saved = JSON.parse(raw) as {
      version?: number;
      bindings?: Record<string, unknown>;
      options?: Record<string, unknown>;
    };
    if (saved.version !== STORAGE_VERSION && saved.version !== 1) return out;
    for (const a of ALL_ACTIONS) {
      const b = saved.bindings?.[a];
      if (!Array.isArray(b) || !b.every((t) => typeof t === 'string')) continue;
      // Version 1 had the old pad layout: keep its keys and mouse, take the new pad defaults.
      out.bindings[a] =
        saved.version === 1 ? [...b.filter((t) => !isPadToken(t)), ...DEFAULT_BINDINGS[a].filter(isPadToken)] : b;
    }
    const o = saved.options;
    if (o) {
      if (typeof o.oneButtonAttack === 'boolean') out.options.oneButtonAttack = o.oneButtonAttack;
      if (typeof o.invertY === 'boolean') out.options.invertY = o.invertY;
      if (typeof o.mouseSensitivity === 'number' && o.mouseSensitivity > 0 && o.mouseSensitivity < 0.1) {
        out.options.mouseSensitivity = o.mouseSensitivity;
      }
      const t = o.toggle as Record<string, unknown> | undefined;
      if (t) for (const a of TOGGLEABLE) if (t[a] === true) out.options.toggle[a] = true;
    }
  } catch {
    // Missing, blocked, or corrupt storage: use the defaults.
  }
  return out;
}

export function saveSettings(settings: InputSettings): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ version: STORAGE_VERSION, ...settings }));
  } catch {
    // Storage is unavailable; the settings last for this session only.
  }
}

/**
 * Splits a token into its Shift requirement, the pad button it needs held (`Pad3` in the
 * chord `Pad3+Pad5`), and the base token that triggers it.
 */
export function parseToken(token: string): { shift: boolean; hold: string | null; base: string } {
  if (token.startsWith('Shift+')) return { shift: true, hold: null, base: token.slice(6) };
  const chord = /^(Pad\d+)\+(Pad\d+)$/.exec(token);
  if (chord) return { shift: false, hold: chord[1], base: chord[2] };
  return { shift: false, hold: null, base: token };
}

/** True for a gamepad token, chord or not. */
export function isPadToken(token: string): boolean {
  return parseToken(token).base.startsWith('Pad');
}

/**
 * The actions a pressed input triggers. `Shift+X` bindings match only while Shift is held.
 * A plain `X` binding is suppressed while Shift is held if some `Shift+X` binding exists,
 * so Shift and left mouse is Heavy and not Light and Heavy. Pad chords work the same way:
 * with Y in `held`, RB triggers `Pad3+Pad5` (Two-hand) instead of the plain RB binding.
 */
export function resolveActions(bindings: Bindings, base: string, shift: boolean, held: ReadonlySet<string> = new Set()): Action[] {
  const plain: Action[] = [];
  const shifted: Action[] = [];
  const chorded: Action[] = [];
  for (const a of ALL_ACTIONS) {
    for (const t of bindings[a]) {
      const p = parseToken(t);
      if (p.base !== base) continue;
      if (p.hold !== null) {
        if (held.has(p.hold)) chorded.push(a);
      } else (p.shift ? shifted : plain).push(a);
    }
  }
  if (chorded.length > 0) return chorded;
  if (shift && shifted.length > 0) return shifted;
  return plain;
}

/** Replaces the tokens of one action. */
export function setBinding(settings: InputSettings, action: Action, tokens: string[]): void {
  settings.bindings[action] = [...tokens];
}
