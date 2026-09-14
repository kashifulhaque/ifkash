// Night sky: the moon, a few planets, named constellations drawn from the
// planet's own scenery, and the occasional shooting star. All of them hang far
// outside the planet on their own shell, well inside the camera's far plane.

import * as THREE from 'three';
import { deriveSeed, seededRng, valueNoise3 } from './noise';
import { tangentBasis } from './planet';
import { hashSeed } from './seed';
import { planetProfile, type PlanetProfile, type SpaceDestination } from './space';

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

// ---------------------------------------------------------------- destination planets

/** The reachable worlds sit far enough away to read as celestial bodies while
 * still being close enough for a short, continuous flight. */
const DESTINATION_DISTANCE = 245;

type PreviewField = { x: number; y: number; z: number };

export type PlanetTarget = {
  destination: SpaceDestination;
  position: THREE.Vector3;
  radius: number;
  body: THREE.Mesh;
  water: THREE.Mesh;
  atmosphere: THREE.Mesh;
  ring: THREE.Mesh;
  label: THREE.Sprite;
  /** Terrain radius in the target's local radial direction. */
  surfaceRadius: (direction: THREE.Vector3) => number;
};

/** Coherent noise independent of the active world's global terrain seed. */
function previewFbm(direction: THREE.Vector3, scale: number, field: PreviewField, octaves: number): number {
  let sum = 0;
  let amplitude = 1;
  let normalizer = 0;
  let frequency = 1;
  const x = direction.x * scale + field.x;
  const y = direction.y * scale + field.y;
  const z = direction.z * scale + field.z;
  for (let octave = 0; octave < octaves; octave++) {
    sum += valueNoise3(x * frequency + octave * 17.3, y * frequency - octave * 9.1, z * frequency + octave * 4.7) * amplitude;
    normalizer += amplitude;
    amplitude *= 0.5;
    frequency *= 2;
  }
  return sum / normalizer;
}


/** Low-poly relief matching the destination's generated archetype. */
function previewSurfaceRadius(
  direction: THREE.Vector3,
  radius: number,
  profile: PlanetProfile,
  field: PreviewField,
  continent = previewFbm(direction, 1.35, field, 4)
): number {
  const coast = -0.04 - profile.landBias * 0.55;
  if (continent <= coast) return radius * (0.965 - Math.min(0.035, (coast - continent) * 0.06));

  const broad = Math.max(0, continent - coast);
  const detail = previewFbm(direction, 5.2, field, 3);
  const ridge = 1 - Math.abs(previewFbm(direction, 2.8, field, 3));
  let height = 0.018 + broad * 0.12 * profile.relief + detail * 0.018 * profile.ruggedness + ridge * ridge * ridge * 0.035 * profile.ruggedness;
  if (profile.archetype === 'crystal') height = Math.round(height / 0.018) * 0.018;
  else if (profile.archetype === 'fracture') height = Math.round(height / 0.026) * 0.026;
  return radius * (1 + Math.max(0.008, height));
}

function destinationTerrain(destination: SpaceDestination, radius: number): { body: THREE.Mesh; water: THREE.Mesh; surfaceRadius: (d: THREE.Vector3) => number } {
  const profile = planetProfile(destination.seed);
  const fieldRng = seededRng(deriveSeed(hashSeed(destination.seed), 14));
  const field = { x: fieldRng() * 500 - 250, y: fieldRng() * 500 - 250, z: fieldRng() * 500 - 250 };
  const geometry = new THREE.IcosahedronGeometry(1, 4);
  const positions = geometry.attributes.position as THREE.BufferAttribute;
  const colors = new Float32Array(positions.count * 3);
  const a = new THREE.Vector3();
  const b = new THREE.Vector3();
  const c = new THREE.Vector3();
  const midpoint = new THREE.Vector3();
  const color = new THREE.Color();
  const coast = -0.04 - profile.landBias * 0.55;

  for (let i = 0; i < positions.count; i += 3) {
    a.fromBufferAttribute(positions, i).normalize();
    b.fromBufferAttribute(positions, i + 1).normalize();
    c.fromBufferAttribute(positions, i + 2).normalize();
    midpoint.copy(a).add(b).add(c).normalize();
    const continent = previewFbm(midpoint, 1.35, field, 4);
    const surface = previewSurfaceRadius(midpoint, radius, profile, field, continent);
    if (continent <= coast) color.set(profile.terrain[0]).multiplyScalar(0.52);
    else if (continent < coast + 0.075) color.set(profile.shore);
    else {
      const rise = (surface / radius - 1) / 0.12;
      color.set(profile.terrain[rise < 0.38 ? 0 : rise < 0.72 ? 1 : 2]);
    }
    color.multiplyScalar(0.92 + valueNoise3(midpoint.x * 9 + field.x, midpoint.y * 9 + field.y, midpoint.z * 9 + field.z) * 0.08);

    for (let vertex = 0; vertex < 3; vertex++) {
      const direction = vertex === 0 ? a : vertex === 1 ? b : c;
      const vertexRadius = previewSurfaceRadius(direction, radius, profile, field);
      positions.setXYZ(i + vertex, direction.x * vertexRadius, direction.y * vertexRadius, direction.z * vertexRadius);
      colors[(i + vertex) * 3] = color.r;
      colors[(i + vertex) * 3 + 1] = color.g;
      colors[(i + vertex) * 3 + 2] = color.b;
    }
  }

  geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));
  geometry.deleteAttribute('uv');
  geometry.computeVertexNormals();
  const body = new THREE.Mesh(
    geometry,
    new THREE.MeshStandardMaterial({
      vertexColors: true,
      emissive: new THREE.Color(profile.accent).multiplyScalar(0.055),
      roughness: 0.94,
      metalness: 0,
      flatShading: true
    })
  );
  const water = new THREE.Mesh(
    new THREE.SphereGeometry(radius, 48, 32),
    new THREE.MeshStandardMaterial({
      color: profile.water,
      emissive: new THREE.Color(profile.water).multiplyScalar(0.08),
      transparent: true,
      opacity: 0.78,
      roughness: 0.28,
      metalness: 0.08
    })
  );
  return {
    body,
    water,
    surfaceRadius: (direction) => Math.max(radius, previewSurfaceRadius(direction, radius, profile, field))
  };
}

