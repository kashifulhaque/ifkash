// Gamepad normalization: any connected pad becomes one standard-layout snapshot, so the
// game, the menus, and the remap screen all read button N as the same physical button.
// Pure maths except `pickPad` and `readPad`, which read `navigator.getGamepads()`.
//
// Most browsers already report an Xbox pad with `mapping === 'standard'`: Chrome and Edge
// over USB and Bluetooth, and Safari through the GameController framework. Some don't: a
// Bluetooth Xbox pad in Firefox, or a firmware revision a browser has no mapping for,
// reports `mapping === ''` with raw HID indexes and the triggers as axes. Those pads are
// matched by vendor and product in `Gamepad.id` and remapped with a table below; any other
// non-standard pad is read as if it were standard, which is right for most generic pads.

/** Standard-layout button indexes (https://w3c.github.io/gamepad/#remapping), Xbox names. */
export const STD = {
  A: 0,
  B: 1,
  X: 2,
  Y: 3,
  LB: 4,
  RB: 5,
  LT: 6,
  RT: 7,
  VIEW: 8,
  MENU: 9,
  LS: 10,
  RS: 11,
  UP: 12,
  DOWN: 13,
  LEFT: 14,
  RIGHT: 15,
  HOME: 16
} as const;

/** Buttons in a normalized snapshot: the 17 standard ones plus 17, a touchpad or share. */
export const STD_BUTTONS = 18;

/** Xbox glyph of each standard button, for prompts, the HUD, and the remap screen. */
export const PAD_GLYPHS = [
  'A',
  'B',
  'X',
  'Y',
  'LB',
  'RB',
  'LT',
  'RT',
  'View',
  'Menu',
  'LS',
  'RS',
  'D-pad up',
  'D-pad down',
  'D-pad left',
  'D-pad right',
  'Xbox',
  'Share'
] as const;

/** The glyph of a `Pad<N>` binding token, or null if the token isn't a pad button. */
export function padGlyph(token: string): string | null {
  const chord = /^(Pad\d+)\+(Pad\d+)$/.exec(token);
  if (chord) return `${padGlyph(chord[1])} + ${padGlyph(chord[2])}`;
  const m = /^Pad(\d+)$/.exec(token);
  if (!m) return null;
  return PAD_GLYPHS[Number(m[1])] ?? `Button ${m[1]}`;
}

/** Radial stick deadzone, 0 to 1. Worn Xbox sticks drift up to about 0.12 at rest. */
export const STICK_DEADZONE = 0.15;
/** A trigger counts as pressed above this pull (0 to 1) and released below `TRIGGER_OFF`. */
export const TRIGGER_ON = 0.35;
export const TRIGGER_OFF = 0.2;

/** The fields of a browser `Gamepad` this module reads, so tests can pass plain snapshots. */
export interface PadSnapshot {
  id: string;
  mapping: string;
  connected?: boolean;
  timestamp?: number;
  axes: readonly number[];
  buttons: readonly { pressed: boolean; value: number }[];
  /** Read only for its `effects` list, which older TypeScript DOM types don't declare. */
  vibrationActuator?: unknown;
}

/** One pad in standard layout. Sticks are raw (no deadzone); y is positive downward. */
export interface StdPad {
  /** Pressed state of each standard button, `STD_BUTTONS` long. */
  pressed: boolean[];
  /** Analog value of each standard button, 0 to 1. */
  values: number[];
  /** Left x, left y, right x, right y, each -1 to 1. */
  axes: [number, number, number, number];
  /** The layout table used, such as `standard` or `xbox-ble-chrome`. */
  layout: string;
  /** True for a Microsoft (vendor 045e) or Xbox-named pad. */
  xbox: boolean;
  /**
   * Whether each axis trigger (LT, RT) has reported a value other than exactly 0. Browsers
   * report 0 for an axis until it first moves, which on a signed trigger is a half pull.
   */
  triggerSeen: [boolean, boolean];
}

