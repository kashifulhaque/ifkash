// Pool water (WP-042, WP-060): the surface of every `Pool` in a level manifest, drawn as a
// translucent plane that follows the core's water level. Read-only: nothing here writes
// simulation state.
//
// The core's state word for a pool is its depth in meters (it changes when a lever raises the
// pool or the Belfry's sluice lowers it). The surface is `floor_y_m + depth`: a pool on flat
// ground has `floor_y_m = 0`, so its surface is `depth` above the ground, and a lake in a basin
// has its floor below the shore, so its surface sits level with the shore. When the depth
// changes, the surface eases to the new level over a few seconds, so lowering the lake is a
// slow, visible drain rather than a pop.
//
// Mirrormere's lake is deep and still: a darker blue, nearly opaque, with a faint shimmer. The
// shallows (village, backwater) are lighter and clearer; Blight water is green and pulses.

import { Color, DoubleSide, Group, Mesh, MeshStandardMaterial, PlaneGeometry, type BufferGeometry, type Material } from 'three';

/** The fields of a manifest `Pool` the renderer reads. */
export interface PoolData {
  name?: string;
  center: [number, number];
  half_extents: [number, number];
  depth_m: number;
  blight?: boolean;
  floor_y_m?: number;
}

/** Deeper than this reads as a lake: a darker, denser water (matches the core's `water_deep_over_m`). */
const DEEP_M = 1.4;
/** Time constant for the surface easing toward the core's level, in milliseconds. */
const LEVEL_EASE_MS = 1200;

export interface PoolWater {
  group: Group;
  geometries: BufferGeometry[];
  materials: Material[];
  /** Moves the surface toward the level for `depth`. `clockMs` drives the shimmer. */
  update(depth: number, frameMs: number, clockMs: number): void;
}

/** The surface height, in meters, of a pool at `depth`. */
export function surfaceY(d: Pick<PoolData, 'floor_y_m'>, depth: number): number {
  return (d.floor_y_m ?? 0) + depth;
}

/** The base color of a pool's water by kind: Blight, clear shallows, or a lake. */
function waterColor(d: PoolData, clear: Color, blight: Color): { color: Color; opacity: number } {
  if (d.blight) return { color: blight.clone(), opacity: 0.62 };
  if (d.depth_m > DEEP_M) return { color: clear.clone().multiplyScalar(0.55), opacity: 0.86 };
  return { color: clear.clone(), opacity: 0.5 };
}

/**
 * Builds a pool's water. `clear` and `blight` are the site's water tokens (see
 * `PropsRenderer`'s `token`); the returned object owns its geometry and materials, which the
 * caller disposes.
 */
export function createPoolWater(d: PoolData, clear: Color, blight: Color): PoolWater {
  const [hx, hz] = d.half_extents;
  const [x, z] = d.center;
  const base = waterColor(d, clear, blight);
  const geometry = new PlaneGeometry(hx * 2, hz * 2);
  const material = new MeshStandardMaterial({
    color: base.color,
    roughness: 0.18,
    metalness: 0.1,
    transparent: true,
    opacity: base.opacity,
    side: DoubleSide,
    emissive: d.blight ? new Color('#3d5a1c') : new Color('#000000'),
    emissiveIntensity: d.blight ? 0.25 : 0
  });
  const plane = new Mesh(geometry, material);
  plane.rotation.x = -Math.PI / 2;
  plane.position.set(x, 0, z);
  const group = new Group();
  group.add(plane);
  let y = surfaceY(d, d.depth_m);
  group.position.y = y;
  return {
    group,
    geometries: [geometry],
    materials: [material],
    update(depth, frameMs, clockMs) {
      const goal = surfaceY(d, depth);
      y += (goal - y) * (1 - Math.exp(-frameMs / LEVEL_EASE_MS));
      group.position.y = y;
      // A slow shimmer, a little stronger on shallow water, and a Blight pulse.
      const shimmer = Math.sin(clockMs / 1700 + x * 0.05 + z * 0.03) * 0.025;
      material.opacity = Math.min(0.95, base.opacity + shimmer);
      if (d.blight) material.emissiveIntensity = 0.25 + Math.sin(clockMs / 700) * 0.1;
    }
  };
}
