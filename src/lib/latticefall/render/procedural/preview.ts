// Dev-only enemy model preview. Inert unless the URL has a `preview` parameter. No Three.js.
//
//   /game?engine=lf&dev=1&scenario=m0_flat&freeze=1&preview=husk,thorn_hound&cam=fixed:0,2,1,0,12
//
//   &preview=ID,ID    lays the named models (content ids, e.g. husk_frost, b01_hollow_sentry) in a
//                     row, 3 m apart, centered on x = 0, facing the player's side (-Z)
//   &pmove=N|NAME     poses each model in its Nth move (0 is the first; names match too), or leave
//                     it off for the idle pose
//   &pt=0.4           freezes the move at that fraction (0 to 1) of its frames, or the idle at that
//                     many seconds when there is no `pmove`; without it, the preview loops
//   &pspeed=2         walks in place at that ground speed (m/s) instead of idling
//   &pdead=1          shows the fallen pose
//   &pz=6             distance of the row from the origin along +Z (default 6)
//
// The simulation still runs underneath (use `freeze=1` to stop it).

export interface PropPreview {
  ids: string[];
  move: string | null;
  /** Fraction of the move, or seconds of idle, or undefined to loop. */
  t?: number;
  speed?: number;
  dead: boolean;
  z: number;
}

export function parsePropPreview(search: string): PropPreview | null {
  const p = new URLSearchParams(search);
  const raw = p.get('preview');
  if (!raw) return null;
  const num = (k: string) => {
    const v = p.get(k);
    const x = v === null ? NaN : Number(v);
    return Number.isFinite(x) ? x : undefined;
  };
  return {
    ids: raw.split(',').filter(Boolean),
    move: p.get('pmove'),
    t: num('pt'),
    speed: num('pspeed'),
    dead: p.get('pdead') === '1',
    z: num('pz') ?? 6
  };
}
