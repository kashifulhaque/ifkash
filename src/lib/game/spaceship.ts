import * as THREE from 'three';
import type { ColliderGrid } from './collision';
import { emitter, type Emitter, type LightPool } from './lights';
import { deriveSeed, seededRng } from './noise';
import { isOcean, PLANET_RADIUS, tangentBasis, walkRadius } from './planet';
import { SHIP_PARTS } from './space';

const PICKUP_RANGE = 1.2;
const PART_SPACING = 8;
const AVOID_CLEARANCE = 5;
const POP_TIME = 0.45;
const FLOAT_HEIGHT = 0.85;
const PICKUP_DOT = Math.cos(PICKUP_RANGE / PLANET_RADIUS);
const PART_COLOR = 0xe8d8bd;
const ACCENT_COLOR = 0xd97b55;
const GLOW_COLOR = 0x7edbd1;

const _right = new THREE.Vector3();
const _forward = new THREE.Vector3();
const _frame = new THREE.Matrix4();
const _spin = new THREE.Quaternion();
const _ringTurn = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), -Math.PI / 2);

type AnimatedMaterial = {
  material: THREE.MeshStandardMaterial;
  emissive: number;
};

type Part = {
  dir: THREE.Vector3;
  groundRadius: number;
  root: THREE.Group;
  ring: THREE.Mesh<THREE.RingGeometry, THREE.MeshBasicMaterial>;
  light: Emitter;
  frame: THREE.Quaternion;
  materials: AnimatedMaterial[];
  found: boolean;
  pop: number;
};

function farEnough(dir: THREE.Vector3, others: THREE.Vector3[], clearance: number): boolean {
  for (let i = 0; i < others.length; i++) {
    if (dir.angleTo(others[i]) * PLANET_RADIUS < clearance) return false;
  }
  return true;
}

/** Stable placement with a uniform seeded pass and a deterministic dense fallback. */
function placeParts(seed: number, colliders: ColliderGrid, avoid: THREE.Vector3[]): THREE.Vector3[] {
  const rng = seededRng(deriveSeed(seed, 37));
  const placed: THREE.Vector3[] = [];
  const candidate = new THREE.Vector3();

  const accept = (): boolean => {
    if (isOcean(candidate) || colliders.blocked(candidate, 0.85)) return false;
    if (!farEnough(candidate, avoid, AVOID_CLEARANCE)) return false;
    if (!farEnough(candidate, placed, PART_SPACING)) return false;
    placed.push(candidate.clone());
    return true;
  };

  for (let tries = 0; tries < 6000 && placed.length < SHIP_PARTS.length; tries++) {
    const z = rng() * 2 - 1;
    const phi = rng() * Math.PI * 2;
    const radius = Math.sqrt(1 - z * z);
    candidate.set(radius * Math.cos(phi), radius * Math.sin(phi), z);
    accept();
  }

  // This should only matter on unusually crowded generated worlds. A Fibonacci
  // sphere searches evenly without weakening any placement constraint.
  const samples = 16384;
  const phase = rng() * Math.PI * 2;
  const goldenAngle = Math.PI * (3 - Math.sqrt(5));
  for (let i = 0; i < samples && placed.length < SHIP_PARTS.length; i++) {
    const z = 1 - (2 * (i + 0.5)) / samples;
    const phi = phase + i * goldenAngle;
    const radius = Math.sqrt(1 - z * z);
    candidate.set(radius * Math.cos(phi), radius * Math.sin(phi), z);
    accept();
  }

  if (placed.length !== SHIP_PARTS.length) {
    throw new Error(`Could not place all ${SHIP_PARTS.length} spaceship parts on this planet`);
  }
  return placed;
}

function material(color: number, emissive = 0): THREE.MeshStandardMaterial {
  return new THREE.MeshStandardMaterial({
    color,
    emissive,
    emissiveIntensity: emissive ? 0.9 : 0,
    roughness: 0.68,
    metalness: 0.18,
    flatShading: true,
    transparent: true
  });
}

function addPartMesh(
  parent: THREE.Group,
  geometry: THREE.BufferGeometry,
  mat: THREE.MeshStandardMaterial,
  x = 0,
  y = 0,
  z = 0
): THREE.Mesh {
  const mesh = new THREE.Mesh(geometry, mat);
  mesh.position.set(x, y, z);
  mesh.castShadow = true;
  parent.add(mesh);
  return mesh;
}

