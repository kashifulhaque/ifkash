// The procedural model framework: a small bone hierarchy (plain Object3D joints) with
// primitive meshes attached, posed by Euler offsets from the rest pose. Models are built in
// code, so they need no art tools or asset files. Spec: docs/game/07-presentation.md,
// "Procedural models". Render-only: nothing here touches simulation state.

import {
  BoxGeometry,
  BufferGeometry,
  Bone,
  Color,
  ConeGeometry,
  CylinderGeometry,
  Group,
  Float32BufferAttribute,
  Matrix4,
  Mesh,
  MeshStandardMaterial,
  Object3D,
  Skeleton,
  SkinnedMesh,
  SphereGeometry,
  TorusGeometry,
  Uint16BufferAttribute
} from 'three';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';

export type Vec3 = [number, number, number];

/**
 * A pose: for each bone, an Euler rotation in radians added to the rest pose, optionally
 * followed by a position offset in meters (so 3 or 6 numbers). Bones that are absent stay at
 * rest. Rotation order is XYZ: x pitches (positive tips the bone's +Y toward +Z), z rolls.
 */
export type Pose = Record<string, number[]>;

export interface MeshOpts {
  /** Rotation of the mesh inside its bone, radians. */
  rot?: Vec3;
  /** Emissive strength; the mesh glows in its own color. */
  glow?: number;
  /** Shared material key: meshes with the same key and color share one material. */
  key?: string;
  /** Base opacity below 1 for glass and the like. A cloaked model multiplies it down further. */
  alpha?: number;
}

export class Rig {
  readonly root = new Group();
  readonly bones = new Map<string, Object3D>();
  private rest = new Map<string, { r: Vec3; p: Vec3 }>();
  private geoms: BufferGeometry[] = [];
  private mats: MeshStandardMaterial[] = [];
  private glowMats: Array<{ mat: MeshStandardMaterial; base: number }> = [];
  private alphaOf = new Map<MeshStandardMaterial, number>();
  /** Plain meshes (no glow, no alpha) that `batch()` may merge, with the bone each hangs from. */
  private plain: Array<{ mesh: Mesh; bone: Object3D; material: MeshStandardMaterial }> = [];
  private batched: SkinnedMesh | null = null;

  /** Adds a bone under `parent` (or the root) at a rest position. */
  bone(name: string, parent: string | null, pos: Vec3 = [0, 0, 0]): this {
    const b = new Object3D();
    b.name = name;
    b.position.set(...pos);
    (parent ? this.bones.get(parent)! : this.root).add(b);
    this.bones.set(name, b);
    this.rest.set(name, { r: [0, 0, 0], p: pos });
    return this;
  }

  /** Sets a bone's rest rotation (for splayed limbs, say). */
  restRot(name: string, r: Vec3): this {
    this.bones.get(name)!.rotation.set(...r);
    this.rest.get(name)!.r = r;
    return this;
  }

  private material(color: number | string, opts: MeshOpts): MeshStandardMaterial {
    const mat = new MeshStandardMaterial({ color: new Color(color), flatShading: true, roughness: 0.85, metalness: 0.05 });
    if (opts.glow) {
      mat.emissive = new Color(color);
      mat.emissiveIntensity = opts.glow;
      this.glowMats.push({ mat, base: opts.glow });
    }
    if (opts.alpha !== undefined && opts.alpha < 1) {
      mat.transparent = true;
      mat.opacity = opts.alpha;
      mat.depthWrite = false;
      this.alphaOf.set(mat, opts.alpha);
    }
    this.mats.push(mat);
    return mat;
  }

  private add(bone: string, geom: BufferGeometry, pos: Vec3, color: number | string, opts: MeshOpts): Mesh {
    this.geoms.push(geom);
    const m = new Mesh(geom, this.material(color, opts));
    m.position.set(...pos);
    if (opts.rot) m.rotation.set(...opts.rot);
    this.bones.get(bone)!.add(m);
    if (!opts.glow && !(opts.alpha !== undefined && opts.alpha < 1)) {
      this.plain.push({ mesh: m, bone: this.bones.get(bone)!, material: m.material as MeshStandardMaterial });
    }
    return m;
  }

