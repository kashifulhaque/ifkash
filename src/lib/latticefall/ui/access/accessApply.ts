// Turns the saved menu settings into what the engine and the page root take (WP-082). Pure: no
// Three.js or WASM imports (the engine import is a type only), so the page can use it before
// the engine loads.

import type { AccessOptions } from '../../engine';
import type { MenuSettings } from '../menus/menuSettings';

/** What `Engine.applyAccess` takes. */
export function accessOptions(s: MenuSettings): AccessOptions {
  return {
    reducedMotion: s.accessibility.reducedMotion,
    colorBlind: s.accessibility.colorBlind,
    gameSpeed: s.difficulty.gameSpeed,
    fov: s.accessibility.fov,
    cameraDistance: s.accessibility.cameraDistance
  };
}

/** The attributes and custom property the page root carries, so DOM UI can follow the settings. */
export function rootAccess(s: MenuSettings): { cb: string; calm: string | undefined; text: number } {
  return {
    cb: s.accessibility.colorBlind,
    calm: s.accessibility.reducedMotion ? '' : undefined,
    text: s.accessibility.textSize / 100
  };
}
