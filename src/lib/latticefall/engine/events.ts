// Decodes the core's event words (`LfWorld.drain_events`, game/crates/wasm/src/lib.rs) into
// typed events for the renderer. Events are the one-way channel from the simulation to
// presentation: nothing here writes simulation state.
//
// Framing: each record is its kind word followed by a fixed number of payload words, with
// no length prefix (see `EVENT_WORDS`). A kind this table doesn't know ends decoding,
// because without a length the next record can't be located. When the core adds a kind, or
// appends words to an existing one (WP-023's damage words on `Hit`), update `EVENT_WORDS`
// and the matching branch below in the same change.

/** Words per record, kind word included, by event kind. Mirrors the doc on `drain_events`. */
export const EVENT_WORDS: Readonly<Record<number, number>> = {
  1: 17, // Hit: 13 words plus damage, result, flags, and hp after (WP-023)
  2: 6, // Life: code, value, position (WP-029)
  3: 4, // Stagger: target, heavy, ticks (WP-024)
  4: 3, // StanceBreak: target, kneel ticks (WP-024)
  5: 4, // Parry: attacker, target, parried move (WP-024)
  6: 5, // Critical: attacker, target, crit kind, crit move (WP-024)
  7: 4, // StatusProc: target, status, burst damage (WP-025)
  8: 10, // Projectile: what, handle, def, shooter, target, position, ended (WP-027)
  9: 4, // Awareness: AI actor, state left, state entered (WP-030)
  // Kinds 10 and 12 are reserved for other work packages.
  11: 4, // Boss: boss index, what happened, phase (WP-033)
  13: 3, // Revive: enemy, HP it got back up with (WP-032)
  17: 3 // Pickup: index in the scenario's pickup list, item registry number (WP-049)
  // Kinds 2-9, 11, and 13 aren't decoded into events yet; the decoder skips them by length.
};

export const EVENT_HIT = 1;

export interface HitEvent {
  kind: 'hit';
  /** `pack_id` of the attacker and target; equal to `ActorSnap.id`. */
  attacker: number;
  target: number;
  /** The attacker's move, as its registry number (equal to `ActorSnap.moveId`). */
  moveNum: number;
  /** Index of the hitbox in the move's `hitboxes` list. */
  hitbox: number;
  motionValue: number;
  /** How far the move was charged, 0 to 1. */
  chargeFrac: number;
  /** World-space contact point. */
  point: [number, number, number];
  /** Unit ground-plane direction from the attacker to the target. */
  dir: [number, number, number];
  /** Damage the target took. */
  damage: number;
  /** 0 connected, 1 guarded, 2 guard broken. */
  result: number;
  /** Bit 0 counter hit, bit 1 killed. */
  flags: number;
  /** Target HP after the hit. */
  hpAfter: number;
}

export const HIT_CONNECTED = 0;
export const HIT_GUARDED = 1;
export const HIT_GUARD_BROKEN = 2;

export const EVENT_PICKUP = 17;

/** The player took a pickup (WP-049). The core decided it; the codex store records it. */
export interface PickupEvent {
  kind: 'pickup';
  /** Index in the scenario's pickup list. */
  index: number;
  /** The item's registry number. */
  item: number;
}

export type GameEvent = HitEvent | PickupEvent;

const scratch = new DataView(new ArrayBuffer(4));
function f32(bits: number): number {
  scratch.setUint32(0, bits >>> 0, true);
  return scratch.getFloat32(0, true);
}

/** Decodes every complete record in `words`. A truncated trailing record is dropped. */
export function decodeEvents(words: ArrayLike<number>): GameEvent[] {
  const out: GameEvent[] = [];
  let i = 0;
  while (i < words.length) {
    const kind = words[i];
    const len = EVENT_WORDS[kind];
    if (len === undefined || i + len > words.length) break;
    if (kind === EVENT_HIT) {
      out.push({
        kind: 'hit',
        attacker: words[i + 1] >>> 0,
        target: words[i + 2] >>> 0,
        moveNum: words[i + 3] >>> 0,
        hitbox: words[i + 4] >>> 0,
        motionValue: f32(words[i + 5]),
        chargeFrac: f32(words[i + 6]),
        point: [f32(words[i + 7]), f32(words[i + 8]), f32(words[i + 9])],
        dir: [f32(words[i + 10]), f32(words[i + 11]), f32(words[i + 12])],
        damage: words[i + 13] >>> 0,
        result: words[i + 14] >>> 0,
        flags: words[i + 15] >>> 0,
        hpAfter: words[i + 16] >>> 0
      });
    } else if (kind === EVENT_PICKUP) {
      out.push({ kind: 'pickup', index: words[i + 1] >>> 0, item: words[i + 2] >>> 0 });
    }
    i += len;
  }
  return out;
}
