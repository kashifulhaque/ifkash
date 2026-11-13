// Distance culling for the instances of a `BatchedMesh` (WP-081). The batch already culls per
// instance against the camera's frustum; this adds the draw-distance cut and the grass-density
// thinning, by flipping each instance's visibility. It only touches instances whose answer
// changed, and only when the focus has moved enough to change any answer.

import type { BatchedMesh } from 'three';

export interface CullEntry {
  /** The instance id the batch returned from `addInstance`. */
  id: number;
  x: number;
  z: number;
  /**
   * A fixed number in [0, 1). The instance draws only while the thinning fraction is above it,
   * so lowering the fraction hides an even, repeatable share of the field. Negative never thins.
   */
  rank: number;
  /** The instance's own radius in meters: it stays until its nearest part is out of range. */
  pad?: number;
}

/** Meters of overlap between "show" and "hide", so an instance on the edge doesn't flicker. */
const HYSTERESIS_M = 6;
/** The focus must move this far (meters) before every instance is retested. */
const MOVE_M = 1.5;

export class DistanceCuller {
  private visible: Uint8Array;
  private fx = Infinity;
  private fz = Infinity;
  private radius = -1;
  private keep = -1;
  /** How many instances are visible now. */
  shown = 0;

  constructor(
    private mesh: BatchedMesh,
    private entries: readonly CullEntry[]
  ) {
    this.visible = new Uint8Array(entries.length).fill(1);
    this.shown = entries.length;
  }

  get count(): number {
    return this.entries.length;
  }

  /** Retests every instance against `radius` around (fx, fz) and the `keep` fraction in [0, 1]. */
  update(fx: number, fz: number, radius: number, keep = 1): void {
    const moved = Math.hypot(fx - this.fx, fz - this.fz);
    if (moved < MOVE_M && radius === this.radius && keep === this.keep) return;
    this.fx = fx;
    this.fz = fz;
    this.radius = radius;
    this.keep = keep;
    let shown = 0;
    for (let i = 0; i < this.entries.length; i++) {
      const e = this.entries[i];
      const dx = e.x - fx;
      const dz = e.z - fz;
      const d2 = dx * dx + dz * dz;
      const was = this.visible[i] === 1;
      const reach = radius + (e.pad ?? 0) + (was ? HYSTERESIS_M : 0);
      let now = d2 <= reach * reach;
      if (now && e.rank >= keep) now = false;
      if (now) shown++;
      if (now !== was) {
        this.visible[i] = now ? 1 : 0;
        this.mesh.setVisibleAt(e.id, now);
      }
    }
    this.shown = shown;
  }
}
