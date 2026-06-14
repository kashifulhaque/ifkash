// A seeded xorshift32 stream in [0, 1). Shared by the chunk RNG and the daily
// challenge so both are deterministic from a single integer seed.
export function seededRng(seed: number): () => number {
  let state = seed >>> 0 || 1;
  return () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    state >>>= 0;
    return state / 4294967296;
  };
}

// Deterministic per-chunk random numbers so the infinite world is stable.
export function chunkRng(cx: number, cz: number, seed = 1337): () => number {
  let h = seed ^ (cx * 374761393) ^ (cz * 668265263);
  h = (h ^ (h >>> 13)) * 1274126177;
  h ^= h >>> 16;
  return seededRng(h);
}

/** Hash an arbitrary string into a 32-bit seed (FNV-1a). */
export function hashSeed(str: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

/** Local YYYY-MM-DD string for a date (defaults to today). */
export function dayString(date = new Date()): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

/** Deterministic seed for a given day, used for the daily challenge. */
export function dailySeed(date = new Date()): number {
  return hashSeed('daily:' + dayString(date));
}
