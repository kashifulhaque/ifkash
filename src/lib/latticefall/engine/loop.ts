// Fixed-step loop after "Fix Your Timestep" (Glenn Fiedler): an accumulator advances the
// core in fixed ticks, and the renderer interpolates by alpha = accumulator / dt.

/** Core tick rate. Matches `DT = 1/30` in lf-core. */
export const TICK_HZ = 30;
export const TICK_MS = 1000 / TICK_HZ;
/** At most this many ticks per rendered frame; excess time is dropped, not queued. */
export const MAX_TICKS_PER_FRAME = 4;
/** Frames longer than this (tab was hidden, breakpoint) count as this long. */
const MAX_FRAME_MS = 250;

export interface LoopHandlers {
  /** Advance the core by exactly one tick. */
  tick(): void;
  /** Draw. `alpha` in [0, 1) is how far the renderer is between the last two snapshots. */
  render(alpha: number, frameMs: number): void;
}

export class FixedStepLoop {
  private handlers: LoopHandlers;
  private raf = 0;
  private last = 0;
  private acc = 0;
  private running = false;
  private paused = false;

  constructor(handlers: LoopHandlers) {
    this.handlers = handlers;
  }

  get isPaused(): boolean {
    return this.paused;
  }

  start(): void {
    if (this.running) return;
    this.running = true;
    this.last = performance.now();
    this.raf = requestAnimationFrame(this.frame);
  }

  stop(): void {
    this.running = false;
    cancelAnimationFrame(this.raf);
  }

  /** Pausing stops ticking; rendering continues so the scene stays responsive to resizes. */
  setPaused(paused: boolean): void {
    this.paused = paused;
    this.acc = 0;
  }

  private frame = (now: number): void => {
    if (!this.running) return;
    this.raf = requestAnimationFrame(this.frame);
    const frameMs = Math.min(now - this.last, MAX_FRAME_MS);
    this.last = now;

    if (!this.paused) {
      this.acc += frameMs;
      let ticks = 0;
      while (this.acc >= TICK_MS && ticks < MAX_TICKS_PER_FRAME) {
        this.handlers.tick();
        this.acc -= TICK_MS;
        ticks++;
      }
      // Hit the cap: forget the backlog so we never spiral.
      if (this.acc >= TICK_MS) this.acc %= TICK_MS;
    }
    this.handlers.render(this.acc / TICK_MS, frameMs);
  };
}
