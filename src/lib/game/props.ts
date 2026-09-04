// Low-poly scenery. Everything static is baked into three merged meshes (one
// lit, one unlit "glow" for lava, embers, and crystals, and one "night" batch of
// lit windows and doorways) so the whole planet's props cost a handful of draw
// calls. Animated pieces (windmill blades, flames) are built separately and
// placed by the Game at the anchor frames returned here.
//
// Anything that glows also registers an `Emitter` through `emit(...)`, the way
// a solid prop registers a footprint, so the light pool in `lights.ts` can cast
// real light from it when the player is near.

import * as THREE from 'three';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import { BIOMES, biomeById, landBiomeAt, isOcean, oceanField, type BiomeId } from './biomes';
import { deriveSeed, seededRng } from './noise';
import { PLANET_RADIUS, offsetDir, surfaceFrame } from './planet';
import type { Collider } from './collision';
import { emitter, type Emitter, type EmitterOptions } from './lights';
import { WONDERS, wonderDir } from './wonders';

// ---------------------------------------------------------------- batching

export class PropBatch {
  private parts: THREE.BufferGeometry[] = [];
  private color = new THREE.Color();

  add(geo: THREE.BufferGeometry, hex: number, matrix: THREE.Matrix4): void {
    const g = geo.index ? geo.toNonIndexed() : geo.clone();
    g.applyMatrix4(matrix);
    // A mirrored placement — a negative scale, or a left-handed frame — reverses
    // triangle winding. Because the batch bakes the matrix into the vertices,
    // the renderer cannot flip the cull face for us the way it does for a mesh
    // with a mirrored world matrix, so the prop would render inside out: front
    // faces culled, normals pointing in, and shadows cast off its far side.
    if (matrix.determinant() < 0) reverseWinding(g);
    for (const name of Object.keys(g.attributes)) if (name !== 'position') g.deleteAttribute(name);
    const n = g.attributes.position.count;
    const arr = new Float32Array(n * 3);
    this.color.setHex(hex);
    for (let i = 0; i < n; i++) {
      arr[i * 3] = this.color.r;
      arr[i * 3 + 1] = this.color.g;
      arr[i * 3 + 2] = this.color.b;
    }
    g.setAttribute('color', new THREE.BufferAttribute(arr, 3));
    this.parts.push(g);
  }

  build(material: THREE.Material): THREE.Mesh | null {
    if (this.parts.length === 0) return null;
    const merged = mergeGeometries(this.parts, false);
    if (!merged) return null;
    merged.computeVertexNormals();
    for (const p of this.parts) p.dispose();
    this.parts = [];
    return new THREE.Mesh(merged, material);
  }
}

/** Swap two corners of every triangle, flipping the geometry's facing. */
function reverseWinding(g: THREE.BufferGeometry): void {
  for (const attribute of Object.values(g.attributes)) {
    const array = attribute.array as Float32Array;
    const size = attribute.itemSize;
    for (let i = 0; i < attribute.count; i += 3) {
      for (let k = 0; k < size; k++) {
        const a = (i + 1) * size + k;
        const b = (i + 2) * size + k;
        const t = array[a];
        array[a] = array[b];
        array[b] = t;
      }
    }
    attribute.needsUpdate = true;
  }
}

// Shared unit geometries, all sitting on y = 0 so `s[1]` is the height and
// `p[1]` is the base.
const G = {
  box: new THREE.BoxGeometry(1, 1, 1).translate(0, 0.5, 0),
  cone4: new THREE.ConeGeometry(0.5, 1, 4).translate(0, 0.5, 0),
  cone6: new THREE.ConeGeometry(0.5, 1, 6).translate(0, 0.5, 0),
  cone8: new THREE.ConeGeometry(0.5, 1, 8).translate(0, 0.5, 0),
  cyl6: new THREE.CylinderGeometry(0.5, 0.5, 1, 6).translate(0, 0.5, 0),
  cyl8: new THREE.CylinderGeometry(0.5, 0.5, 1, 8).translate(0, 0.5, 0),
  taper8: new THREE.CylinderGeometry(0.36, 0.5, 1, 8).translate(0, 0.5, 0),
  ico: new THREE.IcosahedronGeometry(0.5, 0).translate(0, 0.5, 0),
  dodeca: new THREE.DodecahedronGeometry(0.5, 0).translate(0, 0.5, 0),
  sphere: new THREE.SphereGeometry(0.5, 8, 6).translate(0, 0.5, 0),
  tetra: new THREE.TetrahedronGeometry(0.5).translate(0, 0.3, 0)
};

type PartOpts = { p?: [number, number, number]; s?: [number, number, number] | number; r?: [number, number, number] };

const _m = new THREE.Matrix4();
const _p = new THREE.Vector3();
const _q = new THREE.Quaternion();
const _s = new THREE.Vector3();
const _e = new THREE.Euler();

function part(b: PropBatch, geo: THREE.BufferGeometry, color: number, frame: THREE.Matrix4, o: PartOpts = {}): void {
  const p = o.p ?? [0, 0, 0];
  const r = o.r ?? [0, 0, 0];
  const s = o.s ?? 1;
  _p.set(p[0], p[1], p[2]);
  _e.set(r[0], r[1], r[2]);
  _q.setFromEuler(_e);
  if (typeof s === 'number') _s.set(s, s, s);
  else _s.set(s[0], s[1], s[2]);
  _m.compose(_p, _q, _s).premultiply(frame);
  b.add(geo, color, _m);
}

// ---------------------------------------------------------------- palette

