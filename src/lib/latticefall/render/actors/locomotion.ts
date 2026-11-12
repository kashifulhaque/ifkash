// Locomotion blending by ground speed. Pure math: no Three.js, no I/O.
//
// docs/game/07-presentation.md ("Characters and animation"): the renderer blends locomotion
// by speed. Each looping clip covers a known distance per cycle (`metersPerCycle`, measured
// from the library's root-motion variant and scaled to the model's leg length). Playing the
// blend at `speed / metersPerCycle` cycles a second plants the feet, so they don't slide.

/** Below this ground speed (m/s) the actor idles. */
export const IDLE_SPEED = 0.15;
/** The walk clip is fully faded in at this fraction of its nominal speed. */
export const WALK_FULL_FRACTION = 0.7;
/** Cycle rate floor, so the legs don't freeze when the idle-to-walk fade begins. */
export const MIN_CYCLES_PER_SEC = 0.35;
/** Above the sprint clip's nominal speed, playback speeds up by at most this factor. */
export const MAX_OVERSPEED = 1.35;

export interface LocoBand {
  /** Ground distance, in meters, one cycle of the clip covers on this model. */
  metersPerCycle: number;
  /** Nominal speed in m/s: `metersPerCycle / seconds`. */
  speed: number;
}

export interface LocoBands {
  walk: LocoBand;
  run: LocoBand;
  sprint: LocoBand;
}

export interface LocoBlend {
  idle: number;
  walk: number;
  run: number;
  sprint: number;
  /** Shared normalized-phase advance for the locomotion clips, in cycles per second. */
  cyclesPerSec: number;
}

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const clamp01 = (x: number) => (x < 0 ? 0 : x > 1 ? 1 : x);

/** Clip weights and shared cycle rate for a ground speed (m/s). Weights sum to 1. */
export function locomotionBlend(speed: number, bands: LocoBands): LocoBlend {
  const out: LocoBlend = { idle: 0, walk: 0, run: 0, sprint: 0, cyclesPerSec: 0 };
  if (!(speed > IDLE_SPEED)) {
    out.idle = 1;
    return out;
  }
  const { walk, run, sprint } = bands;
  let mpc: number;
  const walkFull = walk.speed * WALK_FULL_FRACTION;
  if (speed < walkFull) {
    const t = clamp01((speed - IDLE_SPEED) / Math.max(walkFull - IDLE_SPEED, 1e-3));
    out.idle = 1 - t;
    out.walk = t;
    mpc = walk.metersPerCycle;
  } else if (speed < walk.speed) {
    out.walk = 1;
    mpc = walk.metersPerCycle;
  } else if (speed < run.speed) {
    const t = (speed - walk.speed) / (run.speed - walk.speed);
    out.walk = 1 - t;
    out.run = t;
    mpc = lerp(walk.metersPerCycle, run.metersPerCycle, t);
  } else if (speed < sprint.speed) {
    const t = (speed - run.speed) / (sprint.speed - run.speed);
    out.run = 1 - t;
    out.sprint = t;
    mpc = lerp(run.metersPerCycle, sprint.metersPerCycle, t);
  } else {
    out.sprint = 1;
    mpc = sprint.metersPerCycle;
    speed = Math.min(speed, sprint.speed * MAX_OVERSPEED);
  }
  out.cyclesPerSec = Math.max(speed / mpc, MIN_CYCLES_PER_SEC);
  return out;
}
