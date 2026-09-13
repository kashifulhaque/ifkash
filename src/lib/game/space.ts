// Shared, Three.js-free state and generation for travel between planets.

import { seededRng } from './noise';
import { hashSeed } from './seed';

const SHIP_KEY = 'planet_ship';

export const EARTH_SEED = 'earth';

/** Parts are identified by their stable index in this list. */
export const SHIP_PARTS = [
  'Hull Plating',
  'Cockpit Canopy',
  'Port Wing',
  'Starboard Wing',
  'Thruster Core'
] as const;

export const MAX_FUEL = 100;
export const SHARD_FUEL = 10;

export type ShipProgress = {
  parts: number[];
  crafted: boolean;
  fuel: number;
};

export type SpaceDestination = {
  seed: string;
  name: string;
  kind: string;
  depth: number;
  fuelCost: number;
  color: number;
};

export type PlanetArchetype = 'earth' | 'bloom' | 'reef' | 'crystal' | 'rime' | 'ember' | 'fracture' | 'spore';

export type PlanetProfile = {
  seed: string;
  name: string;
  kind: string;
  archetype: PlanetArchetype;
  sky: number;
  water: number;
  terrain: readonly [number, number, number];
  shore: number;
  accent: number;
  cloud: number;
  cloudCount: number;
  /** Additive bias to the ocean field. Positive values expose more land. */
  landBias: number;
  /** Multipliers over Earth's terrain amplitudes. */
  relief: number;
  ruggedness: number;
};

export type SpaceStatus = {
  parts: number;
  totalParts: number;
  crafted: boolean;
  fuel: number;
  maxFuel: number;
  planetName: string;
  planetKind: string;
  depth: number;
  isEarth: boolean;
};

type AlienArchetype = Exclude<PlanetArchetype, 'earth'>;

type Archetype = {
  id: AlienArchetype;
  kind: string;
  sky: number;
  water: number;
  terrain: readonly [number, number, number];
  shore: number;
  accent: number;
  cloud: number;
  cloudCount: readonly [number, number];
  landBias: readonly [number, number];
  relief: readonly [number, number];
  ruggedness: readonly [number, number];
};

/** Deliberately strong silhouettes and palettes: every destination is a different physical idea, not recoloured Earth. */
const ARCHETYPES: readonly Archetype[] = [
  {
    id: 'bloom',
    kind: 'Bioluminescent canopy',
    sky: 0x071d2b,
    water: 0x123f55,
    terrain: [0x163f4b, 0x235b61, 0x3a7168],
    shore: 0x57a88f,
    accent: 0x69ffe0,
    cloud: 0x74c8ba,
    cloudCount: [7, 11],
    landBias: [0.02, 0.15],
    relief: [0.9, 1.14],
    ruggedness: [0.78, 1.02]
  },
  {
    id: 'reef',
    kind: 'Pelagic reef',
    sky: 0x07172e,
    water: 0x075b78,
    terrain: [0x174058, 0x1c7180, 0x3c9b91],
    shore: 0x62d6b3,
    accent: 0xff8bd7,
    cloud: 0x80dfe4,
    cloudCount: [16, 23],
    landBias: [-0.3, -0.18],
    relief: [0.55, 0.76],
    ruggedness: [0.48, 0.7]
  },
  {
    id: 'crystal',
    kind: 'Glass desert',
    sky: 0x24122f,
    water: 0x493255,
    terrain: [0x6d365f, 0x9a4f74, 0xd18492],
    shore: 0xf2b59b,
    accent: 0xffd38a,
    cloud: 0xe4a6c8,
    cloudCount: [2, 5],
    landBias: [0.18, 0.3],
    relief: [0.86, 1.08],
    ruggedness: [1.02, 1.3]
  },
  {
    id: 'rime',
    kind: 'Cometary ice hollow',
    sky: 0x07182d,
    water: 0x295c7b,
    terrain: [0x8eb5c5, 0xc4dce2, 0xe8f3ed],
    shore: 0xa7edf0,
    accent: 0x9dfcff,
    cloud: 0xb8e6ee,
    cloudCount: [10, 16],
    landBias: [0.08, 0.2],
    relief: [1.04, 1.3],
    ruggedness: [1.2, 1.5]
  },
  {
    id: 'ember',
    kind: 'Volcanic crucible',
    sky: 0x240b0b,
    water: 0xe34719,
    terrain: [0x211b24, 0x3b2830, 0x6a332c],
    shore: 0xff8a2a,
    accent: 0xffc247,
    cloud: 0x6f3530,
    cloudCount: [18, 25],
    landBias: [0.2, 0.34],
    relief: [1.14, 1.4],
    ruggedness: [1.28, 1.58]
  },
  {
    id: 'fracture',
    kind: 'Fractured gravity moon',
    sky: 0x100d2b,
    water: 0x211d4f,
    terrain: [0x302d60, 0x514682, 0x7564a0],
    shore: 0x9e86cb,
    accent: 0xbba5ff,
    cloud: 0x7d73b2,
    cloudCount: [0, 3],
    landBias: [0.28, 0.4],
    relief: [1.18, 1.48],
    ruggedness: [1.32, 1.6]
  },
  {
    id: 'spore',
    kind: 'Mycelial dreamworld',
    sky: 0x101d22,
    water: 0x283b43,
    terrain: [0x334936, 0x5a6041, 0x7d7553],
    shore: 0xa09a68,
    accent: 0xc58cff,
    cloud: 0x9b91b6,
    cloudCount: [8, 14],
    landBias: [-0.04, 0.1],
    relief: [0.78, 1.02],
    ruggedness: [0.7, 0.94]
  }
];

