// Scatter (grass, stones, dead trees) that has no model yet, drawn through one `BatchedMesh`
// per material instead of one `InstancedMesh` per (batch, cell). Scatter is sparse, so the cell
// chunks of `InstanceLayer` cost a draw call for every handful of instances (the Verge had 277
// chunks); here the whole field is one call, culled per instance against the frustum by the
// batch and against the draw distance by `DistanceCuller`.
//
// Scatter whose piece has a model stays in `InstanceLayer`, which swaps the geometry when the
// model streams in.

import { BatchedMesh, Matrix4, Quaternion, Scene, Vector3, type Material } from 'three';
import { DistanceCuller, type CullEntry } from '../perf/culling';
import type { PieceLibrary } from './library';
import type { LevelManifest } from './manifest';

function hash(seed: number): number {
  let h = Math.imul(seed | 0, 2654435761);
  h = Math.imul(h ^ (h >>> 15), 2246822519);
  return ((h ^ (h >>> 13)) >>> 0) / 4294967296;
}

const UP = new Vector3(0, 1, 0);

interface Layer {
  mesh: BatchedMesh;
  cull: DistanceCuller;
}

export class ScatterBatch {
  private layers: Layer[] = [];
  private grass = 1;
  private lastFx = 0;
  private lastFz = 0;
  private lastDistance = 0;

  constructor(
    private scene: Scene,
    manifest: LevelManifest,
    library: PieceLibrary
  ) {
    // Group the modelless scatter batches by material (single- and double-sided pieces can't share).
    const groups = new Map<Material, LevelManifest['scatter']>();
    for (const batch of manifest.scatter) {
      if (manifest.pieces[batch.piece]?.model) continue;
      const mat = library.material(batch.piece);
      let list = groups.get(mat);
      if (!list) groups.set(mat, (list = []));
      list.push(batch);
    }
    const m = new Matrix4();
    const q = new Quaternion();
    const p = new Vector3();
    const s = new Vector3();
    for (const [material, batches] of groups) {
      const pieces = [...new Set(batches.map((b) => b.piece))];
      let verts = 0;
      for (const id of pieces) verts += library.geometry(id).getAttribute('position').count;
      let total = 0;
      for (const b of batches) total += b.transforms.length;
      if (total === 0) continue;
      const mesh = new BatchedMesh(total, verts, 0, material);
      const geo = new Map<string, number>();
      for (const id of pieces) geo.set(id, mesh.addGeometry(library.geometry(id)));
      const entries: CullEntry[] = [];
      for (const batch of batches) {
        const thin = batch.tag.includes('grass');
        const gid = geo.get(batch.piece)!;
        batch.transforms.forEach((t, i) => {
          const id = mesh.addInstance(gid);
          q.setFromAxisAngle(UP, t[3]);
          mesh.setMatrixAt(id, m.compose(p.set(t[0], t[1], t[2]), q, s.setScalar(t[4])));
          // A fixed shuffle rank, so thinning removes an even share of each field.
          entries.push({ id, x: t[0], z: t[2], rank: thin ? hash(i * 7919 + batch.rule * 31) : -1 });
        });
      }
      const tags = pieces.map((id) => library.piece(id).tags);
      mesh.castShadow = !tags.every((t) => t.includes('ash_grass'));
      mesh.receiveShadow = true;
      mesh.computeBoundingSphere();
      mesh.computeBoundingBox();
      scene.add(mesh);
      this.layers.push({ mesh, cull: new DistanceCuller(mesh, entries) });
    }
  }

  /** How many batched meshes (draw calls) the scatter takes at most. */
  get meshCount(): number {
    return this.layers.length;
  }

  /** Instances currently drawn, before the frustum cull. */
  get shown(): number {
    return this.layers.reduce((n, l) => n + l.cull.shown, 0);
  }

  setGrassDensity(f: number): void {
    this.grass = Math.min(1, Math.max(0, f));
    this.refresh();
  }

  private refresh(): void {
    for (const l of this.layers) l.cull.update(this.lastFx, this.lastFz, this.lastDistance, this.grass);
  }

  /** Shows the instances within `drawDistance` of the focus. Call every frame. */
  update(fx: number, fz: number, drawDistance: number): void {
    this.lastFx = fx;
    this.lastFz = fz;
    this.lastDistance = drawDistance;
    for (const l of this.layers) l.cull.update(fx, fz, drawDistance, this.grass);
  }

  dispose(): void {
    for (const l of this.layers) {
      this.scene.remove(l.mesh);
      l.mesh.dispose();
    }
    this.layers = [];
  }
}
