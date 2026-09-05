// Starlight shards: a dozen small glowing crystals scattered over each planet.
// They have no story to tell, so unlike the wonders there is nothing to press;
// walking over one picks it up. Which ones are gone is remembered per seed.

import * as THREE from 'three';
import { isOcean } from './biomes';
import type { ColliderGrid } from './collision';
import { SHARD_COUNT } from './journal';
import { emitter, type Emitter, type LightPool } from './lights';
import { deriveSeed, seededRng } from './noise';
import { PLANET_RADIUS, tangentBasis, walkRadius } from './planet';
import { WONDERS, wonderDir } from './wonders';

/** How close the explorer has to pass, in surface units. */
const PICKUP_RANGE = 1.15;
/** Shards keep this far from each other and from the wonders, in surface units. */
const SPACING = 7;
const WONDER_CLEARANCE = 4;
/** Seconds the pickup flourish lasts. */
const POP_TIME = 0.45;
const COLOR = 0xd7c4ff;
const GLOW = 0xa98bff;

const _right = new THREE.Vector3();
const _fwd = new THREE.Vector3();
const _m = new THREE.Matrix4();
const _q = new THREE.Quaternion();

type Shard = {
  dir: THREE.Vector3;
  mesh: THREE.Mesh;
  ring: THREE.Mesh;
  light: Emitter;
  /** Quaternion that stands the shard on the local ground. */
  frameQ: THREE.Quaternion;
  found: boolean;
  /** Seconds left of the pickup flourish; zero when idle or gone. */
  pop: number;
};

/**
 * Seeded shard positions: uniformly random over the sphere, rejecting the sea,
 * anything solid, the wonders' own clearings, and other shards.
 */
function placeShards(seed: number, colliders: ColliderGrid): THREE.Vector3[] {
  const rng = seededRng(deriveSeed(seed, 21));
  const wonders = WONDERS.map((w) => wonderDir(w));
  const out: THREE.Vector3[] = [];
  const cand = new THREE.Vector3();
  for (let tries = 0; tries < 4000 && out.length < SHARD_COUNT; tries++) {
    const z = rng() * 2 - 1;
    const phi = rng() * Math.PI * 2;
    const r = Math.sqrt(1 - z * z);
    cand.set(r * Math.cos(phi), r * Math.sin(phi), z);
    if (isOcean(cand)) continue;
    if (colliders.blocked(cand, 0.7)) continue;
    if (wonders.some((w) => w.angleTo(cand) * PLANET_RADIUS < WONDER_CLEARANCE)) continue;
    if (out.some((s) => s.angleTo(cand) * PLANET_RADIUS < SPACING)) continue;
    out.push(cand.clone());
  }
  return out;
}

export class Shards {
  readonly group = new THREE.Group();
  private shards: Shard[] = [];
  private material: THREE.MeshStandardMaterial;

  constructor(seed: number, colliders: ColliderGrid, lights: LightPool, found: number[]) {
    this.material = new THREE.MeshStandardMaterial({
      color: COLOR,
      emissive: GLOW,
      emissiveIntensity: 1.4,
      roughness: 0.25,
      flatShading: true,
      transparent: true
    });
    const geo = new THREE.OctahedronGeometry(0.26, 0);
    geo.scale(0.7, 1.35, 0.7);
    const ringGeo = new THREE.RingGeometry(0.5, 0.62, 24);
    const dirs = placeShards(seed, colliders);
    dirs.forEach((dir, i) => {
      const mesh = new THREE.Mesh(geo, this.material.clone());
      const ring = new THREE.Mesh(
        ringGeo,
        new THREE.MeshBasicMaterial({ color: COLOR, transparent: true, opacity: 0.35, side: THREE.DoubleSide, depthWrite: false })
      );
      tangentBasis(dir, 0, _right, _fwd);
      _m.makeBasis(_right, dir, _fwd);
      const frameQ = new THREE.Quaternion().setFromRotationMatrix(_m);
      ring.quaternion.copy(frameQ).multiply(_q.setFromAxisAngle(new THREE.Vector3(1, 0, 0), -Math.PI / 2));
      ring.position.copy(dir).multiplyScalar(walkRadius(dir) + 0.06);
      const light = lights.add(emitter(mesh.position, GLOW, 6, 6, { flicker: 0.2, phase: i * 2.3 }));
      const isFound = found.includes(i);
      const shard: Shard = { dir, mesh, ring, light, frameQ, found: isFound, pop: 0 };
      if (isFound) {
        mesh.visible = false;
        ring.visible = false;
        light.on = false;
      }
      this.group.add(mesh, ring);
      this.shards.push(shard);
    });
  }

  /** How many shards this planet has; a crowded seed can place fewer than `SHARD_COUNT`. */
  get total(): number {
    return this.shards.length;
  }

  get foundCount(): number {
    return this.shards.filter((s) => s.found).length;
  }

  /** Indices of the collected shards, for persistence. */
  get foundIndices(): number[] {
    return this.shards.flatMap((s, i) => (s.found ? [i] : []));
  }

  /** Pick up the shard under the explorer, if any. Returns its index, or -1. */
  collect(playerDir: THREE.Vector3): number {
    for (let i = 0; i < this.shards.length; i++) {
      const s = this.shards[i];
      if (s.found) continue;
      if (s.dir.angleTo(playerDir) * PLANET_RADIUS < PICKUP_RANGE) {
        s.found = true;
        s.pop = POP_TIME;
        s.light.on = false;
        s.ring.visible = false;
        return i;
      }
    }
    return -1;
  }

  update(dt: number, t: number): void {
    for (let i = 0; i < this.shards.length; i++) {
      const s = this.shards[i];
      if (s.found && s.pop <= 0) continue;
      const mat = s.mesh.material as THREE.MeshStandardMaterial;
      if (s.pop > 0) {
        // Pickup flourish: the shard leaps, swells, and fades.
        s.pop -= dt;
        const age = 1 - Math.max(0, s.pop) / POP_TIME;
        const lift = 0.9 + age * 1.6;
        s.mesh.position.copy(s.dir).multiplyScalar(walkRadius(s.dir) + lift);
        const scale = 1 + age * 1.8;
        s.mesh.scale.setScalar(scale);
        mat.opacity = 1 - age;
        mat.emissiveIntensity = 1.4 + age * 3;
        if (s.pop <= 0) s.mesh.visible = false;
        continue;
      }
      const bob = Math.sin(t * 2.1 + i * 1.3) * 0.12;
      s.mesh.position.copy(s.dir).multiplyScalar(walkRadius(s.dir) + 0.9 + bob);
      _q.setFromAxisAngle(s.dir, t * 1.4 + i);
      s.mesh.quaternion.copy(_q).multiply(s.frameQ);
      const pulse = 0.8 + Math.sin(t * 3.2 + i) * 0.3;
      mat.emissiveIntensity = 1.4 * pulse;
      s.light.intensity = 6 * pulse;
      (s.ring.material as THREE.MeshBasicMaterial).opacity = 0.25 + Math.sin(t * 3.2 + i) * 0.12;
    }
  }
}
