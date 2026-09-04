import * as THREE from 'three';
import { PLANET_RADIUS, isOcean, tangentBasis, walkRadius } from './planet';
import type { ColliderGrid } from './collision';

/** Footprint radius of a critter, in surface units. */
const CRITTER_RADIUS = 0.5;

const _right = new THREE.Vector3();
const _fwd = new THREE.Vector3();
const _to = new THREE.Vector3();
const _m = new THREE.Matrix4();

/**
 * An animal that ambles around its home spot on the sphere: pick a nearby
 * target, walk there, pause, repeat.
 */
export class Critter {
  group: THREE.Group;
  private dir: THREE.Vector3;
  private heading = new THREE.Vector3();
  private home: THREE.Vector3;
  private target: THREE.Vector3 | null = null;
  private wait = 0;
  private phase = 0;
  private stuck = 0;
  private legs: THREE.Object3D[] = [];

  constructor(
    model: THREE.Group,
    home: THREE.Vector3,
    private roam: number,
    private speed: number,
    seed: number,
    private colliders?: ColliderGrid
  ) {
    this.group = model;
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

  update(dt: number, time: number): void {
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

  private place(): void {
    const r = walkRadius(this.dir);
    this.group.position.copy(this.dir).multiplyScalar(r);
    _right.crossVectors(this.dir, this.heading).normalize();
    _m.makeBasis(_right, this.dir, this.heading);
    this.group.quaternion.setFromRotationMatrix(_m);
  }
}
