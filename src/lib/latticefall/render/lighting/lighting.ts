// Area lighting: a hemisphere light for ambient, one shadowed sun, and the fog. It reads the
// area's `ambient` block from the manifest, bends it by the region the focus point is in
// (`regions.ts`), and eases between looks so a border crossing is not a pop.

import { Color, DirectionalLight, Fog, HemisphereLight, Object3D, Scene, Vector3 } from 'three';
import type { Ambient, Region } from '../kits/manifest';
import { regionAt } from '../terrain/regionMath';
import { heightFogUniforms } from './fogPatch';
import { NEUTRAL_LOOK, REGION_LOOKS, type RegionLook } from './regions';
import type { QualityPreset } from './quality';

/** The manifest's 0..1 light intensities to Three's physically scaled ones. */
const SUN_GAIN = 3.4;
const AMBIENT_GAIN = 1.9;
/** Half-width of the sun shadow's coverage around the focus, meters. */
const SHADOW_HALF = 46;
const SUN_DISTANCE = 90;
/** Seconds-ish for a look change to settle. */
const EASE_MS = 700;

interface Look {
  fogColor: Color;
  fogNear: number;
  fogFar: number;
  sunColor: Color;
  sunIntensity: number;
  ambientColor: Color;
  ambientIntensity: number;
  haze: number;
}

export class LevelLighting {
  readonly fog: Fog;
  readonly hemi: HemisphereLight;
  readonly sun: DirectionalLight;
  private target = new Object3D();
  private look: Look;
  private goal: Look;
  private snapNext = true;
  private sunDir: Vector3;
  private shadowSize = 0;
  /** The region the focus is in right now (null outside every region). */
  region: string | null = null;

  constructor(
    private scene: Scene,
    private ambient: Ambient,
    private regions: readonly Region[]
  ) {
    this.sunDir = new Vector3(...ambient.sun_dir).normalize();
    this.look = this.compute(NEUTRAL_LOOK);
    this.goal = this.compute(NEUTRAL_LOOK);
    this.fog = new Fog(this.look.fogColor.clone(), this.look.fogNear, this.look.fogFar);
    scene.fog = this.fog;
    this.hemi = new HemisphereLight(0xffffff, 0xffffff, 1);
    this.sun = new DirectionalLight(0xffffff, 1);
    this.sun.shadow.camera.left = -SHADOW_HALF;
    this.sun.shadow.camera.right = SHADOW_HALF;
    this.sun.shadow.camera.top = SHADOW_HALF;
    this.sun.shadow.camera.bottom = -SHADOW_HALF;
    this.sun.shadow.camera.near = 1;
    this.sun.shadow.camera.far = SUN_DISTANCE * 2.2;
    this.sun.shadow.bias = -0.0004;
    this.sun.shadow.normalBias = 0.04;
    this.sun.target = this.target;
    scene.add(this.hemi, this.sun, this.target);
    this.apply();
  }

  /** Whether the sun casts shadows right now. */
  get shadows(): boolean {
    return this.sun.castShadow;
  }

  private compute(look: RegionLook): Look {
    const a = this.ambient;
    const t = look.fogTint ?? [1, 1, 1];
    const s = look.sunTint ?? [1, 1, 1];
    const fs = look.fogScale ?? 1;
    return {
      fogColor: new Color(a.fog_color[0] * t[0], a.fog_color[1] * t[1], a.fog_color[2] * t[2]),
      fogNear: a.fog_near_m * fs,
      fogFar: a.fog_far_m * fs,
      sunColor: new Color(a.sun_color[0] * s[0], a.sun_color[1] * s[1], a.sun_color[2] * s[2]),
      sunIntensity: a.sun_intensity * (look.sunScale ?? 1),
      ambientColor: new Color(...a.ambient_color),
      ambientIntensity: a.ambient_intensity * (look.ambientScale ?? 1),
      haze: look.haze ?? 0.4
    };
  }

  /** Applies a quality preset: the sun shadow's size, or no sun shadow. */
  setQuality(p: QualityPreset): void {
    const on = p.shadowMapSize > 0;
    this.sun.castShadow = on;
    if (on && p.shadowMapSize !== this.shadowSize) {
      this.shadowSize = p.shadowMapSize;
      this.sun.shadow.map?.dispose();
      this.sun.shadow.map = null;
      this.sun.shadow.mapSize.set(p.shadowMapSize, p.shadowMapSize);
    }
  }

  /** Moves the sun's shadow window to the focus and eases toward the focus's region look. */
  update(fx: number, fy: number, fz: number, dtMs: number): void {
    const name = regionAt(this.regions, fx, fz)?.name ?? null;
    if (name !== this.region || this.snapNext) {
      this.region = name;
      this.goal = this.compute((name && REGION_LOOKS[name]) || NEUTRAL_LOOK);
    }
    const k = this.snapNext ? 1 : 1 - Math.exp(-Math.min(dtMs, 100) / EASE_MS);
    this.snapNext = false;
    const l = this.look;
    const g = this.goal;
    l.fogColor.lerp(g.fogColor, k);
    l.sunColor.lerp(g.sunColor, k);
    l.ambientColor.lerp(g.ambientColor, k);
    l.fogNear += (g.fogNear - l.fogNear) * k;
    l.fogFar += (g.fogFar - l.fogFar) * k;
    l.sunIntensity += (g.sunIntensity - l.sunIntensity) * k;
    l.ambientIntensity += (g.ambientIntensity - l.ambientIntensity) * k;
    l.haze += (g.haze - l.haze) * k;
    this.apply();

    // Snap the shadow window to whole texels so shadows don't shimmer as the focus moves.
    const texel = (SHADOW_HALF * 2) / Math.max(this.shadowSize, 1);
    const sx = Math.round(fx / texel) * texel;
    const sz = Math.round(fz / texel) * texel;
    this.target.position.set(sx, fy, sz);
    this.sun.position.set(sx - this.sunDir.x * SUN_DISTANCE, fy - this.sunDir.y * SUN_DISTANCE, sz - this.sunDir.z * SUN_DISTANCE);
    this.target.updateMatrixWorld();
  }

  /** Writes the current look into the Three objects and the shared fog uniforms. */
  private apply(): void {
    const l = this.look;
    this.fog.color.copy(l.fogColor);
    this.fog.near = l.fogNear;
    this.fog.far = l.fogFar;
    this.hemi.color.copy(l.ambientColor);
    // The ground bounce: the ambient color, darker and a touch warmer.
    this.hemi.groundColor.copy(l.ambientColor).multiplyScalar(0.55).lerp(new Color(0.55, 0.48, 0.38), 0.25);
    this.hemi.intensity = l.ambientIntensity * AMBIENT_GAIN;
    this.sun.color.copy(l.sunColor);
    this.sun.intensity = l.sunIntensity * SUN_GAIN;
    heightFogUniforms.lfFog.value.z = l.haze;
  }

  /** The current fog color, for the sky's horizon. */
  get fogColor(): Color {
    return this.look.fogColor;
  }

  get sunDirection(): Vector3 {
    return this.sunDir;
  }

  get sunColor(): Color {
    return this.look.sunColor;
  }

  dispose(): void {
    this.scene.remove(this.hemi, this.sun, this.target);
    this.sun.shadow.map?.dispose();
    if (this.scene.fog === this.fog) this.scene.fog = null;
  }
}
