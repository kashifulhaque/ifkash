// Light particle weather that follows the player: rain streaks, drifting snow,
// and rising embers. Each kind is one draw call. Particles live in the player's
// tangent frame (local +Y is the surface normal), and the whole emitter is
// re-oriented every frame with the player's direction, so the weather comes
// along without any per-particle re-projection.

import * as THREE from 'three';
import type { BiomeId } from './biomes';
import { PLANET_RADIUS, tangentBasis, walkRadius } from './planet';

export type WeatherKind = 'rain' | 'snow' | 'embers' | 'leaves' | null;

/** Weather for each biome; biomes not listed have clear air. */
export function weatherFor(biome: BiomeId): WeatherKind {
  switch (biome) {
    case 'forest':
    case 'marsh':
      return 'rain';
    case 'arctic':
      return 'snow';
    case 'ember':
      return 'embers';
    case 'grove':
      return 'leaves';
    default:
      return null;
  }
}

/** Horizontal radius around the player that particles occupy, in surface units. */
const RANGE = 12;
/** Seconds for a layer to fade fully in or out. */
const FADE = 2;

const RAIN_COUNT = 900;
const SNOW_COUNT = 1400;
const EMBER_COUNT = 500;
const LEAF_COUNT = 420;
const SPARK_COUNT = 140;
/** Amberfall's falling leaves, picked per particle. */
const LEAF_COLORS = [0xd9762f, 0xe0a12f, 0xc44a2c, 0xb8862f, 0xe4c25a];

const _right = new THREE.Vector3();
const _fwd = new THREE.Vector3();
const _m = new THREE.Matrix4();

/** Height of the ground below a local (x, z) offset, approximating the sphere's curvature. */
function groundAt(x: number, z: number): number {
  return -(x * x + z * z) / (2 * PLANET_RADIUS) - 0.6;
}

/** One particle system with its own fade. */
abstract class Layer {
  abstract object: THREE.Object3D;
  protected abstract material: THREE.Material & { opacity: number };
  protected abstract maxOpacity: number;
  /** Normalised fade level in [0, 1]. */
  level = 0;
  target = 0;

  fade(dt: number): boolean {
    const step = dt / FADE;
    if (this.level < this.target) this.level = Math.min(this.target, this.level + step);
    else if (this.level > this.target) this.level = Math.max(this.target, this.level - step);
    const on = this.level > 0.005;
    this.object.visible = on;
    if (on) this.material.opacity = this.level * this.maxOpacity;
    return on;
  }

  abstract simulate(dt: number, t: number): void;
}

class Rain extends Layer {
  object: THREE.LineSegments;
  protected material: THREE.LineBasicMaterial;
  protected maxOpacity = 0.32;
  private pos: Float32Array;
  private speed = new Float32Array(RAIN_COUNT);

  constructor() {
    super();
    this.pos = new Float32Array(RAIN_COUNT * 6);
    for (let i = 0; i < RAIN_COUNT; i++) this.spawn(i, Math.random() * 16);
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(this.pos, 3));
    this.material = new THREE.LineBasicMaterial({ color: 0xcfe3f2, transparent: true, opacity: 0, depthWrite: false });
    this.object = new THREE.LineSegments(geo, this.material);
    this.object.frustumCulled = false;
    this.object.visible = false;
  }

  private spawn(i: number, y: number): void {
    const a = Math.random() * Math.PI * 2;
    const r = Math.sqrt(Math.random()) * RANGE;
    const o = i * 6;
    this.pos[o] = Math.cos(a) * r;
    this.pos[o + 1] = y;
    this.pos[o + 2] = Math.sin(a) * r;
    this.speed[i] = 18 + Math.random() * 8;
  }

  simulate(dt: number): void {
    const p = this.pos;
    const slant = 0.18;
    for (let i = 0; i < RAIN_COUNT; i++) {
      const o = i * 6;
      p[o + 1] -= this.speed[i] * dt;
      p[o] += this.speed[i] * slant * dt;
      if (p[o + 1] < groundAt(p[o], p[o + 2]) || p[o] > RANGE) this.spawn(i, 14 + Math.random() * 3);
      // Tail: a short streak trailing the drop's motion.
      const len = 0.55;
      p[o + 3] = p[o] - slant * len;
      p[o + 4] = p[o + 1] + len;
      p[o + 5] = p[o + 2];
    }
    (this.object.geometry.attributes.position as THREE.BufferAttribute).needsUpdate = true;
  }
}

