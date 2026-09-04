// Circular footprints on the sphere. Prop builders register one footprint per
// solid object; the Game and the critters push their position out of any
// footprint they step into, which slides them along the obstacle.

import * as THREE from 'three';
import { PLANET_RADIUS } from './planet';

/** A solid disc on the surface: unit direction plus radius in surface units. */
export type Collider = { dir: THREE.Vector3; radius: number };

/** Side of one grid cube in unit-sphere space, about 4 surface units. */
const CELL = 0.1;
/** Largest agent radius that can query the grid, in surface units. */
const MAX_AGENT_RADIUS = 0.8;
const PASSES = 3;

const _t = new THREE.Vector3();
const _ref = new THREE.Vector3();

/**
 * Colliders bucketed by a cubic grid over the unit sphere. Each collider is
 * inserted into every cell its inflated footprint touches, so a query only
 * reads the single cell that contains the query direction.
 */
export class ColliderGrid {
  private cells = new Map<number, Collider[]>();
  readonly count: number;

  constructor(colliders: Collider[]) {
    for (const c of colliders) this.insert(c);
    this.count = colliders.length;
  }

  private static key(ix: number, iy: number, iz: number): number {
    return ((ix + 32) * 64 + (iy + 32)) * 64 + (iz + 32);
  }

  private insert(c: Collider): void {
    const reach = (c.radius + MAX_AGENT_RADIUS) / PLANET_RADIUS;
    const d = c.dir;
    const x0 = Math.floor((d.x - reach) / CELL);
    const x1 = Math.floor((d.x + reach) / CELL);
    const y0 = Math.floor((d.y - reach) / CELL);
    const y1 = Math.floor((d.y + reach) / CELL);
    const z0 = Math.floor((d.z - reach) / CELL);
    const z1 = Math.floor((d.z + reach) / CELL);
    for (let ix = x0; ix <= x1; ix++) {
      for (let iy = y0; iy <= y1; iy++) {
        for (let iz = z0; iz <= z1; iz++) {
          const k = ColliderGrid.key(ix, iy, iz);
          const list = this.cells.get(k);
          if (list) list.push(c);
          else this.cells.set(k, [c]);
        }
      }
    }
  }

  private cellAt(d: THREE.Vector3): Collider[] | undefined {
    return this.cells.get(ColliderGrid.key(Math.floor(d.x / CELL), Math.floor(d.y / CELL), Math.floor(d.z / CELL)));
  }

  /** True if a disc of `radius` at `dir` overlaps any collider. */
  blocked(dir: THREE.Vector3, radius: number): boolean {
    const list = this.cellAt(dir);
    if (!list) return false;
    for (const c of list) {
      if (dir.angleTo(c.dir) * PLANET_RADIUS < c.radius + radius) return true;
    }
    return false;
  }

  /**
   * Push `dir` (in place) out of any collider it overlaps, moving along the
   * tangent away from the collider centre. Returns true if `dir` changed.
   */
  resolve(dir: THREE.Vector3, radius: number): boolean {
    let changed = false;
    for (let pass = 0; pass < PASSES; pass++) {
      const list = this.cellAt(dir);
      if (!list) break;
      let moved = false;
      for (const c of list) {
        const cos = THREE.MathUtils.clamp(dir.dot(c.dir), -1, 1);
        _t.copy(dir).addScaledVector(c.dir, -cos);
        const sin = _t.length();
        const ang = Math.atan2(sin, cos);
        const minAng = (c.radius + radius) / PLANET_RADIUS;
        if (ang >= minAng) continue;
        if (sin < 1e-8) {
          // Standing exactly on the centre: pick any tangent.
          _ref.set(0, 1, 0);
          if (Math.abs(c.dir.y) > 0.98) _ref.set(1, 0, 0);
          _t.crossVectors(_ref, c.dir);
        }
        _t.normalize();
        const push = minAng - ang + 1e-5;
        dir.multiplyScalar(Math.cos(push)).addScaledVector(_t, Math.sin(push)).normalize();
        moved = true;
      }
      if (!moved) break;
      changed = true;
    }
    return changed;
  }
}
