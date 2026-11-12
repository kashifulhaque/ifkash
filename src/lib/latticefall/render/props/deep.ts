// Hollowdeep and the Cinderdeep Mine's props (WP-062): mine carts on their rails, and the Root
// Stair's trapdoor. Greybox, drawn from the level manifest's `Cart` and `Trapdoor` entities and the
// core's state words (a cart's distance along its rail in meters, a trapdoor's 1 when open).
// Read-only: nothing here writes simulation state. The art pass swaps the geometry, not the
// plumbing, the same as the other props.

import {
  BoxGeometry,
  Color,
  CylinderGeometry,
  Euler,
  Group,
  Mesh,
  MeshStandardMaterial,
  type BufferGeometry,
  type Material,
  type Object3D
} from 'three';

type V3 = [number, number, number];

/** Cart body, in meters: the core's boarding reach assumes half this length (`CART_HALF_LENGTH_M`). */
const CART_LENGTH = 2.0;
const CART_WIDTH = 1.1;
const CART_HEIGHT = 0.6;
/** Gauge: the rails are this far apart, and a sleeper lies every `SLEEPER_GAP` meters. */
const GAUGE = 1.0;
const SLEEPER_GAP = 1.4;
/** Seconds-ish for a trapdoor to slide, and for a cart to catch up to its word. */
const SLIDE_MS = 220;
const CART_MS = 50;

interface Cart {
  group: Group;
  rail: V3[];
  /** Cumulative length at each rail point, in meters. */
  at: number[];
  s: number;
}

interface Hatch {
  slab: Mesh;
  x0: number;
  slide: number;
  x: number;
}

function ease(cur: number, goal: number, dt: number, ms: number): number {
  return cur + (goal - cur) * (1 - Math.exp(-dt / ms));
}

export class DeepProps {
  private geometries: BufferGeometry[] = [];
  private materials: Material[] = [];
  private carts: Cart[] = [];
  private hatches: Hatch[] = [];
  readonly group = new Group();

  constructor(parent: Object3D) {
    this.group.name = 'latticefall-deep';
    parent.add(this.group);
  }

  get cartCount(): number {
    return this.carts.length;
  }

  get hatchCount(): number {
    return this.hatches.length;
  }

  add(kind: 'Cart' | 'Trapdoor', d: Record<string, unknown>): void {
    if (kind === 'Cart') this.cart(d);
    else this.trapdoor(d);
  }

  /**
   * Moves the props to the state words. `cartAt` and `hatchAt` are the offsets of the first cart
   * word and the first trapdoor word in `state`.
   */
  update(dtMs: number, state: ArrayLike<number>, cartAt: number, hatchAt: number): void {
    this.carts.forEach((c, i) => {
      const word = state[cartAt + i];
      if (word === undefined) return;
      c.s = ease(c.s, word, dtMs, CART_MS);
      // A big jump (a ride back, a saved game) shouldn't slide the length of the rail.
      if (Math.abs(c.s - word) > 20) c.s = word;
      this.place(c);
    });
    this.hatches.forEach((h, i) => {
      const word = state[hatchAt + i];
      if (word === undefined) return;
      h.x = ease(h.x, word > 0.5 ? h.slide : 0, dtMs, SLIDE_MS);
      h.slab.position.x = h.x0 + h.x;
    });
  }

  dispose(): void {
    this.group.parent?.remove(this.group);
    for (const g of this.geometries) g.dispose();
    for (const m of this.materials) m.dispose();
    this.geometries = [];
    this.materials = [];
    this.carts = [];
    this.hatches = [];
  }

  // -------------------------------------------------------------------------------------

  private geo<T extends BufferGeometry>(g: T): T {
    this.geometries.push(g);
    return g;
  }

  private mat(color: string): MeshStandardMaterial {
    const m = new MeshStandardMaterial({ color: new Color(color), roughness: 0.8, metalness: 0.2 });
    this.materials.push(m);
    return m;
  }

