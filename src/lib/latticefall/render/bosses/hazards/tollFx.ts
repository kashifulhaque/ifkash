// B10's bell toll and ice (docs/game/05-enemies-and-bosses.md, "B10 The Bellwright": every fourth
// beat the bell tolls, "a Rime shockwave that can be jumped or rolled", and "cracked ice zones
// break after two bell tolls").
//
//   - The toll: the core winds the bell up (`MechanicEvent` 191), then tolls (192). A thin ring
//     gathers at the Bellwright's feet during the wind-up; on the toll a pale ring and a low wall
//     of frost run outward at the core's speed (0.9 m a tick) to its reach (32 m). The wall is
//     knee height, so a jump or a roll reads as the answer to it.
//   - The ice: four patches, drawn as one instanced mesh per layer. While the fight runs each shows as a pale sheen on the ground; a crack
//     (194) draws dark seams across it, and when it breaks (195) it becomes black water with a
//     bright rim. A burn (196) makes that patch's rim flare.
//
// Nothing here is told when the fight ends: the beat arrives every 32 to 40 ticks, so a silence of
// several beats (or a `Boss` defeated/reset event, handled by the caller) clears everything.

import {
  AdditiveBlending,
  BufferAttribute,
  BufferGeometry,
  CircleGeometry,
  CylinderGeometry,
  Color,
  DoubleSide,
  Euler,
  Float32BufferAttribute,
  Group,
  InstancedMesh,
  Matrix4,
  Mesh,
  MeshBasicMaterial,
  Quaternion,
  RingGeometry,
  Vector3,
  type Scene
} from 'three';
import { B10, type Vec3 } from './arenas';

/** Ticks without a beat after which the fight counts as over. A beat is at most 40 ticks apart. */
const SILENCE_TICKS = 130;
/** Height of the shockwave's wall in meters: below a jump's apex, above a roll. */
const WALL_HEIGHT = 0.8;
/** Ticks a crack takes to spread, and a break takes to open. */
const CRACK_TICKS = 10;
const BREAK_TICKS = 12;
/** Ticks a burn flare lasts. */
const BURN_TICKS = 12;
const LIFT = 0.06;

type IceState = 0 | 1 | 2;

interface Patch {
  /** Where the patch's middle lies on the floor. */
  x: number;
  y: number;
  z: number;
  /** Shown once the fight's beats have begun. */
  shown: boolean;
  state: IceState;
  changedAt: number;
  burnedAt: number;
}

const PATCHES = B10.ice.length;
const FLAT = new Quaternion().setFromEuler(new Euler(-Math.PI / 2, 0, 0));
const NO_TURN = new Quaternion();

/** A few jagged seams spreading from the middle, as thin flat quads in the XZ plane (unit radius). */
function crackGeometry(): BufferGeometry {
  const pos: number[] = [];
  let seed = 7;
  const rand = () => {
    seed = (seed * 1664525 + 1013904223) >>> 0;
    return seed / 4294967296;
  };
  const rays = 9;
  for (let r = 0; r < rays; r++) {
    let a = (r / rays) * Math.PI * 2 + (rand() - 0.5) * 0.4;
    let x = 0;
    let z = 0;
    const segments = 4;
    for (let s = 0; s < segments; s++) {
      const len = (0.9 / segments) * (0.8 + rand() * 0.5);
      a += (rand() - 0.5) * 0.7;
      const nx = x + Math.cos(a) * len;
      const nz = z + Math.sin(a) * len;
      // A quad around the segment, tapering toward the tip.
      const w = 0.035 * (1 - s / (segments + 1));
      const px = -Math.sin(a) * w;
      const pz = Math.cos(a) * w;
      pos.push(x + px, 0, z + pz, x - px, 0, z - pz, nx, 0, nz);
      pos.push(x - px, 0, z - pz, nx - px * 0.5, 0, nz - pz * 0.5, nx, 0, nz);
      x = nx;
      z = nz;
    }
  }
  const g = new BufferGeometry();
  g.setAttribute('position', new Float32BufferAttribute(pos, 3));
  return g;
}

/** An open cylinder of unit radius, 0 to 1 tall, bright at the foot and clear at the top. */
function waveGeometry(): CylinderGeometry {
  const g = new CylinderGeometry(1, 1, 1, 72, 1, true);
  g.translate(0, 0.5, 0);
  const count = g.attributes.position.count;
  const colors = new Float32Array(count * 3);
  for (let i = 0; i < count; i++) {
    const v = 1 - g.attributes.position.getY(i);
    colors[i * 3] = colors[i * 3 + 1] = colors[i * 3 + 2] = v;
  }
  g.setAttribute('color', new BufferAttribute(colors, 3));
  return g;
}

const POS = new Vector3();
const SCALE = new Vector3();
const MAT = new Matrix4();
const ZERO = new Matrix4().makeScale(0, 0, 0);
const WHITE = new Color(1, 1, 1);
const FLARE = new Color();

