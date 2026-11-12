// The heightfield mesh: flat-shaded triangles with one vertex color per face, picked from the
// region palette, the face's slope, and its height. Vertex positions are the core's
// (`Heightfield` in game/crates/content/src/scenario.rs), so the surface the player
// walks on and the one drawn agree at every vertex.

import { BufferAttribute, BufferGeometry, Color } from 'three';
import type { Heightfield, Region } from '../kits/manifest';
import { paletteFor } from './palette';
import { regionAt } from './regionMath';

/** Slope, as the face normal's Y, at and below which a face is fully rock. */
const ROCK_NY = 0.62;
/** And at and above which it is fully grass. */
const GRASS_NY = 0.86;
/** Faces above this height (meters) lean toward the lighter high-ground color. */
const HIGH_FROM = 1.5;
const HIGH_TO = 6;

/** A small deterministic hash in [0, 1) for per-face color variation. */
function hash2(a: number, b: number): number {
  let h = Math.imul(a, 374761393) ^ Math.imul(b, 668265263);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

const smooth = (a: number, b: number, v: number): number => {
  const t = Math.min(1, Math.max(0, (v - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

export function buildTerrainGeometry(hf: Heightfield, regions: readonly Region[]): BufferGeometry {
  const { rows, cols } = hf;
  const x0 = hf.center[0] - hf.size_x / 2;
  const z0 = hf.center[2] - hf.size_z / 2;
  const dx = hf.size_x / (cols - 1);
  const dz = hf.size_z / (rows - 1);
  const px = (c: number): number => x0 + c * dx;
  const pz = (r: number): number => z0 + r * dz;
  const py = (r: number, c: number): number => hf.center[1] + hf.heights[r * cols + c];

  const faces = (rows - 1) * (cols - 1) * 2;
  const pos = new Float32Array(faces * 9);
  const col = new Float32Array(faces * 9);
  const tmp = new Color();
  const grassA = new Color();
  const grassB = new Color();
  const rock = new Color();
  const high = new Color();
  let f = 0;

  const face = (r: number, c: number, i: Array<[number, number]>): void => {
    const o = f * 9;
    for (let k = 0; k < 3; k++) {
      pos[o + k * 3] = px(i[k][1]);
      pos[o + k * 3 + 1] = py(i[k][0], i[k][1]);
      pos[o + k * 3 + 2] = pz(i[k][0]);
    }
    const ux = pos[o + 3] - pos[o];
    const uy = pos[o + 4] - pos[o + 1];
    const uz = pos[o + 5] - pos[o + 2];
    const vx = pos[o + 6] - pos[o];
    const vy = pos[o + 7] - pos[o + 1];
    const vz = pos[o + 8] - pos[o + 2];
    const nx = uy * vz - uz * vy;
    const ny = uz * vx - ux * vz;
    const nz = ux * vy - uy * vx;
    const ny01 = Math.abs(ny) / (Math.hypot(nx, ny, nz) || 1);
    const cx = (pos[o] + pos[o + 3] + pos[o + 6]) / 3;
    const cy = (pos[o + 1] + pos[o + 4] + pos[o + 7]) / 3;
    const cz = (pos[o + 2] + pos[o + 5] + pos[o + 8]) / 3;
    const pal = paletteFor(regionAt(regions, cx, cz)?.name ?? null);
    grassA.set(pal.grass[0]);
    grassB.set(pal.grass[1]);
    rock.set(pal.rock);
    high.set(pal.high);
    const n = hash2(r * 2 + (f & 1), c);
    tmp.copy(grassA).lerp(grassB, n);
    tmp.lerp(high, smooth(HIGH_FROM, HIGH_TO, cy) * 0.6);
    tmp.lerp(rock, 1 - smooth(ROCK_NY, GRASS_NY, ny01));
    // A few percent of brightness variation keeps flat ground from reading as one slab.
    tmp.multiplyScalar(0.94 + hash2(c + 91, r * 7 + (f & 1)) * 0.12);
    for (let k = 0; k < 3; k++) {
      col[o + k * 3] = tmp.r;
      col[o + k * 3 + 1] = tmp.g;
      col[o + k * 3 + 2] = tmp.b;
    }
    f++;
  };

  for (let r = 0; r < rows - 1; r++) {
    for (let c = 0; c < cols - 1; c++) {
      // Counter-clockwise from above (+Y up).
      face(r, c, [[r, c], [r + 1, c], [r, c + 1]]);
      face(r, c, [[r, c + 1], [r + 1, c], [r + 1, c + 1]]);
    }
  }

  const g = new BufferGeometry();
  g.setAttribute('position', new BufferAttribute(pos, 3));
  g.setAttribute('color', new BufferAttribute(col, 3));
  g.computeVertexNormals(); // Non-indexed, so every normal is its face's: flat shading.
  g.computeBoundingSphere();
  g.computeBoundingBox();
  return g;
}