/** Where a trigger is read from on a pad that reports it as an axis. */
interface AxisTrigger {
  axis: number;
  /** `signed` axes rest at -1 and reach 1; `unsigned` ones rest at 0. */
  range: 'signed' | 'unsigned';
}

/** A raw layout: for each standard button, the raw button index that feeds it. */
export interface Layout {
  name: string;
  /** Indexed by standard button; null if this pad has no such button. */
  buttons: (number | null)[];
  /** Raw axis indexes of left x, left y, right x, right y. */
  sticks: [number, number, number, number];
  /** Triggers reported as axes, if they aren't buttons 6 and 7. */
  triggers?: { lt: AxisTrigger; rt: AxisTrigger };
  /** Raw axis index of a hat switch that carries the D-pad, if the D-pad isn't buttons. */
  hat?: number;
}

const IDENTITY: (number | null)[] = Array.from({ length: STD_BUTTONS }, (_, i) => i);

/** The standard layout, and the generic fallback for unknown pads. */
export const STANDARD_LAYOUT: Layout = { name: 'standard', buttons: IDENTITY, sticks: [0, 1, 2, 3] };
const GENERIC_LAYOUT: Layout = { ...STANDARD_LAYOUT, name: 'generic' };

// Xbox Wireless (Series X|S, and One S on firmware 5 and later) over Bluetooth LE sends HID
// buttons 1 to 15 with gaps at 3, 6, 9, and 10: A, B, -, X, Y, -, LB, RB, -, -, View, Menu,
// Xbox, LS, RS. Its axes are X, Y, Z, Rz (the sticks), Brake and Accelerator (the
// triggers, rest at -1), and a hat switch for the D-pad.
const XBOX_BLE_BUTTONS: (number | null)[] = [0, 1, 3, 4, 6, 7, null, null, 10, 11, 13, 14, null, null, null, null, 12, 15];

/**
 * Chrome and Edge without a built-in mapping place axes by HID usage (X 0, Y 1, Z 2, Rz 5,
 * hat 9) and fill the free slots with the rest in report order (Brake 3, Accelerator 4).
 */
export const XBOX_BLE_CHROME: Layout = {
  name: 'xbox-ble-chrome',
  buttons: XBOX_BLE_BUTTONS,
  sticks: [0, 1, 2, 5],
  triggers: { lt: { axis: 3, range: 'signed' }, rt: { axis: 4, range: 'signed' } },
  hat: 9
};

/** Firefox lists axes in report order: X 0, Y 1, Z 2, Rz 3, Brake 4, Accelerator 5, hat 6. */
export const XBOX_BLE_FIREFOX: Layout = {
  name: 'xbox-ble-firefox',
  buttons: XBOX_BLE_BUTTONS,
  sticks: [0, 1, 2, 3],
  triggers: { lt: { axis: 4, range: 'signed' }, rt: { axis: 5, range: 'signed' } },
  hat: 6
};

// The Xbox One S (045e:02e0) on its 2016 Bluetooth firmware is a DirectInput-style pad:
// A, B, X, Y, LB, RB, View, Menu, LS, RS as buttons 0 to 9; the left trigger on Z and the
// right on Rz, each 0 to 1; the right stick on Rx and Ry; a hat for the D-pad.
const XBOX_2016_BUTTONS: (number | null)[] = [0, 1, 2, 3, 4, 5, null, null, 6, 7, 8, 9, null, null, null, null, null, null];

export const XBOX_2016_CHROME: Layout = {
  name: 'xbox-2016-chrome',
  buttons: XBOX_2016_BUTTONS,
  sticks: [0, 1, 3, 4],
  triggers: { lt: { axis: 2, range: 'unsigned' }, rt: { axis: 5, range: 'unsigned' } },
  hat: 9
};

export const XBOX_2016_FIREFOX: Layout = {
  name: 'xbox-2016-firefox',
  buttons: XBOX_2016_BUTTONS,
  sticks: [0, 1, 3, 4],
  triggers: { lt: { axis: 2, range: 'unsigned' }, rt: { axis: 5, range: 'unsigned' } },
  hat: 6
};

