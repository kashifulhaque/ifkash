// Grounded or airborne, from the core's `AIRBORNE` snapshot flag. Pure math, read-only.
//
// The flag says whether the actor is off the ground; the vertical speed from position deltas
// only splits that into rising and falling. Heights alone used to decide it, which latched:
// landing a few centimeters higher than the takeoff (a step, a slope, or the controller
// settling) kept the actor in the jump or fall pose while it stood still (issues #16, #17).

/** How long the landing animation holds control, in seconds. */
export const LAND_SECONDS = 0.38;
/** Upward speed (m/s) that counts as rising rather than falling. */
const RISE_VY = 0.3;

export type AirPhase = 'ground' | 'rising' | 'falling' | 'landing';

export class AirTracker {
  phase: AirPhase = 'ground';
  /** Seconds in the current phase. */
  t = 0;
  private lastY = NaN;

  /**
   * Advance by `dt` seconds (> 0) with the actor's current height and the core's airborne
   * flag.
   */
  update(y: number, dt: number, airborne: boolean): AirPhase {
    const vy = Number.isNaN(this.lastY) || dt <= 0 ? 0 : (y - this.lastY) / dt;
    this.lastY = y;
    this.t += dt;

    if (airborne) {
      // A jump rises until its peak; anything else in the air falls.
      const up = vy > RISE_VY || (this.phase === 'rising' && vy > -RISE_VY);
      this.set(up ? 'rising' : 'falling');
    } else if (this.phase === 'rising' || this.phase === 'falling') {
      this.set('landing');
    } else if (this.phase === 'landing' && this.t >= LAND_SECONDS) {
      this.set('ground');
    }
    return this.phase;
  }

  /** Force a phase (dev preview). */
  force(phase: AirPhase, t: number): void {
    this.phase = phase;
    this.t = t;
  }

  private set(p: AirPhase): void {
    if (p !== this.phase) {
      this.phase = p;
      this.t = 0;
    }
  }
}
