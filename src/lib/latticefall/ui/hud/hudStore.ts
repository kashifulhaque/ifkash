// Glue between the engine and the HUD component: the engine pushes parsed state, raw event
// words, and the lock-on marker position here; the component reads the stores. Read-only
// with respect to the simulation.

import { writable, type Readable } from 'svelte/store';
import { EVENT_WORDS } from '../../engine/events';
import { emptyHud, type HudState } from './hudState';

export interface Notice {
  id: number;
  text: string;
}

export interface ScreenPoint {
  x: number;
  y: number;
}

/** Where the engine reports to. */
export interface HudSink {
  state(s: HudState): void;
  /** The raw words from `drain_events`, for notifications. */
  events(words: ArrayLike<number>): void;
  /** Screen position of the locked-on actor in CSS pixels over the canvas, or null. */
  lock(p: ScreenPoint | null): void;
}

export interface HudController {
  sink: HudSink;
  state: Readable<HudState>;
  notices: Readable<Notice[]>;
  lock: Readable<ScreenPoint | null>;
  /** Adds a notification. */
  notify(text: string): void;
  /** Cancels pending notification timers. */
  dispose(): void;
}

/** How long a notification stays, in milliseconds, and how many show at once. */
const NOTICE_MS = 3200;
const NOTICE_MAX = 3;

/** Notification text for the Life events worth a line, from raw event words. */
export function noticesFromWords(words: ArrayLike<number>): string[] {
  const out: string[] = [];
  let i = 0;
  while (i < words.length) {
    const kind = words[i];
    const len = EVENT_WORDS[kind];
    if (len === undefined || i + len > words.length) break;
    if (kind === 2) {
      const code = words[i + 1];
      if (code === 3) out.push('Lantern lit');
      else if (code === 5) out.push(`Remnant recovered: ${words[i + 2]} Glimmer`);
      else if (code === 6) out.push('Remnant lost');
    }
    i += len;
  }
  return out;
}

export function createHudController(): HudController {
  const state = writable<HudState>(emptyHud());
  const notices = writable<Notice[]>([]);
  const lock = writable<ScreenPoint | null>(null);
  const timers = new Set<ReturnType<typeof setTimeout>>();
  let nextId = 1;
  let prev: HudState | null = null;
  let lastLock: ScreenPoint | null = null;

  function notify(text: string): void {
    const id = nextId++;
    notices.update((list) => [...list, { id, text }].slice(-NOTICE_MAX));
    const t = setTimeout(() => {
      timers.delete(t);
      notices.update((list) => list.filter((n) => n.id !== id));
    }, NOTICE_MS);
    timers.add(t);
  }

  const sink: HudSink = {
    state(s) {
      // Pickups have no event of their own yet, so read them off the change in what's held.
      if (prev && !s.dead) {
        if (s.glimmer > prev.glimmer) notify(`+${s.glimmer - prev.glimmer} Glimmer`);
        if (s.item.num !== 0 && s.item.num === prev.item.num && !s.item.flask && (s.item.count ?? 0) > (prev.item.count ?? 0)) {
          notify(`Picked up ${s.item.name}`);
        }
      }
      prev = s;
      state.set(s);
    },
    events(words) {
      for (const text of noticesFromWords(words)) notify(text);
    },
    lock(p) {
      // The marker follows the camera every frame; skip the store write when it hasn't moved.
      if (p === null && lastLock === null) return;
      if (p && lastLock && Math.abs(p.x - lastLock.x) < 0.5 && Math.abs(p.y - lastLock.y) < 0.5) return;
      lastLock = p;
      lock.set(p);
    }
  };

  return {
    sink,
    state,
    notices,
    lock,
    notify,
    dispose() {
      for (const t of timers) clearTimeout(t);
      timers.clear();
    }
  };
}
