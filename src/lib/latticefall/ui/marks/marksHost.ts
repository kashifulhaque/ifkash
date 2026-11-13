// The Marks' side of the engine (WP-088): what the UI reads (the earned Marks, the timer, the
// gesture under way, the toasts) and the one command it sends (a gesture). No Three.js or WASM
// imports; `LfWorld` satisfies `MarksWorld` structurally. Marks are decided in the core: this
// host only shows them, records them in the codex store (local), and, when the player is signed
// in, syncs them to the cloud (`syncMarks`).

import { writable, type Readable } from 'svelte/store';
import { codex } from '../../codex/store';
import { syncMarks } from '../../online';
import { loadToken } from '$lib/splitterApi';
import type { GameEvent } from '../../engine/events';
import { COMMAND } from '../menus/menuHost';
import {
  EMPTY_MARKS,
  earnedSlugs,
  parseMarksJson,
  parseMarksState,
  type MarkInfo,
  type MarksState
} from './marksData';

/** The part of `LfWorld` the Marks read. */
export interface MarksWorld {
  marks_json(): string;
  marks_state(): ArrayLike<number>;
}

export interface MarkToastItem {
  id: number;
  mark: MarkInfo;
}

export interface MarksHost {
  /** Every Mark, in the core's order. */
  list(): readonly MarkInfo[];
  /** The slugs of the Marks earned in this world. */
  earned: Readable<ReadonlySet<string>>;
  /** In-game ticks, updated every tenth of a second. */
  ticks: Readable<number>;
  /** The gesture under way (0 to 5), or null. */
  gesture: Readable<number | null>;
  /** Marks just earned, newest last, each for a few seconds. */
  toasts: Readable<MarkToastItem[]>;
  /** Reads the world. The engine calls it after every tick. */
  refresh(): void;
  /** The engine passes every tick's decoded events. */
  handleEvents(events: readonly GameEvent[]): void;
  /** Queues `Command::Gesture(n)`. */
  perform(n: number): void;
  dismiss(id: number): void;
  dispose(): void;
}

/** How long a Mark's toast stays, in milliseconds. */
const TOAST_MS = 5500;
/** The timer store updates when the tenth changes: every 3 ticks. */
const TIMER_STEP = 3;

export function createMarksHost(
  world: MarksWorld,
  sink: { command(kind: number, a: number, b: number): void }
): MarksHost {
  const list = parseMarksJson(world.marks_json());
  const earned = writable<ReadonlySet<string>>(new Set());
  const ticks = writable(0);
  const gesture = writable<number | null>(null);
  const toasts = writable<MarkToastItem[]>([]);
  const timers = new Set<ReturnType<typeof setTimeout>>();
  let state: MarksState = EMPTY_MARKS;
  let nextId = 1;
  let syncing: ReturnType<typeof setTimeout> | undefined;

  /** Records the Mark locally, and in the cloud when signed in. A failure is only "no online data". */
  function record(slug: string): void {
    codex.mark(slug);
    if (!loadToken()) return;
    clearTimeout(syncing);
    syncing = setTimeout(() => {
      syncMarks().catch(() => {});
    }, 800);
  }

  function refresh(): void {
    const next = parseMarksState(world.marks_state());
    if (next.mask !== state.mask) {
      const slugs = earnedSlugs(next.mask, list);
      earned.set(new Set(slugs));
      // A Mark the core already had (a loaded save) still belongs in the local list.
      for (const slug of slugs) if (!codex.has(slug)) record(slug);
    }
    if (next.gesture !== state.gesture) gesture.set(next.gesture);
    if (Math.floor(next.ticks / TIMER_STEP) !== Math.floor(state.ticks / TIMER_STEP)) ticks.set(next.ticks);
    state = next;
  }

  return {
    list: () => list,
    earned,
    ticks,
    gesture,
    toasts,
    refresh,
    handleEvents(events) {
      for (const e of events) {
        if (e.kind !== 'mark') continue;
        const mark = list[e.mark];
        if (!mark) continue;
        record(mark.slug);
        const id = nextId++;
        toasts.update((t) => [...t, { id, mark }]);
        const timer = setTimeout(() => {
          timers.delete(timer);
          toasts.update((t) => t.filter((x) => x.id !== id));
        }, TOAST_MS);
        timers.add(timer);
      }
    },
    perform(n) {
      sink.command(COMMAND.Gesture, n, 0);
    },
    dismiss(id) {
      toasts.update((t) => t.filter((x) => x.id !== id));
    },
    dispose() {
      for (const t of timers) clearTimeout(t);
      timers.clear();
      clearTimeout(syncing);
    }
  };
}
