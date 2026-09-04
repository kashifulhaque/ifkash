// The rowboat: its mesh, the wake it leaves, and the shore scan that decides
// where it can be launched. The boat controller itself lives in the Game.

import * as THREE from 'three';
import { SEA_LEVEL, offsetDir, surfaceRadius, tangentBasis } from './planet';

const HULL = 0x8a5a3a;
const HULL_D = 0x5f3e28;
const DECK = 0xd9b98a;
const TRIM = 0xe8e0cc;

/** How far from the player, in surface units, the shore scan looks for water. */
export const LAUNCH_RANGE = 4.5;

/** Where the explorer sits, in boat space. The character's origin is at its feet. */
export const BOAT_SEAT = new THREE.Vector3(0, -0.1, -0.1);

function mesh(geo: THREE.BufferGeometry, color: number): THREE.Mesh {
  const m = new THREE.Mesh(geo, new THREE.MeshStandardMaterial({ color, roughness: 0.85, flatShading: true }));
  m.castShadow = true;
  return m;
}

/** Wedge that closes one end of the hull. Points along +Z, base at z = 0. */
function wedge(length: number): THREE.BufferGeometry {
  // A four-sided cone squashed to the hull's cross-section, then laid on its side.
  return new THREE.ConeGeometry(0.636, length, 4, 1, false, Math.PI / 4)
    .scale(1, 1, 0.61)
    .rotateX(Math.PI / 2)
    .translate(0, 0, length / 2);
}

/**
 * Low-poly rowboat. Local +Z is the bow, +Y is up, and the waterline is y = 0.
 * About 3.2 units long.
 */
export function buildBoat(): THREE.Group {
  const g = new THREE.Group();
  const hull = mesh(new THREE.BoxGeometry(0.9, 0.55, 1.6), HULL);
  hull.position.y = 0.025;
  const bow = mesh(wedge(0.8), HULL);
  bow.position.set(0, 0.025, 0.8);
  const stern = mesh(wedge(0.5), HULL_D);
  stern.rotation.y = Math.PI;
  stern.position.set(0, 0.025, -0.8);
  const floor = mesh(new THREE.BoxGeometry(0.72, 0.02, 1.42), DECK);
  floor.position.y = 0.31;
  floor.castShadow = false;
  const bench = mesh(new THREE.BoxGeometry(0.8, 0.08, 0.3), HULL_D);
  bench.position.set(0, 0.36, -0.1);
  const railL = mesh(new THREE.BoxGeometry(0.06, 0.06, 1.7), TRIM);
  railL.position.set(-0.44, 0.32, 0);
  const railR = railL.clone();
  railR.position.x = 0.44;
  g.add(hull, bow, stern, floor, bench, railL, railR);
  for (const side of [-1, 1]) {
    const oar = mesh(new THREE.CylinderGeometry(0.03, 0.03, 1.5, 5), HULL_D);
    oar.position.set(side * 0.72, 0.28, 0.15);
    oar.rotation.z = side * 1.15;
    oar.rotation.x = 0.25;
    const blade = mesh(new THREE.BoxGeometry(0.16, 0.03, 0.34), HULL_D);
    blade.position.set(side * 1.32, -0.02, 0.32);
    blade.rotation.z = side * 1.15;
    g.add(oar, blade);
  }
  return g;
}

const _d = new THREE.Vector3();

/**
 * Nearest direction within `LAUNCH_RANGE` of `from` where the water is deep
 * enough to float a boat, or `null` when the player is not near a shore.
 */
export function findLaunchPoint(from: THREE.Vector3): THREE.Vector3 | null {
  for (let dist = 0.8; dist <= LAUNCH_RANGE; dist += 0.4) {
    for (let k = 0; k < 16; k++) {
      const a = (k / 16) * Math.PI * 2;
      _d.copy(from);
      offsetDir(_d, Math.cos(a) * dist, Math.sin(a) * dist);
      if (surfaceRadius(_d) < SEA_LEVEL - 0.35) return _d.clone();
    }
  }
  return null;
}

type Ring = { mesh: THREE.Mesh; age: number };

const RING_LIFE = 1.7;
const _right = new THREE.Vector3();
const _fwd = new THREE.Vector3();
const _m = new THREE.Matrix4();

/** Flat rings that spread and fade behind a moving boat. */
export class Wake {
  group = new THREE.Group();
  private rings: Ring[] = [];
  private timer = 0;

  constructor(count = 14) {
    const geo = new THREE.RingGeometry(0.34, 0.5, 20);
    for (let i = 0; i < count; i++) {
      const mat = new THREE.MeshBasicMaterial({ color: 0xeaf6ff, transparent: true, opacity: 0, depthWrite: false, side: THREE.DoubleSide });
      const m = new THREE.Mesh(geo, mat);
      m.visible = false;
      this.group.add(m);
      this.rings.push({ mesh: m, age: RING_LIFE });
    }
  }

  /**
   * Age the rings and, while the boat is under way, drop a new one at its stern.
   * `dir` is the boat's unit direction and `heading` its tangent forward.
   */
  update(dt: number, dir: THREE.Vector3, heading: THREE.Vector3, speed: number): void {
    this.timer -= dt;
    if (speed > 1.2 && this.timer <= 0) {
      this.timer = 0.24 - Math.min(0.1, speed * 0.008);
      const r = this.rings.find((x) => x.age >= RING_LIFE);
      if (r) {
        r.age = 0;
        r.mesh.visible = true;
        r.mesh.position.copy(dir).multiplyScalar(SEA_LEVEL + 0.04).addScaledVector(heading, -1.0);
        tangentBasis(dir, 0, _right, _fwd);
        _m.makeBasis(_right, dir, _fwd);
        r.mesh.quaternion.setFromRotationMatrix(_m);
        r.mesh.rotateX(-Math.PI / 2);
      }
    }
    for (const r of this.rings) {
      if (r.age >= RING_LIFE) continue;
      r.age += dt;
      const u = Math.min(1, r.age / RING_LIFE);
      const s = 1 + u * 2.4;
      r.mesh.scale.set(s, s, s);
      (r.mesh.material as THREE.MeshBasicMaterial).opacity = 0.42 * (1 - u) * (1 - u);
      if (u >= 1) r.mesh.visible = false;
    }
  }
}
