// Tests for the gamepad remap tables and stick and trigger maths. Run with `bun test`.
//
// The `wired` and `bluetooth` snapshots were recorded from the owner's Xbox Wireless pad in
// Chromium 156 on macOS (2026-11-13), over USB and over Bluetooth. Chromium reports both with
// `mapping: 'standard'`. The raw (`mapping: ''`) Bluetooth snapshots are built from the HID
// layouts documented in `gamepad.ts`; no browser here produced one.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  STD,
  STD_BUTTONS,
  hatToDpad,
  isXboxPad,
  layoutFor,
  normalizePad,
  padActive,
  padGlyph,
  parsePadId,
  pickPad,
  triggerValue,
  type PadSnapshot
} from './gamepad';
import { radialDeadzone } from './frame';
import { actionGlyph, keyGlyph } from './device';
import { defaultSettings, isPadToken, parseToken, resolveActions } from './bindings';

const NEUTRAL_HAT_CHROME = 3.2857141494750977;

function buttons(n: number, pressed: Record<number, number> = {}): PadSnapshot['buttons'] {
  return Array.from({ length: n }, (_, i) => ({ pressed: (pressed[i] ?? 0) > 0, value: pressed[i] ?? 0 }));
}

/** The owner's wired pad, as Chromium on macOS reports it: no vendor in the id. */
function wired(pressed: Record<number, number> = {}, axes = [0, 0, 0, 0]): PadSnapshot {
  return {
    id: 'Controller (STANDARD GAMEPAD)',
    mapping: 'standard',
    connected: true,
    timestamp: 88074.1,
    axes,
    buttons: buttons(18, pressed),
    vibrationActuator: { effects: ['trigger-rumble', 'dual-rumble'] }
  };
}

/** The owner's pad over Bluetooth in Chromium on macOS: standard, and Share is button 17. */
function bluetooth(pressed: Record<number, number> = {}): PadSnapshot {
  return { ...wired(pressed), id: 'Xbox Wireless Controller (STANDARD GAMEPAD)' };
}

/** An Xbox Series pad over Bluetooth in Chrome with no built-in mapping. */
function bleChrome(pressed: Record<number, number> = {}, axes: Record<number, number> = {}): PadSnapshot {
  const a = Array.from({ length: 10 }, (_, i) => axes[i] ?? 0);
  if (axes[9] === undefined) a[9] = NEUTRAL_HAT_CHROME;
  return {
    id: 'Xbox Wireless Controller (Vendor: 045e Product: 0b13)',
    mapping: '',
    connected: true,
    timestamp: 1,
    axes: a,
    buttons: buttons(16, pressed)
  };
}

/** The same pad in Firefox, which numbers ids and axes differently. */
function bleFirefox(pressed: Record<number, number> = {}, axes: Record<number, number> = {}): PadSnapshot {
  const a = Array.from({ length: 7 }, (_, i) => axes[i] ?? 0);
  if (axes[6] === undefined) a[6] = 1.2857;
  return { id: '45e-b13-Xbox Wireless Controller', mapping: '', connected: true, timestamp: 1, axes: a, buttons: buttons(16, pressed) };
}

test('parsePadId reads Chrome and Firefox ids and rejects names', () => {
  assert.deepEqual(parsePadId('Xbox Wireless Controller (STANDARD GAMEPAD Vendor: 045e Product: 0b13)'), {
    vendor: 0x045e,
    product: 0x0b13,
    format: 'chrome'
  });
  assert.deepEqual(parsePadId('45e-2e0-Xbox Wireless Controller'), { vendor: 0x045e, product: 0x02e0, format: 'firefox' });
  assert.equal(parsePadId('Controller (STANDARD GAMEPAD)'), null);
  assert.equal(parsePadId('Xbox Wireless Controller'), null);
});