export class TollFx {
  readonly group = new Group();
  private readonly ringGeom = new RingGeometry(0.93, 1, 72);
  private readonly thinRingGeom = new RingGeometry(0.96, 1, 48);
  private readonly discGeom = new CircleGeometry(1, 40);
  private readonly crackGeom = crackGeometry();
  private readonly waveGeom = waveGeometry();
  private readonly ringMat = new MeshBasicMaterial({ color: 0xcdeeff, transparent: true, opacity: 0, depthWrite: false, blending: AdditiveBlending, side: DoubleSide, fog: false });
  private readonly waveMat = new MeshBasicMaterial({ color: 0x9fd8f2, vertexColors: true, transparent: true, opacity: 0, depthWrite: false, blending: AdditiveBlending, side: DoubleSide, fog: false });
  private readonly warnMat = new MeshBasicMaterial({ color: 0xe8f7ff, transparent: true, opacity: 0, depthWrite: false, blending: AdditiveBlending, side: DoubleSide, fog: false });
  private readonly sheenMat = new MeshBasicMaterial({ color: 0xd4ecf7, transparent: true, opacity: 0.2, depthWrite: false, side: DoubleSide });
  private readonly crackMat = new MeshBasicMaterial({ color: 0x1d4a66, transparent: true, opacity: 0.9, depthWrite: false, side: DoubleSide });
  private readonly waterMat = new MeshBasicMaterial({ color: 0x06202f, transparent: true, opacity: 0.92, depthWrite: false, side: DoubleSide });
  private readonly rimMat = new MeshBasicMaterial({ color: 0x8fd6ee, transparent: true, opacity: 0.8, depthWrite: false, blending: AdditiveBlending, side: DoubleSide });
  private readonly ring = new Mesh(this.ringGeom, this.ringMat);
  private readonly wave = new Mesh(this.waveGeom, this.waveMat);
  private readonly warn = new Mesh(this.thinRingGeom, this.warnMat);
  /** The patches, drawn as one instanced mesh per layer: four draw calls for all of them. */
  private readonly sheens: InstancedMesh;
  private readonly cracks: InstancedMesh;
  private readonly waters: InstancedMesh;
  private readonly rims: InstancedMesh;
  private readonly patches: Patch[] = [];
  private lastBeat: number | null = null;
  private warnAt: number | null = null;
  private tollAt: number | null = null;
  private origin: Vec3 = B10.home;

  constructor(scene: Scene) {
    this.group.name = 'b10-toll';
    for (const m of [this.ring, this.warn]) {
      m.rotation.x = -Math.PI / 2;
      m.visible = false;
      m.renderOrder = 3;
    }
    this.wave.visible = false;
    this.wave.renderOrder = 3;
    this.group.add(this.ring, this.wave, this.warn);
    const [hx, hy, hz] = B10.home;
    for (const [dx, dz] of B10.ice) {
      this.patches.push({ x: hx + dx, y: hy + LIFT, z: hz + dz, shown: false, state: 0, changedAt: -1e9, burnedAt: -1e9 });
    }
    const layer = (geom: BufferGeometry, mat: MeshBasicMaterial, order: number, colored = false): InstancedMesh => {
      const m = new InstancedMesh(geom, mat, PATCHES);
      m.renderOrder = order;
      // The instances are laid out by hand each frame, so a bounding sphere made once would go stale.
      m.frustumCulled = false;
      for (let i = 0; i < PATCHES; i++) {
        m.setMatrixAt(i, ZERO);
        if (colored) m.setColorAt(i, WHITE);
      }
      this.group.add(m);
      return m;
    };
    this.sheens = layer(this.discGeom, this.sheenMat, 1);
    this.cracks = layer(this.crackGeom, this.crackMat, 2);
    this.waters = layer(this.discGeom, this.waterMat, 2);
    this.rims = layer(this.thinRingGeom, this.rimMat, 3, true);
    scene.add(this.group);
  }

  private get layers(): InstancedMesh[] {
    return [this.sheens, this.cracks, this.waters, this.rims];
  }

  /** A beat of the fight: the patches show while beats keep coming. */
  beat(now: number): void {
    if (this.lastBeat === null) for (const p of this.patches) p.shown = true;
    this.lastBeat = now;
  }

  /** The bell winds up at `origin`; the toll follows in `toll_warn_ticks`. */
  warned(now: number, origin: Vec3 | null): void {
    this.warnAt = now;
    if (origin) this.origin = origin;
  }

  /** The bell tolled: the shockwave starts from `origin` (the Bellwright's feet). */
  tolled(now: number, origin: Vec3 | null): void {
    this.tollAt = now;
    this.warnAt = null;
    if (origin) this.origin = origin;
    this.beat(now);
  }

  /** Ice patch `i` cracked (1) or broke (2). */
  ice(i: number, state: 1 | 2, now: number): void {
    const p = this.patches[i];
    if (!p || p.state >= state) return;
    p.state = state;
    p.changedAt = now;
    p.shown = true;
    if (this.lastBeat === null) this.lastBeat = now;
  }

  /** Freezing water burned the player on patch `i`. */
  burned(i: number, now: number): void {
    const p = this.patches[i];
    if (p) p.burnedAt = now;
  }

