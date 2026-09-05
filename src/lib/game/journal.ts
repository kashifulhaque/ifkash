// The traveller's journal: everything the explorer has seen and done across
// every planet, persisted in localStorage. This module has no Three.js import,
// so pages outside the game can read it without pulling in the game bundle.

import type { BiomeId } from './biomes';

const JOURNAL_KEY = 'planet_journal';
const SHARDS_KEY = 'planet_shards';

/** Land biomes plus the sea, in the order the journal lists them. */
export const JOURNAL_BIOMES: BiomeId[] = ['forest', 'farm', 'shore', 'ember', 'arctic', 'desert', 'marsh', 'grove', 'ocean'];

/** Every animal on the planet, by the name shown in the pet prompt. */
export const SPECIES = [
  'sheep',
  'goat',
  'polar bear',
  'penguin',
  'fox',
  'rabbit',
  'deer',
  'crab',
  'turtle',
  'duck',
  'frog',
  'fennec fox',
  'jackrabbit'
] as const;

export type Species = (typeof SPECIES)[number];

/** Starlight shards scattered over each planet. */
export const SHARD_COUNT = 12;

export type MilestoneId =
  | 'sailor'
  | 'stargazer'
  | 'friend'
  | 'zoologist'
  | 'cartographer'
  | 'wanderer'
  | 'grasshopper'
  | 'starfall'
  | 'wonders'
  | 'traveller';

export type Milestone = { id: MilestoneId; title: string; blurb: string };

export const MILESTONES: Milestone[] = [
  { id: 'wonders', title: 'Every small wonder', blurb: 'Found all seven wonders on one planet.' },
  { id: 'cartographer', title: 'Cartographer', blurb: 'Set foot in every biome, and the sea between them.' },
  { id: 'zoologist', title: 'Friend to all', blurb: 'Petted every kind of animal on the planet.' },
  { id: 'friend', title: 'A friend for the road', blurb: 'An animal liked you enough to follow you.' },
  { id: 'sailor', title: 'Sailor', blurb: 'Launched a boat and left the shore behind.' },
  { id: 'stargazer', title: 'Stargazer', blurb: 'Caught a shooting star crossing the sky.' },
  { id: 'starfall', title: 'Starfall', blurb: 'Gathered every starlight shard on one planet.' },
  { id: 'wanderer', title: 'Wanderer', blurb: 'Walked a thousand paces.' },
  { id: 'grasshopper', title: 'Grasshopper', blurb: 'Hopped a hundred times.' },
  { id: 'traveller', title: 'Traveller', blurb: 'Visited three different planets.' }
];

export type Journal = {
  /** Biomes the explorer has stood in, across every planet. */
  biomes: string[];
  /** Animal species petted at least once. */
  species: string[];
  milestones: MilestoneId[];
  /** Surface units walked on foot. */
  paces: number;
  hops: number;
  /** Seed labels of the planets visited. */
  planets: string[];
  /** Shards collected, summed over every planet. */
  shards: number;
};

export const emptyJournal = (): Journal => ({
  biomes: [],
  species: [],
  milestones: [],
  paces: 0,
  hops: 0,
  planets: [],
  shards: 0
});

const strings = (v: unknown): string[] => (Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : []);
const num = (v: unknown): number => (typeof v === 'number' && Number.isFinite(v) ? v : 0);

export function loadJournal(): Journal {
  try {
    const raw = localStorage.getItem(JOURNAL_KEY);
    if (!raw) return emptyJournal();
    const j = JSON.parse(raw) as Partial<Journal>;
    const ids = new Set<string>(MILESTONES.map((m) => m.id));
    return {
      biomes: strings(j.biomes),
      species: strings(j.species),
      milestones: strings(j.milestones).filter((id): id is MilestoneId => ids.has(id)),
      paces: num(j.paces),
      hops: num(j.hops),
      planets: strings(j.planets),
      shards: num(j.shards)
    };
  } catch {
    return emptyJournal();
  }
}

export function saveJournal(j: Journal): void {
  try {
    localStorage.setItem(JOURNAL_KEY, JSON.stringify(j));
  } catch {
    /* storage unavailable */
  }
}

/** Indices of the shards already collected on the planet with this seed. */
export function loadShards(seed: string): number[] {
  try {
    const raw = localStorage.getItem(`${SHARDS_KEY}:${seed}`);
    if (!raw) return [];
    return (JSON.parse(raw) as unknown[]).filter((x): x is number => typeof x === 'number' && x >= 0 && x < SHARD_COUNT);
  } catch {
    return [];
  }
}

export function saveShards(seed: string, found: number[]): void {
  try {
    localStorage.setItem(`${SHARDS_KEY}:${seed}`, JSON.stringify(found));
  } catch {
    /* storage unavailable */
  }
}

export function milestoneById(id: MilestoneId): Milestone {
  return MILESTONES.find((m) => m.id === id)!;
}
