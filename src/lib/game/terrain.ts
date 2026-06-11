// World-space terrain: rolling hills plus deterministic settlements, the dirt
// roads that connect them, and rivers carved between regions. Every function
// here is pure in (x, z) so chunk seams always match and revisited areas
// regenerate identically.
import { chunkRng } from './rng';

export const REGION = 120; // 5 chunks per region side

export type SettlementTier = 'hamlet' | 'village' | 'town';
export type Village = {
  x: number;
  z: number;
  radius: number;
  height: number;
  tier: SettlementTier;
};
export type RoadHit = { dist: number; height: number; width: number };
export type HouseSlot = {
  x: number;
  z: number;
  rot: number; // faces the village center
  width: number;
  depth: number;
  height: number;
  variant: number; // wall/roof palette pick
  stories: number; // 1 = cottage, 2 = stacked town building
};
export type PropSlot = { x: number; z: number; rot: number; modelIndex: number; height: number };
export type CarSlot = { x: number; z: number; rot: number; key: string };
export type VillagePlan = {
  village: Village;
  houses: HouseSlot[];
  props: PropSlot[];
  cars: CarSlot[];
};

const ROAD_FADE_BAND = 2.3; // blended back into grass over this distance
const RIVER_FULL = 2.5; // riverbed half-width at full depth
const RIVER_FADE = 6; // banks blended back to terrain by here
const RIVER_DEPTH = 1.4;

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

// Steepness of the base terrain (height units per world unit) — used to pick
// cliff-rock spots on mountain faces.
export function slopeAt(x: number, z: number): number {
  const d = 1.5;
  const gx = (baseHeight(x + d, z) - baseHeight(x - d, z)) / (2 * d);
  const gz = (baseHeight(x, z + d) - baseHeight(x, z - d)) / (2 * d);
  return Math.hypot(gx, gz);
}

// ── Villages ──────────────────────────────────────────────────────────────

const TIER_PARAMS: Record<
  SettlementTier,
  { rMin: number; rSpan: number; houses: [number, number]; props: [number, number]; cars: number }
> = {
  hamlet: { rMin: 9, rSpan: 2, houses: [3, 4], props: [1, 2], cars: 0 }, // car rolled at 50%
  village: { rMin: 13, rSpan: 4, houses: [6, 10], props: [3, 6], cars: 1 },
  town: { rMin: 20, rSpan: 6, houses: [12, 16], props: [6, 10], cars: 2 }
};

const villageCache = new Map<string, Village | null>();