const MICROSOFT = 0x045e;
/** Xbox One S on the 2016 Bluetooth firmware. */
const XBOX_2016_PRODUCTS = new Set([0x02e0]);

/**
 * Vendor and product from a `Gamepad.id`. Chrome and Edge write
 * `Name (... Vendor: 045e Product: 0b13)`; Firefox writes `45e-b13-Name`. Safari writes only
 * the name. Returns `format` so the caller can tell how the browser numbers raw axes.
 */
export function parsePadId(id: string): { vendor: number; product: number; format: 'chrome' | 'firefox' } | null {
  const c = /Vendor:\s*([0-9a-f]{1,4})\s+Product:\s*([0-9a-f]{1,4})/i.exec(id);
  if (c) return { vendor: parseInt(c[1], 16), product: parseInt(c[2], 16), format: 'chrome' };
  const f = /^([0-9a-f]{1,4})-([0-9a-f]{1,4})-/i.exec(id);
  if (f) return { vendor: parseInt(f[1], 16), product: parseInt(f[2], 16), format: 'firefox' };
  return null;
}

/**
 * True for an Xbox pad: vendor 045e, a name that says Xbox, or impulse-trigger rumble.
 * Chrome on macOS reads pads through the GameController framework and names a wired Xbox
 * pad just `Controller (STANDARD GAMEPAD)`, with no vendor; only Xbox pads offer the
 * `trigger-rumble` effect.
 */
export function isXboxPad(pad: Pick<PadSnapshot, 'id' | 'vibrationActuator'>): boolean {
  if (parsePadId(pad.id)?.vendor === MICROSOFT || /xbox|x-box/i.test(pad.id)) return true;
  const effects = (pad.vibrationActuator as { effects?: unknown } | null | undefined)?.effects;
  return Array.isArray(effects) && effects.includes('trigger-rumble');
}

/** The layout table for a pad. Standard pads need none; unknown raw pads get the generic one. */
export function layoutFor(pad: Pick<PadSnapshot, 'id' | 'mapping'>): Layout {
  if (pad.mapping === 'standard') return STANDARD_LAYOUT;
  const ids = parsePadId(pad.id);
  if (ids?.vendor === MICROSOFT) {
    const old = XBOX_2016_PRODUCTS.has(ids.product);
    if (ids.format === 'firefox') return old ? XBOX_2016_FIREFOX : XBOX_BLE_FIREFOX;
    return old ? XBOX_2016_CHROME : XBOX_BLE_CHROME;
  }
  return GENERIC_LAYOUT;
}

/**
 * Decodes a hat-switch axis into D-pad up, down, left, right. Browsers report the eight
 * directions as -1 + k * 2/7, k = 0 for up and counting clockwise, and neutral as a value
 * outside [-1, 1] (Chrome uses 3.29, Firefox about 1.29).
 */
export function hatToDpad(v: number | undefined): [boolean, boolean, boolean, boolean] {
  if (v === undefined || !Number.isFinite(v) || v < -1.05 || v > 1.05) return [false, false, false, false];
  const k = Math.round(((v + 1) * 7) / 2);
  return [k === 7 || k <= 1, k >= 3 && k <= 5, k >= 5 && k <= 7, k >= 1 && k <= 3];
}

/** A trigger axis as a 0-to-1 pull. Signed axes run -1 (released) to 1 (full). */
export function triggerValue(raw: number | undefined, range: 'signed' | 'unsigned'): number {
  const v = raw ?? (range === 'signed' ? -1 : 0);
  const pull = range === 'signed' ? (v + 1) / 2 : v;
  return Math.max(0, Math.min(1, pull));
}

/** Applies a trigger's hysteresis: on above `TRIGGER_ON`, off below `TRIGGER_OFF`. */
export function triggerPressed(value: number, wasPressed: boolean): boolean {
  return value >= (wasPressed ? TRIGGER_OFF : TRIGGER_ON);
}

/**
 * Converts a pad to standard layout. `prev` is the last result for the same pad; it gives
 * the triggers their hysteresis.
 */
