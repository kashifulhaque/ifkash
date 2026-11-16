// Control placeholders in authored text. Tutorial messages say `Press {interact} to read`
// instead of naming a key, and the HUD fills each placeholder in from the player's real
// bindings for the device they used last: a key on the keyboard, a button on a pad. Pure: no
// DOM, Three.js, or WASM.
//
// A placeholder is an action name from `bindings.ts` (`{jump}`, `{useItem}`) or one of the
// two stick placeholders, `{move}` and `{look}`. Unknown placeholders stay as written.

import { ALL_ACTIONS, type Action, type Bindings } from './bindings';
import { actionGlyph, type InputDevice } from './device';

/** A run of plain text, or a control to draw as a key cap or a pad button. */
export type PromptPart = { text: string } | { control: string; pad: boolean };

const ACTIONS = new Set<string>(ALL_ACTIONS);

/** The glyph for a placeholder name, or null if the name isn't one. */
export function placeholderGlyph(name: string, bindings: Bindings, device: InputDevice): string | null {
  // Touch shows the keyboard's words: its on-screen buttons carry the same action names.
  const pad = device === 'pad';
  if (name === 'move') {
    if (pad) return 'LS';
    const keys = (['moveForward', 'moveLeft', 'moveBack', 'moveRight'] as const).map((a) => actionGlyph(bindings, a, 'keyboard'));
    return keys.every(Boolean) ? keys.join(' ') : 'W A S D';
  }
  if (name === 'look') return pad ? 'RS' : 'Mouse';
  if (!ACTIONS.has(name)) return null;
  return actionGlyph(bindings, name as Action, pad ? 'pad' : 'keyboard') || null;
}

/** Splits `text` into plain runs and controls for `device`. */
export function promptParts(text: string, bindings: Bindings, device: InputDevice): PromptPart[] {
  const out: PromptPart[] = [];
  const pad = device === 'pad';
  let last = 0;
  for (const m of text.matchAll(/\{(\w+)\}/g)) {
    const glyph = placeholderGlyph(m[1], bindings, device);
    if (glyph === null) continue;
    if (m.index > last) out.push({ text: text.slice(last, m.index) });
    out.push({ control: glyph, pad });
    last = m.index + m[0].length;
  }
  if (last < text.length) out.push({ text: text.slice(last) });
  return out;
}

/** `text` with every placeholder replaced by its glyph, for plain-text uses. */
export function promptText(text: string, bindings: Bindings, device: InputDevice): string {
  return promptParts(text, bindings, device)
    .map((p) => ('text' in p ? p.text : p.control))
    .join('');
}
