// Catalyst and spell effects: motes that drift around a catalyst's head, and cast bursts by
// school and element. A stylus (Formula) throws cool aether motes and a bright bolt; a
// reliquary (Litany) breathes warm candle sparks and rising embers. All particles live in one
// point cloud and are refilled every frame, like the combat VFX (`render/vfx/points.ts`).
// Render-only: it reads weapon transforms and never writes simulation state.

import { type Object3D, type PerspectiveCamera, type Points, Vector3 } from 'three';
import { PointCloud, makePointMaterial } from '../vfx/points';
import { ELEMENT_COLOR, SCHOOL_COLOR } from './weapons';

type RGB = [number, number, number];

const rgb = (hex: number): RGB => [((hex >> 16) & 255) / 255, ((hex >> 8) & 255) / 255, (hex & 255) / 255];

const CAPACITY = 900;
/** Idle motes drawn per catalyst. */
const IDLE_MOTES = 5;

interface Particle {
  p: Vector3;
  v: Vector3;
  age: number;
  life: number;
  size: number;
  color: RGB;
  /** Upward acceleration (embers rise), meters per second squared. */
  lift: number;
  drag: number;
}

export interface CastKind {
  school: 'Formula' | 'Litany';
  /** Overrides the school's color when a spell has an element. */
  element?: string | null;
}

interface Tracked {
  object: Object3D;
  /** The effect's anchor in the object's own frame. */
  at: Vector3;
  color: RGB;
  school: string;
  phase: number;
}

const tmp = new Vector3();

export class SpellFx {
  readonly points: Points;
  private cloud: PointCloud;
  private material = makePointMaterial();
  private live: Particle[] = [];
  private tracked = new Map<Object3D, Tracked>();
  private clock = 0;

  constructor() {
    this.cloud = new PointCloud(CAPACITY, this.material);
    this.points = this.cloud.points;
  }

  /** Starts the idle shimmer on a catalyst. `at` is the effect anchor in `object`'s frame. */
  track(object: Object3D, at: [number, number, number], school: string, element: string | null = null): void {
    this.tracked.set(object, {
      object,
      at: new Vector3(...at),
      color: rgb(element ? ELEMENT_COLOR[element] : (SCHOOL_COLOR[school] ?? SCHOOL_COLOR.Formula)),
      school,
      phase: this.tracked.size * 1.7
    });
  }

  untrack(object: Object3D): void {
    this.tracked.delete(object);
  }

  /** A cast from `from` toward `dir` (a unit vector): the bolt, the flash and the sparks. */
  cast(from: Vector3, dir: Vector3, kind: CastKind): void {
    const color = rgb(kind.element ? ELEMENT_COLOR[kind.element] : SCHOOL_COLOR[kind.school]);
    const hot: RGB = [Math.min(color[0] + 0.5, 1), Math.min(color[1] + 0.5, 1), Math.min(color[2] + 0.5, 1)];
    if (kind.school === 'Formula') {
      // A tight ring of motes at the nib, then a fast bolt with a tail.
      for (let i = 0; i < 18; i++) {
        const a = (i / 18) * Math.PI * 2;
        const side = new Vector3(Math.cos(a), Math.sin(a), 0.2).multiplyScalar(0.9);
        this.spawn(from, side, 0.8, 0.11, color, 0, 3);
      }
      for (let i = 0; i < 26; i++) {
        const k = i / 26;
        const v = dir.clone().multiplyScalar(5 + k * 3);
        v.x += (Math.random() - 0.5) * 0.3;
        v.y += (Math.random() - 0.5) * 0.3;
        this.spawn(from, v, 0.7 + k * 0.5, 0.17 - k * 0.08, k < 0.2 ? hot : color, 0, 2.2);
      }
    } else {
      // A warm flare: embers lift off the shrine and a few sparks fly out.
      for (let i = 0; i < 28; i++) {
        const v = new Vector3((Math.random() - 0.5) * 1.2, 0.4 + Math.random() * 1.2, (Math.random() - 0.5) * 1.2);
        this.spawn(from, v, 1.1 + Math.random() * 0.7, 0.1 + Math.random() * 0.06, i % 3 === 0 ? hot : color, 1.8, 1.2);
      }
      for (let i = 0; i < 10; i++) {
        const v = dir.clone().multiplyScalar(3.5 + Math.random() * 2);
        v.y += (Math.random() - 0.3) * 0.8;
        this.spawn(from, v, 0.8, 0.11, hot, -2, 1.5);
      }
    }
    // A bright flash at the source.
    this.spawn(from, new Vector3(), 0.22, 0.55, hot, 0, 0);
  }

  private spawn(from: Vector3, v: Vector3, life: number, size: number, color: RGB, lift: number, drag: number): void {
    if (this.live.length >= CAPACITY - this.tracked.size * IDLE_MOTES - 8) return;
    this.live.push({ p: from.clone(), v: v.clone(), age: 0, life, size, color, lift, drag });
  }

  /** Advances the particles and refills the cloud. `viewportPx` is the canvas height in pixels. */
  update(dt: number, camera: PerspectiveCamera, viewportPx: number): void {
    this.clock += dt;
    this.material.uniforms.uScale.value = viewportPx / (2 * Math.tan((camera.fov * Math.PI) / 360));
    const c = this.cloud;
    c.clear();
    for (const t of this.tracked.values()) {
      t.object.updateWorldMatrix(true, false);
      const o = tmp.copy(t.at).applyMatrix4(t.object.matrixWorld);
      const [r, g, b] = t.color;
      // The core glow, steady.
      c.push(o.x, o.y, o.z, 0.17 + Math.sin(this.clock * 3 + t.phase) * 0.02, r, g, b, 0.55);
      for (let i = 0; i < IDLE_MOTES; i++) {
        const a = this.clock * (1.2 + i * 0.17) + t.phase + i * 1.9;
        if (t.school === 'Litany') {
          // Embers: rise from the flame and fade, each on its own cycle.
          const u = (this.clock * 0.55 + i / IDLE_MOTES + t.phase) % 1;
          c.push(o.x + Math.sin(a) * 0.025, o.y + u * 0.2, o.z + Math.cos(a) * 0.025, 0.045 * (1 - u), r, g, b, 0.9 * (1 - u));
        } else {
          // Aether motes orbit the crystal.
          const rad = 0.075 + (i % 2) * 0.03;
          c.push(o.x + Math.cos(a) * rad, o.y + Math.sin(a * 0.7) * 0.05, o.z + Math.sin(a) * rad, 0.04, r, g, b, 0.9);
        }
      }
    }
    for (let i = this.live.length - 1; i >= 0; i--) {
      const q = this.live[i];
      q.age += dt;
      if (q.age >= q.life) {
        this.live.splice(i, 1);
        continue;
      }
      q.v.multiplyScalar(Math.max(0, 1 - q.drag * dt));
      q.v.y += q.lift * dt;
      q.p.addScaledVector(q.v, dt);
      const u = q.age / q.life;
      c.push(q.p.x, q.p.y, q.p.z, q.size * (1 - u * 0.6), q.color[0], q.color[1], q.color[2], 1 - u);
    }
    c.commit();
  }

  dispose(): void {
    this.cloud.dispose();
    this.material.dispose();
    this.live = [];
    this.tracked.clear();
  }
}
