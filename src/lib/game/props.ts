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
import { deriveSeed, fbm3, seededRng } from './noise';
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
const BIRCH = 0xe8e4dc;
const BIRCH_MARK = 0x4a4a48;
const BIRCH_LEAF = [0x8fc45a, 0xa8d46c, 0xd9c04a];
/** Amberfall's canopy, and the leaves that fall out of it. */
const AUTUMN = [0xd9762f, 0xc44a2c, 0xe0a12f, 0xb8862f];
const WILLOW = 0x7fae6a;
const WILLOW_D = 0x6b9a58;
const FERN = [0x4f9a5c, 0x3f8a4c, 0x5fae68];
const TOADSTOOL = [0xd9463f, 0xe08a3a, 0xc95a8a];
const MOSS = 0x5c8a4a;
const MESA = [0xb06a3a, 0x9a5730];
const BASALT = 0x3a3540;
const CRYSTAL = 0xb98fe0;
const BONE = 0xe6dfcb;
const LANTERN = 0xffd27a;

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

// ------------------------------------------------------- trees and undergrowth

function birch(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.9, 1.3);
  footprint(c, f, 0.34 * s);
  part(c.solid, G.taper8, BIRCH, f, { s: [0.22 * s, 2.4 * s, 0.22 * s] });
  for (let i = 0; i < 3; i++) {
    part(c.solid, G.box, BIRCH_MARK, f, {
      p: [0, (0.4 + i * 0.6) * s, 0.1 * s],
      s: [0.16 * s, 0.05 * s, 0.05 * s],
      r: [0, c.rng() * 3, 0]
    });
  }
  const col = pick(c, BIRCH_LEAF);
  part(c.solid, G.ico, col, f, { p: [0, 2.1 * s, 0], s: [1.3 * s, 1.5 * s, 1.3 * s], r: [0, c.rng() * 3, 0] });
  part(c.solid, G.ico, col, f, { p: [0.35 * s, 2.7 * s, -0.2 * s], s: [0.8 * s, 0.9 * s, 0.8 * s], r: [0.2, c.rng() * 3, 0] });
}

/** Broad autumn maple. The whole point of Amberfall. */
function maple(c: Ctx, f: THREE.Matrix4, scale = 1): void {
  const s = scale * rnd(c, 0.85, 1.25);
  const col = pick(c, AUTUMN);
  footprint(c, f, 0.5 * s);
  part(c.solid, G.taper8, 0x6e4a30, f, { s: [0.36 * s, 1.3 * s, 0.36 * s] });
  part(c.solid, G.box, 0x6e4a30, f, { p: [0.3 * s, 1.1 * s, 0], s: [0.7 * s, 0.14 * s, 0.14 * s], r: [0, 0, 0.5] });
  part(c.solid, G.ico, col, f, { p: [0, 1.5 * s, 0], s: [2.4 * s, 1.5 * s, 2.4 * s], r: [0, c.rng() * 3, 0] });
  part(c.solid, G.ico, pick(c, AUTUMN), f, { p: [0.8 * s, 1.9 * s, 0.3 * s], s: [1.3 * s, 1.0 * s, 1.3 * s], r: [0.3, c.rng() * 3, 0] });
  part(c.solid, G.ico, pick(c, AUTUMN), f, { p: [-0.7 * s, 1.8 * s, -0.4 * s], s: [1.1 * s, 0.9 * s, 1.1 * s], r: [0.2, c.rng() * 3, 0] });
}

/** Willow: a low crown with fronds hanging off it. */
function willow(c: Ctx, f: THREE.Matrix4, scale = 1): void {
  const s = scale * rnd(c, 0.9, 1.2);
  footprint(c, f, 0.55 * s);
  part(c.solid, G.taper8, 0x6a5138, f, { s: [0.42 * s, 1.6 * s, 0.42 * s] });
  part(c.solid, G.ico, WILLOW, f, { p: [0, 1.9 * s, 0], s: [2.6 * s, 1.1 * s, 2.6 * s], r: [0, c.rng() * 3, 0] });
  for (let i = 0; i < 9; i++) {
    const a = (i / 9) * Math.PI * 2 + c.rng() * 0.3;
    const r = (0.9 + c.rng() * 0.35) * s;
    part(c.solid, G.cone6, i % 2 ? WILLOW : WILLOW_D, f, {
      p: [Math.cos(a) * r, 1.55 * s, Math.sin(a) * r],
      s: [0.3 * s, -1.3 * s * rnd(c, 0.7, 1.3), 0.3 * s]
    });
  }
}

/** Apple tree; the fruit is a handful of red pips in the canopy. */
function appleTree(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.9, 1.15);
  footprint(c, f, 0.45 * s);
  part(c.solid, G.cyl6, TRUNK, f, { s: [0.3 * s, 1.0 * s, 0.3 * s] });
  part(c.solid, G.ico, 0x4f9f40, f, { p: [0, 1.0 * s, 0], s: [1.9 * s, 1.6 * s, 1.9 * s], r: [0, c.rng() * 3, 0] });
  for (let i = 0; i < 5; i++) {
    const a = c.rng() * Math.PI * 2;
    const r = 0.55 * s + c.rng() * 0.3;
    part(c.solid, G.sphere, 0xd94f3a, f, { p: [Math.cos(a) * r, (1.0 + c.rng() * 0.7) * s, Math.sin(a) * r], s: 0.17 * s });
  }
}

function stump(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.7, 1.1);
  footprint(c, f, 0.4 * s);
  part(c.solid, G.cyl8, WOOD_D, f, { s: [0.7 * s, 0.42 * s, 0.7 * s] });
  part(c.solid, G.cyl8, 0xb08b5f, f, { p: [0, 0.42 * s, 0], s: [0.62 * s, 0.06, 0.62 * s] });
  if (c.rng() < 0.5) part(c.solid, G.ico, MOSS, f, { p: [0.2 * s, 0.4 * s, 0.1 * s], s: [0.4 * s, 0.2 * s, 0.4 * s] });
}

function fallenLog(c: Ctx, f: THREE.Matrix4): void {
  const len = rnd(c, 1.8, 3.2);
  footprint(c, f, 0.45, [0, len * 0.25]);
  footprint(c, f, 0.45, [0, -len * 0.25]);
  part(c.solid, G.cyl8, WOOD_D, f, { p: [0, 0.24, 0], s: [0.48, len, 0.48], r: [Math.PI / 2, c.rng() * 0.6, 0] });
  part(c.solid, G.ico, MOSS, f, { p: [0.1, 0.42, len * 0.15], s: [0.5, 0.2, 0.6] });
}

function fern(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.6, 1.0);
  for (let i = 0; i < 5; i++) {
    const a = (i / 5) * Math.PI * 2 + c.rng() * 0.5;
    part(c.solid, G.cone4, pick(c, FERN), f, {
      p: [Math.cos(a) * 0.1 * s, 0, Math.sin(a) * 0.1 * s],
      s: [0.18 * s, 0.85 * s, 0.1 * s],
      r: [Math.cos(a) * 0.6, a, Math.sin(a) * 0.6]
    });
  }
}

