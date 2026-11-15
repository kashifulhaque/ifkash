// The area art manifest: which model files draw an area's layout, props, Lantern, and NPCs.
// `lf-tools assets` copies it from `game/assets-src/art/AREA.json` to
// `static/latticefall/art/AREA.json`. It is render data only. The simulation never reads it,
// and collision still comes from the level's own colliders, so an area with no art manifest,
// or with a model that fails to load, draws its greybox as before.
//
// Format 1:
//   - `models`: model ID to file. `file` is relative to `static/latticefall/`. `shadow: false`
//     stops a small model from casting shadows.
//   - `instances`: layout instance name (from the level manifest) to a fill. The model
//     replaces that instance's greybox once it loads.
//   - `props`: models placed freely in world space, with no collider behind them. A prop
//     with `covers` replaces those layout instances' greybox once it loads, so a model built
//     in world space around the colliders (Waking Hollow's cave) can stand in for many boxes.
//   - `entities.lantern`: the model for every Lantern in the area.
//   - `entities.npcs`: NPC ID to the figure that replaces its greybox stand-in.
//   - `entities.hollow_wall`: the model for every hollow wall, centered on its collider.
//   - `lights`: fixed light spots. The renderer lights the nearest few with a small pool of
//     point lights (`../lighting/lightPool.ts`); the quality preset sets the pool's size.
//   - `look`: overrides of the level's `ambient` block (fog, sun, ambient light) and the
//     height-fog strength, for art that needs a different mood than the greybox did. The
//     level data, and so the content hash, stays as it is.
//   - `grade`: tone mapping and a color grade for the area (`../lighting/grade.ts`).

import { Matrix4, Quaternion, Vector3, type Box3 } from 'three';
import type { ManifestInstance, V3 } from './manifest';

export interface ArtModel {
  file: string;
  /** Whether instances cast shadows. Defaults to true. */
  shadow?: boolean;
  /**
   * Keeps each mesh node of the file a separate part instead of merging the file into one
   * part per material, so the renderer can cull and sort a large world-space model (a whole
   * cave) piece by piece. Costs one draw call per node.
   */
  nodes?: boolean;
}

export interface ArtFill {
  model: string;
  /**
   * Copies along the instance's x, y, and z. A 0 or missing entry is chosen from the model's
   * size, so a 4 m wall model fills a 60 m wall 15 times.
   */
  count?: [number, number, number];
  /**
   * `stretch` (the default) scales each copy to fill its cell exactly, so the drawing matches
   * the collider. `contain` keeps the model's proportions and stands it on the cell's floor.
   */
  fit?: 'stretch' | 'contain';
  /** Quarter turns about Y applied to the model before it is fitted. */
  turns?: number;
}

export interface ArtProp {
  model: string;
  pos: V3;
  /** Radians from +Z toward +X, like every yaw in the level manifest. */
  yaw?: number;
  scale?: number | V3;
  /** Layout instance names whose greybox this prop replaces once it loads. */
  covers?: string[];
}

export interface ArtLantern {
  model: string;
  /** A node that sways gently, by name. */
  sway?: string;
  /** A point light at `at` (the Lantern's frame, meters). */
  light?: { color: string; intensity: number; distance: number; at: V3 };
}

export interface ArtNpc {
  model: string;
  /** Standing height in meters. */
  height: number;
  /** The animation-library clip the figure loops. */
  clip: string;
}

/** A fixed light spot. Intensity is in candela, with the renderer's decay of 1.6. */
export interface ArtLight {
  name?: string;
  pos: V3;
  color: string;
  intensity: number;
  distance: number;
  /** How much the light flickers, 0 to 1. */
  flicker?: number;
}

/** Overrides of the level's `ambient` block; see `Ambient` in `manifest.ts`. */
export interface ArtLook {
  fog_color?: V3;
  fog_near_m?: number;
  fog_far_m?: number;
  sun_color?: V3;
  sun_intensity?: number;
  ambient_color?: V3;
  ambient_intensity?: number;
  /** Strength of the low-lying height fog, 0 to 1. */
  haze?: number;
}

/** Tone mapping and a color grade, applied in every material's shader (no extra pass). */
export interface ArtGrade {
  exposure?: number;
  /** 1 keeps the color, 0 is grey. */
  saturation?: number;
  /** Around mid grey; 1 keeps it. */
  contrast?: number;
  /** Added to the shadows, per channel. */
  lift?: V3;
  /** Multiplies the highlights, per channel. */
  gain?: V3;
}

export interface AreaArt {
  format: number;
  models: Record<string, ArtModel>;
  instances?: Record<string, ArtFill>;
  props?: ArtProp[];
  entities?: {
    lantern?: ArtLantern;
    npcs?: Record<string, ArtNpc>;
    hollow_wall?: { model: string };
  };
  lights?: ArtLight[];
  look?: ArtLook;
  grade?: ArtGrade;
}

