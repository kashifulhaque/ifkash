// Settings the menus own: graphics, difficulty, and accessibility. Saved in `localStorage`;
// no Three.js or WASM imports. Controls live in `input/bindings.ts`, audio in
// `audio/settings.ts`, and the HUD's scale and visibility in `ui/hud/hudSettings.ts`; the
// settings menu drives those rather than copying them.
//
// `APPLIED` says which of these the game reads today. The rest are saved and shown, and the
// menu labels them, until the system that consumes them lands.

import { writable, type Readable } from 'svelte/store';

export type Quality = 'low' | 'medium' | 'high';
export type ColorBlind = 'off' | 'protanopia' | 'deuteranopia' | 'tritanopia';

export interface MenuSettings {
  graphics: { quality: Quality };
  difficulty: {
    /** Percent, 25 to 200. */
    damageTaken: number;
    /** Percent, 50 to 200. */
    enemyHp: number;
    infiniteStamina: boolean;
    /** Percent, 50 to 100. A browser-side setting. */
    gameSpeed: number;
    keepGlimmer: boolean;
    bossSkip: boolean;
  };
  accessibility: {
    /** Percent, 75 to 200. */
    textSize: number;
    reducedMotion: boolean;
    colorBlind: ColorBlind;
    subtitles: boolean;
  };
}

export const RANGES = {
  damageTaken: { min: 25, max: 200, step: 5 },
  enemyHp: { min: 50, max: 200, step: 5 },
  gameSpeed: { min: 50, max: 100, step: 5 },
  textSize: { min: 75, max: 200, step: 5 }
} as const;

export const DEFAULT_MENU_SETTINGS: MenuSettings = {
  graphics: { quality: 'medium' },
  difficulty: {
    damageTaken: 100,
    enemyHp: 100,
    infiniteStamina: false,
    gameSpeed: 100,
    keepGlimmer: false,
    bossSkip: false
  },
  accessibility: { textSize: 100, reducedMotion: false, colorBlind: 'off', subtitles: true }
};

/** The Wayfarer mode preset of the combat spec's difficulty table. */
export const WAYFARER_PRESET: MenuSettings['difficulty'] = {
  damageTaken: 25,
  enemyHp: 60,
  infiniteStamina: true,
  gameSpeed: 100,
  keepGlimmer: true,
  bossSkip: true
};

/** Settings the game reads today: only the text size (in the menus). */
export const APPLIED = {
  quality: false,
  difficulty: false,
  textSize: true,
  reducedMotion: false,
  colorBlind: false,
  subtitles: false
} as const;

const KEY = 'latticefall-menu-settings';

function clamp(n: unknown, min: number, max: number, fallback: number): number {
  return typeof n === 'number' && Number.isFinite(n) ? Math.round(Math.min(max, Math.max(min, n))) : fallback;
}

/** Merges saved values over the defaults, ignoring anything malformed. */
export function sanitize(raw: unknown): MenuSettings {
  const d = structuredClone(DEFAULT_MENU_SETTINGS);
  if (!raw || typeof raw !== 'object') return d;
  const r = raw as Partial<Record<keyof MenuSettings, Record<string, unknown>>>;
  const q = r.graphics?.quality;
  if (q === 'low' || q === 'medium' || q === 'high') d.graphics.quality = q;
  const df = r.difficulty ?? {};
  d.difficulty.damageTaken = clamp(df.damageTaken, 25, 200, d.difficulty.damageTaken);
  d.difficulty.enemyHp = clamp(df.enemyHp, 50, 200, d.difficulty.enemyHp);
  d.difficulty.gameSpeed = clamp(df.gameSpeed, 50, 100, d.difficulty.gameSpeed);
  for (const k of ['infiniteStamina', 'keepGlimmer', 'bossSkip'] as const) {
    if (typeof df[k] === 'boolean') d.difficulty[k] = df[k] as boolean;
  }
  const ac = r.accessibility ?? {};
  d.accessibility.textSize = clamp(ac.textSize, 75, 200, d.accessibility.textSize);
  if (typeof ac.reducedMotion === 'boolean') d.accessibility.reducedMotion = ac.reducedMotion;
  if (typeof ac.subtitles === 'boolean') d.accessibility.subtitles = ac.subtitles;
  const cb = ac.colorBlind;
  if (cb === 'off' || cb === 'protanopia' || cb === 'deuteranopia' || cb === 'tritanopia') d.accessibility.colorBlind = cb;
  return d;
}

export interface MenuSettingsStore extends Readable<MenuSettings> {
  /** Replaces the settings with `fn`'s result and saves them. */
  update(fn: (s: MenuSettings) => MenuSettings): void;
  reset(): void;
}

function load(): MenuSettings {
  let settings = structuredClone(DEFAULT_MENU_SETTINGS);
  try {
    settings = sanitize(JSON.parse(localStorage.getItem(KEY) ?? 'null'));
  } catch {
    // Blocked or corrupt storage: use the defaults.
  }
  // Reduced motion is on by default when the operating system asks for it.
  try {
    if (localStorage.getItem(KEY) === null && matchMedia('(prefers-reduced-motion: reduce)').matches) {
      settings.accessibility.reducedMotion = true;
    }
  } catch {
    // No matchMedia: leave it off.
  }
  return settings;
}

export function createMenuSettings(): MenuSettingsStore {
  const store = writable<MenuSettings>(load());
  const save = (s: MenuSettings) => {
    try {
      localStorage.setItem(KEY, JSON.stringify(s));
    } catch {
      // Not saving is harmless.
    }
  };
  return {
    subscribe: store.subscribe,
    update(fn) {
      store.update((s) => {
        const next = sanitize(fn(structuredClone(s)));
        save(next);
        return next;
      });
    },
    reset() {
      const d = structuredClone(DEFAULT_MENU_SETTINGS);
      save(d);
      store.set(d);
    }
  };
}