/** Toadstool. A quarter of them glow, which the marsh and the forest need. */
function mushroom(c: Ctx, f: THREE.Matrix4, glowing = false): void {
  const s = rnd(c, 0.5, 1.1);
  const cap = glowing ? 0x9fe4d6 : pick(c, TOADSTOOL);
  part(c.solid, G.cyl6, 0xf0e6d2, f, { s: [0.16 * s, 0.42 * s, 0.16 * s] });
  part(glowing ? c.glow : c.solid, G.sphere, cap, f, { p: [0, 0.4 * s, 0], s: [0.5 * s, 0.36 * s, 0.5 * s] });
  if (!glowing) {
    part(c.solid, G.sphere, 0xfdf6ea, f, { p: [0.12 * s, 0.52 * s, 0.05 * s], s: 0.09 * s });
    part(c.solid, G.sphere, 0xfdf6ea, f, { p: [-0.1 * s, 0.5 * s, -0.1 * s], s: 0.07 * s });
  } else if (c.rng() < 0.4) {
    emit(c, f, 0x8fe8d8, 7, 5, [0, 0.5 * s, 0], { flicker: 0.15 });
  }
}

/** Knee-high cluster of toadstools around a stone. */
function mushroomRing(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.5);
  part(c.solid, G.dodeca, ROCK_D, f, { s: [0.6, 0.3, 0.6], r: [0, c.rng() * 3, 0] });
  for (let i = 0; i < 6; i++) {
    const a = (i / 6) * Math.PI * 2 + c.rng() * 0.4;
    const m = new THREE.Matrix4().makeTranslation(Math.cos(a) * 0.75, 0, Math.sin(a) * 0.75).premultiply(f);
    mushroom(c, m, c.rng() < 0.3);
  }
}

/** Head-high glowing mushroom. A landmark for the marsh. */
function bigMushroom(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 1.6, 2.4);
  footprint(c, f, 0.4 * s);
  part(c.solid, G.taper8, 0xe6dcc4, f, { s: [0.34 * s, 1.5 * s, 0.34 * s] });
  part(c.glow, G.sphere, 0x8fe0cf, f, { p: [0, 1.35 * s, 0], s: [1.5 * s, 0.9 * s, 1.5 * s] });
  part(c.solid, G.cyl8, 0x6fbfae, f, { p: [0, 1.28 * s, 0], s: [1.2 * s, 0.12 * s, 1.2 * s] });
  emit(c, f, 0x7fe0cc, 20, 12, [0, 1.5 * s, 0], { flicker: 0.2 });
}

function berryBush(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.6, 1.0);
  footprint(c, f, 0.5 * s);
  part(c.solid, G.ico, 0x3f7f43, f, { s: [s * 1.3, s * 0.9, s * 1.3], r: [0, c.rng() * 3, 0] });
  for (let i = 0; i < 4; i++) {
    const a = c.rng() * Math.PI * 2;
    part(c.solid, G.sphere, pick(c, [0xc23a5a, 0x6a3fa0, 0xd9603f]), f, {
      p: [Math.cos(a) * 0.4 * s, (0.3 + c.rng() * 0.4) * s, Math.sin(a) * 0.4 * s],
      s: 0.12 * s
    });
  }
}

function hedge(c: Ctx, f: THREE.Matrix4): void {
  const len = rnd(c, 2.0, 3.6);
  const links = Math.max(1, Math.round(len / 0.6));
  for (let i = 0; i <= links; i++) footprint(c, f, 0.36, [-len / 2 + (i / links) * len, 0]);
  part(c.solid, G.box, 0x3f8a45, f, { s: [len, 0.75, 0.7] });
  part(c.solid, G.box, 0x4c9c50, f, { p: [0, 0.75, 0], s: [len * 0.96, 0.12, 0.66] });
}

function cattail(c: Ctx, f: THREE.Matrix4): void {
  for (let i = 0; i < 4; i++) {
    const x = rnd(c, -0.3, 0.3);
    const z = rnd(c, -0.3, 0.3);
    const h = rnd(c, 0.9, 1.5);
    part(c.solid, G.cyl6, 0x6f9a4a, f, { p: [x, 0, z], s: [0.05, h, 0.05], r: [rnd(c, -0.12, 0.12), 0, rnd(c, -0.12, 0.12)] });
    if (c.rng() < 0.7) part(c.solid, G.cyl6, 0x6b4a2c, f, { p: [x, h * 0.86, z], s: [0.11, 0.34, 0.11] });
  }
}

function reeds(c: Ctx, f: THREE.Matrix4): void {
  for (let i = 0; i < 6; i++) {
    part(c.solid, G.cone4, pick(c, [0x7fae5a, 0x94bd68, 0x6b9a4a]), f, {
      p: [rnd(c, -0.35, 0.35), 0, rnd(c, -0.35, 0.35)],
      s: [0.08, rnd(c, 0.6, 1.2), 0.08],
      r: [rnd(c, -0.35, 0.35), c.rng() * 3, rnd(c, -0.35, 0.35)]
    });
  }
}

function lilyPad(c: Ctx, f: THREE.Matrix4): void {
  for (let i = 0; i < 3; i++) {
    part(c.solid, G.cyl6, pick(c, [0x4f8f52, 0x5ea05f]), f, {
      p: [rnd(c, -0.5, 0.5), 0.02, rnd(c, -0.5, 0.5)],
      s: [rnd(c, 0.5, 0.85), 0.05, rnd(c, 0.5, 0.85)],
      r: [0, c.rng() * 3, 0]
    });
  }
  if (c.rng() < 0.5) part(c.solid, G.ico, 0xf6e8f0, f, { p: [0.1, 0.1, 0.1], s: 0.2 });
}

function pumpkin(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.5, 0.85);
  part(c.solid, G.sphere, 0xe0812a, f, { p: [0, 0.22 * s, 0], s: [0.62 * s, 0.5 * s, 0.62 * s] });
  part(c.solid, G.cyl6, 0x4f7a35, f, { p: [0, 0.42 * s, 0], s: [0.1 * s, 0.2 * s, 0.1 * s] });
}

function leafPile(c: Ctx, f: THREE.Matrix4): void {
  for (let i = 0; i < 4; i++) {
    part(c.solid, G.sphere, pick(c, AUTUMN), f, {
      p: [rnd(c, -0.5, 0.5), 0, rnd(c, -0.5, 0.5)],
      s: [rnd(c, 0.5, 1.0), rnd(c, 0.15, 0.3), rnd(c, 0.5, 1.0)]
    });
  }
}

function tumbleweed(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.4, 0.7);
  part(c.solid, G.ico, 0xa88a55, f, { p: [0, s * 0.8, 0], s: s * 1.6, r: [c.rng(), c.rng() * 3, c.rng()] });
}

function agave(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.6, 1.0);
  footprint(c, f, 0.4 * s);
  for (let i = 0; i < 7; i++) {
    const a = (i / 7) * Math.PI * 2;
    part(c.solid, G.cone4, pick(c, [0x6f9a6a, 0x5c8a5c]), f, {
      p: [0, 0.05, 0],
      s: [0.2 * s, 1.0 * s, 0.1 * s],
      r: [0.9, a, 0]
    });
  }
}

function barrelCactus(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.5, 0.9);
  footprint(c, f, 0.4 * s);
  part(c.solid, G.cyl8, CACTUS, f, { s: [0.9 * s, 0.7 * s, 0.9 * s] });
  part(c.solid, G.sphere, CACTUS, f, { p: [0, 0.7 * s, 0], s: [0.9 * s, 0.4 * s, 0.9 * s] });
  if (c.rng() < 0.6) part(c.solid, G.ico, 0xf2c14e, f, { p: [0, 0.85 * s, 0], s: 0.22 * s });
}