const TRUNK = 0x7a5236;
const PINE = [0x3b8f5e, 0x46a06a, 0x338254];
const LEAF = [0x5cb04a, 0x6dc257, 0x4f9f40];
const ROCK = 0x8d939a;
const ROCK_D = 0x6f757d;
const SNOW = 0xf7fafb;
const WOOD = 0x9a6b45;
const WOOD_D = 0x6e4a2f;
const RED = 0xc9392e;
const CREAM = 0xf1ebdc;
const STONE = 0xe7d7a8;
const LAVA = 0xff7a1a;
const LAVA_B = 0xffb340;
const ICE = 0x8fd9ea;
const ICE_B = 0xbdeef7;
const SOIL = 0x6b4a32;
const STRAW = 0xe0c060;
const PALM = 0x3aa35a;
const CACTUS = 0x4f9a5c;

type Ctx = { solid: PropBatch; glow: PropBatch; night: PropBatch; rng: () => number; colliders: Collider[]; lights: Emitter[] };

const _cp = new THREE.Vector3();

/**
 * Register a solid footprint of `radius` surface units at the frame's origin,
 * or at a local `[x, z]` offset from it. Props that are low or thin, such as
 * grass, flowers, crops, and path stones, register nothing.
 */
function footprint(c: Ctx, frame: THREE.Matrix4, radius: number, offset?: [number, number]): void {
  if (offset) _cp.set(offset[0], 0, offset[1]).applyMatrix4(frame);
  else _cp.setFromMatrixPosition(frame);
  c.colliders.push({ dir: _cp.clone().normalize(), radius });
}

/**
 * Register a light source at a local `[x, y, z]` offset in the prop's frame.
 * `intensity` is the peak; the pool fades it with distance and, for a flame,
 * flickers it. `distance` is the light's reach in world units.
 */
function emit(
  c: Ctx,
  frame: THREE.Matrix4,
  color: number,
  intensity: number,
  distance: number,
  offset: [number, number, number],
  o: EmitterOptions = {}
): void {
  _cp.set(offset[0], offset[1], offset[2]).applyMatrix4(frame);
  // The flicker phase comes from the position, not `c.rng()`: drawing here would
  // shift every later draw and reshuffle the scatter for a given seed.
  const phase = Math.abs(_cp.x * 12.9898 + _cp.y * 78.233 + _cp.z * 37.719) % 6.283;
  c.lights.push(emitter(_cp.clone(), color, intensity, distance, { phase, ...o }));
}

const rnd = (c: Ctx, a: number, b: number) => a + c.rng() * (b - a);
const pick = <T>(c: Ctx, arr: T[]): T => arr[Math.floor(c.rng() * arr.length)];

// ---------------------------------------------------------------- builders

function pine(c: Ctx, f: THREE.Matrix4, scale = 1): void {
  const s = scale * rnd(c, 0.85, 1.25);
  const col = pick(c, PINE);
  footprint(c, f, 0.55 * s);
  part(c.solid, G.cyl6, TRUNK, f, { s: [0.28 * s, 0.7 * s, 0.28 * s] });
  part(c.solid, G.cone6, col, f, { p: [0, 0.45 * s, 0], s: [1.7 * s, 1.5 * s, 1.7 * s] });
  part(c.solid, G.cone6, col, f, { p: [0, 1.25 * s, 0], s: [1.3 * s, 1.3 * s, 1.3 * s], r: [0, 0.4, 0] });
  part(c.solid, G.cone6, col, f, { p: [0, 2.0 * s, 0], s: [0.85 * s, 1.1 * s, 0.85 * s] });
}

function snowPine(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.7, 1.0);
  footprint(c, f, 0.5 * s);
  part(c.solid, G.cyl6, TRUNK, f, { s: [0.24 * s, 0.6 * s, 0.24 * s] });
  part(c.solid, G.cone6, 0x3b7f63, f, { p: [0, 0.4 * s, 0], s: [1.5 * s, 1.3 * s, 1.5 * s] });
  part(c.solid, G.cone6, SNOW, f, { p: [0, 1.1 * s, 0], s: [1.1 * s, 1.1 * s, 1.1 * s], r: [0, 0.5, 0] });
  part(c.solid, G.cone6, SNOW, f, { p: [0, 1.75 * s, 0], s: [0.7 * s, 0.9 * s, 0.7 * s] });
}

function roundTree(c: Ctx, f: THREE.Matrix4, scale = 1): void {
  const s = scale * rnd(c, 0.85, 1.2);
  const col = pick(c, LEAF);
  footprint(c, f, 0.45 * s);
  part(c.solid, G.cyl6, TRUNK, f, { s: [0.3 * s, 1.0 * s, 0.3 * s] });
  part(c.solid, G.ico, col, f, { p: [0, 0.8 * s, 0], s: [1.7 * s, 1.5 * s, 1.7 * s], r: [0, c.rng() * 3, 0] });
  part(c.solid, G.ico, col, f, { p: [0.4 * s, 1.3 * s, 0.2 * s], s: [1.0 * s, 0.9 * s, 1.0 * s], r: [0.3, c.rng(), 0] });
}

function bush(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.5, 0.9);
  footprint(c, f, 0.5 * s);
  part(c.solid, G.ico, pick(c, LEAF), f, { s: [s * 1.2, s * 0.8, s * 1.2], r: [0, c.rng() * 3, 0] });
}

function rock(c: Ctx, f: THREE.Matrix4, scale = 1, color = ROCK): void {
  const s = scale * rnd(c, 0.4, 1.0);
  footprint(c, f, 0.55 * s);
  part(c.solid, G.dodeca, c.rng() < 0.5 ? color : ROCK_D, f, {
    s: [s * rnd(c, 0.9, 1.4), s * rnd(c, 0.6, 1.0), s * rnd(c, 0.9, 1.3)],
    r: [0, c.rng() * 3, 0]
  });
}

