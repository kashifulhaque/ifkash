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

/** The locomotion loop an enemy plays, one at a time (`pickGait`). */
export type Gait = 'idle' | 'walk' | 'run';

/** Playback rates `pickGait` keeps a loop within, so a stretched clip still reads as itself. */
export const GAIT_MIN_RATE = 0.6;
export const GAIT_MAX_RATE = 1.7;
/** How much better the other loop must fit before `pickGait` switches, so it doesn't flicker. */
const GAIT_HYSTERESIS = 1.2;

/**
 * One locomotion loop and its playback rate for a ground speed (m/s), for enemies. Unlike
 * `locomotionBlend`, it never mixes two loops: Mixamo loops differ in length and in which
 * foot leads, so a blend of them makes the legs fight. It plays the loop whose own speed is
 * nearest (by ratio) at `speed / band.speed`, which keeps the feet planted, and holds the
 * `current` loop unless the other fits clearly better.
 */
export function pickGait(speed: number, bands: LocoBands, current: Gait): { gait: Gait; rate: number } {
  if (!(speed > IDLE_SPEED)) return { gait: 'idle', rate: 1 };
  const misfit = (g: 'walk' | 'run') => Math.abs(Math.log(speed / bands[g].speed));
  let gait: 'walk' | 'run' = misfit('walk') <= misfit('run') ? 'walk' : 'run';
  if (current === 'walk' || current === 'run') {
    const other = current === 'walk' ? 'run' : 'walk';
    gait = misfit(current) - misfit(other) > Math.log(GAIT_HYSTERESIS) ? other : current;
  }
  const rate = Math.min(Math.max(speed / bands[gait].speed, GAIT_MIN_RATE), GAIT_MAX_RATE);
  return { gait, rate };
}

/** Travel within this angle of the facing (radians) counts as forward. */
const FORWARD_ARC = (35 * Math.PI) / 180;
/** Travel beyond this angle of the facing counts as backing away. */
const BACKWARD_ARC = (125 * Math.PI) / 180;
/** Slower than this (m/s), an enemy faces where the simulation faces it. */
const STEER_SPEED = 0.4;

/** An angle wrapped to (-pi, pi]. */
export function wrapAngle(a: number): number {
  const t = a % (Math.PI * 2);
  if (t > Math.PI) return t - Math.PI * 2;
  if (t <= -Math.PI) return t + Math.PI * 2;
  return t;
}

/**
 * Where an enemy's body faces while it walks, and whether its loop plays backward. The
 * simulation keeps an enemy facing its target while it circles or backs away, but a
 * walk clip only walks forward. So: travel near the facing keeps the facing; sideways
 * travel turns the body along the path; backing away keeps the facing and plays the loop in
 * reverse. Yaw 0 faces +Z, as in the simulation; `heading` is the travel direction.
 */
export function gaitFacing(simYaw: number, heading: number, speed: number): { yaw: number; reverse: boolean } {
  if (!(speed > STEER_SPEED)) return { yaw: simYaw, reverse: false };
  const off = Math.abs(wrapAngle(heading - simYaw));
  if (off <= FORWARD_ARC) return { yaw: simYaw, reverse: false };
  if (off >= BACKWARD_ARC) return { yaw: wrapAngle(heading + Math.PI), reverse: true };
  return { yaw: heading, reverse: false };
}

/** The way a locked-on actor travels relative to where it faces. */
export type StrafeDir = 'forward' | 'left' | 'right' | 'back';

/** Half the forward and backward sectors (radians); the sideways sectors fill the rest. */
const STRAFE_SECTOR = Math.PI / 4;
/** How far past its sector's edge the travel must turn before the direction changes. */
const STRAFE_HYSTERESIS = (12 * Math.PI) / 180;

/** The middle of each direction's sector, as travel direction minus facing. Left is +X at yaw 0. */
const STRAFE_CENTER: Readonly<Record<StrafeDir, number>> = { forward: 0, left: Math.PI / 2, right: -Math.PI / 2, back: Math.PI };

/**
 * The strafe direction for travel along `heading` by an actor that faces `faceYaw` (both
 * radians, yaw 0 facing +Z and turning toward +X). Each direction owns a quarter turn, and
 * `current` keeps the travel until it is `STRAFE_HYSTERESIS` past that quarter's edge, so a
 * diagonal doesn't flicker between two loops.
 */
export function strafeDirection(faceYaw: number, heading: number, current?: StrafeDir): StrafeDir {
  const off = wrapAngle(heading - faceYaw);
  if (current && Math.abs(wrapAngle(off - STRAFE_CENTER[current])) <= STRAFE_SECTOR + STRAFE_HYSTERESIS) return current;
  const mag = Math.abs(off);
  if (mag <= STRAFE_SECTOR) return 'forward';
  if (mag >= Math.PI - STRAFE_SECTOR) return 'back';
  return off > 0 ? 'left' : 'right';
}
