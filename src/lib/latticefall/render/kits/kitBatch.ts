// The kit batch: every layout piece, landmark, and prop that has no model draws through one
// `BatchedMesh`, so any mix of pieces (the greybox blocks, today) costs a single draw call.
// Pieces with a model go through `InstanceLayer` instead.

import { BatchedMesh, Color, Matrix4, Quaternion, Scene, Vector3 } from 'three';
import { DistanceCuller, type CullEntry } from '../perf/culling';
import type { PieceLibrary } from './library';
import type { LevelManifest } from './manifest';

function hash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619);
  return (h >>> 0) / 4294967296;
}

/** Radius of a piece's bounding sphere, cached per geometry. */
function pieceRadius(library: PieceLibrary, piece: string): number {
  const g = library.geometry(piece);
  if (!g.boundingSphere) g.computeBoundingSphere();
  return g.boundingSphere?.radius ?? 0;
}

export class KitBatch {
  readonly mesh: BatchedMesh | null = null;
  private cull: DistanceCuller | null = null;

  constructor(
    private scene: Scene,
    manifest: LevelManifest,
    library: PieceLibrary
  ) {
    const insts = manifest.instances.filter((i) => !manifest.pieces[i.piece]?.model);
    if (insts.length === 0) return;
    const ids = [...new Set(insts.map((i) => i.piece))];
    let verts = 0;
    for (const id of ids) verts += library.geometry(id).getAttribute('position').count;
    // One shared material: the batch can't mix single- and double-sided pieces, and the
    // greybox pieces are all single-sided.
    const mesh = new BatchedMesh(insts.length, verts, 0, library.material(ids[0]));
    const geo = new Map<string, number>();
    for (const id of ids) geo.set(id, mesh.addGeometry(library.geometry(id)));
    const m = new Matrix4();
    const q = new Quaternion();
    const up = new Vector3(0, 1, 0);
    const p = new Vector3();
    const s = new Vector3();
    const tint = new Color();
    const entries: CullEntry[] = [];
    for (const inst of insts) {
      const id = mesh.addInstance(geo.get(inst.piece)!);
      entries.push({
        id,
        x: inst.pos[0],
        z: inst.pos[2],
        rank: -1,
        pad: pieceRadius(library, inst.piece) * Math.max(...inst.scale)
      });
      q.setFromAxisAngle(up, inst.yaw);
      mesh.setMatrixAt(id, m.compose(p.set(...inst.pos), q, s.set(...inst.scale)));
      // A few percent of per-instance tint so stacked steps and blocks read as separate pieces.
      const v = 0.9 + hash(inst.name) * 0.2;
      mesh.setColorAt(id, tint.setRGB(v, v, v));
    }
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    // Instances span the whole area; a single bounding sphere over them is what culling uses.
    mesh.computeBoundingSphere();
    mesh.computeBoundingBox();
    scene.add(mesh);
    this.mesh = mesh;
    this.cull = new DistanceCuller(mesh, entries);
  }

  /** Hides the pieces beyond `drawDistance` of the focus (they are also culled by the frustum). */
  update(fx: number, fz: number, drawDistance: number): void {
    this.cull?.update(fx, fz, drawDistance);
  }

  get instanceCount(): number {
    return this.mesh?.instanceCount ?? 0;
  }

  dispose(): void {
    if (!this.mesh) return;
    this.scene.remove(this.mesh);
    this.mesh.dispose();
  }
}
