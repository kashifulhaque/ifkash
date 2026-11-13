// The performance budgets of docs/game/01-architecture.md ("Performance budgets" and "Bundle
// and loading"), as numbers the benchmark and the dev tools can check. No Three.js import.

export const BUDGETS = {
  /** Milliseconds of work per frame: 60 fps on a 2021 integrated-graphics laptop at 1080p. */
  frameMs: 16.7,
  /** Draw calls per frame, shadow passes included (that is what `renderer.info` counts). */
  drawCalls: 250,
  /** Triangles per frame, shadow passes included. */
  triangles: 600_000,
  /** Actors drawn at once (the activity cells keep this down). */
  activeActors: 40,
  /** Whole-game download, bytes: JS, WASM, and every static/latticefall asset. */
  totalDownloadBytes: 60 * 1024 * 1024,
  /** First region's download, bytes. */
  firstRegionBytes: 8 * 1024 * 1024,
  /** Gzipped WASM, bytes. */
  wasmGzipBytes: 1.5 * 1024 * 1024,
  /** Gzipped initial JavaScript, bytes. */
  initialJsGzipBytes: 400 * 1024
} as const;

/** GPU texture memory the world may hold at once, bytes (a guess for an integrated GPU). */
export const TEXTURE_BUDGET_BYTES = 96 * 1024 * 1024;

/** The longest edge any texture may have per quality preset, in pixels. */
export const TEXTURE_MAX_EDGE = { low: 512, medium: 1024, high: 2048 } as const;

export interface FrameSample {
  /** Work per frame in milliseconds: JS plus the GPU, waited out with `gl.finish()`. */
  workMs: number;
  calls: number;
  triangles: number;
}

export interface Verdict {
  ok: boolean;
  /** Which budgets were broken, for the report. */
  broken: string[];
}

export function percentile(sorted: readonly number[], p: number): number {
  if (sorted.length === 0) return 0;
  const i = Math.min(sorted.length - 1, Math.max(0, Math.ceil(p * sorted.length) - 1));
  return sorted[i];
}

export interface Stats {
  mean: number;
  p95: number;
  max: number;
}

export function stats(values: readonly number[]): Stats {
  if (values.length === 0) return { mean: 0, p95: 0, max: 0 };
  const sorted = [...values].sort((a, b) => a - b);
  const sum = values.reduce((a, b) => a + b, 0);
  return { mean: sum / values.length, p95: percentile(sorted, 0.95), max: sorted[sorted.length - 1] };
}

/**
 * Checks a run of frames against the budgets. Draw calls and triangles are held to their
 * worst frame; frame time to its 95th percentile (a lone GC pause is not a budget miss).
 */
export function judge(samples: readonly FrameSample[]): Verdict {
  const broken: string[] = [];
  const work = stats(samples.map((s) => s.workMs));
  const calls = stats(samples.map((s) => s.calls));
  const tris = stats(samples.map((s) => s.triangles));
  if (work.p95 > BUDGETS.frameMs) broken.push(`frame p95 ${work.p95.toFixed(1)} ms > ${BUDGETS.frameMs}`);
  if (calls.max > BUDGETS.drawCalls) broken.push(`draw calls max ${calls.max} > ${BUDGETS.drawCalls}`);
  if (tris.max > BUDGETS.triangles) broken.push(`triangles max ${tris.max} > ${BUDGETS.triangles}`);
  return { ok: broken.length === 0, broken };
}
