// Hollowhearth's people and stations (WP-068): greybox figures for the five NPCs, and the small
// props that make each station read at a glance (candles, the Hearth's fire, the open Ledger, the
// Mirror's glass, the keepsakes). Drawn from the level manifest's `Npc` and `Station` entities.
// Read-only: nothing here writes simulation state. The art pass swaps the geometry, not the
// plumbing, the same as the other props.

import {
  BoxGeometry,
  Color,
  ConeGeometry,
  CylinderGeometry,
  DoubleSide,
  Group,
  Mesh,
  MeshStandardMaterial,
  PlaneGeometry,
  PointLight,
  SphereGeometry,
  type BufferGeometry,
  type Material,
  type Object3D
} from 'three';

type V3 = [number, number, number];

/** What distinguishes each NPC: a robe color and the thing they carry. */
const LOOKS: Record<string, { robe: string; trim: string; carry: 'book' | 'map' | 'hammer' | 'satchel' | 'pack' }> = {
  npc_archivist: { robe: '#667a63', trim: '#d8c58a', carry: 'book' },
  npc_cartographer: { robe: '#9c5a35', trim: '#e6d6a8', carry: 'map' },
  npc_smith: { robe: '#44413f', trim: '#a95b2f', carry: 'hammer' },
  npc_courier: { robe: '#3f6c8a', trim: '#e9e2cf', carry: 'satchel' },
  npc_peddler: { robe: '#7b6a3c', trim: '#c97b4a', carry: 'pack' }
};

const FALLBACK = { robe: '#6b6558', trim: '#d8c58a', carry: 'book' as const };

export class HearthProps {
  private geometries: BufferGeometry[] = [];
  private materials: Material[] = [];
  private flames: Array<{ mat: MeshStandardMaterial; light: PointLight | null; base: number; phase: number }> = [];
  private clock = 0;
  readonly group = new Group();

  constructor(parent: Object3D) {
    this.group.name = 'latticefall-hearth';
    parent.add(this.group);
  }

  /** Adds an NPC or station from its manifest entity. */
  add(kind: 'Npc' | 'Station', d: Record<string, unknown>): void {
    const pos = d.pos as V3;
    const yaw = d.yaw as number;
    const g = new Group();
    g.position.set(pos[0], pos[1], pos[2]);
    g.rotation.y = yaw;
    if (kind === 'Npc') this.npc(g, String(d.npc));
    else this.station(g, String(d.station));
    this.group.add(g);
  }

  /** Flickers the flames. Call once per frame. */
  update(frameMs: number): void {
    this.clock += frameMs;
    for (const f of this.flames) {
      const t = this.clock / 1000 + f.phase;
      const k = 0.85 + 0.15 * Math.sin(t * 9) * Math.sin(t * 5.3 + 1.7);
      f.mat.emissiveIntensity = f.base * k;
      if (f.light) f.light.intensity = f.base * 1.4 * k;
    }
  }

  dispose(): void {
    this.group.removeFromParent();
    for (const g of this.geometries) g.dispose();
    for (const m of this.materials) m.dispose();
    this.geometries = [];
    this.materials = [];
    this.flames = [];
  }

  // -------------------------------------------------------------------------------------

  private geo<T extends BufferGeometry>(g: T): T {
    this.geometries.push(g);
    return g;
  }

  private mat(color: string | Color, extra: Partial<ConstructorParameters<typeof MeshStandardMaterial>[0]> = {}): MeshStandardMaterial {
    const m = new MeshStandardMaterial({ color, roughness: 0.85, ...extra });
    this.materials.push(m);
    return m;
  }

  private mesh(g: BufferGeometry, m: Material, at: V3, parent: Object3D): Mesh {
    const mesh = new Mesh(this.geo(g), m);
    mesh.position.set(at[0], at[1], at[2]);
    parent.add(mesh);
    return mesh;
  }

  /** A candle: a wax stub with a flame, and (optionally) a light. */
  private candle(parent: Object3D, at: V3, light: { color: string; distance: number } | null, base = 1.4): void {
    const wax = this.mat('#e9e2cf');
    this.mesh(new CylinderGeometry(0.035, 0.04, 0.16, 8), wax, [at[0], at[1] + 0.08, at[2]], parent);
    const flame = this.mat('#ffb347', { emissive: new Color('#ff9a2e'), emissiveIntensity: base });
    this.mesh(new SphereGeometry(0.035, 8, 6), flame, [at[0], at[1] + 0.2, at[2]], parent);
    let pl: PointLight | null = null;
    if (light) {
      pl = new PointLight(light.color, base * 1.4, light.distance, 1.6);
      pl.position.set(at[0], at[1] + 0.5, at[2]);
      parent.add(pl);
    }
    this.flames.push({ mat: flame, light: pl, base, phase: at[0] * 1.3 + at[2] * 0.7 });
  }

