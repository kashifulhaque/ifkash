// Instanced scatter and model-backed kit pieces. Each (batch, activity cell) pair is one
// `InstancedMesh` chunk, so a chunk is one draw call and a whole cell drops out beyond the
// draw distance. Grass thins by hiding the tail of a chunk's shuffled instance list.

import { InstancedMesh, Matrix4, Quaternion, Scene, Vector3 } from 'three';
import type { PieceLibrary } from './library';
import type { LevelManifest } from './manifest';

interface Chunk {
  mesh: InstancedMesh;
  piece: string;
  total: number;
  /** Cell bounds on the ground plane: minX, minZ, maxX, maxZ. */
  box: [number, number, number, number];
  /** Grass thins with the grass-density setting; everything else draws in full. */
  thins: boolean;
}

function hash(seed: number): number {
  let h = Math.imul(seed | 0, 2654435761);
  h = Math.imul(h ^ (h >>> 15), 2246822519);
  return ((h ^ (h >>> 13)) >>> 0) / 4294967296;
}

const UP = new Vector3(0, 1, 0);
/** A model nothing has been near for this long is unloaded (WP-081). */
const RELEASE_AFTER_MS = 20_000;

export class InstanceLayer {
  private chunks: Chunk[] = [];
  private grass = 1;
  private offModel: () => void;
  /** When each piece last had a chunk within the draw distance, in `performance.now()` ms. */
  private lastNear = new Map<string, number>();

  constructor(
    private scene: Scene,
    manifest: LevelManifest,
    private library: PieceLibrary
  ) {
    const size = manifest.area.cells.size_m;
    const m = new Matrix4();
    const q = new Quaternion();
    const p = new Vector3();
    const s = new Vector3();

    // Scatter: one chunk per (batch, cell).
    // Scatter with no model is `ScatterBatch`'s job (one draw call for the whole field).
    for (const batch of manifest.scatter) {
      if (!manifest.pieces[batch.piece]?.model) continue;
      const cells = new Map<string, Array<(typeof batch.transforms)[number]>>();
      for (const t of batch.transforms) {
        const key = `${Math.floor(t[0] / size)},${Math.floor(t[2] / size)}`;
        let list = cells.get(key);
        if (!list) cells.set(key, (list = []));
        list.push(t);
      }
      for (const [key, list] of cells) {
        // A fixed shuffle, so cutting the tail thins the chunk evenly across the cell.
        const order = list.map((t, i) => ({ t, k: hash(i * 7919 + batch.rule * 31 + key.length) })).sort((a, b) => a.k - b.k);
        const [cx, cz] = key.split(',').map(Number);
        const mesh = this.makeMesh(batch.piece, order.length);
        order.forEach(({ t }, i) => {
          q.setFromAxisAngle(UP, t[3]);
          mesh.setMatrixAt(i, m.compose(p.set(t[0], t[1], t[2]), q, s.setScalar(t[4])));
        });
        this.chunks.push({
          mesh,
          piece: batch.piece,
          total: order.length,
          box: [cx * size, cz * size, (cx + 1) * size, (cz + 1) * size],
          thins: batch.tag.includes('grass')
        });
      }
    }

    // Layout pieces and props that have a model draw instanced, one chunk per piece. (Pieces
    // without a model are the kit batch's job.)
    const byPiece = new Map<string, LevelManifest['instances']>();
    for (const inst of manifest.instances) {
      if (!manifest.pieces[inst.piece]?.model) continue;
      let list = byPiece.get(inst.piece);
      if (!list) byPiece.set(inst.piece, (list = []));
      list.push(inst);
    }
    for (const [piece, list] of byPiece) {
      const mesh = this.makeMesh(piece, list.length);
      let x0 = Infinity;
      let z0 = Infinity;
      let x1 = -Infinity;
      let z1 = -Infinity;
      list.forEach((inst, i) => {
        q.setFromAxisAngle(UP, inst.yaw);
        mesh.setMatrixAt(i, m.compose(p.set(...inst.pos), q, s.set(...inst.scale)));
        x0 = Math.min(x0, inst.pos[0]);
        x1 = Math.max(x1, inst.pos[0]);
        z0 = Math.min(z0, inst.pos[2]);
        z1 = Math.max(z1, inst.pos[2]);
      });
      this.chunks.push({ mesh, piece, total: list.length, box: [x0 - 10, z0 - 10, x1 + 10, z1 + 10], thins: false });
    }

    for (const c of this.chunks) c.mesh.visible = false;
    this.offModel = library.onModel((id, geometry) => {
      for (const c of this.chunks) if (c.piece === id) c.mesh.geometry = geometry;
    });
  }

  private makeMesh(piece: string, count: number): InstancedMesh {
    const mesh = new InstancedMesh(this.library.geometry(piece), this.library.material(piece), count);
    const tags = this.library.piece(piece).tags;
    mesh.castShadow = !tags.includes('ash_grass');
    mesh.receiveShadow = true;
    mesh.instanceMatrix.needsUpdate = true;
    this.scene.add(mesh);
    return mesh;
  }

  /** How many chunks exist, for the report and dev tools. */
  get chunkCount(): number {
    return this.chunks.length;
  }

  setGrassDensity(f: number): void {
    this.grass = Math.min(1, Math.max(0, f));
    for (const c of this.chunks) c.mesh.count = c.thins ? Math.ceil(c.total * this.grass) : c.total;
  }

  /** Shows the chunks within `drawDistance` of the focus and starts streaming their models. */
  update(fx: number, fz: number, drawDistance: number): void {
    const now = performance.now();
    for (const c of this.chunks) {
      const dx = Math.max(c.box[0] - fx, 0, fx - c.box[2]);
      const dz = Math.max(c.box[1] - fz, 0, fz - c.box[3]);
      const near = dx * dx + dz * dz <= drawDistance * drawDistance;
      c.mesh.visible = near && c.mesh.count > 0;
      if (near) this.lastNear.set(c.piece, now);
      if (near && this.library.needsModel(c.piece)) this.library.ensureModel(c.piece);
    }
    // Streaming: a piece nothing has been near for a while gives its model back.
    for (const [piece, at] of this.lastNear) {
      if (now - at > RELEASE_AFTER_MS) {
        this.lastNear.delete(piece);
        this.library.release(piece);
      }
    }
  }

  dispose(): void {
    this.offModel();
    for (const c of this.chunks) {
      this.scene.remove(c.mesh);
      c.mesh.dispose();
    }
    this.chunks = [];
  }
}
