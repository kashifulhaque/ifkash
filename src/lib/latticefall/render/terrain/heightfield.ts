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

/** A rectangle of the heightfield, in vertex indices (rows `r0` up to `r1`, columns `c0` up to `c1`). */
export interface ChunkSpec {
  r0: number;
  r1: number;
  c0: number;
  c1: number;
  /** Heightfield cells per drawn quad edge: 1 draws every cell, 2 every other, and so on. */
  stride: number;
  /** Meters of skirt hung below the chunk's border, to hide cracks against a neighbor of another stride. 0 for none. */
  skirt: number;
}

/** Builds the whole heightfield as one geometry, every cell, no skirts. */
export function buildTerrainGeometry(hf: Heightfield, regions: readonly Region[]): BufferGeometry {
  return buildTerrainChunk(hf, regions, { r0: 0, r1: hf.rows - 1, c0: 0, c1: hf.cols - 1, stride: 1, skirt: 0 });
}

/**
 * Builds one rectangle of the heightfield. The quads are `stride` cells wide; a rectangle whose
 * size isn't a multiple of `stride` ends in a narrower quad, so neighbors always meet exactly at
 * the chunk border's corners. Vertex positions are the core's at every drawn vertex.
 */
export function buildTerrainChunk(hf: Heightfield, regions: readonly Region[], spec: ChunkSpec): BufferGeometry {
  const { cols } = hf;
  const { r0, r1, c0, c1, stride, skirt } = spec;
  const x0 = hf.center[0] - hf.size_x / 2;
  const z0 = hf.center[2] - hf.size_z / 2;
  const dx = hf.size_x / (cols - 1);
  const dz = hf.size_z / (hf.rows - 1);
  const px = (c: number): number => x0 + c * dx;
  const pz = (r: number): number => z0 + r * dz;
  const py = (r: number, c: number): number => hf.center[1] + hf.heights[r * cols + c];

  const nr = Math.ceil((r1 - r0) / stride);
  const nc = Math.ceil((c1 - c0) / stride);
  const skirtFaces = skirt > 0 ? 2 * (nr + nc) * 2 : 0;
  const faces = nr * nc * 2 + skirtFaces;
  const pos = new Float32Array(faces * 9);
  const col = new Float32Array(faces * 9);
  const tmp = new Color();
  const grassA = new Color();
  const grassB = new Color();
  const rock = new Color();
  const high = new Color();
  let f = 0;
  /** The surface color of each border quad, for the skirt under it: north, south, west, east. */
  const edge = { n: new Float32Array(nc * 3), s: new Float32Array(nc * 3), w: new Float32Array(nr * 3), e: new Float32Array(nr * 3) };

  const face = (r: number, c: number, i: Array<[number, number]>, qr: number, qc: number): void => {
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
    if (skirt > 0) {
      const put = (a: Float32Array, at: number): void => {
        a[at * 3] = tmp.r;
        a[at * 3 + 1] = tmp.g;
        a[at * 3 + 2] = tmp.b;
      };
      if (qr === 0) put(edge.n, qc);
      if (qr === nr - 1) put(edge.s, qc);
      if (qc === 0) put(edge.w, qr);
      if (qc === nc - 1) put(edge.e, qr);
    }
    f++;
  };

  for (let qr = 0; qr < nr; qr++) {
    const r = r0 + qr * stride;
    const rn = Math.min(r + stride, r1);
    for (let qc = 0; qc < nc; qc++) {
      const c = c0 + qc * stride;
      const cn = Math.min(c + stride, c1);
      // Counter-clockwise from above (+Y up).
      face(r, c, [[r, c], [rn, c], [r, cn]], qr, qc);
      face(r, c, [[r, cn], [rn, c], [rn, cn]], qr, qc);
    }
  }

  if (skirt > 0) {
    // A vertical strip under each border segment, wound to face outward.
    const strip = (ax: number, ay: number, az: number, bx: number, by: number, bz: number, rgb: Float32Array, k: number, ox: number, oz: number): void => {
      const quad: Array<[number, number, number]> = [
        [ax, ay, az],
        [bx, by, bz],
        [bx, by - skirt, bz],
        [ax, ay - skirt, az]
      ];
      for (const tri of [[0, 1, 2], [0, 2, 3]]) {
        const [p, q, r] = tri.map((i) => quad[i]);
        // Flip the winding if the face's normal points inward.
        const nx = (q[1] - p[1]) * (r[2] - p[2]) - (q[2] - p[2]) * (r[1] - p[1]);
        const nz = (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
        const out = nx * ox + nz * oz >= 0;
        const order = out ? [p, q, r] : [p, r, q];
        const o = f * 9;
        order.forEach((v, j) => {
          pos[o + j * 3] = v[0];
          pos[o + j * 3 + 1] = v[1];
          pos[o + j * 3 + 2] = v[2];
          col[o + j * 3] = rgb[k * 3];
          col[o + j * 3 + 1] = rgb[k * 3 + 1];
          col[o + j * 3 + 2] = rgb[k * 3 + 2];
        });
        f++;
      }
    };
    for (let qc = 0; qc < nc; qc++) {
      const c = c0 + qc * stride;
      const cn = Math.min(c + stride, c1);
      strip(px(c), py(r0, c), pz(r0), px(cn), py(r0, cn), pz(r0), edge.n, qc, 0, -1);
      strip(px(c), py(r1, c), pz(r1), px(cn), py(r1, cn), pz(r1), edge.s, qc, 0, 1);
    }
    for (let qr = 0; qr < nr; qr++) {
      const r = r0 + qr * stride;
      const rn = Math.min(r + stride, r1);
      strip(px(c0), py(r, c0), pz(r), px(c0), py(rn, c0), pz(rn), edge.w, qr, -1, 0);
      strip(px(c1), py(r, c1), pz(r), px(c1), py(rn, c1), pz(rn), edge.e, qr, 1, 0);
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
