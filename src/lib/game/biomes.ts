import * as THREE from 'three';
import { deriveSeed, fbm3, seededRng } from './noise';

export type BiomeId = 'forest' | 'farm' | 'shore' | 'ember' | 'arctic' | 'ocean';

export type Biome = {
  id: BiomeId;
  /** Display name shown in the bottom-left caption. */
  name: string;
  /** Category label, for example FOREST or FARM. */
  kind: string;
  /** Two-digit ordinal shown next to the category. */
  index: string;
  tagline: string;
  /** Unit vector pointing at the biome's heart on the sphere. */
  center: THREE.Vector3;
  /** Ground palette: base colour plus a lighter and darker variant. */
  ground: [number, number, number];
  /** Coastline colour where the land meets the water. */
  beach: number;
};

function dir(x: number, y: number, z: number): THREE.Vector3 {
  return new THREE.Vector3(x, y, z).normalize();
}

export const BIOMES: Biome[] = [
  {
    id: 'forest',
    name: 'Fernwood',
    kind: 'FOREST',
    index: '01',
    tagline: 'Every great adventure starts with a small step.',
    center: dir(0.55, 0.3, 0.78),
    ground: [0x4d9c3f, 0x5cae4b, 0x3f8735],
    beach: 0xd8c27c
  },
  {
    id: 'farm',
    name: 'Clover Fields',
    kind: 'FARM',
    index: '02',
    tagline: 'A little care makes a world of difference.',
    center: dir(-0.72, 0.28, 0.6),
    ground: [0x8fc94a, 0x9fd55b, 0x7fb93f],
    beach: 0xe2cb84
  },
  {
    id: 'shore',
    name: 'Sunmark Shore',
    kind: 'COAST',
    index: '03',
    tagline: 'Warm sand remembers every footprint.',
    center: dir(0.82, -0.2, -0.5),
    ground: [0xe9b45c, 0xf2c26e, 0xd9a24c],
    beach: 0xf3d58a
  },
  {
    id: 'ember',
    name: 'Ember Heights',
    kind: 'VOLCANO',
    index: '04',
    tagline: 'Even the fiercest ground can grow a garden.',
    center: dir(-0.35, -0.55, -0.72),
    ground: [0x4f3628, 0x5c4030, 0x422c20],
    beach: 0x6b5340
  },
  {
    id: 'arctic',
    name: 'Northlight',
    kind: 'ARCTIC',
    index: '05',
    tagline: 'The quietest places hold the brightest wonders.',
    center: dir(0.05, 1, -0.05),
    ground: [0xe8f0f2, 0xf4f8f9, 0xd6e4e8],
    beach: 0xcfe3e8
  }
];

export const OCEAN: Biome = {
  id: 'ocean',
  name: 'The Shallows',
  kind: 'OCEAN',
  index: '00',
  tagline: 'Cold feet, warm heart.',
  center: dir(0, -1, 0),
  ground: [0x2b8ca3, 0x2f97ae, 0x267d92],
  beach: 0x2b8ca3
};

const LAND_BIOMES = BIOMES;

/** Uniformly random unit direction. */
function randomDir(rng: () => number, out: THREE.Vector3): THREE.Vector3 {
  const z = rng() * 2 - 1;
  const phi = rng() * Math.PI * 2;
  const r = Math.sqrt(1 - z * z);
  return out.set(r * Math.cos(phi), r * Math.sin(phi), z);
}

/**
 * Move the five land biome centres to seed-derived directions, in place, so
 * every module that reads `BIOMES[i].center` (wonders, landmarks, critters,
 * the aurora) follows. Centres are chosen by best-candidate sampling and then
 * pushed apart, which keeps them at least about 70 degrees from each other so
 * landmark clusters never overlap. Call before generating anything.
 */
export function layoutBiomes(seed: number): void {
  const rng = seededRng(deriveSeed(seed, 2));
  const centers = LAND_BIOMES.map((b) => b.center);
  const cand = new THREE.Vector3();
  for (let i = 0; i < centers.length; i++) {
    let bestScore = -1;
    const tries = i === 0 ? 1 : 48;
    for (let k = 0; k < tries; k++) {
      randomDir(rng, cand);
      let score = 0;
      if (i > 0) {
        score = Infinity;
        for (let j = 0; j < i; j++) score = Math.min(score, centers[j].angleTo(cand));
      }
      if (score > bestScore) {
        bestScore = score;
        centers[i].copy(cand);
      }
    }
  }
  // A few repulsion passes even out the spacing without erasing the layout.
  const force = new THREE.Vector3();
  for (let pass = 0; pass < 12; pass++) {
    for (const c of centers) {
      force.set(0, 0, 0);
      for (const o of centers) {
        if (o === c) continue;
        const ang = c.angleTo(o);
        force.addScaledVector(cand.copy(c).sub(o), 1 / (ang * ang));
      }
      c.addScaledVector(force, 0.02).normalize();
    }
  }
}

/** Angular Voronoi over the land biomes with noise-warped borders. */
export function landBiomeAt(d: THREE.Vector3): Biome {
  let best = LAND_BIOMES[0];
  let bestScore = Infinity;
  const warp = fbm3(d.x * 2.3 + 5, d.y * 2.3, d.z * 2.3 - 3, 3) * 0.22;
  for (const b of LAND_BIOMES) {
    // Angle to centre, warped so borders wobble instead of being straight arcs.
    const ang = Math.acos(THREE.MathUtils.clamp(b.center.dot(d), -1, 1));
    const score = ang + warp * (b.id === 'arctic' ? 0.6 : 1);
    if (score < bestScore) {
      bestScore = score;
      best = b;
    }
  }
  return best;
}

/** Angle (radians) from `d` to the nearest land biome centre. */
export function nearestCenterAngle(d: THREE.Vector3): number {
  let best = Math.PI;
  for (const b of LAND_BIOMES) {
    const ang = Math.acos(THREE.MathUtils.clamp(b.center.dot(d), -1, 1));
    if (ang < best) best = ang;
  }
  return best;
}

/**
 * Signed ocean field: negative values are sea, positive are land. Land is
 * forced near biome centres so landmarks always stay dry, and forced sea in a
 * belt around the south so the planet reads as islands rather than one blob.
 */
export function oceanField(d: THREE.Vector3): number {
  const n = fbm3(d.x * 1.7 + 11, d.y * 1.7 - 2, d.z * 1.7 + 7, 4);
  const keep = THREE.MathUtils.smoothstep(nearestCenterAngle(d), 0.42, 0.7); // 0 near centres
  // Southern belt bias: below the equator the sea wins more often.
  const south = THREE.MathUtils.smoothstep(-d.y, 0.1, 0.9) * 0.35;
  return n + 0.22 - (keep * 0.55 + south * keep);
}

export function isOcean(d: THREE.Vector3): boolean {
  return oceanField(d) < 0;
}

export function biomeAt(d: THREE.Vector3): Biome {
  return isOcean(d) ? OCEAN : landBiomeAt(d);
}

export function biomeById(id: BiomeId): Biome {
  return id === 'ocean' ? OCEAN : BIOMES.find((b) => b.id === id)!;
}
