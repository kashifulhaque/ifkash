// Loads the lf-wasm module. Only ever import this file dynamically, from the game route,
// so the WASM glue stays out of every other bundle.

import type * as Glue from '../wasm/lf_wasm';

export interface LfWasm {
  glue: typeof Glue;
  /** The module's linear memory. Its `.buffer` is replaced whenever memory grows. */
  memory: WebAssembly.Memory;
}

let cached: Promise<LfWasm> | null = null;

/** Fetches and instantiates the WASM module once; later calls reuse it. */
export function loadWasm(): Promise<LfWasm> {
  cached ??= (async () => {
    const glue = await import('../wasm/lf_wasm.js');
    const exports = await glue.default();
    return { glue, memory: exports.memory };
  })().catch((err) => {
    cached = null;
    throw err;
  });
  return cached;
}
