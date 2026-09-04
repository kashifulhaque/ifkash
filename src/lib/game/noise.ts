// Deterministic hash and value noise on R^3. The planet is generated on the
// client every load, so everything here must be pure: the same seed has to
// produce the same world on every visit and in every browser. That is why the
// hash uses integer arithmetic instead of `Math.sin`, whose last bits differ
// between JavaScript engines.

/** Hash an integer lattice point to [0, 1). */
export function hash3(x: number, y: number, z: number): number {
  let h = Math.imul(x | 0, 0x9e3779b1) ^ 0x85ebca6b;
  h = Math.imul(h ^ (y | 0), 0xc2b2ae35);
  h = Math.imul(h ^ (z | 0), 0x27d4eb2f);
  // murmur3 finaliser to spread the low bits.
  h ^= h >>> 16;
  h = Math.imul(h, 0x85ebca6b);
  h ^= h >>> 13;
  h = Math.imul(h, 0xc2b2ae35);
  h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}

function smooth(t: number): number {
  return t * t * (3 - 2 * t);
}

/** Trilinear value noise in [-1, 1]. */
export function valueNoise3(x: number, y: number, z: number): number {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const zi = Math.floor(z);
  const tx = smooth(x - xi);
  const ty = smooth(y - yi);
  const tz = smooth(z - zi);

  const c000 = hash3(xi, yi, zi);
  const c100 = hash3(xi + 1, yi, zi);
  const c010 = hash3(xi, yi + 1, zi);
  const c110 = hash3(xi + 1, yi + 1, zi);
  const c001 = hash3(xi, yi, zi + 1);
  const c101 = hash3(xi + 1, yi, zi + 1);
  const c011 = hash3(xi, yi + 1, zi + 1);
  const c111 = hash3(xi + 1, yi + 1, zi + 1);

  const x00 = c000 + (c100 - c000) * tx;
  const x10 = c010 + (c110 - c010) * tx;
  const x01 = c001 + (c101 - c001) * tx;
  const x11 = c011 + (c111 - c011) * tx;
  const y0 = x00 + (x10 - x00) * ty;
  const y1 = x01 + (x11 - x01) * ty;
  return (y0 + (y1 - y0) * tz) * 2 - 1;
}

// Seed-derived offset of the noise domain. Every fbm3 caller (biome borders,
// coastlines, relief) samples through this, so one seed reshapes the whole
// planet without each caller knowing about seeds.
const offset = { x: 0, y: 0, z: 0 };

/** Move the noise domain to a location derived from `seed`. Call before generating anything. */
export function setNoiseSeed(seed: number): void {
  const rng = seededRng(deriveSeed(seed, 1));
  offset.x = (rng() - 0.5) * 400;
  offset.y = (rng() - 0.5) * 400;
  offset.z = (rng() - 0.5) * 400;
}

/** Fractal Brownian motion over valueNoise3, roughly in [-1, 1]. */
export function fbm3(x: number, y: number, z: number, octaves = 4, lacunarity = 2, gain = 0.5): number {
  let sum = 0;
  let amp = 1;
  let norm = 0;
  let f = 1;
  x += offset.x;
  y += offset.y;
  z += offset.z;
  for (let i = 0; i < octaves; i++) {
    sum += valueNoise3(x * f + i * 17.3, y * f - i * 9.1, z * f + i * 4.7) * amp;
    norm += amp;
    amp *= gain;
    f *= lacunarity;
  }
  return sum / norm;
}

/** Small seeded PRNG (mulberry32) so prop placement is stable across loads. */
export function seededRng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * Independent sub-seed for one generator stage (`salt`), so the biome layout,
 * scatter, clouds, and stars each get their own stream from the world seed.
 */
export function deriveSeed(seed: number, salt: number): number {
  let h = Math.imul(seed ^ Math.imul(salt, 0x9e3779b1), 0x85ebca6b);
  h ^= h >>> 15;
  h = Math.imul(h, 0xc2b2ae35);
  h ^= h >>> 13;
  return h >>> 0;
}
