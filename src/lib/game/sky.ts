// Night sky: the moon, a few planets, named constellations drawn from the
// planet's own scenery, and the occasional shooting star. All of them hang far
// outside the planet on their own shell, well inside the camera's far plane.

import * as THREE from 'three';
import { deriveSeed, seededRng } from './noise';
import { tangentBasis } from './planet';

/** Radius of the shell the constellations sit on, just inside the star field. */
const RADIUS = 500;
/** Angular half-size of a figure in the sky, in radians. */
const FIGURE_SIZE = 0.13;

/**
 * A figure to draw in the sky: points in a local square roughly spanning
 * [-1, 1], and the pairs of points joined by a line.
 */
type Figure = { name: string; points: [number, number][]; edges: [number, number][] };

/** Eight figures, each one a thing the planet already has somewhere on it. */
const FIGURES: Figure[] = [
  {
    name: 'The Rowboat',
    points: [[-1, -0.1], [-0.5, -0.6], [0.5, -0.6], [1, -0.1], [0, 0.15], [0, 0.9]],
    edges: [[0, 1], [1, 2], [2, 3], [3, 4], [4, 0], [4, 5]]
  },
  {
    name: 'The Fox',
    points: [[-1, -0.3], [-0.55, 0.5], [-0.1, -0.25], [0.55, -0.2], [0.9, 0.45], [1, -0.75]],
    edges: [[0, 1], [1, 2], [2, 3], [3, 4], [3, 5]]
  },
  {
    name: 'The Windmill',
    points: [[0, 0.1], [-0.8, 0.8], [0.8, 0.8], [-0.8, -0.6], [0.8, -0.6], [0, -1]],
    edges: [[1, 0], [0, 2], [3, 0], [0, 4], [0, 5]]
  },
  {
    name: 'The Small Wonder',
    points: [[0, 1], [-0.7, 0.1], [0, -1], [0.7, 0.1], [-0.25, 0.1], [0.25, 0.1]],
    edges: [[0, 1], [1, 2], [2, 3], [3, 0], [4, 5], [0, 4], [0, 5]]
  },
  {
    name: 'The Great Bear',
    points: [[-1, -0.35], [-0.6, 0.35], [0.1, 0.4], [0.7, 0.1], [1, 0.6], [0.3, -0.6], [-0.5, -0.8]],
    edges: [[0, 1], [1, 2], [2, 3], [3, 4], [3, 5], [5, 6], [6, 0]]
  },
  {
    name: 'The Pine',
    points: [[0, 1], [-0.7, -0.1], [0.7, -0.1], [-0.35, 0.4], [0.35, 0.4], [0, -0.9]],
    edges: [[0, 3], [3, 1], [0, 4], [4, 2], [1, 2], [0, 5]]
  },
  {
    name: 'The Lantern',
    points: [[-0.5, 0.4], [0.5, 0.4], [0.6, -0.5], [-0.6, -0.5], [0, 0.6], [0, 1]],
    edges: [[0, 1], [1, 2], [2, 3], [3, 0], [0, 4], [1, 4], [4, 5]]
  },
  {
    name: 'The Long Wave',
    points: [[-1, -0.2], [-0.5, 0.45], [0, -0.2], [0.5, 0.45], [1, -0.2], [0.75, -0.8]],
    edges: [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5]]
  }
];

const _e1 = new THREE.Vector3();
const _e2 = new THREE.Vector3();
const _p = new THREE.Vector3();

// ---------------------------------------------------------------- moon

/** Radius the moon hangs at, outside the constellations. */
const MOON_RADIUS = 460;
/** Half-width of the moon's quad. The disc fills `DISC` of it; the rest is halo. */
const MOON_SIZE = 40;

