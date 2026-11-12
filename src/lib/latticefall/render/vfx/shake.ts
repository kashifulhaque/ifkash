// Camera shake: a trauma value that decays, drives a smooth offset and a small roll, and is
// refused entirely under reduced motion. Presentation only: it moves the draw camera after
// the camera rig has run, so the rig's smoothing and the simulation never see it.

import type { Camera } from 'three';

/** Largest position offset in meters at trauma 1. */
const MAX_OFFSET = 0.14;
/** Largest roll in radians at trauma 1 (about 1.7 degrees). */
const MAX_ROLL = 0.03;
/** Trauma lost per second. */
const DECAY = 1.8;

export interface ShakeOffset {
  x: number;
  y: number;
  roll: number;
}

export class CameraShake {
  trauma = 0;
  private t = 0;
  private reduced = false;
  private query: MediaQueryList | null = null;
  /** The offset applied by the last `apply`, for the dev readout. */
  readonly last: ShakeOffset = { x: 0, y: 0, roll: 0 };

  constructor() {
    if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
      this.query = window.matchMedia('(prefers-reduced-motion: reduce)');
      this.reduced = this.query.matches;
      this.query.addEventListener('change', this.onChange);
    }
  }

  private onChange = (e: MediaQueryListEvent): void => this.setReducedMotion(e.matches);

  /** True when shake is refused. */
  get reducedMotion(): boolean {
    return this.reduced;
  }

  /** Called by the media query listener; also a dev hook to test the reduced-motion path. */
  setReducedMotion(on: boolean): void {
    this.reduced = on;
    if (on) this.trauma = 0;
  }

  /** Adds trauma (clamped to 1). Ignored under reduced motion. */
  add(amount: number): void {
    if (this.reduced || !(amount > 0)) return;
    this.trauma = Math.min(1, this.trauma + amount);
  }

  /** Advances the shake clock. Pass 0 to hold it (hitstop, frozen). */
  update(dt: number): void {
    if (dt <= 0) return;
    this.t += dt;
    this.trauma = Math.max(0, this.trauma - DECAY * dt);
  }

  /** The offset for the current trauma. Zero when there is none or reduced motion is on. */
  offset(out: ShakeOffset = this.last): ShakeOffset {
    const s = this.reduced ? 0 : this.trauma * this.trauma;
    // Sums of incommensurate sines: smooth, never repeats audibly, needs no random state.
    const t = this.t;
    out.x = s * MAX_OFFSET * (Math.sin(t * 61.3) * 0.6 + Math.sin(t * 37.1 + 1.3) * 0.4);
    out.y = s * MAX_OFFSET * (Math.sin(t * 53.7 + 2.1) * 0.6 + Math.sin(t * 29.9 + 0.4) * 0.4);
    out.roll = s * MAX_ROLL * Math.sin(t * 43.1 + 0.7);
    return out;
  }

  /** Shifts `camera` by the current offset, in the camera's own axes. Call right before drawing. */
  apply(camera: Camera): void {
    const o = this.offset();
    if (o.x === 0 && o.y === 0 && o.roll === 0) return;
    camera.translateX(o.x);
    camera.translateY(o.y);
    camera.rotateZ(o.roll);
    camera.updateMatrixWorld();
  }

  dispose(): void {
    this.query?.removeEventListener('change', this.onChange);
  }
}
