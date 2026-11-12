// World-mechanic props (WP-042): ladders, lifts, doors, levers, hollow walls, pickups,
// chests, messages, pools, snow, and fog gates, drawn from the level manifest's `entities`
// list and the core's `mechanics_state()` words. Read-only: nothing here writes simulation
// state. The props are greybox shapes; the art pass swaps the geometry, not the plumbing.
//
// The manifest lists entities in scenario order, and the core's state words follow the same
// order per kind (see `lf_core::world_state::mechanics::render_state`), so the n-th Door in
// the manifest is the n-th door word.

import {
  BoxGeometry,
  Color,
  CylinderGeometry,
  DoubleSide,
  Group,
  Mesh,
  MeshStandardMaterial,
  OctahedronGeometry,
  PlaneGeometry,
  type BufferGeometry,
  type Material,
  type Object3D,
  type Scene
} from 'three';
import { manifestUrl } from '../kits/manifest';
import { HearthProps } from './hearth';
import { createPoolWater, type PoolData } from '../water';

type V3 = [number, number, number];

interface Entity {
  [kind: string]: Record<string, unknown>;
}

/** Entity kinds in the order of the core's state words. */
const STATE_KINDS = ['Ladder', 'Lift', 'Door', 'Lever', 'HollowWall', 'Pickup', 'Chest', 'Message', 'Pool'] as const;
type StateKind = (typeof STATE_KINDS)[number];

/** A prop's per-frame update. `v` is its state word. */
type Updater = (v: number, dt: number) => void;

/** Reads a CSS custom property from `<html>`, with a fallback. */
function token(name: string, fallback: string): Color {
  try {
    const raw = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    if (raw) return new Color(raw);
  } catch {
    // Fall through.
  }
  return new Color(fallback);
}

/** Frame-rate independent smoothing of `cur` toward `goal`. */
function ease(cur: number, goal: number, dt: number, ms = 90): number {
  return cur + (goal - cur) * (1 - Math.exp(-dt / ms));
}

export class PropsRenderer {
  private root = new Group();
  /** Hollowhearth's NPCs and stations (WP-068). */
  private hearth = new HearthProps(this.root);
  private geometries: BufferGeometry[] = [];
  private materials: Material[] = [];
  private updaters: Array<{ kind: StateKind; index: number; update: Updater; value: number }> = [];
  private offsets = new Map<StateKind, number>();
  private counts = new Map<StateKind, number>();
  private state: ArrayLike<number> = [];
  private disposed = false;
  private clock = 0;
  /** True once a level with mechanics is loaded; the engine skips the state read until then. */
  active = false;

  constructor(private scene: Scene) {
    this.root.name = 'latticefall-props';
    scene.add(this.root);
  }

  /** Loads `area`'s entities from its manifest and builds the props. Resolves false if there are none. */
  async load(area: string): Promise<boolean> {
    const res = await fetch(manifestUrl(area));
    if (!res.ok || !(res.headers.get('content-type') ?? '').includes('json')) return false;
    const data = (await res.json()) as { entities?: Entity[] };
    if (this.disposed) return false;
    return this.build(data.entities ?? []);
  }

  /** Builds props from a manifest's entity list. Exposed for tests. */
  build(entities: Entity[]): boolean {
    for (const k of STATE_KINDS) this.counts.set(k, 0);
    // Pass 1: counts, so the offsets into the state words are known.
    for (const e of entities) {
      const kind = Object.keys(e)[0] as StateKind;
      if (this.counts.has(kind)) this.counts.set(kind, (this.counts.get(kind) ?? 0) + 1);
    }
    let at = 1; // Word 0 is the ladder the player is on.
    for (const k of STATE_KINDS) {
      this.offsets.set(k, at);
      at += this.counts.get(k) ?? 0;
    }
    const seen = new Map<StateKind, number>();
    let any = false;
    for (const e of entities) {
      const kind = Object.keys(e)[0];
      const d = e[kind];
      if (kind === 'Boss') {
        this.fogGate((d.gate ?? {}) as Record<string, unknown>);
        any = true;
        continue;
      }
      if (kind === 'Npc' || kind === 'Station') {
        this.hearth.add(kind, d);
        any = true;
        continue;
      }
      if (kind === 'Snow') {
        this.snow(d);
        any = true;
        continue;
      }
      if (kind === 'EchoStone') {
        this.echoStone(d);
        any = true;
        continue;
      }
      if (!(STATE_KINDS as readonly string[]).includes(kind)) continue;
      const k = kind as StateKind;
      const index = seen.get(k) ?? 0;
      seen.set(k, index + 1);
      const update = this.make(k, d);
      if (update) this.updaters.push({ kind: k, index, update, value: NaN });
      any = true;
    }
    this.active = any;
    return any;
  }

  /** The core's `mechanics_state()` words for this tick. */
  setState(words: ArrayLike<number>): void {
    this.state = words;
  }

  /** Moves the props toward the state. Call once per frame. */
  update(frameMs: number): void {
    this.clock += frameMs;
    this.hearth.update(frameMs);
    for (const u of this.updaters) {
      const word = this.state[(this.offsets.get(u.kind) ?? 0) + u.index];
      if (word === undefined) continue;
      u.update(word, frameMs);
    }
  }

