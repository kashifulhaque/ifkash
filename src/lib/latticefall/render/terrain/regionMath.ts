// Region lookup over the manifest's polygons. Pure math, no Three.js.

import type { Region } from '../kits/manifest';

/** Even-odd test of a point against a polygon. */
export function pointInPolygon(x: number, z: number, pts: ReadonlyArray<readonly [number, number]>): boolean {
  let inside = false;
  for (let i = 0, j = pts.length - 1; i < pts.length; j = i++) {
    const [xi, zi] = pts[i];
    const [xj, zj] = pts[j];
    if (zi > z !== zj > z && x < ((xj - xi) * (z - zi)) / (zj - zi) + xi) inside = !inside;
  }
  return inside;
}

/** The first region containing the point, or null when it is in none. */
export function regionAt(regions: readonly Region[], x: number, z: number): Region | null {
  for (const r of regions) if (pointInPolygon(x, z, r.points)) return r;
  return null;
}
