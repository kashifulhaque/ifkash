// Mapping from a move's `anim` name to a clip, scrubbing by `move_frame`, and the phase used
// by the generic pose when no clip exists. Pure: no Three.js, no I/O.
// Spec: docs/game/07-presentation.md, "Characters and animation".

/** What the renderer needs to know about the move an actor is performing. */
export interface MoveAnimInfo {
  /** The move's `anim` field. */
  anim: string;
  /** The move's `total_frames`. */
  totalFrames: number;
  /** First frame of the strike (the first hitbox frame), if known. */
  strikeStart?: number;
  /** First frame of the recovery (after the last hitbox frame), if known. */
  recoveryStart?: number;
}

/** Looks up a `move_id` from the snapshot. Returns undefined for "no move" (locomotion). */
export type MoveResolver = (moveId: number) => MoveAnimInfo | undefined;

/** A stretch of one library clip, in 30 Hz clip frames. `to` is exclusive. */
export interface ClipSeg {
  clip: string;
  from?: number;
  to?: number;
}

/**
 * How a move plays: one or more clip stretches back to back, and the frame of that strip
 * where the blade (or fist) passes through the target, measured as the hand's peak speed.
 */
export interface ClipSpec {
  segs: readonly ClipSeg[];
  /** Contact frame, counted from the start of the first stretch. */
  contact: number;
  /** The anticipation pose's frame, when the clip's bake gives one (see `anticipationFrame`). */
  anticipation?: number;
  /** Set when the stretches don't meet on the same pose, so the renderer crossfades each join. */
  blend?: boolean;
}

const spec = (contact: number, ...segs: ClipSeg[]): ClipSpec => ({ segs, contact });

// Universal Animation Library 1 (`ual.glb`). Contact frames are the hand's peak speed.
const SWORD_ATTACK = spec(12, { clip: 'Sword_Attack' });
const PUNCH_JAB = spec(5, { clip: 'Punch_Jab' });
const PUNCH_CROSS = spec(6, { clip: 'Punch_Cross' });

// Universal Animation Library 2 (`ual2.glb`, staged: ADR 0009). The recovery clips end on a
// frame that snaps back to the start pose, so that frame is cut.
const SLASH_A = spec(8, { clip: 'Sword_Regular_A' }, { clip: 'Sword_Regular_A_Rec', to: 28 });
const SLASH_B = spec(8, { clip: 'Sword_Regular_B' }, { clip: 'Sword_Regular_B_Rec', to: 30 });
const SPIN = spec(20, { clip: 'Sword_Regular_C', to: 59 });
// The four hits of `Sword_Heavy_Combo`: a wound-up sweep, an overhead chop, a low lunge,
// and a rising cut.
const HEAVY_SWEEP = spec(12, { clip: 'Sword_Heavy_Combo', from: 0, to: 24 });
const OVERHEAD = spec(13, { clip: 'Sword_Heavy_Combo', from: 22, to: 46 });
const LUNGE = spec(10, { clip: 'Sword_Heavy_Combo', from: 44, to: 64 });
const RISING = spec(15, { clip: 'Sword_Heavy_Combo', from: 62, to: 100 });
const DASH_SLASH = spec(10, { clip: 'Sword_Dash' });
const HOOK = spec(8, { clip: 'Melee_Hook' }, { clip: 'Melee_Hook_Rec' });
const THROW_OVERHEAD = spec(11, { clip: 'OverhandThrow' });
// The chopping loop starts mid-swing: play its wind-up, then the chop from the loop's start.
const CHOP = spec(23, { clip: 'TreeChopping_Loop', from: 10, to: 29 }, { clip: 'TreeChopping_Loop', from: 0, to: 14 });

