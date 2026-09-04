// Night sky extras: named constellations drawn from the planet's own scenery,
// and the occasional shooting star. Both hang far outside the planet on their
// own shell and fade with the same star opacity the day-night clock samples.

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

  /** Match the star field's fade, so the figures come and go with the night. */
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
