// Drivable cars: arcade physics over the procedural terrain, spawned parked
// on settlement outskirts from the deterministic village plans.
import * as THREE from 'three';
import type { InputState } from './input';
import { resolveCircleAABB, type AABB } from './collision';
import { terrainHeight, roadFactor, villagePlan, REGION, type CarSlot } from './terrain';
import { loadModel, cloneStatic, VEHICLE_MODELS } from './assets';
import { normalizeHeight } from './chunks';

const CAR_HEIGHT = 1.5;
const RADIUS = 1.3; // collision circle
const ACCEL = 14;
const BRAKE = 26;
const REVERSE_ACCEL = 9;
const MAX_FWD = 16;
const MAX_REV = 5;
const COAST_DECEL = 3.5; // rolling to a stop on a road
const OFFROAD_DECEL = 5.5; // extra drag on grass
const SPAWN_DIST = 150;
const DESPAWN_DIST = 180;

export class Vehicle {
  group = new THREE.Group();
  x: number;
  z: number;
  yaw: number;
  speed = 0;
  slotKey: string;

  constructor(slot: CarSlot, modelIndex: number) {
    this.x = slot.x;
    this.z = slot.z;
    this.yaw = slot.rot;
    this.slotKey = slot.key;
    const url = VEHICLE_MODELS[modelIndex % VEHICLE_MODELS.length];
    loadModel(url)
      .then((gltf) => {
        const obj = cloneStatic(gltf);
        normalizeHeight(obj, CAR_HEIGHT);
        // Rest the wheels on the ground regardless of model origin
        const box = new THREE.Box3().setFromObject(obj);
        obj.position.y -= box.min.y;
        this.group.add(obj);
      })
      .catch(() => this.group.add(makeFallbackCar()));
    this.pose();
  }

  /** Driver-eye position in world space — just above the roofline so the
   * double-sided body shell never blocks the view. */
  seatWorld(out: THREE.Vector3): THREE.Vector3 {
    return out.set(this.x, terrainHeight(this.x, this.z) + CAR_HEIGHT + 0.25, this.z);
  }

  /** Spot beside the driver door, for stepping out. */
  exitSpot(out: THREE.Vector3): THREE.Vector3 {
    const rx = Math.cos(this.yaw);
    const rz = -Math.sin(this.yaw);
    out.set(this.x + rx * 2.2, 0, this.z + rz * 2.2);
    out.y = terrainHeight(out.x, out.z);
    return out;
  }

  collider(): AABB {
    return {
      minX: this.x - RADIUS,
      maxX: this.x + RADIUS,
      minZ: this.z - RADIUS,
      maxZ: this.z + RADIUS
    };
  }

  update(dt: number, input: InputState, colliders: AABB[]) {
    // Throttle/brake: W accelerates, S brakes then reverses
    const throttle = input.moveY;
    if (throttle > 0) {
      this.speed += (this.speed < 0 ? BRAKE : ACCEL) * throttle * dt;
    } else if (throttle < 0) {
      this.speed += (this.speed > 0 ? BRAKE : REVERSE_ACCEL) * throttle * dt;
    } else {
      const decel = COAST_DECEL + OFFROAD_DECEL * (1 - roadFactor(this.x, this.z));
      const drop = decel * dt;
      this.speed = Math.abs(this.speed) <= drop ? 0 : this.speed - Math.sign(this.speed) * drop;
    }
    this.speed = Math.max(-MAX_REV, Math.min(MAX_FWD, this.speed));

    // Speed-sensitive steering, reversed when backing up (like a real car)
    if (Math.abs(this.speed) > 0.3) {
      const turnRate = 2.2 - 1.3 * Math.min(1, Math.abs(this.speed) / MAX_FWD);
      this.yaw -= input.moveX * turnRate * dt * Math.sign(this.speed);
    }

    let nx = this.x + Math.sin(this.yaw) * this.speed * dt;
    let nz = this.z + Math.cos(this.yaw) * this.speed * dt;
    let hit = false;
    for (const box of colliders) {
      const [rx, rz] = resolveCircleAABB(nx, nz, RADIUS, box);
      if (rx !== nx || rz !== nz) hit = true;
      nx = rx;
      nz = rz;
    }
    // Crunch once on a fast impact; below that just scrape and slide so the
    // car can still push along walls instead of sticking to them
    if (hit && Math.abs(this.speed) > 5) this.speed *= 0.4;
    this.x = nx;
    this.z = nz;
    this.pose();
  }

