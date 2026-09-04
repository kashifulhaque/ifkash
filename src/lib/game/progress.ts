// Found-wonder progress, persisted in localStorage. This module has no Three.js
// import so the home page can read progress without pulling in the game bundle.

/** Ids of the seven small wonders, in the order they appear in `wonders.ts`. */
export const WONDER_IDS = ['about', 'work', 'contact', 'resume', 'blog', 'projects', 'education'] as const;

export type WonderId = (typeof WONDER_IDS)[number];

export const WONDER_COUNT = WONDER_IDS.length;

const FOUND_KEY = 'planet_found';
/** Seed of the planet where the most recent wonder was found. */
const SEED_KEY = 'planet_found_seed';

export type Progress = {
  found: string[];
  /** Seed label of the planet where the last wonder was found, if any. */
  seed: string | null;
  complete: boolean;
};

export function loadFound(): string[] {
  try {
    const raw = localStorage.getItem(FOUND_KEY);
    if (!raw) return [];
    const ids = new Set<string>(WONDER_IDS);
    return (JSON.parse(raw) as string[]).filter((id) => ids.has(id));
  } catch {
    return [];
  }
}

/** Persist the found ids and, when given, the seed of the planet they were found on. */
export function saveFound(found: string[], seed?: string): void {
  try {
    localStorage.setItem(FOUND_KEY, JSON.stringify(found));
    if (seed) localStorage.setItem(SEED_KEY, seed);
  } catch {
    /* storage unavailable */
  }
}

/** Progress summary for the badge on the home page. Call from `onMount`, never during SSR. */
export function loadProgress(): Progress {
  const found = loadFound();
  let seed: string | null = null;
  try {
    seed = localStorage.getItem(SEED_KEY);
  } catch {
    /* storage unavailable */
  }
  return { found, seed, complete: found.length >= WONDER_COUNT };
}