// Arts (moves/arts.ron): each Art plays its own clip or chain of clips (issue #22).
const CAST = spec(16 + 6, { clip: 'Spell_Simple_Enter' }, { clip: 'Spell_Simple_Shoot' }, { clip: 'Spell_Simple_Exit' });
const ART_CHOICES: Readonly<Record<string, readonly ClipSpec[]>> = {
  // Feint Step: a low forward dash with i-frames.
  dash_feint: [spec(0, { clip: 'Sword_Dash', from: 0, to: 14 }), spec(0, { clip: 'Roll' })],
  // Rending Sweep: a full spin.
  spin_slash: [SPIN, SWORD_ATTACK],
  // Bastion: brace behind the off arm, hold the shield guard, then lower it.
  guard_brace: [
    spec(0, { clip: 'Shield_OneShot', from: 0, to: 6 }, { clip: 'Idle_Shield_Loop' }, { clip: 'Shield_OneShot', from: 6 })
  ],
  // Windward Parry: a sharp brace and release.
  parry_raise: [spec(0, { clip: 'Shield_OneShot' }), spec(0, { clip: 'Hit_Chest' })],
  // Rising Cleave: an upward cut.
  sword_rising: [RISING, SWORD_ATTACK],
  // Rallying Roar: chest out and arms flung wide.
  roar: [spec(0, { clip: 'Idle_Shield_Break' }), spec(0, { clip: 'Interact' })],
  // Lunging Thrust: gather low, then drive forward.
  thrust_lunge: [spec(6 + 10, { clip: 'NinjaJump_Start', from: 0, to: 6 }, { clip: 'Sword_Heavy_Combo', from: 44, to: 64 }), PUNCH_CROSS],
  // Earthsplit: crouch to gather, then a full overhead slam.
  hammer_slam: [spec(20 + 13, { clip: 'NinjaJump_Start', from: 0, to: 20 }, { clip: 'Sword_Heavy_Combo', from: 22, to: 46 }), SWORD_ATTACK],
  // Rime Edge and Radiant Blade: raise the free hand to the blade, then release.
  edge_infuse: [CAST],
  // Spark Volley: the bow arm thrust forward.
  bow_volley: [CAST],
  // Lantern Flare: fling the flare overhand.
  lantern_swing: [THROW_OVERHEAD, SWORD_ATTACK],
  // Glass Barrage: a cast from the off hand.
  glass_cast: [CAST],
  // Bell Toll and Choir Toll: a wound-up sweep that turns into an overhead strike.
  mallet_toll: [spec(22 + 13, { clip: 'Sword_Heavy_Combo', from: 0, to: 46 }), SWORD_ATTACK]
};

