// Instanced drawing of everything that has a model file: scatter and layout pieces whose kit
// piece names a model, and the fills and props of the area art manifest (`art.ts`).
//
// A chunk is one model's instances in one place. It draws one `InstancedMesh` per material in
// the model, so a chunk costs as many draw calls as its model has materials, and it drops out
// whole beyond the draw distance. Until the model loads, a kit piece draws its stand-in, and
// an art fill leaves the greybox of the kit batch in place. Grass thins by hiding the tail of
// a chunk's shuffled instance list.

import { InstancedMesh, Matrix4, Quaternion, Scene, Vector3, type Box3 } from 'three';
import { fillMatrices, propMatrix, type AreaArt } from './art';
import type { KitBatch } from './kitBatch';
import { artKey, type PieceLibrary } from './library';
import type { LevelManifest } from './manifest';
import type { KitModel } from './modelCache';

interface Chunk {
  /** The library key of the chunk's model. */
  key: string;
  /** The kit piece's stand-in, drawn until the model loads. Null for art fills and props. */
  standIn: InstancedMesh | null;
  /** One mesh per material of the loaded model. */
  meshes: InstancedMesh[];
  /** Instance transforms. Art fills need the model's size, so theirs wait for the model. */
  matrices: Matrix4[] | null;
  layout: ((bounds: Box3) => Matrix4[]) | null;
  /** Greybox instances the model replaces, by name. */
  covers: string[];
  total: number;
  /** Bounds on the ground plane: minX, minZ, maxX, maxZ. */
  box: [number, number, number, number];
  /** Grass thins with the grass-density setting; everything else draws in full. */
  thins: boolean;
  near: boolean;
}

function hash(seed: number): number {
  let h = Math.imul(seed | 0, 2654435761);
  h = Math.imul(h ^ (h >>> 15), 2246822519);
  return ((h ^ (h >>> 13)) >>> 0) / 4294967296;
}

const UP = new Vector3(0, 1, 0);
/** A model nothing has been near for this long is unloaded (WP-081). */
const RELEASE_AFTER_MS = 20_000;
/** Meters added around a chunk's instances, so a wide model at the edge still counts as near. */
const CHUNK_PAD_M = 10;

export class InstanceLayer {
  private chunks: Chunk[] = [];
  private grass = 1;
  private offModel: () => void;
  /** When each model last had a chunk within the draw distance, in `performance.now()` ms. */
  private lastNear = new Map<string, number>();

