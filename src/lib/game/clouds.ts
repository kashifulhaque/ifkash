import * as THREE from 'three';
import { deriveSeed, seededRng } from './noise';
import { PLANET_RADIUS } from './planet';

type Cloud = { mesh: THREE.Mesh; axis: THREE.Vector3; base: THREE.Vector3; speed: number; angle: number };

/** Puffy clouds drifting around the planet, casting soft shadows. */
export class Clouds {
  group = new THREE.Group();
  private clouds: Cloud[] = [];

  constructor(seed: number, count = 14) {
    const rng = seededRng(deriveSeed(seed, 5));
    const mat = new THREE.MeshStandardMaterial({ color: 0xf6f8fa, roughness: 1, transparent: true, opacity: 0.92 });
    const puff = new THREE.SphereGeometry(1, 7, 5);
    for (let i = 0; i < count; i++) {
      const parts: THREE.BufferGeometry[] = [];
      const n = 3 + Math.floor(rng() * 3);
      for (let k = 0; k < n; k++) {
        const g = puff.clone();
        const s = 0.9 + rng() * 1.1;
        g.scale(s * 1.2, s * 0.7, s);
        g.translate((k - n / 2) * 1.3 + rng() * 0.6, rng() * 0.4, (rng() - 0.5) * 1.2);
        parts.push(g);
      }
      const merged = mergeSimple(parts);
      const mesh = new THREE.Mesh(merged, mat);
      mesh.castShadow = true;
      const base = new THREE.Vector3(rng() * 2 - 1, rng() * 2 - 1, rng() * 2 - 1).normalize();
      const axis = new THREE.Vector3(rng() * 2 - 1, rng() * 2 - 1, rng() * 2 - 1).normalize();
      this.clouds.push({ mesh, axis, base, speed: 0.01 + rng() * 0.02, angle: rng() * 6 });
      this.group.add(mesh);
    }
  }

  update(dt: number): void {
    const q = new THREE.Quaternion();
    const up = new THREE.Vector3();
    const m = new THREE.Matrix4();
    const right = new THREE.Vector3();
    const fwd = new THREE.Vector3();
    for (const c of this.clouds) {
      c.angle += c.speed * dt;
      q.setFromAxisAngle(c.axis, c.angle);
      up.copy(c.base).applyQuaternion(q);
      c.mesh.position.copy(up).multiplyScalar(PLANET_RADIUS + 9);
      right.crossVectors(c.axis, up).normalize();
      // right x up = fwd keeps the basis right-handed; a mirrored one makes
      // setFromRotationMatrix read a reflection as a rotation, and the cloud
      // snaps to a new orientation as it drifts.
      fwd.crossVectors(right, up);
      m.makeBasis(right, up, fwd);
      c.mesh.quaternion.setFromRotationMatrix(m);
    }
  }
}

function mergeSimple(parts: THREE.BufferGeometry[]): THREE.BufferGeometry {
  let count = 0;
  for (const p of parts) count += (p.index ? p.toNonIndexed() : p).attributes.position.count;
  const pos = new Float32Array(count * 3);
  let off = 0;
  for (const p of parts) {
    const g = p.index ? p.toNonIndexed() : p;
    pos.set(g.attributes.position.array as Float32Array, off);
    off += g.attributes.position.count * 3;
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
  geo.computeVertexNormals();
  return geo;
}