function deadBush(c: Ctx, f: THREE.Matrix4): void {
  for (let i = 0; i < 5; i++) {
    part(c.solid, G.box, 0x8a6f4a, f, {
      p: [0, 0.2, 0],
      s: [0.05, rnd(c, 0.4, 0.7), 0.05],
      r: [rnd(c, -0.7, 0.7), c.rng() * 3, rnd(c, -0.7, 0.7)]
    });
  }
}

// ------------------------------------------------------- rock and ground forms

function boulder(c: Ctx, f: THREE.Matrix4, color = ROCK, mossy = true): void {
  const s = rnd(c, 1.1, 2.0);
  footprint(c, f, 0.7 * s);
  part(c.solid, G.dodeca, color, f, { s: [s * 1.3, s * 0.9, s * 1.2], r: [c.rng() * 0.3, c.rng() * 3, c.rng() * 0.3] });
  part(c.solid, G.dodeca, ROCK_D, f, { p: [s * 0.5, 0, s * 0.3], s: [s * 0.6, s * 0.4, s * 0.6], r: [0, c.rng() * 3, 0] });
  if (mossy && c.rng() < 0.6) part(c.solid, G.sphere, MOSS, f, { p: [0, s * 0.68, 0], s: [s * 0.9, s * 0.18, s * 0.85] });
}

function standingStone(c: Ctx, f: THREE.Matrix4, scale = 1): void {
  const h = rnd(c, 2.0, 3.4) * scale;
  footprint(c, f, 0.45 * scale);
  part(c.solid, G.box, pick(c, [0x8a8f96, 0x777d85]), f, {
    s: [0.8 * scale, h, 0.5 * scale],
    r: [rnd(c, -0.06, 0.06), c.rng() * 3, rnd(c, -0.06, 0.06)]
  });
  part(c.solid, G.box, MOSS, f, { p: [0, h * 0.2, 0.25 * scale], s: [0.6 * scale, 0.3, 0.06] });
}

/** Layered desert butte. Reads as the horizon line of Dune Reach. */
function mesa(c: Ctx, f: THREE.Matrix4, radius: number, height: number): void {
  footprint(c, f, radius * 0.95);
  footprint(c, f, radius * 0.6, [radius * 0.8, radius * 0.4]);
  const bands = 4;
  for (let i = 0; i < bands; i++) {
    const t = i / bands;
    part(c.solid, G.cyl6, i % 2 ? MESA[0] : MESA[1], f, {
      p: [0, height * t, 0],
      s: [radius * 2 * (1 - t * 0.35), (height / bands) * 1.02, radius * 2 * (1 - t * 0.35)],
      r: [0, i * 0.2, 0]
    });
  }
  part(c.solid, G.cyl6, 0xd9a068, f, { p: [0, height, 0], s: [radius * 1.32, height * 0.06, radius * 1.32] });
  part(c.solid, G.cyl6, MESA[1], f, {
    p: [radius * 0.8, 0, radius * 0.4],
    s: [radius * 1.1, height * 0.55, radius * 1.1],
    r: [0, 0.6, 0]
  });
}

/** Wind-cut rock arch. */
function rockArch(c: Ctx, f: THREE.Matrix4): void {
  const w = rnd(c, 3.4, 4.6);
  const h = rnd(c, 3.0, 4.2);
  footprint(c, f, 0.9, [-w / 2, 0]);
  footprint(c, f, 0.9, [w / 2, 0]);
  part(c.solid, G.box, MESA[0], f, { p: [-w / 2, 0, 0], s: [1.5, h, 1.7], r: [0, 0.2, 0.05] });
  part(c.solid, G.box, MESA[1], f, { p: [w / 2, 0, 0], s: [1.5, h, 1.7], r: [0, -0.2, -0.05] });
  part(c.solid, G.box, MESA[0], f, { p: [0, h, 0], s: [w + 1.4, 0.9, 1.5] });
  part(c.solid, G.box, MESA[1], f, { p: [0, h + 0.9, 0], s: [w * 0.7, 0.5, 1.3] });
}

/** Ribs and a skull, bleached by the sun. */
function boneArch(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.6);
  for (let i = 0; i < 5; i++) {
    part(c.solid, G.box, BONE, f, {
      p: [0, 0, -0.9 + i * 0.45],
      s: [0.12, 1.1 + Math.sin(i) * 0.2, 0.12],
      r: [0, 0, 0.35]
    });
    part(c.solid, G.box, BONE, f, {
      p: [0, 0, -0.9 + i * 0.45],
      s: [0.12, 1.1 + Math.sin(i) * 0.2, 0.12],
      r: [0, 0, -0.35]
    });
  }
  part(c.solid, G.ico, BONE, f, { p: [0, 0.2, 1.3], s: [0.5, 0.42, 0.66] });
  part(c.solid, G.box, 0x30271f, f, { p: [-0.12, 0.3, 1.55], s: [0.1, 0.09, 0.05] });
  part(c.solid, G.box, 0x30271f, f, { p: [0.12, 0.3, 1.55], s: [0.1, 0.09, 0.05] });
}

function basaltColumn(c: Ctx, f: THREE.Matrix4): void {
  const n = 3 + Math.floor(c.rng() * 3);
  footprint(c, f, 0.6);
  for (let i = 0; i < n; i++) {
    const a = (i / n) * Math.PI * 2;
    part(c.solid, G.cyl6, i % 2 ? BASALT : 0x2e2a33, f, {
      p: [Math.cos(a) * 0.4, 0, Math.sin(a) * 0.4],
      s: [0.55, rnd(c, 1.2, 2.8), 0.55],
      r: [rnd(c, -0.05, 0.05), a, rnd(c, -0.05, 0.05)]
    });
  }
}

/** Vent of hot gas: a crusted cone with a glowing throat. */
function steamVent(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.6);
  part(c.solid, G.cone6, 0x3a2c26, f, { s: [1.5, 0.8, 1.5] });
  part(c.glow, G.cyl6, LAVA_B, f, { p: [0, 0.72, 0], s: [0.4, 0.14, 0.4] });
  emit(c, f, 0xff7a2a, 18, 9, [0, 0.9, 0], { flicker: 0.8 });
}

function crystalCluster(c: Ctx, f: THREE.Matrix4, color = CRYSTAL): void {
  const s = rnd(c, 0.7, 1.4);
  footprint(c, f, 0.4 * s);
  part(c.solid, G.dodeca, 0x4a4453, f, { s: [0.9 * s, 0.3 * s, 0.9 * s] });
  for (let i = 0; i < 3; i++) {
    const a = (i / 3) * Math.PI * 2 + c.rng();
    part(c.glow, G.cone6, color, f, {
      p: [Math.cos(a) * 0.22 * s, 0.15 * s, Math.sin(a) * 0.22 * s],
      s: [0.3 * s, rnd(c, 0.8, 1.6) * s, 0.3 * s],
      r: [Math.sin(a) * 0.25, a, -Math.cos(a) * 0.25]
    });
  }
  if (c.rng() < 0.35) emit(c, f, color, 12, 8, [0, 1.0 * s, 0], { flicker: 0.2 });
}

function icicles(c: Ctx, f: THREE.Matrix4): void {
  for (let i = 0; i < 4; i++) {
    part(c.solid, G.cone6, pick(c, [ICE, ICE_B]), f, {
      p: [rnd(c, -0.4, 0.4), 0, rnd(c, -0.4, 0.4)],
      s: [0.16, rnd(c, 0.5, 1.1), 0.16]
    });
  }
}

// ------------------------------------------------------- built things

