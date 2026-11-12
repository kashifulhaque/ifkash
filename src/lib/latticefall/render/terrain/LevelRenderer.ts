// Draws an area from its render manifest: the heightfield, the kit batch, scatter, sky,
// lighting, and fog. It reads data only; it never writes simulation state.
//
// This is the one object the scene talks to. `load(area)` fetches the manifest and builds
// everything; `update(...)` each frame moves the sun's shadow window, eases the region
// lighting, and shows or hides scatter chunks by distance. Quality changes arrive through
// `subscribeQuality` (see `../lighting/quality.ts`).

import { Mesh, MeshStandardMaterial, PerspectiveCamera, Scene, type WebGLRenderer, PCFSoftShadowMap } from 'three';
import { InstanceLayer } from '../kits/instances';
import { KitBatch } from '../kits/kitBatch';
import { PieceLibrary } from '../kits/library';
import { fetchManifest, type LevelManifest } from '../kits/manifest';
import { applyHeightFog, heightFogUniforms } from '../lighting/fogPatch';
import { LevelLighting } from '../lighting/lighting';
import { getQuality, subscribeQuality, type QualityPreset } from '../lighting/quality';
import { Sky } from '../sky/sky';
import { buildTerrainGeometry } from './heightfield';

export class LevelRenderer {
  manifest: LevelManifest | null = null;
  private terrain: Mesh | null = null;
  private terrainMaterial: MeshStandardMaterial | null = null;
  private library: PieceLibrary | null = null;
  private batch: KitBatch | null = null;
  private instances: InstanceLayer | null = null;
  private lighting: LevelLighting | null = null;
  private sky: Sky | null = null;
  private quality: QualityPreset = getQuality();
  private unsubscribe: (() => void) | null = null;
  private disposed = false;

  constructor(
    private scene: Scene,
    private renderer: WebGLRenderer
  ) {}

  /** The region the player is standing in, or null. Read-only; for the HUD and dev tools. */
  get region(): string | null {
    return this.lighting?.region ?? null;
  }

  /**
   * Fetches `area`'s manifest and builds the level. Resolves true when something was
   * drawn, false when the area has no manifest (the flat scenarios keep their own ground).
   */
  async load(area: string): Promise<boolean> {
    const manifest = await fetchManifest(area);
    if (!manifest || this.disposed) return false;
    this.manifest = manifest;
    const a = manifest.area;

    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = PCFSoftShadowMap;

    this.lighting = new LevelLighting(this.scene, a.ambient, a.regions);
    this.sky = new Sky(this.scene, a.ambient.sky, this.lighting.sunDirection);
    this.scene.background = this.lighting.fogColor;

    if (manifest.terrain) {
      this.terrainMaterial = new MeshStandardMaterial({ vertexColors: true, flatShading: true, roughness: 1, metalness: 0 });
      applyHeightFog(this.terrainMaterial);
      this.terrain = new Mesh(buildTerrainGeometry(manifest.terrain, a.regions), this.terrainMaterial);
      this.terrain.receiveShadow = true;
      this.scene.add(this.terrain);
    }

    this.library = new PieceLibrary(manifest);
    this.batch = new KitBatch(this.scene, manifest, this.library);
    this.instances = new InstanceLayer(this.scene, manifest, this.library);

    this.applyQuality(this.quality);
    this.unsubscribe = subscribeQuality((p) => this.applyQuality(p));
    return true;
  }

  private applyQuality(p: QualityPreset): void {
    this.quality = p;
    this.lighting?.setQuality(p);
    this.instances?.setGrassDensity(p.grassDensity);
  }

  /** Hollowdeep's darkness (WP-062): how far the player sees, in meters; 0 for no limit. */
  setVisibility(meters: number): void {
    this.lighting?.setVisibility(meters);
  }

  /** Call every frame, before the scene renders. `focus` is the player's position. */
  update(camera: PerspectiveCamera, focus: { x: number; y: number; z: number }, frameMs: number): void {
    if (!this.lighting || !this.sky) return;
    this.lighting.update(focus.x, focus.y, focus.z, frameMs);
    camera.updateMatrixWorld();
    heightFogUniforms.lfViewInv.value.copy(camera.matrixWorld);
    this.sky.update(camera, this.lighting.fogColor, this.lighting.sunColor);
    this.instances?.update(focus.x, focus.z, this.quality.drawDistance);
  }

  dispose(): void {
    this.disposed = true;
    this.unsubscribe?.();
    this.instances?.dispose();
    this.batch?.dispose();
    this.library?.dispose();
    if (this.terrain) {
      this.scene.remove(this.terrain);
      this.terrain.geometry.dispose();
    }
    this.terrainMaterial?.dispose();
    this.sky?.dispose();
    this.lighting?.dispose();
  }
}
