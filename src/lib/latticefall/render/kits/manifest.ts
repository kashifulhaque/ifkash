// The render manifest `lf-tools bake` writes to static/latticefall/levels/AREA.json
// (format documented in game/crates/tools/src/bake/manifest.rs). Data only: the renderer
// reads it and never writes simulation state.

export type V3 = [number, number, number];

export interface Ambient {
  sky: string;
  fog_color: V3;
  fog_near_m: number;
  fog_far_m: number;
  sun_dir: V3;
  sun_color: V3;
  sun_intensity: number;
  ambient_color: V3;
  ambient_intensity: number;
}

export interface Region {
  name: string;
  points: Array<[number, number]>;
}

export interface Heightfield {
  center: V3;
  rows: number;
  cols: number;
  size_x: number;
  size_z: number;
  heights: number[];
}

export type Collider =
  | { Box: { center: V3; half_extents: V3 } }
  | { Capsule: { center: V3; half_height: number; radius: number } };

export interface ManifestPiece {
  model: string | null;
  tags: string[];
  colliders: Collider[];
}

export interface ManifestInstance {
  name: string;
  piece: string;
  pos: V3;
  yaw: number;
  scale: V3;
  source: 'piece' | 'landmark' | 'prop';
}

export interface ScatterBatch {
  rule: number;
  tag: string;
  piece: string;
  collider: boolean;
  /** Each `[x, y, z, yaw, scale]`. */
  transforms: Array<[number, number, number, number, number]>;
}

export interface LevelManifest {
  format: number;
  area: {
    id: string;
    name: string;
    bounds: { min: [number, number]; max: [number, number] };
    cells: { size_m: number; min: [number, number]; max: [number, number] };
    kill_y: number;
    ambient: Ambient;
    regions: Region[];
  };
  terrain: Heightfield | null;
  pieces: Record<string, ManifestPiece>;
  instances: ManifestInstance[];
  scatter: ScatterBatch[];
  spawn: { pos: V3; yaw: number } | null;
}

/** Where the baker puts a level's manifest, relative to the site root. */
export function manifestUrl(area: string): string {
  return `/latticefall/levels/${encodeURIComponent(area)}.json`;
}

/** Fetches and checks a manifest. Returns null when the area has none (the flat scenarios). */
export async function fetchManifest(area: string): Promise<LevelManifest | null> {
  const res = await fetch(manifestUrl(area));
  if (!res.ok) return null;
  const type = res.headers.get('content-type') ?? '';
  if (!type.includes('json')) return null; // A dev server's HTML fallback page.
  const data = (await res.json()) as LevelManifest;
  if (data.format !== 1) throw new Error(`latticefall: level manifest format ${data.format} is not supported`);
  return data;
}
