// Art characters: models baked by the Blender pipeline (`game/art/`, run with
// `mise run art:character -- NAME`) from a Mixamo-rigged mesh and Mixamo clips. Each one
// is a GLB with its clips already on its own skeleton, named by the game clip ids below,
// and a JSON sidecar with each clip's length and contact frame. This file maps move `anim`
// names to those clips. Pure: no Three.js, no I/O.
// Pipeline guide: docs/game/tutorial-production-plan.md ("Running the character pipeline").

import type { ClipSpec } from './moveClips';
import type { StanceClip } from './stancePose';

/** Where `lf-tools assets` writes art characters: `NAME.glb` and `NAME.json`. */
export const ART_ASSET_BASE = '/latticefall/characters/';

/** Where the dev server serves private art, baked locally and never committed. */
export const PRIVATE_ART_LOCAL_BASE = '/latticefall/private/';

/** The API route that serves private art from R2 in production (`api/src/handlers/art.rs`). */
export const PRIVATE_ART_API_PATH = '/api/art/private/';

/**
 * Characters whose license forbids a public repository (Mixamo's own characters). Their
 * baked files live in the `ifkash-art` R2 bucket, not in Git.
 */
export const PRIVATE_ART: ReadonlySet<string> = new Set(['paladin', 'husk', 'sentry']);

/** The character every player plays, unless `&art=` names another (or `off`, for the Wayfarer). */
export const PLAYER_ART = 'paladin';

/**
 * The URL folder of a character's `NAME.glb` and `NAME.json`. Private art comes from the
 * local build in development and from the API in production.
 */
export function artAssetBase(name: string, dev: boolean, apiBase: string): string {
  if (!PRIVATE_ART.has(name)) return ART_ASSET_BASE;
  return dev ? PRIVATE_ART_LOCAL_BASE : `${apiBase}${PRIVATE_ART_API_PATH}`;
}

/** The character to draw as the player, from the `&art=` parameter (null when it's absent). */
export function playerArt(param: string | null): string | null {
  if (param === 'off') return null;
  return param ?? PLAYER_ART;
}

/** One clip in the sidecar. Frames are 30 Hz; `contact` is null for clips without a strike. */
export interface ArtClipInfo {
  frames: number;
  duration: number;
  loop: boolean;
  contact: number | null;
  /**
   * The frame of the wound-up pose right before the swing, when a bake records one. Without
   * it the renderer takes two thirds of the way to the contact (`anticipationFrame`).
   */
  anticipation?: number | null;
  source: string;
}

/** The sidecar the pipeline writes next to the GLB. */
export interface ArtSidecar {
  name: string;
  fps: number;
  triangles: number;
  bones: string[];
  clips: Record<string, ArtClipInfo>;
  /**
   * How each hand holds a weapon, from the config's `grips`: a column-major 4x4 in the GLB's
   * scene coordinates at rest, with the grip at the origin, the blade along +Y, and the flat
   * facing +Z (the layout of `equipment/weapons.ts`).
   */
  grips?: Partial<Record<'L' | 'R', number[]>>;
}

/**
 * Mixamo bone names (the pipeline strips the `mixamorig:` prefix) to the renderer's bone
 * names, parents first. The renderer's stances, gear, and weapon trails find bones by the
 * names on the right. Every other bone of the skeleton (fingers, toes) plays its clip too.
 */
export const MIXAMO_BONE_MAP: ReadonlyArray<readonly [string, string]> = [
  ['Hips', 'Hips'],
  ['Spine', 'Abdomen'],
  ['Spine1', 'Torso'],
  ['Spine2', 'Chest'],
  ['Neck', 'Neck'],
  ['Head', 'Head'],
  ['LeftShoulder', 'Shoulder.L'],
  ['LeftArm', 'UpperArm.L'],
  ['LeftForeArm', 'LowerArm.L'],
  ['LeftHand', 'Hand.L'],
  ['RightShoulder', 'Shoulder.R'],
  ['RightArm', 'UpperArm.R'],
  ['RightForeArm', 'LowerArm.R'],
  ['RightHand', 'Hand.R'],
  ['LeftUpLeg', 'UpperLeg.L'],
  ['LeftLeg', 'LowerLeg.L'],
  ['LeftFoot', 'Foot.L'],
  ['RightUpLeg', 'UpperLeg.R'],
  ['RightLeg', 'LowerLeg.R'],
  ['RightFoot', 'Foot.R']
];