class Flakes extends Layer {
  object: THREE.Points;
  protected material: THREE.PointsMaterial;
  protected maxOpacity: number;
  private pos: Float32Array;
  private phase: Float32Array;
  private speed: Float32Array;
  private count: number;
  private rising: boolean;
  private fall: number;
  private sway: number;

  constructor(count: number, color: number, size: number, opacity: number, rising: boolean, additive: boolean, o: FlakeOptions = {}) {
    super();
    this.count = count;
    this.rising = rising;
    this.maxOpacity = opacity;
    this.fall = o.fall ?? 1;
    this.sway = o.sway ?? 1;
    this.pos = new Float32Array(count * 3);
    this.phase = new Float32Array(count);
    this.speed = new Float32Array(count);
    for (let i = 0; i < count; i++) this.spawn(i, true);
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(this.pos, 3));
    // A palette turns one draw call into a mixed drift: every particle keeps its
    // own colour instead of the whole layer sharing the material's.
    if (o.palette) {
      const tint = new THREE.Color();
      const arr = new Float32Array(count * 3);
      for (let i = 0; i < count; i++) {
        tint.setHex(o.palette[Math.floor(Math.random() * o.palette.length)]);
        tint.multiplyScalar(0.75 + Math.random() * 0.45);
        arr[i * 3] = tint.r;
        arr[i * 3 + 1] = tint.g;
        arr[i * 3 + 2] = tint.b;
      }
      geo.setAttribute('color', new THREE.BufferAttribute(arr, 3));
    }
    this.material = new THREE.PointsMaterial({
      color,
      size,
      map: o.sprite === 'leaf' ? leafSprite() : softDot(),
      vertexColors: !!o.palette,
      transparent: true,
      opacity: 0,
      depthWrite: false,
      blending: additive ? THREE.AdditiveBlending : THREE.NormalBlending
    });
    this.object = new THREE.Points(geo, this.material);
    this.object.frustumCulled = false;
    this.object.visible = false;
  }

  private spawn(i: number, scatter: boolean): void {
    const a = Math.random() * Math.PI * 2;
    const r = Math.sqrt(Math.random()) * (this.rising ? RANGE * 0.75 : RANGE);
    const o = i * 3;
    this.pos[o] = Math.cos(a) * r;
    this.pos[o + 2] = Math.sin(a) * r;
    if (this.rising) this.pos[o + 1] = scatter ? Math.random() * 8 : groundAt(this.pos[o], this.pos[o + 2]) + 0.4 + Math.random() * 1.2;
    else this.pos[o + 1] = scatter ? Math.random() * 14 : 12 + Math.random() * 3;
    this.phase[i] = Math.random() * Math.PI * 2;
    this.speed[i] = (this.rising ? 1.2 + Math.random() * 1.6 : 1.4 + Math.random() * 1.2) * this.fall;
  }

  simulate(dt: number, t: number): void {
    const p = this.pos;
    for (let i = 0; i < this.count; i++) {
      const o = i * 3;
      const ph = this.phase[i];
      const sway = (this.rising ? 0.9 : 0.6) * this.sway;
      p[o] += Math.sin(t * 0.9 + ph) * sway * dt;
      p[o + 2] += Math.cos(t * 0.7 + ph * 1.3) * sway * dt;
      if (this.rising) {
        p[o + 1] += this.speed[i] * dt;
        if (p[o + 1] > 8 + Math.sin(ph) * 2) this.spawn(i, false);
      } else {
        p[o + 1] -= this.speed[i] * dt;
        if (p[o + 1] < groundAt(p[o], p[o + 2])) this.spawn(i, false);
      }
    }
    (this.object.geometry.attributes.position as THREE.BufferAttribute).needsUpdate = true;
  }
}