test('isXboxPad finds the wired pad by its rumble effects, and others by id', () => {
  assert.equal(isXboxPad(wired()), true);
  assert.equal(isXboxPad({ id: 'Controller (STANDARD GAMEPAD)', vibrationActuator: { effects: ['dual-rumble'] } }), false);
  assert.equal(isXboxPad({ id: 'Xbox Wireless Controller' }), true);
  assert.equal(isXboxPad(bleFirefox()), true);
  assert.equal(isXboxPad({ id: '54c-ce6-DualSense Wireless Controller' }), false);
});

test('layoutFor picks a table by mapping, vendor, product, and id format', () => {
  assert.equal(layoutFor(wired()).name, 'standard');
  assert.equal(layoutFor(bleChrome()).name, 'xbox-ble-chrome');
  assert.equal(layoutFor(bleFirefox()).name, 'xbox-ble-firefox');
  assert.equal(layoutFor({ id: 'Xbox Wireless Controller (Vendor: 045e Product: 02e0)', mapping: '' }).name, 'xbox-2016-chrome');
  assert.equal(layoutFor({ id: '45e-2e0-Xbox Wireless Controller', mapping: '' }).name, 'xbox-2016-firefox');
  assert.equal(layoutFor({ id: '79-11-Generic USB Joystick', mapping: '' }).name, 'generic');
});

test('wired: every face, shoulder, menu, stick, and D-pad button keeps its index', () => {
  for (const i of [0, 1, 2, 3, 4, 5, 8, 9, 10, 11, 12, 13, 14, 15]) {
    const p = normalizePad(wired({ [i]: 1 }));
    assert.deepEqual(
      p.pressed.map((d, k) => (d ? k : -1)).filter((k) => k >= 0),
      [i],
      `button ${i}`
    );
  }
});

test('recorded Bluetooth: standard layout, Xbox by name, Share on 17', () => {
  const p = normalizePad(bluetooth({ 17: 1 }));
  assert.equal(p.layout, 'standard');
  assert.equal(p.xbox, true);
  assert.equal(p.pressed[17], true);
  assert.equal(padGlyph('Pad17'), 'Share');
  // Recorded: the first LT frame Chrome called pressed was 0.616, the first RT 0.142.
  assert.equal(normalizePad(bluetooth({ 6: 0.616 })).pressed[STD.LT], true);
  assert.equal(normalizePad(bluetooth({ 7: 0.142 })).pressed[STD.RT], false);
});

test('wired: a light trigger brush that Chrome calls pressed does not press', () => {
  // Recorded: Chrome set LT pressed at 0.054 and RT at 0.111.
  assert.equal(normalizePad(wired({ 6: 0.054 })).pressed[STD.LT], false);
  assert.equal(normalizePad(wired({ 7: 0.111 })).pressed[STD.RT], false);
  assert.equal(normalizePad(wired({ 7: 0.6 })).pressed[STD.RT], true);
});

test('triggers have hysteresis: on at 0.35, off below 0.2', () => {
  let p = normalizePad(wired({ 7: 0.3 }));
  assert.equal(p.pressed[STD.RT], false);
  p = normalizePad(wired({ 7: 0.4 }), p);
  assert.equal(p.pressed[STD.RT], true);
  p = normalizePad(wired({ 7: 0.25 }), p);
  assert.equal(p.pressed[STD.RT], true, 'held between the thresholds');
  p = normalizePad(wired({ 7: 0.1 }), p);
  assert.equal(p.pressed[STD.RT], false);
});

test('Bluetooth (Chrome): raw buttons remap to standard', () => {
  const cases: [number, number][] = [
    [0, STD.A],
    [1, STD.B],
    [3, STD.X],
    [4, STD.Y],
    [6, STD.LB],
    [7, STD.RB],
    [10, STD.VIEW],
    [11, STD.MENU],
    [12, STD.HOME],
    [13, STD.LS],
    [14, STD.RS]
  ];
  for (const [raw, std] of cases) {
    const p = normalizePad(bleChrome({ [raw]: 1 }));
    assert.equal(p.pressed[std], true, `raw ${raw} -> std ${std}`);
    assert.equal(p.pressed.filter(Boolean).length, 1, `raw ${raw} presses only one button`);
  }
  // Raw 2 and 5 are gaps in the HID report and press nothing.
  assert.equal(normalizePad(bleChrome({ 2: 1, 5: 1 })).pressed.some(Boolean), false);
});

