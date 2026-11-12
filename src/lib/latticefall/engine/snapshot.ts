// Reads the core's `ActorSnap` array straight out of WASM memory (layout in
// game/crates/core/src/snapshot.rs) and keeps the previous and current copies for
// interpolation.

import type { LfWasm } from './loader';

/** Bytes per record. Matches `snapshot_stride()` from the module. */
export const SNAP_STRIDE = 40;

export interface ActorSnap {
  id: number;
  kind: number;
  flags: number;
  x: number;
  y: number;
  z: number;
  yaw: number;
  moveId: number;
  moveFrame: number;
  hpFrac: number;
  poiseFrac: number;
  statusMask: number;
}

/** What the renderer draws: a snapshot pair blended at some alpha. */
export type DrawActor = ActorSnap;

const TAU = Math.PI * 2;

/** Interpolates two angles along the shortest arc. */
export function lerpAngle(a: number, b: number, t: number): number {
  let d = (b - a) % TAU;
  if (d > Math.PI) d -= TAU;
  else if (d < -Math.PI) d += TAU;
  return a + d * t;
}

export class SnapshotBuffer {
  prev: ActorSnap[] = [];
  curr: ActorSnap[] = [];
  private view: DataView | null = null;
  private wasm: LfWasm;
  private world: InstanceType<LfWasm['glue']['LfWorld']>;

  constructor(wasm: LfWasm, world: InstanceType<LfWasm['glue']['LfWorld']>) {
    this.wasm = wasm;
    this.world = world;
    if (wasm.glue.snapshot_stride() !== SNAP_STRIDE) {
      throw new Error(`snapshot stride mismatch: module ${wasm.glue.snapshot_stride()}, host ${SNAP_STRIDE}`);
    }
    this.read();
    this.prev = this.curr;
  }

  /** Call after each core tick: the old current becomes previous, then re-read. */
  push(): void {
    this.prev = this.curr;
    this.read();
  }

  /** The previous/current pair blended at `alpha`. Actors with no previous record snap. */
  interpolate(alpha: number): DrawActor[] {
    const prevById = new Map<number, ActorSnap>();
    for (const p of this.prev) prevById.set(p.id, p);
    return this.curr.map((c) => {
      const p = prevById.get(c.id);
      if (!p) return c;
      return {
        ...c,
        x: p.x + (c.x - p.x) * alpha,
        y: p.y + (c.y - p.y) * alpha,
        z: p.z + (c.z - p.z) * alpha,
        yaw: lerpAngle(p.yaw, c.yaw, alpha)
      };
    });
  }

  private read(): void {
    const buffer = this.wasm.memory.buffer;
    // Memory growth detaches the old ArrayBuffer, so rebuild the view when it changes.
    if (!this.view || this.view.buffer !== buffer) this.view = new DataView(buffer);
    const v = this.view;
    const base = this.world.snapshot_ptr();
    const n = this.world.snapshot_len();
    const out: ActorSnap[] = new Array(n);
    for (let i = 0; i < n; i++) {
      const o = base + i * SNAP_STRIDE;
      out[i] = {
        id: v.getUint32(o, true),
        kind: v.getUint16(o + 4, true),
        flags: v.getUint16(o + 6, true),
        x: v.getFloat32(o + 8, true),
        y: v.getFloat32(o + 12, true),
        z: v.getFloat32(o + 16, true),
        yaw: v.getFloat32(o + 20, true),
        moveId: v.getUint16(o + 24, true),
        moveFrame: v.getUint16(o + 26, true),
        hpFrac: v.getFloat32(o + 28, true),
        poiseFrac: v.getFloat32(o + 32, true),
        statusMask: v.getUint32(o + 36, true)
      };
    }
    this.curr = out;
  }
}