function mountain(c: Ctx, f: THREE.Matrix4, radius: number, height: number): void {
  footprint(c, f, radius * 0.9);
  footprint(c, f, radius * 0.6, [radius * 0.6, radius * 0.3]);
  part(c.solid, G.cone6, 0x8a939e, f, { s: [radius * 2, height, radius * 2], r: [0, c.rng() * 1, 0] });
  part(c.solid, G.cone6, 0x7a838e, f, {
    p: [radius * 0.6, 0, radius * 0.3],
    s: [radius * 1.3, height * 0.62, radius * 1.3],
    r: [0, c.rng() * 1, 0]
  });
  // Snow cap: a smaller cone sitting on the top third.
  const capBase = height * 0.68;
  part(c.solid, G.cone6, SNOW, f, {
    p: [0, capBase, 0],
    s: [radius * 2 * (1 - capBase / height) * 1.04, height - capBase + 0.05, radius * 2 * (1 - capBase / height) * 1.04]
  });
}

function grass(c: Ctx, f: THREE.Matrix4, color = 0x6fc45a): void {
  for (let i = 0; i < 3; i++) {
    part(c.solid, G.cone4, color, f, {
      p: [rnd(c, -0.2, 0.2), 0, rnd(c, -0.2, 0.2)],
      s: [0.12, rnd(c, 0.35, 0.65), 0.12],
      r: [rnd(c, -0.3, 0.3), c.rng() * 3, rnd(c, -0.3, 0.3)]
    });
  }
}

function flower(c: Ctx, f: THREE.Matrix4): void {
  const col = pick(c, [0xff8fb1, 0xffe066, 0xfff4e6, 0xc59bff]);
  part(c.solid, G.cyl6, 0x4f9f40, f, { s: [0.06, 0.35, 0.06] });
  part(c.solid, G.ico, col, f, { p: [0, 0.3, 0], s: 0.22 });
}

function sunflower(c: Ctx, f: THREE.Matrix4): void {
  part(c.solid, G.cyl6, 0x4f9f40, f, { s: [0.08, 1.1, 0.08] });
  part(c.solid, G.cyl8, 0xffd23f, f, { p: [0, 1.05, 0.05], s: [0.5, 0.08, 0.5], r: [Math.PI / 2, 0, 0] });
  part(c.solid, G.cyl8, 0x5a3b1e, f, { p: [0, 1.05, 0.1], s: [0.26, 0.06, 0.26], r: [Math.PI / 2, 0, 0] });
}

function cabin(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 1.3);
  part(c.solid, G.box, WOOD, f, { s: [2.0, 1.2, 1.7] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 1.2, 0], s: [2.2, 0.12, 1.9] });
  part(c.solid, G.cone4, 0x3e5f4a, f, { p: [0, 1.28, 0], s: [2.9, 1.1, 2.6], r: [0, Math.PI / 4, 0] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 0, 0.86], s: [0.5, 0.85, 0.06] });
  part(c.solid, G.box, 0xfff0b8, f, { p: [0.62, 0.55, 0.86], s: [0.4, 0.4, 0.05] });
  part(c.solid, G.box, 0xfff0b8, f, { p: [-0.62, 0.55, 0.86], s: [0.4, 0.4, 0.05] });
  // Lamplight in the windows, faded in by the Game after dark.
  part(c.night, G.box, 0xffd27a, f, { p: [0.62, 0.55, 0.87], s: [0.42, 0.42, 0.06] });
  part(c.night, G.box, 0xffd27a, f, { p: [-0.62, 0.55, 0.87], s: [0.42, 0.42, 0.06] });
  // Lamplight spilling out of the two windows, pooled into one source.
  emit(c, f, 0xffc978, 22, 10, [0, 0.7, 1.15], { flicker: 0.2 });
  part(c.solid, G.box, ROCK_D, f, { p: [0.65, 1.2, -0.4], s: [0.3, 0.9, 0.3] });
}

function tent(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.9);
  part(c.solid, G.cone4, 0xe07b39, f, { s: [1.6, 1.15, 1.8], r: [0, Math.PI / 4, 0] });
  part(c.solid, G.box, 0x2a2a30, f, { p: [0, 0.05, 0.62], s: [0.45, 0.6, 0.1] });
}

function logPile(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.7, [0, -0.6]);
  for (const [x, y] of [
    [-0.3, 0],
    [0.3, 0],
    [0, 0.5]
  ]) {
    part(c.solid, G.cyl8, WOOD_D, f, { p: [x, y, -0.6], s: [0.5, 1.2, 0.5], r: [Math.PI / 2, 0, 0] });
  }
}

/** Campfire ring; the flame is animated and added by the Game. */
function campfire(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.8);
  for (let i = 0; i < 7; i++) {
    const a = (i / 7) * Math.PI * 2;
    part(c.solid, G.dodeca, ROCK_D, f, { p: [Math.cos(a) * 0.62, 0, Math.sin(a) * 0.62], s: [0.32, 0.26, 0.32], r: [0, a, 0] });
  }
  part(c.solid, G.cyl6, WOOD_D, f, { p: [0, 0.08, 0], s: [0.18, 0.9, 0.18], r: [Math.PI / 2, 0, 0.5] });
  part(c.solid, G.cyl6, WOOD_D, f, { p: [0, 0.08, 0], s: [0.18, 0.9, 0.18], r: [Math.PI / 2, 0, -0.5] });
  part(c.glow, G.dodeca, LAVA, f, { p: [0, 0.05, 0], s: [0.5, 0.18, 0.5] });
  emit(c, f, 0xff9040, 52, 12, [0, 1.0, 0], { flicker: 1 });
}

