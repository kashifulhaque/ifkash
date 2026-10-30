// Pure numeric helpers for Token Lens. No I/O and no model imports, so they can
// run anywhere and be reasoned about on their own.

const LN2 = Math.LN2;

export interface SoftmaxStats {
  /** Largest logit, used to keep exp() stable. */
  max: number;
  /** Natural log of the softmax denominator, relative to `max`: ln(sum(exp(x - max))). */
  logZ: number;
  /** Shannon entropy of softmax(x / temperature), in bits. */
  entropyBits: number;
}

/**
 * One pass over the logits to get what softmax needs: the max, the log of the
 * normalizer, and the entropy. Entropy uses H = lnZ - sum(p * z) with
 * z = (x - max) / T, so no probability array is allocated.
 */
export function softmaxStats(logits: Float32Array, temperature = 1, offset = 0, length = logits.length - offset): SoftmaxStats {
  const end = offset + length;
  let max = -Infinity;
  for (let i = offset; i < end; i++) if (logits[i] > max) max = logits[i];

  const inv = 1 / temperature;
  let sum = 0;
  let weighted = 0; // sum(e^z * z)
  for (let i = offset; i < end; i++) {
    const z = (logits[i] - max) * inv;
    if (z === -Infinity) continue;
    const e = Math.exp(z);
    sum += e;
    weighted += e * z;
  }
  const logZ = Math.log(sum);
  const entropyNats = logZ - weighted / sum;
  return { max, logZ, entropyBits: Math.max(0, entropyNats / LN2) };
}

/** Probability of a single logit under softmax at the given temperature. */
export function probOf(logit: number, stats: SoftmaxStats, temperature = 1): number {
  return Math.exp((logit - stats.max) / temperature - stats.logZ);
}

/**
 * Indices of the k largest logits, sorted by descending value. Keeps a small
 * sorted buffer and rejects most entries with one comparison, so it is O(V) in
 * the common case and O(V * k) at worst. Never sorts the whole vocabulary.
 */
export function topKIndices(logits: Float32Array, k: number, offset = 0, length = logits.length - offset): number[] {
  const end = offset + length;
  const kk = Math.max(0, Math.min(k, length));
  const idx: number[] = [];
  const val: number[] = [];
  if (kk === 0) return idx;

  for (let i = offset; i < end; i++) {
    const v = logits[i];
    if (idx.length === kk && v <= val[kk - 1]) continue;
    // Insertion point in the descending buffer.
    let j = idx.length < kk ? idx.length : kk - 1;
    while (j > 0 && val[j - 1] < v) {
      if (j < kk) {
        idx[j] = idx[j - 1];
        val[j] = val[j - 1];
      }
      j--;
    }
    idx[j] = i - offset;
    val[j] = v;
  }
  return idx;
}

/** Softmax top-k at temperature 1: ids and probabilities, descending. */
export function topKProbs(logits: Float32Array, k: number, stats: SoftmaxStats, offset = 0, length = logits.length - offset): { id: number; p: number }[] {
  return topKIndices(logits, k, offset, length).map((id) => ({
    id,
    p: probOf(logits[offset + id], stats),
  }));
}

/**
 * Sample an index from softmax(x / temperature) restricted to the top-p nucleus.
 * Grows a top-k pool (64, 256, 1024, then the whole vocabulary) until it covers
 * `topP` of the mass, so the usual case never sorts 150k entries.
 * `rand` is injectable for tests and returns a value in [0, 1).
 */
export function sampleTopP(
  logits: Float32Array,
  temperature: number,
  topP: number,
  rand: () => number = Math.random,
  offset = 0,
  length = logits.length - offset,
): number {
  const stats = softmaxStats(logits, temperature, offset, length);
  let pool: number[] = [];
  let probs: number[] = [];
  let cut = 0;
  for (const k of [64, 256, 1024, length]) {
    pool = topKIndices(logits, k, offset, length);
    probs = pool.map((id) => probOf(logits[offset + id], stats, temperature));
    let cum = 0;
    cut = probs.length;
    for (let i = 0; i < probs.length; i++) {
      cum += probs[i];
      if (cum >= topP) {
        cut = i + 1;
        break;
      }
    }
    if (cum >= topP || pool.length >= length) break;
  }
  let total = 0;
  for (let i = 0; i < cut; i++) total += probs[i];
  let r = rand() * total;
  for (let i = 0; i < cut; i++) {
    r -= probs[i];
    if (r <= 0) return pool[i];
  }
  return pool[cut - 1];
}
