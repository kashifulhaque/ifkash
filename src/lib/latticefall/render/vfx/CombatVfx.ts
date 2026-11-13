// Combat VFX: weapon trails, hit sparks, hitstop, camera shake and status auras. All of it is
// render-only (docs/game/07-presentation.md, "VFX"): it reads core events and snapshots,
// holds its own state, and never writes simulation state. Hitstop freezes the drawn
// animation of the two actors for a few frames and the core keeps ticking.
//
// Per tick:   `handleEvents(events)`   spawns sparks and flashes, starts hitstop, adds trauma.
// Per frame:  `update(...)`            advances particles, trails and auras and fills the buffers.
//             `applyCamera(camera)`    shakes the draw camera, right before the scene renders.

import { type MeshBasicMaterial, type PerspectiveCamera, type Scene, type ShaderMaterial, Vector3 } from 'three';
import type { GameEvent, HitEvent } from '../../engine/events';
import type { DrawActor } from '../../engine/snapshot';
import { movePhase, type MoveResolver } from '../actors/moveClips';
import { PointCloud, makePointMaterial } from './points';
import { type HitFxResolver, fallbackHitstop, hitTrauma, sparkStyle, type SparkStyle } from './palette';
import { CameraShake } from './shake';
import { Trail, makeTrailMaterial, trailStyleForAnim } from './trail';

/** What the VFX need from the actor renderer. */
export interface VfxActorHooks {
  /** Freezes an actor's drawn animation for `seconds`. */
  holdPose(id: number, seconds: number): void;
  isHeld(id: number): boolean;
  /** World positions of the weapon's tip and base (hand). False if the actor isn't drawn yet. */
  weaponTip(id: number, tip: Vector3, base: Vector3): boolean;
}

export interface CombatVfxOptions {
  /** Move number to animation info; decides which moves leave a trail and when. */
  resolveMove?: MoveResolver;
  /** Move number and hitbox to damage type and hitstop; see `palette.ts`. */
  resolveHit?: HitFxResolver;
}

/**
 * A status effect's aura: the `statusMask` bit and the mote colour (linear 0-1 RGB).
 * Empty on purpose. WP-025 defines the bits; add one entry per status here and its aura
 * draws. Nothing is invented for bits the core doesn't set yet.
 */
export const STATUS_AURAS: Array<{ bit: number; color: [number, number, number] }> = [];

const SPARK_CAPACITY = 160;
const FLASH_CAPACITY = 12;
const AURA_MOTES = 10;
const AURA_CAPACITY = 256;
const GRAVITY = 9;
const TICK_SECONDS = 1 / 30;

interface Spark {
  x: number;
  y: number;
  z: number;
  vx: number;
  vy: number;
  vz: number;
  age: number;
  life: number;
  size: number;
  style: SparkStyle;
}

interface Flash {
  x: number;
  y: number;
  z: number;
  age: number;
  life: number;
  size: number;
  color: [number, number, number];
}

export interface VfxDebug {
  sparks: number;
  flashes: number;
  trails: Array<{ id: number; samples: number }>;
  auraPoints: number;
  hold: number;
  hitsSeen: number;
  shake: { trauma: number; x: number; y: number; roll: number; reducedMotion: boolean };
}

export class CombatVfx {
  readonly shake = new CameraShake();
  private hooks: VfxActorHooks;
  private scene: Scene;
  private resolveMove: MoveResolver;
  private resolveHit: HitFxResolver | undefined;

  private pointMaterial: ShaderMaterial = makePointMaterial();
  private trailMaterial: MeshBasicMaterial = makeTrailMaterial();
  private sparkCloud = new PointCloud(SPARK_CAPACITY * 3 + FLASH_CAPACITY, this.pointMaterial);
  private auraCloud = new PointCloud(AURA_CAPACITY, this.pointMaterial);
  private sparks: Spark[] = [];
  private flashes: Flash[] = [];
  private trails = new Map<number, Trail>();
  private auraPreview = new Map<number, [number, number, number]>();
  private clock = 0;
  private hold = 0;
  private playerId = -1;
  private hitsSeen = 0;
  private rng = 0x9e3779b9;
  private tip = new Vector3();
  private base = new Vector3();

  constructor(scene: Scene, hooks: VfxActorHooks, opts: CombatVfxOptions = {}) {
    this.scene = scene;
    this.hooks = hooks;
    this.resolveMove = opts.resolveMove ?? (() => undefined);
    this.resolveHit = opts.resolveHit;
    scene.add(this.sparkCloud.points, this.auraCloud.points);
  }

  /** Spawns the effects for this tick's events. Call once per core tick. */
  handleEvents(events: GameEvent[]): void {
    for (const e of events) if (e.kind === 'hit') this.onHit(e);
  }

  private rand(): number {
    // xorshift32: visual variety only, never feeds the simulation.
    let x = this.rng;
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    this.rng = x >>> 0;
    return this.rng / 4294967296;
  }

