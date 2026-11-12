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
  1: 17 // Hit (WP-023: 13 words plus damage, result, flags, hp after)
};

/**
 * Hit record lengths this decoder accepts. The record has no length prefix, so a buffer
 * made only of Hit records is read with the longest candidate that divides it cleanly and
 * starts every record with the Hit kind. 13 is the pre-WP-023 encoder; drop it once every
 * build has the 17-word record.
 */
const HIT_WORDS_CANDIDATES = [17, 13];

function hitWords(words: ArrayLike<number>): number {
  for (const n of HIT_WORDS_CANDIDATES) {
    if (words.length % n !== 0) continue;
    let ok = true;
    for (let i = 0; i < words.length; i += n) if (words[i] !== EVENT_HIT) ok = false;
    if (ok) return n;
  }
  return EVENT_WORDS[EVENT_HIT];
}

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
  /** WP-023 words; undefined from a 13-word (older) record. */
  damage?: number;
  /** 0 connected, 1 guarded, 2 guard broken. */
  result?: number;
  /** Bit 0 counter hit, bit 1 killed. */
  flags?: number;
  /** Target HP after the hit. */
  hpAfter?: number;
}

export const HIT_CONNECTED = 0;
export const HIT_GUARDED = 1;
export const HIT_GUARD_BROKEN = 2;

export type GameEvent = HitEvent;

const scratch = new DataView(new ArrayBuffer(4));
function f32(bits: number): number {
  scratch.setUint32(0, bits >>> 0, true);
  return scratch.getFloat32(0, true);
}

/** Decodes every complete record in `words`. A truncated trailing record is dropped. */
export function decodeEvents(words: ArrayLike<number>): GameEvent[] {
  const out: GameEvent[] = [];
  let i = 0;
  const hitLen = hitWords(words);
  while (i < words.length) {
    const kind = words[i];
    const len = kind === EVENT_HIT ? hitLen : EVENT_WORDS[kind];
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
        ...(len >= 17
          ? { damage: words[i + 13] >>> 0, result: words[i + 14] >>> 0, flags: words[i + 15] >>> 0, hpAfter: words[i + 16] >>> 0 }
          : {})
      });
    }
    i += len;
  }
  return out;
}
