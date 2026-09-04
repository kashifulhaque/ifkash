import * as THREE from 'three';
import { biomeAt, isOcean, oceanField, OCEAN, type Biome } from './biomes';
import { deriveSeed, fbm3, hash3, seededRng } from './noise';

export const PLANET_RADIUS = 42;
/** Radius of the translucent water sphere. */
export const SEA_LEVEL = PLANET_RADIUS + 0.4;

/** Distance from the planet centre to the ground along unit direction `d`. */
export function surfaceRadius(d: THREE.Vector3): number {
  const f = oceanField(d);
  const relief = fbm3(d.x * 6 + 3, d.y * 6, d.z * 6 - 1, 3) * 0.45;
  if (f < 0) {
    // Seabed: falls away from the coast so the water reads as deeper offshore.
    const depth = Math.min(1, -f * 4);
    return PLANET_RADIUS - 0.25 - depth * 1.6 + relief * 0.3;
  }
  // Land: a short cliff at the waterline, then gentle rolling ground.
  const rise = Math.min(1, f * 5);
  return PLANET_RADIUS + 0.7 + rise * 0.5 + relief * rise;
}

/** Radius a walker stands at: ground on land, the water surface at sea. */
export function walkRadius(d: THREE.Vector3): number {
  return Math.max(surfaceRadius(d), SEA_LEVEL - 0.15);
}

export function surfacePoint(d: THREE.Vector3, out = new THREE.Vector3()): THREE.Vector3 {
  return out.copy(d).multiplyScalar(surfaceRadius(d));
}

const _ref = new THREE.Vector3();
const _right = new THREE.Vector3();
const _fwd = new THREE.Vector3();
const _q = new THREE.Quaternion();

/**
 * Tangent basis at `up`: `right` points roughly east and `fwd` roughly north,
 * rotated about `up` by `yaw`.
 *
 * The three axes are ordered so that `right x up = fwd`, which makes
 * `makeBasis(right, up, fwd)` a right-handed rotation. Getting this backwards
 * mirrors every frame built on it: triangle winding reverses, so baked geometry
 * turns inside out and `setFromRotationMatrix` reads a reflection as a rotation.
 */
export function tangentBasis(up: THREE.Vector3, yaw: number, right: THREE.Vector3, fwd: THREE.Vector3): void {
  _ref.set(0, 1, 0);
  if (Math.abs(up.y) > 0.98) _ref.set(1, 0, 0);
  right.crossVectors(up, _ref).normalize();
  fwd.crossVectors(right, up).normalize();
  if (yaw !== 0) {
    _q.setFromAxisAngle(up, yaw);
    right.applyQuaternion(_q);
    fwd.applyQuaternion(_q);
  }
}

/**
 * Placement transform for a prop standing on the ground at direction `d`.
 * Local +Y is the surface normal and local +Z faces `yaw` radians from north.
 */
export function surfaceFrame(d: THREE.Vector3, yaw: number, out: THREE.Matrix4, lift = 0): THREE.Matrix4 {
  tangentBasis(d, yaw, _right, _fwd);
  out.makeBasis(_right, d, _fwd);
  const r = surfaceRadius(d) + lift;
  out.setPosition(d.x * r, d.y * r, d.z * r);
  return out;
}

/** Move a unit direction `east`/`north` units across the surface, in place. */
export function offsetDir(d: THREE.Vector3, east: number, north: number): THREE.Vector3 {
  tangentBasis(d, 0, _right, _fwd);
  const step = _right.multiplyScalar(east).addScaledVector(_fwd, north);
  const len = step.length();
  if (len < 1e-6) return d;
  const ang = len / PLANET_RADIUS;
  step.divideScalar(len);
  d.multiplyScalar(Math.cos(ang)).addScaledVector(step, Math.sin(ang)).normalize();
  return d;
}

function pickShade(b: Biome, h: number): number {
  return h < 0.62 ? b.ground[0] : h < 0.84 ? b.ground[1] : b.ground[2];
}

export function buildGround(): THREE.Mesh {
  const geo = new THREE.IcosahedronGeometry(1, 44);
  const pos = geo.attributes.position as THREE.BufferAttribute;
  const colors = new Float32Array(pos.count * 3);
  const a = new THREE.Vector3();
  const b = new THREE.Vector3();
  const c = new THREE.Vector3();
  const mid = new THREE.Vector3();
  const col = new THREE.Color();

  for (let i = 0; i < pos.count; i += 3) {
    a.fromBufferAttribute(pos, i).normalize();
    b.fromBufferAttribute(pos, i + 1).normalize();
    c.fromBufferAttribute(pos, i + 2).normalize();
    mid.copy(a).add(b).add(c).normalize();

    const biome = biomeAt(mid);
    const f = oceanField(mid);
    const h = hash3(Math.round(mid.x * 997), Math.round(mid.y * 991), Math.round(mid.z * 983));
    let hex: number;
    if (biome === OCEAN) {
      hex = pickShade(OCEAN, h);
    } else if (f < 0.09) {
      hex = biome.beach;
    } else {
      hex = pickShade(biome, h);
    }
    col.setHex(hex);
    // Faint per-face variation keeps the flat shading from looking like a texture.
    const v = 0.96 + hash3(i, 7, 13) * 0.08;
    col.multiplyScalar(v);

    for (let k = 0; k < 3; k++) {
      const p = k === 0 ? a : k === 1 ? b : c;
      const r = surfaceRadius(p);
      pos.setXYZ(i + k, p.x * r, p.y * r, p.z * r);
      colors[(i + k) * 3] = col.r;
      colors[(i + k) * 3 + 1] = col.g;
      colors[(i + k) * 3 + 2] = col.b;
    }
  }
  geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
  geo.deleteAttribute('uv');
  geo.computeVertexNormals();

  const mat = new THREE.MeshStandardMaterial({
    vertexColors: true,
    flatShading: true,
    roughness: 0.95,
    metalness: 0
  });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.receiveShadow = true;
  mesh.name = 'ground';
  return mesh;
}

export function buildWater(): THREE.Mesh {
  const geo = new THREE.SphereGeometry(SEA_LEVEL, 72, 48);
  const mat = new THREE.MeshStandardMaterial({
    color: 0x3fb3d3,
    transparent: true,
    opacity: 0.8,
    roughness: 0.3,
    metalness: 0.05
  });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.receiveShadow = true;
  mesh.name = 'water';
  return mesh;
}

export function buildStars(seed: number): THREE.Points {
  const rng = seededRng(deriveSeed(seed, 4));
  const n = 1400;
  const arr = new Float32Array(n * 3);
  const d = new THREE.Vector3();
  for (let i = 0; i < n; i++) {
    d.set(rng() * 2 - 1, rng() * 2 - 1, rng() * 2 - 1).normalize();
    const r = 520 + rng() * 60;
    arr[i * 3] = d.x * r;
    arr[i * 3 + 1] = d.y * r;
    arr[i * 3 + 2] = d.z * r;
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(arr, 3));
  const mat = new THREE.PointsMaterial({
    color: 0xcfe3ef,
    size: 1.6,
    sizeAttenuation: false,
    transparent: true,
    opacity: 0.7
  });
  return new THREE.Points(geo, mat);
}

export { isOcean };
