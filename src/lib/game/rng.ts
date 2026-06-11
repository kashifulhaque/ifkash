// Deterministic per-chunk random numbers so the infinite world is stable.
export function chunkRng(cx: number, cz: number, seed = 1337): () => number {
  let h = seed ^ (cx * 374761393) ^ (cz * 668265263);
  h = (h ^ (h >>> 13)) * 1274126177;
  h ^= h >>> 16;
  let state = h >>> 0 || 1;
  return () => {
    // xorshift32
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    state >>>= 0;
    return state / 4294967296;
  };
}