  /** Place the chassis on the terrain with pitch/roll from sampled heights. */
  private pose() {
    const fx = Math.sin(this.yaw);
    const fz = Math.cos(this.yaw);
    const rx = Math.cos(this.yaw);
    const rz = -Math.sin(this.yaw);
    const hC = terrainHeight(this.x, this.z);
    const hF = terrainHeight(this.x + fx * 1.4, this.z + fz * 1.4);
    const hB = terrainHeight(this.x - fx * 1.4, this.z - fz * 1.4);
    const hR = terrainHeight(this.x + rx * 0.9, this.z + rz * 0.9);
    const hL = terrainHeight(this.x - rx * 0.9, this.z - rz * 0.9);
    this.group.position.set(this.x, hC, this.z);
    this.group.rotation.set(
      -Math.atan2(hF - hB, 2.8),
      this.yaw,
      Math.atan2(hR - hL, 1.8),
      'YXZ'
    );
  }
}

// Procedural stand-in if a GLB fails to load: body + cabin + four wheels.
const carBodyMat = new THREE.MeshLambertMaterial({ color: 0xc0392b, flatShading: true });
const carDarkMat = new THREE.MeshLambertMaterial({ color: 0x2c3e50, flatShading: true });

function makeFallbackCar(): THREE.Group {
  const g = new THREE.Group();
  const body = new THREE.Mesh(new THREE.BoxGeometry(1.7, 0.55, 3.6), carBodyMat);
  body.position.y = 0.55;
  const cabin = new THREE.Mesh(new THREE.BoxGeometry(1.5, 0.5, 1.8), carDarkMat);
  cabin.position.set(0, 1.05, -0.2);
  g.add(body, cabin);
  for (const sx of [-1, 1]) {
    for (const sz of [-1, 1]) {
      const wheel = new THREE.Mesh(new THREE.CylinderGeometry(0.32, 0.32, 0.25, 10), carDarkMat);
      wheel.rotation.z = Math.PI / 2;
      wheel.position.set(sx * 0.85, 0.32, sz * 1.15);
      g.add(wheel);
    }
  }
  return g;
}

// Spawns/despawns parked cars from the village plans around the player and
// tracks which slots have been driven off.
export class VehicleManager {
  private scene: THREE.Scene;
  private parked = new Map<string, Vehicle>();
  private taken = new Set<string>();

  constructor(scene: THREE.Scene) {
    this.scene = scene;
  }

  update(px: number, pz: number) {
    const rx = Math.floor(px / REGION);
    const rz = Math.floor(pz / REGION);
    for (let dx = -1; dx <= 1; dx++) {
      for (let dz = -1; dz <= 1; dz++) {
        const plan = villagePlan(rx + dx, rz + dz);
        if (!plan) continue;
        for (let i = 0; i < plan.cars.length; i++) {
          const slot = plan.cars[i];
          if (this.taken.has(slot.key) || this.parked.has(slot.key)) continue;
          if (Math.hypot(slot.x - px, slot.z - pz) > SPAWN_DIST) continue;
          // Deterministic paint job per slot
          const v = new Vehicle(slot, Math.abs((rx + dx) * 31 + (rz + dz) * 17 + i * 7));
          this.parked.set(slot.key, v);
          this.scene.add(v.group);
        }
      }
    }
    for (const [key, v] of this.parked) {
      if (Math.hypot(v.x - px, v.z - pz) > DESPAWN_DIST) {
        this.scene.remove(v.group);
        this.parked.delete(key);
      }
    }
  }

  nearestParked(x: number, z: number, range: number): Vehicle | null {
    let best: Vehicle | null = null;
    let bestD = range;
    for (const v of this.parked.values()) {
      const d = Math.hypot(v.x - x, v.z - z);
      if (d < bestD) {
        best = v;
        bestD = d;
      }
    }
    return best;
  }

  /** Player takes this car: it stops being "parked" for the session. */
  claim(v: Vehicle) {
    this.taken.add(v.slotKey);
    this.parked.delete(v.slotKey);
  }

  /** Player steps out: track the car at its new spot so it can be re-entered. */
  park(v: Vehicle) {
    this.parked.set(v.slotKey, v);
  }

  /** Parked-car colliders so the player and NPCs can't walk through them. */
  colliders(x: number, z: number): AABB[] {
    const out: AABB[] = [];
    for (const v of this.parked.values()) {
      if (Math.hypot(v.x - x, v.z - z) < 40) out.push(v.collider());
    }
    return out;
  }

  dispose() {
    for (const v of this.parked.values()) this.scene.remove(v.group);
    this.parked.clear();
  }
}
