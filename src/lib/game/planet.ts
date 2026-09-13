import * as THREE from 'three';
import { biomeAt, configureBiomeOcean, isOcean, oceanField, OCEAN, type Biome } from './biomes';
import { deriveSeed, fbm3, hash3, seededRng } from './noise';
import type { PlanetProfile } from './space';

export const PLANET_RADIUS = 42;
/** Radius of the translucent water sphere. */
export const SEA_LEVEL = PLANET_RADIUS + 0.4;

const EARTH_HUE = 0;
const EARTH_SATURATION = 1;
let planetHue = EARTH_HUE;
let planetSaturation = EARTH_SATURATION;
let terrainReliefScale = 1;
let terrainRuggedScale = 1;
let terrainDetailScale = 1;
let terrainCreaseFrequency = 3.2;
let planetWater = 0x256a8c;
const _hsl = { h: 0, s: 0, l: 0 };

/** Apply a profile to all subsequently generated terrain and water. */
export function configurePlanet(profile: PlanetProfile): void {
  planetHue = THREE.MathUtils.clamp(profile.hue, -0.5, 0.5);
  planetSaturation = THREE.MathUtils.clamp(profile.saturation, 0.25, 2);
  const relief = THREE.MathUtils.clamp(profile.relief, 0.4, 1.6);
  const ruggedness = THREE.MathUtils.clamp(profile.ruggedness, 0.4, 1.6);
  terrainReliefScale = relief;
  terrainRuggedScale = ruggedness;
  terrainDetailScale = THREE.MathUtils.lerp(1, ruggedness, 0.65);
  terrainCreaseFrequency = 3.2 * (1 + (ruggedness - 1) * 0.25);
  planetWater = profile.water;
  configureBiomeOcean(relief, ruggedness, profile.seed === 'earth');
}

/**
 * Distance from the planet centre to the ground along unit direction `d`.
 *
 * Land is three layers of noise on top of each other: a broad swell that makes
 * the difference between a valley and a plateau, a ridged term gated by a
 * "ruggedness" field so some regions crumple into hills while others stay open,
 * and a fine roll for texture. Every layer's amplitude is kept in proportion to
 * its wavelength, because the props stand on this ground and a slope steep
 * enough to leave them hanging in the air is worse than a flat world.
 */
export function surfaceRadius(d: THREE.Vector3): number {
  const f = oceanField(d);
  const reliefScale = terrainReliefScale;
  const ruggedScale = terrainRuggedScale;
  const detailScale = terrainDetailScale;
  if (f < 0) {
    // Seabed: falls away from the coast so the water reads as deeper offshore.
    const depth = Math.min(1, -f * 4);
    const relief = fbm3(d.x * 6 + 3, d.y * 6, d.z * 6 - 1, 3) * 0.45;
    return PLANET_RADIUS - 0.25 - depth * 1.6 + relief * 0.3 * reliefScale;
  }
  // A short cliff at the waterline, then the relief eases in over the shore so
  // the beaches stay walkable and the coastline keeps its shape.
  const rise = Math.min(1, f * 5);
  const swell = fbm3(d.x * 1.9 - 7, d.y * 1.9 + 4, d.z * 1.9 + 9, 3) * 2.2 * reliefScale;
  const roll = fbm3(d.x * 6 + 3, d.y * 6, d.z * 6 - 1, 3) * 0.55 * detailScale;
  const rugged = Math.max(0, fbm3(d.x * 1.4 + 21, d.y * 1.4 - 8, d.z * 1.4 + 3, 2));
  // A ridged fBm: folding the noise about zero turns its valleys into creases.
  const crease = 1 - Math.abs(
    fbm3(d.x * terrainCreaseFrequency - 12, d.y * terrainCreaseFrequency + 6, d.z * terrainCreaseFrequency - 2, 4)
  );
  const h = 0.5 + swell + roll + rugged * crease * crease * 2.8 * ruggedScale;
  // Dips are squashed and floored. Land is decided by `oceanField`, not by
  // height, so a valley deep enough to fall under `SEA_LEVEL` would leave dry
  // ground — and any prop or wonder standing on it — hidden under the water
  // sphere. The floor keeps every land point just clear of the waterline.
  const shaped = h >= 0 ? h : Math.max(-0.25, h * 0.15);
  return PLANET_RADIUS + 0.7 + rise * shaped;
}

const _na = new THREE.Vector3();
const _nb = new THREE.Vector3();
const _nr = new THREE.Vector3();
const _nf = new THREE.Vector3();

/**
 * Unit normal of the terrain at `d`, found by sampling the height a step east
 * and a step north. Props stand along this rather than along the radius, so a
 * tree on a hillside leans with the hill instead of hovering over it.
 */
export function surfaceNormal(d: THREE.Vector3, out = new THREE.Vector3()): THREE.Vector3 {
  tangentBasis(d, 0, _nr, _nf);
  const step = 0.6;
  const h = surfaceRadius(d);
  const east = (surfaceRadius(offsetDir(_na.copy(d), step, 0)) - h) / step;
  const north = (surfaceRadius(offsetDir(_nb.copy(d), 0, step)) - h) / step;
  return out.copy(d).addScaledVector(_nr, -east).addScaledVector(_nf, -north).normalize();
}

