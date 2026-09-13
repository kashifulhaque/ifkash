// Found-wonder progress, persisted in localStorage. This module has no Three.js
// import so the home page can read progress without pulling in the game bundle.

/** Ids of the seven small wonders, in the order they appear in `wonders.ts`. */
export const WONDER_IDS = ['about', 'work', 'contact', 'resume', 'blog', 'projects', 'education'] as const;

export type WonderId = (typeof WONDER_IDS)[number];

export const WONDER_COUNT = WONDER_IDS.length;

const FOUND_KEY = 'planet_found';
const LEGACY_SEED_KEY = 'planet_found_seed';
const STORE_VERSION = 2;

type PlanetProgress = {
  seed: string;
  depth: number;
  found: WonderId[];
};

type ProgressStore = {
  version: typeof STORE_VERSION;
  planets: PlanetProgress[];
};

export type Progress = {
  found: string[];
  seed: string | null;
  depth: number;
  complete: boolean;
};

function validFound(value: unknown): WonderId[] {
  if (!Array.isArray(value)) return [];
  return [
    ...new Set(value.filter((id): id is WonderId => typeof id === 'string' && WONDER_IDS.includes(id as WonderId)))
  ];
}

function validDepth(seed: string, value: unknown): number {
  if (seed === 'earth') return 0;
  return typeof value === 'number' && Number.isInteger(value) && value > 0 ? value : 1;
}

function readStore(): ProgressStore {
  try {
    const raw = localStorage.getItem(FOUND_KEY);
    if (!raw) return { version: STORE_VERSION, planets: [] };
    const parsed = JSON.parse(raw) as unknown;

    // Migrate the original single-planet array in memory. The next discovery
    // writes the versioned form and removes its separate seed key.
    if (Array.isArray(parsed)) {
      const seed = localStorage.getItem(LEGACY_SEED_KEY) || 'earth';
      return {
        version: STORE_VERSION,
        planets: [{ seed, depth: validDepth(seed, null), found: validFound(parsed) }]
      };
    }

    if (!parsed || typeof parsed !== 'object') return { version: STORE_VERSION, planets: [] };
    const record = parsed as Record<string, unknown>;
    if (record.version !== STORE_VERSION || !Array.isArray(record.planets)) return { version: STORE_VERSION, planets: [] };

    const planets = new Map<string, PlanetProgress>();
    for (const value of record.planets) {
      if (!value || typeof value !== 'object') continue;
      const candidate = value as Record<string, unknown>;
      if (typeof candidate.seed !== 'string' || candidate.seed.length === 0) continue;
      const progress = {
        seed: candidate.seed,
        depth: validDepth(candidate.seed, candidate.depth),
        found: validFound(candidate.found)
      };
      // Keep the latest occurrence and its insertion order.
      planets.delete(progress.seed);
      planets.set(progress.seed, progress);
    }
    return { version: STORE_VERSION, planets: [...planets.values()] };
  } catch {
    return { version: STORE_VERSION, planets: [] };
  }
}

function writeStore(store: ProgressStore): void {
  try {
    localStorage.setItem(FOUND_KEY, JSON.stringify(store));
    localStorage.removeItem(LEGACY_SEED_KEY);
  } catch {
    /* storage unavailable */
  }
}

/** Wonders found on one planet. */
export function loadFound(seed: string): string[] {
  return readStore().planets.find((planet) => planet.seed === seed)?.found ?? [];
}

/** Persist one planet's found wonders and its route depth. */
export function saveFound(found: string[], seed: string, depth: number): void {
  const store = readStore();
  const progress: PlanetProgress = {
    seed,
    depth: validDepth(seed, depth),
    found: validFound(found)
  };
  writeStore({
    version: STORE_VERSION,
    planets: [...store.planets.filter((planet) => planet.seed !== seed), progress]
  });
}

/** Most recently completed planet, used by the home-page badge. */
export function loadProgress(): Progress {
  const planets = readStore().planets;
  const complete = [...planets].reverse().find((planet) => planet.found.length >= WONDER_COUNT);
  return complete
    ? { found: complete.found, seed: complete.seed, depth: complete.depth, complete: true }
    : { found: [], seed: null, depth: 0, complete: false };
}