function barn(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 1.3, [-0.6, 0]);
  footprint(c, f, 1.3, [0.6, 0]);
  part(c.solid, G.box, RED, f, { s: [2.8, 1.7, 2.2] });
  part(c.solid, G.box, 0xa22e25, f, { p: [0, 1.7, 0], s: [3.0, 0.9, 2.4] });
  part(c.solid, G.cone4, 0x8b261f, f, { p: [0, 2.55, 0], s: [3.2, 0.8, 2.6], r: [0, Math.PI / 4, 0] });
  part(c.solid, G.box, CREAM, f, { p: [0, 0, 1.11], s: [0.9, 1.2, 0.06] });
  part(c.solid, G.box, RED, f, { p: [0, 0.05, 1.14], s: [0.06, 1.1, 0.04], r: [0, 0, 0.6] });
  part(c.solid, G.box, CREAM, f, { p: [0, 1.95, 1.21], s: [0.5, 0.5, 0.05] });
  part(c.solid, G.box, CREAM, f, { p: [0, 0, 0], s: [2.9, 0.14, 2.3] });
}

function silo(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.6);
  part(c.solid, G.cyl8, 0xc9d3d8, f, { s: [1.1, 2.6, 1.1] });
  part(c.solid, G.sphere, 0xb0bcc3, f, { p: [0, 2.5, 0], s: [1.15, 0.9, 1.15] });
  part(c.solid, G.box, 0xa3aeb5, f, { p: [0, 1.2, 0], s: [1.16, 0.08, 1.16] });
}

/** Windmill tower; returns the hub frame for the animated blades. */
function windmill(c: Ctx, f: THREE.Matrix4): THREE.Matrix4 {
  footprint(c, f, 0.85);
  part(c.solid, G.taper8, CREAM, f, { s: [1.6, 3.0, 1.6] });
  part(c.solid, G.cone8, 0x8f5a3c, f, { p: [0, 3.0, 0], s: [1.5, 0.8, 1.5] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 0, 0.62], s: [0.5, 0.9, 0.1] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 2.7, 0.55], s: [0.3, 0.3, 0.55] });
  const hub = new THREE.Matrix4().makeTranslation(0, 2.85, 0.85).premultiply(f);
  return hub;
}

function fence(c: Ctx, f: THREE.Matrix4, length: number): void {
  const n = Math.max(1, Math.round(length / 0.9));
  for (let i = 0; i <= n; i++) {
    part(c.solid, G.box, CREAM, f, { p: [-length / 2 + (i / n) * length, 0, 0], s: [0.12, 0.7, 0.12] });
  }
  // A chain of small discs along the rails keeps the fence solid end to end.
  const links = Math.max(1, Math.round(length / 0.5));
  for (let i = 0; i <= links; i++) footprint(c, f, 0.3, [-length / 2 + (i / links) * length, 0]);
  part(c.solid, G.box, CREAM, f, { p: [0, 0.5, 0], s: [length, 0.07, 0.06] });
  part(c.solid, G.box, CREAM, f, { p: [0, 0.25, 0], s: [length, 0.07, 0.06] });
}

function cropRow(c: Ctx, f: THREE.Matrix4): void {
  part(c.solid, G.box, SOIL, f, { s: [0.9, 0.12, 3.4] });
  for (let i = 0; i < 6; i++) {
    part(c.solid, G.ico, pick(c, [0x5cae4b, 0x7cc86b, 0xff9f43]), f, { p: [0, 0.1, -1.4 + i * 0.56], s: 0.28 });
  }
}

function hayBale(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.5);
  part(c.solid, G.cyl8, STRAW, f, { p: [0, 0.4, 0], s: [0.8, 0.9, 0.8], r: [Math.PI / 2, 0, c.rng() * 3] });
}

function scarecrow(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.25);
  part(c.solid, G.box, WOOD_D, f, { s: [0.1, 1.6, 0.1] });
  part(c.solid, G.box, 0x6b8cc9, f, { p: [0, 0.8, 0], s: [1.1, 0.5, 0.3] });
  part(c.solid, G.sphere, STRAW, f, { p: [0, 1.3, 0], s: 0.4 });
  part(c.solid, G.cone6, 0xc9a24c, f, { p: [0, 1.62, 0], s: [0.8, 0.3, 0.8] });
}

function mailbox(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.35);
  part(c.solid, G.box, WOOD_D, f, { s: [0.12, 1.0, 0.12] });
  part(c.solid, G.box, 0x3d6fc9, f, { p: [0, 1.0, 0], s: [0.42, 0.34, 0.6] });
  part(c.solid, G.cyl8, 0x3d6fc9, f, { p: [0, 1.34, 0], s: [0.42, 0.6, 0.42], r: [Math.PI / 2, 0, 0] });
  part(c.glow, G.box, 0xff5d5d, f, { p: [0.22, 1.2, 0.1], s: [0.05, 0.3, 0.08] });
}

function signpost(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.25);
  part(c.solid, G.box, WOOD_D, f, { s: [0.14, 1.9, 0.14] });
  part(c.solid, G.box, WOOD, f, { p: [0.2, 1.5, 0], s: [1.1, 0.3, 0.08], r: [0, 0.2, 0.08] });
  part(c.solid, G.box, WOOD, f, { p: [-0.15, 1.1, 0], s: [0.9, 0.28, 0.08], r: [0, -0.35, -0.05] });
  part(c.solid, G.box, 0x2a2a30, f, { p: [0.2, 1.6, 0.05], s: [0.6, 0.05, 0.02] });
  part(c.solid, G.box, 0x2a2a30, f, { p: [-0.15, 1.2, 0.05], s: [0.5, 0.05, 0.02] });
}