/** Lamp on a post. The workhorse of a world that is always night. */
function lanternPost(c: Ctx, f: THREE.Matrix4, color = LANTERN): void {
  footprint(c, f, 0.28);
  part(c.solid, G.cyl6, 0x4a4038, f, { s: [0.26, 0.16, 0.26] });
  part(c.solid, G.cyl6, WOOD_D, f, { s: [0.12, 1.9, 0.12] });
  part(c.solid, G.box, WOOD_D, f, { p: [0.18, 1.88, 0], s: [0.44, 0.08, 0.08] });
  part(c.solid, G.box, 0x3a352f, f, { p: [0.34, 1.66, 0], s: [0.26, 0.06, 0.26] });
  part(c.glow, G.box, color, f, { p: [0.34, 1.5, 0], s: [0.2, 0.28, 0.2] });
  part(c.solid, G.cone6, 0x3a352f, f, { p: [0.34, 1.78, 0], s: [0.34, 0.18, 0.34] });
  emit(c, f, color, 16, 9, [0.34, 1.6, 0], { flicker: 0.3 });
}

/** Torch on a stake: cheaper and smaller than a lantern post. */
function torch(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.22);
  part(c.solid, G.cyl6, WOOD_D, f, { s: [0.1, 1.3, 0.1], r: [rnd(c, -0.1, 0.1), 0, rnd(c, -0.1, 0.1)] });
  part(c.glow, G.ico, 0xffb04a, f, { p: [0, 1.3, 0], s: [0.28, 0.4, 0.28] });
  emit(c, f, 0xff9a3c, 20, 9, [0, 1.4, 0], { flicker: 0.9 });
}

function bench(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.7);
  part(c.solid, G.box, WOOD_D, f, { p: [-0.6, 0, 0], s: [0.12, 0.42, 0.4] });
  part(c.solid, G.box, WOOD_D, f, { p: [0.6, 0, 0], s: [0.12, 0.42, 0.4] });
  part(c.solid, G.box, WOOD, f, { p: [0, 0.42, 0], s: [1.6, 0.1, 0.5] });
  part(c.solid, G.box, WOOD, f, { p: [0, 0.62, -0.22], s: [1.6, 0.35, 0.08], r: [-0.15, 0, 0] });
}

function well(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.8);
  part(c.solid, G.cyl8, ROCK, f, { s: [1.5, 0.7, 1.5] });
  part(c.solid, G.cyl8, 0x2b3a44, f, { p: [0, 0.7, 0], s: [1.2, 0.08, 1.2] });
  part(c.solid, G.box, WOOD_D, f, { p: [-0.55, 0.7, 0], s: [0.14, 1.3, 0.14] });
  part(c.solid, G.box, WOOD_D, f, { p: [0.55, 0.7, 0], s: [0.14, 1.3, 0.14] });
  part(c.solid, G.cone4, 0x8b4a3c, f, { p: [0, 1.95, 0], s: [2.0, 0.7, 1.6], r: [0, Math.PI / 4, 0] });
  part(c.solid, G.cyl6, WOOD, f, { p: [0, 1.75, 0], s: [0.14, 1.1, 0.14], r: [0, 0, Math.PI / 2] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 1.3, 0], s: [0.3, 0.3, 0.3] });
}

function beehive(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.45);
  part(c.solid, G.box, 0xd9c8a0, f, { s: [0.7, 0.2, 0.7] });
  for (let i = 0; i < 3; i++) {
    part(c.solid, G.box, i % 2 ? 0xe8d9b2 : 0xd2bd92, f, { p: [0, 0.2 + i * 0.28, 0], s: [0.75, 0.28, 0.75] });
  }
  part(c.solid, G.box, 0x8b4a3c, f, { p: [0, 1.04, 0], s: [0.86, 0.1, 0.86] });
}

function birdhouse(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.22);
  part(c.solid, G.box, WOOD_D, f, { s: [0.1, 1.8, 0.1] });
  part(c.solid, G.box, 0xe0d2b4, f, { p: [0, 1.8, 0], s: [0.44, 0.42, 0.4] });
  part(c.solid, G.cone4, 0xc9524a, f, { p: [0, 2.2, 0], s: [0.62, 0.3, 0.56], r: [0, Math.PI / 4, 0] });
  part(c.solid, G.cyl6, 0x2a2420, f, { p: [0, 1.95, 0.2], s: [0.14, 0.06, 0.14], r: [Math.PI / 2, 0, 0] });
}

function cart(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.8);
  part(c.solid, G.box, WOOD, f, { p: [0, 0.5, 0], s: [1.1, 0.4, 1.8] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 0.75, -0.85], s: [1.1, 0.5, 0.1] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 0.75, 0.85], s: [1.1, 0.5, 0.1] });
  part(c.solid, G.cyl8, WOOD_D, f, { p: [-0.6, 0.42, -0.4], s: [0.84, 0.12, 0.84], r: [0, 0, Math.PI / 2] });
  part(c.solid, G.cyl8, WOOD_D, f, { p: [0.6, 0.42, -0.4], s: [0.84, 0.12, 0.84], r: [0, 0, Math.PI / 2] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 0.5, 1.4], s: [0.1, 0.1, 1.0], r: [0.25, 0, 0] });
  for (let i = 0; i < 3; i++) {
    part(c.solid, G.ico, pick(c, [0xe0812a, 0xc9524a, 0x5cae4b]), f, {
      p: [rnd(c, -0.3, 0.3), 0.75, rnd(c, -0.6, 0.6)],
      s: 0.3
    });
  }
}

function crate(c: Ctx, f: THREE.Matrix4): void {
  const s = rnd(c, 0.55, 0.8);
  footprint(c, f, 0.45 * s);
  part(c.solid, G.box, WOOD, f, { s: [s, s, s], r: [0, c.rng() * 3, 0] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, s * 0.5, 0], s: [s * 1.04, s * 0.08, s * 1.04], r: [0, c.rng() * 3, 0] });
}

function barrel(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.4);
  part(c.solid, G.cyl8, WOOD, f, { s: [0.66, 0.9, 0.66] });
  part(c.solid, G.cyl8, 0x4a4038, f, { p: [0, 0.22, 0], s: [0.7, 0.08, 0.7] });
  part(c.solid, G.cyl8, 0x4a4038, f, { p: [0, 0.62, 0], s: [0.7, 0.08, 0.7] });
}

function laundryLine(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.25, [-1.6, 0]);
  footprint(c, f, 0.25, [1.6, 0]);
  part(c.solid, G.box, WOOD_D, f, { p: [-1.6, 0, 0], s: [0.1, 1.8, 0.1] });
  part(c.solid, G.box, WOOD_D, f, { p: [1.6, 0, 0], s: [0.1, 1.8, 0.1] });
  part(c.solid, G.box, 0x8a7a5c, f, { p: [0, 1.75, 0], s: [3.2, 0.03, 0.03] });
  for (let i = 0; i < 4; i++) {
    part(c.solid, G.box, pick(c, [0xe8e2d2, 0x8fb8d9, 0xe0a0a8, 0xd9c98a]), f, {
      p: [-1.1 + i * 0.75, 1.4, 0],
      s: [0.45, 0.6, 0.04],
      r: [0, 0, rnd(c, -0.06, 0.06)]
    });
  }
}

function flagPole(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.24);
  part(c.solid, G.cyl6, 0xd9d2c4, f, { s: [0.12, 3.0, 0.12] });
  part(c.solid, G.box, pick(c, [0xc9524a, 0x3d6fc9, 0xf2c14e]), f, { p: [0.42, 2.5, 0], s: [0.8, 0.5, 0.04] });
}

