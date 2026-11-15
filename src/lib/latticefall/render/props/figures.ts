// Entity props drawn from model files: the Lantern and the NPC figures. Which file each one
// uses comes from the area art manifest (`../kits/art.ts`), so an area with no art manifest
// draws what it drew before: no Lantern, and the greybox figures of `hearth.ts`. A file that
// fails to load leaves the same fallback in place and logs a warning.
//
// The models keep their own scene graph and materials. A Lantern's named nodes move; an NPC
// is a skinned figure on the Wayfarer's rig, looping one clip of the animation library
// through the same retargeting the Wayfarer uses. Read-only: nothing here writes simulation
// state.

import { Box3, Group, PointLight, PropertyBinding, type Bone, type Material, type MeshStandardMaterial, type Object3D } from 'three';
import type { GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { retargetClips, sampleClip, type BakedClip } from '../actors/retarget';
import type { ArtLantern, ArtNpc } from '../kits/art';
import { disposeScene, ModelCache } from '../kits/modelCache';
import { applyHeightFog } from '../lighting/fogPatch';

type V3 = [number, number, number];

/** The animation library the Wayfarer uses, relative to `static/latticefall/`. */
const CLIP_LIBRARY = 'actors/ual.glb';

interface Glow {
  material: MeshStandardMaterial;
  base: number;
}

interface LanternFigure {
  sway: Object3D | null;
  glows: Glow[];
  light: PointLight | null;
  lightBase: number;
  phase: number;
}

interface NpcFigure {
  bones: Bone[];
  hips: Bone;
  clip: BakedClip;
  q: Float32Array;
  hipsPos: Float32Array;
  t: number;
}

function materialsOf(root: Object3D): Set<Material> {
  const found = new Set<Material>();
  root.traverse((o) => {
    const mat = (o as { material?: Material | Material[] }).material;
    if (!mat) return;
    for (const m of Array.isArray(mat) ? mat : [mat]) found.add(m);
  });
  return found;
}

export class FigureProps {
  private cache = new ModelCache();
  private scenes: Object3D[] = [];
  private lanterns: LanternFigure[] = [];
  private npcs: NpcFigure[] = [];
  private clips: Promise<GLTF> | null = null;
  private loading = 0;
  private clock = 0;
  private disposed = false;
  readonly group = new Group();

  constructor(parent: Object3D) {
    this.group.name = 'latticefall-figures';
    parent.add(this.group);
  }

  /** How many model files are still loading. The benchmark waits for 0 before it measures. */
  get pending(): number {
    return this.loading;
  }

  private track<T>(work: Promise<T>, what: string, fallback: string): void {
    this.loading++;
    work
      .catch((err) => {
        if (!this.disposed) console.warn(`latticefall: ${what} failed to load; ${fallback}`, err);
      })
      .finally(() => this.loading--);
  }

  private async scene(path: string): Promise<Object3D> {
    const gltf = await this.cache.loadScene(path);
    this.scenes.push(gltf.scene);
    for (const m of materialsOf(gltf.scene)) applyHeightFog(m);
    return gltf.scene;
  }

  /** Draws a Lantern from its manifest entity. Before this module, Lanterns drew nothing. */
  lantern(d: Record<string, unknown>, spec: ArtLantern, file: string): void {
    const pos = d.pos as V3;
    const holder = new Group();
    holder.position.set(pos[0], pos[1], pos[2]);
    holder.rotation.y = (d.yaw as number) ?? 0;
    this.group.add(holder);
    this.track(
      this.scene(file).then((scene) => {
        scene.traverse((o) => {
          o.castShadow = true;
          o.receiveShadow = true;
        });
        holder.add(scene);
        const glows: Glow[] = [];
        for (const m of materialsOf(scene)) {
          const std = m as MeshStandardMaterial;
          // A material that emits light is the flame; it flickers.
          if (std.emissive && std.emissive.getHex() !== 0) glows.push({ material: std, base: std.emissiveIntensity });
        }
        let light: PointLight | null = null;
        if (spec.light) {
          light = new PointLight(spec.light.color, spec.light.intensity, spec.light.distance, 1.6);
          light.position.set(...spec.light.at);
          holder.add(light);
        }
        this.lanterns.push({
          sway: spec.sway ? (scene.getObjectByName(spec.sway) ?? null) : null,
          glows,
          light,
          lightBase: spec.light?.intensity ?? 0,
          phase: pos[0] * 1.3 + pos[2] * 0.7
        });
      }),
      `Lantern model ${file}`,
      'the Lantern draws nothing'
    );
  }

  /**
   * Draws a static model in `holder` in place of `greybox`, which stays until the model
   * loads (and for good if it fails). The model is authored in the holder's frame.
   */
  replace(holder: Object3D, greybox: Object3D, file: string): void {
    this.track(
      this.scene(file).then((scene) => {
        if (this.disposed) return;
        scene.traverse((o) => {
          o.receiveShadow = true;
        });
        holder.add(scene);
        greybox.visible = false;
      }),
      `model ${file}`,
      'keeping the greybox'
    );
  }

  /**
   * Draws an NPC from a model in place of `greybox`, which stays until the model and its clip
   * are ready. `holder` is the group at the NPC's feet, facing the NPC's way.
   */
  npc(holder: Object3D, greybox: Object3D, spec: ArtNpc, file: string): void {
    this.track(
      Promise.all([this.scene(file), (this.clips ??= this.cache.loadScene(CLIP_LIBRARY))]).then(([scene, library]) => {
        // Measure the rest pose with no transform on the scene, like the Wayfarer.
        scene.updateMatrixWorld(true);
        const box = new Box3().setFromObject(scene, true);
        const scale = spec.height / (box.max.y - box.min.y);
        const source = library.animations.filter((c) => c.name === spec.clip);
        if (source.length === 0) throw new Error(`animation library has no clip ${spec.clip}`);
        const rt = retargetClips(library.scene, source, scene, scale);
        const clip = rt.clips.get(spec.clip)!;
        const bones = rt.boneNames.map((n) => scene.getObjectByName(PropertyBinding.sanitizeNodeName(n)) as Bone);
        scene.position.y = -box.min.y;
        const scaled = new Group();
        scaled.scale.setScalar(scale);
        scaled.add(scene);
        // The greybox figures cast no shadows, and neither do these: each figure is a dozen
        // draw calls, and a shadow pass would double them.
        scene.traverse((o) => {
          o.receiveShadow = true;
        });
        holder.add(scaled);
        greybox.visible = false;
        this.npcs.push({
          bones,
          hips: bones[0],
          clip,
          q: new Float32Array(bones.length * 4),
          hipsPos: new Float32Array(3),
          // Start each figure at its own point in the loop, so they don't breathe in step.
          t: (Math.abs(holder.position.x * 1.7 + holder.position.z * 0.9) % 1) * clip.duration
        });
      }),
      `NPC model ${file}`,
      'keeping the greybox figure'
    );
  }

  /** Animates the figures. Call once per frame. */
  update(frameMs: number): void {
    this.clock += frameMs;
    const dt = frameMs / 1000;
    for (const l of this.lanterns) {
      const t = this.clock / 1000 + l.phase;
      const k = 0.85 + 0.15 * Math.sin(t * 9) * Math.sin(t * 5.3 + 1.7);
      for (const g of l.glows) g.material.emissiveIntensity = g.base * k;
      if (l.light) l.light.intensity = l.lightBase * k;
      if (l.sway) l.sway.rotation.z = Math.sin(t * 0.9) * 0.05;
    }
    for (const n of this.npcs) {
      n.t += dt;
      sampleClip(n.clip, n.t, true, n.q, n.hipsPos);
      for (let i = 0; i < n.bones.length; i++) n.bones[i].quaternion.fromArray(n.q, i * 4);
      n.hips.position.set(n.hipsPos[0], n.hipsPos[1], n.hipsPos[2]);
    }
  }

  dispose(): void {
    this.disposed = true;
    this.group.removeFromParent();
    for (const s of this.scenes) disposeScene(s);
    this.clips?.then((g) => disposeScene(g.scene), () => undefined);
    this.cache.dispose();
    this.scenes = [];
    this.lanterns = [];
    this.npcs = [];
  }
}