  /** The fight ended or reset: the ice is whole again and the wave is gone. */
  clear(): void {
    this.lastBeat = null;
    this.warnAt = null;
    this.tollAt = null;
    this.ring.visible = this.wave.visible = this.warn.visible = false;
    for (const p of this.patches) {
      p.state = 0;
      p.shown = false;
    }
    for (const m of this.layers) {
      for (let i = 0; i < PATCHES; i++) m.setMatrixAt(i, ZERO);
      m.instanceMatrix.needsUpdate = true;
    }
  }

  get active(): boolean {
    return this.lastBeat !== null;
  }

  /** Lays instance `i` of `mesh` at patch `p`, `up` meters above the floor and `radius` wide; flat discs and rings, or the crack seams as they are. */
  private place(mesh: InstancedMesh, i: number, p: Patch, up: number, radius: number, flat: boolean): void {
    POS.set(p.x, p.y + up, p.z);
    SCALE.setScalar(Math.max(radius, 1e-4));
    mesh.setMatrixAt(i, MAT.compose(POS, flat ? FLAT : NO_TURN, SCALE));
  }

  update(now: number): void {
    if (this.lastBeat === null) return;
    if (now - this.lastBeat > SILENCE_TICKS) {
      this.clear();
      return;
    }
    const [ox, oy, oz] = this.origin;

    // The wind-up: a ring gathers in around the bell.
    if (this.warnAt !== null) {
      const u = Math.min(Math.max((now - this.warnAt) / B10.tollWarn, 0), 1);
      this.warn.visible = true;
      this.warn.position.set(ox, oy + 0.1, oz);
      this.warn.scale.setScalar(5 - 3.6 * u);
      this.warnMat.opacity = 0.15 + 0.55 * u * (0.7 + 0.3 * Math.sin(now * 1.4));
    } else this.warn.visible = false;

    // The toll: the ring and the low wall run out at the core's speed.
    if (this.tollAt !== null) {
      const r = Math.max((now - this.tollAt) * B10.tollSpeed, 0.01);
      if (r > B10.tollMax + 2) {
        this.tollAt = null;
        this.ring.visible = this.wave.visible = false;
      } else {
        const fade = Math.max(0, 1 - Math.max(r - B10.tollMax * 0.55, 0) / (B10.tollMax * 0.45 + 2));
        this.ring.visible = this.wave.visible = true;
        this.ring.position.set(ox, oy + 0.12, oz);
        this.ring.scale.setScalar(r);
        this.ringMat.opacity = 0.9 * fade;
        this.wave.position.set(ox, oy + 0.05, oz);
        this.wave.scale.set(r, WALL_HEIGHT, r);
        this.waveMat.opacity = 0.55 * fade;
      }
    }

    // The ice: a patch that isn't in a layer's state has zero size in that layer.
    const R = B10.iceRadius;
    let flare = 0;
    for (let i = 0; i < PATCHES; i++) {
      const p = this.patches[i];
      const age = now - p.changedAt;
      if (!p.shown) {
        for (const m of this.layers) m.setMatrixAt(i, ZERO);
        continue;
      }
      if (p.state < 2) this.place(this.sheens, i, p, 0, R, true);
      else this.sheens.setMatrixAt(i, ZERO);
      if (p.state >= 1) this.place(this.cracks, i, p, 0.01, R * (p.state === 2 ? 1 : 0.35 + 0.65 * Math.min(age / CRACK_TICKS, 1)), false);
      else this.cracks.setMatrixAt(i, ZERO);
      if (p.state === 2) {
        const k = Math.min(age / BREAK_TICKS, 1);
        this.place(this.waters, i, p, 0.01, R * (0.3 + 0.7 * k), true);
        this.place(this.rims, i, p, 0.02, R * (0.97 + 0.03 * Math.sin(now * 0.25)) * (0.3 + 0.7 * k), true);
        // A burn makes that patch's rim glow: an instance color above 1 reads as a flare.
        const f = Math.max(0, 1 - (now - p.burnedAt) / BURN_TICKS);
        this.rims.setColorAt(i, FLARE.setScalar(1 + f * 1.2));
        flare = Math.max(flare, f);
      } else {
        this.waters.setMatrixAt(i, ZERO);
        this.rims.setMatrixAt(i, ZERO);
      }
    }
    for (const m of this.layers) m.instanceMatrix.needsUpdate = true;
    if (this.rims.instanceColor) this.rims.instanceColor.needsUpdate = true;
    this.rimMat.opacity = 0.45 + 0.2 * Math.sin(now * 0.2) + flare * 0.2;
  }

  dispose(): void {
    this.group.removeFromParent();
    for (const g of [this.ringGeom, this.thinRingGeom, this.discGeom, this.crackGeom, this.waveGeom]) g.dispose();
    for (const m of [this.ringMat, this.waveMat, this.warnMat, this.sheenMat, this.crackMat, this.waterMat, this.rimMat]) m.dispose();
    for (const m of this.layers) m.dispose();
  }
}
