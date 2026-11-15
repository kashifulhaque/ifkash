// The piece library: the stand-in geometry and material that model-less kit pieces share,
// and streaming of model files.
//
// A model has a key. A kit piece whose catalog entry names a model uses its piece ID as the
// key; a model from the area art manifest (`art.ts`) uses `art:MODEL_ID`. A piece with no
// model draws its procedural stand-in for good. Anything with a model draws its stand-in (or
// the greybox behind it) until the file arrives, then the `onModel` callbacks fire and the
// instance code swaps the model in. A file that fails to load keeps the stand-in.
//
// Models are fetched only when something that uses them first comes inside the draw distance,
// so an area's files stream in as the player approaches. A loaded model keeps the materials
// its file carries (see `modelCache.ts`).

import { BufferGeometry, MeshStandardMaterial, DoubleSide } from 'three';
import { applyHeightFog } from '../lighting/fogPatch';
import type { AreaArt } from './art';
import type { LevelManifest, ManifestPiece } from './manifest';
import { ModelCache, type KitModel } from './modelCache';
import { needsDoubleSide, placeholderGeometry } from './pieceGeometry';

/** Called with the model when it loads, and with null when it is unloaded again. */
type ModelListener = (key: string, model: KitModel | null) => void;

/** The key of a model from the area art manifest. */
export function artKey(modelId: string): string {
  return `art:${modelId}`;
}

export class PieceLibrary {
  private geometries = new Map<string, BufferGeometry>();
  private models = new Map<string, KitModel>();
  private loading = new Set<string>();
  private listeners = new Set<ModelListener>();
  private single: MeshStandardMaterial;
  private double: MeshStandardMaterial;
  private cache = new ModelCache();
  private disposed = false;
  /** Keys whose model failed to load; they keep their stand-in. */
  readonly failed = new Set<string>();

  constructor(
    private manifest: LevelManifest,
    private art: AreaArt | null = null
  ) {
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

  /** The stand-in material of a kit piece. */
  material(id: string): MeshStandardMaterial {
    return needsDoubleSide(this.piece(id)) ? this.double : this.single;
  }

  /** The stand-in geometry of a kit piece. */
  geometry(id: string): BufferGeometry {
    let g = this.geometries.get(id);
    if (!g) {
      g = placeholderGeometry(id, this.piece(id));
      this.geometries.set(id, g);
    }
    return g;
  }

  /** The model file behind a key, or null when it has none. */
  private path(key: string): string | null {
    if (key.startsWith('art:')) return this.art?.models[key.slice(4)]?.file ?? null;
    return this.manifest.pieces[key]?.model ?? null;
  }

  /** Whether instances of the model cast shadows. */
  castsShadow(key: string): boolean {
    if (key.startsWith('art:')) return this.art?.models[key.slice(4)]?.shadow !== false;
    return !this.piece(key).tags.includes('ash_grass');
  }

  /** The loaded model, or null while it is loading, unloaded, or failed. */
  model(key: string): KitModel | null {
    return this.models.get(key) ?? null;
  }

  /** True when the key has a model that is neither loaded, loading, nor failed. */
  needsModel(key: string): boolean {
    return !!this.path(key) && !this.loading.has(key) && !this.failed.has(key) && !this.models.has(key);
  }

  /** How many models are still loading. The benchmark waits for 0 before it measures. */
  get pending(): number {
    return this.loading.size;
  }

  onModel(fn: ModelListener): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  /** Starts loading a model once. Failure keeps the stand-in and logs a warning. */
  ensureModel(key: string): void {
    const path = this.path(key);
    if (!path || !this.needsModel(key)) return;
    this.loading.add(key);
    this.cache
      .acquire(path)
      .then((model) => {
        if (this.disposed) return;
        this.models.set(key, model);
        for (const fn of this.listeners) fn(key, model);
      })
      .catch((err) => {
        this.failed.add(key);
        console.warn(`latticefall: model ${path} for ${key} failed to load; keeping the stand-in`, err);
      })
      .finally(() => this.loading.delete(key));
  }

  /**
   * Unloads a model (WP-081 streaming): the instance code goes back to the stand-in through
   * `onModel`, then the model's GPU memory is freed. The next time something that uses it
   * comes near, `ensureModel` fetches it again (from the browser's cache, normally).
   */
  release(key: string): void {
    const path = this.path(key);
    if (!path || !this.models.delete(key)) return;
    for (const fn of this.listeners) fn(key, null);
    this.cache.release(path);
  }

  dispose(): void {
    this.disposed = true;
    for (const g of this.geometries.values()) g.dispose();
    this.geometries.clear();
    this.models.clear();
    this.cache.dispose();
    this.single.dispose();
    this.double.dispose();
    this.listeners.clear();
  }
}