/** Plank jetty running out over the shallows. */
function pier(c: Ctx, f: THREE.Matrix4): void {
  const len = 6;
  for (let i = 0; i < 8; i++) {
    const z = i * (len / 8);
    part(c.solid, G.box, WOOD, f, { p: [0, 0.55, z], s: [1.5, 0.1, 0.62] });
    if (i % 2 === 0) {
      part(c.solid, G.cyl6, WOOD_D, f, { p: [-0.6, -0.6, z], s: [0.16, 1.3, 0.16] });
      part(c.solid, G.cyl6, WOOD_D, f, { p: [0.6, -0.6, z], s: [0.16, 1.3, 0.16] });
      footprint(c, f, 0.3, [0, z]);
    }
  }
  part(c.solid, G.cyl6, WOOD_D, f, { p: [0.75, 0.55, len - 0.4], s: [0.2, 0.8, 0.2] });
}

function beachHut(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 1.0);
  part(c.solid, G.box, 0xe6d7b4, f, { s: [1.9, 1.3, 1.6] });
  part(c.solid, G.box, 0x4fa3b8, f, { p: [0, 0.65, 0], s: [1.94, 0.16, 1.64] });
  part(c.solid, G.cone4, STRAW, f, { p: [0, 1.3, 0], s: [2.8, 1.0, 2.5], r: [0, Math.PI / 4, 0] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 0, 0.81], s: [0.55, 0.95, 0.06] });
  part(c.solid, G.box, 0xfff0b8, f, { p: [0.6, 0.7, 0.81], s: [0.4, 0.4, 0.05] });
  part(c.night, G.box, 0xffd27a, f, { p: [0.6, 0.7, 0.83], s: [0.42, 0.42, 0.06] });
  emit(c, f, 0xffc978, 18, 9, [0, 0.8, 1.1], { flicker: 0.2 });
}

function sandcastle(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.5);
  part(c.solid, G.box, 0xe4c78c, f, { s: [1.2, 0.35, 1.2] });
  for (const [x, z] of [
    [-0.45, -0.45],
    [0.45, -0.45],
    [-0.45, 0.45],
    [0.45, 0.45]
  ]) {
    part(c.solid, G.cyl6, 0xefd79c, f, { p: [x, 0.35, z], s: [0.34, 0.5, 0.34] });
    part(c.solid, G.cone6, 0xe4c78c, f, { p: [x, 0.85, z], s: [0.4, 0.3, 0.4] });
  }
  part(c.solid, G.box, 0xefd79c, f, { p: [0, 0.35, 0], s: [0.5, 0.7, 0.5] });
  part(c.solid, G.box, 0xc9524a, f, { p: [0, 1.1, 0], s: [0.3, 0.2, 0.02] });
}

function netRack(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.5);
  part(c.solid, G.box, WOOD_D, f, { p: [-0.8, 0, 0], s: [0.1, 1.6, 0.1], r: [0, 0, 0.12] });
  part(c.solid, G.box, WOOD_D, f, { p: [0.8, 0, 0], s: [0.1, 1.6, 0.1], r: [0, 0, -0.12] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 1.55, 0], s: [1.8, 0.08, 0.08] });
  part(c.solid, G.box, 0x9aa08c, f, { p: [0, 0.9, 0], s: [1.5, 1.1, 0.05], r: [0, 0, 0.05] });
  part(c.solid, G.sphere, 0xd9c98a, f, { p: [0.5, 0.4, 0.05], s: 0.18 });
}

function buoy(c: Ctx, f: THREE.Matrix4): void {
  part(c.solid, G.cone6, 0xd94f3a, f, { p: [0, 0.2, 0], s: [0.5, 0.7, 0.5] });
  part(c.solid, G.sphere, 0xf1ebdc, f, { p: [0, 0.9, 0], s: 0.24 });
}

function driftwood(c: Ctx, f: THREE.Matrix4): void {
  part(c.solid, G.cyl6, 0xb8a88a, f, { s: [0.3, 1.8, 0.3], r: [Math.PI / 2, c.rng() * 3, 0] });
}

function shell(c: Ctx, f: THREE.Matrix4): void {
  const col = pick(c, [0xf2e2d0, 0xefc9c2, 0xe8d4a8]);
  part(c.solid, G.cone8, col, f, { p: [0, 0.04, 0], s: [rnd(c, 0.24, 0.4), rnd(c, 0.2, 0.35), rnd(c, 0.24, 0.4)], r: [1.6, c.rng() * 3, 0] });
}

function starfish(c: Ctx, f: THREE.Matrix4): void {
  const col = pick(c, [0xe08a4a, 0xd9647a]);
  for (let i = 0; i < 5; i++) {
    const a = (i / 5) * Math.PI * 2;
    part(c.solid, G.box, col, f, { p: [Math.cos(a) * 0.2, 0.04, Math.sin(a) * 0.2], s: [0.14, 0.07, 0.4], r: [0, -a, 0] });
  }
}

function snowman(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.55);
  part(c.solid, G.sphere, SNOW, f, { p: [0, 0.45, 0], s: 1.0 });
  part(c.solid, G.sphere, SNOW, f, { p: [0, 1.15, 0], s: 0.72 });
  part(c.solid, G.sphere, SNOW, f, { p: [0, 1.7, 0], s: 0.54 });
  part(c.solid, G.box, 0x2a2a30, f, { p: [-0.1, 1.78, 0.24], s: [0.06, 0.06, 0.04] });
  part(c.solid, G.box, 0x2a2a30, f, { p: [0.1, 1.78, 0.24], s: [0.06, 0.06, 0.04] });
  part(c.solid, G.cone6, 0xe08a3a, f, { p: [0, 1.68, 0.3], s: [0.12, 0.34, 0.12], r: [1.5, 0, 0] });
  part(c.solid, G.cyl8, 0x2f3d4a, f, { p: [0, 1.95, 0], s: [0.5, 0.08, 0.5] });
  part(c.solid, G.cyl8, 0x2f3d4a, f, { p: [0, 1.98, 0], s: [0.34, 0.3, 0.34] });
  part(c.solid, G.box, WOOD_D, f, { p: [-0.42, 1.2, 0], s: [0.7, 0.06, 0.06], r: [0, 0, 0.5] });
  part(c.solid, G.box, WOOD_D, f, { p: [0.42, 1.2, 0], s: [0.7, 0.06, 0.06], r: [0, 0, -0.5] });
  part(c.solid, G.box, 0xc9524a, f, { p: [0, 1.44, 0.02], s: [0.5, 0.16, 0.42] });
}

/** Hole cut in the ice, with a stool and a rod beside it. */
function fishingHole(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.45, [0.8, 0]);
  part(c.solid, G.cyl8, 0x2d5e73, f, { s: [1.1, 0.06, 1.1] });
  part(c.solid, G.cyl8, ICE_B, f, { s: [1.35, 0.05, 1.35] });
  part(c.solid, G.cyl6, WOOD_D, f, { p: [0.9, 0, 0], s: [0.5, 0.45, 0.5] });
  part(c.solid, G.box, WOOD, f, { p: [0.5, 0.7, 0], s: [1.3, 0.06, 0.06], r: [0, 0, -0.5] });
}

