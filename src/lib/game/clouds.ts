import * as THREE from 'three';
import { deriveSeed, seededRng } from './noise';
import { PLANET_RADIUS } from './planet';
import type { PlanetProfile } from './space';

type Cloud = { mesh: THREE.Mesh; axis: THREE.Vector3; base: THREE.Vector3; speed: number; angle: number };

/** Puffy clouds drifting around the planet, casting soft shadows. */
export class Clouds {
  group = new THREE.Group();
  private clouds: Cloud[] = [];

  constructor(seed: number, profile: PlanetProfile) {
    const rng = seededRng(deriveSeed(seed, 5));
    const isEarth = profile.archetype === 'earth';
    const mat = new THREE.MeshStandardMaterial({
      color: profile.cloud,
      roughness: 1,
      transparent: true,
      opacity: isEarth ? 0.92 : profile.archetype === 'ember' ? 0.58 : 0.78
    });
    const puff = new THREE.SphereGeometry(1, 7, 5);
    const count = profile.cloudCount;
    for (let i = 0; i < count; i++) {
      const parts: THREE.BufferGeometry[] = [];
      const n = 3 + Math.floor(rng() * 3);
      for (let k = 0; k < n; k++) {
        const g = puff.clone();
        const s = 0.9 + rng() * 1.1;
        const broad = profile.archetype === 'reef' ? 1.7 : profile.archetype === 'ember' ? 1.45 : 1.2;
        const thin = profile.archetype === 'rime' ? 0.28 : profile.archetype === 'ember' ? 0.42 : 0.7;
        g.scale(s * broad, s * thin, s);
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

    if (profile.archetype === 'fracture' || profile.archetype === 'rime') {
      const debrisCount = profile.archetype === 'fracture' ? 30 : 18;
      const debris = new THREE.InstancedMesh(
        new THREE.DodecahedronGeometry(0.5, 0),
        new THREE.MeshStandardMaterial({ color: profile.terrain[1], roughness: 0.92, flatShading: true }),
        debrisCount
      );
      const matrix = new THREE.Matrix4();
      const position = new THREE.Vector3();
      const rotation = new THREE.Quaternion();
      const scale = new THREE.Vector3();
      for (let i = 0; i < debrisCount; i++) {
        const a = (i / debrisCount) * Math.PI * 2 + rng() * 0.12;
        const radius = 53 + rng() * 7;
        position.set(Math.cos(a) * radius, (rng() - 0.5) * 2.2, Math.sin(a) * radius);
        rotation.setFromEuler(new THREE.Euler(rng() * 3, rng() * 3, rng() * 3));
        const s = 0.45 + rng() * (profile.archetype === 'fracture' ? 1.6 : 0.8);
        scale.set(s, s * (0.55 + rng()), s);
        debris.setMatrixAt(i, matrix.compose(position, rotation, scale));
      }
      debris.rotation.set(0.38, 0.16, -0.24);
      debris.name = 'orbital-debris';
      this.group.add(debris);
    }

    if (profile.archetype === 'crystal') {
      const ringMaterial = new THREE.MeshBasicMaterial({
        color: profile.accent,
        transparent: true,
        opacity: 0.5,
        side: THREE.DoubleSide
      });
      for (let i = 0; i < 2; i++) {
        const ring = new THREE.Mesh(new THREE.TorusGeometry(55 + i * 2.4, 0.11 + i * 0.05, 5, 160), ringMaterial);
        ring.rotation.set(Math.PI / 2 + i * 0.08, 0.25, 0.38 + i * 0.12);
        ring.name = 'crystal-ring';
        this.group.add(ring);
      }
    }

    if (profile.archetype === 'bloom' || profile.archetype === 'spore') {
      const positions = new Float32Array(180 * 3);
      const point = new THREE.Vector3();
      for (let i = 0; i < 180; i++) {
        point.set(rng() * 2 - 1, rng() * 2 - 1, rng() * 2 - 1).normalize().multiplyScalar(47 + rng() * 7);
        positions.set([point.x, point.y, point.z], i * 3);
      }
      const spores = new THREE.BufferGeometry();
      spores.setAttribute('position', new THREE.BufferAttribute(positions, 3));
      const motes = new THREE.Points(
        spores,
        new THREE.PointsMaterial({ color: profile.accent, size: 0.16, transparent: true, opacity: 0.72, sizeAttenuation: true })
      );
      motes.name = 'atmospheric-spores';
      this.group.add(motes);
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