const MOON_FRAG = `
uniform vec3 uFace;
uniform vec3 uDark;
uniform vec3 uHalo;
uniform float uPhase;
varying vec2 vUv;

const float DISC = 0.45;

float blob(vec2 p, vec2 c, float r) {
  return smoothstep(r, r * 0.35, distance(p, c));
}

void main() {
  vec2 p = (vUv - 0.5) * 2.0;
  float d = length(p);

  // A soft halo that reaches well past the disc and lifts the sky around it.
  float halo = pow(max(0.0, 1.0 - d), 3.5);
  vec3 col = uHalo * halo;
  float a = halo * 0.5;

  // The terminator is an ellipse across the face, so the phase reads as a
  // crescent rather than a straight cut. The unlit side keeps a little
  // earthshine so the whole disc stays visible.
  float ny = clamp(p.y / DISC, -1.0, 1.0);
  float term = uPhase * sqrt(max(0.0, 1.0 - ny * ny)) * DISC;
  float lit = smoothstep(term - 0.035, term + 0.035, p.x);
  vec3 face = mix(uDark, uFace, lit);
  face *= 1.0 - 0.22 * blob(p, vec2(-0.13, 0.15), 0.14);
  face *= 1.0 - 0.17 * blob(p, vec2(0.17, -0.04), 0.11);
  face *= 1.0 - 0.13 * blob(p, vec2(0.01, -0.26), 0.09);

  float disc = smoothstep(DISC, DISC - 0.015, d);
  col = mix(col, face, disc);
  a = max(a, disc);
  gl_FragColor = vec4(col, a);
}
`;

