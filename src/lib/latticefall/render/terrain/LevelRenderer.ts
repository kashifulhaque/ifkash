// Draws an area from its render manifest: the heightfield, the kit batch, scatter, sky,
// lighting, and fog. It reads data only; it never writes simulation state.
//
// This is the one object the scene talks to. `load(area)` fetches the manifest and builds
// everything; `update(...)` each frame moves the sun's shadow window, eases the region
// lighting, and shows or hides scatter chunks by distance. Quality changes arrive through
// `subscribeQuality` (see `../lighting/quality.ts`).

import { MeshStandardMaterial, PerspectiveCamera, Scene, type WebGLRenderer, PCFSoftShadowMap } from 'three';
import type { KitModel } from '../kits/modelCache';
import { InstanceLayer } from '../kits/instances';
import { KitBatch } from '../kits/kitBatch';
import { ScatterBatch } from '../kits/scatterBatch';
import { PieceLibrary, artKey } from '../kits/library';
import { fetchManifest, type LevelManifest } from '../kits/manifest';
import { fetchAreaArt, propMatrix, type AreaArt } from '../kits/art';
import { CeilingMap } from '../camera/ceiling';
import { applyGrade } from '../lighting/grade';
import { LightPool } from '../lighting/lightPool';
import { applyHeightFog, heightFogUniforms } from '../lighting/fogPatch';
import { LevelLighting } from '../lighting/lighting';
import { getQuality, subscribeQuality, type QualityPreset } from '../lighting/quality';
import { Sky } from '../sky/sky';
import { TerrainChunks, type TerrainStats } from './TerrainChunks';

export class LevelRenderer {
  manifest: LevelManifest | null = null;
  private terrain: TerrainChunks | null = null;
  private terrainMaterial: MeshStandardMaterial | null = null;
  private library: PieceLibrary | null = null;
  private batch: KitBatch | null = null;
  private instances: InstanceLayer | null = null;
  private scatter: ScatterBatch | null = null;
  private lighting: LevelLighting | null = null;
  private sky: Sky | null = null;
  /** The art manifest's torches and candles, lit by a few point lights. */
  private lights: LightPool | null = null;
  private undoGrade: (() => void) | null = null;
  private quality: QualityPreset = getQuality();
  private unsubscribe: (() => void) | null = null;
  private offCeiling: (() => void) | null = null;
  private disposed = false;
  /**
   * The ceilings of the art manifest's world-space props (Waking Hollow's cave), for the
   * camera. Null until such a model loads; the level's colliders have no ceilings.
   */
  ceiling: CeilingMap | null = null;
  /** Art models already in the ceiling map, so a streamed reload doesn't add them twice. */
  private mapped = new Set<string>();

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
    // The art manifest names the model files that replace the area's greybox. Most areas have none.
    const [manifest, art] = await Promise.all([fetchManifest(area), fetchAreaArt(area)]);
    if (!manifest || this.disposed) return false;
    this.manifest = manifest;
    const a = manifest.area;

    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = PCFSoftShadowMap;

    // The art manifest may darken or thicken the level's look; the level data stays as it is.
    this.lighting = new LevelLighting(this.scene, withLook(a.ambient, art), a.regions, art?.look?.haze ?? 0.4);
    if (art?.lights?.length) this.lights = new LightPool(this.scene, art.lights, this.quality.lights);
    if (art?.grade) this.undoGrade = applyGrade(this.renderer, art.grade);
    this.sky = new Sky(this.scene, a.ambient.sky, this.lighting.sunDirection);
    this.scene.background = this.lighting.fogColor;

    if (manifest.terrain) {
      this.terrainMaterial = new MeshStandardMaterial({ vertexColors: true, flatShading: true, roughness: 1, metalness: 0 });
      applyHeightFog(this.terrainMaterial);
      this.terrain = new TerrainChunks(this.scene, manifest.terrain, a.regions, this.terrainMaterial);
    }

    this.library = new PieceLibrary(manifest, art);
    this.batch = new KitBatch(this.scene, manifest, this.library);
    this.scatter = new ScatterBatch(this.scene, manifest, this.library);
    this.instances = new InstanceLayer(this.scene, manifest, this.library, art, this.batch);
    if (art?.props?.length) this.offCeiling = this.library.onModel((key, model) => this.mapCeiling(art, key, model));

    this.applyQuality(this.quality);
    this.unsubscribe = subscribeQuality((p) => this.applyQuality(p));
    return true;
  }

  /** Adds a loaded world-space art model to the ceiling map, once per model. */
  private mapCeiling(art: AreaArt, key: string, model: KitModel | null): void {
    if (!model || this.mapped.has(key)) return;
    const props = (art.props ?? []).filter((p) => artKey(p.model) === key);
    if (props.length === 0) return;
    this.mapped.add(key);
    const map = this.ceiling ?? new CeilingMap();
    for (const p of props) {
      const m = propMatrix(p);
      for (const part of model.parts) map.add(part.geometry, m);
    }
    this.ceiling = map;
  }

  private applyQuality(p: QualityPreset): void {
    this.quality = p;
    this.lighting?.setQuality(p);
    this.lights?.setSize(p.lights);
    this.scatter?.setGrassDensity(p.grassDensity);
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
    this.lights?.update(focus.x, focus.y + 1, focus.z, frameMs);
    camera.updateMatrixWorld();
    heightFogUniforms.lfViewInv.value.copy(camera.matrixWorld);
    this.sky.update(camera, this.lighting.fogColor, this.lighting.sunColor);
    const draw = this.quality.drawDistance;
    this.terrain?.update(focus.x, focus.z, frameMs, this.quality);
    this.batch?.update(focus.x, focus.z, draw);
    this.scatter?.update(focus.x, focus.z, draw);
    this.instances?.update(focus.x, focus.z, draw);
  }

  /** How many model files are still loading. The benchmark waits for 0 before it measures. */
  get modelsPending(): number {
    return this.library?.pending ?? 0;
  }

  /** Terrain streaming numbers, for the benchmark and the dev tools. Null with no terrain. */
  get terrainStats(): TerrainStats | null {
    return this.terrain?.stats ?? null;
  }

  dispose(): void {
    this.disposed = true;
    this.unsubscribe?.();
    this.offCeiling?.();
    this.instances?.dispose();
    this.scatter?.dispose();
    this.batch?.dispose();
    this.library?.dispose();
    this.terrain?.dispose();
    this.terrainMaterial?.dispose();
    this.sky?.dispose();
    this.lighting?.dispose();
    this.lights?.dispose();
    this.undoGrade?.();
  }
}

/** The level's ambient block with the art manifest's `look` laid over it. */
function withLook(ambient: LevelManifest['area']['ambient'], art: AreaArt | null): LevelManifest['area']['ambient'] {
  const look = art?.look;
  if (!look) return ambient;
  return {
    ...ambient,
    fog_color: look.fog_color ?? ambient.fog_color,
    fog_near_m: look.fog_near_m ?? ambient.fog_near_m,
    fog_far_m: look.fog_far_m ?? ambient.fog_far_m,
    sun_color: look.sun_color ?? ambient.sun_color,
    sun_intensity: look.sun_intensity ?? ambient.sun_intensity,
    ambient_color: look.ambient_color ?? ambient.ambient_color,
    ambient_intensity: look.ambient_intensity ?? ambient.ambient_intensity
  };
}
