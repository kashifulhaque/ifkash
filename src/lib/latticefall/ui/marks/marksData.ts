// Marks, gestures, and the in-game timer as the UI sees them (WP-088). Pure and read-only: no
// Three.js or WASM imports. The core decides what is earned and what is under way; this parses
// what `LfWorld.marks_json` and `LfWorld.marks_state` return (game/crates/wasm/src/lib.rs) and
// formats it. See docs/game/06-progression-and-systems.md, "Achievements", "Gestures and photo
// mode", and "Timer and new game plus".

/** Simulation ticks per second. The in-game timer is the tick counter, never the wall clock. */
export const TICKS_PER_SECOND = 30;

export interface MarkInfo {
  /** The ID stored locally and in the cloud. */
  slug: string;
  name: string;
  /** How to earn it. */
  how: string;
}

/** `marks_state`: the earned mask, the gesture under way, and the timer. */
export interface MarksState {
  /** Bit `i` is set when Mark `i` of `marks_json` is earned. */
  mask: number;
  /** The gesture being performed (0 to 5), or null. */
  gesture: number | null;
  /** In-game ticks. */
  ticks: number;
  /** Ordinary enemies killed after the Verge (Pacifist Path ends at 1). */
  kills: number;
}

export const EMPTY_MARKS: MarksState = { mask: 0, gesture: null, ticks: 0, kills: 0 };

/** Parses `marks_state`; anything shorter than four words is an empty state. */
export function parseMarksState(w: ArrayLike<number>): MarksState {
  if (w.length < 4) return EMPTY_MARKS;
  return { mask: w[0] >>> 0, gesture: w[1] > 0 ? w[1] - 1 : null, ticks: w[2] >>> 0, kills: w[3] >>> 0 };
}

/** Parses `marks_json`; a bad document is an empty list. */
export function parseMarksJson(json: string): MarkInfo[] {
  try {
    const data: unknown = JSON.parse(json);
    if (!Array.isArray(data)) return [];
    return data.flatMap((m): MarkInfo[] => {
      const o = m as Partial<MarkInfo> | null;
      return o && typeof o.slug === 'string' && typeof o.name === 'string' && typeof o.how === 'string'
        ? [{ slug: o.slug, name: o.name, how: o.how }]
        : [];
    });
  } catch {
    return [];
  }
}

export function isEarned(mask: number, index: number): boolean {
  return index >= 0 && index < 32 && ((mask >>> index) & 1) === 1;
}

/** The slugs of the Marks set in `mask`, in list order. */
export function earnedSlugs(mask: number, list: readonly MarkInfo[]): string[] {
  return list.filter((_, i) => isEarned(mask, i)).map((m) => m.slug);
}

/**
 * The timer as text: `M:SS.t`, or `H:MM:SS.t` from an hour on, tenths of a second included.
 * Ticks only: 30 make a second.
 */
export function formatTimer(ticks: number): string {
  const t = Math.max(0, Math.floor(ticks));
  const tenths = Math.floor((t * 10) / TICKS_PER_SECOND);
  const d = tenths % 10;
  const secs = Math.floor(tenths / 10);
  const s = secs % 60;
  const m = Math.floor(secs / 60) % 60;
  const h = Math.floor(secs / 3600);
  const two = (n: number) => String(n).padStart(2, '0');
  return h > 0 ? `${h}:${two(m)}:${two(s)}.${d}` : `${m}:${two(s)}.${d}`;
}

/** The six gestures, in the order of the core's `Command::Gesture` numbers. */
export const GESTURES = [
  { id: 0, name: 'Wave', hint: 'A raised hand.' },
  { id: 1, name: 'Bow', hint: 'A courtesy, and the right thing to do for some company.' },
  { id: 2, name: 'Point forward', hint: 'Look there.' },
  { id: 3, name: 'Sit', hint: 'Stays until you move.' },
  { id: 4, name: 'Shrug', hint: 'No idea.' },
  { id: 5, name: 'Beckon', hint: 'Come here.' }
] as const;
