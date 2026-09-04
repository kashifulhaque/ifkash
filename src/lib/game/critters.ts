import * as THREE from 'three';
import { PLANET_RADIUS, isOcean, tangentBasis, walkRadius } from './planet';
import type { ColliderGrid } from './collision';

/** Footprint radius of a critter, in surface units. */
const CRITTER_RADIUS = 0.5;
/** Seconds an animal stays pleased after a pat. */
const HAPPY_TIME = 2.4;

const _right = new THREE.Vector3();
const _fwd = new THREE.Vector3();
const _to = new THREE.Vector3();
const _m = new THREE.Matrix4();

/**
 * An animal that ambles around its home spot on the sphere: pick a nearby
 * target, walk there, pause, repeat. Walk up and press `E` to pet it, and it
 * stops to face you and bounce for a couple of seconds.
 */
export class Critter {
  group: THREE.Group;
  /** Unit direction of the animal on the sphere. Mutated in place as it walks. */
  readonly dir: THREE.Vector3;
  /** Stable id, for example `sheep-2`; the interaction prompt is keyed by it. */
  readonly id: string;
  /** Plain-language name used in the prompt, for example "polar bear". */
  readonly name: string;
  /** True once the player has petted this animal. */
  petted = false;
  private heading = new THREE.Vector3();
  private home: THREE.Vector3;
  private target: THREE.Vector3 | null = null;
  private wait = 0;
  private phase = 0;
  private stuck = 0;
  /** Seconds left of the pleased bounce; movement is suspended while it runs. */
  private happy = 0;
  private lift = 0;
  private legs: THREE.Object3D[] = [];

  constructor(
    model: THREE.Group,
    id: string,
    name: string,
    home: THREE.Vector3,
    private roam: number,
    private speed: number,
    seed: number,
    private colliders?: ColliderGrid
  ) {
    this.group = model;
    this.id = id;
    this.name = name;
    this.home = home.clone().normalize();
    this.colliders?.resolve(this.home, CRITTER_RADIUS);
    this.dir = this.home.clone();
    tangentBasis(this.dir, seed * 2.1, _right, _fwd);
    this.heading.copy(_fwd);
    this.wait = seed % 3;
    // Legs are the short boxes near the ground; wiggle them while walking.
    model.traverse((o) => {
      if (o instanceof THREE.Mesh && o.position.y < 0.2 && o.position.y > 0) this.legs.push(o);
    });
    this.place();
  }

  /** True while the animal is still enjoying a pat. */
  get pleased(): boolean {
    return this.happy > 0;
  }

  /** Take a pat: turn toward `from`, stand still, and bounce. */
  pet(from: THREE.Vector3): void {
    this.petted = true;
    this.happy = HAPPY_TIME;
    this.target = null;
    this.wait = 0;
    this.stuck = 0;
    _to.copy(from).addScaledVector(this.dir, -from.dot(this.dir));
    if (_to.lengthSq() > 1e-8) this.heading.copy(_to).normalize();
  }

  update(dt: number, time: number): void {
    if (this.happy > 0) {
      this.happy -= dt;
      if (this.happy > 0) {
        this.rejoice(dt, time);
        return;
      }
      // Settle back onto four feet before wandering off again.
      this.lift = 0;
      this.group.scale.set(1, 1, 1);
      this.place();
    }
    if (this.wait > 0) {
      this.wait -= dt;
      this.idle(time);
      return;
    }
    if (!this.target) {
      // New wander target within `roam` units of home, on land.
      for (let tries = 0; tries < 6; tries++) {
        tangentBasis(this.home, Math.random() * Math.PI * 2, _right, _fwd);
        const dist = Math.random() * this.roam;
        const t = this.home.clone().multiplyScalar(Math.cos(dist / PLANET_RADIUS)).addScaledVector(_fwd, Math.sin(dist / PLANET_RADIUS)).normalize();
        if (!isOcean(t) && !this.colliders?.blocked(t, CRITTER_RADIUS)) {
          this.target = t;
          break;
        }
      }
      if (!this.target) {
        this.wait = 2;
        return;
      }
    }
    // Tangent direction toward the target.
    _to.copy(this.target).addScaledVector(this.dir, -this.target.dot(this.dir));
    const remaining = this.dir.angleTo(this.target) * PLANET_RADIUS;
    if (remaining < 0.15 || _to.lengthSq() < 1e-8) {
      this.target = null;
      this.wait = 1.5 + Math.random() * 3;
      return;
    }
    _to.normalize();
    this.heading.lerp(_to, Math.min(1, dt * 4)).addScaledVector(this.dir, -this.heading.dot(this.dir)).normalize();
    const ang = (this.speed * dt) / PLANET_RADIUS;
    this.dir.multiplyScalar(Math.cos(ang)).addScaledVector(this.heading, Math.sin(ang)).normalize();
    if (this.colliders?.resolve(this.dir, CRITTER_RADIUS)) {
      // Pressed against something: drop the target after a moment and wander elsewhere.
      this.stuck += dt;
      if (this.stuck > 1) {
        this.stuck = 0;
        this.target = null;
        this.wait = 1 + Math.random() * 2;
        return;
      }
    } else {
      this.stuck = 0;
    }
    this.heading.addScaledVector(this.dir, -this.heading.dot(this.dir)).normalize();
    this.phase += dt * 9;
    this.legs.forEach((l, i) => {
      l.rotation.x = Math.sin(this.phase + (i % 2) * Math.PI) * 0.5;
    });
    this.place();
  }