function palm(c: Ctx, f: THREE.Matrix4): void {
  const h = rnd(c, 2.0, 2.8);
  const lean = rnd(c, -0.2, 0.2);
  footprint(c, f, 0.3);
  part(c.solid, G.taper8, 0x8b6b48, f, { s: [0.32, h, 0.32], r: [lean, 0, lean * 0.5] });
  const top: [number, number, number] = [Math.sin(lean * 0.5) * h * 0.4, h * 0.97, -Math.sin(lean) * h * 0.45];
  for (let i = 0; i < 6; i++) {
    const a = (i / 6) * Math.PI * 2 + c.rng() * 0.4;
    part(c.solid, G.cone4, PALM, f, {
      p: top,
      s: [0.35, 1.5, 0.12],
      r: [Math.PI / 2 + 0.55, a, 0]
    });
  }
  part(c.solid, G.sphere, 0x6b4a2a, f, { p: [top[0], top[1] - 0.15, top[2]], s: 0.3 });
}

function deadTree(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.25);
  part(c.solid, G.taper8, 0x3a2b24, f, { s: [0.28, rnd(c, 1.2, 2.0), 0.28] });
  part(c.solid, G.box, 0x3a2b24, f, { p: [0, 1.0, 0], s: [0.1, 0.8, 0.1], r: [0, 0, 0.9] });
  part(c.solid, G.box, 0x3a2b24, f, { p: [0, 1.3, 0], s: [0.1, 0.6, 0.1], r: [0.5, 0, -0.9] });
}

function volcano(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 3.9);
  footprint(c, f, 2.5, [2.2, -1.5]);
  part(c.solid, G.cone6, 0x3f2c24, f, { s: [8.5, 4.6, 8.5] });
  part(c.solid, G.cone6, 0x4a3328, f, { p: [2.2, 0, -1.5], s: [5.5, 3.0, 5.5], r: [0, 0.5, 0] });
  part(c.solid, G.cyl6, 0x2e1f1a, f, { p: [0, 4.2, 0], s: [2.0, 0.5, 2.0] });
  part(c.glow, G.cyl6, LAVA, f, { p: [0, 4.45, 0], s: [1.5, 0.25, 1.5] });
  part(c.glow, G.cone6, LAVA_B, f, { p: [0, 4.6, 0], s: [0.8, 0.9, 0.8] });
  // Lava streaks running down two faces.
  for (const [a, len] of [
    [0.4, 2.6],
    [2.6, 3.2],
    [4.4, 2.0]
  ]) {
    part(c.glow, G.box, LAVA, f, {
      p: [Math.sin(a) * 1.2, 3.8, Math.cos(a) * 1.2],
      s: [0.28, len, 0.14],
      r: [Math.atan2(4.25, 1.0) - Math.PI / 2 + 0.2, a, 0]
    });
  }
  // Above the rim, not down in the crater: from inside the cone the light
  // would never reach the outer slopes it is meant to set glowing.
  emit(c, f, LAVA, 95, 30, [0, 6.4, 0], { flicker: 0.4 });
}

function lavaRock(c: Ctx, f: THREE.Matrix4): void {
  rock(c, f, 0.9, 0x3a2a24);
  if (c.rng() < 0.3) part(c.glow, G.box, LAVA, f, { p: [0, 0.1, 0.1], s: [0.35, 0.12, 0.2], r: [0, c.rng() * 3, 0] });
}

function forge(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 1.0);
  footprint(c, f, 0.45, [1.4, 0.4]);
  part(c.solid, G.box, 0x6d4a3a, f, { s: [1.6, 1.0, 1.2] });
  part(c.solid, G.box, 0x5a3d30, f, { p: [0, 1.0, -0.3], s: [1.0, 1.3, 0.6] });
  part(c.solid, G.box, 0x4a3128, f, { p: [0, 2.3, -0.3], s: [0.5, 0.6, 0.5] });
  part(c.glow, G.box, LAVA, f, { p: [0, 0.55, 0.62], s: [0.8, 0.45, 0.05] });
  part(c.glow, G.box, LAVA_B, f, { p: [0, 0.65, 0.63], s: [0.35, 0.22, 0.03] });
  emit(c, f, 0xff8a34, 36, 11, [0, 0.7, 0.8], { flicker: 0.7 });
  // Anvil on a stump beside the furnace.
  part(c.solid, G.cyl8, WOOD_D, f, { p: [1.4, 0, 0.4], s: [0.55, 0.55, 0.55] });
  part(c.solid, G.box, 0x4a4f57, f, { p: [1.4, 0.55, 0.4], s: [0.75, 0.22, 0.32] });
  part(c.solid, G.box, 0x4a4f57, f, { p: [1.4, 0.4, 0.4], s: [0.3, 0.16, 0.24] });
}

function lighthouse(c: Ctx, f: THREE.Matrix4): THREE.Matrix4 {
  footprint(c, f, 1.0);
  part(c.solid, G.cyl8, ROCK_D, f, { s: [2.0, 0.35, 2.0] });
  part(c.solid, G.taper8, CREAM, f, { p: [0, 0.3, 0], s: [1.5, 3.6, 1.5] });
  part(c.solid, G.cyl8, RED, f, { p: [0, 1.0, 0], s: [1.4, 0.6, 1.4] });
  part(c.solid, G.cyl8, RED, f, { p: [0, 2.3, 0], s: [1.25, 0.6, 1.25] });
  part(c.solid, G.cyl8, 0x333940, f, { p: [0, 3.9, 0], s: [1.3, 0.15, 1.3] });
  part(c.glow, G.cyl6, 0xfff2b8, f, { p: [0, 4.05, 0], s: [0.8, 0.6, 0.8] });
  part(c.solid, G.cone8, RED, f, { p: [0, 4.65, 0], s: [1.2, 0.6, 1.2] });
  return new THREE.Matrix4().makeTranslation(0, 4.35, 0).premultiply(f);
}