export function villageAt(rx: number, rz: number): Village | null {
  const key = `${rx},${rz}`;
  const cached = villageCache.get(key);
  if (cached !== undefined) return cached;
  const rng = chunkRng(rx, rz, 9001);
  let v: Village | null = null;
  if (rng() < 0.45) {
    const tierRoll = rng();
    const tier: SettlementTier = tierRoll < 0.4 ? 'hamlet' : tierRoll < 0.8 ? 'village' : 'town';
    const p = TIER_PARAMS[tier];
    // Try a few candidate centers and keep one on reasonably flat ground, so
    // the plateau flatten in terrainHeight never carves cliffs into mountains.
    for (let tries = 0; tries < 3 && !v; tries++) {
      const x = (rx + 0.5) * REGION + (rng() - 0.5) * 0.5 * REGION;
      const z = (rz + 0.5) * REGION + (rng() - 0.5) * 0.5 * REGION;
      const radius = p.rMin + rng() * p.rSpan;
      if (localRelief(x, z, radius) < 2.5) v = { x, z, radius, height: baseHeight(x, z), tier };
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

// 0 away from settlements → 1 deep inside a plateau skirt. Shared by the
// height flatten and the river carve (rivers must not trench a plateau).
function villageWeight(x: number, z: number): { w: number; h: number } {
  let w = 0;
  let h = 0;
  for (const v of villagesNear(x, z)) {
    const d = Math.hypot(x - v.x, z - v.z);
    const wv = 1 - smoothstep(v.radius * 0.7, v.radius * 1.6, d); // wide skirt: no cliff ring
    if (wv > w) {
      w = wv;
      h = v.height;
    }
  }
  return { w, h };
}

// ── Roads ─────────────────────────────────────────────────────────────────

type Segment = {
  ax: number;
  az: number;
  bx: number;
  bz: number;
  ah: number;
  bh: number;
  width: number;
};

const segmentCache = new Map<string, Segment[]>();

// Straight road segments between villages in adjacent regions. Both sides of
// a border compute the identical segment, so roads are cross-chunk coherent.
// Roads that touch a town are wider.
function segmentsNear(x: number, z: number): Segment[] {
  const rx = Math.floor(x / REGION);
  const rz = Math.floor(z / REGION);
  const key = `${rx},${rz}`;
  let segs = segmentCache.get(key);
  if (!segs) {
    segs = [];
    const link = (a: Village, b: Village): Segment => ({
      ax: a.x,
      az: a.z,
      bx: b.x,
      bz: b.z,
      ah: a.height,
      bh: b.height,
      width: a.tier === 'town' || b.tier === 'town' ? 3.2 : 2.2
    });
    for (let ax = rx - 2; ax <= rx + 1; ax++) {
      for (let az = rz - 2; az <= rz + 1; az++) {
        const a = villageAt(ax, az);
        if (!a) continue;
        const east = villageAt(ax + 1, az);
        if (east) segs.push(link(a, east));
        const south = villageAt(ax, az + 1);
        if (south) segs.push(link(a, south));
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
    if (!best || dist - s.width < best.dist - best.width)
      best = { dist, height: s.ah + (s.bh - s.ah) * t, width: s.width };
  }
  return best;
}

// 0 off-road → 1 on the road surface; used for coloring and spawn filtering.
export function roadFactor(x: number, z: number): number {
  const r = nearestRoad(x, z);
  return r ? 1 - smoothstep(r.width, r.width + ROAD_FADE_BAND, r.dist) : 0;
}

// ── Rivers ────────────────────────────────────────────────────────────────
// Same region-graph idea as roads: a deterministic control point per region
// (skipping regions that hold a settlement), linked to neighbors. The carve is
// applied in terrainHeight so meshes, player physics and spawns all agree.

const riverPointCache = new Map<string, { x: number; z: number } | null>();

function riverAt(rx: number, rz: number): { x: number; z: number } | null {
  const key = `${rx},${rz}`;
  const cached = riverPointCache.get(key);
  if (cached !== undefined) return cached;
  const rng = chunkRng(rx, rz, 5555);
  let p: { x: number; z: number } | null = null;
  if (rng() < 0.3 && !villageAt(rx, rz)) {
    p = {
      x: (rx + 0.5) * REGION + (rng() - 0.5) * 0.7 * REGION,
      z: (rz + 0.5) * REGION + (rng() - 0.5) * 0.7 * REGION
    };
  }
  riverPointCache.set(key, p);
  return p;
}

type RiverSeg = { ax: number; az: number; bx: number; bz: number };

const riverSegCache = new Map<string, RiverSeg[]>();

function riverSegmentsNear(x: number, z: number): RiverSeg[] {
  const rx = Math.floor(x / REGION);
  const rz = Math.floor(z / REGION);
  const key = `${rx},${rz}`;
  let segs = riverSegCache.get(key);
  if (!segs) {
    segs = [];
    for (let ax = rx - 2; ax <= rx + 1; ax++) {
      for (let az = rz - 2; az <= rz + 1; az++) {
        const a = riverAt(ax, az);
        if (!a) continue;
        const east = riverAt(ax + 1, az);
        if (east) segs.push({ ax: a.x, az: a.z, bx: east.x, bz: east.z });
        const south = riverAt(ax, az + 1);
        if (south) segs.push({ ax: a.x, az: a.z, bx: south.x, bz: south.z });
      }
    }
    riverSegCache.set(key, segs);
  }
  return segs;
}

function nearestRiverDist(x: number, z: number): number | null {
  let best: number | null = null;
  for (const s of riverSegmentsNear(x, z)) {
    const dx = s.bx - s.ax;
    const dz = s.bz - s.az;
    const len2 = dx * dx + dz * dz;
    let t = len2 > 0 ? ((x - s.ax) * dx + (z - s.az) * dz) / len2 : 0;
    t = Math.min(1, Math.max(0, t));
    const dist = Math.hypot(x - (s.ax + dx * t), z - (s.az + dz * t));
    if (best === null || dist < best) best = dist;
  }
  return best;
}

// Depth removed from the terrain by the river at this point (0 away from
// rivers). Damped inside settlement skirts so plateaus stay walkable.
export function riverCarveAt(x: number, z: number): number {
  const d = nearestRiverDist(x, z);
  if (d === null || d > RIVER_FADE) return 0;
  const vw = villageWeight(x, z).w;
  return RIVER_DEPTH * (1 - smoothstep(RIVER_FULL, RIVER_FADE, d)) * (1 - vw);
}

// 0 dry land → 1 middle of the riverbed; for coloring and spawn filtering.
export function riverFactor(x: number, z: number): number {
  const d = nearestRiverDist(x, z);
  return d === null ? 0 : 1 - smoothstep(RIVER_FULL, RIVER_FADE, d);
}

// Water surface height: sits below the un-carved bank line, so it pokes above
// the riverbed but dives underground at the banks.
export function waterLevel(x: number, z: number): number {
  return terrainHeight(x, z) + riverCarveAt(x, z) - 0.55;
}

// ── Combined height ───────────────────────────────────────────────────────

export function terrainHeight(x: number, z: number): number {
  let h = baseHeight(x, z);
  // Flatten toward village plateaus
  const vil = villageWeight(x, z);
  if (vil.w > 0) h += (vil.h - h) * vil.w;
  // Carve the riverbed (damped inside settlement skirts)
  h -= riverCarveAt(x, z);
  // Flatten toward the road profile last → fords where roads cross rivers
  const r = nearestRoad(x, z);
  if (r) {
    const w = 1 - smoothstep(r.width, r.width + ROAD_FADE_BAND, r.dist);
    if (w > 0) h += (r.height - h) * w;
  }
  return h;
}

// ── Village plan (houses + props + parked cars) ───────────────────────────
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
    const p = TIER_PARAMS[village.tier];
    const town = village.tier === 'town';
    const heightBoost = town ? 1.5 : 1;
    const houses: HouseSlot[] = [];
    const nHouses = p.houses[0] + Math.floor(rng() * (p.houses[1] - p.houses[0] + 1));
    // Towns split their houses across two rings; smaller places use one.
    const innerCount = town ? Math.floor(nHouses / 2) : nHouses;
    const ringDef = town
      ? [
          { n: innerCount, lo: 0.35, span: 0.15 },
          { n: nHouses - innerCount, lo: 0.65, span: 0.2 }
        ]
      : [{ n: nHouses, lo: 0.5, span: 0.25 }];
    for (const ring of ringDef) {
      for (let i = 0; i < ring.n; i++) {
        const angle = (i / ring.n) * Math.PI * 2 + (rng() - 0.5) * 0.5;
        const dist = village.radius * (ring.lo + rng() * ring.span);
        const x = village.x + Math.cos(angle) * dist;
        const z = village.z + Math.sin(angle) * dist;
        houses.push({
          x,
          z,
          rot: Math.atan2(village.x - x, village.z - z),
          width: 3.2 + rng() * 1.6,
          depth: 3.0 + rng() * 1.4,
          height: (2.2 + rng() * 0.7) * heightBoost,
          variant: Math.floor(rng() * 4),
          stories: 1
        });
      }
    }
    if (town) {
      // Town hall: a wide two-story building at the center
      const angle = rng() * Math.PI * 2;
      houses.push({
        x: village.x + Math.cos(angle) * 2,
        z: village.z + Math.sin(angle) * 2,
        rot: rng() * Math.PI * 2,
        width: 6 + rng() * 2,
        depth: 5 + rng() * 2,
        height: 3 + rng() * 0.8,
        variant: Math.floor(rng() * 4),
        stories: 2
      });
    }
    const props: PropSlot[] = [];
    const nProps = p.props[0] + Math.floor(rng() * (p.props[1] - p.props[0] + 1));
    for (let i = 0; i < nProps; i++) {
      const angle = rng() * Math.PI * 2;
      const dist = village.radius * (town ? 0.5 : 0.3) * rng();
      props.push({
        x: village.x + Math.cos(angle) * dist,
        z: village.z + Math.sin(angle) * dist,
        rot: rng() * Math.PI * 2,
        modelIndex: Math.floor(rng() * 8),
        height: 1.0 + rng() * 0.5
      });
    }
    // Parked cars on the outskirts: hamlets 50% chance of one, villages one,
    // towns two. Tangential rotation reads as "parked along the ring road".
    const cars: CarSlot[] = [];
    const nCars = village.tier === 'hamlet' ? (rng() < 0.5 ? 1 : 0) : p.cars;
    for (let i = 0; i < nCars; i++) {
      const angle = rng() * Math.PI * 2;
      const dist = village.radius * 1.05;
      cars.push({
        x: village.x + Math.cos(angle) * dist,
        z: village.z + Math.sin(angle) * dist,
        rot: angle + Math.PI / 2,
        key: `${rx},${rz},${i}`
      });
    }
    plan = { village, houses, props, cars };
  }
  planCache.set(key, plan);
  return plan;
}