const NAME_FIRST = [
  'Aevra', 'Caelum', 'Ceryx', 'Eidolon', 'Ilyra', 'Khepri', 'Lacuna', 'Myrr',
  'Nacre', 'Ossia', 'Penum', 'Qiroth', 'Rhyme', 'Serein', 'Thessa', 'Umbriel',
  'Vael', 'Xanthe', 'Ysil', 'Zephra'
] as const;

const NAME_LAST = [
  'Atoll', 'Breach', 'Calyx', 'Cradle', 'Crucible', 'Drift', 'Echo', 'Halo',
  'Hollow', 'Maw', 'Reach', 'Remnant', 'Rift', 'Shroud', 'Spindle', 'Veil'
] as const;

const SEED_ADJECTIVES = [
  'ashen', 'auroral', 'benthic', 'broken', 'echoing', 'glass', 'hollow', 'luminous',
  'orbital', 'riven', 'silent', 'tidal', 'umbra', 'verdant', 'violet', 'wandering'
] as const;

const SEED_NOUNS = [
  'calyx', 'comet', 'coral', 'ember', 'halo', 'islet', 'lantern', 'maw',
  'monolith', 'prism', 'reef', 'rift', 'signal', 'spore', 'vault', 'whorl'
] as const;

const earthProfile: PlanetProfile = {
  seed: EARTH_SEED,
  name: 'Earth',
  kind: 'Homeworld',
  archetype: 'earth',
  sky: 0x16243f,
  water: 0x256a8c,
  terrain: [0, 0, 0],
  shore: 0,
  accent: 0x9fe8ff,
  cloud: 0xf6f8fa,
  cloudCount: 14,
  landBias: 0,
  relief: 1,
  ruggedness: 1
};

const pick = <T>(values: readonly T[], rng: () => number): T => values[Math.floor(rng() * values.length)]!;
const between = (range: readonly [number, number], rng: () => number): number => range[0] + (range[1] - range[0]) * rng();

function validShipProgress(value: unknown): ShipProgress {
  const record = value !== null && typeof value === 'object' ? (value as Record<string, unknown>) : {};
  const parts = Array.isArray(record.parts)
    ? [...new Set(record.parts.filter((part): part is number => Number.isInteger(part) && part >= 0 && part < SHIP_PARTS.length))].sort(
        (a, b) => a - b
      )
    : [];
  const allPartsFound = parts.length === SHIP_PARTS.length;
  const rawFuel = typeof record.fuel === 'number' && Number.isFinite(record.fuel) ? record.fuel : 0;

  return {
    parts,
    crafted: record.crafted === true && allPartsFound,
    fuel: Math.floor(Math.min(MAX_FUEL, Math.max(0, rawFuel)))
  };
}

/** Read and validate global ship progress. Invalid or unavailable storage is treated as a fresh ship. */
export function loadShipProgress(): ShipProgress {
  try {
    const raw = localStorage.getItem(SHIP_KEY);
    return raw ? validShipProgress(JSON.parse(raw)) : validShipProgress(null);
  } catch {
    return validShipProgress(null);
  }
}

/** Persist a validated snapshot; private browsing and storage quotas are harmless. */
export function saveShipProgress(progress: ShipProgress): void {
  try {
    localStorage.setItem(SHIP_KEY, JSON.stringify(validShipProgress(progress)));
  } catch {
    /* storage unavailable */
  }
}

/** Stable visual and display parameters for a planet seed. */
export function planetProfile(seed: string): PlanetProfile {
  if (seed === EARTH_SEED) return { ...earthProfile };

  const rng = seededRng(hashSeed(`planet-profile:${seed}`));
  const archetype = pick(ARCHETYPES, rng);
  return {
    seed,
    name: `${pick(NAME_FIRST, rng)} ${pick(NAME_LAST, rng)}`,
    kind: archetype.kind,
    archetype: archetype.id,
    sky: archetype.sky,
    water: archetype.water,
    terrain: archetype.terrain,
    shore: archetype.shore,
    accent: archetype.accent,
    cloud: archetype.cloud,
    cloudCount: Math.round(between(archetype.cloudCount, rng)),
    landBias: between(archetype.landBias, rng),
    relief: between(archetype.relief, rng),
    ruggedness: between(archetype.ruggedness, rng)
  };
}

/** Three reproducible onward routes. Including depth in the hash keeps the graph endless. */
export function spaceDestinations(seed: string, depth: number): SpaceDestination[] {
  const nextDepth = depth + 1;
  const destinations: SpaceDestination[] = [];

  for (let i = 0; i < 3; i++) {
    const routeHash = hashSeed(`space-route:${seed}:${nextDepth}:${i}`);
    const rng = seededRng(routeHash);
    const label = `${pick(SEED_ADJECTIVES, rng)}-${pick(SEED_NOUNS, rng)}-${routeHash.toString(36).padStart(7, '0')}-${i + 1}`;
    const profile = planetProfile(label);
    destinations.push({
      seed: label,
      name: profile.name,
      kind: profile.kind,
      depth: nextDepth,
      fuelCost: 20 + Math.floor(rng() * 21),
      color: profile.water
    });
  }

  return destinations;
}