function cactus(c: Ctx, f: THREE.Matrix4): void {
  const h = rnd(c, 0.9, 1.6);
  footprint(c, f, 0.35);
  part(c.solid, G.cyl6, CACTUS, f, { s: [0.4, h, 0.4] });
  part(c.solid, G.cyl6, CACTUS, f, { p: [0.28, h * 0.45, 0], s: [0.25, 0.6, 0.25], r: [0, 0, -0.9] });
  part(c.solid, G.cyl6, CACTUS, f, { p: [0.5, h * 0.45 + 0.3, 0], s: [0.25, 0.5, 0.25] });
  if (c.rng() < 0.5) part(c.solid, G.ico, 0xff8fb1, f, { p: [0, h, 0], s: 0.22 });
}

function igloo(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 1.3);
  footprint(c, f, 0.5, [0, 1.3]);
  part(c.solid, G.sphere, 0xf1f5f7, f, { s: [2.6, 1.5, 2.6] });
  part(c.solid, G.cyl8, 0xe4ecef, f, { p: [0, 0, 1.2], s: [0.9, 0.9, 0.9], r: [Math.PI / 2, 0, 0] });
  part(c.solid, G.box, 0x2f3d4a, f, { p: [0, 0.05, 1.6], s: [0.5, 0.55, 0.1] });
  part(c.night, G.box, 0xffc46a, f, { p: [0, 0.05, 1.61], s: [0.46, 0.5, 0.1] });
  emit(c, f, 0xffb865, 16, 8, [0, 0.35, 1.85], { flicker: 0.25 });
}

function iceShard(c: Ctx, f: THREE.Matrix4, big = false): void {
  const h = big ? rnd(c, 2.5, 4.5) : rnd(c, 0.6, 1.6);
  const col = pick(c, [ICE, ICE_B, 0x6fc9de]);
  footprint(c, f, h * (big ? 0.2 : 0.18));
  part(c.rng() < 0.35 ? c.glow : c.solid, G.cone6, col, f, {
    s: [h * 0.3, h, h * 0.3],
    r: [rnd(c, -0.25, 0.25), c.rng() * 3, rnd(c, -0.25, 0.25)]
  });
  if (big) part(c.solid, G.cone6, ICE_B, f, { p: [h * 0.25, 0, h * 0.15], s: [h * 0.18, h * 0.55, h * 0.18], r: [0.2, 1, -0.1] });
}

function snowPile(c: Ctx, f: THREE.Matrix4): void {
  part(c.solid, G.sphere, SNOW, f, { s: [rnd(c, 0.8, 1.6), rnd(c, 0.35, 0.6), rnd(c, 0.8, 1.4)] });
}

function seal(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.6);
  part(c.solid, G.sphere, 0x9aa5ad, f, { s: [0.55, 0.45, 1.2] });
  part(c.solid, G.sphere, 0x9aa5ad, f, { p: [0, 0.2, 0.6], s: 0.4 });
  part(c.solid, G.box, 0x9aa5ad, f, { p: [0, 0.1, -0.7], s: [0.5, 0.08, 0.3], r: [0, 0, 0] });
}

/** Aurora crystal that "wakes" the aurora. */
function auroraCrystal(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.95);
  part(c.solid, G.cyl6, 0x5f9aa8, f, { s: [1.8, 0.35, 1.8] });
  part(c.solid, G.cyl6, 0x4c8290, f, { p: [0, 0.35, 0], s: [1.3, 0.25, 1.3] });
  part(c.glow, G.cone6, 0xb6f3ff, f, { p: [0, 0.6, 0], s: [0.5, 1.6, 0.5] });
  part(c.glow, G.cone6, 0x9be7ff, f, { p: [0.3, 0.6, 0.15], s: [0.3, 0.9, 0.3], r: [0.1, 0.5, -0.3] });
  emit(c, f, 0xa8ecff, 32, 14, [0, 1.3, 0], { flicker: 0.3 });
}

function pathStone(c: Ctx, f: THREE.Matrix4): void {
  part(c.solid, G.cyl6, STONE, f, { s: [rnd(c, 0.5, 0.7), 0.1, rnd(c, 0.4, 0.55)], r: [0, c.rng() * 3, 0] });
}

// ---------------------------------------------------------------- population

export type WorldProps = {
  solid: THREE.Mesh | null;
  glow: THREE.Mesh | null;
  /** Lit windows and doorways. */
  night: THREE.Mesh | null;
  /** Hub of the farm windmill: the Game mounts the spinning blades here. */
  windmillHub: THREE.Matrix4;
  /** Base of the campfire flame. */
  fire: THREE.Matrix4;
  /** Lighthouse lamp position (lit once the blog wonder is found). */
  lamp: THREE.Matrix4;
  /** Every glowing prop's light source, for the pool in `lights.ts`. */
  lights: Emitter[];
  /** Ground frame of each wonder, keyed by id, for floating markers. */
  wonderFrames: Map<string, THREE.Matrix4>;
  /** Footprints of every solid prop, for the collision grid. */
  colliders: Collider[];
};

type Exclusion = { dir: THREE.Vector3; radius: number };

function landAt(ctx: Ctx, biome: BiomeId, east: number, north: number, yaw: number, lift = 0): THREE.Matrix4 {
  const d = offsetDir(biomeById(biome).center.clone(), east, north);
  return surfaceFrame(d, yaw, new THREE.Matrix4(), lift);
}

