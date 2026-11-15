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
export const PRIVATE_ART: ReadonlySet<string> = new Set(['paladin']);

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
  'slash_a',
  'slash_b',
  'slash_c',
  'heavy',
  'block',
  'block_idle',
  'roll',
  'hit',
  'death',
  'drink',
  'twohand_idle',
  'twohand_slash',
  'twohand_heavy'
] as const;

/** Locomotion loops. A character without `sprint` runs faster instead. */
export const ART_LOOPS = { idle: 'idle', walk: 'walk', run: 'run', sprint: 'sprint' } as const;

/** The guard: the looping block idle, else the block clip held at its raised frame. */
export const ART_GUARD: readonly StanceClip[] = [{ clip: 'block_idle' }, { clip: 'block', frame: 8 }];

/** The two-handed stance: a two-handed idle that loops, with both hands on the grip. */
export const ART_TWO_HAND: readonly StanceClip[] = [{ clip: 'twohand_idle' }];

/** The clips an attack with no listed mapping plays, best first. */
const ATTACK_FALLBACK = ['slash_a', 'slash_b', 'slash_c', 'heavy'] as const;

/** Move `anim` names to art clip ids, best first. Today: the starting sword and shield. */
export const ART_ANIMS: Readonly<Record<string, readonly string[]>> = {
  // Light chain: forehand, backhand, then the thrust-like third swing.
  sword_slash_r: ['slash_a'],
  sword_backstep_slash: ['slash_a'],
  sword_jump_slash: ['slash_a'],
  sword_slash_l: ['slash_b', 'slash_a'],
  sword_roll_slash: ['slash_b', 'slash_a'],
  sword_thrust: ['slash_c', 'slash_a'],
  sword_lunge_run: ['slash_c', 'slash_a'],
  sword_counter: ['slash_c', 'slash_b'],
  sword_rising: ['slash_c', 'heavy'],
  spin_slash: ['slash_b', 'slash_a'],
  // Heavies and criticals.
  sword_overhead: ['heavy', 'slash_a'],
  sword_overhead_charged: ['heavy', 'slash_a'],
  sword_jump_overhead: ['heavy', 'slash_a'],
  sword_plunge: ['heavy', 'slash_a'],
  sword_riposte: ['heavy', 'slash_c'],
  sword_backstab: ['heavy', 'slash_c'],
  // The shield: bashes push the guard out, parries and braces raise it.
  shield_bash_r: ['slash_c', 'block'],
  shield_bash_l: ['slash_c', 'block'],
  shield_guard_bash: ['slash_c', 'block'],
  parry_raise: ['block'],
  guard_brace: ['block'],
  // Everything else the player does.
  roll: ['roll'],
  roll_quick: ['roll'],
  roll_heavy: ['roll'],
  dash_feint: ['roll'],
  hit_chest: ['hit'],
  receive_riposte: ['hit', 'death'],
  receive_backstab: ['hit', 'death'],
  death: ['death'],
  flask_drink: ['drink']
};

/**
 * The clip a move plays on an art character: the first listed clip the character has, or
 * for an attack without a listing, its first slash. `contact` comes from the sidecar; a
 * clip without one is scaled to the move. Undefined means "hold the idle pose".
 */
export function artSpecFor(
  anim: string,
  isAttack: boolean,
  clips: Readonly<Record<string, ArtClipInfo>>,
  twoHanded = false
): ClipSpec | undefined {
  let choices = ART_ANIMS[anim] ?? (isAttack ? ATTACK_FALLBACK : []);
  // Two-handed, an attack swings with both hands: heavies take the two-handed heavy, the
  // rest the two-handed slash. A character without them keeps its one-handed clips.
  if (twoHanded && isAttack) {
    const own = choices[0] === 'heavy' ? ['twohand_heavy', 'twohand_slash'] : ['twohand_slash'];
    choices = [...own, ...choices];
  }
  const id = choices.find((c) => c in clips) ?? (anim in clips ? anim : undefined);
  if (id === undefined) return undefined;
  return { segs: [{ clip: id }], contact: clips[id].contact ?? 0 };
}

/** The two-handed stance clip for a character with these clips, or undefined. */
export function artTwoHand(has: (clip: string) => boolean): StanceClip | undefined {
  return ART_TWO_HAND.find((s) => has(s.clip));
}

/** The guard stance clip for a character with these clips, or undefined. */
export function artGuard(has: (clip: string) => boolean): StanceClip | undefined {
  return ART_GUARD.find((s) => has(s.clip));
}