/** Flat triangular prism used by both the loose wings and the finished ship. */
function wingGeometry(side: -1 | 1, span: number, chord: number, thickness: number): THREE.BufferGeometry {
  const root = 0.15 * side;
  const tip = span * side;
  const y0 = -thickness / 2;
  const y1 = thickness / 2;
  const vertices = new Float32Array([
    root, y0, -chord / 2,
    root, y0, chord / 2,
    tip, y0, -chord * 0.3,
    root, y1, -chord / 2,
    root, y1, chord / 2,
    tip, y1, -chord * 0.3
  ]);
  const triangles = [0, 2, 1, 3, 4, 5, 0, 1, 4, 0, 4, 3, 1, 2, 5, 1, 5, 4, 2, 0, 3, 2, 3, 5];
  if (side < 0) {
    for (let i = 0; i < triangles.length; i += 3) {
      const swap = triangles[i + 1];
      triangles[i + 1] = triangles[i + 2];
      triangles[i + 2] = swap;
    }
  }
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
  geometry.setIndex(triangles);
  geometry.computeVertexNormals();
  return geometry;
}

function buildPartVisual(index: number, pipGeometry: THREE.BufferGeometry): { root: THREE.Group; materials: AnimatedMaterial[] } {
  const root = new THREE.Group();
  root.name = `spaceship-part-${index + 1}`;
  root.userData.partIndex = index;
  root.userData.partName = SHIP_PARTS[index];

  const main = material(PART_COLOR, GLOW_COLOR);
  const accent = material(index === 3 ? 0x79b8c9 : ACCENT_COLOR, GLOW_COLOR);
  const dark = material(0x45505a, GLOW_COLOR);
  const materials: AnimatedMaterial[] = [
    { material: main, emissive: 0.75 },
    { material: accent, emissive: 1.05 },
    { material: dark, emissive: 0.55 }
  ];

  if (index === 0) {
    const plate = addPartMesh(root, new THREE.BoxGeometry(1.25, 0.16, 0.82), main);
    plate.rotation.y = 0.12;
    addPartMesh(root, new THREE.BoxGeometry(0.14, 0.24, 0.92), accent, -0.42, 0.05);
    addPartMesh(root, new THREE.BoxGeometry(0.14, 0.24, 0.92), accent, 0.42, 0.05);
    addPartMesh(root, new THREE.CylinderGeometry(0.08, 0.08, 0.2, 6), dark, 0, 0.16, 0);
  } else if (index === 1) {
    const canopy = addPartMesh(root, new THREE.DodecahedronGeometry(0.62, 0), accent, 0, 0.12);
    canopy.scale.set(0.9, 0.72, 1.15);
    addPartMesh(root, new THREE.CylinderGeometry(0.53, 0.62, 0.18, 8), dark, 0, -0.38);
    const frameBand = addPartMesh(root, new THREE.TorusGeometry(0.46, 0.055, 5, 10), main, 0, 0.09, 0);
    frameBand.rotation.x = Math.PI / 2;
  } else if (index === 2 || index === 3) {
    const side: -1 | 1 = index === 2 ? -1 : 1;
    addPartMesh(root, wingGeometry(side, 1.3, 1.05, 0.16), main);
    const stripe = addPartMesh(root, new THREE.BoxGeometry(0.65, 0.08, 0.13), accent, side * 0.52, 0.13, -0.12);
    stripe.rotation.y = side * -0.18;
    addPartMesh(root, new THREE.CylinderGeometry(0.1, 0.13, 0.34, 6), dark, side * 0.22, 0.08, 0);
  } else {
    const core = addPartMesh(root, new THREE.CylinderGeometry(0.42, 0.5, 0.82, 8), dark);
    core.rotation.x = Math.PI / 2;
    const rim = addPartMesh(root, new THREE.TorusGeometry(0.43, 0.08, 5, 12), accent, 0, 0, 0.42);
    rim.rotation.z = Math.PI / 8;
    addPartMesh(root, new THREE.CircleGeometry(0.34, 12), main, 0, 0, 0.435);
  }

  // One to five glowing pips make the persistent array index visible on the
  // model without canvas textures or a browser-only text dependency.
  const pipStart = -((index + 1) - 1) * 0.105;
  for (let i = 0; i <= index; i++) {
    addPartMesh(root, pipGeometry, accent, pipStart + i * 0.21, 0.56, 0);
  }
  return { root, materials };
}