  /**
   * Merges every plain mesh (all but the glowing and the see-through ones) into one
   * `SkinnedMesh`, each part weighted fully to its own bone. The model poses and moves exactly
   * as before, but a dozen draw calls become one (WP-081). Call it once, after the model is
   * built and before it is posed or parented, and only for a model whose meshes are never
   * hidden or moved on their own: a bone's visibility or a mesh's own transform is not
   * carried into the merged mesh. The meshes on the bones named in `keep` stay separate, for
   * a tick hook that hides them. Returns how many meshes it merged.
   */
  batch(keep: readonly string[] = []): number {
    const kept = new Set(keep.map((n) => this.bones.get(n)).filter((b): b is Object3D => !!b));
    const plain = this.plain.filter((p) => !kept.has(p.bone));
    if (this.batched || plain.length < 2) return 0;
    this.root.updateMatrixWorld(true);
    const bones = [...this.bones.values()];
    const boneIndex = new Map<Object3D, number>(bones.map((b, i) => [b, i]));
    const rootInv = new Matrix4().copy(this.root.matrixWorld).invert();
    const parts: BufferGeometry[] = [];
    const m = new Matrix4();
    for (const { mesh, bone, material } of plain) {
      const g = mesh.geometry.clone();
      g.deleteAttribute('uv');
      g.applyMatrix4(m.multiplyMatrices(rootInv, mesh.matrixWorld));
      const n = g.getAttribute('position').count;
      const color = new Float32Array(n * 3);
      const skinIndex = new Uint16Array(n * 4);
      const skinWeight = new Float32Array(n * 4);
      const bi = boneIndex.get(bone)!;
      for (let i = 0; i < n; i++) {
        color[i * 3] = material.color.r;
        color[i * 3 + 1] = material.color.g;
        color[i * 3 + 2] = material.color.b;
        skinIndex[i * 4] = bi;
        skinWeight[i * 4] = 1;
      }
      g.setAttribute('color', new Float32BufferAttribute(color, 3));
      g.setAttribute('skinIndex', new Uint16BufferAttribute(skinIndex, 4));
      g.setAttribute('skinWeight', new Float32BufferAttribute(skinWeight, 4));
      parts.push(g);
    }
    const merged = mergeGeometries(parts, false);
    for (const g of parts) g.dispose();
    if (!merged) return 0;
    const proto = plain[0].material;
    const material = new MeshStandardMaterial({
      vertexColors: true,
      flatShading: true,
      roughness: proto.roughness,
      metalness: proto.metalness
    });
    const skinned = new SkinnedMesh(merged, material);
    this.root.add(skinned);
    this.root.updateMatrixWorld(true);
    // The skeleton only reads each joint's world matrix, so plain `Object3D` joints work as bones.
    skinned.bind(new Skeleton(bones as unknown as Bone[], bones.map((b) => new Matrix4().copy(b.matrixWorld).invert())), skinned.matrixWorld);
    // Limbs swing past the rest pose's bounds, and the whole body tips over when it dies.
    skinned.computeBoundingSphere();
    if (skinned.boundingSphere) skinned.boundingSphere.radius *= 1.6;
    const count = plain.length;
    for (const { mesh, material: old } of plain) {
      mesh.removeFromParent();
      const gi = this.geoms.indexOf(mesh.geometry);
      if (gi >= 0) this.geoms.splice(gi, 1);
      mesh.geometry.dispose();
      const mi = this.mats.indexOf(old);
      if (mi >= 0) this.mats.splice(mi, 1);
      old.dispose();
    }
    this.plain = this.plain.filter((p) => kept.has(p.bone));
    this.geoms.push(merged);
    this.mats.push(material);
    this.batched = skinned;
    return count;
  }

  box(bone: string, size: Vec3, pos: Vec3, color: number | string, opts: MeshOpts = {}): Mesh {
    return this.add(bone, new BoxGeometry(...size), pos, color, opts);
  }

  sphere(bone: string, radius: number, pos: Vec3, color: number | string, opts: MeshOpts & { squash?: Vec3 } = {}): Mesh {
    const m = this.add(bone, new SphereGeometry(radius, 10, 8), pos, color, opts);
    if (opts.squash) m.scale.set(...opts.squash);
    return m;
  }

  /** A cylinder along the bone's Y axis (tapered when the radii differ). */
  cyl(bone: string, rTop: number, rBottom: number, height: number, pos: Vec3, color: number | string, opts: MeshOpts = {}): Mesh {
    return this.add(bone, new CylinderGeometry(rTop, rBottom, height, 8), pos, color, opts);
  }

  cone(bone: string, radius: number, height: number, pos: Vec3, color: number | string, opts: MeshOpts = {}): Mesh {
    return this.add(bone, new ConeGeometry(radius, height, 7), pos, color, opts);
  }

  /** A partial ring in the bone's XY plane (a bow's limbs, a helmet's brow). */
  arc(bone: string, radius: number, tube: number, arcRad: number, pos: Vec3, color: number | string, opts: MeshOpts = {}): Mesh {
    return this.add(bone, new TorusGeometry(radius, tube, 5, 14, arcRad), pos, color, opts);
  }

  /** Sets every bone to `pose` over its rest pose. */
  apply(pose: Pose): void {
    for (const [name, bone] of this.bones) {
      const rest = this.rest.get(name)!;
      const p = pose[name];
      if (!p) {
        bone.rotation.set(rest.r[0], rest.r[1], rest.r[2]);
        bone.position.set(rest.p[0], rest.p[1], rest.p[2]);
        continue;
      }
      bone.rotation.set(rest.r[0] + (p[0] ?? 0), rest.r[1] + (p[1] ?? 0), rest.r[2] + (p[2] ?? 0));
      bone.position.set(rest.p[0] + (p[3] ?? 0), rest.p[1] + (p[4] ?? 0), rest.p[2] + (p[5] ?? 0));
    }
  }

  /** Scales every glowing material's emissive strength (lanterns flaring, eyes burning). */
  setGlow(scale: number): void {
    for (const g of this.glowMats) g.mat.emissiveIntensity = g.base * scale;
  }

  private opacity = 1;

  /**
   * Draws the whole model at `o` (0 to 1) opacity: faint for a cloaked enemy. Materials turn
   * transparent only while `o` is below 1, so an ordinary model keeps its opaque sorting.
   */
  setOpacity(o: number): void {
    if (o === this.opacity) return;
    this.opacity = o;
    for (const m of this.mats) {
      const base = this.alphaOf.get(m) ?? 1;
      m.transparent = o * base < 1;
      m.opacity = o * base;
      m.depthWrite = o * base >= 1;
      m.needsUpdate = true;
    }
  }

  dispose(): void {
    // The skeleton owns a bone-matrix texture on the GPU, which `geometry.dispose()` doesn't free.
    this.batched?.skeleton.dispose();
    for (const g of this.geoms) g.dispose();
    for (const m of this.mats) m.dispose();
    this.geoms = [];
    this.mats = [];
  }
}
