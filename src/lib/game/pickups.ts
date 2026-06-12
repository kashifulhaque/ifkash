// Ammo pickups dropped by killed NPCs: a small glowing box the player
// collects just by walking over it (no key press, unlike loot crates).
import * as THREE from 'three';
import { terrainHeight } from './terrain';

export const PICKUP_RANGE = 1.5;
export const PICKUP_DESPAWN_DIST = 60;

const shellMat = new THREE.MeshLambertMaterial({ color: 0x3a4a2a, flatShading: true });
const brassMat = new THREE.MeshLambertMaterial({
  color: 0xd9a441,
  flatShading: true,
  emissive: 0xd9a441,
  emissiveIntensity: 0.45
});
const boxGeo = new THREE.BoxGeometry(0.42, 0.3, 0.3);
const stripGeo = new THREE.BoxGeometry(0.44, 0.1, 0.32);

export class AmmoPickup {
  group = new THREE.Group();
  /** Randomized bullet count granted on collection. */
  amount: number;
  private time = 0;
  private baseY = 0;

  constructor(x: number, z: number) {
    this.amount = 6 + Math.floor(Math.random() * 10); // 6–15 bullets
    const box = new THREE.Mesh(boxGeo, shellMat);
    const strip = new THREE.Mesh(stripGeo, brassMat);
    this.group.add(box, strip);
    this.baseY = terrainHeight(x, z) + 0.35;
    this.group.position.set(x, this.baseY, z);
  }

  update(dt: number) {
    this.time += dt;
    this.group.rotation.y += dt * 1.6;
    this.group.position.y = this.baseY + Math.sin(this.time * 3) * 0.07;
  }

  distanceTo(x: number, z: number): number {
    return Math.hypot(this.group.position.x - x, this.group.position.z - z);
  }
}