/** The Mixamo ankle bones, for measuring each walk and run clip's stride. */
export const MIXAMO_FEET: readonly [string, string] = ['LeftFoot', 'RightFoot'];

/**
 * The game clip ids a character config may fill. `idle`, `walk`, and `run` are required;
 * the rest fall back as `ART_ANIMS` lists.
 */
export const ART_CLIP_IDS = [
  'idle',
  'walk',
  'run',
  'sprint',
  // Lock-on movement (`ART_STRAFE`): sideways and backward loops at a walk and at a run.
  'walk_back',
  'run_back',
  'strafe_l',
  'strafe_r',
  'strafe_run_l',
  'strafe_run_r',
  // The light chain's three swings, then the attacks named for what they do.
  'slash_a',
  'slash_b',
  'slash_c',
  'lunge',
  'spin',
  'leap',
  'heavy',
  'block',
  'block_idle',
  'roll',
  'hit',
  'death',
  // The flask: the hand to the mouth, then back down (`drink` alone is the whole motion).
  'drink',
  'drink_end',
  'jump',
  'jump_run',
  'twohand_idle',
  'twohand_slash',
  'twohand_heavy',
  // Enemies and bosses (see `ART_ANIMS`): a husk wakes with `rise`, and `stagger` plays while
  // an actor is locked out. The attacks are named for what they do, not for one enemy.
  'rise',
  'stagger',
  'attack_a',
  'attack_b',
  'slam',
  'sweep',
  'bash',
  'combo_1',
  'combo_2',
  'combo_3'
] as const;

/** Locomotion loops. A character without `sprint` runs faster instead. */
export const ART_LOOPS = { idle: 'idle', walk: 'walk', run: 'run', sprint: 'sprint' } as const;

/** The guard: the looping block idle, else the block clip held at its raised frame. */
export const ART_GUARD: readonly StanceClip[] = [{ clip: 'block_idle' }, { clip: 'block', frame: 8 }];

/** The two-handed stance: a two-handed idle that loops, with both hands on the grip. */
export const ART_TWO_HAND: readonly StanceClip[] = [{ clip: 'twohand_idle' }];

/** The reactions a character plays when struck, locked out, and killed, best first. */
export const ART_REACTIONS = {
  hit: ['hit'],
  stagger: ['stagger', 'hit'],
  death: ['death']
} as const;

/**
 * The loops for moving while locked on, by travel direction relative to the facing
 * (`strafeDirection` in `locomotion.ts`). Forward travel keeps the normal loops. A direction
 * needs its walk loop; without a run loop the walk plays faster.
 */
export const ART_STRAFE = {
  left: { walk: 'strafe_l', run: 'strafe_run_l' },
  right: { walk: 'strafe_r', run: 'strafe_run_r' },
  back: { walk: 'walk_back', run: 'run_back' }
} as const;

/**
 * A jump clip and its key frames (30 Hz clip frames): the last frame on the ground, the top
 * of the jump, and the first frame back on the ground. The renderer plays takeoff to apex
 * while the actor rises, apex toward landing while it falls, and the rest as the landing.
 */
export interface ArtJump {
  clip: string;
  takeoff: number;
  apex: number;
  land: number;
}

/** The standing jump, and the running jump a character moving faster than `ART_JUMP_RUN_SPEED` plays. */
export const ART_JUMP: ArtJump = { clip: 'jump', takeoff: 1, apex: 9, land: 20 };
export const ART_JUMP_RUN: ArtJump = { clip: 'jump_run', takeoff: 1, apex: 11, land: 22 };
/** Ground speed (m/s) at takeoff above which the running jump plays. */
export const ART_JUMP_RUN_SPEED = 2;

/** A stretch of one art clip, in 30 Hz clip frames (`to` is exclusive), as in `ClipSeg`. */
export interface ArtPart {
  clip: string;
  from?: number;
  to?: number;
}

/**
 * One way to draw a move: a clip id, a stretch of a clip, or stretches played back to back
 * (the renderer crossfades each join, because separate clips don't meet on one pose).
 */
export type ArtChoice = string | ArtPart | readonly ArtPart[];

/** The clips an attack with no listed mapping plays, best first. */
const ATTACK_FALLBACK: readonly ArtChoice[] = ['slash_a', 'slash_b', 'slash_c', 'heavy', 'attack_a', 'slam'];