  dispose(): void {
    this.disposed = true;
    this.hearth.dispose();
    this.scene.remove(this.root);
    for (const g of this.geometries) g.dispose();
    for (const m of this.materials) m.dispose();
    this.geometries = [];
    this.materials = [];
    this.updaters = [];
  }

  // -------------------------------------------------------------------------------------

  private geo<T extends BufferGeometry>(g: T): T {
    this.geometries.push(g);
    return g;
  }

  private mat(color: Color | string, extra: Partial<ConstructorParameters<typeof MeshStandardMaterial>[0]> = {}): MeshStandardMaterial {
    const m = new MeshStandardMaterial({ color, roughness: 0.8, ...extra });
    this.materials.push(m);
    return m;
  }

  private add(o: Object3D, pos: V3, yaw = 0): Group {
    const g = new Group();
    g.position.set(pos[0], pos[1], pos[2]);
    g.rotation.y = yaw;
    g.add(o);
    this.root.add(g);
    return g;
  }

  private make(kind: StateKind, d: Record<string, unknown>): Updater | null {
    switch (kind) {
      case 'Ladder':
        return this.ladder(d);
      case 'Lift':
        return this.lift(d);
      case 'Door':
        return this.door(d);
      case 'Lever':
        return this.lever(d);
      case 'HollowWall':
        return this.hollowWall(d);
      case 'Pickup':
        return this.pickup(d);
      case 'Chest':
        return this.chest(d);
      case 'Message':
        return this.message(d);
      case 'Pool':
        return this.pool(d);
    }
  }

  private ladder(d: Record<string, unknown>): Updater {
    const h = d.height_m as number;
    const wood = this.mat(token('--clay', '#a4502f').lerp(new Color('#7a5a3a'), 0.6));
    const g = new Group();
    const rail = this.geo(new BoxGeometry(0.06, h, 0.06));
    for (const x of [-0.3, 0.3]) {
      const r = new Mesh(rail, wood);
      r.position.set(x, h / 2, 0);
      g.add(r);
    }
    const rung = this.geo(new BoxGeometry(0.6, 0.04, 0.04));
    for (let y = 0.3; y < h; y += 0.3) {
      const r = new Mesh(rung, wood);
      r.position.set(0, y, 0);
      g.add(r);
    }
    // Local +Z is the climber's facing; the rails stand 0.35 m ahead of the foot, on the wall.
    g.position.z = 0.4;
    const holder = this.add(g, d.pos as V3, d.yaw as number);
    const kick = d.kick_down as boolean;
    let y = kick ? h : 0;
    return (v, dt) => {
      // A raised kick-down ladder hangs above the player's reach until it drops.
      y = ease(y, v > 0.5 ? 0 : h, dt, 140);
      g.position.y = y;
      holder.visible = true;
    };
  }

  private lift(d: Record<string, unknown>): Updater {
    const [hx, hz] = d.half_extents as [number, number];
    const bottom = d.bottom_y as number;
    const [x, z] = d.pos as [number, number];
    const body = new Mesh(this.geo(new BoxGeometry(hx * 2, 0.4, hz * 2)), this.mat(token('--ink-2', '#6b6558')));
    body.position.y = -0.2;
    const g = this.add(body, [x, bottom, z]);
    let rise = 0;
    return (v, dt) => {
      rise = ease(rise, v, dt, 70);
      g.position.y = bottom + rise;
    };
  }

  private door(d: Record<string, unknown>): Updater {
    const hw = d.half_width_m as number;
    const h = d.height_m as number;
    const kind = d.kind as unknown;
    const color =
      kind === 'Locked' ? token('--falu', '#8b2e1f') : kind === 'OneWay' ? token('--fjord', '#2f5d74') : token('--moss', '#6b7f5e');
    const slab = new Mesh(this.geo(new BoxGeometry(hw * 2, h, 0.5)), this.mat(color));
    slab.position.y = h / 2;
    const g = this.add(slab, d.pos as V3, d.yaw as number);
    const y0 = g.position.y;
    let sunk = 0;
    return (v, dt) => {
      sunk = ease(sunk, v > 0.5 ? h + 0.1 : 0, dt, 120);
      g.position.y = y0 - sunk;
    };
  }

  private lever(d: Record<string, unknown>): Updater {
    const base = new Mesh(this.geo(new BoxGeometry(0.3, 0.5, 0.3)), this.mat(token('--ink-2', '#6b6558')));
    base.position.y = 0.25;
    const handle = new Mesh(this.geo(new CylinderGeometry(0.035, 0.035, 0.7, 8)), this.mat(token('--clay', '#a4502f')));
    handle.position.set(0, 0.85, 0);
    const pivot = new Group();
    pivot.position.set(0, 0.5, 0);
    handle.position.set(0, 0.35, 0);
    pivot.add(handle);
    const g = new Group();
    g.add(base, pivot);
    this.add(g, d.pos as V3, d.yaw as number);
    let a = -0.6;
    return (v, dt) => {
      a = ease(a, v > 0.5 ? 0.6 : -0.6, dt, 80);
      pivot.rotation.x = a;
    };
  }

