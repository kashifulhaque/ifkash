// The one place the dev URL parameters are parsed. No Three.js or WASM imports, so any
// module can read it cheaply.
//
//   &dev=1                  turns the developer tools on: the panel, overlays, dev commands.
//                           Without it, `spawn`, `freeze`, `seed` and `overlay` do nothing.
//   &scenario=NAME          starts that scenario (default m0_flat); test_field has the ledge
//   &spawn=ID               puts the player at a named spawn point (see `spawns.ts`)
//   &seed=N                 world seed, a decimal u64 (default 1)
//   &freeze=1               starts with the simulation paused
//   &freezeonhit=MS         pauses the simulation MS milliseconds after the first Hit event (1 means
//                           120), so the effects of that hit stay on screen for a screenshot
//   &overlay=a,b            debug draw: colliders, hitboxes, hurtboxes, nav, ai, or `all`
//   &cam=MODE               orbit | orbit-script | fixed:X,Y,Z,YAW,PITCH
//                           (YAW and PITCH in degrees; yaw 0 looks toward +Z and turns toward +X,
//                           positive pitch looks down)
//   &camt=SECONDS           with orbit-script, holds the script at that time (0 to 72)
//   &fakelock=X,Y,Z         frames a fixed point as if it were the locked-on actor's feet
//
// `cam`, `camt`, `fakelock` and `scenario` predate the tools and work without `dev`.
// The animation preview parameters (`anim*`) are parsed in `render/actors/devPreview.ts`.

export const OVERLAY_NAMES = ['colliders', 'hitboxes', 'hurtboxes', 'nav', 'ai'] as const;
export type OverlayName = (typeof OVERLAY_NAMES)[number];

export type Vec3 = [number, number, number];

export interface FixedCam {
  pos: Vec3;
  /** Radians, 0 looks toward +Z and turns toward +X. */
  yaw: number;
  /** Radians, positive looks down. */
  pitch: number;
}

export type CamMode = { kind: 'orbit' } | { kind: 'orbit-script' } | ({ kind: 'fixed' } & FixedCam);

export interface DevParams {
  /** `&dev=1`: the panel, overlays and dev commands are on. */
  dev: boolean;
  scenario: string | null;
  spawn: string | null;
  seed: bigint | null;
  freeze: boolean;
  /** Milliseconds after the first hit at which to pause, or null. */
  freezeOnHit: number | null;
  overlays: ReadonlySet<OverlayName>;
  /** Null when no `&cam=` was given. */
  cam: CamMode | null;
  camT: number | null;
  fakeLock: Vec3 | null;
  /** Parameters that were present but unusable, for the panel and the console. */
  warnings: string[];
}

const DEG = Math.PI / 180;
const U64_MAX = (1n << 64n) - 1n;

function numbers(raw: string | null, count: number): number[] | null {
  if (raw === null) return null;
  const parts = raw.split(',');
  if (parts.length !== count) return null;
  const nums = parts.map((s) => (s.trim() === '' ? NaN : Number(s)));
  return nums.every(Number.isFinite) ? nums : null;
}

export function parseDevParams(search: string): DevParams {
  const p = new URLSearchParams(search);
  const warnings: string[] = [];
  const dev = p.get('dev') === '1';

  let cam: CamMode | null = null;
  const rawCam = p.get('cam');
  if (rawCam !== null) {
    if (rawCam === 'orbit-script') cam = { kind: 'orbit-script' };
    else if (rawCam.startsWith('fixed:')) {
      const n = numbers(rawCam.slice('fixed:'.length), 5);
      if (n) cam = { kind: 'fixed', pos: [n[0], n[1], n[2]], yaw: n[3] * DEG, pitch: n[4] * DEG };
      else warnings.push(`cam=${rawCam}: expected fixed:X,Y,Z,YAW,PITCH`);
    } else {
      // `cam=orbit` and any other value keep the old behavior: the normal rig plus the level proxy.
      cam = { kind: 'orbit' };
      if (rawCam !== 'orbit') warnings.push(`cam=${rawCam}: unknown mode, using orbit`);
    }
  }

  const rawAt = p.get('camt');
  const camT = rawAt !== null && Number.isFinite(Number(rawAt)) ? Number(rawAt) : null;
  const lock = numbers(p.get('fakelock'), 3);

  let seed: bigint | null = null;
  const rawSeed = p.get('seed');
  if (rawSeed !== null) {
    if (/^\d+$/.test(rawSeed) && BigInt(rawSeed) <= U64_MAX) seed = BigInt(rawSeed);
    else warnings.push(`seed=${rawSeed}: expected an unsigned 64-bit integer`);
  }

  const rawHit = p.get('freezeonhit');
  const hitMs = rawHit !== null && /^\d+$/.test(rawHit) ? Number(rawHit) : null;

  const overlays = new Set<OverlayName>();
  for (const raw of (p.get('overlay') ?? '').split(',')) {
    const name = raw.trim().toLowerCase();
    if (!name) continue;
    if (name === 'all') OVERLAY_NAMES.forEach((o) => overlays.add(o));
    else if ((OVERLAY_NAMES as readonly string[]).includes(name)) overlays.add(name as OverlayName);
    else warnings.push(`overlay=${name}: unknown overlay (${OVERLAY_NAMES.join(', ')}, all)`);
  }

  return {
    dev,
    scenario: p.get('scenario') || null,
    spawn: dev ? p.get('spawn') || null : null,
    seed: dev ? seed : null,
    freeze: dev && ['1', 'true'].includes(p.get('freeze') ?? ''),
    freezeOnHit: dev && hitMs !== null ? (hitMs === 1 ? 120 : hitMs) : null,
    overlays: dev ? overlays : new Set(),
    cam,
    camT,
    fakeLock: lock ? [lock[0], lock[1], lock[2]] : null,
    warnings
  };
}