const ART_ROOT = '/latticefall/art/';
/** `lf-tools assets` lists every output here, so an area with no art costs no 404. */
const INDEX_URL = '/latticefall/manifest.json';

let index: Promise<Set<string>> | null = null;
const cache = new Map<string, Promise<AreaArt | null>>();

async function json(url: string): Promise<unknown> {
  const res = await fetch(url);
  if (!res.ok || !(res.headers.get('content-type') ?? '').includes('json')) return null;
  return res.json();
}

function artIndex(): Promise<Set<string>> {
  index ??= json(INDEX_URL)
    .then((data) => {
      const files = (data as { regions?: { art?: Array<{ path?: string }> } } | null)?.regions?.art ?? [];
      return new Set(files.map((f) => String(f.path)));
    })
    .catch(() => new Set<string>());
  return index;
}

/**
 * Fetches `area`'s art manifest. Resolves null when the area has none or the file is
 * unreadable; the area then draws its greybox. Shared between the level and the props.
 */
export function fetchAreaArt(area: string): Promise<AreaArt | null> {
  let p = cache.get(area);
  if (!p) {
    p = artIndex()
      .then(async (files) => {
        if (!files.has(`${area}.json`)) return null;
        const art = (await json(ART_ROOT + encodeURIComponent(area) + '.json')) as AreaArt | null;
        if (!art) return null;
        if (art.format !== 1 || typeof art.models !== 'object') {
          console.warn(`latticefall: art manifest for ${area} has an unsupported format; drawing the greybox`);
          return null;
        }
        return art;
      })
      .catch((err) => {
        console.warn(`latticefall: art manifest for ${area} failed to load; drawing the greybox`, err);
        return null;
      });
    cache.set(area, p);
  }
  return p;
}

const UP = new Vector3(0, 1, 0);

/**
 * The transforms that fill a layout instance's box with copies of a model. The box is the
 * instance's unit cube: centered on `pos`, `scale` meters on each side, turned by `yaw`.
 */
export function fillMatrices(inst: ManifestInstance, fill: ArtFill, bounds: Box3): Matrix4[] {
  const turns = (((fill.turns ?? 0) % 4) + 4) % 4;
  const odd = turns % 2 === 1;
  const size = bounds.getSize(new Vector3());
  const center = bounds.getCenter(new Vector3());
  // The model's size along the box's axes, after the quarter turns.
  const m: V3 = odd ? [size.z, size.y, size.x] : [size.x, size.y, size.z];
  const box: V3 = [Math.abs(inst.scale[0]), Math.abs(inst.scale[1]), Math.abs(inst.scale[2])];
  const n = [0, 1, 2].map((k) => {
    const asked = fill.count?.[k] ?? 0;
    return asked > 0 ? Math.round(asked) : Math.max(1, Math.round(box[k] / Math.max(m[k], 1e-4)));
  });
  const cell: V3 = [box[0] / n[0], box[1] / n[1], box[2] / n[2]];
  // Scale along the box's axes, then mapped back to the model's own axes.
  let s: V3 = [cell[0] / Math.max(m[0], 1e-4), cell[1] / Math.max(m[1], 1e-4), cell[2] / Math.max(m[2], 1e-4)];
  const contain = fill.fit === 'contain';
  if (contain) s = [Math.min(...s), Math.min(...s), Math.min(...s)];
  const local = new Vector3(odd ? s[2] : s[0], s[1], odd ? s[0] : s[2]);

  const boxM = new Matrix4().compose(new Vector3(...inst.pos), new Quaternion().setFromAxisAngle(UP, inst.yaw), new Vector3(1, 1, 1));
  const turn = new Matrix4().makeRotationY((turns * Math.PI) / 2);
  // Moves the model's center (or, for `contain`, the center of its base) to the origin.
  const recenter = new Matrix4().makeTranslation(-center.x, contain ? -bounds.min.y : -center.y, -center.z);
  const fitted = turn.clone().multiply(new Matrix4().makeScale(local.x, local.y, local.z)).multiply(recenter);
  const out: Matrix4[] = [];
  for (let i = 0; i < n[0]; i++) {
    for (let j = 0; j < n[1]; j++) {
      for (let k = 0; k < n[2]; k++) {
        const x = -box[0] / 2 + cell[0] * (i + 0.5);
        const y = -box[1] / 2 + cell[1] * (contain ? j : j + 0.5);
        const z = -box[2] / 2 + cell[2] * (k + 0.5);
        out.push(boxM.clone().multiply(new Matrix4().makeTranslation(x, y, z)).multiply(fitted));
      }
    }
  }
  return out;
}

/** The transform of a freely placed prop. */
export function propMatrix(p: ArtProp): Matrix4 {
  const s = p.scale ?? 1;
  const scale = typeof s === 'number' ? new Vector3(s, s, s) : new Vector3(...s);
  return new Matrix4().compose(new Vector3(...p.pos), new Quaternion().setFromAxisAngle(UP, p.yaw ?? 0), scale);
}
