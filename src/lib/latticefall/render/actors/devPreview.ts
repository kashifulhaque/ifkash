// Dev-only animation preview. Inert unless the URL has an `anim` parameter.
//
//   /game?engine=lf&anim=run             hold the run state (the player model still follows the sim)
//   &animt=0.25                          freeze the state at 0.25 s (clip time for jump, land and
//                                        move states, a 0 to 1 phase for locomotion)
//   &animcam=side                        close camera: front, side, back or three (3/4 view)
//   &animspeed=5                         override the ground speed (m/s) for locomotion states
//
// States: idle, walk, run, sprint, jump (rising), fall, land, and `move:ANIM_NAME` to preview
// a move's clip or its generic-pose fallback, scrubbed over `animt` as 0 to 1 of the move.

export type PreviewState = 'idle' | 'walk' | 'run' | 'sprint' | 'jump' | 'fall' | 'land' | { move: string };

export interface PreviewRequest {
  state: PreviewState;
  /** Frozen time or phase, or undefined to run freely. */
  t?: number;
  speed?: number;
  cam?: 'front' | 'side' | 'back' | 'three';
}

const STATES = new Set(['idle', 'walk', 'run', 'sprint', 'jump', 'fall', 'land']);

export function parsePreview(search: string): PreviewRequest | null {
  const p = new URLSearchParams(search);
  const raw = p.get('anim');
  if (!raw) return null;
  let state: PreviewState;
  if (raw.startsWith('move:')) state = { move: raw.slice(5) };
  else if (STATES.has(raw)) state = raw as PreviewState;
  else return null;
  const num = (k: string) => {
    const v = p.get(k);
    const x = v === null ? NaN : Number(v);
    return Number.isFinite(x) ? x : undefined;
  };
  const cam = p.get('animcam');
  return {
    state,
    t: num('animt'),
    speed: num('animspeed'),
    cam: cam === 'front' || cam === 'side' || cam === 'back' || cam === 'three' ? cam : undefined
  };
}