  constructor(
    private scene: Scene,
    manifest: LevelManifest,
    private library: PieceLibrary,
    art: AreaArt | null = null,
    private batch: KitBatch | null = null
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
        const matrices = order.map(({ t }) => {
          q.setFromAxisAngle(UP, t[3]);
          return new Matrix4().compose(p.set(t[0], t[1], t[2]), q, s.setScalar(t[4]));
        });
        this.addChunk(batch.piece, matrices, [cx * size, cz * size, (cx + 1) * size, (cz + 1) * size], batch.tag.includes('grass'), true);
      }
    }

    // Layout pieces and props whose kit piece has a model draw instanced, one chunk per piece.
    // (Pieces without a model are the kit batch's job.)
    const byPiece = new Map<string, LevelManifest['instances']>();
    for (const inst of manifest.instances) {
      if (!manifest.pieces[inst.piece]?.model) continue;
      let list = byPiece.get(inst.piece);
      if (!list) byPiece.set(inst.piece, (list = []));
      list.push(inst);
    }
    for (const [piece, list] of byPiece) {
      const matrices = list.map((inst) => {
        q.setFromAxisAngle(UP, inst.yaw);
        return new Matrix4().compose(p.set(...inst.pos), q, s.set(...inst.scale));
      });
      this.addChunk(piece, matrices, boxOf(list.map((i) => [i.pos[0], i.pos[2], 0])), false, true);
    }

    // The area art manifest: one chunk per model, holding every fill and prop that uses it.
    if (art) {
      const fills = new Map<string, Array<{ inst: LevelManifest['instances'][number]; fill: NonNullable<AreaArt['instances']>[string] }>>();
      const props = new Map<string, NonNullable<AreaArt['props']>>();
      const byName = new Map(manifest.instances.map((i) => [i.name, i]));
      for (const [name, fill] of Object.entries(art.instances ?? {})) {
        const inst = byName.get(name);
        // A kit piece with its own model already draws it; the art manifest only replaces greybox.
        if (!inst || manifest.pieces[inst.piece]?.model || !art.models[fill.model]) {
          console.warn(`latticefall: art manifest fill "${name}" has no greybox instance or no model; skipping it`);
          continue;
        }
        let list = fills.get(fill.model);
        if (!list) fills.set(fill.model, (list = []));
        list.push({ inst, fill });
      }
      for (const prop of art.props ?? []) {
        if (!art.models[prop.model]) {
          console.warn(`latticefall: art manifest prop uses unknown model "${prop.model}"; skipping it`);
          continue;
        }
        let list = props.get(prop.model);
        if (!list) props.set(prop.model, (list = []));
        list.push(prop);
      }
      for (const id of new Set([...fills.keys(), ...props.keys()])) {
        const f = fills.get(id) ?? [];
        const pr = props.get(id) ?? [];
        // A prop's `covers` are layout instances its model replaces; they also give its extent.
        const covered = pr.flatMap((x) => (x.covers ?? []).map((n) => byName.get(n)).filter((i): i is LevelManifest['instances'][number] => !!i));
        const spots: Array<[number, number, number]> = [
          ...[...f.map(({ inst }) => inst), ...covered].map((inst): [number, number, number] => [
            inst.pos[0],
            inst.pos[2],
            Math.hypot(inst.scale[0], inst.scale[2]) / 2
          ]),
          ...pr.map((x): [number, number, number] => [x.pos[0], x.pos[2], 0])
        ];
        this.chunks.push({
          key: artKey(id),
          standIn: null,
          meshes: [],
          matrices: null,
          layout: (bounds) => [...f.flatMap(({ inst, fill }) => fillMatrices(inst, fill, bounds)), ...pr.map(propMatrix)],
          covers: [...f.map(({ inst }) => inst.name), ...covered.map((inst) => inst.name)],
          total: 0,
          box: boxOf(spots),
          thins: false,
          near: false
        });
      }
    }

    this.offModel = library.onModel((key, model) => {
      for (const c of this.chunks) if (c.key === key) this.setModel(c, model);
    });
  }

  private addChunk(piece: string, matrices: Matrix4[], box: Chunk['box'], thins: boolean, standIn: boolean): void {
    const c: Chunk = { key: piece, standIn: null, meshes: [], matrices, layout: null, covers: [], total: matrices.length, box, thins, near: false };
    if (standIn) {
      c.standIn = this.makeMesh(c, this.library.geometry(piece), this.library.material(piece));
    }
    this.chunks.push(c);
  }

  private makeMesh(c: Chunk, geometry: InstancedMesh['geometry'], material: InstancedMesh['material']): InstancedMesh {
    const matrices = c.matrices ?? [];
    const mesh = new InstancedMesh(geometry, material, matrices.length);
    matrices.forEach((mat, i) => mesh.setMatrixAt(i, mat));
    mesh.instanceMatrix.needsUpdate = true;
    mesh.castShadow = this.library.castsShadow(c.key);
    mesh.receiveShadow = true;
    mesh.count = c.thins ? Math.ceil(c.total * this.grass) : c.total;
    mesh.visible = false;
    this.scene.add(mesh);
    return mesh;
  }

  /** Swaps a chunk to its loaded model, or back to its stand-in when `model` is null. */
  private setModel(c: Chunk, model: KitModel | null): void {
    for (const mesh of c.meshes) {
      this.scene.remove(mesh);
      mesh.dispose();
    }
    c.meshes = [];
    if (model) {
      if (c.layout) {
        c.matrices = c.layout(model.bounds);
        c.total = c.matrices.length;
      }
      c.meshes = model.parts.map((part) => this.makeMesh(c, part.geometry, part.material));
    }
    this.batch?.setCovered(c.covers, !!model);
    this.show(c);
  }

  private show(c: Chunk): void {
    const loaded = c.meshes.length > 0;
    if (c.standIn) c.standIn.visible = c.near && !loaded && c.standIn.count > 0;
    for (const mesh of c.meshes) mesh.visible = c.near && mesh.count > 0;
  }

  /** How many chunks exist, for the report and dev tools. */
  get chunkCount(): number {
    return this.chunks.length;
  }

  setGrassDensity(f: number): void {
    this.grass = Math.min(1, Math.max(0, f));
    for (const c of this.chunks) {
      const count = c.thins ? Math.ceil(c.total * this.grass) : c.total;
      if (c.standIn) c.standIn.count = count;
      for (const mesh of c.meshes) mesh.count = count;
    }
  }

  /** Shows the chunks within `drawDistance` of the focus and starts streaming their models. */
  update(fx: number, fz: number, drawDistance: number): void {
    const now = performance.now();
    for (const c of this.chunks) {
      const dx = Math.max(c.box[0] - fx, 0, fx - c.box[2]);
      const dz = Math.max(c.box[1] - fz, 0, fz - c.box[3]);
      c.near = dx * dx + dz * dz <= drawDistance * drawDistance;
      this.show(c);
      if (c.near) this.lastNear.set(c.key, now);
      if (c.near && this.library.needsModel(c.key)) this.library.ensureModel(c.key);
    }
    // Streaming: a model nothing has been near for a while is unloaded.
    for (const [key, at] of this.lastNear) {
      if (now - at > RELEASE_AFTER_MS) {
        this.lastNear.delete(key);
        this.library.release(key);
      }
    }
  }

  dispose(): void {
    this.offModel();
    for (const c of this.chunks) {
      for (const mesh of c.standIn ? [c.standIn, ...c.meshes] : c.meshes) {
        this.scene.remove(mesh);
        mesh.dispose();
      }
    }
    this.chunks = [];
  }
}

/** The ground-plane bounds of `[x, z, radius]` spots, padded. */
function boxOf(spots: Array<[number, number, number]>): Chunk['box'] {
  let x0 = Infinity;
  let z0 = Infinity;
  let x1 = -Infinity;
  let z1 = -Infinity;
  for (const [x, z, r] of spots) {
    x0 = Math.min(x0, x - r);
    x1 = Math.max(x1, x + r);
    z0 = Math.min(z0, z - r);
    z1 = Math.max(z1, z + r);
  }
  return [x0 - CHUNK_PAD_M, z0 - CHUNK_PAD_M, x1 + CHUNK_PAD_M, z1 + CHUNK_PAD_M];
}
