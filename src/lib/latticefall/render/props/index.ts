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
  ShaderMaterial,
  MeshBasicMaterial,
  CanvasTexture,
  SRGBColorSpace,
  AdditiveBlending,
  type Texture,
  type BufferGeometry,
  type Material,
  type Object3D,
  type Scene
} from 'three';
import { fetchAreaArt, type AreaArt } from '../kits/art';
import { manifestUrl } from '../kits/manifest';
import { DeepProps } from './deep';
import { FigureProps } from './figures';
import { GlassProps } from './glass';
import { HearthProps } from './hearth';
import { RimeProps } from './rime';
import { createPoolWater, type PoolData } from '../water';

type V3 = [number, number, number];

interface Entity {
  [kind: string]: Record<string, unknown>;
}

const FOG_GATE_VERT = `varying vec2 vUv;
varying float vDist;
void main() {
  vUv = uv;
  vec4 mv = modelViewMatrix * vec4(position, 1.0);
  vDist = length(mv.xyz);
  gl_Position = projectionMatrix * mv;
}`;

// Value-noise mist rising slowly, brightest low and in the middle, gone at the edges. It
// fades out with distance, as the scene fog doesn't reach a shader material.
const FOG_GATE_FRAG = `varying vec2 vUv;
varying float vDist;
uniform float uTime;
uniform float uSeed;
uniform float uFade;
float h(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7)) + uSeed) * 43758.5453); }
float n(vec2 p) {
  vec2 i = floor(p), f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(mix(h(i), h(i + vec2(1.0, 0.0)), u.x), mix(h(i + vec2(0.0, 1.0)), h(i + vec2(1.0, 1.0)), u.x), u.y);
}
void main() {
  vec2 p = vUv * vec2(4.0, 2.5) + vec2(uTime * 0.05, -uTime * 0.18);
  float m = 0.5 * n(p) + 0.3 * n(p * 2.1 + 3.7) + 0.2 * n(p * 4.3 - uTime * 0.3);
  float edge = smoothstep(0.0, 0.12, vUv.x) * smoothstep(1.0, 0.88, vUv.x) * smoothstep(1.0, 0.7, vUv.y) * smoothstep(0.0, 0.04, vUv.y);
  float a = uFade * (0.18 + 0.5 * m) * edge * (1.0 - 0.35 * vUv.y) * (1.0 - smoothstep(12.0, 28.0, vDist));
  gl_FragColor = vec4(vec3(0.62, 0.68, 0.74) * (0.7 + 0.5 * m), a);
}`;

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

/** A boss's fog gate: its mist, how much of it shows, and where it stands. */
interface FogGate {
  group: Group | null;
  fade: { value: number };
  open: boolean;
  at: V3 | null;
}

/** A fixed yaw per entity from its position, so props don't all face the same way. */
function rngYaw(d: Record<string, unknown>): number {
  const p = d.pos as V3;
  return ((Math.sin(p[0] * 12.9898 + p[2] * 78.233) * 43758.5453) % 1) * Math.PI * 2;
}

/** Frame-rate independent smoothing of `cur` toward `goal`. */
function ease(cur: number, goal: number, dt: number, ms = 90): number {
  return cur + (goal - cur) * (1 - Math.exp(-dt / ms));
}

export class PropsRenderer {
  private root = new Group();
  /** Hollowhearth's NPCs and stations (WP-068). */
  private hearth = new HearthProps(this.root);
  /** The Lantern and the NPC figures, where the area art manifest gives them a model file. */
  private figures = new FigureProps(this.root);
  /** Mine carts and the Root Stair's trapdoor (WP-062). */
  private deep = new DeepProps(this.root);
  private cartAt = -1;
  private hatchAt = -1;
  /** Glasswing Grotto's emitters, mirrors, and sockets (WP-063), after the visibility word. */
  private glass = new GlassProps(this.root);
  private mirrorAt = -1;
  private receptorAt = -1;
  private beamAt = -1;
  /** The Catacombs' braziers and the Ring's sigil (WP-066), after the beams' words. */
  private rime = new RimeProps(this.root);
  private brazierAt = -1;
  private sigilAt = -1;
  /** The state word holding the player's visibility in meters, or -1 in an area with no darkness. */
  private visAt = -1;
  private geometries: BufferGeometry[] = [];
  private materials: Material[] = [];
  /** Props that animate on their own, with no state word (the fog gate's mist). */
  private animated: Array<(dt: number) => void> = [];
  private updaters: Array<{ kind: StateKind; index: number; update: Updater; value: number }> = [];
  private offsets = new Map<StateKind, number>();
  private counts = new Map<StateKind, number>();
  private state: ArrayLike<number> = [];
  private disposed = false;
  private clock = 0;
  /** The area art manifest, for entity models (the hollow wall). */
  private art: AreaArt | null = null;
  /** True once a level with mechanics is loaded; the engine skips the state read until then. */
  active = false;

