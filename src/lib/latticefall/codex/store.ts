// The codex unlock store: which entries the visitor has found, saved in `localStorage`.
// No Three.js or WASM import (it must stay that way), so the home
// page can read it without pulling in the game bundle.

import { writable, type Readable } from 'svelte/store';

const KEY = 'ifkash-latticefall-codex';
const VERSION = 1;

export interface CodexState {
  /** Unlocked entry keys, in the order they were found. */
  entries: string[];
  /** Marks earned, such as `reader` for opening the Ledger's résumé. */
  marks: string[];
}

const empty = (): CodexState => ({ entries: [], marks: [] });

function strings(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return [...new Set(value.filter((v): v is string => typeof v === 'string' && v.length > 0 && v.length < 120))];
}

/** Reads the saved state; an unreadable or missing save is an empty codex. */
export function readCodex(): CodexState {
  try {
    if (typeof localStorage === 'undefined') return empty();
    const raw = localStorage.getItem(KEY);
    if (!raw) return empty();
    const data = JSON.parse(raw) as Partial<CodexState>;
    return { entries: strings(data.entries), marks: strings(data.marks) };
  } catch {
    return empty();
  }
}

function write(state: CodexState): void {
  try {
    localStorage.setItem(KEY, JSON.stringify({ version: VERSION, ...state }));
  } catch {
    // Private windows and blocked storage: the unlock lasts for this page only.
  }
}

export interface CodexStore extends Readable<CodexState> {
  /** Unlocks an entry. Returns true when it was new. */
  unlock(key: string): boolean;
  /** Earns a mark. Returns true when it was new. */
  mark(name: string): boolean;
  has(key: string): boolean;
  /** Forgets everything. */
  reset(): void;
  /** Re-reads storage, for example after another tab saved. */
  reload(): void;
}

export function createCodexStore(): CodexStore {
  const { subscribe, set } = writable<CodexState>(empty());
  let current = empty();
  const publish = (next: CodexState) => {
    current = next;
    set(next);
  };
  const reload = () => publish(readCodex());
  reload();
  return {
    subscribe,
    unlock(key) {
      reload(); // Another tab may have saved since.
      if (current.entries.includes(key)) return false;
      const next = { ...current, entries: [...current.entries, key] };
      write(next);
      publish(next);
      return true;
    },
    mark(name) {
      reload();
      if (current.marks.includes(name)) return false;
      const next = { ...current, marks: [...current.marks, name] };
      write(next);
      publish(next);
      return true;
    },
    has: (key) => current.entries.includes(key),
    reset() {
      write(empty());
      publish(empty());
    },
    reload
  };
}

/** The shared store the game and the Ledger use. */
export const codex = createCodexStore();