/** Radius a walker stands at: ground on land, the water surface at sea. */
export function walkRadius(d: THREE.Vector3): number {
  return Math.max(surfaceRadius(d), SEA_LEVEL - 0.15);
}

export function surfacePoint(d: THREE.Vector3, out = new THREE.Vector3()): THREE.Vector3 {
  return out.copy(d).multiplyScalar(surfaceRadius(d));
}

const _ref = new THREE.Vector3();
const _up = new THREE.Vector3();
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
  const up = surfaceNormal(d, _up);
  tangentBasis(up, yaw, _right, _fwd);
  out.makeBasis(_right, up, _fwd);
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

function applyPlanetPalette(color: THREE.Color): void {
  color.offsetHSL(planetHue, color.getHSL(_hsl).s * (planetSaturation - 1), 0);
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
    applyPlanetPalette(col);
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

/** Water depth, in world units, at which the sea reads as fully deep. */
const SHALLOW = 2.6;

/**
 * The sea. A plain sphere, but the shader on it does three things a flat
 * translucent ball cannot: it rolls a low swell through the surface, it darkens
 * with depth, and it breaks into foam where the seabed comes up to meet it.
 *
 * Depth is baked into a `aShore` attribute at build time rather than sampled in
 * the shader, because the seabed lives in noise the GPU has no access to. The
 * Game drives `uTime` through `mesh.userData.uniforms`.
 */
export function buildWater(): THREE.Mesh {
  const geo = new THREE.SphereGeometry(SEA_LEVEL, 96, 64);
  const pos = geo.attributes.position as THREE.BufferAttribute;
  const shore = new Float32Array(pos.count);
  const d = new THREE.Vector3();
  for (let i = 0; i < pos.count; i++) {
    d.fromBufferAttribute(pos, i).normalize();
    const depth = SEA_LEVEL - surfaceRadius(d);
    shore[i] = 1 - THREE.MathUtils.clamp(depth / SHALLOW, 0, 1);
  }
  geo.setAttribute('aShore', new THREE.BufferAttribute(shore, 1));

  const surface = new THREE.Color(planetWater);
  const deep = surface.clone().offsetHSL(-0.02, -0.05, -0.04);
  const foam = surface.clone().offsetHSL(-0.01, -0.25, 0.58);
  const uniforms = {
    uTime: { value: 0 },
    uDeep: { value: deep },
    uFoam: { value: foam }
  };
  const mat = new THREE.MeshStandardMaterial({
    color: surface,
    transparent: true,
    opacity: 0.86,
    roughness: 0.22,
    metalness: 0.05
  });
  mat.onBeforeCompile = (shader) => {
    Object.assign(shader.uniforms, uniforms);
    shader.vertexShader =
      `uniform float uTime;
attribute float aShore;
varying float vShore;
varying vec3 vWave;
` +
      shader.vertexShader.replace(
        '#include <begin_vertex>',
        `#include <begin_vertex>
  vShore = aShore;
  vec3 sphereN = normalize(transformed);
  // Two crossed swells, damped to nothing in the shallows so the surface does
  // not saw through the beaches.
  float open = 1.0 - vShore;
  float w1 = sin(transformed.x * 0.7 + transformed.z * 0.4 + uTime * 1.1);
  float w2 = sin(transformed.z * 0.9 - transformed.y * 0.5 + uTime * 1.7);
  transformed += sphereN * (w1 * 0.06 + w2 * 0.04) * open;
  // The slope of those swells, handed to the fragment stage as a fake normal.
  vWave = vec3(cos(transformed.x * 0.7 + uTime * 1.1), 0.0, cos(transformed.z * 0.9 + uTime * 1.7)) * 0.12 * open;`
      );
    shader.fragmentShader =
      `uniform float uTime;
uniform vec3 uDeep;
uniform vec3 uFoam;
varying float vShore;
varying vec3 vWave;
` +
      shader.fragmentShader
        .replace(
          '#include <color_fragment>',
          `#include <color_fragment>
  // Open water is darker and colder than the shallows.
  diffuseColor.rgb = mix(uDeep, diffuseColor.rgb, 0.45 + 0.55 * vShore);
  // Foam: a band that follows the waterline, banded so it reads as surf rather
  // than a flat rim, and pulsed so it breathes with the swell.
  float band = sin(vShore * 34.0 - uTime * 2.2) * 0.5 + 0.5;
  float surf = smoothstep(0.62, 0.99, vShore) * (0.35 + 0.65 * band);
  diffuseColor.rgb = mix(diffuseColor.rgb, uFoam, surf * 0.85);
  diffuseColor.a = mix(diffuseColor.a, 1.0, surf);`
        )
        .replace(
          '#include <normal_fragment_maps>',
          `#include <normal_fragment_maps>
  normal = normalize(normal + vWave);`
        );
  };
  mat.customProgramCacheKey = () => 'water-fx';
  const mesh = new THREE.Mesh(geo, mat);
  mesh.receiveShadow = true;
  mesh.name = 'water';
  mesh.userData.uniforms = uniforms;
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