  private onHit(e: HitEvent): void {
    this.hitsSeen++;
    const info = this.resolveHit?.(e.moveNum, e.hitbox);
    const guarded = e.result === 1; // WP-023: blocked hits throw fewer, paler sparks and don't shake
    const style = sparkStyle(guarded ? 'Pierce' : (info?.phys ?? 'Neutral'), guarded ? null : info?.element);
    const frames = info?.hitstop ?? fallbackHitstop(e.motionValue, e.chargeFrac);
    const [px, py, pz] = e.point;
    const [dx, , dz] = e.dir;

    // Sparks fly back toward the attacker and up, scattered over a wide cone.
    // Reduced motion: half the sparks and no flash (the flash is the brightest, briefest effect).
    const calm = this.shake.reducedMotion;
    const count = Math.round((12 + 10 * e.motionValue + 8 * e.chargeFrac) * style.count * (guarded ? 0.6 : 1) * (calm ? 0.5 : 1));
    for (let i = 0; i < count; i++) {
      let vx = -dx * 0.55 + (this.rand() - 0.5) * 1.8;
      let vy = 0.35 + this.rand() * 0.9;
      let vz = -dz * 0.55 + (this.rand() - 0.5) * 1.8;
      const len = Math.hypot(vx, vy, vz) || 1;
      const speed = (3.5 + this.rand() * 4.5) * style.speed * (0.8 + 0.3 * e.motionValue);
      vx = (vx / len) * speed;
      vy = (vy / len) * speed;
      vz = (vz / len) * speed;
      if (this.sparks.length >= SPARK_CAPACITY) this.sparks.shift();
      this.sparks.push({
        x: px,
        y: py,
        z: pz,
        vx,
        vy,
        vz,
        age: 0,
        life: 0.28 + this.rand() * 0.22,
        size: (0.09 + this.rand() * 0.07) * style.size,
        style
      });
    }
    if (!calm) {
      if (this.flashes.length >= FLASH_CAPACITY) this.flashes.shift();
      this.flashes.push({ x: px, y: py, z: pz, age: 0, life: 0.14, size: (0.5 + 0.25 * e.motionValue) * style.size, color: style.core });
    }

    // Hitstop freezes both actors' drawn animation; the core keeps ticking.
    const seconds = frames * TICK_SECONDS;
    this.hooks.holdPose(e.attacker, seconds);
    this.hooks.holdPose(e.target, seconds);
    this.hold = Math.max(this.hold, seconds);

    // Only hits the player is part of shake the camera.
    if (!guarded && (e.attacker === this.playerId || e.target === this.playerId)) {
      this.shake.add(hitTrauma(e.motionValue, e.chargeFrac, e.target === this.playerId));
    }
  }

  /**
   * Advances everything by `dt` seconds (0 while the simulation is frozen: nothing ages, but
   * the buffers still refill from current state) and refills the GPU buffers.
   */
  update(actors: DrawActor[], dt: number, camera: PerspectiveCamera, viewportHeightPx: number): void {
    this.playerId = actors[0]?.id ?? -1;
    this.clock += dt;
    this.pointMaterial.uniforms.uScale.value = viewportHeightPx / (2 * Math.tan((camera.fov * Math.PI) / 360));

    // Shake time stands still during hitstop, so the camera holds with the actors.
    this.hold = Math.max(0, this.hold - dt);
    this.shake.update(this.hold > 0 ? 0 : dt);

    this.stepSparks(dt);
    this.stepTrails(actors, dt);
    this.stepAuras(actors);
  }

  /** Shakes the draw camera. Call after the camera rig and right before rendering. */
  applyCamera(camera: PerspectiveCamera): void {
    this.shake.apply(camera);
  }

  private stepSparks(dt: number): void {
    const cloud = this.sparkCloud;
    cloud.clear();
    for (let i = this.sparks.length - 1; i >= 0; i--) {
      const s = this.sparks[i];
      s.age += dt;
      if (s.age >= s.life) {
        this.sparks.splice(i, 1);
        continue;
      }
      s.vy -= GRAVITY * dt;
      s.x += s.vx * dt;
      s.y += s.vy * dt;
      s.z += s.vz * dt;
      if (s.y < 0.02) {
        s.y = 0.02;
        s.vy *= -0.3;
      }
    }
    for (const s of this.sparks) {
      const u = s.age / s.life;
      const fade = 1 - u * u;
      const [cr, cg, cb] = s.style.core;
      const [er, eg, eb] = s.style.ember;
      const r = cr + (er - cr) * u;
      const g = cg + (eg - cg) * u;
      const b = cb + (eb - cb) * u;
      // A streak is three points along the velocity: head, middle, tail.
      for (let k = 0; k < 3; k++) {
        const back = k * 0.022;
        cloud.push(s.x - s.vx * back, s.y - s.vy * back, s.z - s.vz * back, s.size * (1 - k * 0.28), r, g, b, fade * (1 - k * 0.3));
      }
    }
    for (let i = this.flashes.length - 1; i >= 0; i--) {
      const f = this.flashes[i];
      f.age += dt;
      if (f.age >= f.life) {
        this.flashes.splice(i, 1);
        continue;
      }
    }
    for (const f of this.flashes) {
      const u = f.age / f.life;
      const [r, g, b] = f.color;
      cloud.push(f.x, f.y, f.z, f.size * (0.6 + 0.8 * u), r, g, b, (1 - u) * 0.9);
    }
    cloud.commit();
  }