  private hollowWall(d: Record<string, unknown>): Updater {
    const [hx, hy, hz] = d.half_extents as V3;
    // A shade off the wall around it: a faint tell, as the hum is the audible one.
    const m = new Mesh(this.geo(new BoxGeometry(hx * 2, hy * 2, hz * 2)), this.mat(token('--ink-3', '#8a8373')));
    const g = this.add(m, d.center as V3);
    const y0 = g.position.y;
    let sunk = 0;
    return (v, dt) => {
      sunk = ease(sunk, v > 0.5 ? hy * 2 + 0.1 : 0, dt, 160);
      g.position.y = y0 - sunk;
    };
  }

  private pickup(d: Record<string, unknown>): Updater {
    const gem = new Mesh(
      this.geo(new OctahedronGeometry(0.16)),
      this.mat('#f2d27a', { emissive: new Color('#f2b84a'), emissiveIntensity: 0.9 })
    );
    const g = this.add(gem, d.pos as V3);
    const y0 = g.position.y + 0.5;
    return (v) => {
      g.visible = v < 0.5;
      g.position.y = y0 + Math.sin(this.clock / 400) * 0.08;
      g.rotation.y = this.clock / 700;
    };
  }

  private chest(d: Record<string, unknown>): Updater {
    const wood = this.mat(token('--clay', '#a4502f').lerp(new Color('#5a3d28'), 0.5));
    const body = new Mesh(this.geo(new BoxGeometry(0.9, 0.45, 0.6)), wood);
    body.position.y = 0.225;
    const lidPivot = new Group();
    lidPivot.position.set(0, 0.45, -0.3);
    const lid = new Mesh(this.geo(new BoxGeometry(0.9, 0.2, 0.6)), wood);
    lid.position.set(0, 0.1, 0.3);
    lidPivot.add(lid);
    const g = new Group();
    g.add(body, lidPivot);
    this.add(g, d.pos as V3, d.yaw as number);
    let a = 0;
    return (v, dt) => {
      a = ease(a, v > 0.5 ? -1.1 : 0, dt, 100);
      lidPivot.rotation.x = a;
    };
  }

  private message(d: Record<string, unknown>): Updater {
    const m = this.mat('#f4e9c8', { emissive: new Color('#c9a64a'), emissiveIntensity: 0.4, side: DoubleSide });
    const sign = new Mesh(this.geo(new PlaneGeometry(0.5, 0.35)), m);
    sign.rotation.x = -Math.PI / 2 + 0.2;
    sign.position.y = 0.05;
    this.add(sign, d.pos as V3);
    return (v) => {
      m.emissiveIntensity = v > 0.5 ? 0.1 : 0.4 + Math.sin(this.clock / 500) * 0.15;
    };
  }

  private pool(d: Record<string, unknown>): Updater {
    // The water is drawn by `render/water` (WP-060): its surface is the pool's floor plus the
    // core's depth, so a lake in a basin sits level with its shore and drains when the sluice opens.
    const water = createPoolWater(
      d as unknown as PoolData,
      token('--fjord', '#2f6d8f'),
      token('--moss', '#6a8f3a')
    );
    this.root.add(water.group);
    this.geometries.push(...water.geometries);
    this.materials.push(...water.materials);
    return (v, dt) => water.update(v, dt, this.clock);
  }

  private snow(d: Record<string, unknown>): void {
    const [hx, hz] = d.half_extents as [number, number];
    const [x, z] = d.center as [number, number];
    const plane = new Mesh(this.geo(new PlaneGeometry(hx * 2, hz * 2)), this.mat('#f2f5f7', { side: DoubleSide }));
    plane.rotation.x = -Math.PI / 2;
    this.add(plane, [x, 0.12, z]);
  }

  /** An Echo Stone (WP-060): a short standing stone with a faint glow, where the Echo Bell works. */
  private echoStone(d: Record<string, unknown>): void {
    const stone = new Mesh(this.geo(new CylinderGeometry(0.35, 0.5, 1.3, 6)), this.mat('#8a8f98'));
    stone.position.y = 0.65;
    const glow = new Mesh(
      this.geo(new OctahedronGeometry(0.16)),
      this.mat('#d8e2ea', { emissive: new Color('#9fb6c6'), emissiveIntensity: 0.7 })
    );
    glow.position.y = 1.5;
    const g = new Group();
    g.add(stone, glow);
    this.add(g, d.pos as V3, d.yaw as number);
  }

  private fogGate(g: Record<string, unknown>): void {
    const hw = (g.half_width_m as number) ?? 2;
    const pos = g.pos as V3 | undefined;
    if (!pos) return;
    const m = this.mat('#d8e2ea', { transparent: true, opacity: 0.35, side: DoubleSide, emissive: new Color('#9fb6c6'), emissiveIntensity: 0.4 });
    const plane = new Mesh(this.geo(new PlaneGeometry(hw * 2, 3.2)), m);
    plane.position.y = 1.6;
    this.add(plane, pos, g.yaw as number);
  }
}