/** Hut on stilts at the water's edge, with a lit window. */
function stiltHut(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 1.2);
  for (const [x, z] of [
    [-0.8, -0.7],
    [0.8, -0.7],
    [-0.8, 0.7],
    [0.8, 0.7]
  ]) {
    part(c.solid, G.cyl6, WOOD_D, f, { p: [x, 0, z], s: [0.2, 1.2, 0.2] });
  }
  part(c.solid, G.box, WOOD, f, { p: [0, 1.2, 0], s: [2.2, 0.14, 2.0] });
  part(c.solid, G.box, 0x8a7550, f, { p: [0, 1.34, 0], s: [1.9, 1.2, 1.7] });
  part(c.solid, G.cone4, STRAW, f, { p: [0, 2.54, 0], s: [2.9, 1.0, 2.7], r: [0, Math.PI / 4, 0] });
  part(c.solid, G.box, WOOD_D, f, { p: [0, 1.34, 0.86], s: [0.5, 0.9, 0.06] });
  part(c.solid, G.box, 0xfff0b8, f, { p: [0.6, 1.9, 0.86], s: [0.4, 0.4, 0.05] });
  part(c.night, G.box, 0xffd27a, f, { p: [0.6, 1.9, 0.88], s: [0.42, 0.42, 0.06] });
  emit(c, f, 0xffbe74, 24, 11, [0, 2.0, 1.2], { flicker: 0.25 });
  // Ladder down to the ground.
  part(c.solid, G.box, WOOD_D, f, { p: [-0.2, 0.6, 1.2], s: [0.06, 1.5, 0.06], r: [0.3, 0, 0] });
  part(c.solid, G.box, WOOD_D, f, { p: [0.2, 0.6, 1.2], s: [0.06, 1.5, 0.06], r: [0.3, 0, 0] });
}

/** Two logs and a handrail over a dip. */
function logBridge(c: Ctx, f: THREE.Matrix4): void {
  part(c.solid, G.cyl8, WOOD_D, f, { p: [-0.28, 0.3, 0], s: [0.42, 3.6, 0.42], r: [Math.PI / 2, 0, 0] });
  part(c.solid, G.cyl8, WOOD_D, f, { p: [0.28, 0.3, 0], s: [0.42, 3.6, 0.42], r: [Math.PI / 2, 0, 0] });
  for (const z of [-1.6, 1.6]) {
    part(c.solid, G.box, WOOD, f, { p: [0.62, 0.5, z], s: [0.1, 1.0, 0.1] });
    footprint(c, f, 0.24, [0.62, z]);
  }
  part(c.solid, G.box, WOOD, f, { p: [0.62, 1.0, 0], s: [0.07, 0.07, 3.2] });
}

/** Desert camp: two tents around a lantern. */
function caravan(c: Ctx, f: THREE.Matrix4): void {
  tent(c, new THREE.Matrix4().makeTranslation(-1.6, 0, 0.4).premultiply(f));
  tent(c, new THREE.Matrix4().makeTranslation(1.7, 0, -0.5).premultiply(f));
  crate(c, new THREE.Matrix4().makeTranslation(0.2, 0, 1.5).premultiply(f));
  lanternPost(c, new THREE.Matrix4().makeTranslation(0, 0, 0).premultiply(f), 0xffcf8a);
}

/** Stone circle: a ring of standing stones with an altar in the middle. */
function stoneCircle(c: Ctx, f: THREE.Matrix4): void {
  for (let i = 0; i < 8; i++) {
    const a = (i / 8) * Math.PI * 2;
    const m = new THREE.Matrix4().makeTranslation(Math.cos(a) * 4.2, 0, Math.sin(a) * 4.2).premultiply(f);
    standingStone(c, m, 1.1);
  }
  footprint(c, f, 0.9);
  part(c.solid, G.cyl6, 0x7d838b, f, { s: [2.2, 0.5, 2.2] });
  part(c.glow, G.dodeca, 0xb6a5ff, f, { p: [0, 0.6, 0], s: [0.7, 0.6, 0.7] });
  emit(c, f, 0xb6a5ff, 26, 14, [0, 0.9, 0], { flicker: 0.2 });
}

/** Squat stone lantern, the kind that lines a temple path. */
function stoneLantern(c: Ctx, f: THREE.Matrix4): void {
  footprint(c, f, 0.35);
  part(c.solid, G.cyl6, 0x8a8f96, f, { s: [0.6, 0.5, 0.6] });
  part(c.solid, G.cyl6, 0x7a8087, f, { p: [0, 0.5, 0], s: [0.3, 0.5, 0.3] });
  part(c.glow, G.box, 0xffd9a0, f, { p: [0, 1.0, 0], s: [0.42, 0.4, 0.42] });
  part(c.solid, G.cone6, 0x6f757d, f, { p: [0, 1.4, 0], s: [0.9, 0.35, 0.9] });
  emit(c, f, 0xffcf94, 14, 8, [0, 1.1, 0], { flicker: 0.25 });
}

// ---------------------------------------------------------------- scatter table

type LandBiomeId = Exclude<BiomeId, 'ocean'>;

/** One entry in a biome's scatter table: how often, and what to build. */
type Scatter = { p: number; build: (c: Ctx, f: THREE.Matrix4) => void };

/**
 * Directions sampled over the sphere before filtering. Land, density, and the
 * per-biome probability that a sample builds anything at all cut this down to
 * roughly a quarter of it; raising this number is the way to make the whole
 * planet busier.
 */
const SCATTER_SAMPLES = 13000;

/**
 * Local density multiplier in roughly [0.25, 1.2]. Scenery gathers into
 * thickets and leaves clearings between them, which reads far more like a place
 * than an even sprinkle at the same average count does.
 */
function clumpAt(d: THREE.Vector3): number {
  const n = fbm3(d.x * 5.5 + 31, d.y * 5.5 - 13, d.z * 5.5 + 7, 2);
  return Math.max(0.22, Math.min(1.2, 0.72 + n * 0.75));
}

/**
 * What grows where. Probabilities are absolute, not cumulative: a sample that
 * falls past the end of its biome's table leaves bare ground, so each table
 * sums to well under one.
 */
