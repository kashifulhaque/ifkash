// Swing cues: decides, from each tick's snapshot, when an actor's attack should make its
// swing sound and when an enemy's attack starts. Pure: no Web Audio, no I/O. The engine
// feeds it the snapshot once per core tick (never per rendered frame), so a cue fires once
// per swing however many frames are drawn between ticks.

/** The fields of a snapshot record the cues read. */
export interface SwingActor {
  id: number;
  kind: number;
  flags: number;
  moveId: number;
  moveFrame: number;
}

/** The move timing the cues read, as in the content digest. */
export interface SwingTiming {
  /** First hitbox frame; a move without one makes no swing sound. */
  strikeStart?: number;
}

export interface SwingCue {
  id: number;
  kind: number;
  /** `start`: the move's first tick. `swing`: `SWING_LEAD_FRAMES` before the hitbox opens. */
  what: 'start' | 'swing';
  moveId: number;
  /** The move's wind-up length in frames (its `strikeStart`). */
  windup: number;
}

/** The swing is heard this many frames before the first hitbox frame, as the blade speeds up. */
export const SWING_LEAD_FRAMES = 2;

/** The snapshot's `DEAD` flag (`FLAG_DEAD` in `engine/snapshot.ts`). */
const DEAD = 1;

export class SwingCues {
  private seen = new Map<number, { moveId: number; frame: number; swung: boolean }>();

  /**
   * The cues for this tick's actors. `resolve` looks a move number up in the digest, so the
   * timing follows the frame data and nothing here is tuned to one move.
   */
  step(actors: readonly SwingActor[], resolve: (moveId: number) => SwingTiming | undefined): SwingCue[] {
    const out: SwingCue[] = [];
    const live = new Set<number>();
    for (const a of actors) {
      live.add(a.id);
      const last = this.seen.get(a.id);
      if (a.moveId === 0 || (a.flags & DEAD) !== 0) {
        if (last) this.seen.delete(a.id);
        continue;
      }
      // A new move, or the same move started again (its frame went back).
      const fresh = !last || last.moveId !== a.moveId || a.moveFrame < last.frame;
      const state = fresh ? { moveId: a.moveId, frame: a.moveFrame, swung: false } : last;
      state.frame = a.moveFrame;
      this.seen.set(a.id, state);
      const strike = resolve(a.moveId)?.strikeStart;
      if (strike === undefined) continue;
      const cue = { id: a.id, kind: a.kind, moveId: a.moveId, windup: strike };
      if (fresh) out.push({ ...cue, what: 'start' });
      if (!state.swung && a.moveFrame >= Math.max(strike - SWING_LEAD_FRAMES, 0)) {
        state.swung = true;
        // A move first seen after its hitbox closed (an actor that walked into view) stays quiet.
        if (!fresh || a.moveFrame <= strike) out.push({ ...cue, what: 'swing' });
      }
    }
    for (const id of this.seen.keys()) if (!live.has(id)) this.seen.delete(id);
    return out;
  }
}

/** Wind-ups at least this long (frames) are heavy swings: the heavy attack, not the light chain. */
export const HEAVY_WINDUP_FRAMES = 15;