/** Paint one crisp, camera-facing route label without bringing DOM overlays
 * into the render loop. The texture is built once per generated destination. */
function destinationLabel(destination: SpaceDestination): THREE.Sprite {
  const canvas = document.createElement('canvas');
  canvas.width = 768;
  canvas.height = 160;
  const ctx = canvas.getContext('2d')!;
  ctx.fillStyle = 'rgba(6, 15, 24, 0.82)';
  ctx.beginPath();
  ctx.roundRect(4, 4, canvas.width - 8, canvas.height - 8, 28);
  ctx.fill();
  ctx.strokeStyle = 'rgba(242, 239, 230, 0.55)';
  ctx.lineWidth = 4;
  ctx.stroke();
  ctx.fillStyle = '#f2efe6';
  ctx.font = '600 42px Inter, system-ui, sans-serif';
  ctx.textAlign = 'center';
  ctx.fillText(destination.name, canvas.width / 2, 65);
  ctx.fillStyle = '#e9c46a';
  ctx.font = '500 25px Inter, system-ui, sans-serif';
  ctx.fillText(`${destination.kind.replace(/[-_]/g, ' ')} · ${destination.fuelCost} fuel`, canvas.width / 2, 111);

  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.minFilter = THREE.LinearFilter;
  const sprite = new THREE.Sprite(
    new THREE.SpriteMaterial({
      map: texture,
      transparent: true,
      depthTest: true,
      depthWrite: false,
      toneMapped: false
    })
  );
  sprite.scale.set(42, 8.75, 1);
  return sprite;
}

/**
 * Three generated onward routes rendered as terrain-bearing worlds. Placement
 * is a stable fan above the landing pad so each route is visible before launch.
 */
export class Planets {
  readonly group = new THREE.Group();
  readonly targets: PlanetTarget[] = [];

  constructor(seed: number, destinations: readonly SpaceDestination[], launchDir: THREE.Vector3) {
    const rng = seededRng(deriveSeed(seed, 12));
    const anchor = launchDir.clone().normalize();
    const east = new THREE.Vector3();
    const north = new THREE.Vector3();
    tangentBasis(anchor, rng() * Math.PI * 2, east, north);
    const fan: ReadonlyArray<readonly [number, number]> = [
      [-0.62, 0.3],
      [0, 0.44],
      [0.62, 0.24]
    ];

    for (let i = 0; i < destinations.length; i++) {
      const destination = destinations[i];
      const [across, radial] = fan[i] ?? [rng() - 0.5, 0.2 + rng() * 0.25];
      const direction = north.clone().addScaledVector(east, across).addScaledVector(anchor, radial).normalize();
      const distance = DESTINATION_DISTANCE + i * 24;
      const radius = 18 + rng() * 4;
      const position = direction.multiplyScalar(distance);
      const profile = planetProfile(destination.seed);
      const color = new THREE.Color(profile.water);
      const root = new THREE.Group();
      root.position.copy(position);

      const terrain = destinationTerrain(destination, radius);
      const atmosphere = new THREE.Mesh(
        new THREE.SphereGeometry(radius * 1.16, 32, 20),
        new THREE.MeshBasicMaterial({
          color: color.clone().lerp(new THREE.Color(profile.accent), 0.42),
          transparent: true,
          opacity: 0.16,
          side: THREE.BackSide,
          depthWrite: false,
          toneMapped: false
        })
      );
      const ring = new THREE.Mesh(
        new THREE.TorusGeometry(radius * 1.52, 0.32, 6, 64),
        new THREE.MeshBasicMaterial({
          color: new THREE.Color(profile.accent).lerp(new THREE.Color(0xffffff), 0.35),
          transparent: true,
          opacity: 0.5,
          depthWrite: false,
          toneMapped: false
        })
      );
      ring.rotation.set(Math.PI / 2 + (rng() - 0.5) * 0.5, rng() * Math.PI, (rng() - 0.5) * 0.45);
      const label = destinationLabel(destination);
      label.position.set(0, radius * 1.78, 0);
      root.add(terrain.body, terrain.water, atmosphere, ring, label);
      this.group.add(root);
      this.targets.push({ destination, position, radius, body: terrain.body, water: terrain.water, atmosphere, ring, label, surfaceRadius: terrain.surfaceRadius });
    }
  }

  /** Remove route graphics once a ship commits to a landing approach. */
  beginLanding(target: PlanetTarget): void {
    target.label.visible = false;
    target.ring.visible = false;
  }

  update(dt: number, time: number): void {
    for (let i = 0; i < this.targets.length; i++) {
      const target = this.targets[i];
      target.water.rotation.y += dt * (0.025 + i * 0.008);
      target.ring.rotation.z += dt * (i % 2 === 0 ? 0.05 : -0.04);
      const pulse = 1 + Math.sin(time * 1.4 + i * 1.8) * 0.025;
      target.atmosphere.scale.setScalar(pulse);
    }
  }

  dispose(): void {
    for (const target of this.targets) {
      const material = target.label.material as THREE.SpriteMaterial;
      material.map?.dispose();
      material.dispose();
    }
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