export function buildWorldProps(seed: number): WorldProps {
  const ctx: Ctx = {
    solid: new PropBatch(),
    glow: new PropBatch(),
    night: new PropBatch(),
    rng: seededRng(deriveSeed(seed, 3)),
    colliders: [],
    lights: []
  };
  const exclusions: Exclusion[] = [];
  const exclude = (biome: BiomeId, east: number, north: number, radius: number) => {
    exclusions.push({ dir: offsetDir(biomeById(biome).center.clone(), east, north), radius });
  };
  const wonderFrames = new Map<string, THREE.Matrix4>();
  const wonderDirs = new Map<string, THREE.Vector3>();
  for (const w of WONDERS) {
    const d = wonderDir(w);
    wonderDirs.set(w.id, d);
    wonderFrames.set(w.id, surfaceFrame(d, w.yaw, new THREE.Matrix4()));
    exclusions.push({ dir: d, radius: 2.4 });
  }
  const wf = (id: string) => wonderFrames.get(id)!;

  // --- Fernwood: campfire camp, cabin, and a mountain range behind it.
  campfire(ctx, wf('about'));
  const fire = wf('about').clone();
  tent(ctx, landAt(ctx, 'forest', 2.6, -1.4, -0.7));
  logPile(ctx, landAt(ctx, 'forest', -2.2, 0.4, 0.9));
  exclude('forest', 2.6, -1.4, 1.6);
  exclude('forest', -2.2, 0.4, 1.4);
  cabin(ctx, wf('work'));
  exclusions.push({ dir: wonderDirs.get('work')!, radius: 3.0 });
  for (const [e, n, r, h] of [
    [1.5, 9.5, 2.4, 5.2],
    [5.0, 8.0, 2.0, 4.4],
    [-2.5, 10.5, 2.1, 4.6],
    [8.0, 6.0, 1.7, 3.6],
    [-6.0, 9.0, 1.8, 3.9]
  ]) {
    mountain(ctx, landAt(ctx, 'forest', e, n, ctx.rng() * 3), r, h);
    exclude('forest', e, n, r + 0.8);
  }

  // --- Clover Fields: barn yard, crops, windmill, fences, and the two wonders.
  barn(ctx, landAt(ctx, 'farm', 0.5, 5.5, 0.25));
  exclude('farm', 0.5, 5.5, 3.0);
  silo(ctx, landAt(ctx, 'farm', 3.0, 6.2, 0));
  exclude('farm', 3.0, 6.2, 1.4);
  const windmillHub = windmill(ctx, landAt(ctx, 'farm', -3.6, 6.8, 0.1));
  exclude('farm', -3.6, 6.8, 1.8);
  for (let i = 0; i < 4; i++) {
    cropRow(ctx, landAt(ctx, 'farm', 1.2 + i * 1.1, 1.2, 0.15));
  }
  exclude('farm', 2.8, 1.2, 3.2);
  fence(ctx, landAt(ctx, 'farm', -1.5, -0.5, 0.15), 6);
  fence(ctx, landAt(ctx, 'farm', -4.5, 1.8, 0.15 + Math.PI / 2), 4.5);
  exclude('farm', -1.5, -0.5, 1.2);
  exclude('farm', -4.5, 1.8, 1.2);
  for (const [e, n] of [
    [-2.5, 3.2],
    [-1.6, 3.9],
    [-3.2, 4.4]
  ]) {
    hayBale(ctx, landAt(ctx, 'farm', e, n, 0));
    exclude('farm', e, n, 0.9);
  }
  scarecrow(ctx, landAt(ctx, 'farm', 4.6, 2.6, -0.3));
  exclude('farm', 4.6, 2.6, 0.8);
  for (let i = 0; i < 5; i++) sunflower(ctx, landAt(ctx, 'farm', -1.4 + i * 0.7, 7.4, 0));
  exclude('farm', 0, 7.4, 2.2);
  mailbox(ctx, wf('contact'));
  signpost(ctx, wf('resume'));

  // --- Sunmark Shore: lighthouse on the dunes.
  const lamp = lighthouse(ctx, wf('blog'));
  exclusions.push({ dir: wonderDirs.get('blog')!, radius: 3.2 });

  // --- Ember Heights: the volcano and the forge.
  volcano(ctx, landAt(ctx, 'ember', -1.5, 3.5, 0));
  exclude('ember', -1.5, 3.5, 6.5);
  forge(ctx, wf('projects'));
  exclusions.push({ dir: wonderDirs.get('projects')!, radius: 3.2 });

  // --- Northlight: the crystal, an igloo, and a ring of big shards.
  auroraCrystal(ctx, wf('education'));
  igloo(ctx, landAt(ctx, 'arctic', 4.5, 3.2, -0.9));
  exclude('arctic', 4.5, 3.2, 2.6);
  for (let i = 0; i < 7; i++) {
    const a = (i / 7) * Math.PI * 2 + 0.3;
    const e = Math.cos(a) * 9.5;
    const n = Math.sin(a) * 9.5;
    iceShard(ctx, landAt(ctx, 'arctic', e, n, ctx.rng() * 3), true);
    exclude('arctic', e, n, 1.6);
  }
  for (const [e, n, y] of [
    [-3.5, -2.5, 0.4],
    [2.5, -4.5, 2.2],
    [-6, 2, 1.0]
  ]) {
    seal(ctx, landAt(ctx, 'arctic', e, n, y));
    exclude('arctic', e, n, 1.0);
  }

  // --- Path stones linking the wonders in a loop.
  const loop = ['about', 'work', 'resume', 'contact', 'blog', 'projects', 'education', 'about'];
  const pa = new THREE.Vector3();
  const pb = new THREE.Vector3();
  const pd = new THREE.Vector3();
  const pf = new THREE.Matrix4();
  for (let i = 0; i < loop.length - 1; i++) {
    pa.copy(wonderDirs.get(loop[i])!);
    pb.copy(wonderDirs.get(loop[i + 1])!);
    const ang = pa.angleTo(pb);
    const steps = Math.floor((ang * PLANET_RADIUS) / 1.6);
    for (let s = 1; s < steps; s++) {
      const t = s / steps;
      // Spherical interpolation along the great circle, with a gentle wobble.
      pd.copy(pa).multiplyScalar(Math.sin((1 - t) * ang)).addScaledVector(pb, Math.sin(t * ang)).divideScalar(Math.sin(ang)).normalize();
      offsetDir(pd, Math.sin(t * 9 + i) * 0.5, Math.cos(t * 7) * 0.4);
      if (isOcean(pd) || oceanField(pd) < 0.05) continue;
      pathStone(ctx, surfaceFrame(pd, 0, pf, 0.02));
      exclusions.push({ dir: pd.clone(), radius: 0.7 });
    }
  }

  // --- Scatter: random props by biome, avoiding landmarks and the path.
  const d = new THREE.Vector3();
  const f = new THREE.Matrix4();
  const blocked = (dir: THREE.Vector3): boolean => {
    for (const ex of exclusions) {
      if (dir.angleTo(ex.dir) * PLANET_RADIUS < ex.radius) return true;
    }
    return false;
  };
  const N = 3400;
  for (let i = 0; i < N; i++) {
    d.set(ctx.rng() * 2 - 1, ctx.rng() * 2 - 1, ctx.rng() * 2 - 1);
    if (d.lengthSq() < 0.05) continue;
    d.normalize();
    if (oceanField(d) < 0.07) continue;
    if (blocked(d)) continue;
    const biome = landBiomeAt(d);
    const r = ctx.rng();
    surfaceFrame(d, ctx.rng() * Math.PI * 2, f);
    switch (biome.id) {
      case 'forest':
        if (r < 0.2) pine(ctx, f);
        else if (r < 0.27) roundTree(ctx, f);
        else if (r < 0.32) rock(ctx, f);
        else if (r < 0.37) bush(ctx, f);
        else if (r < 0.55) grass(ctx, f, 0x5fb44e);
        else if (r < 0.62) flower(ctx, f);
        break;
      case 'farm':
        if (r < 0.08) roundTree(ctx, f);
        else if (r < 0.3) grass(ctx, f, 0x8fdb60);
        else if (r < 0.42) flower(ctx, f);
        else if (r < 0.47) bush(ctx, f);
        else if (r < 0.49) hayBale(ctx, f);
        break;
      case 'shore':
        if (r < 0.1) palm(ctx, f);
        else if (r < 0.2) cactus(ctx, f);
        else if (r < 0.28) rock(ctx, f, 0.8, 0xc9b07a);
        else if (r < 0.36) grass(ctx, f, 0xd9c37a);
        break;
      case 'ember':
        if (r < 0.22) lavaRock(ctx, f);
        else if (r < 0.28) deadTree(ctx, f);
        else if (r < 0.34) palm(ctx, f);
        else if (r < 0.4) grass(ctx, f, 0x8a6a3a);
        break;
      case 'arctic':
        if (r < 0.14) iceShard(ctx, f);
        else if (r < 0.26) snowPile(ctx, f);
        else if (r < 0.3) rock(ctx, f, 0.7, 0xb8c4ca);
        else if (r < 0.34) snowPine(ctx, f);
        break;
    }
  }

  const solidMat = new THREE.MeshStandardMaterial({ vertexColors: true, flatShading: true, roughness: 0.85 });
  const glowMat = new THREE.MeshBasicMaterial({ vertexColors: true });
  const solid = ctx.solid.build(solidMat);
  const glow = ctx.glow.build(glowMat);
  const night = ctx.night.build(new THREE.MeshBasicMaterial({ vertexColors: true }));
  if (night) night.name = 'night';
  if (solid) {
    solid.castShadow = true;
    solid.receiveShadow = true;
    solid.name = 'props';
  }
  if (glow) glow.name = 'glow';
  return { solid, glow, night, windmillHub, fire, lamp, wonderFrames, colliders: ctx.colliders, lights: ctx.lights };
}