export class ShipParts {
  readonly group = new THREE.Group();
  private readonly parts: Part[] = [];

  constructor(seed: number, colliders: ColliderGrid, lights: LightPool, found: number[], avoid: THREE.Vector3[]) {
    this.group.name = 'spaceship-parts';
    const directions = placeParts(seed, colliders, avoid);
    const pipGeometry = new THREE.OctahedronGeometry(0.075, 0);
    const ringGeometry = new THREE.RingGeometry(0.56, 0.69, 24);

    for (let i = 0; i < directions.length; i++) {
      const dir = directions[i];
      const groundRadius = walkRadius(dir);
      const visual = buildPartVisual(i, pipGeometry);
      tangentBasis(dir, 0, _right, _forward);
      _frame.makeBasis(_right, dir, _forward);
      const frame = new THREE.Quaternion().setFromRotationMatrix(_frame);
      visual.root.quaternion.copy(frame);
      visual.root.position.copy(dir).multiplyScalar(groundRadius + FLOAT_HEIGHT);

      const ringMaterial = new THREE.MeshBasicMaterial({
        color: GLOW_COLOR,
        transparent: true,
        opacity: 0.35,
        side: THREE.DoubleSide,
        depthWrite: false
      });
      const ring = new THREE.Mesh(ringGeometry, ringMaterial);
      ring.name = `spaceship-part-${i + 1}-marker`;
      ring.quaternion.copy(frame).multiply(_ringTurn);
      ring.position.copy(dir).multiplyScalar(groundRadius + 0.06);

      const light = lights.add(emitter(visual.root.position, GLOW_COLOR, 6, 6, { flicker: 0.15, phase: i * 2.3 }));
      const isFound = found.includes(i);
      if (isFound) {
        visual.root.visible = false;
        ring.visible = false;
        light.on = false;
      }

      this.group.add(visual.root, ring);
      this.parts.push({
        dir,
        groundRadius,
        root: visual.root,
        ring,
        light,
        frame,
        materials: visual.materials,
        found: isFound,
        pop: 0
      });
    }
  }

  get total(): number {
    return this.parts.length;
  }

  get foundCount(): number {
    let count = 0;
    for (let i = 0; i < this.parts.length; i++) {
      if (this.parts[i].found) count++;
    }
    return count;
  }

  get foundIndices(): number[] {
    const indices: number[] = [];
    for (let i = 0; i < this.parts.length; i++) {
      if (this.parts[i].found) indices.push(i);
    }
    return indices;
  }

  /** Pick up the part under the explorer, returning its stable index or -1. */
  collect(playerDir: THREE.Vector3): number {
    for (let i = 0; i < this.parts.length; i++) {
      const part = this.parts[i];
      if (part.found) continue;
      if (part.dir.dot(playerDir) > PICKUP_DOT) {
        part.found = true;
        part.pop = POP_TIME;
        part.light.on = false;
        part.ring.visible = false;
        return i;
      }
    }
    return -1;
  }

  update(dt: number, t: number): void {
    for (let i = 0; i < this.parts.length; i++) {
      const part = this.parts[i];
      if (part.found && part.pop <= 0) continue;

      if (part.pop > 0) {
        part.pop -= dt;
        const age = 1 - Math.max(0, part.pop) / POP_TIME;
        part.root.position.copy(part.dir).multiplyScalar(part.groundRadius + FLOAT_HEIGHT + age * 1.6);
        part.root.scale.setScalar(1 + age * 1.8);
        for (let j = 0; j < part.materials.length; j++) {
          const animated = part.materials[j];
          animated.material.opacity = 1 - age;
          animated.material.emissiveIntensity = animated.emissive + age * 3;
        }
        if (part.pop <= 0) part.root.visible = false;
        continue;
      }

      const bob = Math.sin(t * 2.1 + i * 1.3) * 0.12;
      part.root.position.copy(part.dir).multiplyScalar(part.groundRadius + FLOAT_HEIGHT + bob);
      _spin.setFromAxisAngle(part.dir, t * 1.4 + i);
      part.root.quaternion.copy(_spin).multiply(part.frame);
      const pulse = 0.8 + Math.sin(t * 3.2 + i) * 0.3;
      for (let j = 0; j < part.materials.length; j++) {
        const animated = part.materials[j];
        animated.material.emissiveIntensity = animated.emissive * pulse;
      }
      part.light.intensity = 6 * pulse;
      part.ring.material.opacity = 0.25 + Math.sin(t * 3.2 + i) * 0.12;
    }
  }
}