  constructor(private scene: Scene) {
    this.root.name = 'latticefall-props';
    scene.add(this.root);
  }

  /** Loads `area`'s entities from its manifest and builds the props. Resolves false if there are none. */
  async load(area: string): Promise<boolean> {
    const [res, art] = await Promise.all([fetch(manifestUrl(area)), fetchAreaArt(area)]);
    if (!res.ok || !(res.headers.get('content-type') ?? '').includes('json')) return false;
    const data = (await res.json()) as { entities?: Entity[] };
    if (this.disposed) return false;
    return this.build(data.entities ?? [], art);
  }

  /** How many model files are still loading. The benchmark waits for 0 before it measures. */
  get modelsPending(): number {
    return this.figures.pending;
  }

  /**
   * Builds props from a manifest's entity list. `art` is the area art manifest, which names
   * the model files for the Lantern and the NPCs; without it they draw as greybox or not at
   * all. Exposed for tests.
   */
  build(entities: Entity[], art: AreaArt | null = null): boolean {
    this.art = art;
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
    // WP-062: after the pools come each mine cart (its distance along the rail), each trapdoor
    // (1 when open), and, in a dark area only, the player's visibility in meters (0: no limit).
    const kindCount = (name: string) => entities.filter((e) => Object.keys(e)[0] === name).length;
    this.cartAt = at;
    at += kindCount('Cart');
    this.hatchAt = at;
    at += kindCount('Trapdoor');
    this.visAt = kindCount('Darkness') > 0 ? at : -1;
    // WP-063: after the visibility word (present only in a dark area) come each mirror's facing, each
    // receptor (1 when lit), and each beam source's legs (a count, then 8 legs of x0, z0, x1, z1).
    if (this.visAt >= 0) at += 1;
    this.mirrorAt = at;
    at += kindCount('Mirror');
    this.receptorAt = at;
    at += kindCount('Receptor');
    this.beamAt = at;
    // WP-066: after the beams' words (a count and 8 legs of 4 words each, per source) come each
    // brazier (1 when lit) and each sigil (0 cold, 1 touched, 2 sealed).
    at += kindCount('BeamSource') * (1 + 4 * 8);
    this.brazierAt = at;
    at += kindCount('Brazier');
    this.sigilAt = at;
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
      if (kind === 'Cart' || kind === 'Trapdoor') {
        this.deep.add(kind, d);
        any = true;
        continue;
      }
      if (kind === 'Darkness') {
        any = true;
        continue;
      }
      if (kind === 'BeamSource' || kind === 'Mirror' || kind === 'Receptor') {
        this.glass.add(kind, d);
        any = true;
        continue;
      }
      if (kind === 'Brazier' || kind === 'Sigil') {
        this.rime.add(kind, d);
        any = true;
        continue;
      }
      if (kind === 'Npc' || kind === 'Station') {
        const npc = this.hearth.add(kind, d);
        const spec = npc ? art?.entities?.npcs?.[String(d.npc)] : undefined;
        const file = spec ? art?.models[spec.model]?.file : undefined;
        if (npc && spec && file) this.figures.npc(npc.holder, npc.figure, spec, file);
        any = true;
        continue;
      }
      if (kind === 'Lantern') {
        const spec = art?.entities?.lantern;
        const file = spec ? art?.models[spec.model]?.file : undefined;
        if (spec && file) {
          this.figures.lantern(d, spec, file);
          any = true;
        }
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

  /** How far the player sees in meters (Hollowdeep's darkness, WP-062); 0 for no limit. */
  get visibility(): number {
    return this.visAt < 0 ? 0 : (this.state[this.visAt] ?? 0);
  }

  /** Moves the props toward the state. Call once per frame. */
  update(frameMs: number): void {
    this.clock += frameMs;
    this.hearth.update(frameMs);
    this.figures.update(frameMs);
    this.deep.update(frameMs, this.state, this.cartAt, this.hatchAt);
    this.glass.update(frameMs, this.state, this.mirrorAt, this.receptorAt, this.beamAt);
    this.rime.update(frameMs, this.state, this.brazierAt, this.sigilAt);
    for (const a of this.animated) a(frameMs);
    for (const u of this.updaters) {
      const word = this.state[(this.offsets.get(u.kind) ?? 0) + u.index];
      if (word === undefined) continue;
      u.update(word, frameMs);
    }
  }

  dispose(): void {
    this.disposed = true;
    this.hearth.dispose();
    this.figures.dispose();
    this.deep.dispose();
    this.glass.dispose();
    this.rime.dispose();
    this.scene.remove(this.root);
    for (const g of this.geometries) g.dispose();
    for (const m of this.materials) m.dispose();
    for (const t of this.textures) t.dispose();
    this.textures = [];
    this.geometries = [];
    this.materials = [];
    this.updaters = [];
    this.animated = [];
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
    // The art manifest's model, built around the collider, replaces the box once it loads.
    const spec = this.art?.entities?.hollow_wall;
    const file = spec ? this.art?.models[spec.model]?.file : undefined;
    if (file) this.figures.replace(g, m, file);
    const y0 = g.position.y;
    let sunk = 0;
    return (v, dt) => {
      sunk = ease(sunk, v > 0.5 ? hy * 2 + 0.1 : 0, dt, 160);
      g.position.y = y0 - sunk;
    };
  }

  /** The art manifest's file for an entity model, or undefined. */
  private entityFile(kind: 'chest' | 'pickup' | 'message'): string | undefined {
    const spec = this.art?.entities?.[kind];
    return spec ? this.art?.models[spec.model]?.file : undefined;
  }

  private pickup(d: Record<string, unknown>): Updater {
    const gem = new Mesh(
      this.geo(new OctahedronGeometry(0.16)),
      this.mat('#f2d27a', { emissive: new Color('#f2b84a'), emissiveIntensity: 0.9 })
    );
    const g = this.add(gem, d.pos as V3);
    const y0 = g.position.y + 0.5;
    const file = this.entityFile('pickup');
    if (file) {
      // A small object on the ground with a soft glint above it, in place of the floating gem.
      const holder = new Group();
      const glint = new Mesh(this.glintQuad(), this.glintMaterial());
      glint.position.y = 0.16;
      // Always faces the camera, like a sprite.
      glint.onBeforeRender = (_r, _s, camera) => {
        glint.parent?.getWorldQuaternion(glint.quaternion).invert().multiply(camera.quaternion);
      };
      holder.add(glint);
      const phase = (d.pos as V3)[0] * 1.3 + (d.pos as V3)[2] * 0.7;
      this.add(holder, d.pos as V3, rngYaw(d));
      this.figures.replace(holder, gem, file);
      return (v) => {
        holder.visible = v < 0.5;
        const t = this.clock / 1000 + phase;
        glint.material.opacity = 0.55 + 0.3 * Math.sin(t * 2.1) + 0.15 * Math.sin(t * 5.3);
        glint.scale.setScalar(0.5 + 0.08 * Math.sin(t * 1.7));
      };
    }
    return (v) => {
      g.visible = v < 0.5;
      g.position.y = y0 + Math.sin(this.clock / 400) * 0.08;
      g.rotation.y = this.clock / 700;
    };
  }

  private glint: MeshBasicMaterial | null = null;
  private glintGeometry: PlaneGeometry | null = null;
  private textures: Texture[] = [];

  private glintQuad(): PlaneGeometry {
    return (this.glintGeometry ??= this.geo(new PlaneGeometry(1, 1)));
  }

  /** A soft, warm star of light for ground pickups: a radial gradient drawn once. */
  private glintMaterial(): MeshBasicMaterial {
    if (this.glint) return this.glint;
    const c = document.createElement('canvas');
    c.width = c.height = 64;
    const g = c.getContext('2d');
    if (g) {
      const grad = g.createRadialGradient(32, 32, 0, 32, 32, 32);
      grad.addColorStop(0, 'rgba(255,240,205,1)');
      grad.addColorStop(0.18, 'rgba(255,214,150,0.55)');
      grad.addColorStop(1, 'rgba(255,190,110,0)');
      g.fillStyle = grad;
      g.fillRect(0, 0, 64, 64);
      // Four thin rays.
      g.fillStyle = 'rgba(255,236,200,0.5)';
      g.fillRect(31, 4, 2, 56);
      g.fillRect(4, 31, 56, 2);
    }
    const tex = new CanvasTexture(c);
    tex.colorSpace = SRGBColorSpace;
    this.textures.push(tex);
    this.glint = new MeshBasicMaterial({ map: tex, transparent: true, depthWrite: false, blending: AdditiveBlending, fog: true });
    this.materials.push(this.glint);
    return this.glint;
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
    const file = this.entityFile('chest');
    if (file) {
      // The model's `lid` node hinges at its origin, the same hinge as the greybox lid.
      this.figures.replace(g, [body, lid], file, (scene) => {
        const node = scene.getObjectByName('lid');
        if (!node) return;
        // A hinge inside the loaded scene, so freeing the scene frees the lid too.
        const pivot = new Group();
        pivot.position.copy(node.position);
        node.position.set(0, 0, 0);
        pivot.add(node);
        scene.add(pivot);
        hinge = pivot;
      });
    }
    let hinge: Object3D = lidPivot;
    let a = 0;
    return (v, dt) => {
      a = ease(a, v > 0.5 ? -1.1 : 0, dt, 100);
      hinge.rotation.x = a;
    };
  }

  private message(d: Record<string, unknown>): Updater {
    const m = this.mat('#f4e9c8', { emissive: new Color('#c9a64a'), emissiveIntensity: 0.4, side: DoubleSide });
    const sign = new Mesh(this.geo(new PlaneGeometry(0.5, 0.35)), m);
    sign.rotation.x = -Math.PI / 2 + 0.2;
    sign.position.y = 0.05;
    const holder = this.add(sign, d.pos as V3, rngYaw(d));
    const file = this.entityFile('message');
    // A glowing inscription on the floor: its glyphs breathe, and dim once read.
    const glows: Array<{ m: MeshStandardMaterial; base: number }> = [];
    if (file) {
      this.figures.replace(holder, sign, file, (scene) => {
        scene.traverse((o) => {
          const mat = (o as Mesh).material as MeshStandardMaterial | undefined;
          if (mat?.emissive && mat.emissive.getHex() !== 0) glows.push({ m: mat, base: mat.emissiveIntensity });
        });
      });
    }
    const phase = (d.pos as V3)[2] * 0.37;
    return (v) => {
      const pulse = 0.8 + 0.2 * Math.sin(this.clock / 700 + phase);
      m.emissiveIntensity = v > 0.5 ? 0.1 : 0.4 + Math.sin(this.clock / 500) * 0.15;
      for (const g of glows) g.m.emissiveIntensity = g.base * (v > 0.5 ? 0.45 : pulse);
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

  /**
   * The boss's fog gate: two layers of drifting mist across the gate, soft at the edges. Purely
   * a look; the gate's rule lives in the core.
   */
  private fogGate(g: Record<string, unknown>): void {
    const hw = (g.half_width_m as number) ?? 2;
    const pos = g.pos as V3 | undefined;
    // Gates go in boss order, so the core's boss index finds its gate.
    const gate: FogGate = { group: null, fade: { value: 1 }, open: false, at: pos ?? null };
    this.gates.push(gate);
    if (!pos) return;
    const height = 4.8;
    const group = new Group();
    gate.group = group;
    const time = { value: 0 };
    for (const [i, z] of [0, 0.25].entries()) {
      const m = new ShaderMaterial({
        uniforms: { uTime: time, uSeed: { value: i * 17.3 }, uFade: gate.fade },
        vertexShader: FOG_GATE_VERT,
        fragmentShader: FOG_GATE_FRAG,
        transparent: true,
        depthWrite: false,
        side: DoubleSide
      });
      this.materials.push(m);
      const plane = new Mesh(this.geo(new PlaneGeometry(hw * 2 + 0.4, height)), m);
      plane.position.set(0, height / 2, z);
      plane.renderOrder = 2;
      group.add(plane);
    }
    this.add(group, pos, g.yaw as number);
    this.animated.push((dt) => {
      time.value += dt / 1000;
      if (gate.open && gate.fade.value > 0) {
        // The mist thins over about a second and a half, then stops drawing.
        gate.fade.value = Math.max(0, gate.fade.value - dt / 1500);
        if (gate.fade.value === 0) group.visible = false;
      }
    });
    if (this.defeatedAt.some((p) => this.sameGate(gate.at, p))) this.openGate(gate, true);
  }

  /** The boss gates in boss order, and the gates of bosses already defeated (from the save). */
  private gates: FogGate[] = [];
  private defeatedAt: Array<{ x: number; z: number }> = [];

  private sameGate(at: V3 | null, p: { x: number; z: number }): boolean {
    return !!at && Math.hypot(at[0] - p.x, at[2] - p.z) < 1.5;
  }

  private openGate(gate: FogGate, now: boolean): void {
    gate.open = true;
    if (now) {
      gate.fade.value = 0;
      if (gate.group) gate.group.visible = false;
    }
  }

  /** The core's boss-defeated event: that boss's fog gate fades out and stops drawing. */
  bossDefeated(index: number): void {
    const gate = this.gates[index];
    if (gate) this.openGate(gate, false);
  }

  /**
   * Bosses already defeated when the area loads, by their gate's position (the map lists
   * them that way): their gates never draw. Safe to call before or after the props load.
   */
  setDefeatedBosses(at: ReadonlyArray<{ x: number; z: number }>): void {
    this.defeatedAt = [...at];
    for (const gate of this.gates) if (at.some((p) => this.sameGate(gate.at, p))) this.openGate(gate, true);
  }
}