  private stepTrails(actors: DrawActor[], dt: number): void {
    const seen = new Set<number>();
    for (const a of actors) {
      seen.add(a.id);
      const info = a.moveId !== 0 ? this.resolveMove(a.moveId) : undefined;
      const style = info ? trailStyleForAnim(info.anim) : null;
      // The strike third of the move stands in for the hitbox's active frames until the
      // resolver supplies `strikeStart` and `recoveryStart` from content.
      const striking = info !== undefined && movePhase(info, a.moveFrame) === 'strike';
      let emit: { tip: Vector3; base: Vector3; style: NonNullable<typeof style> } | null = null;
      if (style && striking && !this.hooks.isHeld(a.id) && this.hooks.weaponTip(a.id, this.tip, this.base)) {
        emit = { tip: this.tip, base: this.base, style };
      }
      let trail = this.trails.get(a.id);
      if (!trail && emit) {
        trail = new Trail(this.trailMaterial);
        this.trails.set(a.id, trail);
        this.scene.add(trail.mesh);
      }
      trail?.step(dt, emit);
    }
    for (const [id, trail] of this.trails) {
      if (seen.has(id)) continue;
      trail.step(dt, null);
      if (trail.sampleCount === 0) this.dropTrail(id, trail);
    }
  }

  private dropTrail(id: number, trail: Trail): void {
    this.scene.remove(trail.mesh);
    trail.dispose();
    this.trails.delete(id);
  }

  /**
   * Status auras: a ring of rising motes around an actor for each set `statusMask` bit that
   * `STATUS_AURAS` knows. Motes are a pure function of the clock, so they hold no state.
   */
  private stepAuras(actors: DrawActor[]): void {
    const cloud = this.auraCloud;
    cloud.clear();
    for (const a of actors) {
      if ((a.flags & 1) !== 0) continue; // dead
      const colors: Array<[number, number, number]> = [];
      for (const entry of STATUS_AURAS) if ((a.statusMask & entry.bit) !== 0) colors.push(entry.color);
      const forced = this.auraPreview.get(a.id);
      if (forced) colors.push(forced);
      colors.forEach((c, layer) => {
        for (let k = 0; k < AURA_MOTES; k++) {
          const phase = k / AURA_MOTES;
          const rise = (this.clock * 0.5 + phase + layer * 0.37) % 1;
          const ang = this.clock * 1.6 + phase * Math.PI * 2 + layer;
          const radius = 0.55 + 0.1 * Math.sin(rise * Math.PI * 2 + k);
          const fade = Math.sin(rise * Math.PI);
          cloud.push(a.x + Math.cos(ang) * radius, a.y + 0.1 + rise * 1.8, a.z + Math.sin(ang) * radius, 0.09, c[0], c[1], c[2], fade * 0.85);
        }
      });
    }
    cloud.commit();
  }

  /** Dev hook: forces an aura of `color` on an actor (null clears it) to check the renderer. */
  previewAura(id: number, color: [number, number, number] | null): void {
    if (color) this.auraPreview.set(id, color);
    else this.auraPreview.delete(id);
  }

  /** Dev hook: the same effects a `Hit` event with these values would start. */
  previewHit(e: Partial<HitEvent> & Pick<HitEvent, 'point'>): void {
    this.onHit({
      kind: 'hit',
      attacker: this.playerId,
      target: -2,
      moveNum: 0,
      hitbox: 0,
      motionValue: 1,
      chargeFrac: 0,
      dir: [0, 0, 1],
      damage: 0,
      result: 0,
      flags: 0,
      hpAfter: 0,
      ...e
    } as HitEvent);
  }

  debug(): VfxDebug {
    const o = this.shake.offset();
    return {
      sparks: this.sparks.length,
      flashes: this.flashes.length,
      trails: [...this.trails].map(([id, t]) => ({ id, samples: t.sampleCount })),
      auraPoints: this.auraCloud.count,
      hold: this.hold,
      hitsSeen: this.hitsSeen,
      shake: { trauma: this.shake.trauma, x: o.x, y: o.y, roll: o.roll, reducedMotion: this.shake.reducedMotion }
    };
  }

  dispose(): void {
    for (const [id, trail] of [...this.trails]) this.dropTrail(id, trail);
    this.scene.remove(this.sparkCloud.points, this.auraCloud.points);
    this.sparkCloud.dispose();
    this.auraCloud.dispose();
    this.pointMaterial.dispose();
    this.trailMaterial.dispose();
    this.shake.dispose();
  }
}
