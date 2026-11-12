// Audio volume and mute settings, saved in `localStorage`. This module has no Web Audio
// import, so menus and the home page can read and write the settings without pulling in
// the engine. It follows the Svelte store contract: `subscribe(fn)` calls `fn` at once and
// returns an unsubscribe function, so `$audioSettings` works in components.

/** The mixer buses. `master` scales the other four. */
export const BUSES = ['master', 'music', 'effects', 'ambience', 'ui'] as const;
export type Bus = (typeof BUSES)[number];

export interface AudioSettings {
  /** Volume per bus, 0 to 1. */
  volumes: Record<Bus, number>;
  muted: boolean;
}

const KEY = 'latticefall-audio';

/** The game starts with sound on at 60% (the presentation spec's defaults). */
export const DEFAULT_SETTINGS: AudioSettings = {
  volumes: { master: 0.6, music: 1, effects: 1, ambience: 1, ui: 1 },
  muted: false
};

function clamp01(v: number): number {
  return Number.isFinite(v) ? Math.min(1, Math.max(0, v)) : 0;
}

function load(): AudioSettings {
  const out: AudioSettings = { volumes: { ...DEFAULT_SETTINGS.volumes }, muted: DEFAULT_SETTINGS.muted };
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return out;
    const saved = JSON.parse(raw) as Partial<AudioSettings> | null;
    if (saved && typeof saved === 'object') {
      if (typeof saved.muted === 'boolean') out.muted = saved.muted;
      if (saved.volumes && typeof saved.volumes === 'object') {
        for (const bus of BUSES) {
          const v = (saved.volumes as Record<string, unknown>)[bus];
          if (typeof v === 'number') out.volumes[bus] = clamp01(v);
        }
      }
    }
  } catch {
    // Storage blocked or the saved value is corrupt: use the defaults.
  }
  return out;
}

function save(s: AudioSettings): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(s));
  } catch {
    // Private mode or a full quota: the settings last for this visit only.
  }
}

type Listener = (s: AudioSettings) => void;

class SettingsStore {
  private value: AudioSettings | null = null;
  private listeners = new Set<Listener>();

  /** Loads lazily so that importing this module on the server touches no storage. */
  get(): AudioSettings {
    if (!this.value) this.value = load();
    return this.value;
  }

  subscribe = (fn: Listener): (() => void) => {
    this.listeners.add(fn);
    fn(this.get());
    return () => {
      this.listeners.delete(fn);
    };
  };

  private commit(next: AudioSettings): void {
    this.value = next;
    save(next);
    for (const fn of this.listeners) fn(next);
  }

  /** Sets one bus's volume (clamped to 0-1) and saves it. */
  setVolume(bus: Bus, volume: number): void {
    const cur = this.get();
    this.commit({ ...cur, volumes: { ...cur.volumes, [bus]: clamp01(volume) } });
  }

  /** Mutes or unmutes everything. Volumes are kept, so unmuting restores them. */
  setMuted(muted: boolean): void {
    this.commit({ ...this.get(), muted });
  }

  toggleMuted(): boolean {
    this.setMuted(!this.get().muted);
    return this.get().muted;
  }

  reset(): void {
    this.commit({ volumes: { ...DEFAULT_SETTINGS.volumes }, muted: DEFAULT_SETTINGS.muted });
  }
}

export const audioSettings = new SettingsStore();

/** Convenience wrappers for the settings menu (WP-046). */
export const setBusVolume = (bus: Bus, volume: number): void => audioSettings.setVolume(bus, volume);
export const setMuted = (muted: boolean): void => audioSettings.setMuted(muted);
export const toggleMuted = (): boolean => audioSettings.toggleMuted();
export const getAudioSettings = (): AudioSettings => audioSettings.get();
