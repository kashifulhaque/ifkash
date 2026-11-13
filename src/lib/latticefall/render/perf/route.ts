// The benchmark's fixed camera path: where the "player" walks through an area so that every
// named region, the spawn, and every occupied activity cell is seen. Pure math, no Three.js,
// and no randomness: the same manifest always gives the same route.

import type { Heightfield, LevelManifest, Region } from '../kits/manifest';
import { pointInPolygon, regionAt } from '../terrain/regionMath';

export interface Waypoint {
  x: number;
  z: number;
  /** Region name, `cell X,Z`, or `spawn`. */
  label: string;
}

/** Ground height at (x, z), or null off the heightfield. */
export function heightAt(hf: Heightfield | null, x: number, z: number): number | null {
  if (!hf) return null;
  const x0 = hf.center[0] - hf.size_x / 2;
  const z0 = hf.center[2] - hf.size_z / 2;
  const fc = ((x - x0) / hf.size_x) * (hf.cols - 1);
  const fr = ((z - z0) / hf.size_z) * (hf.rows - 1);
  if (fc < 0 || fr < 0 || fc > hf.cols - 1 || fr > hf.rows - 1) return null;
  const c = Math.min(hf.cols - 2, Math.floor(fc));
  const r = Math.min(hf.rows - 2, Math.floor(fr));
  const tc = fc - c;
  const tr = fr - r;
  const h = (rr: number, cc: number): number => hf.heights[rr * hf.cols + cc];
  const top = h(r, c) * (1 - tc) + h(r, c + 1) * tc;
  const bot = h(r + 1, c) * (1 - tc) + h(r + 1, c + 1) * tc;
  return hf.center[1] + top * (1 - tr) + bot * tr;
}

/** A point inside `region` close to its centroid (regions can be concave). */
function interiorPoint(region: Region): [number, number] {
  let cx = 0;
  let cz = 0;
  for (const [x, z] of region.points) {
    cx += x;
    cz += z;
  }
  cx /= region.points.length;
  cz /= region.points.length;
  if (pointInPolygon(cx, cz, region.points)) return [cx, cz];
  let best: [number, number] = [region.points[0][0], region.points[0][1]];
  let bestD = Infinity;
  const xs = region.points.map((p) => p[0]);
  const zs = region.points.map((p) => p[1]);
  const x0 = Math.min(...xs);
  const x1 = Math.max(...xs);
  const z0 = Math.min(...zs);
  const z1 = Math.max(...zs);
  for (let i = 0; i <= 24; i++) {
    for (let j = 0; j <= 24; j++) {
      const x = x0 + ((x1 - x0) * i) / 24;
      const z = z0 + ((z1 - z0) * j) / 24;
      if (!pointInPolygon(x, z, region.points)) continue;
      const d = (x - cx) ** 2 + (z - cz) ** 2;
      if (d < bestD) {
        bestD = d;
        best = [x, z];
      }
    }
  }
  return best;
}

const MAX_WAYPOINTS = 28;