export function normalizePad(pad: PadSnapshot, prev?: StdPad | null): StdPad {
  const layout = layoutFor(pad);
  const pressed: boolean[] = new Array(STD_BUTTONS).fill(false);
  const values: number[] = new Array(STD_BUTTONS).fill(0);
  for (let i = 0; i < STD_BUTTONS; i++) {
    const raw = layout.buttons[i];
    const b = raw === null || raw === undefined ? undefined : pad.buttons[raw];
    if (!b) continue;
    pressed[i] = !!b.pressed;
    values[i] = b.value ?? (b.pressed ? 1 : 0);
  }
  const triggerSeen: [boolean, boolean] = [prev?.triggerSeen[0] ?? false, prev?.triggerSeen[1] ?? false];
  if (layout.triggers) {
    const { lt, rt } = layout.triggers;
    [lt, rt].forEach((t, k) => {
      const raw = pad.axes[t.axis];
      if (raw !== undefined && raw !== 0) triggerSeen[k] = true;
      // Until a signed trigger has moved, its 0 means "no data", not a half pull.
      values[k === 0 ? STD.LT : STD.RT] = t.range === 'signed' && !triggerSeen[k] ? 0 : triggerValue(raw, t.range);
    });
  }
  // Triggers use our own threshold everywhere: Chrome's `pressed` fires at a light touch.
  for (const t of [STD.LT, STD.RT]) {
    if (layout.triggers || values[t] > 0 || pressed[t]) {
      const v = values[t] > 0 ? values[t] : pressed[t] ? 1 : 0;
      pressed[t] = triggerPressed(v, !!prev?.pressed[t]);
    }
  }
  if (layout.hat !== undefined) {
    const [u, d, l, r] = hatToDpad(pad.axes[layout.hat]);
    pressed[STD.UP] = u;
    pressed[STD.DOWN] = d;
    pressed[STD.LEFT] = l;
    pressed[STD.RIGHT] = r;
    for (const i of [STD.UP, STD.DOWN, STD.LEFT, STD.RIGHT]) values[i] = pressed[i] ? 1 : 0;
  }
  const [a, b, c, d] = layout.sticks;
  const axes: [number, number, number, number] = [pad.axes[a] ?? 0, pad.axes[b] ?? 0, pad.axes[c] ?? 0, pad.axes[d] ?? 0];
  return { pressed, values, axes, layout: layout.name, xbox: isXboxPad(pad), triggerSeen };
}

/**
 * The pad to read: the connected one used most recently, by `Gamepad.timestamp`. A pad that
 * was plugged in before the page loaded shows up only after its first button press, which
 * this picks up on the next poll. Null if there is none or the API is missing.
 */
export function pickPad(pads?: readonly (PadSnapshot | null)[]): PadSnapshot | null {
  let list = pads;
  if (!list) {
    if (typeof navigator === 'undefined' || !navigator.getGamepads) return null;
    try {
      list = navigator.getGamepads();
    } catch {
      // Blocked by a permissions policy.
      return null;
    }
  }
  let best: PadSnapshot | null = null;
  for (const p of list) {
    if (!p || p.connected === false) continue;
    if (!best || (p.timestamp ?? 0) > (best.timestamp ?? 0)) best = p;
  }
  return best;
}

/** True if a normalized pad is in use: any button down or a stick past the deadzone. */
export function padActive(p: StdPad): boolean {
  if (p.pressed.some(Boolean)) return true;
  return Math.hypot(p.axes[0], p.axes[1]) > STICK_DEADZONE || Math.hypot(p.axes[2], p.axes[3]) > STICK_DEADZONE;
}

/** Reads the current pad in standard layout, or null. Holds trigger state between calls. */
export function createPadReader(): () => StdPad | null {
  let prev: StdPad | null = null;
  let prevId = '';
  return () => {
    const raw = pickPad();
    if (!raw) {
      prev = null;
      return null;
    }
    if (raw.id !== prevId) prev = null;
    prevId = raw.id;
    prev = normalizePad(raw, prev);
    return prev;
  };
}
