// Subtitles for important sounds (WP-082). The captions come from the same simulation events
// the audio module plays from (`engine/events.ts`), never from the audio code's clock, so a
// caption shows on the tick its sound is cued and a muted game still gets it. NPC lines need no
// caption: the dialogue (WP-068) is text. No Three.js or WASM imports.

import { writable, type Readable } from 'svelte/store';
import {
  BOSS_DEFEATED,
  BOSS_FIGHT_STARTED,
  BOSS_PHASE_STARTED,
  BOSS_SEALED,
  HIT_GUARD_BROKEN,
  MECHANIC_BRIAR_CHARGE_STARTED,
  MECHANIC_BRIAR_TREE_STRUCK,
  type GameEvent
} from '../../engine/events';

export interface Caption {
  id: number;
  text: string;
}

/** `MechanicEvent` sub-kinds the audio module cues (see `audio/engine.ts`), by caption. */
const MECHANIC_CAPTIONS: Record<number, string> = {
  191: 'The bell winds up',
  192: 'The bell tolls',
  193: 'The toll’s shockwave hits you',
  194: 'Ice cracks',
  195: 'Ice breaks into freezing water',
  197: 'A chain snaps',
  [MECHANIC_BRIAR_CHARGE_STARTED]: 'The Stag charges',
  [MECHANIC_BRIAR_TREE_STRUCK]: 'The Stag crashes into a tree'
};

/**
 * The caption texts for a batch of events, in order. `bossName` names the fight under way (the
 * HUD's boss bar carries it); without one the captions say "The boss".
 */
export function captionTexts(events: readonly GameEvent[], bossName?: string): string[] {
  const out: string[] = [];
  const boss = bossName || 'The boss';
  for (const e of events) {
    if (e.kind === 'mechanic') {
      const text = MECHANIC_CAPTIONS[e.what];
      if (text) out.push(text);
    } else if (e.kind === 'boss') {
      if (e.what === BOSS_SEALED) out.push('A gate seals');
      else if (e.what === BOSS_FIGHT_STARTED || e.what === BOSS_PHASE_STARTED) out.push(`${boss} roars`);
      else if (e.what === BOSS_DEFEATED) out.push(`${boss} falls`);
    } else if (e.kind === 'hit' && e.result === HIT_GUARD_BROKEN) {
      out.push('A guard breaks');
    }
  }
  return out;
}

/** How long a caption stays, in milliseconds, and the most that show at once. */
export const CAPTION_MS = 3500;
export const CAPTION_MAX = 3;
/** The same caption within this many milliseconds shows once (a toll can fire several events). */
const DEDUPE_MS = 900;

export interface CaptionStore extends Readable<Caption[]> {
  /** Adds the captions for `events`. */
  push(events: readonly GameEvent[], bossName?: string): void;
  /** Adds one caption by its text (the settings menu's preview uses it). */
  add(text: string): void;
  clear(): void;
  dispose(): void;
}

export function createCaptions(now: () => number = () => performance.now()): CaptionStore {
  const { subscribe, update, set } = writable<Caption[]>([]);
  const lastShown = new Map<string, number>();
  const timers = new Set<ReturnType<typeof setTimeout>>();
  let next = 1;

  function add(text: string): void {
    const t = now();
    const prev = lastShown.get(text);
    if (prev !== undefined && t - prev < DEDUPE_MS) return;
    lastShown.set(text, t);
    const id = next++;
    update((list) => [...list, { id, text }].slice(-CAPTION_MAX));
    const timer = setTimeout(() => {
      timers.delete(timer);
      update((list) => list.filter((c) => c.id !== id));
    }, CAPTION_MS);
    timers.add(timer);
  }

  return {
    subscribe,
    add,
    push(events, bossName) {
      for (const text of captionTexts(events, bossName)) add(text);
    },
    clear() {
      for (const t of timers) clearTimeout(t);
      timers.clear();
      lastShown.clear();
      set([]);
    },
    dispose() {
      for (const t of timers) clearTimeout(t);
      timers.clear();
    }
  };
}