/**
 * Fireflies. Not weather — they hang over the green biomes whatever the sky is
 * doing — but they live here because they want exactly what the weather layers
 * have: a pool of particles carried along in the player's tangent frame for one
 * draw call. Each one pulses on its own through the vertex colours.
 */
class Sparks extends Layer {
  object: THREE.Points;
  protected material: THREE.PointsMaterial;
  protected maxOpacity = 1;
  private pos = new Float32Array(SPARK_COUNT * 3);
  private col = new Float32Array(SPARK_COUNT * 3);
  private phase = new Float32Array(SPARK_COUNT);
  private drift = new Float32Array(SPARK_COUNT * 3);

  constructor() {
    super();
    for (let i = 0; i < SPARK_COUNT; i++) this.spawn(i);
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(this.pos, 3));
    geo.setAttribute('color', new THREE.BufferAttribute(this.col, 3));
    this.material = new THREE.PointsMaterial({
      size: 0.34,
      map: softDot(),
      vertexColors: true,
      transparent: true,
      opacity: 0,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
      toneMapped: false
    });
    this.object = new THREE.Points(geo, this.material);
    this.object.frustumCulled = false;
    this.object.visible = false;
  }

  private spawn(i: number): void {
    const a = Math.random() * Math.PI * 2;
    const r = Math.sqrt(Math.random()) * RANGE * 0.8;
    const o = i * 3;
    this.pos[o] = Math.cos(a) * r;
    this.pos[o + 2] = Math.sin(a) * r;
    this.pos[o + 1] = groundAt(this.pos[o], this.pos[o + 2]) + 0.4 + Math.random() * 2.6;
    this.phase[i] = Math.random() * Math.PI * 2;
    this.drift[o] = (Math.random() - 0.5) * 0.7;
    this.drift[o + 1] = (Math.random() - 0.5) * 0.3;
    this.drift[o + 2] = (Math.random() - 0.5) * 0.7;
  }

  simulate(dt: number, t: number): void {
    const p = this.pos;
    for (let i = 0; i < SPARK_COUNT; i++) {
      const o = i * 3;
      const ph = this.phase[i];
      p[o] += (this.drift[o] + Math.sin(t * 0.8 + ph) * 0.35) * dt;
      p[o + 1] += (this.drift[o + 1] + Math.sin(t * 1.3 + ph * 1.7) * 0.25) * dt;
      p[o + 2] += (this.drift[o + 2] + Math.cos(t * 0.7 + ph * 1.3) * 0.35) * dt;
      const floor = groundAt(p[o], p[o + 2]);
      if (p[o + 1] < floor + 0.3 || p[o + 1] > floor + 3.6 || p[o] * p[o] + p[o + 2] * p[o + 2] > RANGE * RANGE) this.spawn(i);
      // Slow uneven blink, never quite dark.
      const glow = 0.2 + 0.8 * Math.pow(Math.max(0, Math.sin(t * 1.6 + ph)), 3);
      this.col[o] = 0.85 * glow;
      this.col[o + 1] = 1.0 * glow;
      this.col[o + 2] = 0.45 * glow;
    }
    (this.object.geometry.attributes.position as THREE.BufferAttribute).needsUpdate = true;
    (this.object.geometry.attributes.color as THREE.BufferAttribute).needsUpdate = true;
  }
}

type FlakeOptions = {
  /** Sprite shape. Leaves get a lopsided blade rather than a soft dot. */
  sprite?: 'dot' | 'leaf';
  /** Per-particle colours; the layer draws mixed hues in one call. */
  palette?: number[];
  /** Multiplier on the fall or rise speed. */
  fall?: number;
  /** Multiplier on the horizontal drift. */
  sway?: number;
};

let dotTexture: THREE.Texture | null = null;
let leafTexture: THREE.Texture | null = null;

