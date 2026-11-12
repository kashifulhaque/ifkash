// Pose math and move choreography for procedural models. Pure: no Three.js.
//
// A move is a list of keyframes over its own frames. `attack` builds the usual shape from the
// move's frame data (wind-up end, strike end) and four poses, so the strike lands on the same
// frames as the hitbox. The move's own `total_frames` is never changed to fit the animation.

import type { Pose } from './rig';

export type Keyframes = Array<[frame: number, pose: Pose]>;

const smooth = (x: number) => {
  const t = Math.min(Math.max(x, 0), 1);
  return t * t * (3 - 2 * t);
};

/** a + (b - a) * w, bone by bone. Bones missing from one side are read as the zero offset. */
export function lerpPose(a: Pose, b: Pose, w: number): Pose {
  const out: Pose = {};
  const names = new Set([...Object.keys(a), ...Object.keys(b)]);
  for (const n of names) {
    const pa = a[n] ?? [];
    const pb = b[n] ?? [];
    const len = Math.max(pa.length, pb.length);
    const r: number[] = new Array(len);
    for (let i = 0; i < len; i++) {
      const x = pa[i] ?? 0;
      r[i] = x + ((pb[i] ?? 0) - x) * w;
    }
    out[n] = r;
  }
  return out;
}

/** Samples keyframes at a (fractional) frame, easing between neighbors. */
export function sampleKeys(keys: Keyframes, frame: number): Pose {
  if (keys.length === 0) return {};
  if (frame <= keys[0][0]) return keys[0][1];
  const last = keys[keys.length - 1];
  if (frame >= last[0]) return last[1];
  let i = 1;
  while (keys[i][0] < frame) i++;
  const [f0, p0] = keys[i - 1];
  const [f1, p1] = keys[i];
  return lerpPose(p0, p1, smooth((frame - f0) / Math.max(f1 - f0, 1e-6)));
}

/**
 * The usual attack: rest, a quick `ready` lift, the cocked `wind` held to the end of the
 * wind-up, a fast `strike` through the hitbox frames, a `follow` settle, then back to rest.
 * `windEnd` is the move's first hitbox frame; `strikeEnd` its last.
 */
export function attack(
  windEnd: number,
  strikeEnd: number,
  total: number,
  poses: { ready?: Pose; wind: Pose; strike: Pose; follow?: Pose }
): Keyframes {
  const ready = poses.ready ?? lerpPose({}, poses.wind, 0.35);
  const follow = poses.follow ?? poses.strike;
  const end = Math.max(total, strikeEnd + 2);
  const settle = strikeEnd + (end - strikeEnd) * 0.4;
  return [
    [0, {}],
    [Math.max(windEnd * 0.4, 1), ready],
    // The wind-up eases to its peak a few frames early and holds, so the tell is readable.
    [Math.max(windEnd - 3, 2), poses.wind],
    [windEnd, poses.wind],
    [strikeEnd, poses.strike],
    [settle, follow],
    [end, {}]
  ];
}

export interface Choreo {
  /** The frame count to use when the move is previewed without game data. */
  total: number;
  /** Builds the keyframes for a move of `total` frames. */
  keys(total: number): Keyframes;
  /** Lantern or eye glow multiplier over the move (1 = resting). */
  glow?(frame: number, total: number): number;
}

/** Replaces `base` with `over` on the bones `over` names, by weight `w`; other bones keep `base`. */
export function overlayPose(base: Pose, over: Pose, w: number): Pose {
  if (w <= 0) return base;
  const out: Pose = { ...base };
  for (const [n, p] of Object.entries(over)) {
    const b = base[n] ?? [];
    const len = Math.max(b.length, p.length);
    out[n] = Array.from({ length: len }, (_, i) => {
      const x = b[i] ?? 0;
      return x + ((p[i] ?? 0) - x) * w;
    });
  }
  return out;
}
