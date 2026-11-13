// A dev-only way to put a boss hazard on screen without winning half a fight first (WP-100). The
// hidden browser runs slowly and a hazard such as B08's lava needs phase 2, so a screenshot asks for
// the hazard by name and the effects get the same events the core would send, at a held clock:
//
//   &dev=1&hazard=b08|b10|b09|b04|b12|b02[&hzt=SECONDS]
//
// `hzt` is how many seconds of the hazard have played (default per hazard). The clock is held there,
// so the frame doesn't move and a screenshot is stable; pair it with `&freeze=1`. The events built
// here go through exactly the code a real fight uses (`BossHazardFx.handleEvents`, and for B12's
// lines `LoomFx.handleEvents`), so the picture shows what a fight would. Without `&dev=1` this
// does nothing, and it never touches the simulation.

import type { GameEvent } from '../../../engine/events';
import { MECH, TICK_HZ } from './arenas';

export type HazardName = 'b02' | 'b04' | 'b08' | 'b09' | 'b10' | 'b12';

export interface HazardPreview {
  name: HazardName;
  /** Seconds of the hazard that have played. */
  seconds: number;
}

const DEFAULT_SECONDS: Record<HazardName, number> = { b02: 1.5, b04: 0, b08: 25, b09: 3, b10: 0.4, b12: 0.5 };

export function parseHazardPreview(search: string): HazardPreview | null {
  const p = new URLSearchParams(search);
  if (p.get('dev') !== '1') return null;
  const name = p.get('hazard');
  if (name !== 'b02' && name !== 'b04' && name !== 'b08' && name !== 'b09' && name !== 'b10' && name !== 'b12') return null;
  const raw = Number(p.get('hzt'));
  const seconds = p.get('hzt') !== null && Number.isFinite(raw) && raw >= 0 ? raw : DEFAULT_SECONDS[name];
  return { name, seconds };
}

/** One synthetic event and the game tick it "happened" at (relative to the held clock: negative is earlier). */
export interface Staged {
  /** Ticks before the held clock. */
  before: number;
  event: GameEvent;
}

const mech = (what: number, index = 0): GameEvent => ({ kind: 'mechanic', what, index });
const boss = (what: number): GameEvent => ({ kind: 'boss', boss: 0, what, phase: 0 });

/** The events that stage `name`'s hazard, as the core would send them. */
export function stagedEvents(name: HazardName, ticks: number): Staged[] {
  switch (name) {
    case 'b08':
      // The lava began `ticks` ago (the held clock minus the start is the elapsed time).
      return [{ before: ticks, event: mech(MECH.LAVA_STARTED) }];
    case 'b10':
      return [
        { before: 400, event: mech(MECH.BELL_BEAT, 1) },
        { before: 300, event: mech(MECH.ICE_CRACKED, 0) },
        { before: 200, event: mech(MECH.ICE_CRACKED, 1) },
        { before: 160, event: mech(MECH.ICE_BROKE, 0) },
        { before: 90, event: mech(MECH.ICE_BROKE, 1) },
        { before: 60, event: mech(MECH.ICE_CRACKED, 3) },
        { before: ticks + 20, event: mech(MECH.BELL_TOLL_WARNED) },
        { before: ticks, event: mech(MECH.BELL_TOLLED) }
      ];
    case 'b09':
      return [
        { before: ticks + 40, event: mech(MECH.MATRIARCH_DRAINED) },
        { before: ticks + 30, event: mech(MECH.LAMP_LIT, 0) },
        { before: ticks + 20, event: mech(MECH.LAMP_LIT, 3) },
        { before: ticks + 10, event: mech(MECH.LAMP_LIT, 4) },
        { before: ticks, event: mech(MECH.LAMP_LIT, 1) }
      ];
    case 'b02':
      // The gate sealed `ticks` ago; the intro runs 90 ticks.
      return [{ before: ticks, event: boss(0) }];
    case 'b04':
      return [{ before: 0, event: mech(MECH.WEAK_POINT_EXPOSED) }];
    case 'b12':
      return [];
  }
}

/** B12's loom lines in the Spire's arena (the top of the tower, centered on the Weaver's spawn). */
export function staged12Loom(): GameEvent[] {
  const loom = (what: number, slot: number, cx: number, cz: number, dx: number, dz: number, ticks: number): GameEvent => ({
    kind: 'loom',
    what,
    boss: 0,
    slot,
    center: [cx, 22, cz],
    dir: [dx, dz],
    halfLength: 18,
    halfWidth: 0.6,
    ticks
  });
  const s = Math.SQRT1_2;
  return [
    // A parallel set, still warning (the 30-tick warn is stretched so the frame holds).
    loom(0, 0, -4, 8, 1, 0, 900),
    loom(0, 1, 0, 8, 1, 0, 900),
    loom(0, 2, 4, 8, 1, 0, 900),
    // A crossing set at right angles, already a beam.
    loom(0, 3, 0, 4, 0, 1, 900),
    loom(1, 3, 0, 4, 0, 1, 900),
    // And one on the diagonal, warning.
    loom(0, 4, 2, 10, s, s, 900)
  ];
}

export { TICK_HZ };