function shipMesh(geometry: THREE.BufferGeometry, mat: THREE.Material, parent: THREE.Group): THREE.Mesh {
  const mesh = new THREE.Mesh(geometry, mat);
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  parent.add(mesh);
  return mesh;
}

function buildConstructionSite(group: THREE.Group): void {
  const padMaterial = new THREE.MeshStandardMaterial({ color: 0x6f716b, roughness: 0.9, flatShading: true });
  const stripeMaterial = new THREE.MeshStandardMaterial({ color: 0xe4ae5f, roughness: 0.78, flatShading: true });
  const scaffoldMaterial = new THREE.MeshStandardMaterial({ color: 0xb8784f, roughness: 0.84, flatShading: true });
  const hologramMaterial = new THREE.MeshStandardMaterial({
    color: GLOW_COLOR,
    emissive: GLOW_COLOR,
    emissiveIntensity: 0.65,
    transparent: true,
    opacity: 0.28,
    wireframe: true,
    depthWrite: false
  });

  const pad = shipMesh(new THREE.CylinderGeometry(2.25, 2.35, 0.16, 16), padMaterial, group);
  pad.position.y = 0.08;
  const rim = shipMesh(new THREE.TorusGeometry(1.92, 0.08, 5, 24), stripeMaterial, group);
  rim.position.y = 0.18;
  rim.rotation.x = Math.PI / 2;

  const postGeometry = new THREE.BoxGeometry(0.12, 1.75, 0.12);
  const footGeometry = new THREE.BoxGeometry(0.42, 0.1, 0.42);
  const positions: ReadonlyArray<readonly [number, number]> = [
    [-1.45, -1.15],
    [1.45, -1.15],
    [-1.45, 1.15],
    [1.45, 1.15]
  ];
  for (let i = 0; i < positions.length; i++) {
    const [x, z] = positions[i];
    const post = shipMesh(postGeometry, scaffoldMaterial, group);
    post.position.set(x, 0.98, z);
    const foot = shipMesh(footGeometry, stripeMaterial, group);
    foot.position.set(x, 0.23, z);
  }

  const longBeamGeometry = new THREE.BoxGeometry(3.02, 0.11, 0.11);
  const sideBeamGeometry = new THREE.BoxGeometry(0.11, 0.11, 2.42);
  for (const z of [-1.15, 1.15]) {
    const beam = shipMesh(longBeamGeometry, scaffoldMaterial, group);
    beam.position.set(0, 1.72, z);
  }
  for (const x of [-1.45, 1.45]) {
    const beam = shipMesh(sideBeamGeometry, scaffoldMaterial, group);
    beam.position.set(x, 1.72, 0);
  }

  const plannedHull = shipMesh(new THREE.CylinderGeometry(0.72, 0.86, 2.5, 8), hologramMaterial, group);
  plannedHull.position.y = 1.18;
  plannedHull.rotation.x = Math.PI / 2;
  const plannedNose = shipMesh(new THREE.ConeGeometry(0.72, 1.1, 8), hologramMaterial, group);
  plannedNose.position.set(0, 1.18, 1.77);
  plannedNose.rotation.x = Math.PI / 2;
}

