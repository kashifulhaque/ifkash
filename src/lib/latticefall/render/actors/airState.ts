// Grounded or airborne, derived from position deltas. Read-only: the snapshot has no
// grounded flag yet (see the WP-013 report), so the renderer infers it. Pure math.

/** Vertical speed (m/s) above which an actor counts as leaving the ground. */
export const AIR_VY = 1.2;
/** Height above the last ground height (m) that also counts as airborne. */
export const AIR_HEIGHT = 0.12;
/** How long the landing animation holds control, in seconds. */
export const LAND_SECONDS = 0.38;

export type AirPhase = 'ground' | 'rising' | 'falling' | 'landing';

export class AirTracker {
  phase: AirPhase = 'ground';
  /** Seconds in the current phase. */
  t = 0;
  private groundY = NaN;
  private lastY = NaN;

  /** Advance by `dt` seconds (> 0) with the actor's current height. */
  update(y: number, dt: number): AirPhase {
    if (Number.isNaN(this.groundY)) {
      this.groundY = y;
      this.lastY = y;
    }
    const vy = dt > 0 ? (y - this.lastY) / dt : 0;
    this.lastY = y;
    const offGround = y - this.groundY > AIR_HEIGHT || Math.abs(vy) > AIR_VY;
    const inAir = this.phase === 'rising' || this.phase === 'falling';
    this.t += dt;

    if (offGround) {
      const up = vy > 0.3 || (this.phase === 'rising' && vy > -0.3);
      this.set(up ? 'rising' : 'falling');
    } else if (inAir) {
      this.groundY = y;
      this.set('landing');
    } else {
      // Follow slopes and steps slowly while grounded.
      this.groundY = y;
      if (this.phase === 'landing' && this.t >= LAND_SECONDS) this.set('ground');
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
