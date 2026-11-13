// The heightfield as a grid of chunks (WP-081). Each chunk is its own mesh, so the renderer
// culls it against the camera's frustum; it draws at a resolution picked by its distance from
// the player (`terrainStride`); it is built when the player comes near, and its geometry is
// freed again once the player has been far from it for a while. Streaming terrain this way
// keeps an area's ground out of memory until it is needed, and the far ground cheap.
//
// The chunk the player stands on, and its neighbors, always draw every cell, so the surface
// the player sees is the one the core simulates. Coarser chunks hang a skirt below their
// border to hide the cracks against a neighbor of another resolution.

import { Mesh, type BufferGeometry, type Material, type Scene } from 'three';
import type { Heightfield, Region } from '../kits/manifest';
import type { QualityPreset } from '../lighting/quality';
import { terrainStride } from '../perf/lod';
import { buildTerrainChunk } from './heightfield';

/** Heightfield cells per chunk edge (64 m at the usual 2 m spacing). */
export const CHUNK_CELLS = 32;
/** Seconds a chunk stays built after leaving the unload distance. */
const UNLOAD_AFTER_S = 6;
/** Unload distance as a multiple of the draw distance. */
const UNLOAD_FACTOR = 1.35;
/** Chunk builds per frame once the first frame is past; the first builds everything in range. */
const BUILDS_PER_FRAME = 2;

interface Chunk {
  r0: number;
  r1: number;
  c0: number;
  c1: number;
  /** World bounds on the ground plane. */
  minX: number;
  maxX: number;
  minZ: number;
  maxZ: number;
  mesh: Mesh | null;
  stride: number;
  farFor: number;
}

export interface TerrainStats {
  chunks: number;
  built: number;
  drawn: number;
  /** Triangles in the chunks that are built and in range (before the frustum cull). */
  triangles: number;
}

export class TerrainChunks {
  private chunks: Chunk[] = [];
  private first = true;
  private builtTriangles = 0;

  constructor(
    private scene: Scene,
    private hf: Heightfield,
    private regions: readonly Region[],
    private material: Material
  ) {
    const x0 = hf.center[0] - hf.size_x / 2;
    const z0 = hf.center[2] - hf.size_z / 2;
    const dx = hf.size_x / (hf.cols - 1);
    const dz = hf.size_z / (hf.rows - 1);
    for (let r0 = 0; r0 < hf.rows - 1; r0 += CHUNK_CELLS) {
      const r1 = Math.min(r0 + CHUNK_CELLS, hf.rows - 1);
      for (let c0 = 0; c0 < hf.cols - 1; c0 += CHUNK_CELLS) {
        const c1 = Math.min(c0 + CHUNK_CELLS, hf.cols - 1);
        this.chunks.push({
          r0,
          r1,
          c0,
          c1,
          minX: x0 + c0 * dx,
          maxX: x0 + c1 * dx,
          minZ: z0 + r0 * dz,
          maxZ: z0 + r1 * dz,
          mesh: null,
          stride: 0,
          farFor: 0
        });
      }
    }
  }

  get stats(): TerrainStats {
    let built = 0;
    let drawn = 0;
    for (const c of this.chunks) {
      if (c.mesh) built++;
      if (c.mesh?.visible) drawn++;
    }
    return { chunks: this.chunks.length, built, drawn, triangles: this.builtTriangles };
  }

  private build(c: Chunk, stride: number): void {
    const geometry = buildTerrainChunk(this.hf, this.regions, {
      r0: c.r0,
      r1: c.r1,
      c0: c.c0,
      c1: c.c1,
      stride,
      skirt: 1.2 + stride * 0.35
    });
    this.builtTriangles -= c.mesh ? triangles(c.mesh.geometry) : 0;
    if (c.mesh) c.mesh.geometry.dispose();
    else {
      c.mesh = new Mesh(geometry, this.material);
      c.mesh.receiveShadow = true;
      this.scene.add(c.mesh);
    }
    c.mesh.geometry = geometry;
    c.stride = stride;
    this.builtTriangles += triangles(geometry);
  }

  private free(c: Chunk): void {
    if (!c.mesh) return;
    this.builtTriangles -= triangles(c.mesh.geometry);
    this.scene.remove(c.mesh);
    c.mesh.geometry.dispose();
    c.mesh = null;
    c.stride = 0;
  }

  /** Builds, re-resolves, shows, hides, and frees chunks around the focus. Call every frame. */
  update(fx: number, fz: number, frameMs: number, quality: QualityPreset): void {
    const draw = quality.drawDistance;
    const unload = draw * UNLOAD_FACTOR;
    let budget = this.first ? Infinity : BUILDS_PER_FRAME;
    this.first = false;
    const wanted: Array<{ c: Chunk; d: number; stride: number }> = [];
    for (const c of this.chunks) {
      const dx = Math.max(c.minX - fx, 0, fx - c.maxX);
      const dz = Math.max(c.minZ - fz, 0, fz - c.maxZ);
      const d = Math.hypot(dx, dz);
      if (d > unload) {
        c.farFor += frameMs / 1000;
        if (c.mesh) c.mesh.visible = false;
        if (c.farFor > UNLOAD_AFTER_S) this.free(c);
        continue;
      }
      c.farFor = 0;
      if (d > draw) {
        if (c.mesh) c.mesh.visible = false;
        continue;
      }
      let stride = terrainStride(d, quality);
      // Hysteresis: a chunk keeps its resolution until the distance is clearly past the boundary.
      if (c.stride && stride !== c.stride && terrainStride(d * (stride > c.stride ? 0.92 : 1.08), quality) === c.stride) stride = c.stride;
      if (!c.mesh || c.stride !== stride) wanted.push({ c, d, stride });
      else c.mesh.visible = true;
    }
    // The nearest first, so the ground under the player is never the one left waiting.
    wanted.sort((a, b) => a.d - b.d);
    for (const { c, stride } of wanted) {
      if (budget <= 0) {
        if (c.mesh) c.mesh.visible = true; // Keep drawing the old resolution.
        continue;
      }
      budget--;
      this.build(c, stride);
      c.mesh!.visible = true;
    }
  }

  dispose(): void {
    for (const c of this.chunks) this.free(c);
    this.chunks = [];
  }
}

function triangles(g: BufferGeometry): number {
  return g.getAttribute('position').count / 3;
}
