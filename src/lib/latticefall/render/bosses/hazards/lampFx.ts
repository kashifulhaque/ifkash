// B09's lit lamps (docs/game/05-enemies-and-bosses.md, "B09 The Lamplight Matriarch": "The player
// can light the six lamps ... each lit lamp restores light and stuns her briefly when she flies
// past it").
//
// The core says which lamp lit (`MechanicEvent` 183), which one stunned her (184), and when they all
// go out (185). A lit lamp draws a flame on top of its pillar, a soft glow around it, and a pool
// of warm light on the floor `lamp_light_m` wide, so the lit ground reads in the drained cavern.
// All of it ignores fog, since the cavern's fog is what the drain closes in.
//
// Cheap on purpose: the two flame cones and the floor pools are instanced meshes (one draw call
// each for all six lamps), and the six glows are sprites that share one material.

import {
  AdditiveBlending,
  ConeGeometry,
  DoubleSide,
  Euler,
  Group,
  InstancedMesh,
  Matrix4,
  MeshBasicMaterial,
  PlaneGeometry,
  Quaternion,
  Sprite,
  SpriteMaterial,
  Vector3,
  type CanvasTexture,
  type Scene
} from 'three';
import { B09 } from './arenas';
import { glowTexture } from './textures';

/** Ticks a flame takes to flare up when its lamp lights, and a stun flash lasts. */
const IGNITE_TICKS = 12;
const STUN_TICKS = 24;
/** Half the height of each flame cone, in meters at scale 1. */
const OUTER_HALF = 1.0;
const INNER_HALF = 0.675;

interface Lamp {
  /** Where the lamp's top is, in world space. */
  x: number;
  y: number;
  z: number;
  halo: Sprite;
  litAt: number | null;
  stunnedAt: number;
}

const POS = new Vector3();
const SCALE = new Vector3();
const MAT = new Matrix4();
const UPRIGHT = new Quaternion();
const FLAT = new Quaternion().setFromEuler(new Euler(-Math.PI / 2, 0, 0));
const ZERO = new Matrix4().makeScale(0, 0, 0);

export class LampFx {
  readonly group = new Group();
  private readonly glow: CanvasTexture | null = glowTexture();
  private readonly outerGeom = new ConeGeometry(0.7, 2.0, 12, 1, true);
  private readonly innerGeom = new ConeGeometry(0.38, 1.35, 10, 1, true);
  private readonly poolGeom = new PlaneGeometry(1, 1);
  private readonly outerMat = new MeshBasicMaterial({ color: 0xff8a2a, transparent: true, opacity: 0.85, blending: AdditiveBlending, depthWrite: false, side: DoubleSide, fog: false });
  private readonly innerMat = new MeshBasicMaterial({ color: 0xffe9a8, transparent: true, opacity: 0.95, blending: AdditiveBlending, depthWrite: false, side: DoubleSide, fog: false });
  private readonly haloMat = new SpriteMaterial({ map: this.glow, color: 0xffb25a, transparent: true, opacity: 0.8, blending: AdditiveBlending, depthWrite: false, fog: false });
  private readonly poolMat = new MeshBasicMaterial({ map: this.glow, color: 0xffa24a, transparent: true, opacity: 0.4, blending: AdditiveBlending, depthWrite: false, side: DoubleSide, fog: false });
  private readonly outers: InstancedMesh;
  private readonly inners: InstancedMesh;
  private readonly pools: InstancedMesh;
  private readonly lamps: Lamp[] = [];

  constructor(scene: Scene) {
    this.group.name = 'b09-lamps';
    const n = B09.lamps.length;
    const layer = (geom: ConeGeometry | PlaneGeometry, mat: MeshBasicMaterial, order: number): InstancedMesh => {
      const m = new InstancedMesh(geom, mat, n);
      m.renderOrder = order;
      // Laid out by hand each frame, so a bounding sphere made once would go stale.
      m.frustumCulled = false;
      for (let i = 0; i < n; i++) m.setMatrixAt(i, ZERO);
      this.group.add(m);
      return m;
    };
    this.pools = layer(this.poolGeom, this.poolMat, 4);
    this.outers = layer(this.outerGeom, this.outerMat, 4);
    this.inners = layer(this.innerGeom, this.innerMat, 4);
    const [hx, hy, hz] = B09.home;
    for (const [dx, dz] of B09.lamps) {
      const halo = new Sprite(this.haloMat);
      halo.renderOrder = 4;
      halo.visible = false;
      this.group.add(halo);
      this.lamps.push({ x: hx + dx, y: hy + B09.lampHeight, z: hz + dz, halo, litAt: null, stunnedAt: -1e9 });
    }
    scene.add(this.group);
  }