test('Bluetooth (Chrome): sticks come from axes 0, 1, 2, 5 and triggers from 3, 4', () => {
  // The axes start at -1 for released triggers once they have moved.
  let p = normalizePad(bleChrome({}, { 0: 0.5, 1: -0.25, 2: 0.75, 5: -1, 3: -1, 4: -1 }));
  assert.deepEqual(p.axes, [0.5, -0.25, 0.75, -1]);
  assert.equal(p.pressed[STD.LT], false);
  assert.equal(p.pressed[STD.RT], false);
  p = normalizePad(bleChrome({}, { 3: 1, 4: -1 }), p);
  assert.equal(p.pressed[STD.LT], true);
  assert.equal(p.values[STD.LT], 1);
  assert.equal(p.pressed[STD.RT], false);
  p = normalizePad(bleChrome({}, { 3: -1, 4: 0.2 }), p);
  assert.equal(p.pressed[STD.RT], true, '0.2 raw is a 0.6 pull');
});

test('Bluetooth: a signed trigger that has never moved reads 0, not a half pull', () => {
  const p = normalizePad(bleChrome());
  assert.equal(p.values[STD.LT], 0);
  assert.equal(p.pressed[STD.LT], false);
  assert.equal(p.pressed[STD.RT], false);
});

test('Bluetooth (Firefox): sticks on 0-3, triggers on 4 and 5, hat on 6', () => {
  let p = normalizePad(bleFirefox({ 3: 1 }, { 2: 0.5, 3: -0.5, 4: -1, 5: -1 }));
  assert.equal(p.layout, 'xbox-ble-firefox');
  assert.equal(p.pressed[STD.X], true);
  assert.deepEqual(p.axes, [0, 0, 0.5, -0.5]);
  p = normalizePad(bleFirefox({}, { 4: -1, 5: 1, 6: -1 }), p);
  assert.equal(p.pressed[STD.RT], true);
  assert.equal(p.pressed[STD.UP], true);
});

test('hatToDpad decodes eight directions and neutral', () => {
  const step = 2 / 7;
  const dirs = [0, 1, 2, 3, 4, 5, 6, 7].map((k) => hatToDpad(-1 + k * step));
  // [up, down, left, right]
  assert.deepEqual(dirs, [
    [true, false, false, false],
    [true, false, false, true],
    [false, false, false, true],
    [false, true, false, true],
    [false, true, false, false],
    [false, true, true, false],
    [false, false, true, false],
    [true, false, true, false]
  ]);
  assert.deepEqual(hatToDpad(NEUTRAL_HAT_CHROME), [false, false, false, false]);
  assert.deepEqual(hatToDpad(1.2857), [false, false, false, false]);
  assert.deepEqual(hatToDpad(undefined), [false, false, false, false]);
});

test('Bluetooth (Chrome): the hat on axis 9 drives D-pad buttons 12 to 15', () => {
  const p = normalizePad(bleChrome({}, { 9: -1 + 4 * (2 / 7) }));
  assert.equal(p.pressed[STD.DOWN], true);
  assert.equal(p.pressed[STD.UP], false);
});

test('triggerValue maps signed and unsigned axes to a 0-to-1 pull', () => {
  assert.equal(triggerValue(-1, 'signed'), 0);
  assert.equal(triggerValue(0, 'signed'), 0.5);
  assert.equal(triggerValue(1, 'signed'), 1);
  assert.equal(triggerValue(0.4, 'unsigned'), 0.4);
  assert.equal(triggerValue(undefined, 'unsigned'), 0);
});