  /** The cart on its rail, with the rails and sleepers drawn from the manifest's points. */
  private cart(d: Record<string, unknown>): void {
    const rail = d.rail as V3[];
    const at = [0];
    for (let i = 1; i < rail.length; i++) at.push(at[i - 1] + this.dist(rail[i - 1], rail[i]));
    const iron = this.mat('#4a4036');
    const timber = this.mat('#6b5238');
    const ore = this.mat('#3a3a40');
    // The rails and sleepers are fixed to the world: one group per segment.
    for (let i = 1; i < rail.length; i++) {
      const [a, b] = [rail[i - 1], rail[i]];
      const seg = new Group();
      const len = at[i] - at[i - 1];
      const { yaw, pitch } = this.aim(a, b);
      seg.position.set(a[0], a[1], a[2]);
      seg.rotation.copy(new Euler(-pitch, yaw, 0, 'YXZ'));
      const bar = this.geo(new BoxGeometry(0.08, 0.08, len));
      for (const x of [-GAUGE / 2, GAUGE / 2]) {
        const r = new Mesh(bar, iron);
        r.position.set(x, 0.1, len / 2);
        seg.add(r);
      }
      const tie = this.geo(new BoxGeometry(GAUGE + 0.5, 0.06, 0.18));
      for (let t = SLEEPER_GAP / 2; t < len; t += SLEEPER_GAP) {
        const s = new Mesh(tie, timber);
        s.position.set(0, 0.05, t);
        seg.add(s);
      }
      this.group.add(seg);
    }
    // The cart: a box with an ore heap and four wheels. Local +Z is the way the rail runs.
    const g = new Group();
    const body = new Mesh(this.geo(new BoxGeometry(CART_WIDTH, CART_HEIGHT, CART_LENGTH)), iron);
    body.position.y = 0.55;
    const heap = new Mesh(this.geo(new BoxGeometry(CART_WIDTH - 0.2, 0.25, CART_LENGTH - 0.3)), ore);
    heap.position.y = 0.95;
    g.add(body, heap);
    const wheel = this.geo(new CylinderGeometry(0.2, 0.2, 0.08, 12));
    for (const x of [-GAUGE / 2, GAUGE / 2]) {
      for (const z of [-0.6, 0.6]) {
        const w = new Mesh(wheel, timber);
        w.rotation.z = Math.PI / 2;
        w.position.set(x, 0.22, z);
        g.add(w);
      }
    }
    this.group.add(g);
    const cart: Cart = { group: g, rail, at, s: 0 };
    this.carts.push(cart);
    this.place(cart);
  }

  private trapdoor(d: Record<string, unknown>): void {
    const pos = d.pos as V3;
    const [hx, hz] = d.half_extents as [number, number];
    const slab = new Mesh(this.geo(new BoxGeometry(hx * 2, 0.3, hz * 2)), this.mat('#6b5238'));
    slab.position.set(pos[0], pos[1] + 0.15, pos[2]);
    this.group.add(slab);
    // It slides toward -x by twice its half width when it opens (the core's `trapdoor.rs`).
    this.hatches.push({ slab, x0: pos[0], slide: -hx * 2, x: 0 });
  }

  private dist(a: V3, b: V3): number {
    return Math.hypot(b[0] - a[0], b[1] - a[1], b[2] - a[2]);
  }

  /** The yaw (from +Z toward +X) and upward pitch of the way from `a` to `b`. */
  private aim(a: V3, b: V3): { yaw: number; pitch: number } {
    const dx = b[0] - a[0];
    const dz = b[2] - a[2];
    return { yaw: Math.atan2(dx, dz), pitch: Math.atan2(b[1] - a[1], Math.hypot(dx, dz)) };
  }

  /** Puts a cart `c.s` meters along its rail, facing the way the rail runs there. */
  private place(c: Cart): void {
    const n = c.rail.length;
    let i = 1;
    while (i < n - 1 && c.s > c.at[i]) i++;
    const [a, b] = [c.rail[i - 1], c.rail[i]];
    const len = c.at[i] - c.at[i - 1];
    const t = len > 0 ? Math.min(1, Math.max(0, (c.s - c.at[i - 1]) / len)) : 0;
    const { yaw, pitch } = this.aim(a, b);
    c.group.position.set(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t);
    c.group.rotation.copy(new Euler(-pitch, yaw, 0, 'YXZ'));
  }
}
