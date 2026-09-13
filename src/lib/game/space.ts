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

export type PlanetProfile = {
  seed: string;
  name: string;
  kind: string;
  sky: number;
  water: number;
  /** Additive HSL hue rotation; Earth is zero. */
  hue: number;
  /** Multipliers over the familiar Earth terrain; Earth is one. */
  saturation: number;
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

type Archetype = {
  kind: string;
  sky: number;
  water: number;
  hue: readonly [number, number];
  saturation: readonly [number, number];
  relief: readonly [number, number];
  ruggedness: readonly [number, number];
};

const ARCHETYPES: readonly Archetype[] = [
  {
    kind: 'Garden world',
    sky: 0x172944,
    water: 0x286986,
    hue: [-0.03, 0.04],
    saturation: [0.92, 1.14],
    relief: [0.82, 1.08],
    ruggedness: [0.75, 1.12]
  },
  {
    kind: 'Ocean world',
    sky: 0x132d4c,
    water: 0x176d8a,
    hue: [0.08, 0.13],
    saturation: [1.02, 1.2],
    relief: [0.7, 0.9],
    ruggedness: [0.65, 0.82]
  },
  {
    kind: 'Desert world',
    sky: 0x39231d,
    water: 0x315c61,
    hue: [-0.12, -0.07],
    saturation: [0.92, 1.16],
    relief: [0.78, 1.08],
    ruggedness: [0.9, 1.24]
  },
  {
    kind: 'Frost world',
    sky: 0x18273e,
    water: 0x39788f,
    hue: [0.06, 0.12],
    saturation: [0.75, 0.92],
    relief: [1.02, 1.28],
    ruggedness: [0.95, 1.3]
  },
  {
    kind: 'Ember world',
    sky: 0x32191d,
    water: 0x49383e,
    hue: [-0.11, -0.06],
    saturation: [1.08, 1.2],
    relief: [1.08, 1.3],
    ruggedness: [1.16, 1.4]
  },
  {
    kind: 'Amethyst world',
    sky: 0x251c43,
    water: 0x453e78,
    hue: [0.1, 0.15],
    saturation: [1, 1.2],
    relief: [0.9, 1.18],
    ruggedness: [0.86, 1.22]
  },
  {
    kind: 'Moss world',
    sky: 0x192b2a,
    water: 0x285d59,
    hue: [-0.06, -0.01],
    saturation: [0.86, 1.08],
    relief: [0.84, 1.14],
    ruggedness: [0.8, 1.16]
  }
];

const NAME_FIRST = [
  'Aster', 'Brindle', 'Cinder', 'Dapple', 'Elara', 'Fable', 'Glimmer', 'Hollow',
  'Iris', 'Juniper', 'Luma', 'Mallow', 'Nimbus', 'Opal', 'Peregrine', 'Quill',
  'Ripple', 'Sorrel', 'Tansy', 'Umber', 'Vesper', 'Wren', 'Yarrow', 'Zephyr'
] as const;

const NAME_LAST = [
  'Bloom', 'Cairn', 'Cove', 'Drift', 'Fields', 'Haven', 'Isle', 'Light',
  'Mere', 'Reach', 'Rest', 'Ridge', 'Shore', 'Vale', 'Wilds', 'Wood'
] as const;

const SEED_ADJECTIVES = [
  'amber', 'brisk', 'calm', 'dusky', 'fern', 'gentle', 'hidden', 'ivory',
  'lucky', 'misty', 'quiet', 'rosy', 'silver', 'tiny', 'velvet', 'windy'
] as const;

const SEED_NOUNS = [
  'beacon', 'comet', 'cove', 'dune', 'finch', 'harbour', 'lantern', 'meadow',
  'moon', 'orchard', 'pebble', 'reef', 'sparrow', 'thistle', 'willow', 'wren'
] as const;

const earthProfile: PlanetProfile = {
  seed: EARTH_SEED,
  name: 'Earth',
  kind: 'Homeworld',
  sky: 0x16243f,
  water: 0x256a8c,
  hue: 0,
  saturation: 1,
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
    fuel: Math.min(MAX_FUEL, Math.max(0, rawFuel))
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
    sky: archetype.sky,
    water: archetype.water,
    hue: between(archetype.hue, rng),
    saturation: between(archetype.saturation, rng),
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
