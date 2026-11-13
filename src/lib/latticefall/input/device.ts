// The input device the player used last, so prompts can show that device's glyphs. No DOM,
// no Three.js, no WASM: the HUD reads it directly.

import { writable } from 'svelte/store';
import { parseToken, type Action, type Bindings } from './bindings';
import { padGlyph } from './gamepad';

export type InputDevice = 'keyboard' | 'pad' | 'touch';

/** Set by `InputController` on each input; svelte skips the update when it doesn't change. */
export const lastDevice = writable<InputDevice>('keyboard');

/** A keyboard or mouse token as a short key cap: `KeyE` is `E`, `Mouse0` is `LMB`. */
export function keyGlyph(token: string): string {
  const { shift, base } = parseToken(token);
  const name =
    /^Key([A-Z])$/.exec(base)?.[1] ??
    /^Digit(\d)$/.exec(base)?.[1] ??
    { Mouse0: 'LMB', Mouse1: 'MMB', Mouse2: 'RMB', WheelUp: 'Wheel up', WheelDown: 'Wheel down', Space: 'Space', Escape: 'Esc' }[base] ??
    base.replace(/^Arrow/, '');
  return shift ? `Shift ${name}` : name;
}

/** The glyph to show for `action` on `device`: its first pad or first key binding. */
export function actionGlyph(bindings: Bindings, action: Action, device: InputDevice): string {
  const tokens = bindings[action] ?? [];
  if (device === 'pad') {
    for (const t of tokens) {
      const g = padGlyph(t);
      if (g) return g;
    }
  }
  const key = tokens.find((t) => !t.startsWith('Pad'));
  return key ? keyGlyph(key) : '';
}
