// World-space terrain: rolling hills plus deterministic villages and the dirt
// roads that connect them. Every function here is pure in (x, z) so chunk
// seams always match and revisited areas regenerate identically.
import { chunkRng } from './rng';

export const REGION = 120; // 5 chunks per region side

export type Village = { x: number; z: number; radius: number; height: number };
export type RoadHit = { dist: number; height: number };
export type HouseSlot = {
  x: number;
  z: number;
  rot: number; // faces the village center
  width: number;
  depth: number;
  height: number;
  variant: number; // wall/roof palette pick
};
export type PropSlot = { x: number; z: number; rot: number; modelIndex: number; height: number };
export type VillagePlan = { village: Village; houses: HouseSlot[]; props: PropSlot[] };

const ROAD_FULL = 2.2; // full road width from centerline
const ROAD_FADE = 4.5; // blended back into grass by here

function smoothstep(e0: number, e1: number, x: number): number {
  const t = Math.min(1, Math.max(0, (x - e0) / (e1 - e0)));
  return t * t * (3 - 2 * t);
}

function latticeHash(ix: number, iz: number, seed: number): number {
  let h = seed ^ Math.imul(ix, 374761393) ^ Math.imul(iz, 668265263);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}

// Value noise: hashed lattice corners, smoothstep bilinear blend. Returns 0..1.
function valueNoise(x: number, z: number, seed: number): number {
  const ix = Math.floor(x);
  const iz = Math.floor(z);
  const fx = x - ix;
  const fz = z - iz;
  const sx = fx * fx * (3 - 2 * fx);
  const sz = fz * fz * (3 - 2 * fz);
  const a = latticeHash(ix, iz, seed);
  const b = latticeHash(ix + 1, iz, seed);
  const c = latticeHash(ix, iz + 1, seed);
  const d = latticeHash(ix + 1, iz + 1, seed);
  const top = a + (b - a) * sx;
  const bot = c + (d - c) * sx;
  return top + (bot - top) * sz;
}

// Rolling hills plus ridged mountains gated by a low-frequency "continent"
// mask, so the world alternates between gentle lowlands, valleys and proper
// mountain ranges. Pure in (x, z) — chunk seams always match.
export function baseHeight(x: number, z: number): number {
  const broad = (valueNoise(x / 60, z / 60, 101) - 0.5) * 4.5;
  const detail = (valueNoise(x / 17, z / 17, 202) - 0.5) * 1.2;
  const micro = Math.sin(x * 0.7) * Math.cos(z * 0.6) * 0.05;
  // Ridged noise → sharp crests; only expressed where the continent mask is high
  const continent = valueNoise(x / 220, z / 220, 303);
  const ridge = 1 - Math.abs(2 * valueNoise(x / 90, z / 90, 404) - 1);
  const mountains = ridge * ridge * smoothstep(0.55, 0.85, continent) * 16;
  // Low-continent zones dip into shallow valleys
  const valleys = smoothstep(0.45, 0.2, continent) * 3;
  return broad + detail + micro + mountains - valleys;
}

// Max height difference across a disc — used to keep villages and NPC spawns
// off mountain faces.
export function localRelief(x: number, z: number, r: number): number {
  const c = baseHeight(x, z);
  let min = c;
  let max = c;
  for (let i = 0; i < 4; i++) {
    const a = (i / 4) * Math.PI * 2;
    const h = baseHeight(x + Math.cos(a) * r, z + Math.sin(a) * r);
    if (h < min) min = h;
    if (h > max) max = h;
  }
  return max - min;
}

// ── Villages ──────────────────────────────────────────────────────────────

const villageCache = new Map<string, Village | null>();

export function villageAt(rx: number, rz: number): Village | null {
  const key = `${rx},${rz}`;
  const cached = villageCache.get(key);
  if (cached !== undefined) return cached;
  const rng = chunkRng(rx, rz, 9001);
  let v: Village | null = null;
  if (rng() < 0.45) {
    // Try a few candidate centers and keep one on reasonably flat ground, so
    // the plateau flatten in terrainHeight never carves cliffs into mountains.
    for (let tries = 0; tries < 3 && !v; tries++) {
      const x = (rx + 0.5) * REGION + (rng() - 0.5) * 0.5 * REGION;
      const z = (rz + 0.5) * REGION + (rng() - 0.5) * 0.5 * REGION;
      const radius = 13 + rng() * 4;
      if (localRelief(x, z, radius) < 2.5) v = { x, z, radius, height: baseHeight(x, z) };
    }
  }
  villageCache.set(key, v);
  return v;
}

const villagesNearCache = new Map<string, Village[]>();

export function villagesNear(x: number, z: number): Village[] {
  const rx = Math.floor(x / REGION);
  const rz = Math.floor(z / REGION);
  const key = `${rx},${rz}`;
  let list = villagesNearCache.get(key);
  if (!list) {
    list = [];
    for (let dx = -1; dx <= 1; dx++) {
      for (let dz = -1; dz <= 1; dz++) {
        const v = villageAt(rx + dx, rz + dz);
        if (v) list.push(v);
      }
    }
    villagesNearCache.set(key, list);
  }
  return list;
}