const SCATTER: Record<LandBiomeId, Scatter[]> = {
  forest: [
    { p: 0.15, build: (c, f) => pine(c, f) },
    { p: 0.05, build: (c, f) => pine(c, f, 1.5) },
    { p: 0.06, build: (c, f) => roundTree(c, f) },
    { p: 0.05, build: birch },
    { p: 0.08, build: fern },
    { p: 0.13, build: (c, f) => grass(c, f, 0x5fb44e) },
    { p: 0.05, build: flower },
    { p: 0.04, build: bush },
    { p: 0.03, build: berryBush },
    { p: 0.035, build: (c, f) => mushroom(c, f, c.rng() < 0.2) },
    { p: 0.012, build: mushroomRing },
    { p: 0.018, build: stump },
    { p: 0.012, build: fallenLog },
    { p: 0.025, build: (c, f) => rock(c, f) },
    { p: 0.01, build: (c, f) => boulder(c, f) },
    { p: 0.004, build: (c, f) => lanternPost(c, f) }
  ],
  farm: [
    { p: 0.2, build: (c, f) => grass(c, f, 0x8fdb60) },
    { p: 0.12, build: flower },
    { p: 0.03, build: (c, f) => roundTree(c, f) },
    { p: 0.025, build: appleTree },
    { p: 0.03, build: bush },
    { p: 0.012, build: hedge },
    { p: 0.012, build: hayBale },
    { p: 0.03, build: pumpkin },
    { p: 0.03, build: sunflower },
    { p: 0.008, build: cropRow },
    { p: 0.006, build: (c, f) => fence(c, f, rnd(c, 3, 6)) },
    { p: 0.006, build: beehive },
    { p: 0.006, build: birdhouse },
    { p: 0.004, build: cart },
    { p: 0.003, build: laundryLine },
    { p: 0.004, build: scarecrow },
    { p: 0.005, build: (c, f) => lanternPost(c, f) }
  ],
  shore: [
    { p: 0.07, build: palm },
    { p: 0.1, build: (c, f) => grass(c, f, 0xd9c37a) },
    { p: 0.05, build: (c, f) => rock(c, f, 0.8, 0xc9b07a) },
    { p: 0.06, build: shell },
    { p: 0.02, build: starfish },
    { p: 0.03, build: driftwood },
    { p: 0.006, build: sandcastle },
    { p: 0.004, build: beachHut },
    { p: 0.006, build: netRack },
    { p: 0.01, build: crate },
    { p: 0.008, build: barrel },
    { p: 0.008, build: buoy },
    { p: 0.02, build: cactus },
    { p: 0.012, build: bush },
    { p: 0.006, build: torch }
  ],
  ember: [
    { p: 0.16, build: lavaRock },
    { p: 0.05, build: deadTree },
    { p: 0.03, build: basaltColumn },
    { p: 0.03, build: (c, f) => boulder(c, f, 0x4a3f3a, false) },
    { p: 0.06, build: (c, f) => grass(c, f, 0x8a6a3a) },
    { p: 0.012, build: steamVent },
    { p: 0.012, build: (c, f) => crystalCluster(c, f, 0xff8a5c) },
    { p: 0.03, build: deadBush },
    { p: 0.04, build: (c, f) => rock(c, f, 0.9, 0x4a3f3a) }
  ],
  arctic: [
    { p: 0.11, build: snowPine },
    { p: 0.11, build: (c, f) => iceShard(c, f) },
    { p: 0.15, build: snowPile },
    { p: 0.05, build: (c, f) => rock(c, f, 0.7, 0xb8c4ca) },
    { p: 0.06, build: icicles },
    { p: 0.02, build: (c, f) => boulder(c, f, 0xc4d2d8, false) },
    { p: 0.015, build: (c, f) => crystalCluster(c, f, 0x9be7ff) },
    { p: 0.005, build: snowman },
    { p: 0.005, build: fishingHole },
    { p: 0.005, build: (c, f) => lanternPost(c, f, 0xbfe4ff) }
  ],
  desert: [
    { p: 0.05, build: cactus },
    { p: 0.04, build: barrelCactus },
    { p: 0.03, build: agave },
    { p: 0.03, build: tumbleweed },
    { p: 0.05, build: deadBush },
    { p: 0.06, build: (c, f) => rock(c, f, 0.9, 0xc9a06a) },
    { p: 0.02, build: (c, f) => boulder(c, f, 0xb06a3a, false) },
    { p: 0.05, build: (c, f) => grass(c, f, 0xc9a45c) },
    { p: 0.006, build: boneArch },
    { p: 0.003, build: (c, f) => mesa(c, f, rnd(c, 1.2, 2.0), rnd(c, 2.4, 4.0)) },
    { p: 0.008, build: palm },
    { p: 0.005, build: torch }
  ],
  marsh: [
    { p: 0.14, build: reeds },
    { p: 0.08, build: cattail },
    { p: 0.03, build: (c, f) => willow(c, f) },
    { p: 0.05, build: lilyPad },
    { p: 0.05, build: (c, f) => mushroom(c, f) },
    { p: 0.02, build: (c, f) => mushroom(c, f, true) },
    { p: 0.06, build: fern },
    { p: 0.03, build: bush },
    { p: 0.06, build: (c, f) => grass(c, f, 0x6fae5c) },
    { p: 0.015, build: fallenLog },
    { p: 0.02, build: stump },
    { p: 0.015, build: (c, f) => boulder(c, f) },
    { p: 0.004, build: bigMushroom },
    { p: 0.004, build: (c, f) => lanternPost(c, f, 0xd9f0b8) }
  ],
  grove: [
    { p: 0.13, build: (c, f) => maple(c, f) },
    { p: 0.05, build: birch },
    { p: 0.03, build: (c, f) => roundTree(c, f) },
    { p: 0.08, build: leafPile },
    { p: 0.05, build: fern },
    { p: 0.1, build: (c, f) => grass(c, f, 0x9cae4a) },
    { p: 0.04, build: (c, f) => mushroom(c, f, c.rng() < 0.15) },
    { p: 0.03, build: bush },
    { p: 0.04, build: flower },
    { p: 0.012, build: (c, f) => standingStone(c, f) },
    { p: 0.02, build: (c, f) => boulder(c, f) },
    { p: 0.02, build: stump },
    { p: 0.015, build: fallenLog },
    { p: 0.004, build: bench },
    { p: 0.008, build: stoneLantern }
  ]
};

/**
 * A dry spot on the coast to run a jetty out from, and the yaw that points it
 * at the water. Scans headings out of the biome centre for the first waterline
 * and steps back onto land. Null when the biome has no reachable coast.
 */