/** Move `anim` names to art clips, best first. Today: the starting sword and shield. */
export const ART_ANIMS: Readonly<Record<string, readonly ArtChoice[]>> = {
  // Light chain: a forehand cut, a downward diagonal, then a thrust.
  sword_slash_r: ['slash_a'],
  sword_slash_l: ['slash_b', 'slash_a'],
  sword_thrust: ['slash_c', 'slash_a'],
  // The running attack is the same thrust; the clip starts at a run.
  sword_lunge_run: ['slash_c', 'slash_a'],
  sword_backstep_slash: ['slash_a'],
  // Out of a roll and for Rending Sweep: a turning backhand.
  sword_roll_slash: ['spin', 'slash_a'],
  spin_slash: ['spin', 'slash_b', 'slash_a'],
  // The guard counter steps in with a low horizontal cut.
  sword_counter: ['lunge', 'slash_c', 'slash_b'],
  sword_rising: ['slash_c', 'heavy'],
  // Heavies and criticals.
  sword_overhead: ['heavy', 'slash_a'],
  sword_overhead_charged: ['heavy', 'slash_a'],
  sword_riposte: ['heavy', 'slash_c'],
  sword_backstab: ['heavy', 'slash_c'],
  // In the air: a leaping overhead cut.
  sword_jump_slash: ['leap', 'slash_a'],
  sword_jump_overhead: ['leap', 'heavy', 'slash_a'],
  sword_plunge: ['leap', 'heavy', 'slash_a'],
  // The shield: bashes push the guard out, parries and braces raise it.
  shield_bash_r: ['lunge', 'slash_c', 'block'],
  shield_bash_l: ['lunge', 'slash_c', 'block'],
  shield_guard_bash: ['lunge', 'slash_c', 'block'],
  parry_raise: ['block'],
  guard_brace: ['block'],
  // Everything else the player does.
  roll: ['roll'],
  roll_quick: ['roll'],
  roll_heavy: ['roll'],
  dash_feint: ['roll'],
  // No backstep clip exists yet: one stride of the backward run stands in.
  backstep: ['run_back'],
  hit_chest: ['hit'],
  receive_riposte: ['hit', 'death'],
  receive_backstab: ['hit', 'death'],
  death: ['death'],
  flask_drink: [[{ clip: 'drink' }, { clip: 'drink_end' }], 'drink'],
  // The Husk (game/data/moves/enemies/husk.ron): a slow two-hit combo.
  husk_swipe_1: ['attack_a'],
  husk_swipe_2: ['attack_b', 'attack_a'],
  // B01, the Hollow Sentry (game/data/moves/bosses/b01.ron).
  sentry_overhead: ['slam'],
  sentry_sweep: ['sweep', 'combo_1'],
  sentry_bash: ['bash'],
  sentry_combo_1: ['combo_1', 'sweep'],
  sentry_combo_2: ['combo_2', 'sweep'],
  sentry_combo_3: ['combo_3', 'slam']
};

/** A chest lantern: an emissive flame on a bone, with a light that flares through wind-ups. */
export interface ArtLantern {
  /** Mixamo bone name the lantern rides on. */
  bone: string;
  /** Offset from the bone at rest, in meters of the scaled character: forward (+Z) and up (+Y). */
  forward: number;
  up: number;
  /** Flame radius in meters. */
  radius: number;
  color: number;
  /** Light intensity at rest; a wind-up multiplies it by `lanternGlow`. */
  intensity: number;
  /** Light range in meters. */
  distance: number;
}

/** How the game draws one art character, beyond what its bake holds. */
export interface ArtCharacter {
  /** Standing height in meters at rest (the T-pose). */
  height: number;
  /** A weapon id (`game/data/weapons/`) held at the right-hand grip, for an enemy. */
  weapon?: string;
  /** Scales the held weapon, so a boss's blade matches its size. Default 1. */
  weaponScale?: number;
  /** Reach of the weapon past the hand in meters, for trails. Default 0.95. */
  weaponLength?: number;
  lantern?: ArtLantern;
}

/** Every art character the game knows, by name. Unlisted names draw at the player's height. */
export const ART_CHARACTERS: Readonly<Record<string, ArtCharacter>> = {
  paladin: { height: 1.75 },
  husk: { height: 1.7, weaponLength: 0.5 },
  // B01's body: a suit of plate half again a man's height, a greatsword, and a lantern
  // burning in the chest where a heart would be.
  sentry: {
    height: 2.9,
    weapon: 'wpn_gray_greatsword',
    weaponScale: 1.45,
    weaponLength: 2.2,
    lantern: { bone: 'Spine2', forward: 0.2, up: 0.02, radius: 0.09, color: 0xffa23a, intensity: 5, distance: 9 }
  }
};