// ---------------------------------------------------------------- dynamic

/** Four-bladed windmill rotor; spin it around local Z. */
export function buildWindmillBlades(): THREE.Group {
  const g = new THREE.Group();
  const mat = new THREE.MeshStandardMaterial({ color: 0xf4ead6, roughness: 0.8 });
  const hub = new THREE.Mesh(new THREE.CylinderGeometry(0.2, 0.2, 0.3, 8), new THREE.MeshStandardMaterial({ color: 0x6e4a2f }));
  hub.rotation.x = Math.PI / 2;
  g.add(hub);
  for (let i = 0; i < 4; i++) {
    const arm = new THREE.Mesh(new THREE.BoxGeometry(0.1, 2.4, 0.06), new THREE.MeshStandardMaterial({ color: 0x6e4a2f }));
    arm.position.y = 1.2;
    const sail = new THREE.Mesh(new THREE.BoxGeometry(0.5, 1.7, 0.04), mat);
    sail.position.set(0.3, 1.4, 0.03);
    const blade = new THREE.Group();
    blade.add(arm, sail);
    blade.rotation.z = (i / 4) * Math.PI * 2;
    arm.castShadow = true;
    sail.castShadow = true;
    g.add(blade);
  }
  return g;
}

/** Stacked flame cones; the Game flickers their scale. */
export function buildFlame(): THREE.Group {
  const g = new THREE.Group();
  const outer = new THREE.Mesh(
    new THREE.ConeGeometry(0.45, 1.1, 6).translate(0, 0.55, 0),
    new THREE.MeshBasicMaterial({ color: 0xff7a1a, transparent: true, opacity: 0.9 })
  );
  const inner = new THREE.Mesh(
    new THREE.ConeGeometry(0.25, 0.75, 6).translate(0, 0.37, 0),
    new THREE.MeshBasicMaterial({ color: 0xffd35c })
  );
  g.add(outer, inner);
  return g;
}

export { BIOMES };
