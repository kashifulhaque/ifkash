// A tiny helper for building equipment out of Three.js primitives. Everything is flat-shaded
// and low-poly, like the procedural enemy models (docs/game/07-presentation.md, "Procedural
// models"). The builder remembers what it made so `dispose()` frees every geometry and
// material. Render-only: nothing here touches simulation state.

import {
  BoxGeometry,
  BufferGeometry,
  Color,
  ConeGeometry,
  CylinderGeometry,
  ExtrudeGeometry,
  Group,
  Mesh,
  MeshStandardMaterial,
  OctahedronGeometry,
  Shape,
  SphereGeometry,
  TorusGeometry
} from 'three';

export type V3 = [number, number, number];

export interface PartOpts {
  rot?: V3;
  /** Emissive strength: the part glows in its own color. */
  glow?: number;
  metal?: number;
  rough?: number;
  scale?: V3;
}

export class Builder {
  readonly group = new Group();
  private geoms: BufferGeometry[] = [];
  private mats = new Map<string, MeshStandardMaterial>();

  private material(color: number, o: PartOpts): MeshStandardMaterial {
    const key = `${color}|${o.glow ?? 0}|${o.metal ?? 0}|${o.rough ?? 0}`;
    let m = this.mats.get(key);
    if (!m) {
      m = new MeshStandardMaterial({
        color: new Color(color),
        flatShading: true,
        roughness: o.rough ?? 0.7,
        metalness: o.metal ?? 0.05
      });
      if (o.glow) {
        m.emissive = new Color(color);
        m.emissiveIntensity = o.glow;
      }
      this.mats.set(key, m);
    }
    return m;
  }

  private add(parent: Group, geom: BufferGeometry, pos: V3, color: number, o: PartOpts): Mesh {
    this.geoms.push(geom);
    const m = new Mesh(geom, this.material(color, o));
    m.position.set(...pos);
    if (o.rot) m.rotation.set(...o.rot);
    if (o.scale) m.scale.set(...o.scale);
    parent.add(m);
    return m;
  }

  box(size: V3, pos: V3, color: number, o: PartOpts = {}, parent: Group = this.group): Mesh {
    return this.add(parent, new BoxGeometry(...size), pos, color, o);
  }

  /** A cylinder along Y; radii differ for a taper. */
  cyl(rTop: number, rBottom: number, h: number, pos: V3, color: number, o: PartOpts = {}, parent: Group = this.group): Mesh {
    return this.add(parent, new CylinderGeometry(rTop, rBottom, h, 8), pos, color, o);
  }

  cone(r: number, h: number, pos: V3, color: number, o: PartOpts = {}, parent: Group = this.group): Mesh {
    return this.add(parent, new ConeGeometry(r, h, 6), pos, color, o);
  }

  sphere(r: number, pos: V3, color: number, o: PartOpts = {}, parent: Group = this.group): Mesh {
    return this.add(parent, new SphereGeometry(r, 9, 7), pos, color, o);
  }

  gem(r: number, pos: V3, color: number, o: PartOpts = {}, parent: Group = this.group): Mesh {
    return this.add(parent, new OctahedronGeometry(r, 0), pos, color, o);
  }

  /** A partial ring in the XY plane. */
  arc(radius: number, tube: number, arcRad: number, pos: V3, color: number, o: PartOpts = {}, parent: Group = this.group): Mesh {
    return this.add(parent, new TorusGeometry(radius, tube, 5, 14, arcRad), pos, color, o);
  }

  /** A flat outline (points in XY) extruded `depth` along Z, centered on Z. */
  shape(outline: Array<[number, number]>, depth: number, pos: V3, color: number, o: PartOpts = {}, parent: Group = this.group): Mesh {
    const s = new Shape();
    outline.forEach(([x, y], i) => (i === 0 ? s.moveTo(x, y) : s.lineTo(x, y)));
    s.closePath();
    const g = new ExtrudeGeometry(s, { depth, bevelEnabled: false });
    g.translate(0, 0, -depth / 2);
    return this.add(parent, g, pos, color, o);
  }

  /** A child group for a part that needs its own transform. */
  sub(pos: V3 = [0, 0, 0], rot: V3 = [0, 0, 0], parent: Group = this.group): Group {
    const g = new Group();
    g.position.set(...pos);
    g.rotation.set(...rot);
    parent.add(g);
    return g;
  }

  dispose(): void {
    for (const g of this.geoms) g.dispose();
    for (const m of this.mats.values()) m.dispose();
    this.geoms = [];
    this.mats.clear();
  }
}

/** A small deterministic hash of a string, 0 to 1. */
export function hash01(s: string, salt = 0): number {
  let h = 2166136261 ^ salt;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return ((h >>> 0) % 10007) / 10007;
}

/** Blends two 0xRRGGBB colors; `t` 0 gives `a`. */
export function mix(a: number, b: number, t: number): number {
  return new Color(a).lerp(new Color(b), t).getHex();
}