/** Each family of move `anim` names, and the clip choices it plays, best first. */
const FAMILIES: ReadonlyArray<readonly [readonly ClipSpec[], readonly string[]]> = [
  // Forehand and backhand cuts: the first two hits of most light combos.
  [
    [SLASH_A, SWORD_ATTACK],
    [
      'sword_slash_r',
      'sword_backstep_slash',
      'sword_jump_slash',
      'curved_cut_r',
      'curved_backstep_slash',
      'curved_jump_slash',
      'greatsword_sweep_r',
      'greatsword_backstep_sweep',
      'greatsword_jump_sweep',
      'halberd_sweep_r',
      'halberd_backstep_swing',
      'halberd_jump_swing',
      'hammer_swing_r',
      'hammer_backstep_swing',
      'hammer_jump_swing',
      'colossal_swing_r',
      'colossal_backstep_swing',
      'colossal_jump_swing',
      'catalyst_swing_r',
      'catalyst_backstep_swing',
      'catalyst_jump_swing',
      'spear_backstep_swing',
      'spear_jump_swing',
      'shield_backstep_swing',
      'shield_jump_swing'
    ]
  ],
  [
    [SLASH_B, SWORD_ATTACK],
    [
      'sword_slash_l',
      'sword_roll_slash',
      'curved_cut_l',
      'curved_roll_slash',
      'greatsword_sweep_l',
      'greatsword_roll_sweep',
      'halberd_sweep_l',
      'halberd_roll_swing',
      'hammer_swing_l',
      'hammer_roll_swing',
      'colossal_swing_l',
      'colossal_roll_swing',
      'catalyst_swing_l',
      'catalyst_roll_swing',
      'spear_roll_swing',
      'shield_roll_swing'
    ]
  ],
  // Axes chop.
  [[CHOP, SLASH_A, SWORD_ATTACK], ['axe_chop_r', 'axe_backstep_chop', 'axe_jump_chop', 'axe_roll_chop']],
  [[SLASH_B, SWORD_ATTACK], ['axe_chop_l']],
  // Spinning finishers and counters.
  [
    [SPIN, SWORD_ATTACK],
    [
      'curved_spin_half',
      'curved_spin_full',
      'curved_spin_full_charged',
      'curved_jump_spin',
      'curved_counter',
      'sword_counter',
      'axe_counter',
      'greatsword_counter',
      'halberd_counter',
      'hammer_counter',
      'colossal_counter',
      'spear_counter',
      'catalyst_counter',
      'shield_counter',
      'dagger_counter',
      'rapier_counter',
      'fist_counter'
    ]
  ],
  // Wound-up horizontal sweeps.
  [[HEAVY_SWEEP, SWORD_ATTACK], ['axe_cleave', 'halberd_cleave', 'halberd_cleave_charged', 'hammer_sweep']],
  // Overhead chops and smashes: heavies, jump heavies, and plunges.
  [
    [OVERHEAD, SWORD_ATTACK],
    [
      'sword_overhead',
      'sword_overhead_charged',
      'sword_jump_overhead',
      'sword_plunge',
      'axe_overhead',
      'axe_overhead_charged',
      'axe_jump_overhead',
      'axe_plunge',
      'greatsword_cleave',
      'greatsword_overhead',
      'greatsword_overhead_charged',
      'greatsword_jump_overhead',
      'greatsword_plunge',
      'halberd_chop',
      'halberd_jump_smash',
      'halberd_plunge',
      'hammer_smash',
      'hammer_smash_charged',
      'hammer_jump_smash',
      'hammer_plunge',
      'colossal_slam',
      'colossal_smash',
      'colossal_smash_charged',
      'colossal_jump_smash',
      'colossal_plunge',
      'curved_plunge',
      'dagger_jump_overhead',
      'dagger_plunge',
      'spear_jump_smash',
      'spear_plunge',
      'rapier_plunge',
      'shield_jump_smash',
      'shield_plunge',
      'fist_jump_smash',
      'fist_plunge',
      'bow_plunge'
    ]
  ],
  [[THROW_OVERHEAD, SWORD_ATTACK], ['catalyst_bash', 'catalyst_overhead', 'catalyst_overhead_charged', 'catalyst_jump_smash', 'catalyst_plunge']],
  // Thrusts: quick stabs use the straight punch, committed ones the low lunge.
  [[PUNCH_CROSS], ['dagger_stab_r', 'dagger_jump_stab', 'dagger_backstep_stab', 'rapier_thrust', 'rapier_backstep_thrust', 'rapier_jump_thrust', 'spear_thrust_r', 'spear_guard_thrust']],
  [[PUNCH_JAB], ['dagger_stab_l', 'rapier_thrust_l', 'spear_thrust_l']],
  [
    [LUNGE, PUNCH_CROSS],
    [
      'sword_thrust',
      'dagger_stab_step',
      'dagger_rising_stab',
      'dagger_rising_stab_charged',
      'dagger_roll_stab',
      'rapier_lunge',
      'rapier_pierce',
      'rapier_pierce_charged',
      'rapier_roll_thrust',
      'rapier_jump_pierce',
      'spear_lunge',
      'spear_impale',
      'spear_impale_charged'
    ]
  ],
  // Running attacks.
  [
    [DASH_SLASH, SWORD_ATTACK],
    [
      'sword_lunge_run',
      'axe_run_chop',
      'curved_run_slash',
      'greatsword_run_sweep',
      'halberd_run_swing',
      'hammer_run_swing',
      'colossal_run_swing',
      'catalyst_run_swing',
      'spear_run_swing',
      'shield_run_swing'
    ]
  ],
  [[LUNGE, PUNCH_CROSS], ['dagger_run_stab', 'rapier_run_thrust']],
  // Big finishers and critical hits.
  [[RISING, SWORD_ATTACK], ['sword_riposte', 'sword_backstab']],
  // Fists and shield bashes.
  [[PUNCH_CROSS], ['fist_jab_r', 'fist_backstep_swing', 'fist_jump_swing']],
  [[PUNCH_JAB], ['fist_jab_l']],
  [[HOOK, PUNCH_CROSS], ['fist_hook', 'fist_uppercut', 'fist_run_swing', 'fist_roll_swing', 'shield_bash_r', 'shield_bash_l', 'shield_guard_bash', 'shield_slam']],
  [[HEAVY_SWEEP, PUNCH_CROSS], ['fist_haymaker', 'fist_haymaker_charged', 'shield_rush', 'shield_rush_charged']]
];

