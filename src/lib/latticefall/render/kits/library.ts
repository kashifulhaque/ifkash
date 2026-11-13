// The piece library: the geometry and material every instance of a kit piece shares, and
// streaming of piece models. A piece with `model: null` uses its procedural stand-in for
// good; a piece with a model draws the stand-in until the glTF arrives, then swaps (the
// `onModel` callbacks fire, and the instance code replaces its geometry).
//
// Models are fetched only when a chunk that uses the piece first comes inside the draw
// distance, so an area's assets stream in as the player approaches. Only the geometry is
// kept: art direction allows vertex colors and small gradient textures, and the shared
// material draws vertex colors only.

import { BufferGeometry, Mesh, MeshStandardMaterial, DoubleSide, type Object3D } from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { MeshoptDecoder } from 'three/examples/jsm/libs/meshopt_decoder.module.js';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import { applyHeightFog } from '../lighting/fogPatch';
import type { LevelManifest, ManifestPiece } from './manifest';
import { needsDoubleSide, placeholderGeometry, toKitGeometry } from './pieceGeometry';

/** Where piece models live: `model` paths are relative to `static/latticefall/`. */
const MODEL_ROOT = '/latticefall/';

type ModelListener = (pieceId: string, geometry: BufferGeometry) => void;

export class PieceLibrary {
  private geometries = new Map<string, BufferGeometry>();
  private loading = new Set<string>();
  private listeners = new Set<ModelListener>();
  private single: MeshStandardMaterial;
  private double: MeshStandardMaterial;
  private loader: GLTFLoader | null = null;
  private disposed = false;
  /** Pieces whose model failed to load; they keep their stand-in. */
  readonly failed = new Set<string>();

  constructor(private manifest: LevelManifest) {
    const make = (side: typeof DoubleSide | 0): MeshStandardMaterial => {
      const m = new MeshStandardMaterial({ vertexColors: true, flatShading: true, roughness: 0.95, metalness: 0 });
      if (side) m.side = DoubleSide;
      applyHeightFog(m);
      return m;
    };
    this.single = make(0);
    this.double = make(DoubleSide);
  }

  piece(id: string): ManifestPiece {
    const p = this.manifest.pieces[id];
    if (!p) throw new Error(`latticefall: level uses piece ${id} that its manifest does not describe`);
    return p;
  }

  material(id: string): MeshStandardMaterial {
    return needsDoubleSide(this.piece(id)) ? this.double : this.single;
  }

  /** The geometry to draw now: the loaded model if there is one, else the stand-in. */
  geometry(id: string): BufferGeometry {
    let g = this.geometries.get(id);
    if (!g) {
      g = placeholderGeometry(id, this.piece(id));
      this.geometries.set(id, g);
    }
    return g;
  }

  /** True when the piece has a model that is neither loaded nor failed. */
  needsModel(id: string): boolean {
    return !!this.piece(id).model && !this.loading.has(id) && !this.failed.has(id) && !this.loaded.has(id);
  }

  private loaded = new Set<string>();

  onModel(fn: ModelListener): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  /** Starts loading a piece's model once. Failure keeps the stand-in and logs a warning. */
  ensureModel(id: string): void {
    const path = this.piece(id).model;
    if (!path || this.loading.has(id) || this.loaded.has(id) || this.failed.has(id)) return;
    this.loading.add(id);
    this.loader ??= new GLTFLoader().setMeshoptDecoder(MeshoptDecoder);
    this.loader
      .loadAsync(MODEL_ROOT + path)
      .then((gltf) => {
        if (this.disposed) return;
        const parts: BufferGeometry[] = [];
        gltf.scene.updateMatrixWorld(true);
        gltf.scene.traverse((o: Object3D) => {
          const mesh = o as Mesh;
          if (!mesh.isMesh) return;
          const g = toKitGeometry(mesh.geometry);
          g.applyMatrix4(mesh.matrixWorld);
          parts.push(g);
        });
        const merged = parts.length ? mergeGeometries(parts, false) : null;
        if (!merged) throw new Error('no meshes in model');
        this.geometries.get(id)?.dispose();
        this.geometries.set(id, merged);
        this.loaded.add(id);
        for (const fn of this.listeners) fn(id, merged);
      })
      .catch((err) => {
        this.failed.add(id);
        console.warn(`latticefall: model ${path} for piece ${id} failed to load; keeping the stand-in`, err);
      })
      .finally(() => this.loading.delete(id));
  }

  /**
   * Unloads a piece's model (WP-081 streaming): frees its geometry and goes back to the stand-in,
   * which the instance code swaps in through `onModel`. The next time a chunk of the piece comes
   * near, `ensureModel` fetches it again (from the browser's cache, normally).
   */
  release(id: string): void {
    if (!this.loaded.delete(id)) return;
    this.geometries.get(id)?.dispose();
    this.geometries.delete(id);
    const standIn = this.geometry(id);
    for (const fn of this.listeners) fn(id, standIn);
  }

  dispose(): void {
    this.disposed = true;
    for (const g of this.geometries.values()) g.dispose();
    this.geometries.clear();
    this.single.dispose();
    this.double.dispose();
    this.listeners.clear();
  }
}
