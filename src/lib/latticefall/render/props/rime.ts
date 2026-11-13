// Rimeshelf's props (WP-066): the Warden Braziers of the Whitewarden Catacombs and the seal sigil
// of the Bound Ring. Greybox, drawn from the level manifest's `Brazier` and `Sigil` entities and the
// core's state words (each brazier, 1 once lit, then each sigil, 0 cold, 1 touched, 2 while its
// seal holds, in `lf_core::world_state::mechanics::brazier::render_words`). Read-only: nothing here
// writes simulation state. The art pass swaps the geometry, not the plumbing, the same as the
// other props.

import {
  Color,
  ConeGeometry,
  CylinderGeometry,
  DoubleSide,
  Group,
  Mesh,
  MeshBasicMaterial,
  MeshStandardMaterial,
  PointLight,
  RingGeometry,
  type BufferGeometry,
  type Material,
  type Object3D
} from 'three';

type V3 = [number, number, number];

const FLAME = new Color('#ffb347');
const EMBER = new Color('#3a2a22');
const SIGIL_COLD = new Color('#8fa3b8');
const SIGIL_TOUCHED = new Color('#d8e6f2');
const SIGIL_SEALED = new Color('#ff8a6a');

interface Brazier {
  flame: Mesh;
  light: PointLight;
  bowl: MeshStandardMaterial;
  glow: number;
}

interface Sigil {
  material: MeshBasicMaterial;
  group: Group;
}

function ease(cur: number, goal: number, dt: number, ms: number): number {
  return cur + (goal - cur) * (1 - Math.exp(-dt / ms));
}

export class RimeProps {
  private geometries: BufferGeometry[] = [];
  private materials: Material[] = [];
  private braziers: Brazier[] = [];
  private sigils: Sigil[] = [];
  private clock = 0;
  readonly group = new Group();

  constructor(parent: Object3D) {
    this.group.name = 'latticefall-rime';
    parent.add(this.group);
  }

  get brazierCount(): number {
    return this.braziers.length;
  }

  get sigilCount(): number {
    return this.sigils.length;
  }

  add(kind: 'Brazier' | 'Sigil', d: Record<string, unknown>): void {
    if (kind === 'Brazier') this.brazier(d);
    else this.sigil(d);
  }

  /** `brazierAt` and `sigilAt` are the offsets of the first brazier word and the first sigil word in `state`. */
  update(dtMs: number, state: ArrayLike<number>, brazierAt: number, sigilAt: number): void {
    this.clock += dtMs;
    this.braziers.forEach((b, i) => {
      const word = state[brazierAt + i];
      if (word === undefined) return;
      b.glow = ease(b.glow, word > 0.5 ? 1 : 0, dtMs, 140);
      const flicker = 1 + Math.sin(this.clock / 90 + i) * 0.08;
      b.flame.visible = b.glow > 0.05;
      b.flame.scale.set(b.glow * flicker, b.glow * (1 + (flicker - 1) * 2), b.glow * flicker);
      b.light.intensity = 2.2 * b.glow * flicker;
      b.bowl.emissive.copy(EMBER).lerp(FLAME, b.glow * 0.5);
    });
    this.sigils.forEach((s, i) => {
      const word = state[sigilAt + i];
      if (word === undefined) return;
      const goal = word > 1.5 ? SIGIL_SEALED : word > 0.5 ? SIGIL_TOUCHED : SIGIL_COLD;
      s.material.color.lerp(goal, 1 - Math.exp(-dtMs / 160));
      s.group.rotation.y += dtMs * 0.0003 * (word > 1.5 ? 3 : 1);
    });
  }

  dispose(): void {
    this.group.parent?.remove(this.group);
    for (const g of this.geometries) g.dispose();
    for (const m of this.materials) m.dispose();
    this.geometries = [];
    this.materials = [];
    this.braziers = [];
    this.sigils = [];
  }

  // -------------------------------------------------------------------------------------

  private geo<T extends BufferGeometry>(g: T): T {
    this.geometries.push(g);
    return g;
  }

  private mat<T extends Material>(m: T): T {
    this.materials.push(m);
    return m;
  }

  /** A Warden Brazier: an iron bowl on a stand, with a flame and a light that come up when it is lit. */
  private brazier(d: Record<string, unknown>): void {
    const pos = d.pos as V3;
    const g = new Group();
    g.position.set(pos[0], pos[1], pos[2]);
    const iron = this.mat(new MeshStandardMaterial({ color: new Color('#3b3f46'), roughness: 0.85, metalness: 0.4 }));
    const stand = new Mesh(this.geo(new CylinderGeometry(0.1, 0.18, 0.9, 8)), iron);
    stand.position.y = 0.45;
    const bowlMaterial = this.mat(
      new MeshStandardMaterial({ color: new Color('#2a2d33'), roughness: 0.8, metalness: 0.5, emissive: EMBER.clone() })
    );
    const bowl = new Mesh(this.geo(new CylinderGeometry(0.38, 0.2, 0.3, 10)), bowlMaterial);
    bowl.position.y = 1.05;
    const flameMaterial = this.mat(new MeshBasicMaterial({ color: FLAME, transparent: true, opacity: 0.9 }));
    const flame = new Mesh(this.geo(new ConeGeometry(0.28, 0.8, 8)), flameMaterial);
    flame.position.y = 1.6;
    flame.visible = false;
    const light = new PointLight(FLAME, 0, 9, 1.6);
    light.position.y = 1.6;
    g.add(stand, bowl, flame, light);
    this.group.add(g);
    this.braziers.push({ flame, light, bowl: bowlMaterial, glow: 0 });
  }

  /** A seal sigil: a ring of light on the floor that goes white when touched and red when it seals. */
  private sigil(d: Record<string, unknown>): void {
    const pos = d.pos as V3;
    const group = new Group();
    group.position.set(pos[0], pos[1] + 0.04, pos[2]);
    const material = this.mat(new MeshBasicMaterial({ color: SIGIL_COLD.clone(), side: DoubleSide, transparent: true, opacity: 0.85 }));
    const outer = new Mesh(this.geo(new RingGeometry(1.0, 1.25, 24)), material);
    outer.rotation.x = -Math.PI / 2;
    const inner = new Mesh(this.geo(new RingGeometry(0.45, 0.6, 6)), material);
    inner.rotation.x = -Math.PI / 2;
    group.add(outer, inner);
    this.group.add(group);
    this.sigils.push({ material, group });
  }
}
