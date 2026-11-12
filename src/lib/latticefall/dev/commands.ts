// The dev commands: teleport, god mode, set level, freeze. They call the `dev_*` exports
// that only a `dev` build of the WASM module has (game/crates/wasm/src/debug.rs). The
// renderer writes to the simulation through nothing else.
//
// Build the module with `LF_WASM_FEATURES=dev bash game/scripts/build-wasm.sh`.

import { findSpawn } from './spawns';

/** The dev exports of `LfWorld`, all optional because release builds don't have them. */
export interface DevWorld {
  dev_teleport?(x: number, y: number, z: number, yaw: number): boolean;
  dev_set_god?(on: boolean): void;
  dev_god?(): boolean;
  dev_set_level?(level: number): number;
  dev_level?(): number;
  raycast(origin: Float32Array, dir: Float32Array, max: number): number | undefined;
  state_hash(): string;
}

export interface DevCommandsDeps {
  world: DevWorld;
  scenario: string;
  /** The scenario's own start, where `spawn=spawn` goes. */
  start: { x: number; y: number; z: number; yaw: number } | null;
  /** Called after a command changed the world, so the host re-reads the snapshot. */
  refresh(): void;
  setPaused(paused: boolean): void;
}

// A ray through exactly a heightfield vertex can miss, so sample a little off the grid.
const JITTER_X = 0.0123;
const JITTER_Z = 0.0071;
const RAY_TOP = 80;

export class DevCommands {
  readonly available: boolean;
  private deps: DevCommandsDeps;
  private origin = new Float32Array(3);
  private down = new Float32Array([0, -1, 0]);
  private frozen = false;

  constructor(deps: DevCommandsDeps) {
    this.deps = deps;
    this.available = typeof deps.world.dev_teleport === 'function';
  }

  /** The ground height under (x, z), or null if the ray misses. */
  groundY(x: number, z: number): number | null {
    this.origin[0] = x + JITTER_X;
    this.origin[1] = RAY_TOP;
    this.origin[2] = z + JITTER_Z;
    const hit = this.deps.world.raycast(this.origin, this.down, RAY_TOP * 2);
    return hit === undefined ? null : RAY_TOP - hit;
  }

  /** Moves the player to the named spawn point (or `spawn`, the scenario's start). Returns false if it can't. */
  teleport(id: string): boolean {
    const { world, scenario, start } = this.deps;
    if (!world.dev_teleport) return false;
    if (id === 'spawn') {
      if (!start || !world.dev_teleport(start.x, start.y, start.z, start.yaw)) return false;
    } else {
      const point = findSpawn(scenario, id);
      if (!point) return false;
      const y = point.y ?? this.groundY(point.x, point.z);
      if (y === null || !world.dev_teleport(point.x, y, point.z, point.yaw)) return false;
    }
    this.deps.refresh();
    return true;
  }

  setGod(on: boolean): void {
    this.deps.world.dev_set_god?.(on);
  }

  god(): boolean {
    return this.deps.world.dev_god?.() ?? false;
  }

  /** Records the level the dev tools want. The core has no character level yet, so nothing reads it. */
  setLevel(level: number): number | null {
    return this.deps.world.dev_set_level?.(Math.max(0, Math.floor(level))) ?? null;
  }

  level(): number | null {
    return this.deps.world.dev_level?.() ?? null;
  }

  setFrozen(frozen: boolean): void {
    this.frozen = frozen;
    this.deps.setPaused(frozen);
  }

  get isFrozen(): boolean {
    return this.frozen;
  }
}
