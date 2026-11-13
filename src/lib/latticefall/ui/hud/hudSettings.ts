// HUD options: scale (75 to 150 percent) and when the HUD shows. Saved in localStorage; the
// menus (WP-046) will bind controls to this store. No Three.js or WASM imports.

import { writable, type Writable } from 'svelte/store';

export type HudVisibility = 'always' | 'auto' | 'hidden';

export interface HudSettings {
  /** Percent, 75 to 150. */
  scale: number;
  /** `auto` hides the HUD out of combat when there is nothing to show. */
  visibility: HudVisibility;
  /** Show the in-game timer (WP-088). Off by default; the timer is recorded either way. */
  showTimer: boolean;
}

export const SCALE_MIN = 75;
export const SCALE_MAX = 150;
const KEY = 'latticefall-hud';

export const DEFAULT_HUD_SETTINGS: HudSettings = { scale: 100, visibility: 'always', showTimer: false };

export function clampScale(n: number): number {
  if (!Number.isFinite(n)) return DEFAULT_HUD_SETTINGS.scale;
  return Math.round(Math.min(SCALE_MAX, Math.max(SCALE_MIN, n)));
}

function isVisibility(v: unknown): v is HudVisibility {
  return v === 'always' || v === 'auto' || v === 'hidden';
}

/** Reads the saved settings, then lets `&hudscale=N` and `&hud=always|auto|hidden` override them. */
export function loadHudSettings(search = ''): HudSettings {
  const out = { ...DEFAULT_HUD_SETTINGS };
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) ?? 'null') as Partial<HudSettings> | null;
    if (saved) {
      if (typeof saved.scale === 'number') out.scale = clampScale(saved.scale);
      if (isVisibility(saved.visibility)) out.visibility = saved.visibility;
      if (typeof saved.showTimer === 'boolean') out.showTimer = saved.showTimer;
    }
  } catch {
    // Storage can be blocked or hold junk; fall back to the defaults.
  }
  const q = new URLSearchParams(search);
  const scale = q.get('hudscale');
  if (scale !== null) out.scale = clampScale(Number(scale));
  const vis = q.get('hud');
  if (isVisibility(vis)) out.visibility = vis;
  const timer = q.get('timer');
  if (timer === '1' || timer === '0') out.showTimer = timer === '1';
  return out;
}

export interface HudSettingsStore extends Writable<HudSettings> {
  /** Saves the current settings. Call it when the player changes one. */
  persist(): void;
}

export function createHudSettings(search = ''): HudSettingsStore {
  const store = writable<HudSettings>(loadHudSettings(search));
  return {
    ...store,
    persist() {
      let current = DEFAULT_HUD_SETTINGS;
      store.subscribe((s) => (current = s))();
      try {
        localStorage.setItem(KEY, JSON.stringify(current));
      } catch {
        // Not saving is harmless.
      }
    }
  };
}