/** The player's height in meters, for a character `ART_CHARACTERS` doesn't list. */
export const DEFAULT_ART_HEIGHT_M = 1.75;

/**
 * Enemy kinds (registry numbers, the snapshot's `kind`) drawn as art characters. The
 * procedural model in `render/procedural/` stays the fallback: it draws until the character
 * loads, and for good if it fails. 9011 is B01's body, `B01_KIND` in `procedural/registry.ts`.
 */
export const ENEMY_ART: ReadonlyMap<number, string> = new Map([
  [7000, 'husk'],
  [9011, 'sentry']
]);

/** The art character for an enemy kind, or undefined. `&enemyart=off` turns them all off. */
export function enemyArt(kind: number, param: string | null): string | undefined {
  if (param === 'off') return undefined;
  return ENEMY_ART.get(kind);
}

/** The move timing `lanternGlow` reads: the strike window, as in `MoveAnimInfo`. */
interface GlowTiming {
  totalFrames: number;
  strikeStart?: number;
  recoveryStart?: number;
}

/** How much brighter than rest the lantern burns at the strike. */
export const LANTERN_PEAK = 3.2;
/** Frames the flare takes to settle after the strike. */
const LANTERN_SETTLE_FRAMES = 18;

/**
 * The lantern's brightness for a move at `frame`, 1 at rest: it swells through the wind-up
 * (the telegraph players read), peaks as the hitbox opens, and settles over the recovery. A
 * move without a hitbox leaves it at rest.
 */
export function lanternGlow(info: GlowTiming | undefined, frame: number): number {
  if (!info || info.strikeStart === undefined || info.strikeStart <= 0) return 1;
  const strike = info.strikeStart;
  if (frame < strike) return 1 + (LANTERN_PEAK - 1) * Math.max(frame / strike, 0) ** 2;
  const after = frame - strike;
  return Math.max(1, LANTERN_PEAK - ((LANTERN_PEAK - 1) * after) / LANTERN_SETTLE_FRAMES);
}

/** The stretches of an `ArtChoice`. */
function artParts(choice: ArtChoice): readonly ArtPart[] {
  if (typeof choice === 'string') return [{ clip: choice }];
  return 'clip' in choice ? [choice] : choice;
}

/**
 * The clip a move plays on an art character: the first listed choice whose clips the
 * character has, or for an attack without a listing, its first slash. `contact` comes from
 * the first clip's sidecar entry, counted from the stretch's start; a clip without one is
 * scaled to the move. Undefined means "hold the idle pose".
 */
export function artSpecFor(
  anim: string,
  isAttack: boolean,
  clips: Readonly<Record<string, ArtClipInfo>>,
  twoHanded = false
): ClipSpec | undefined {
  let choices: readonly ArtChoice[] = ART_ANIMS[anim] ?? (isAttack ? ATTACK_FALLBACK : []);
  // Two-handed, an attack swings with both hands: heavies take the two-handed heavy, the
  // rest the two-handed slash. A character without them keeps its one-handed clips.
  if (twoHanded && isAttack) {
    const own = choices[0] === 'heavy' ? ['twohand_heavy', 'twohand_slash'] : ['twohand_slash'];
    choices = [...own, ...choices];
  }
  const pick = choices.find((c) => artParts(c).every((p) => p.clip in clips)) ?? (anim in clips ? anim : undefined);
  if (pick === undefined) return undefined;
  const parts = artParts(pick);
  const first = parts[0];
  const from = first.from ?? 0;
  const contact = Math.max((clips[first.clip].contact ?? 0) - from, 0);
  const spec: ClipSpec = { segs: typeof pick === 'string' ? [{ clip: pick }] : parts, contact };
  const pose = clips[first.clip].anticipation;
  if (typeof pose === 'number' && pose - from > 0) spec.anticipation = pose - from;
  if (parts.length > 1) spec.blend = true;
  return spec;
}

/** The first of `ids` the character has, or undefined. */
export function artFirst(ids: readonly string[], has: (clip: string) => boolean): string | undefined {
  return ids.find(has);
}

/** The two-handed stance clip for a character with these clips, or undefined. */
export function artTwoHand(has: (clip: string) => boolean): StanceClip | undefined {
  return ART_TWO_HAND.find((s) => has(s.clip));
}

/** The guard stance clip for a character with these clips, or undefined. */
export function artGuard(has: (clip: string) => boolean): StanceClip | undefined {
  return ART_GUARD.find((s) => has(s.clip));
}
