// A coarse map of the ceilings in an area's world-space art (Waking Hollow's cave), so the
// camera stays under the rock. The simulation's colliders have no ceilings, so the camera's
// collision casts let it rise through a low roof at steep angles. Render-only: built from
// the art meshes once they load, and read by `CameraRig`.
//
// Every triangle that faces down is a ceiling. Each one is written into the grid cells its
// ground-plane footprint touches, padded by `PAD_M`, at its lowest corner's height. A query
// returns the lowest ceiling in a cell above a given height.

import { Matrix3, Matrix4, Vector3, type BufferGeometry } from 'three';

/** Grid cell size in meters. */
const CELL_M = 0.5;
/** Each triangle also covers this much around its footprint, for the camera's own size. */
const PAD_M = 0.35;
/** A triangle counts as a ceiling when its normal points down at least this much. */
const DOWN = -0.35;
/** Heights are stored rounded down to this step, which keeps the per-cell lists short. */
const STEP_M = 0.05;
/** Cells hold integer keys built from these offsets; areas span far less than this. */
const OFFSET = 4096;
const SPAN = 8192;

export class CeilingMap {
  /** Cell key to the ceiling heights over it, in steps, sorted ascending. */
  private cells = new Map<number, number[]>();

  get size(): number {
    return this.cells.size;
  }

  /** Adds the down-facing triangles of `geometry`, placed in the world by `matrix`. */
  add(geometry: BufferGeometry, matrix: Matrix4): void {
    const pos = geometry.getAttribute('position');
    if (!pos) return;
    const nrm = geometry.getAttribute('normal');
    const index = geometry.getIndex();
    const normalMatrix = new Matrix3().getNormalMatrix(matrix);
    const count = index ? index.count : pos.count;
    const a = new Vector3();
    const b = new Vector3();
    const c = new Vector3();
    const n = new Vector3();
    const t = new Vector3();
    for (let i = 0; i + 2 < count; i += 3) {
      const ia = index ? index.getX(i) : i;
      const ib = index ? index.getX(i + 1) : i + 1;
      const ic = index ? index.getX(i + 2) : i + 2;
      // Which way a triangle faces comes from its vertex normals when the file has them,
      // since the winding of a flattened, mirrored node can be either way round.
      if (nrm) {
        n.fromBufferAttribute(nrm, ia).add(t.fromBufferAttribute(nrm, ib)).add(t.fromBufferAttribute(nrm, ic));
        n.applyMatrix3(normalMatrix);
      }
      a.fromBufferAttribute(pos, ia).applyMatrix4(matrix);
      b.fromBufferAttribute(pos, ib).applyMatrix4(matrix);
      c.fromBufferAttribute(pos, ic).applyMatrix4(matrix);
      if (!nrm) n.subVectors(c, b).cross(t.subVectors(a, b));
      const len = n.length();
      if (len < 1e-9 || n.y / len > DOWN) continue;
      const h = Math.floor(Math.min(a.y, b.y, c.y) / STEP_M);
      const x0 = cellOf(Math.min(a.x, b.x, c.x) - PAD_M);
      const x1 = cellOf(Math.max(a.x, b.x, c.x) + PAD_M);
      const z0 = cellOf(Math.min(a.z, b.z, c.z) - PAD_M);
      const z1 = cellOf(Math.max(a.z, b.z, c.z) + PAD_M);
      for (let x = x0; x <= x1; x++) for (let z = z0; z <= z1; z++) this.insert(key(x, z), h);
    }
  }

  /** The lowest ceiling over (x, z) that is higher than `above`, or Infinity if none is. */
  ceilingAt(x: number, z: number, above: number): number {
    const list = this.cells.get(key(cellOf(x), cellOf(z)));
    if (!list) return Infinity;
    const min = above / STEP_M;
    // The lists are short (a few heights per cell), so a linear scan beats a search.
    for (const h of list) if (h > min) return h * STEP_M;
    return Infinity;
  }

  private insert(k: number, h: number): void {
    const list = this.cells.get(k);
    if (!list) {
      this.cells.set(k, [h]);
      return;
    }
    let i = 0;
    while (i < list.length && list[i] < h) i++;
    if (list[i] !== h) list.splice(i, 0, h);
  }
}

function cellOf(v: number): number {
  return Math.floor(v / CELL_M);
}

function key(x: number, z: number): number {
  return (x + OFFSET) * SPAN + (z + OFFSET);
}