test('an unknown raw pad falls back to the standard indexes', () => {
  const p = normalizePad({ id: '79-11-Generic USB Joystick', mapping: '', axes: [0.3, 0, 0, 0], buttons: buttons(12, { 0: 1, 9: 1 }) });
  assert.equal(p.layout, 'generic');
  assert.equal(p.pressed[STD.A], true);
  assert.equal(p.pressed[STD.MENU], true);
  assert.equal(p.axes[0], 0.3);
  assert.equal(p.pressed.length, STD_BUTTONS);
});

test('radialDeadzone: drift at rest is zero, full tilt stays 1, direction is kept', () => {
  assert.deepEqual(radialDeadzone(0.1, -0.08), [0, 0]);
  assert.deepEqual(radialDeadzone(0.15, 0), [0, 0]);
  const [x, y] = radialDeadzone(1, 0);
  assert.equal(x, 1);
  assert.equal(y, 0);
  const [dx, dy] = radialDeadzone(0.7071, 0.7071);
  assert.ok(Math.abs(Math.hypot(dx, dy) - 1) < 1e-3);
  assert.ok(Math.abs(dx - dy) < 1e-9);
  const [hx] = radialDeadzone(0.575, 0);
  assert.ok(Math.abs(hx - 0.5) < 1e-9, 'halfway past the deadzone is half output');
});

test('padActive ignores stick drift and sees presses and tilts', () => {
  assert.equal(padActive(normalizePad(wired({}, [0.05, -0.1, 0.02, 0]))), false);
  assert.equal(padActive(normalizePad(wired({ 0: 1 }))), true);
  assert.equal(padActive(normalizePad(wired({}, [0, 0, 0.6, 0]))), true);
});

test('pickPad takes the connected pad used last', () => {
  const a = { ...wired(), timestamp: 5 };
  const b = { ...bleChrome(), timestamp: 9 };
  const gone = { ...wired(), connected: false, timestamp: 20 };
  assert.equal(pickPad([null, a, b, gone]), b);
  assert.equal(pickPad([null, null]), null);
});

test('glyphs: Xbox names for pad tokens, key caps for keys', () => {
  assert.equal(padGlyph('Pad2'), 'X');
  assert.equal(padGlyph('Pad7'), 'RT');
  assert.equal(padGlyph('Pad9'), 'Menu');
  assert.equal(padGlyph('KeyE'), null);
  assert.equal(keyGlyph('KeyE'), 'E');
  assert.equal(keyGlyph('Shift+Mouse0'), 'Shift LMB');
  const b = defaultSettings().bindings;
  assert.equal(actionGlyph(b, 'interact', 'pad'), 'Y');
  assert.equal(actionGlyph(b, 'useItem', 'pad'), 'X');
  assert.equal(actionGlyph(b, 'twoHand', 'pad'), 'Y + RB');
  assert.equal(actionGlyph(b, 'interact', 'keyboard'), 'E');
  assert.equal(actionGlyph({ ...b, interact: ['KeyG', 'Pad2'] }, 'interact', 'pad'), 'X');
});

test('Elden Ring layout: Y interacts, X uses items, Y + RB two-hands (issue #23)', () => {
  const b = defaultSettings().bindings;
  assert.deepEqual(resolveActions(b, 'Pad3', false), ['interact']);
  assert.deepEqual(resolveActions(b, 'Pad2', false), ['useItem']);
  assert.deepEqual(resolveActions(b, 'Pad5', false), ['light']);
  assert.deepEqual(resolveActions(b, 'Pad5', false, new Set(['Pad3'])), ['twoHand']);
  assert.deepEqual(resolveActions(b, 'Pad5', false, new Set(['Pad4'])), ['light']);
  assert.deepEqual(parseToken('Pad3+Pad5'), { shift: false, hold: 'Pad3', base: 'Pad5' });
  assert.equal(isPadToken('Pad3+Pad5'), true);
  assert.equal(isPadToken('KeyV'), false);
});