/**
 * Move `anim` names to clip choices, best first. The renderer plays the first choice whose
 * clips are all loaded, so a move keeps a fallback when a library is missing. Anything not
 * listed plays the generic pose (and warns in dev).
 */
export const ANIM_TO_CLIP: Readonly<Record<string, readonly ClipSpec[]>> = {
  ...Object.fromEntries(FAMILIES.flatMap(([choices, anims]) => anims.map((a) => [a, choices]))),
  ...ART_CHOICES,
  roll: [spec(0, { clip: 'Roll' })],
  roll_quick: [spec(0, { clip: 'Roll' })],
  roll_heavy: [spec(0, { clip: 'Roll' })],
  flask_drink: [spec(0, { clip: 'Consume' })],
  hit_chest: [spec(0, { clip: 'Hit_Chest' })],
  death: [spec(0, { clip: 'Death01' })],
  interact: [spec(0, { clip: 'Interact' })]
};

/** The clip choices for a move `anim`, best first, or undefined when there is no mapping. */
export function clipsForAnim(anim: string): readonly ClipSpec[] | undefined {
  return ANIM_TO_CLIP[anim];
}

/** Total length in clip frames of a spec, given each clip's frame count. */
export function specLength(s: ClipSpec, clipFrames: (clip: string) => number): number {
  let n = 0;
  for (const g of s.segs) n += segLength(g, clipFrames(g.clip));
  return n;
}

function segLength(g: ClipSeg, frames: number): number {
  const last = frames - 1;
  const from = Math.min(Math.max(g.from ?? 0, 0), last);
  const to = Math.min(Math.max(g.to ?? last, from), last);
  return to - from;
}

/**
 * Which stretch frame `f` (0 to the spec's length) falls in (`seg`), and the clip time in
 * seconds there. The last frame of one stretch is the first of the next.
 */
export function locateSpec(
  s: ClipSpec,
  f: number,
  clipFrames: (clip: string) => number,
  fps: number
): { clip: string; time: number; seg: number } {
  let rest = Math.max(f, 0);
  for (let i = 0; i < s.segs.length; i++) {
    const g = s.segs[i];
    const len = segLength(g, clipFrames(g.clip));
    if (rest <= len || i === s.segs.length - 1) {
      const from = g.from ?? 0;
      return { clip: g.clip, time: (from + Math.min(rest, len)) / fps, seg: i };
    }
    rest -= len;
  }
  return { clip: s.segs[0].clip, time: 0, seg: 0 };
}

export type MovePhase = 'windup' | 'strike' | 'recovery';

/** Which third of the move this frame is in, from the move's own frame data when known. */
export function movePhase(info: MoveAnimInfo, frame: number): MovePhase {
  const strike = info.strikeStart ?? Math.round(info.totalFrames / 3);
  const recovery = info.recoveryStart ?? Math.round((info.totalFrames * 2) / 3);
  if (frame < strike) return 'windup';
  if (frame < recovery) return 'strike';
  return 'recovery';
}

