// Loads model files (GLB) for level pieces and props, and frees them again. A model keeps
// the materials its file carries: vertex colors, flat colors, or textured PBR maps all draw
// as authored. Nothing here assumes one kind of material.
//
// Two shapes of result:
//   - `acquire` returns a `KitModel` for instancing: static geometry flattened to the model's
//     own frame, one part per material. Models are shared and counted, so the last `release`
//     frees the GPU memory.
//   - `loadScene` returns the scene graph untouched, for props with moving parts and skinned
//     figures. The caller owns it and frees it with `disposeScene`.

import { Box3, type BufferGeometry, type Material, type Mesh, type Object3D, type SkinnedMesh } from 'three';
import type { GLTFLoader, GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import { createGltfLoader } from '../assets/gltfLoader';
import { applyHeightFog } from '../lighting/fogPatch';
import { getQuality } from '../lighting/quality';
import { capSceneTextures, sceneTextures } from '../perf/textures';

/** Where model files live: paths in the manifests are relative to `static/latticefall/`. */
export const MODEL_ROOT = '/latticefall/';

/**
 * The loader for level models: the shared loader, which reads meshopt geometry and KTX2
 * textures (ADR 0010) through the scene's renderer.
 */
export function createModelLoader(): GLTFLoader {
  return createGltfLoader();
}

export interface ModelPart {
  geometry: BufferGeometry;
  material: Material;
}

export interface KitModel {
  /** One part per material in the file, in the model's own frame. */
  parts: ModelPart[];
  /** The bounds of every part together. */
  bounds: Box3;
  triangles: number;
}

/** Frees a scene's geometries, materials, textures, and bone textures. */
export function disposeScene(root: Object3D): void {
  for (const t of sceneTextures(root)) t.dispose();
  root.traverse((o) => {
    const mesh = o as Partial<SkinnedMesh>;
    if (mesh.isSkinnedMesh) mesh.skeleton?.dispose();
    mesh.geometry?.dispose();
    const mat = mesh.material as Material | Material[] | undefined;
    if (Array.isArray(mat)) mat.forEach((m) => m.dispose());
    else mat?.dispose();
  });
}

/** Flattens a loaded scene into one geometry per material. */
function flatten(scene: Object3D): KitModel {
  scene.updateMatrixWorld(true);
  const byMaterial = new Map<Material, BufferGeometry[]>();
  scene.traverse((o) => {
    const mesh = o as Mesh;
    if (!mesh.isMesh) return;
    const material = (Array.isArray(mesh.material) ? mesh.material[0] : mesh.material) as Material;
    const g = mesh.geometry.clone();
    g.applyMatrix4(mesh.matrixWorld);
    let list = byMaterial.get(material);
    if (!list) byMaterial.set(material, (list = []));
    list.push(g);
    mesh.geometry.dispose();
  });
  const parts: ModelPart[] = [];
  const bounds = new Box3();
  let triangles = 0;
  for (const [material, list] of byMaterial) {
    // Geometries with different attribute sets can't merge; they stay separate parts.
    const merged = list.length > 1 ? mergeGeometries(list, false) : null;
    if (merged) list.forEach((g) => g.dispose());
    applyHeightFog(material);
    for (const geometry of merged ? [merged] : list) {
      geometry.computeBoundingBox();
      geometry.computeBoundingSphere();
      bounds.union(geometry.boundingBox!);
      triangles += (geometry.index ? geometry.index.count : geometry.getAttribute('position').count) / 3;
      parts.push({ geometry, material });
    }
  }
  if (parts.length === 0) throw new Error('no meshes in model');
  return { parts, bounds, triangles };
}

function disposeModel(model: KitModel): void {
  for (const p of model.parts) {
    p.geometry.dispose();
    for (const k of Object.values(p.material) as unknown[]) {
      const t = k as { isTexture?: boolean; dispose?: () => void } | null;
      if (t && t.isTexture) t.dispose?.();
    }
    p.material.dispose();
  }
}

interface Entry {
  users: number;
  model: Promise<KitModel>;
}

export class ModelCache {
  private entries = new Map<string, Entry>();
  private loader: GLTFLoader | null = null;
  private disposed = false;

  private load(path: string): Promise<GLTF> {
    this.loader ??= createModelLoader();
    return this.loader.loadAsync(MODEL_ROOT + path).then((gltf) => {
      // Texture budget (WP-081): no texture goes above the quality preset's longest edge.
      capSceneTextures(gltf.scene, getQuality().name);
      return gltf;
    });
  }

  /** Loads `path` once and counts the caller as a user. Rejects if the file fails to load. */
  acquire(path: string): Promise<KitModel> {
    let e = this.entries.get(path);
    if (!e) {
      e = { users: 0, model: this.load(path).then((gltf) => flatten(gltf.scene)) };
      this.entries.set(path, e);
      // A failed file is forgotten, so nothing holds a rejected promise.
      e.model.catch(() => this.entries.delete(path));
    }
    e.users++;
    return e.model;
  }

  /** Gives back one use of `path`. The last one frees the model's GPU memory. */
  release(path: string): void {
    const e = this.entries.get(path);
    if (!e || --e.users > 0) return;
    this.entries.delete(path);
    e.model.then(disposeModel, () => undefined);
  }

  /** Loads `path` with its scene graph intact. The caller frees it with `disposeScene`. */
  async loadScene(path: string): Promise<GLTF> {
    const gltf = await this.load(path);
    if (this.disposed) {
      disposeScene(gltf.scene);
      throw new Error('model cache was disposed');
    }
    return gltf;
  }

  /** How many flattened models are held, loaded or loading. For the dev tools. */
  get size(): number {
    return this.entries.size;
  }

  dispose(): void {
    this.disposed = true;
    for (const e of this.entries.values()) e.model.then(disposeModel, () => undefined);
    this.entries.clear();
  }
}
