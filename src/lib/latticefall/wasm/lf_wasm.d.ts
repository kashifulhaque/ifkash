/* tslint:disable */
/* eslint-disable */

export class LfWorld {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * The world state hash as 16 lowercase hex digits.
   */
  state_hash(): string;
  /**
   * Ticks simulated so far.
   */
  tick_count(): number;
  /**
   * Drains queued events as a `Uint32Array`. The encoding will be `[kind, payload...]`
   * per event once the first `Event` variants land; the queue is always empty today.
   */
  drain_events(): Uint32Array;
  /**
   * Number of `ActorSnap` records (not bytes); the stride is `snapshot_stride()`.
   */
  snapshot_len(): number;
  /**
   * Byte offset of the first `ActorSnap` in WASM memory. Valid until the next `tick`.
   */
  snapshot_ptr(): number;
  /**
   * Advances one tick per 4-byte `ActionFrame` in `frames` (`[move_x, move_z, buttons_lo,
   * buttons_hi]`), then rewrites the snapshot. Trailing bytes short of a full frame are
   * ignored. Returns the number of ticks run.
   */
  tick(frames: Uint8Array): number;
  /**
   * Ray query against the world. Stub until WP-010 adds physics: always `undefined`.
   * On a hit it will return the distance along the ray.
   */
  raycast(origin: Float32Array, dir: Float32Array, max: number): number | undefined;
  /**
   * Starts a fresh world. `scenario` is reserved: only the flat M0 scenario exists so far.
   */
  constructor(seed: bigint, scenario: string);
}

/**
 * The content hash as 16 lowercase hex digits, matching `lf-tools validate`.
 */
export function content_hash(): string;

/**
 * Runs a `.lfr` replay from bytes and returns its final state hash as 16 hex digits.
 * Throws with the mismatch message if any recorded checkpoint differs.
 */
export function run_replay(bytes: Uint8Array): string;

/**
 * Bytes between consecutive snapshot records (`size_of::<ActorSnap>()`).
 */
export function snapshot_stride(): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_lfworld_free: (a: number, b: number) => void;
  readonly content_hash: () => [number, number, number, number];
  readonly lfworld_drain_events: (a: number) => [number, number];
  readonly lfworld_new_world: (a: bigint, b: number, c: number) => number;
  readonly lfworld_raycast: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly lfworld_snapshot_len: (a: number) => number;
  readonly lfworld_snapshot_ptr: (a: number) => number;
  readonly lfworld_state_hash: (a: number) => [number, number];
  readonly lfworld_tick: (a: number, b: number, c: number) => number;
  readonly lfworld_tick_count: (a: number) => number;
  readonly run_replay: (a: number, b: number) => [number, number, number, number];
  readonly snapshot_stride: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
