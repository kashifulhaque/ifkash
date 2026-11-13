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
  14: 3, // Mechanic: what happened, index of the door, lever, lift, ladder, or wall (WP-042)
  15: 6, // Loot: source (1 chest), index, item, count, Glimmer (WP-042)
  16: 2, // Message: index in the scenario's messages (WP-042)
  17: 3, // Pickup: index in the scenario's pickup list, item registry number (WP-049)
  // Kinds 18-20 are reserved for other work packages.
  21: 5, // Echo: what happened, Echo registry number, body, refusal reason (WP-059)
  // Kinds 22-30 are reserved for other work packages.
  31: 5, // Dialogue: what happened, NPC or station, its index, open topic (WP-068)
  32: 3, // Quest: the giver's index, the new state (WP-068)
  33: 3, // Appearance: the Mirror's preset and dye (WP-068)
  // Kinds 34-39 are reserved for other work packages.
  40: 9, // Briar: what happened, boss, patch slot, position, radius, ticks (WP-070)
  50: 12, // Loom: what happened, boss, line slot, center, direction, half-length, half-width, ticks (WP-080)
  // Kinds 51-56 are reserved for other work packages.
  57: 3, // Mark: the Mark's index, the in-game tick it was earned on (WP-088)
  58: 3 // Gesture: started (0) or ended (1), the gesture's number (WP-088)
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

export const EVENT_MECHANIC = 14;
export const EVENT_BRIAR = 40;

/** What a world mechanic did (WP-042; the sub-kinds are listed on `drain_events`). */
export interface MechanicEvent {
  kind: 'mechanic';
  /** 110 a Briar Stag charge started, 111 a charge struck a tree and stunned the Stag (WP-070). */
  what: number;
  /** The index of the door, lever, lift, and so on; for 110 and 111, the boss's index. */
  index: number;
}

/** B02's bramble patches (WP-070). */
export interface BriarEvent {
  kind: 'briar';
  /** 0 a patch grew, 1 one withered, 2 every patch went (the fight ended or reset). */
  what: number;
  boss: number;
  /** The patch's id, unique among the live patches. */
  patch: number;
  /** Where it lies. */
  pos: [number, number, number];
  /** How far it slows the player, in meters. */
  radius: number;
  /** How long it lasts, in ticks (0 for the withering and clearing events). */
  ticks: number;
}

export const BRIAR_SPAWNED = 0;
export const BRIAR_FADED = 1;
export const BRIAR_CLEARED = 2;
export const MECHANIC_BRIAR_CHARGE_STARTED = 110;
export const MECHANIC_BRIAR_TREE_STRUCK = 111;

/** B12's loom lines (WP-080). */
export interface LoomEvent {
  kind: 'loom';
  /** 0 a line was laid (the warning), 1 it became a beam, 2 the beam went out, 3 every line went. */
  what: number;
  boss: number;
  /** The line's slot, 0 to 5; for `LOOM_CLEARED`, 1 if the boss was defeated and 0 for a reset. */
  slot: number;
  center: [number, number, number];
  /** Unit direction on the ground: x and z. */
  dir: [number, number];
  halfLength: number;
  halfWidth: number;
  /** Ticks until the beam (for a laid line) or of the beam (for a beam); 0 otherwise. */
  ticks: number;
}

export const EVENT_LOOM = 50;
export const LOOM_LAID = 0;
export const LOOM_BEAM = 1;
export const LOOM_FADED = 2;
export const LOOM_CLEARED = 3;
export const MECHANIC_UNSPOOLED_REBORN = 210;
export const MECHANIC_LOOM_BEAM_HIT = 211;

export const EVENT_BOSS = 11;

/** A boss fight's life cycle (WP-033; decoded for the hazard effects in WP-100). */
export interface BossEvent {
  kind: 'boss';
  /** The boss's index in the scenario. */
  boss: number;
  /** 0 the gate sealed (the intro begins), 1 the fight started, 2 a phase started, 3 defeated, 4 reset. */
  what: number;
  phase: number;
}

export const BOSS_SEALED = 0;
export const BOSS_FIGHT_STARTED = 1;
export const BOSS_PHASE_STARTED = 2;
export const BOSS_DEFEATED = 3;
export const BOSS_RESET = 4;

/** The player earned a Mark (WP-088). `mark` indexes `marks_json`; `tick` is the in-game tick. */
export interface MarkEvent {
  kind: 'mark';
  mark: number;
  tick: number;
}

/** A gesture started or ended (WP-088). `gesture` is 0 wave, 1 bow, 2 point, 3 sit, 4 shrug, 5 beckon. */
export interface GestureEvent {
  kind: 'gesture';
  what: 'started' | 'ended';
  gesture: number;
}

export const EVENT_MARK = 57;
export const EVENT_GESTURE = 58;

export type GameEvent = HitEvent | PickupEvent | MechanicEvent | BriarEvent | LoomEvent | BossEvent | MarkEvent | GestureEvent;

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
    } else if (kind === EVENT_MECHANIC) {
      out.push({ kind: 'mechanic', what: words[i + 1] >>> 0, index: words[i + 2] >>> 0 });
    } else if (kind === EVENT_BOSS) {
      out.push({ kind: 'boss', boss: words[i + 1] >>> 0, what: words[i + 2] >>> 0, phase: words[i + 3] >>> 0 });
    } else if (kind === EVENT_BRIAR) {
      out.push({
        kind: 'briar',
        what: words[i + 1] >>> 0,
        boss: words[i + 2] >>> 0,
        patch: words[i + 3] >>> 0,
        pos: [f32(words[i + 4]), f32(words[i + 5]), f32(words[i + 6])],
        radius: f32(words[i + 7]),
        ticks: words[i + 8] >>> 0
      });
    } else if (kind === EVENT_LOOM) {
      out.push({
        kind: 'loom',
        what: words[i + 1] >>> 0,
        boss: words[i + 2] >>> 0,
        slot: words[i + 3] >>> 0,
        center: [f32(words[i + 4]), f32(words[i + 5]), f32(words[i + 6])],
        dir: [f32(words[i + 7]), f32(words[i + 8])],
        halfLength: f32(words[i + 9]),
        halfWidth: f32(words[i + 10]),
        ticks: words[i + 11] >>> 0
      });
    } else if (kind === EVENT_MARK) {
      out.push({ kind: 'mark', mark: words[i + 1] >>> 0, tick: words[i + 2] >>> 0 });
    } else if (kind === EVENT_GESTURE) {
      out.push({ kind: 'gesture', what: words[i + 1] === 0 ? 'started' : 'ended', gesture: words[i + 2] >>> 0 });
    }
    i += len;
  }
  return out;
}