  private idle(time: number): void {
    this.legs.forEach((l) => (l.rotation.x = 0));
    this.group.scale.y = 1 + Math.sin(time * 2 + this.phase) * 0.015;
  }

  /** Pleased bounce after a pat: little hops, a squash, and happy feet. */
  private rejoice(dt: number, time: number): void {
    this.phase += dt * 12;
    const bounce = Math.abs(Math.sin(time * 6.5));
    this.lift = bounce * 0.32;
    this.group.scale.set(1 + (1 - bounce) * 0.07, 1 - (1 - bounce) * 0.09, 1 + (1 - bounce) * 0.07);
    this.legs.forEach((l, i) => {
      l.rotation.x = Math.sin(this.phase + (i % 2) * Math.PI) * 0.3;
    });
    this.place();
  }

  private place(): void {
    const r = walkRadius(this.dir) + this.lift;
    this.group.position.copy(this.dir).multiplyScalar(r);
    _right.crossVectors(this.dir, this.heading).normalize();
    _m.makeBasis(_right, this.dir, this.heading);
    this.group.quaternion.setFromRotationMatrix(_m);
  }
}

// ---------------------------------------------------------------- hearts

/** Hearts in flight at once. A pat spends three, so the pool never runs dry. */
const HEART_COUNT = 15;
/** Seconds a heart takes to rise and fade. */
const HEART_LIFE = 1.5;

const _hv = new THREE.Vector3();

/** A small heart outline, about one unit tall, centred on its origin. */
function heartGeometry(): THREE.ShapeGeometry {
  const s = new THREE.Shape();
  s.moveTo(0, -0.5);
  s.bezierCurveTo(0.62, 0.05, 0.34, 0.72, 0, 0.32);
  s.bezierCurveTo(-0.34, 0.72, -0.62, 0.05, 0, -0.5);
  return new THREE.ShapeGeometry(s, 10);
}

type Heart = { mesh: THREE.Mesh; material: THREE.MeshBasicMaterial; life: number; up: THREE.Vector3; drift: THREE.Vector3 };

/**
 * Little hearts that rise from a petted animal. One shared geometry, a handful
 * of billboarded quads, and no allocation once the pool is built.
 */
export class Hearts {
  group = new THREE.Group();
  private pool: Heart[] = [];
  private next = 0;

  constructor() {
    const geo = heartGeometry();
    for (let i = 0; i < HEART_COUNT; i++) {
      // Tone mapping would wash the pink out to grey, so these opt out of it.
      const material = new THREE.MeshBasicMaterial({ color: 0xff6f96, transparent: true, opacity: 0, depthWrite: false, side: THREE.DoubleSide, toneMapped: false });
      const mesh = new THREE.Mesh(geo, material);
      mesh.visible = false;
      this.group.add(mesh);
      this.pool.push({ mesh, material, life: 0, up: new THREE.Vector3(0, 1, 0), drift: new THREE.Vector3() });
    }
  }

  /** Send `count` hearts drifting up from `pos` along the surface normal `up`. */
  burst(pos: THREE.Vector3, up: THREE.Vector3, count = 3): void {
    for (let i = 0; i < count; i++) {
      const h = this.pool[this.next];
      this.next = (this.next + 1) % this.pool.length;
      h.life = HEART_LIFE;
      h.up.copy(up).normalize();
      tangentBasis(h.up, Math.random() * Math.PI * 2, _right, _fwd);
      h.drift.copy(_right).multiplyScalar(0.5 + Math.random() * 0.5);
      h.mesh.position.copy(pos).addScaledVector(h.up, 0.9 + Math.random() * 0.4).addScaledVector(_fwd, (Math.random() - 0.5) * 0.4);
      h.mesh.visible = true;
      h.material.opacity = 0;
      h.material.color.setHSL(0.94 - Math.random() * 0.07, 0.85, 0.62);
    }
  }

  /** Float the live hearts upward and keep them facing the camera. */
  update(dt: number, camera: THREE.Quaternion): void {
    let any = false;
    for (const h of this.pool) {
      if (h.life <= 0) continue;
      h.life -= dt;
      if (h.life <= 0) {
        h.mesh.visible = false;
        continue;
      }
      any = true;
      const age = 1 - h.life / HEART_LIFE;
      _hv.copy(h.up).multiplyScalar(1.5 * dt).addScaledVector(h.drift, dt * (1 - age));
      h.mesh.position.add(_hv);
      h.mesh.quaternion.copy(camera);
      const s = 0.26 + age * 0.18;
      h.mesh.scale.setScalar(s);
      // Pop in quickly, then fade over the rest of the life.
      h.material.opacity = Math.min(1, age * 6) * (1 - age) * 0.95;
    }
    this.group.visible = any;
  }
}