export function villageContaining(x: number, z: number): Village | null {
  for (const v of villagesNear(x, z)) {
    if (Math.hypot(x - v.x, z - v.z) < v.radius) return v;
  }
  return null;
}

// ── Roads ─────────────────────────────────────────────────────────────────

type Segment = { ax: number; az: number; bx: number; bz: number; ah: number; bh: number };

const segmentCache = new Map<string, Segment[]>();

// Straight road segments between villages in adjacent regions. Both sides of
// a border compute the identical segment, so roads are cross-chunk coherent.
function segmentsNear(x: number, z: number): Segment[] {
  const rx = Math.floor(x / REGION);
  const rz = Math.floor(z / REGION);
  const key = `${rx},${rz}`;
  let segs = segmentCache.get(key);
  if (!segs) {
    segs = [];
    for (let ax = rx - 2; ax <= rx + 1; ax++) {
      for (let az = rz - 2; az <= rz + 1; az++) {
        const a = villageAt(ax, az);
        if (!a) continue;
        const east = villageAt(ax + 1, az);
        if (east) segs.push({ ax: a.x, az: a.z, bx: east.x, bz: east.z, ah: a.height, bh: east.height });
        const south = villageAt(ax, az + 1);
        if (south) segs.push({ ax: a.x, az: a.z, bx: south.x, bz: south.z, ah: a.height, bh: south.height });
      }
    }
    segmentCache.set(key, segs);
  }
  return segs;
}

export function nearestRoad(x: number, z: number): RoadHit | null {
  let best: RoadHit | null = null;
  for (const s of segmentsNear(x, z)) {
    const dx = s.bx - s.ax;
    const dz = s.bz - s.az;
    const len2 = dx * dx + dz * dz;
    let t = len2 > 0 ? ((x - s.ax) * dx + (z - s.az) * dz) / len2 : 0;
    t = Math.min(1, Math.max(0, t));
    const px = s.ax + dx * t;
    const pz = s.az + dz * t;
    const dist = Math.hypot(x - px, z - pz);
    if (!best || dist < best.dist) best = { dist, height: s.ah + (s.bh - s.ah) * t };
  }
  return best;
}

// 0 off-road → 1 on the road surface; used for coloring and spawn filtering.
export function roadFactor(x: number, z: number): number {
  const r = nearestRoad(x, z);
  return r ? 1 - smoothstep(ROAD_FULL, ROAD_FADE, r.dist) : 0;
}

// ── Combined height ───────────────────────────────────────────────────────

export function terrainHeight(x: number, z: number): number {
  let h = baseHeight(x, z);
  // Flatten toward village plateaus
  let vw = 0;
  let vh = 0;
  for (const v of villagesNear(x, z)) {
    const d = Math.hypot(x - v.x, z - v.z);
    const w = 1 - smoothstep(v.radius * 0.7, v.radius * 1.6, d); // wide skirt: no cliff ring
    if (w > vw) {
      vw = w;
      vh = v.height;
    }
  }
  if (vw > 0) h += (vh - h) * vw;
  // Flatten toward the road profile
  const r = nearestRoad(x, z);
  if (r) {
    const w = 1 - smoothstep(ROAD_FULL, ROAD_FADE, r.dist);
    if (w > 0) h += (r.height - h) * w;
  }
  return h;
}

// ── Village plan (houses + props) ─────────────────────────────────────────
// Generated from region RNG only — never per-chunk RNG — so every chunk that
// overlaps the village derives the exact same plan and instantiates just the
// slots inside its own bounds.

const planCache = new Map<string, VillagePlan | null>();

export function villagePlan(rx: number, rz: number): VillagePlan | null {
  const key = `${rx},${rz}`;
  const cached = planCache.get(key);
  if (cached !== undefined) return cached;
  const village = villageAt(rx, rz);
  let plan: VillagePlan | null = null;
  if (village) {
    const rng = chunkRng(rx, rz, 7777);
    const houses: HouseSlot[] = [];
    const nHouses = 6 + Math.floor(rng() * 4);
    for (let i = 0; i < nHouses; i++) {
      const angle = (i / nHouses) * Math.PI * 2 + (rng() - 0.5) * 0.5;
      const dist = village.radius * (0.5 + rng() * 0.25);
      const x = village.x + Math.cos(angle) * dist;
      const z = village.z + Math.sin(angle) * dist;
      houses.push({
        x,
        z,
        rot: Math.atan2(village.x - x, village.z - z),
        width: 3.2 + rng() * 1.6,
        depth: 3.0 + rng() * 1.4,
        height: 2.2 + rng() * 0.7,
        variant: Math.floor(rng() * 4)
      });
    }
    const props: PropSlot[] = [];
    const nProps = 3 + Math.floor(rng() * 4);
    for (let i = 0; i < nProps; i++) {
      const angle = rng() * Math.PI * 2;
      const dist = village.radius * 0.3 * rng();
      props.push({
        x: village.x + Math.cos(angle) * dist,
        z: village.z + Math.sin(angle) * dist,
        rot: rng() * Math.PI * 2,
        modelIndex: Math.floor(rng() * 8),
        height: 1.0 + rng() * 0.5
      });
    }
    plan = { village, houses, props };
  }
  planCache.set(key, plan);
  return plan;
}