  private get layers(): InstancedMesh[] {
    return [this.pools, this.outers, this.inners];
  }

  /** Lamp `i` lit at game tick `now`. */
  lit(i: number, now: number): void {
    const l = this.lamps[i];
    if (!l || l.litAt !== null) return;
    l.litAt = now;
    l.halo.visible = true;
  }

  /** A lit lamp stunned the Matriarch as she flew past: it flares. */
  stunned(i: number, now: number): void {
    const l = this.lamps[i];
    if (l) l.stunnedAt = now;
  }

  /** The fight ended or reset: every lamp goes out. */
  douse(): void {
    for (const l of this.lamps) {
      l.litAt = null;
      l.halo.visible = false;
    }
    for (const m of this.layers) {
      for (let i = 0; i < this.lamps.length; i++) m.setMatrixAt(i, ZERO);
      m.instanceMatrix.needsUpdate = true;
    }
  }

  /** How many lamps are lit. */
  get litCount(): number {
    return this.lamps.reduce((n, l) => n + (l.litAt === null ? 0 : 1), 0);
  }

  update(now: number): void {
    if (this.litCount === 0) return;
    for (let i = 0; i < this.lamps.length; i++) {
      const l = this.lamps[i];
      if (l.litAt === null) continue;
      // Flares up with a little overshoot, then flickers.
      const u = Math.min((now - l.litAt) / IGNITE_TICKS, 1);
      const rise = u < 1 ? 1 - (1 - u) * (1 - u) : 1;
      const flick = 1 + 0.12 * Math.sin(now * 0.9 + i * 1.7) + 0.07 * Math.sin(now * 2.3 + i * 0.6);
      const stun = Math.max(0, 1 - (now - l.stunnedAt) / STUN_TICKS);
      const s = rise * flick * (1 + stun * 0.6);
      // The cones stand on the lamp's top: their pivot is the middle, so lift by half their grown height.
      SCALE.set(s * (1 - 0.06 * Math.sin(now * 1.7 + i)), s, s);
      this.outers.setMatrixAt(i, MAT.compose(POS.set(l.x, l.y + OUTER_HALF * s, l.z), UPRIGHT, SCALE));
      const si = s * 0.95;
      SCALE.set(si, s * (1 + 0.1 * Math.sin(now * 2.9 + i * 2)), si);
      this.inners.setMatrixAt(i, MAT.compose(POS.set(l.x, l.y + INNER_HALF * s, l.z), UPRIGHT, SCALE));
      // The pool of light swells in with the flame, on the floor 3 m below the lamp's top.
      SCALE.setScalar(B09.lightRadius * 2 * (0.4 + 0.6 * rise));
      this.pools.setMatrixAt(i, MAT.compose(POS.set(l.x, l.y - B09.lampHeight + 0.08, l.z), FLAT, SCALE));
      l.halo.position.set(l.x, l.y + 1.1, l.z);
      l.halo.scale.setScalar((8.5 + 2.4 * stun) * (0.5 + 0.5 * rise) * (0.96 + 0.04 * flick));
    }
    for (const m of this.layers) m.instanceMatrix.needsUpdate = true;
    this.poolMat.opacity = 0.34 + 0.04 * Math.sin(now * 0.5);
  }

  dispose(): void {
    this.group.removeFromParent();
    for (const g of [this.outerGeom, this.innerGeom, this.poolGeom]) g.dispose();
    for (const m of [this.outerMat, this.innerMat, this.haloMat, this.poolMat]) m.dispose();
    for (const m of this.layers) m.dispose();
    this.glow?.dispose();
  }
}