/** The ordered waypoints the player visits: the spawn first, then nearest-neighbor order. */
export function buildWaypoints(m: LevelManifest): Waypoint[] {
  const a = m.area;
  const found: Waypoint[] = [];
  const spawn: Waypoint = { x: m.spawn?.pos[0] ?? 0, z: m.spawn?.pos[2] ?? 0, label: 'spawn' };

  for (const r of a.regions) {
    const [x, z] = interiorPoint(r);
    found.push({ x, z, label: r.name });
  }

  // Occupied activity cells: where the kit's pieces and the scatter stand.
  const size = a.cells.size_m;
  const cells = new Map<string, { x: number; z: number; n: number }>();
  const add = (x: number, z: number, w: number): void => {
    const key = `${Math.floor(x / size)},${Math.floor(z / size)}`;
    const c = cells.get(key) ?? { x: 0, z: 0, n: 0 };
    c.x += x * w;
    c.z += z * w;
    c.n += w;
    cells.set(key, c);
  };
  for (const i of m.instances) add(i.pos[0], i.pos[2], 1);
  for (const b of m.scatter) for (const t of b.transforms) add(t[0], t[2], 0.1);
  const ranked = [...cells.entries()].sort((p, q) => q[1].n - p[1].n || (p[0] < q[0] ? -1 : 1));
  for (const [key, c] of ranked) {
    if (found.length + 1 >= MAX_WAYPOINTS) break;
    const x = c.x / c.n;
    const z = c.z / c.n;
    // Skip a cell whose centroid is close to a waypoint already chosen.
    if (found.some((w) => Math.hypot(w.x - x, w.z - z) < size * 0.6)) continue;
    found.push({ x, z, label: regionAt(a.regions, x, z)?.name ?? `cell ${key}` });
  }

  // Terrain with nothing else on it: walk a ring at 60% of the bounds so the edges are seen.
  if (m.terrain) {
    const [bx0, bz0] = a.bounds.min;
    const [bx1, bz1] = a.bounds.max;
    const cx = (bx0 + bx1) / 2;
    const cz = (bz0 + bz1) / 2;
    for (const [sx, sz] of [
      [-1, -1],
      [1, -1],
      [1, 1],
      [-1, 1]
    ] as const) {
      const x = cx + ((sx * (bx1 - bx0)) / 2) * 0.6;
      const z = cz + ((sz * (bz1 - bz0)) / 2) * 0.6;
      if (found.length + 1 >= MAX_WAYPOINTS) break;
      if (heightAt(m.terrain, x, z) === null) continue;
      if (found.some((w) => Math.hypot(w.x - x, w.z - z) < size * 0.6)) continue;
      found.push({ x, z, label: regionAt(a.regions, x, z)?.name ?? 'edge' });
    }
  }

  const out: Waypoint[] = [spawn];
  const rest = [...found];
  while (rest.length) {
    const last = out[out.length - 1];
    let bi = 0;
    let bd = Infinity;
    rest.forEach((w, i) => {
      const d = Math.hypot(w.x - last.x, w.z - last.z);
      if (d < bd) {
        bd = d;
        bi = i;
      }
    });
    out.push(rest.splice(bi, 1)[0]);
  }
  return out;
}

export interface RouteFrame {
  /** The player's feet. */
  x: number;
  y: number;
  z: number;
  /** Camera yaw in radians: 0 looks toward +Z, turning toward +X (the dev camera's convention). */
  yaw: number;
  /** The region name the player stands in, or the waypoint label when in none. */
  region: string;
}

/**
 * Expands waypoints into one frame per step: the player walks `stepM` meters per frame between
 * waypoints, and at each one the camera sweeps a full turn over `sweepFrames` frames.
 */
export function buildFrames(m: LevelManifest, waypoints: readonly Waypoint[], stepM = 1.5, sweepFrames = 16): RouteFrame[] {
  const floor = m.spawn?.pos[1] ?? 0;
  const ground = (x: number, z: number): number => heightAt(m.terrain, x, z) ?? floor;
  const where = (x: number, z: number, fallback: string): string => regionAt(m.area.regions, x, z)?.name ?? fallback;
  const frames: RouteFrame[] = [];
  const push = (x: number, z: number, yaw: number, fallback: string): void => {
    frames.push({ x, y: ground(x, z), z, yaw, region: where(x, z, fallback) });
  };
  for (let i = 0; i < waypoints.length; i++) {
    const w = waypoints[i];
    const next = waypoints[i + 1];
    const heading = next ? Math.atan2(next.x - w.x, next.z - w.z) : 0;
    // The sweep at the waypoint, starting on the heading so the walk continues smoothly.
    for (let s = 0; s < sweepFrames; s++) push(w.x, w.z, heading + (s / sweepFrames) * Math.PI * 2, w.label);
    if (!next) break;
    const len = Math.hypot(next.x - w.x, next.z - w.z);
    const steps = Math.max(1, Math.round(len / stepM));
    for (let s = 1; s < steps; s++) {
      const t = s / steps;
      push(w.x + (next.x - w.x) * t, w.z + (next.z - w.z) * t, heading, w.label);
    }
  }
  return frames;
}