/** The fastest the recovery may play, as a multiple of the clip's authored rate. */
export const MAX_RECOVERY_RATE = 2.5;
/** The slowest the recovery may play. A clip that ends early settles into the idle instead. */
export const MIN_RECOVERY_RATE = 0.85;
/** A wind-up that would play slower than this plays at its authored rate and holds instead. */
export const MIN_WINDUP_RATE = 0.75;
/** The fastest the wind-up may play. A longer clip skips its start; the crossfade covers it. */
export const MAX_WINDUP_RATE = 1.8;
/** The same bounds for a move without a hitbox, which is scaled as one piece. */
export const MIN_WHOLE_RATE = 0.85;
export const MAX_WHOLE_RATE = 2.5;
/** Sim frames a clip that ended early takes to settle into the idle pose. */
export const SETTLE_FRAMES = 8;

/**
 * The sim frame where a move's contact should land: the middle of its hitbox window, or
 * undefined for a move with no hitbox.
 */
export function hitFrame(info: MoveAnimInfo): number | undefined {
  if (info.strikeStart === undefined) return undefined;
  const end = info.recoveryStart ?? info.strikeStart + 1;
  return Math.max(info.strikeStart, (info.strikeStart + end - 1) / 2);
}

/**
 * The clip frame of the anticipation pose: the wound-up pose right before the swing
 * accelerates. In the baked clips the swing takes about the last third of the wind-up.
 */
export function anticipationFrame(contact: number): number {
  return Math.round((contact * 2) / 3);
}

/** How a clip is fitted to a move's frame data. Rates are clip frames per sim frame. */
export interface MoveFit {
  /** The clip frame the move starts on: above 0 when the wind-up is too long to play. */
  start: number;
  /** The wind-up's rate. 1 while `hold` is above 0. */
  windup: number;
  /** Sim frames the anticipation pose is held for, when the wind-up is shorter than the move's. */
  hold: number;
  /** The clip frame held during `hold`. */
  anticipation: number;
  /** The sim frame the contact lands on, or undefined without a hitbox. */
  hit: number | undefined;
  /** The recovery's rate (or the whole clip's rate, without a hitbox). */
  recovery: number;
  /** The sim frame the clip ends on, when that is before the move ends; otherwise the move's length. */
  clipEnd: number;
}

/**
 * Fits a clip of `length` frames with its contact on `contact` to a move. The frame data
 * never changes to fit the animation, so the clip is retimed in two pieces around the middle
 * of the hitbox window:
 *
 * - The wind-up stretches or squeezes to put the contact there. If it would play slower
 *   than `MIN_WINDUP_RATE`, it plays at its authored rate and holds the anticipation pose
 *   for the rest of the time, which also reads as a telegraph. If it would play faster than
 *   `MAX_WINDUP_RATE`, the clip starts part way in.
 * - The recovery plays at `MIN_RECOVERY_RATE` to `MAX_RECOVERY_RATE`. A long follow-through
 *   is cut (the crossfade into the next pose covers the rest), and a short one ends before
 *   the move does, after which the pose settles into the idle (`moveClipAt`'s `settle`).
 *
 * Without a hitbox the clip is scaled to the move as one piece, within the same bounds.
 */
export function fitMove(length: number, contact: number, info: MoveAnimInfo, anticipation?: number): MoveFit {
  const total = info.totalFrames;
  const hit = hitFrame(info);
  if (hit === undefined || contact <= 0 || hit <= 0 || hit >= total) {
    const raw = total > 0 ? length / total : 0;
    const rate = Math.min(Math.max(raw, MIN_WHOLE_RATE), MAX_WHOLE_RATE);
    const clipEnd = rate > 0 ? Math.min(total, length / rate) : total;
    return { start: 0, windup: rate, hold: 0, anticipation: 0, hit: undefined, recovery: rate, clipEnd };
  }
  let start = 0;
  let windup = contact / hit;
  let hold = 0;
  let pose = 0;
  if (windup < MIN_WINDUP_RATE) {
    windup = 1;
    hold = hit - contact;
    pose = Math.min(Math.max(anticipation ?? anticipationFrame(contact), 0), contact);
  } else if (windup > MAX_WINDUP_RATE) {
    windup = MAX_WINDUP_RATE;
    start = contact - hit * MAX_WINDUP_RATE;
  }
  const rest = Math.max(length - contact, 0);
  const recovery = Math.min(Math.max(rest / (total - hit), MIN_RECOVERY_RATE), MAX_RECOVERY_RATE);
  const clipEnd = Math.min(total, hit + rest / recovery);
  return { start, windup, hold, anticipation: pose, hit, recovery, clipEnd };
}