  private npc(g: Group, id: string): void {
    const look = LOOKS[id] ?? FALLBACK;
    const robe = this.mat(look.robe);
    const trim = this.mat(look.trim);
    const skin = this.mat('#d9b48f');
    // Robe, shoulders, hood and face: a simple tapering figure about 1.7 m tall.
    this.mesh(new CylinderGeometry(0.26, 0.46, 1.2, 14), robe, [0, 0.6, 0], g);
    this.mesh(new SphereGeometry(0.29, 14, 10), robe, [0, 1.22, 0], g).scale.set(1, 0.7, 0.8);
    this.mesh(new SphereGeometry(0.2, 14, 12), robe, [0, 1.5, -0.02], g);
    this.mesh(new SphereGeometry(0.145, 12, 10), skin, [0, 1.49, 0.07], g);
    // A sash, and arms held forward.
    this.mesh(new CylinderGeometry(0.31, 0.33, 0.07, 14), trim, [0, 0.82, 0], g);
    for (const x of [-0.3, 0.3]) {
      const arm = this.mesh(new CylinderGeometry(0.06, 0.075, 0.55, 8), robe, [x, 1.0, 0.18], g);
      arm.rotation.x = Math.PI / 2.6;
    }
    switch (look.carry) {
      case 'book': {
        const b = this.mesh(new BoxGeometry(0.3, 0.06, 0.22), this.mat('#8a4a38'), [0, 1.02, 0.38], g);
        b.rotation.x = -0.25;
        break;
      }
      case 'map': {
        const m = this.mesh(new CylinderGeometry(0.045, 0.045, 0.5, 8), this.mat('#e6d6a8'), [0.05, 1.02, 0.4], g);
        m.rotation.z = Math.PI / 2;
        break;
      }
      case 'hammer': {
        const h = new Group();
        this.mesh(new CylinderGeometry(0.025, 0.025, 0.7, 6), this.mat('#6f4e37'), [0, 0, 0], h);
        this.mesh(new BoxGeometry(0.2, 0.12, 0.12), this.mat('#5a5a5f', { metalness: 0.5 }), [0, 0.36, 0], h);
        h.position.set(0.36, 0.95, 0.3);
        h.rotation.x = 0.5;
        g.add(h);
        this.mesh(new BoxGeometry(0.5, 0.7, 0.05), this.mat('#6b4a32'), [0, 0.8, 0.27], g);
        break;
      }
      case 'satchel':
        this.mesh(new BoxGeometry(0.32, 0.26, 0.12), this.mat('#8a6a42'), [0.32, 0.72, 0.02], g);
        this.mesh(new CylinderGeometry(0.21, 0.21, 0.05, 12), this.mat('#3f6c8a'), [0, 1.66, 0], g);
        break;
      case 'pack': {
        this.mesh(new BoxGeometry(0.62, 0.7, 0.34), this.mat('#8a6a42'), [0, 1.0, -0.36], g);
        this.mesh(new BoxGeometry(0.5, 0.25, 0.3), this.mat('#6b4a32'), [0, 1.5, -0.36], g);
        this.mesh(new CylinderGeometry(0.02, 0.02, 1.9, 6), this.mat('#6f4e37'), [-0.42, 0.95, -0.2], g);
        break;
      }
    }
    // A candle at the NPC's counter side, lit, so each person stands in a small pool of light.
    this.candle(g, [0.9, 1.0, 1.4], { color: '#ffb066', distance: 7 }, 1.2);
  }

  private station(g: Group, id: string): void {
    switch (id) {
      case 'stn_hearth': {
        // The central fire: stacked logs under a tall flame, and the room's main light.
        const flame = this.mat('#ffb347', { emissive: new Color('#ff8a1f'), emissiveIntensity: 2.2, side: DoubleSide });
        this.mesh(new ConeGeometry(0.55, 1.5, 10), flame, [0, 2.15, 2.6], g);
        const light = new PointLight('#ff9a40', 3.2, 38, 1.2);
        light.position.set(0, 2.6, 2.6);
        g.add(light);
        this.flames.push({ mat: flame, light, base: 2.2, phase: 0 });
        break;
      }
      case 'stn_archive':
        // A sconce on the shelves' face (local +Z points at the shelves).
        this.candle(g, [0, 2.0, 1.3], { color: '#ffc27a', distance: 10 }, 1.4);
        break;
      case 'stn_ledger': {
        // The Ledger lies open on its stand: two pale pages under a candle.
        const page = this.mat('#efe6cf', { emissive: new Color('#c9a64a'), emissiveIntensity: 0.18, side: DoubleSide });
        for (const s of [-1, 1]) {
          const p = this.mesh(new PlaneGeometry(0.42, 0.55), page, [s * 0.22, 1.28, -1.6], g);
          p.rotation.set(-Math.PI / 2 + 0.35, 0, s * 0.12);
        }
        this.candle(g, [0.55, 1.2, -1.45], { color: '#ffc27a', distance: 8 }, 1.3);
        break;
      }
      case 'stn_mirror': {
        // Glass in the root frame: a pale, shiny plane with a faint cool light.
        const glass = this.mat('#cfd9de', { metalness: 0.9, roughness: 0.12, emissive: new Color('#7d98a8'), emissiveIntensity: 0.25 });
        this.mesh(new PlaneGeometry(2.6, 3.0), glass, [0, 1.6, -1.95], g);
        this.candle(g, [1.6, 0.0, -1.8], { color: '#ffc27a', distance: 8 }, 1.2);
        break;
      }
      case 'stn_keepsakes': {
        // A scatter of small things on the table's top.
        const colors = ['#a4623a', '#667a63', '#3f6c8a', '#c9a64a'];
        colors.forEach((c, i) => {
          const h = 0.14 + i * 0.03;
          this.mesh(new BoxGeometry(0.22, h, 0.22), this.mat(c), [(i - 1.5) * 0.6, 0.9 + h / 2, 2.4 + ((i % 2) - 0.5) * 0.4], g);
        });
        this.candle(g, [0, 0.9, 1.9], { color: '#ffc27a', distance: 7 }, 1.2);
        break;
      }
    }
  }
}