function buildCompleteShip(group: THREE.Group): void {
  const hullMaterial = new THREE.MeshStandardMaterial({ color: 0xe6d8bc, roughness: 0.56, metalness: 0.22, flatShading: true });
  const trimMaterial = new THREE.MeshStandardMaterial({ color: ACCENT_COLOR, roughness: 0.62, metalness: 0.15, flatShading: true });
  const darkMaterial = new THREE.MeshStandardMaterial({ color: 0x3f4b52, roughness: 0.7, metalness: 0.3, flatShading: true });
  const canopyMaterial = new THREE.MeshStandardMaterial({
    color: 0x86c4c7,
    emissive: 0x315e61,
    emissiveIntensity: 0.4,
    roughness: 0.25,
    metalness: 0.15,
    flatShading: true
  });
  const glowMaterial = new THREE.MeshBasicMaterial({ color: 0xffb06c, transparent: true, opacity: 0.9, depthWrite: false });

  const hull = shipMesh(new THREE.CylinderGeometry(0.72, 0.88, 2.5, 8), hullMaterial, group);
  hull.position.y = 1.45;
  hull.rotation.x = Math.PI / 2;
  const nose = shipMesh(new THREE.ConeGeometry(0.72, 1.15, 8), hullMaterial, group);
  nose.position.set(0, 1.45, 1.82);
  nose.rotation.x = Math.PI / 2;
  const belt = shipMesh(new THREE.TorusGeometry(0.8, 0.075, 5, 12), trimMaterial, group);
  belt.position.set(0, 1.45, -0.72);

  const cockpit = shipMesh(new THREE.DodecahedronGeometry(0.68, 0), canopyMaterial, group);
  cockpit.position.set(0, 2.02, 0.36);
  cockpit.scale.set(0.78, 0.62, 1.05);
  const cockpitBase = shipMesh(new THREE.BoxGeometry(1.02, 0.16, 1.05), darkMaterial, group);
  cockpitBase.position.set(0, 1.69, 0.28);

  const leftWing = shipMesh(wingGeometry(-1, 2.25, 1.75, 0.18), hullMaterial, group);
  leftWing.position.set(-0.52, 1.36, -0.1);
  const rightWing = shipMesh(wingGeometry(1, 2.25, 1.75, 0.18), hullMaterial, group);
  rightWing.position.set(0.52, 1.36, -0.1);
  const leftStripe = shipMesh(new THREE.BoxGeometry(0.82, 0.08, 0.18), trimMaterial, group);
  leftStripe.position.set(-1.35, 1.52, -0.25);
  leftStripe.rotation.y = 0.16;
  const rightStripe = shipMesh(leftStripe.geometry, trimMaterial, group);
  rightStripe.position.set(1.35, 1.52, -0.25);
  rightStripe.rotation.y = -0.16;

  const legGeometry = new THREE.CylinderGeometry(0.07, 0.09, 0.82, 6);
  const footGeometry = new THREE.BoxGeometry(0.42, 0.1, 0.34);
  const legs: ReadonlyArray<readonly [number, number]> = [
    [-0.66, -0.67],
    [0.66, -0.67],
    [-0.66, 0.58],
    [0.66, 0.58]
  ];
  for (let i = 0; i < legs.length; i++) {
    const [x, z] = legs[i];
    const leg = shipMesh(legGeometry, darkMaterial, group);
    leg.position.set(x, 0.7, z);
    leg.rotation.z = x < 0 ? -0.18 : 0.18;
    const foot = shipMesh(footGeometry, darkMaterial, group);
    foot.position.set(x * 1.08, 0.27, z);
  }

  const nozzle = shipMesh(new THREE.CylinderGeometry(0.53, 0.42, 0.5, 10, 1, true), darkMaterial, group);
  nozzle.position.set(0, 1.45, -1.48);
  nozzle.rotation.x = Math.PI / 2;
  const thrusterRim = shipMesh(new THREE.TorusGeometry(0.44, 0.09, 5, 12), trimMaterial, group);
  thrusterRim.position.set(0, 1.45, -1.75);
  const thrusterGlow = shipMesh(new THREE.CircleGeometry(0.36, 12), glowMaterial, group);
  thrusterGlow.position.set(0, 1.45, -1.765);
  thrusterGlow.rotation.y = Math.PI;
}

/** Build the landed ship state shown at the planet's construction clearing. */
export function buildSpaceship(crafted: boolean): THREE.Group {
  const group = new THREE.Group();
  group.name = crafted ? 'landed-spaceship' : 'spaceship-construction-site';
  if (crafted) buildCompleteShip(group);
  else buildConstructionSite(group);
  return group;
}