/** Where a move frame falls in its clip, and how far the pose has settled into the idle. */
export interface MoveClipAt {
  /** Clip frame, 0 to the clip's length. */
  frame: number;
  /** 0 while the clip plays; rises to 1 over `SETTLE_FRAMES` once it has ended early. */
  settle: number;
}

/** The clip frame for a move frame under `fitMove`. `frame` may be fractional. */
export function moveClipAt(length: number, contact: number, info: MoveAnimInfo, frame: number, anticipation?: number): MoveClipAt {
  const total = info.totalFrames;
  const f = Math.min(Math.max(frame, 0), total);
  const fit = fitMove(length, contact, info, anticipation);
  const settle = f > fit.clipEnd ? Math.min((f - fit.clipEnd) / SETTLE_FRAMES, 1) : 0;
  if (fit.hit === undefined) return { frame: Math.min(f * fit.recovery, length), settle };
  if (f > fit.hit) return { frame: Math.min(contact + (f - fit.hit) * fit.recovery, length), settle };
  if (fit.hold <= 0) return { frame: fit.start + f * fit.windup, settle };
  // Authored rate up to the anticipation pose, the hold, then the swing at its authored rate.
  if (f <= fit.anticipation) return { frame: f, settle };
  if (f <= fit.anticipation + fit.hold) return { frame: fit.anticipation, settle };
  return { frame: f - fit.hold, settle };
}

/** The clip frame alone: `moveClipAt` for callers that don't blend the settle. */
export function moveClipFrame(length: number, contact: number, info: MoveAnimInfo, frame: number): number {
  return moveClipAt(length, contact, info, frame).frame;
}

/** The slowest and fastest a stagger or hit reaction plays, as multiples of its authored rate. */
export const MIN_REACT_RATE = 0.75;
export const MAX_REACT_RATE = 1.75;
/** The share of a short stagger spent on the clip's start (the flinch); the rest shows its end. */
export const REACT_HEAD_SHARE = 0.6;

/**
 * Clip time for a reaction clip of `duration` seconds `t` seconds into a lockout that lasts
 * `lockout` seconds. The clip plays at `MIN_REACT_RATE` to `MAX_REACT_RATE` to span the
 * lockout. When it is too long even at the fastest rate, the lockout shows the clip's start
 * (the flinch) and then jumps to its end (the recovery); `tail` is true from the jump on, so
 * the caller can crossfade over it. A lockout of unknown length (0 or less) plays the clip
 * once at its authored rate.
 */
export function reactClipTime(duration: number, lockout: number, t: number): { time: number; tail: boolean } {
  const at = Math.max(t, 0);
  if (!(lockout > 0)) return { time: Math.min(at, duration), tail: false };
  const rate = Math.min(Math.max(duration / lockout, MIN_REACT_RATE), MAX_REACT_RATE);
  const shown = rate * lockout;
  if (shown >= duration - 1e-6) return { time: Math.min(at * rate, duration), tail: false };
  const headEnd = lockout * REACT_HEAD_SHARE;
  if (at <= headEnd) return { time: at * rate, tail: false };
  // The tail ends with the lockout on the clip's last frame.
  return { time: Math.max(Math.min(duration - (lockout - at) * rate, duration), 0), tail: true };
}