const MOON_VERT = `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

/**
 * A billboarded moon at a seeded point in the sky, with a phase and a halo. It
 * is scenery, not the key light: the moonlight in `Game` rides over the
 * camera's shoulder so the visible face of the planet is always readable, the
 * same trick the sun used before. Walking far enough around the planet still
 * carries the moon down past the horizon and brings it back up the far side.
 */
export class Moon {
  readonly group = new THREE.Group();
  /** Where in the sky this planet's moon sits. */
  readonly direction: THREE.Vector3;
  private mesh: THREE.Mesh;

  constructor(seed: number) {
    const rng = seededRng(deriveSeed(seed, 11));
    this.direction = new THREE.Vector3(rng() * 2 - 1, rng() * 2 - 1, rng() * 2 - 1).normalize();
    const mat = new THREE.ShaderMaterial({
      uniforms: {
        uFace: { value: new THREE.Color(0xe4ecf7) },
        uDark: { value: new THREE.Color(0x2b3a58) },
        uHalo: { value: new THREE.Color(0x6d8cc4) },
        // A waxing or waning gibbous: enough of a terminator to read as a phase,
        // never so thin that the moon stops lighting the sky.
        uPhase: { value: (rng() < 0.5 ? -1 : 1) * (0.25 + rng() * 0.35) }
      },
      vertexShader: MOON_VERT,
      fragmentShader: MOON_FRAG,
      transparent: true,
      depthWrite: false,
      toneMapped: false
    });
    this.mesh = new THREE.Mesh(new THREE.PlaneGeometry(MOON_SIZE * 2, MOON_SIZE * 2), mat);
    this.mesh.position.copy(this.direction).multiplyScalar(MOON_RADIUS);
    this.mesh.renderOrder = -2;
    this.mesh.frustumCulled = false;
    this.group.add(this.mesh);
  }

  /** Turn the quad to face the camera. The moon is too far to move otherwise. */
  update(camera: THREE.Camera): void {
    this.mesh.quaternion.copy(camera.quaternion);
  }
}

// ---------------------------------------------------------------- planets

/** Radius the planets hang at, among the constellations. */
const PLANET_RADIUS_SKY = 490;

/** Colour and point size of each wanderer, brightest first. The star field
 * draws at size 1.6, so even the faintest of these reads as a planet. */
const WANDERERS: [number, number][] = [
  [0xffc9a0, 11],
  [0xd98a5a, 9],
  [0xdfe6ff, 8],
  [0xf0dca8, 7]
];

const PLANET_VERT = `
attribute float size;
attribute vec3 tint;
varying vec3 vTint;
void main() {
  vTint = tint;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  gl_PointSize = size;
}
`;

const PLANET_FRAG = `
varying vec3 vTint;
void main() {
  // A solid core inside a soft halo, so the point reads as a small disc rather
  // than a square or a single lit pixel.
  float d = length(gl_PointCoord - 0.5) * 2.0;
  float core = smoothstep(0.55, 0.0, d);
  float halo = smoothstep(1.0, 0.0, d);
  gl_FragColor = vec4(vTint, core * 0.9 + halo * halo * 0.35);
}
`;

/**
 * Four wanderers: bigger and warmer than the stars behind them, and, unlike the
 * stars and the constellation figures, perfectly steady. Planets do not
 * twinkle, and leaving them still is what tells them apart at a glance.
 */
export class Planets {
  readonly group = new THREE.Group();

  constructor(seed: number) {
    const rng = seededRng(deriveSeed(seed, 12));
    const pos = new Float32Array(WANDERERS.length * 3);
    const col = new Float32Array(WANDERERS.length * 3);
    const size = new Float32Array(WANDERERS.length);
    const c = new THREE.Color();
    for (let i = 0; i < WANDERERS.length; i++) {
      _p.set(rng() * 2 - 1, rng() * 2 - 1, rng() * 2 - 1).normalize().multiplyScalar(PLANET_RADIUS_SKY);
      pos[i * 3] = _p.x;
      pos[i * 3 + 1] = _p.y;
      pos[i * 3 + 2] = _p.z;
      c.setHex(WANDERERS[i][0]);
      col[i * 3] = c.r;
      col[i * 3 + 1] = c.g;
      col[i * 3 + 2] = c.b;
      size[i] = WANDERERS[i][1];
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    geo.setAttribute('tint', new THREE.BufferAttribute(col, 3));
    geo.setAttribute('size', new THREE.BufferAttribute(size, 1));
    const points = new THREE.Points(
      geo,
      new THREE.ShaderMaterial({
        vertexShader: PLANET_VERT,
        fragmentShader: PLANET_FRAG,
        transparent: true,
        depthWrite: false,
        toneMapped: false,
        blending: THREE.AdditiveBlending
      })
    );
    points.frustumCulled = false;
    this.group.add(points);
  }
}

// ---------------------------------------------------------------- figures

/**
 * The named figures, as bright stars joined by faint lines. Each planet's seed
 * decides where in the sky the figures fall and how they are turned.
 */
export class Constellations {
  group = new THREE.Group();
  /** Names in the order they were placed, for the console and future captions. */
  readonly names: string[] = [];
  private points: THREE.Points;
  private lines: THREE.LineSegments;
  private colors: Float32Array;
  private base: Float32Array;
  private phase: Float32Array;
  private opacity = 0;

  constructor(seed: number) {
    const rng = seededRng(deriveSeed(seed, 9));
    const starPos: number[] = [];
    const linePos: number[] = [];
    const centers: THREE.Vector3[] = [];

    for (const figure of FIGURES) {
      // Best-candidate sampling keeps the figures from piling up in one patch of sky.
      let center = new THREE.Vector3();
      let bestGap = -1;
      for (let tries = 0; tries < 8; tries++) {
        const c = new THREE.Vector3(rng() * 2 - 1, rng() * 2 - 1, rng() * 2 - 1).normalize();
        const gap = centers.reduce((min, other) => Math.min(min, c.angleTo(other)), Math.PI);
        if (gap > bestGap) {
          bestGap = gap;
          center = c;
        }
      }
      centers.push(center);
      this.names.push(figure.name);

      tangentBasis(center, rng() * Math.PI * 2, _e1, _e2);
      const scale = FIGURE_SIZE * (0.8 + rng() * 0.5);
      const placed = figure.points.map(([x, y]) => {
        _p.copy(center).addScaledVector(_e1, x * scale).addScaledVector(_e2, y * scale).normalize().multiplyScalar(RADIUS);
        starPos.push(_p.x, _p.y, _p.z);
        return _p.clone();
      });
      for (const [a, b] of figure.edges) {
        linePos.push(placed[a].x, placed[a].y, placed[a].z, placed[b].x, placed[b].y, placed[b].z);
      }
    }

    const count = starPos.length / 3;
    this.colors = new Float32Array(count * 3);
    this.base = new Float32Array(count);
    this.phase = new Float32Array(count);
    for (let i = 0; i < count; i++) {
      this.base[i] = 0.7 + rng() * 0.3;
      this.phase[i] = rng() * Math.PI * 2;
    }

    const starGeo = new THREE.BufferGeometry();
    starGeo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(starPos), 3));
    starGeo.setAttribute('color', new THREE.BufferAttribute(this.colors, 3));
    this.points = new THREE.Points(
      starGeo,
      new THREE.PointsMaterial({ size: 3.2, sizeAttenuation: false, vertexColors: true, transparent: true, opacity: 0, depthWrite: false })
    );

    const lineGeo = new THREE.BufferGeometry();
    lineGeo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(linePos), 3));
    this.lines = new THREE.LineSegments(
      lineGeo,
      new THREE.LineBasicMaterial({ color: 0x8fb6d8, transparent: true, opacity: 0, depthWrite: false })
    );

    this.group.add(this.points, this.lines);
    this.group.visible = false;
  }

  /** Overall brightness of the figures. */
  setOpacity(o: number): void {
    this.opacity = o;
    const on = o > 0.02;
    this.group.visible = on;
    if (!on) return;
    (this.points.material as THREE.PointsMaterial).opacity = Math.min(1, o * 1.15);
    // The joining lines stay much fainter than the stars they connect.
    (this.lines.material as THREE.LineBasicMaterial).opacity = o * 0.3;
  }

  /** Twinkle the figure stars. Cheap: a few dozen vertex colours per frame. */
  update(time: number): void {
    if (this.opacity <= 0.02) return;
    for (let i = 0; i < this.base.length; i++) {
      const v = this.base[i] * (0.78 + Math.sin(time * 1.7 + this.phase[i]) * 0.22);
      this.colors[i * 3] = v * 0.92;
      this.colors[i * 3 + 1] = v * 0.96;
      this.colors[i * 3 + 2] = v;
    }
    (this.points.geometry.attributes.color as THREE.BufferAttribute).needsUpdate = true;
  }
}

// ---------------------------------------------------------------- meteors

/** Points along a streak; the tail fades toward the last one. */
const TAIL = 12;
/** Radius the streaks travel at, just inside the constellations. */
const METEOR_RADIUS = 470;
/** Seconds a streak takes to cross its arc. */
const METEOR_LIFE = 1.1;

/** One shooting star at a time, every half a minute or so, only after dark. */
export class Meteors {
  group = new THREE.Group();
  private line: THREE.Line;
  private colors: Float32Array;
  private from = new THREE.Vector3();
  private toward = new THREE.Vector3();
  private life = 0;
  private wait: number;
  private rng: () => number;
  private opacity = 0;

  constructor(seed: number) {
    this.rng = seededRng(deriveSeed(seed, 10));
    this.wait = 6 + this.rng() * 18;
    this.colors = new Float32Array(TAIL * 3);
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(TAIL * 3), 3));
    geo.setAttribute('color', new THREE.BufferAttribute(this.colors, 3));
    this.line = new THREE.Line(
      geo,
      new THREE.LineBasicMaterial({ vertexColors: true, transparent: true, opacity: 1, depthWrite: false, blending: THREE.AdditiveBlending })
    );
    this.line.frustumCulled = false;
    this.group.add(this.line);
    this.group.visible = false;
  }

  /** Streaks only appear once the sky is dark enough to see them. */
  setOpacity(o: number): void {
    this.opacity = o;
  }

  update(dt: number): void {
    if (this.life <= 0) {
      this.group.visible = false;
      if (this.opacity < 0.35) return;
      this.wait -= dt;
      if (this.wait > 0) return;
      this.launch();
    }

    this.life -= dt;
    if (this.life <= 0) {
      this.group.visible = false;
      this.wait = 14 + this.rng() * 34;
      return;
    }
    this.group.visible = true;
    const age = 1 - this.life / METEOR_LIFE;
    const head = age * 0.5;
    const pos = this.line.geometry.attributes.position as THREE.BufferAttribute;
    for (let i = 0; i < TAIL; i++) {
      const a = Math.max(0, head - (i / (TAIL - 1)) * 0.09);
      _p.copy(this.from).multiplyScalar(Math.cos(a)).addScaledVector(this.toward, Math.sin(a)).multiplyScalar(METEOR_RADIUS);
      pos.setXYZ(i, _p.x, _p.y, _p.z);
      // Bright head, fading tail, the whole streak easing in and out over its life.
      const v = (1 - i / (TAIL - 1)) ** 1.6 * Math.sin(age * Math.PI) * this.opacity;
      this.colors[i * 3] = v;
      this.colors[i * 3 + 1] = v * 0.96;
      this.colors[i * 3 + 2] = v * 0.85;
    }
    pos.needsUpdate = true;
    (this.line.geometry.attributes.color as THREE.BufferAttribute).needsUpdate = true;
  }

  private launch(): void {
    this.from.set(this.rng() * 2 - 1, this.rng() * 2 - 1, this.rng() * 2 - 1).normalize();
    tangentBasis(this.from, this.rng() * Math.PI * 2, _e1, _e2);
    this.toward.copy(_e1);
    this.life = METEOR_LIFE;
  }
}
