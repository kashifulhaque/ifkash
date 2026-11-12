// The procedural model framework: a small bone hierarchy (plain Object3D joints) with
// primitive meshes attached, posed by Euler offsets from the rest pose. Models are built in
// code, so they need no art tools or asset files. Spec: docs/game/07-presentation.md,
// "Procedural models". Render-only: nothing here touches simulation state.

import {
  BoxGeometry,
  BufferGeometry,
  Color,
  ConeGeometry,
  CylinderGeometry,
  Group,
  Mesh,
  MeshStandardMaterial,
  Object3D,
  SphereGeometry,
  TorusGeometry
} from 'three';

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
    return m;
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
    for (const g of this.geoms) g.dispose();
    for (const m of this.mats) m.dispose();
    this.geoms = [];
    this.mats = [];
  }
}