/** A soft round sprite, drawn once on a canvas so points read as flakes rather than squares. */
export function softDot(): THREE.Texture {
  if (dotTexture) return dotTexture;
  const size = 32;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const g = canvas.getContext('2d')!;
  const grad = g.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2);
  grad.addColorStop(0, 'rgba(255,255,255,1)');
  grad.addColorStop(0.55, 'rgba(255,255,255,0.6)');
  grad.addColorStop(1, 'rgba(255,255,255,0)');
  g.fillStyle = grad;
  g.fillRect(0, 0, size, size);
  dotTexture = new THREE.CanvasTexture(canvas);
  return dotTexture;
}

/** A lopsided leaf blade, so falling leaves do not read as round flakes. */
function leafSprite(): THREE.Texture {
  if (leafTexture) return leafTexture;
  const size = 32;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const g = canvas.getContext('2d')!;
  g.fillStyle = '#ffffff';
  g.beginPath();
  g.moveTo(size * 0.5, size * 0.08);
  g.bezierCurveTo(size * 0.95, size * 0.35, size * 0.85, size * 0.8, size * 0.5, size * 0.95);
  g.bezierCurveTo(size * 0.15, size * 0.8, size * 0.05, size * 0.35, size * 0.5, size * 0.08);
  g.fill();
  leafTexture = new THREE.CanvasTexture(canvas);
  return leafTexture;
}

export class Weather {
  group = new THREE.Group();
  private layers: Record<Exclude<WeatherKind, null>, Layer>;
  /** Fireflies, which come and go with the biome rather than with the sky. */
  private sparks = new Sparks();
  private sparksOn = false;
  private kind: WeatherKind = null;
  private hidden = false;

  constructor() {
    this.group.matrixAutoUpdate = false;
    this.layers = {
      rain: new Rain(),
      snow: new Flakes(SNOW_COUNT, 0xf4f8fb, 0.26, 0.85, false, false),
      embers: new Flakes(EMBER_COUNT, 0xffa040, 0.26, 0.95, true, true),
      leaves: new Flakes(LEAF_COUNT, 0xffffff, 0.42, 0.95, false, false, {
        sprite: 'leaf',
        palette: LEAF_COLORS,
        fall: 0.34,
        sway: 2.6
      })
    };
    for (const l of Object.values(this.layers)) this.group.add(l.object);
    this.group.add(this.sparks.object);
  }

  /** How far the rain has faded in, for the wet ground tint. */
  get rainLevel(): number {
    return this.layers.rain.level;
  }

  /** How far the snow has faded in, for the snow settling on roofs. */
  get snowLevel(): number {
    return this.layers.snow.level;
  }

  /** Weather to fade toward. The change takes about two seconds. */
  set(kind: WeatherKind): void {
    this.kind = kind;
    this.applyTargets();
  }

  /** Fireflies on or off, for the biomes with something green to hide in. */
  setFireflies(on: boolean): void {
    if (this.sparksOn === on) return;
    this.sparksOn = on;
    this.applyTargets();
  }

  /** Hide all weather, for example in globe view. Fades like a biome change. */
  setHidden(hidden: boolean): void {
    if (this.hidden === hidden) return;
    this.hidden = hidden;
    this.applyTargets();
  }

  private applyTargets(): void {
    for (const [k, l] of Object.entries(this.layers)) l.target = !this.hidden && k === this.kind ? 1 : 0;
    this.sparks.target = !this.hidden && this.sparksOn ? 1 : 0;
  }

  /** Advance the particles and carry the emitter with the player at `dir`. */
  update(dt: number, t: number, dir: THREE.Vector3): void {
    let any = false;
    for (const l of [...Object.values(this.layers), this.sparks]) {
      if (l.fade(dt)) {
        l.simulate(dt, t);
        any = true;
      }
    }
    this.group.visible = any;
    if (!any) return;
    tangentBasis(dir, 0, _right, _fwd);
    _m.makeBasis(_right, dir, _fwd);
    const r = walkRadius(dir);
    _m.setPosition(dir.x * r, dir.y * r, dir.z * r);
    this.group.matrix.copy(_m);
    this.group.matrixWorldNeedsUpdate = true;
  }
}