function pierSite(center: THREE.Vector3): { dir: THREE.Vector3; yaw: number } | null {
  const probe = new THREE.Vector3();
  for (let k = 0; k < 24; k++) {
    const a = (k / 24) * Math.PI * 2;
    for (let step = 6; step < 22; step++) {
      probe.copy(center);
      offsetDir(probe, Math.cos(a) * step, Math.sin(a) * step);
      if (oceanField(probe) >= 0.0) continue;
      const shore = center.clone();
      offsetDir(shore, Math.cos(a) * (step - 2.5), Math.sin(a) * (step - 2.5));
      if (isOcean(shore)) break;
      // Local +Z runs along the jetty, so yaw is the heading measured from north.
      return { dir: shore, yaw: Math.atan2(Math.cos(a), Math.sin(a)) };
    }
  }
  return null;
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
  bench(ctx, landAt(ctx, 'forest', -1.6, -2.2, 0.3));
  exclude('forest', 2.6, -1.4, 1.6);
  exclude('forest', -2.2, 0.4, 1.4);
  exclude('forest', -1.6, -2.2, 1.2);
  cabin(ctx, wf('work'));
  exclusions.push({ dir: wonderDirs.get('work')!, radius: 3.0 });
  for (const [e, n] of [
    [-4.2, 2.4],
    [-8.4, 5.6],
    [1.2, 3.6]
  ]) {
    lanternPost(ctx, landAt(ctx, 'forest', e, n, 0));
    exclude('forest', e, n, 1.0);
  }
  mushroomRing(ctx, landAt(ctx, 'forest', 5.5, 2.0, 0));
  exclude('forest', 5.5, 2.0, 1.8);
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
  well(ctx, landAt(ctx, 'farm', 2.2, 3.4, 0));
  exclude('farm', 2.2, 3.4, 1.6);
  cart(ctx, landAt(ctx, 'farm', -1.2, 4.4, 0.8));
  exclude('farm', -1.2, 4.4, 1.6);
  laundryLine(ctx, landAt(ctx, 'farm', -2.4, 2.4, -0.4));
  exclude('farm', -2.4, 2.4, 2.2);
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
  for (const [e, n] of [
    [5.4, 4.6],
    [-5.2, 4.0]
  ]) {
    lanternPost(ctx, landAt(ctx, 'farm', e, n, 0));
    exclude('farm', e, n, 1.0);
  }
  mailbox(ctx, wf('contact'));
  signpost(ctx, wf('resume'));

  // --- Sunmark Shore: lighthouse on the dunes, a jetty, and a fishing camp.
  const lamp = lighthouse(ctx, wf('blog'));
  exclusions.push({ dir: wonderDirs.get('blog')!, radius: 3.2 });
  beachHut(ctx, landAt(ctx, 'shore', -4.5, -2.0, 0.4));
  exclude('shore', -4.5, -2.0, 2.4);
  netRack(ctx, landAt(ctx, 'shore', -2.8, -3.2, 1.1));
  exclude('shore', -2.8, -3.2, 1.2);
  sandcastle(ctx, landAt(ctx, 'shore', 3.2, -2.6, 0));
  exclude('shore', 3.2, -2.6, 1.0);
  // A jetty only makes sense if it reaches water, so aim it downhill to the sea.
  const jetty = pierSite(biomeById('shore').center);
  if (jetty) {
    pier(ctx, surfaceFrame(jetty.dir, jetty.yaw, new THREE.Matrix4()));
    exclusions.push({ dir: jetty.dir.clone(), radius: 3.0 });
  }

  // --- Ember Heights: the volcano, the forge, and a field of vents.
  volcano(ctx, landAt(ctx, 'ember', -1.5, 3.5, 0));
  exclude('ember', -1.5, 3.5, 6.5);
  forge(ctx, wf('projects'));
  exclusions.push({ dir: wonderDirs.get('projects')!, radius: 3.2 });
  for (const [e, n] of [
    [4.5, 1.5],
    [-5.5, -2.5],
    [2.0, -5.0]
  ]) {
    steamVent(ctx, landAt(ctx, 'ember', e, n, 0));
    exclude('ember', e, n, 1.4);
  }
  for (const [e, n] of [
    [6.5, -3.0],
    [-7.0, 2.0]
  ]) {
    basaltColumn(ctx, landAt(ctx, 'ember', e, n, ctx.rng() * 3));
    exclude('ember', e, n, 1.6);
  }

  // --- Northlight: the crystal, an igloo, and a ring of big shards.
  auroraCrystal(ctx, wf('education'));
  igloo(ctx, landAt(ctx, 'arctic', 4.5, 3.2, -0.9));
  exclude('arctic', 4.5, 3.2, 2.6);
  snowman(ctx, landAt(ctx, 'arctic', 2.4, 1.4, 0.4));
  exclude('arctic', 2.4, 1.4, 1.2);
  fishingHole(ctx, landAt(ctx, 'arctic', -2.6, -1.6, 0.8));
  exclude('arctic', -2.6, -1.6, 1.8);
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

  // --- Dune Reach: a butte on the skyline, a wind-cut arch, and a camp.
  mesa(ctx, landAt(ctx, 'desert', 1.0, 8.5, 0.4), 3.4, 6.0);
  exclude('desert', 1.0, 8.5, 5.0);
  mesa(ctx, landAt(ctx, 'desert', -6.5, 6.0, 1.2), 2.2, 4.0);
  exclude('desert', -6.5, 6.0, 3.4);
  rockArch(ctx, landAt(ctx, 'desert', -5.0, -2.0, 0.7));
  exclude('desert', -5.0, -2.0, 4.0);
  caravan(ctx, landAt(ctx, 'desert', 3.0, -3.0, -0.4));
  exclude('desert', 3.0, -3.0, 3.4);
  boneArch(ctx, landAt(ctx, 'desert', 5.5, 2.5, 1.9));
  exclude('desert', 5.5, 2.5, 2.0);
  for (const [e, n] of [
    [0.5, 0.5],
    [-2.0, 3.5]
  ]) {
    well(ctx, landAt(ctx, 'desert', e, n, 0));
    exclude('desert', e, n, 1.8);
    for (let i = 0; i < 4; i++) {
      palm(ctx, landAt(ctx, 'desert', e + Math.cos(i * 1.7) * 2.2, n + Math.sin(i * 1.7) * 2.2, 0));
    }
  }

  // --- Willowmere: a hut on stilts, a bridge, and glowing caps.
  stiltHut(ctx, landAt(ctx, 'marsh', 2.5, 3.0, -0.5));
  exclude('marsh', 2.5, 3.0, 3.0);
  logBridge(ctx, landAt(ctx, 'marsh', -1.5, -1.0, 0.9));
  exclude('marsh', -1.5, -1.0, 2.6);
  willow(ctx, landAt(ctx, 'marsh', 0.5, 6.0, 0), 1.8);
  exclude('marsh', 0.5, 6.0, 3.4);
  for (const [e, n] of [
    [-4.5, 2.5],
    [4.0, -3.5],
    [-2.0, -5.0]
  ]) {
    bigMushroom(ctx, landAt(ctx, 'marsh', e, n, 0));
    exclude('marsh', e, n, 2.2);
  }
  for (const [e, n] of [
    [-3.0, 0.5],
    [3.5, 1.0]
  ]) {
    lanternPost(ctx, landAt(ctx, 'marsh', e, n, 0), 0xd9f0b8);
    exclude('marsh', e, n, 1.0);
  }

  // --- Amberfall: the stone circle, an old maple, and a place to sit.
  stoneCircle(ctx, landAt(ctx, 'grove', 0.5, 7.0, 0.2));
  exclude('grove', 0.5, 7.0, 6.0);
  maple(ctx, landAt(ctx, 'grove', -3.5, -2.5, 0), 2.4);
  exclude('grove', -3.5, -2.5, 3.2);
  bench(ctx, landAt(ctx, 'grove', -2.0, -4.0, 1.2));
  exclude('grove', -2.0, -4.0, 1.2);
  for (const [e, n] of [
    [3.0, 1.5],
    [-4.5, 3.0],
    [1.5, -4.5],
    [5.5, -1.5]
  ]) {
    stoneLantern(ctx, landAt(ctx, 'grove', e, n, 0));
    exclude('grove', e, n, 1.0);
  }
  for (const [e, n] of [
    [6.0, 4.0],
    [-7.0, -1.0]
  ]) {
    boulder(ctx, landAt(ctx, 'grove', e, n, ctx.rng() * 3), ROCK, true);
    exclude('grove', e, n, 2.2);
  }

  // --- Path stones linking the wonders in a loop, lit every so often.
  const loop = ['about', 'work', 'resume', 'contact', 'blog', 'projects', 'education', 'about'];
  const pa = new THREE.Vector3();
  const pb = new THREE.Vector3();
  const pd = new THREE.Vector3();
  const pf = new THREE.Matrix4();
  let laid = 0;
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
      // A lamp every dozen stones, set back off the path so it lights the way
      // without standing in it.
      if (++laid % 12 === 0) {
        const side = pd.clone();
        offsetDir(side, 1.5, 0);
        if (!isOcean(side)) {
          lanternPost(ctx, surfaceFrame(side, ctx.rng() * 3, pf));
          exclusions.push({ dir: side.clone(), radius: 1.0 });
        }
      }
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
  for (let i = 0; i < SCATTER_SAMPLES; i++) {
    d.set(ctx.rng() * 2 - 1, ctx.rng() * 2 - 1, ctx.rng() * 2 - 1);
    if (d.lengthSq() < 0.05) continue;
    d.normalize();
    if (oceanField(d) < 0.07) continue;
    // Density varies over a low-frequency field, so scenery gathers into
    // thickets with clearings between them instead of spreading evenly.
    if (ctx.rng() > clumpAt(d)) continue;
    if (blocked(d)) continue;
    const table = SCATTER[landBiomeAt(d).id as LandBiomeId];
    let r = ctx.rng();
    surfaceFrame(d, ctx.rng() * Math.PI * 2, f);
    for (const entry of table) {
      r -= entry.p;
      if (r < 0) {
        entry.build(ctx, f);
        break;
      }
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
