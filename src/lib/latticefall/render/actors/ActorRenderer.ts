// Draws actors as skinned characters and animates them from snapshots: the player as the
// Wayfarer or an art character, and enemies as art characters (`ENEMY_ART`) or procedural
// models. Render-only: it reads snapshot fields and never writes simulation state.
//
// Player pose priority each frame: a move (by `move_id`, scrubbed by `move_frame`), then
// airborne states from the snapshot's `AIRBORNE` flag, then locomotion blended by ground
// speed. An art enemy's: death, waking (`rise`), a move, a stagger, a hit reaction, then
// locomotion. A short crossfade hides every switch between those.

import {
  Box3,
  type Bone,
  Group,
  type Material,
  Color,
  Matrix4,
  Mesh,
  MeshBasicMaterial,
  type Object3D,
  type PerspectiveCamera,
  PointLight,
  PropertyBinding,
  Quaternion,
  type Scene,
  type SkinnedMesh,
  SphereGeometry,
  Euler,
  Vector3,
  type WebGLRenderer
} from 'three';
import type { GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { createGltfLoader } from '../assets/gltfLoader';
import { getApiBase } from '$lib/apiBase';
import { clone as cloneSkinned } from 'three/examples/jsm/utils/SkeletonUtils.js';
import type { DrawActor } from '../../engine/snapshot';
import { PlayerGear, type Equipped, type RigFrame } from '../equipment/playerGear';
import { SpellFx } from '../equipment/spellFx';
import type { ModelDef } from '../procedural';
import { getQuality } from '../lighting/quality';
import { actorCullDistance } from '../perf/lod';
import { capSceneTextures } from '../perf/textures';
import { AirTracker, LAND_SECONDS } from './airState';
import {
  ARM_BONES,
  easeStance,
  GUARD_BONES,
  GUARD_FALLBACK,
  GUARD_HIPS_DROP,
  SHIELD_GUARD_CLIP,
  stanceWeights,
  TWO_HAND,
  WEAPON_GUARD_CLIP,
  type StanceClip
} from './stancePose';
import { weaponByNum, weaponNumById } from '../equipment/catalog';
import type { BoneOffset } from '../../ui/gestures/gesturePose';
import { parsePreview, type PreviewRequest } from './devPreview';
import { digestAnimInfo } from './moveTable';
import { gaitFacing, locomotionBlend, pickGait, wrapAngle, type Gait, type LocoBands } from './locomotion';
import {
  clipsForAnim,
  locateSpec,
  moveClipFrame,
  specLength,
  type ClipSpec,
  type MoveAnimInfo,
  type MoveResolver
} from './moveClips';
import { BAKE_FPS, BONE_MAP, type BakedClip, retargetClips, sampleClip } from './retarget';
import {
  artAssetBase,
  enemyArt,
  ENEMY_ART,
  lanternGlow,
  playerArt,
  ART_CHARACTERS,
  ART_LOOPS,
  DEFAULT_ART_HEIGHT_M,
  MIXAMO_BONE_MAP,
  MIXAMO_FEET,
  artGuard,
  artTwoHand,
  artSpecFor,
  type ArtCharacter,
  type ArtLantern,
  type ArtSidecar
} from './artClips';
import { parseDevParams } from '../../dev/params';
import {
  ProceduralActor,
  findChoreo,
  modelForId,
  modelForKind,
  parsePropPreview,
  type PropPreview
} from '../procedural';
import {
  FLAG_AIRBORNE,
  FLAG_CLOAKED,
  FLAG_DEAD,
  FLAG_GUARDING,
  FLAG_HIDDEN,
  FLAG_STAGGERED,
  FLAG_TWO_HANDING
} from '../../engine/snapshot';
import { gesturePose } from '../../ui/gestures/gesturePose';

/** Where the optimized assets live (`lf-tools assets` writes them). */
export const ACTOR_ASSET_BASE = '/latticefall/actors/';

/** Standing height of the Wayfarer in meters. The sim capsule is 1.7 m; the model adds hair. */
export const WAYFARER_HEIGHT_M = 1.75;
/** Added to the sim yaw. Both the sim (yaw 0 faces +Z) and the model face +Z. */
const MODEL_YAW_OFFSET = 0;

/** Reach of the weapon past the hand, for trails, in meters. Matches the light hitbox (about 1.6 m from the body). */
const WEAPON_LENGTH_M = 0.95;

/** Crossfade between pose sources, in seconds. */
const FADE_SECONDS = 0.12;
/** Crossfade out of a move into locomotion: longer, so a cut follow-through settles. */
const MOVE_EXIT_FADE_SECONDS = 0.2;
/** Jump_Start is played this much faster than authored: the jump has already begun in the sim. */
const JUMP_START_RATE = 2.5;
/** Simulation tick length. Matches `TICK_MS` in `engine/loop.ts` (30 Hz). */
const TICK_SECONDS = 1 / 30;
/**
 * After hitstop the drawn move plays this much faster than the sim until it has caught up,
 * so the follow-through snaps on and the move still ends in step with the sim.
 */
const HITSTOP_CATCHUP = 0.6;
/** A larger per-frame move than this (meters) is a teleport, not a speed. */
const TELEPORT_METERS = 3;
/** Time constant for smoothing the speed derived from position deltas, in seconds. */
const SPEED_SMOOTH_SECONDS = 0.06;
/** Dev preview lifts airborne states this high so the pose reads against the ground. */
const PREVIEW_AIR_LIFT_M = 0.6;

/**
 * The ankle travels a little slower than the sole contact it stands for, so the measured
 * stance speed under-reports the stride. Found by sweeping this factor and picking the one
 * that zeroes the planted ankle's world speed for the walk (about 1.12) and jog (about 1.17).
 */
const STANCE_CALIBRATION = 1.15;

const LOOP_CLIPS = { idle: 'Idle_Loop', walk: 'Walk_Loop', run: 'Jog_Fwd_Loop', sprint: 'Sprint_Loop' } as const;
type LoopClips = { idle: string; walk: string; run: string; sprint: string };

/** An art character without a sprint clip runs this much faster instead. */
const ART_SPRINT_RATE = 1.3;
/** An art character without a run clip walks this much faster instead. */
const ART_RUN_RATE = 1.6;

/** A dormant enemy wakes when it moves faster than this (m/s), starts a move, or is struck. */
const RISE_SPEED = 0.3;
/** The waking clip plays this much faster than authored: the sim has already sent it walking. */
const RISE_RATE = 1.6;
/** Seconds the whole waking clip takes when the enemy is already moving. */
const RISE_MOVING_SECONDS = 0.8;
/** Hit reactions play this much faster than authored, so they read as a flinch. */
const HIT_RATE = 1.3;
/** A hit reaction gives way to locomotion after this long if the enemy is already moving. */
const HIT_MIN_SECONDS = 0.25;
/** Ground speed (m/s) above which a hit reaction gives way to locomotion. */
const HIT_MOVING_SPEED = 1;
/**
 * How long an enemy with art still loading is left undrawn before its procedural model
 * stands in, in milliseconds. Long enough for the art on a normal connection.
 */
const ENEMY_ART_WAIT_MS = 4000;
/** Crossfade between an enemy's locomotion loops, in seconds: long enough to hide the step change. */
const GAIT_FADE_SECONDS = 0.25;
/** Time constant for the slower speed average that picks an enemy's loop, in seconds. */
const GAIT_SPEED_SMOOTH_SECONDS = 0.2;
/** Seconds a slower loop must stay the better fit before an enemy switches down to it. */
const GAIT_HOLD_SECONDS = 0.3;
/** Speed (m/s) a moving loop's rate is figured from at the least, so a slowdown doesn't stall it. */
const IDLE_GAIT_FLOOR = 0.3;
/** Time constant for smoothing an enemy's travel direction, in seconds. */
const HEADING_SMOOTH_SECONDS = 0.15;
/** How fast (radians per second) an enemy's drawn body turns along its path while it walks. */
const GAIT_TURN_RATE = 7;
/** How fast it turns back to the simulation's facing for a move, a hit, or a stagger. */
const ATTACK_TURN_RATE = 16;
/** Seconds the lantern takes to go out after its bearer dies. */
const LANTERN_DIE_SECONDS = 1.2;

/** What an art character (`&art=NAME`, see `artClips.ts`) adds to the template. */
interface ArtKit {
  sidecar: ArtSidecar;
  /** The guard stance clip, if the character has one. */
  guard: StanceClip | undefined;
  /** The two-handed stance clip, if the character has one. */
  twoHand: StanceClip | undefined;
}

interface Template {
  gltf: GLTF;
  /** How the game dresses this art character (`ART_CHARACTERS`), or null for the Wayfarer. */
  look: ArtCharacter | null;
  /** Reach of the weapon past the hand, for trails, in meters. */
  weaponLength: number;
  /** The idle and locomotion clip names. */
  loops: LoopClips;
  /** Set when the player is an art character instead of the Wayfarer. */
  art: ArtKit | null;
  boneNames: string[];
  clips: Map<string, BakedClip>;
  bands: LocoBands;
  /** Model-units scale to meters. */
  scale: number;
  /** Lift in model units so the feet rest on y = 0. */
  footLift: number;
  boneIndex: Map<string, number>;
}

interface Pose {
  q: Float32Array;
  hips: Float32Array;
}

interface Instance {
  holder: Group;
  bones: Bone[];
  hips: Bone;
  hipsRest: [number, number, number];
  air: AirTracker;
  cur: Pose;
  tmp: Pose;
  from: Pose;
  fadeT: number;
  /** Length of the current crossfade, in seconds. */
  fadeLen: number;
  signature: string;
  lastPos: [number, number, number] | null;
  speed: number;
  phase: number;
  idleT: number;
  lastMoveFrame: number;
  moveFrameAt: number;
  previewT: number;
  /** Hitstop: seconds left in which the drawn pose is frozen. */
  hold: number;
  /** Frames the drawn move trails the sim by, from hitstop already served in this move. */
  lagFrames: number;
  lagMove: number;
  /** Counts restarts of the same move (a repeated attack), so each restart crossfades. */
  moveRun: number;
  /** Guard and two-hand stance weights, 0 to 1, eased toward the snapshot flags. */
  guardW: number;
  twoHandW: number;
  /** The template this instance was cloned from. */
  tpl: Template;
  /** Drawn by the enemy pose path (`composeEnemyPose`). */
  enemy: boolean;
  /** Waking: lying where it fell until it first stirs, then rising (`ART_CLIP_IDS`' `rise`). */
  rise: 'dormant' | 'rising' | 'up';
  riseT: number;
  /** Seconds into the hit reaction, or null when none plays. */
  hitT: number | null;
  /** Counts hit reactions, so each restart crossfades. */
  hitRun: number;
  /** Seconds staggered, or -1 when not. */
  staggerT: number;
  /** Seconds dead, or -1 while alive. */
  deathT: number;
  /** The move being drawn and its fractional frame, for the lantern. Undefined between moves. */
  drawnMove: MoveAnimInfo | undefined;
  drawnFrame: number;
  lantern: { spec: ArtLantern; light: PointLight; flame: Mesh; material: MeshBasicMaterial; phase: number } | null;
  /** True once an enemy's weapon is attached. */
  armed: boolean;
  /** An enemy's locomotion loop (`pickGait`) and how far into it, in seconds of clip time. */
  gait: Gait;
  gaitT: number;
  /** Ground speed smoothed slowly, for choosing the loop and its rate. */
  gaitSpeed: number;
  /** A different loop `pickGait` wants, and for how many seconds it has wanted it. */
  gaitWant: Gait;
  gaitWantT: number;
  /** Smoothed ground velocity (m/s), for the travel direction. */
  vx: number;
  vz: number;
  /** The yaw an enemy is drawn at (`gaitFacing`), eased toward `faceYaw`; NaN until set. */
  drawYaw: number;
  faceYaw: number;
  /** Radians per second `drawYaw` turns at this frame. */
  faceRate: number;
}

const newPose = (n: number): Pose => ({ q: new Float32Array(n * 4), hips: new Float32Array(3) });

function copyPose(dst: Pose, src: Pose): void {
  dst.q.set(src.q);
  dst.hips.set(src.hips);
}

const qa = new Quaternion();
const gestureScale = new Vector3();
const qb = new Quaternion();

/** dst = slerp(dst, src, w). */
function blendPose(dst: Pose, src: Pose, w: number): void {
  if (w <= 0) return;
  if (w >= 1) return copyPose(dst, src);
  const n = dst.q.length / 4;
  for (let i = 0; i < n; i++) {
    qa.fromArray(dst.q, i * 4);
    qb.fromArray(src.q, i * 4);
    qa.slerp(qb, w).toArray(dst.q, i * 4);
  }
  for (let k = 0; k < 3; k++) dst.hips[k] += (src.hips[k] - dst.hips[k]) * w;
}

const smooth = (x: number) => {
  const t = Math.min(Math.max(x, 0), 1);
  return t * t * (3 - 2 * t);
};

/** Frees the bone-matrix textures of a cloned model's skeletons (WP-081: they leaked on every despawn). */
function disposeSkeletons(root: Object3D): void {
  root.traverse((o) => {
    const skinned = o as Partial<SkinnedMesh>;
    if (skinned.isSkinnedMesh) skinned.skeleton?.dispose();
  });
}

/** The bones the generic pose turns, in the order `GENERIC_KEYS` lists them. */
const GENERIC_BONES = ['Torso', 'UpperArm.R', 'LowerArm.R'] as const;
type Key = readonly (readonly [number, number, number])[];
/** Rest, wind-up, strike, follow-through, rest: Euler offsets per `GENERIC_BONES`. */
const GENERIC_KEYS: readonly Key[] = [
  [
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0]
  ],
  [
    [-0.2, -0.45, 0],
    [0, 0, -1.1],
    [-0.9, 0, 0]
  ],
  [
    [0.25, 0.55, 0],
    [0, 0, 0.5],
    [-0.2, 0, 0]
  ],
  [
    [0.1, 0.15, 0],
    [0, 0, 0.2],
    [0, 0, 0]
  ],
  [
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0]
  ]
];

/** Move frames of the `GENERIC_KEYS`: the strike lands on the hitbox, or on thirds without one. */
function genericKeys(info: MoveAnimInfo): number[] {
  const total = Math.max(info.totalFrames, 1);
  const strike = info.strikeStart ?? total / 3;
  const recovery = Math.max(info.recoveryStart ?? (total * 2) / 3, strike + 1);
  const windup = Math.max(strike - Math.min(4, strike * 0.4), strike * 0.6);
  return [0, windup, (strike + recovery) / 2, Math.min(recovery + 3, total), total];
}

/**
 * Loads an art character baked by the pipeline: the GLB's own clips on its own skeleton, so
 * the bake is an identity retarget that only resamples them to 30 Hz arrays and measures
 * the strides. Its bones are found by Mixamo name (`MIXAMO_BONE_MAP`), and it is scaled to
 * its `ART_CHARACTERS` height. `idle` and `walk` are required; a missing `run` is a faster
 * walk, and a missing `sprint` a faster run.
 */
async function loadArtTemplate(renderer: WebGLRenderer, name: string): Promise<Template> {
  const loader = createGltfLoader(renderer);
  const base = artAssetBase(name, import.meta.env.DEV, getApiBase());
  const [gltf, sidecar] = await Promise.all([
    loader.loadAsync(`${base}${name}.glb`),
    fetch(`${base}${name}.json`).then((r) => {
      if (!r.ok) throw new Error(`${name}.json: HTTP ${r.status}`);
      return r.json() as Promise<ArtSidecar>;
    })
  ]);
  capSceneTextures(gltf.scene, getQuality().name);
  gltf.scene.updateMatrixWorld(true);
  const look = ART_CHARACTERS[name] ?? null;
  const box = new Box3().setFromObject(gltf.scene, true);
  const scale = (look?.height ?? DEFAULT_ART_HEIGHT_M) / (box.max.y - box.min.y);

  // Every bone plays its clip, the mapped ones first so `boneIndex` finds them.
  const core = new Set(MIXAMO_BONE_MAP.map(([m]) => m));
  const extra: string[] = [];
  gltf.scene.traverse((o) => {
    if ((o as Bone).isBone && !core.has(o.name)) extra.push(o.name);
  });
  const map = [...MIXAMO_BONE_MAP.map(([m]) => [m, m] as const), ...extra.map((n) => [n, n] as const)];
  const source = cloneSkinned(gltf.scene) as Object3D;
  const rt = retargetClips(source, gltf.animations, gltf.scene, scale, map, MIXAMO_FEET);
  for (const id of [ART_LOOPS.idle, ART_LOOPS.walk]) {
    if (!rt.clips.has(id)) throw new Error(`${name}.glb has no clip ${id}`);
  }
  const band = (id: string) => {
    const clip = rt.clips.get(id)!;
    if (clip.stanceSpeed <= 0) throw new Error(`no measurable stride for clip ${id}`);
    const speed = clip.stanceSpeed * STANCE_CALIBRATION * rt.strideScale;
    return { metersPerCycle: speed * clip.duration, speed };
  };
  // A sidecar from another bake must not name clips the GLB lacks.
  sidecar.clips = Object.fromEntries(Object.entries(sidecar.clips).filter(([id]) => rt.clips.has(id)));
  const walk = band(ART_LOOPS.walk);
  const hasRun = rt.clips.has(ART_LOOPS.run);
  const run = hasRun ? band(ART_LOOPS.run) : { metersPerCycle: walk.metersPerCycle, speed: walk.speed * ART_RUN_RATE };
  const runClip = hasRun ? ART_LOOPS.run : ART_LOOPS.walk;
  const hasSprint = rt.clips.has(ART_LOOPS.sprint);
  console.info(`[latticefall] art character ${name}: ${sidecar.triangles} triangles, clips ${[...rt.clips.keys()].join(', ')}`);
  return {
    gltf,
    look,
    weaponLength: look?.weaponLength ?? WEAPON_LENGTH_M,
    loops: { ...ART_LOOPS, run: runClip, sprint: hasSprint ? ART_LOOPS.sprint : runClip },
    art: { sidecar, guard: artGuard((c) => rt.clips.has(c)), twoHand: artTwoHand((c) => rt.clips.has(c)) },
    boneNames: rt.boneNames,
    clips: rt.clips,
    bands: {
      walk,
      run,
      sprint: hasSprint ? band(ART_LOOPS.sprint) : { metersPerCycle: run.metersPerCycle, speed: run.speed * ART_SPRINT_RATE }
    },
    scale,
    footLift: -box.min.y,
    boneIndex: new Map(MIXAMO_BONE_MAP.map(([, logical], i) => [logical, i]))
  };
}

/** Frees a template's geometry and materials. */
function disposeTemplate(tpl: Template): void {
  tpl.gltf.scene.traverse((o) => {
    const m = o as { geometry?: { dispose(): void }; material?: { dispose(): void } | Array<{ dispose(): void }> };
    m.geometry?.dispose();
    if (Array.isArray(m.material)) m.material.forEach((x) => x.dispose());
    else m.material?.dispose();
  });
}

export class ActorRenderer {
  private scene: Scene;
  private renderer: WebGLRenderer;
  private resolveMove: MoveResolver;
  private tpl: Template | null = null;
  private instances = new Map<number, Instance>();
  private preview: PreviewRequest | null = null;
  private warned = new Set<string>();
  private loading: Promise<void> | null = null;
  private sample: Pose | null = null;
  /** Enemies drawn by procedural models, by actor id. They need no assets, so they draw at once. */
  private enemies = new Map<number, ProceduralActor>();
  private propPreview: PropPreview | null = null;
  private propActors: ProceduralActor[] = [];
  private propT = 0;
  /** WP-058: what the player wears, and the catalyst effects that go with it. */
  private gear = new Map<number, PlayerGear>();
  private equipped: Equipped | null = null;
  /** The actor id the equipment belongs to (the snapshot's first actor), or -1. */
  playerId = -1;
  /** The gesture the player is performing, from the core's gesture events (WP-088). */
  private gesture: number | null = null;
  private gestureT = 0;
  readonly spellFx = new SpellFx();
  /** The art character to draw as the player (`PLAYER_ART`, or `&art=` with `&dev=1`), or null. */
  private artName: string | null = null;
  /** Enemy art characters by name: the loaded template, or the load's state. */
  private enemyTpls = new Map<string, Template | 'loading' | 'failed'>();
  /** `&enemyart=off` turns the enemy art characters off. */
  private enemyArtParam: string | null = null;

  constructor(scene: Scene, renderer: WebGLRenderer, opts: { resolveMove?: MoveResolver } = {}) {
    this.scene = scene;
    this.renderer = renderer;
    this.resolveMove = opts.resolveMove ?? (() => undefined);
    if (typeof location !== 'undefined') {
      this.preview = parsePreview(location.search);
      this.propPreview = parsePropPreview(location.search);
      const params = parseDevParams(location.search);
      this.artName = playerArt(params.art);
      this.enemyArtParam = params.enemyArt;
    }
    this.scene.add(this.spellFx.points);
    if (this.preview || import.meta.env.DEV) (globalThis as Record<string, unknown>).__lfActors = this;
  }

  /** Starts (or, with null, stops) drawing the player's gesture. The core decides when. */
  setGesture(gesture: number | null): void {
    if (gesture !== this.gesture) this.gestureT = 0;
    this.gesture = gesture;
  }

  /** True once the model and clips are loaded and `update` draws. */
  get ready(): boolean {
    return this.tpl !== null;
  }

  /** Starts loading; safe to call more than once. Rejects if an asset fails to load. */
  load(): Promise<void> {
    if (!this.loading && this.enemyArtParam !== 'off') {
      // The enemies' art loads beside the player's, so it is usually ready before they show.
      for (const name of new Set(ENEMY_ART.values())) this.startEnemyLoad(name);
    }
    this.loading ??= this.doLoad();
    return this.loading;
  }

  private async doLoad(): Promise<void> {
    if (this.artName) {
      try {
        await this.loadArt(this.artName);
        return;
      } catch (err) {
        console.error(`[latticefall] art character "${this.artName}" failed to load; drawing the Wayfarer`, err);
      }
    }
    const loader = createGltfLoader(this.renderer);
    const [wayfarer, ual, ual2] = await Promise.all([
      loader.loadAsync(`${ACTOR_ASSET_BASE}wayfarer.glb`),
      loader.loadAsync(`${ACTOR_ASSET_BASE}ual.glb`),
      loader.loadAsync(`${ACTOR_ASSET_BASE}ual2.glb`).catch(() => null)
    ]);
    // Texture budget (WP-081): nothing here is above the preset's longest edge today; this keeps it so.
    capSceneTextures(wayfarer.scene, getQuality().name);

    // Measure the rest pose with no transform on the scene.
    wayfarer.scene.updateMatrixWorld(true);
    const box = new Box3().setFromObject(wayfarer.scene, true);
    const height = box.max.y - box.min.y;
    const scale = WAYFARER_HEIGHT_M / height;

    const rt = retargetClips(ual.scene, ual.animations, wayfarer.scene, scale);
    if (ual2) {
      for (const [name, clip] of retargetClips(ual2.scene, ual2.animations, wayfarer.scene, scale).clips) {
        rt.clips.set(name, clip);
      }
    }
    const band = (name: string) => {
      const clip = rt.clips.get(name);
      if (!clip || clip.stanceSpeed <= 0) throw new Error(`no measurable stride for clip ${name}`);
      // Stride scales with leg length; the clip's own cycle time stays.
      const speed = clip.stanceSpeed * STANCE_CALIBRATION * rt.strideScale;
      return { metersPerCycle: speed * clip.duration, speed };
    };
    this.tpl = {
      gltf: wayfarer,
      look: null,
      weaponLength: WEAPON_LENGTH_M,
      loops: LOOP_CLIPS,
      art: null,
      boneNames: rt.boneNames,
      clips: rt.clips,
      bands: { walk: band(LOOP_CLIPS.walk), run: band(LOOP_CLIPS.run), sprint: band(LOOP_CLIPS.sprint) },
      scale,
      footLift: -box.min.y,
      boneIndex: new Map(BONE_MAP.map(([, t], i) => [t, i]))
    };
    for (const name of Object.values(LOOP_CLIPS)) {
      if (!rt.clips.has(name)) throw new Error(`animation library has no clip ${name}`);
    }
  }

  /** Loads an art character as the player's template (see `loadArtTemplate`). */
  private async loadArt(name: string): Promise<void> {
    this.tpl = await loadArtTemplate(this.renderer, name);
  }

  /** Draws one actor. Returns false (and draws nothing) until the assets are loaded. */
  update(a: DrawActor, frameMs: number): boolean {
    const def = a.kind ? modelForKind(a.kind) : undefined;
    if (def) {
      const art = this.enemyTemplate(a.kind);
      if (art === 'wait') return true;
      if (art) this.updateArtEnemy(a, def, art, frameMs);
      else this.updateEnemy(a, def, frameMs);
      return true;
    }
    const tpl = this.tpl;
    if (!tpl) return false;
    const inst = this.instances.get(a.id) ?? this.spawn(a.id, tpl, false);
    const pv = this.preview;
    if (!this.step(inst, a, frameMs)) return true;
    if (a.id === this.playerId) {
      // An art character with its own weapon (a boss drawn as the player for a preview) holds it.
      const own = this.ownWeapon(tpl);
      if (own) this.applyGear(a.id, inst, own, tpl.look?.weaponScale);
      else if (this.equipped) {
        this.applyGear(a.id, inst, this.equipped);
        this.gear.get(a.id)?.setOffHandStowed((a.flags & FLAG_TWO_HANDING) !== 0);
      }
    }
    if (pv && (pv.state === 'jump' || pv.state === 'fall' || pv.state === 'land')) {
      inst.holder.position.y += pv.state === 'land' ? 0 : PREVIEW_AIR_LIFT_M;
    }
    return true;
  }

  /**
   * Moves, times, and poses one skinned instance for this frame. Returns false while the
   * actor is held by hitstop (its last pose stays).
   */
  private step(inst: Instance, a: DrawActor, frameMs: number): boolean {
    const dt = Math.max(frameMs, 0) / 1000;
    inst.holder.position.set(a.x, a.y, a.z);
    inst.holder.rotation.y = a.yaw + MODEL_YAW_OFFSET;

    // Ground speed from position deltas (the snapshot has none).
    if (inst.lastPos && dt > 0) {
      const dx = a.x - inst.lastPos[0];
      const dz = a.z - inst.lastPos[2];
      const far = Math.hypot(dx, dz) > TELEPORT_METERS;
      const raw = far ? 0 : Math.hypot(dx, dz) / dt;
      const k = 1 - Math.exp(-dt / SPEED_SMOOTH_SECONDS);
      inst.speed += (raw - inst.speed) * k;
      // The travel direction needs a steadier signal than the speed: the snapshot moves in
      // 30 Hz steps, so per-frame deltas alternate between a step and nothing.
      const kv = 1 - Math.exp(-dt / HEADING_SMOOTH_SECONDS);
      inst.vx += ((far ? 0 : dx / dt) - inst.vx) * kv;
      inst.vz += ((far ? 0 : dz / dt) - inst.vz) * kv;
    }
    inst.lastPos = [a.x, a.y, a.z];

    // Hitstop: hold the last drawn pose. The sim keeps ticking, so what was held is owed to
    // the move: it plays that many frames behind until the move ends.
    if (a.moveId !== inst.lagMove || a.moveFrame < inst.lastMoveFrame) {
      inst.lagMove = a.moveId;
      inst.lagFrames = 0;
      if (a.moveId !== 0) inst.moveRun++;
    }
    if (inst.hold > 0) {
      const step = Math.min(dt, inst.hold);
      inst.hold -= step;
      inst.lagFrames += step / TICK_SECONDS;
      return false;
    }
    inst.lagFrames = Math.max(0, inst.lagFrames - (dt / TICK_SECONDS) * HITSTOP_CATCHUP);
    if (dt > 0 && !inst.enemy) inst.air.update(a.y, dt, (a.flags & FLAG_AIRBORNE) !== 0);

    const pv = inst.enemy ? null : this.preview;
    if (pv) this.applyPreview(inst, pv, dt);

    // Move frame, made fractional between ticks.
    const now = performance.now();
    if (a.moveFrame !== inst.lastMoveFrame) {
      inst.lastMoveFrame = a.moveFrame;
      inst.moveFrameAt = now;
    }
    const moveFrac = Math.min((now - inst.moveFrameAt) / 1000 / TICK_SECONDS, 1);

    inst.drawnMove = undefined;
    const sig = inst.enemy ? this.composeEnemyPose(inst, a, dt, moveFrac) : this.composePose(inst, a, inst.tpl, dt, moveFrac);
    this.crossfade(inst, sig, dt);
    this.apply(inst);
    if (inst.enemy) {
      // Ease the drawn yaw toward where the pose wants to face (`composeEnemyPose`).
      if (Number.isNaN(inst.drawYaw)) inst.drawYaw = a.yaw;
      const turn = wrapAngle(inst.faceYaw - inst.drawYaw);
      const most = inst.faceRate * dt;
      inst.drawYaw = wrapAngle(inst.drawYaw + Math.max(-most, Math.min(most, turn)));
      inst.holder.rotation.y = inst.drawYaw + MODEL_YAW_OFFSET;
    }
    this.updateLantern(inst, a, dt);
    return true;
  }

  /** The weapon an art character always holds (`ArtCharacter.weapon`), as equipment, or null. */
  private ownWeapon(tpl: Template): Equipped | null {
    const id = tpl.look?.weapon;
    if (!id) return null;
    const right = weaponNumById(id);
    return right ? { right, left: 0, head: 0, body: 0 } : null;
  }

  /**
   * The art template for an enemy kind once it's loaded. While it loads, `'wait'` (draw
   * nothing, so the enemy first appears as its art character, lying where it fell) for up to
   * `ENEMY_ART_WAIT_MS`, then undefined: the procedural model draws until the art arrives,
   * and for good if the load fails.
   */
  private enemyTemplate(kind: number): Template | 'wait' | undefined {
    const name = enemyArt(kind, this.enemyArtParam);
    if (!name) return undefined;
    const state = this.enemyTpls.get(name) ?? this.startEnemyLoad(name);
    if (state === 'loading') return performance.now() - this.enemyLoadAt < ENEMY_ART_WAIT_MS ? 'wait' : undefined;
    return state === 'failed' ? undefined : state;
  }

  /** When the first enemy art load started, for `ENEMY_ART_WAIT_MS`. */
  private enemyLoadAt = 0;

  private startEnemyLoad(name: string): 'loading' {
    if (this.enemyTpls.size === 0) this.enemyLoadAt = performance.now();
    this.enemyTpls.set(name, 'loading');
    loadArtTemplate(this.renderer, name).then(
      (tpl) => {
        if (this.enemyTpls.get(name) === 'loading') this.enemyTpls.set(name, tpl);
        else disposeTemplate(tpl);
      },
      (err) => {
        console.error(`[latticefall] enemy art character "${name}" failed to load; keeping its procedural model`, err);
        if (this.enemyTpls.get(name) === 'loading') this.enemyTpls.set(name, 'failed');
      }
    );
    return 'loading';
  }

  /** Draws one enemy as its art character. Takes over from its procedural model, if it had one. */
  private updateArtEnemy(a: DrawActor, def: ModelDef, tpl: Template, frameMs: number): void {
    const old = this.enemies.get(a.id);
    if (old) {
      this.scene.remove(old.holder);
      old.dispose();
      this.enemies.delete(a.id);
    }
    let inst = this.instances.get(a.id);
    // Beyond the cull distance an enemy is neither posed nor drawn (WP-081). Bosses are exempt.
    const cam = this.cameraAt;
    if (cam && def.height < 3) {
      const limit = actorCullDistance(getQuality());
      const dx = a.x - cam.x;
      const dz = a.z - cam.z;
      if (dx * dx + dz * dz > limit * limit) {
        if (inst) inst.holder.visible = false;
        return;
      }
    }
    if (!inst) {
      inst = this.spawn(a.id, tpl, true);
      // Already on screen as its procedural model: it is awake, so it doesn't lie down.
      if (old) inst.rise = 'up';
    }
    inst.holder.visible = (a.flags & FLAG_HIDDEN) === 0;
    if (!this.step(inst, a, frameMs)) return;
    if (!inst.armed) {
      inst.armed = true;
      const own = this.ownWeapon(tpl);
      if (own) this.applyGear(a.id, inst, own, tpl.look?.weaponScale);
    }
  }

  /** Plays a hit reaction on an art enemy (from a hit event). The pose path decides if it shows. */
  hitReact(id: number): void {
    const inst = this.instances.get(id);
    if (!inst?.enemy || (inst.deathT >= 0)) return;
    inst.hitT = 0;
    inst.hitRun++;
  }

  /** Sets what the player wears (registry numbers from the menu state). Drawn on the next frame. */
  setEquipment(eq: Equipped): void {
    this.equipped = eq;
  }

  /** Advances the catalyst effects. Call once per rendered frame, before the scene draws. */
  updateFx(dt: number, camera: PerspectiveCamera, viewportPx: number): void {
    (this.cameraAt ??= new Vector3()).copy(camera.position);
    this.spellFx.update(dt, camera, viewportPx);
  }

  private applyGear(id: number, inst: Instance, eq: Equipped, weaponScale = 1): void {
    const tpl = inst.tpl;
    let g = this.gear.get(id);
    if (!g) {
      g = new PlayerGear(this.spellFx);
      this.gear.set(id, g);
    }
    // The template's materials are shared by every Wayfarer, so tint a private copy.
    let cloned = false;
    const materials = () => {
      const out: Material[] = [];
      inst.holder.traverse((o) => {
        const m = o as Mesh;
        if (!m.isMesh) return;
        if (!cloned) m.material = Array.isArray(m.material) ? m.material.map((x) => x.clone()) : m.material.clone();
        out.push(...(Array.isArray(m.material) ? m.material : [m.material]));
      });
      cloned = true;
      return out;
    };
    const named = (n: string) => tpl.boneNames[tpl.boneIndex.get(n)!];
    const frame: RigFrame = {
      bodyToTemplate: new Matrix4()
        .makeTranslation(0, -tpl.footLift, 0)
        .multiply(new Matrix4().makeScale(weaponScale / tpl.scale, weaponScale / tpl.scale, weaponScale / tpl.scale)),
      restBone: (n) => tpl.gltf.scene.getObjectByName(PropertyBinding.sanitizeNodeName(named(n)))!,
      bone: (n) => inst.bones[tpl.boneIndex.get(n)!],
      materials,
      // An art character's grips come from its bake (the clips' own prop), in template space.
      grip: (side) => {
        const g = tpl.art?.sidecar.grips?.[side];
        return g ? new Matrix4().fromArray(g) : undefined;
      }
    };
    tpl.gltf.scene.updateMatrixWorld(true);
    // An art character wears its own armor: it carries the weapons only.
    g.apply(tpl.art ? { ...eq, head: 0, body: 0 } : eq, frame);
  }

  /** Hitstop: freezes the actor's drawn animation for `seconds` (never longer than a quarter second). */
  holdPose(id: number, seconds: number): void {
    const inst = this.instances.get(id) ?? this.enemies.get(id);
    if (inst) inst.hold = Math.min(0.25, Math.max(inst.hold, seconds));
  }

  isHeld(id: number): boolean {
    return (this.instances.get(id)?.hold ?? this.enemies.get(id)?.hold ?? 0) > 0;
  }

  /**
   * World positions of the weapon's base (the right hand) and tip: the forearm's direction
   * carried on by `WEAPON_LENGTH_M`. The model has no weapon mesh yet, so this stands in for
   * the blade. Returns false until the actor is drawn.
   */
  weaponTip(id: number, tip: Vector3, base: Vector3): boolean {
    const inst = this.instances.get(id);
    if (!inst || !inst.holder.visible) return false;
    const tpl = inst.tpl;
    inst.holder.updateMatrixWorld(true);
    const hand = inst.bones[tpl.boneIndex.get('Hand.R')!];
    const fore = inst.bones[tpl.boneIndex.get('LowerArm.R')!];
    hand.getWorldPosition(base);
    fore.getWorldPosition(tip);
    tip.sub(base).negate().normalize().multiplyScalar(tpl.weaponLength).add(base);
    return true;
  }

  /** Camera offset from the followed actor for `&animcam=`, or null when not previewing. */
  previewCameraOffset(): [number, number, number] | null {
    switch (this.preview?.cam) {
      case 'front':
        return [0, 1.3, 4.2];
      case 'back':
        return [0, 1.3, -4.2];
      case 'side':
        return [4.2, 1.0, 0];
      case 'three':
        return [3, 1.8, 3.4];
      default:
        return null;
    }
  }

  /** Removes instances for actors no longer in the snapshot. */
  prune(live: Set<number>): void {
    for (const [id, e] of this.enemies) {
      if (!live.has(id)) {
        this.scene.remove(e.holder);
        e.dispose();
        this.enemies.delete(id);
      }
    }
    for (const [id, inst] of this.instances) {
      if (!live.has(id)) {
        this.gear.get(id)?.dispose();
        this.gear.delete(id);
        this.removeInstance(inst);
        this.instances.delete(id);
      }
    }
  }

  dispose(): void {
    for (const g of this.gear.values()) g.dispose();
    this.gear.clear();
    this.scene.remove(this.spellFx.points);
    this.spellFx.dispose();
    for (const inst of this.instances.values()) this.removeInstance(inst);
    this.instances.clear();
    for (const e of [...this.enemies.values(), ...this.propActors]) {
      this.scene.remove(e.holder);
      e.dispose();
    }
    this.enemies.clear();
    this.propActors = [];
    if (this.tpl) disposeTemplate(this.tpl);
    this.tpl = null;
    for (const state of this.enemyTpls.values()) if (typeof state !== 'string') disposeTemplate(state);
    // A load still in flight finds its slot gone and frees what it loaded.
    this.enemyTpls.clear();
    if ((globalThis as Record<string, unknown>).__lfActors === this) delete (globalThis as Record<string, unknown>).__lfActors;
  }

  /** Dev helper: world positions of an actor's feet, to check for sliding. */
  debugFeet(id: number): { l: [number, number, number]; r: [number, number, number]; speed: number } | null {
    const inst = this.instances.get(id);
    if (!inst) return null;
    inst.holder.updateMatrixWorld(true);
    const get = (name: string): [number, number, number] => {
      const b = inst.bones[inst.tpl.boneIndex.get(name)!];
      const p = b.getWorldPosition(new Vector3());
      return [p.x, p.y, p.z];
    };
    return { l: get('Foot.L'), r: get('Foot.R'), speed: inst.speed };
  }

  /** Where the camera was at the last `updateFx`, for the enemies' distance cull. Null before it. */
  private cameraAt: Vector3 | null = null;

  /** Draws one enemy as its procedural model, posed from the snapshot's move and flags. */
  private updateEnemy(a: DrawActor, def: ModelDef, frameMs: number): void {
    let e = this.enemies.get(a.id);
    // Beyond the cull distance an enemy is neither posed nor drawn (WP-081). Bosses are exempt.
    const cam = this.cameraAt;
    if (cam && def.height < 3) {
      const limit = actorCullDistance(getQuality());
      const dx = a.x - cam.x;
      const dz = a.z - cam.z;
      if (dx * dx + dz * dz > limit * limit) {
        if (e) e.holder.visible = false;
        return;
      }
    }
    if (!e) {
      e = new ProceduralActor(def, a.id);
      this.enemies.set(a.id, e);
      this.scene.add(e.holder);
    }
    const info = a.moveId !== 0 ? this.resolveMove(a.moveId) : undefined;
    const frac = Math.min((performance.now() - this.enemyFrameAt(a)) / 1000 / TICK_SECONDS, 1);
    const move = info ? { anim: info.anim, total: info.totalFrames, frame: a.moveFrame + frac } : undefined;
    e.update(
      {
        x: a.x,
        y: a.y,
        z: a.z,
        yaw: a.yaw,
        dead: (a.flags & FLAG_DEAD) !== 0,
        hidden: (a.flags & FLAG_HIDDEN) !== 0,
        cloaked: (a.flags & FLAG_CLOAKED) !== 0,
        staggered: (a.flags & FLAG_STAGGERED) !== 0
      },
      move,
      Math.max(frameMs, 0) / 1000
    );
  }

  private enemyFrames = new Map<number, { frame: number; at: number }>();
  /** When the actor's `move_frame` last changed, to blend the frame between 30 Hz ticks. */
  private enemyFrameAt(a: DrawActor): number {
    const now = performance.now();
    const seen = this.enemyFrames.get(a.id);
    if (!seen || seen.frame !== a.moveFrame) {
      this.enemyFrames.set(a.id, { frame: a.moveFrame, at: now });
      return now;
    }
    return seen.at;
  }

  /** Draws the `?preview=` models (see `procedural/preview.ts`). Call once per rendered frame. */
  tickPreview(frameMs: number): void {
    const pv = this.propPreview;
    if (!pv) return;
    if (this.propActors.length === 0) {
      const defs = pv.ids.map((id) => modelForId(id)).filter((d): d is ModelDef => !!d);
      defs.forEach((d, i) => {
        const pa = new ProceduralActor(d, i);
        this.scene.add(pa.holder);
        this.propActors.push(pa);
      });
    }
    const dt = Math.max(frameMs, 0) / 1000;
    this.propT += dt;
    const n = this.propActors.length;
    this.propActors.forEach((pa, i) => {
      const def = pa.def;
      let move: { anim: string; total: number; frame: number } | undefined;
      if (pv.move !== null) {
        const keys = [...new Set(Object.values(def.moves))].map((c) => Object.keys(def.moves).find((k) => def.moves[k] === c)!);
        const idx = Number(pv.move);
        const anim = Number.isInteger(idx) ? keys[idx % keys.length] : pv.move;
        const c = findChoreo(def, anim);
        const frame = pv.t !== undefined ? pv.t * c.total : (this.propT * 30) % (c.total + 30);
        move = { anim, total: c.total, frame };
      }
      pa.update(
        { x: (i - (n - 1) / 2) * 3, y: 0, z: pv.z, yaw: Math.PI, dead: pv.dead },
        move,
        // A frozen preview still plays out the fall in one step.
        pv.dead ? 1 : pv.t !== undefined ? 0 : dt,
        pv.speed ?? 0
      );
    });
  }

  // -- internals ------------------------------------------------------------------------

  private spawn(id: number, tpl: Template, enemy: boolean): Instance {
    const root = cloneSkinned(tpl.gltf.scene) as Object3D;
    root.position.y = tpl.footLift;
    const holder = new Group();
    holder.scale.setScalar(tpl.scale);
    const outer = new Group();
    outer.add(holder);
    holder.add(root);
    // `outer` is the transform the sim drives; `holder` carries the unit scale.
    this.scene.add(outer);
    const bones = tpl.boneNames.map((n) => root.getObjectByName(PropertyBinding.sanitizeNodeName(n)) as Bone);
    const hips = bones[0];
    const n = bones.length;
    const inst: Instance = {
      holder: outer,
      bones,
      hips,
      hipsRest: [hips.position.x, hips.position.y, hips.position.z],
      air: new AirTracker(),
      cur: newPose(n),
      tmp: newPose(n),
      from: newPose(n),
      fadeT: FADE_SECONDS,
      fadeLen: FADE_SECONDS,
      signature: '',
      lastPos: null,
      speed: 0,
      phase: 0,
      idleT: 0,
      lastMoveFrame: -1,
      moveFrameAt: 0,
      previewT: 0,
      hold: 0,
      lagFrames: 0,
      lagMove: 0,
      moveRun: 0,
      guardW: 0,
      twoHandW: 0,
      tpl,
      enemy,
      rise: enemy && tpl.clips.has('rise') ? 'dormant' : 'up',
      riseT: 0,
      hitT: null,
      hitRun: 0,
      staggerT: -1,
      deathT: -1,
      drawnMove: undefined,
      drawnFrame: 0,
      lantern: null,
      armed: false,
      gait: 'idle',
      gaitT: 0,
      gaitSpeed: 0,
      gaitWant: 'idle',
      gaitWantT: 0,
      vx: 0,
      vz: 0,
      drawYaw: NaN,
      faceYaw: 0,
      faceRate: ATTACK_TURN_RATE
    };
    if (tpl.look?.lantern) inst.lantern = this.makeLantern(tpl, root, tpl.look.lantern, id);
    this.instances.set(id, inst);
    return inst;
  }

  /**
   * A chest lantern on its bone: a flame sphere and a point light, placed `forward` and `up`
   * meters from the bone at rest. The offset is found in the template's rest pose, so it
   * holds for any bone orientation the bake produced.
   */
  private makeLantern(tpl: Template, root: Object3D, spec: ArtLantern, id: number): Instance['lantern'] {
    const key = PropertyBinding.sanitizeNodeName(spec.bone);
    const rest = tpl.gltf.scene.getObjectByName(key);
    const bone = root.getObjectByName(key);
    if (!rest || !bone) {
      console.warn(`[latticefall] lantern bone ${spec.bone} not found on ${tpl.art?.sidecar.name}`);
      return null;
    }
    tpl.gltf.scene.updateMatrixWorld(true);
    const at = rest.getWorldPosition(new Vector3());
    at.z += spec.forward / tpl.scale;
    at.y += spec.up / tpl.scale;
    const local = rest.worldToLocal(at);
    const boneScale = rest.getWorldScale(new Vector3()).x;
    const material = new MeshBasicMaterial({ color: new Color(spec.color), toneMapped: false });
    const flame = new Mesh(new SphereGeometry(spec.radius / tpl.scale / boneScale, 12, 8), material);
    flame.position.copy(local);
    const light = new PointLight(spec.color, spec.intensity, spec.distance, 1.6);
    light.position.copy(local);
    bone.add(flame, light);
    return { spec, light, flame, material, phase: (id % 13) * 0.7 };
  }

  /** Brightens the lantern through a wind-up (`lanternGlow`), flickers it, and puts it out on death. */
  private updateLantern(inst: Instance, a: DrawActor, dt: number): void {
    const l = inst.lantern;
    if (!l) return;
    l.phase += dt;
    const flicker = 1 + 0.07 * Math.sin(l.phase * 11) + 0.05 * Math.sin(l.phase * 23.7);
    let glow = lanternGlow(inst.drawnMove, inst.drawnFrame) * flicker;
    if ((a.flags & FLAG_DEAD) !== 0) glow *= Math.max(0, 1 - Math.max(inst.deathT, 0) / LANTERN_DIE_SECONDS);
    l.light.intensity = l.spec.intensity * glow;
    // Past rest brightness the flame washes toward white.
    l.material.color.setHex(l.spec.color).multiplyScalar(Math.max(glow, 0.05));
    l.flame.scale.setScalar(0.8 + 0.2 * Math.min(glow, 2));
  }

  private removeInstance(inst: Instance): void {
    this.scene.remove(inst.holder);
    disposeSkeletons(inst.holder);
    if (inst.lantern) {
      inst.lantern.flame.geometry.dispose();
      inst.lantern.material.dispose();
      inst.lantern.light.dispose();
    }
  }

  private applyPreview(inst: Instance, pv: PreviewRequest, dt: number): void {
    const tpl = inst.tpl;
    const s = pv.state;
    inst.previewT += dt;
    const t = pv.t ?? inst.previewT % 1.5;
    if (s === 'jump') inst.air.force('rising', t);
    else if (s === 'fall') inst.air.force('falling', t);
    else if (s === 'land') inst.air.force('landing', Math.min(t, LAND_SECONDS));
    else inst.air.force('ground', 0);
    if (s === 'idle') inst.speed = 0;
    else if (s === 'walk') inst.speed = pv.speed ?? tpl.bands.walk.speed;
    else if (s === 'run') inst.speed = pv.speed ?? tpl.bands.run.speed;
    else if (s === 'sprint') inst.speed = pv.speed ?? tpl.bands.sprint.speed;
    if (typeof s === 'string' && pv.t !== undefined && ['idle', 'walk', 'run', 'sprint'].includes(s)) {
      inst.phase = pv.t;
      inst.idleT = pv.t;
    }
  }

  /** Fills `inst.tmp` with this frame's target pose and returns its signature. */
  private composePose(inst: Instance, a: DrawActor, tpl: Template, dt: number, moveFrac: number): string {
    const out = inst.tmp;
    const pv = this.preview;
    const frozen = pv?.t !== undefined;

    // 0. A gesture (WP-088): the idle clip with bone offsets. A move always wins.
    if (this.gesture !== null && a.id === this.playerId && a.moveId === 0 && !pv) {
      this.gestureT += dt;
      this.gestureFrame(inst, tpl, out, this.gesture, this.gestureT);
      return `gesture:${this.gesture}`;
    }

    // 1. A move, from move_id and move_frame.
    let info = a.moveId !== 0 ? this.resolveMove(a.moveId) : undefined;
    let frame = Math.max(0, a.moveFrame + moveFrac - inst.lagFrames);
    if (pv && typeof pv.state === 'object') {
      // A move plays with its own frame data; a raw clip name over 30 frames.
      info = digestAnimInfo(pv.state.move) ?? { anim: pv.state.move, totalFrames: 30 };
      const total = info.totalFrames;
      frame = (pv.t ?? ((performance.now() / 1000 / (total * TICK_SECONDS)) % 1)) * total;
    }
    if (info) {
      const sig = `move:${info.anim}:${inst.moveRun}`;
      const choice = this.pickSpec(tpl, info.anim, !!pv, info, (a.flags & FLAG_TWO_HANDING) !== 0);
      if (choice) {
        const frames = (c: string) => tpl.clips.get(c)!.frames;
        const len = specLength(choice, frames);
        const at = locateSpec(choice, moveClipFrame(len, choice.contact, info, frame), frames, BAKE_FPS);
        sampleClip(tpl.clips.get(at.clip)!, at.time, false, out.q, out.hips);
        inst.drawnMove = info;
        inst.drawnFrame = frame;
        return sig;
      }
      this.warnMissing(info.anim, tpl);
      if (tpl.art) {
        // The generic pose's offsets are for the Wayfarer's bone axes: hold the idle instead.
        sampleClip(tpl.clips.get(tpl.loops.idle)!, 0, true, out.q, out.hips);
      } else {
        this.genericPose(out, tpl, info, frame);
      }
      return sig;
    }

    // 2. Airborne.
    const air = inst.air;
    if (air.phase === 'rising' || air.phase === 'falling') {
      const start = tpl.clips.get('Jump_Start');
      const loop = tpl.clips.get('Jump_Loop');
      if (air.phase === 'rising' && start && air.t * JUMP_START_RATE < start.duration) {
        sampleClip(start, air.t * JUMP_START_RATE, false, out.q, out.hips);
        return 'air:rise';
      }
      if (loop) {
        sampleClip(loop, air.t, true, out.q, out.hips);
        return 'air:loop';
      }
    }
    // 3. Locomotion, with the landing blended into it, and the guard or two-hand stance on top.
    this.locomotion(inst, tpl, dt, frozen, out);
    let sig = 'loco';
    if (air.phase === 'landing') {
      const land = tpl.clips.get('Jump_Land');
      if (land) {
        const sample = (this.sample ??= newPose(inst.bones.length));
        sampleClip(land, (air.t / LAND_SECONDS) * land.duration * 0.5, false, sample.q, sample.hips);
        // Full landing pose at touchdown, easing into locomotion over LAND_SECONDS.
        blendPose(sample, out, smooth(air.t / LAND_SECONDS));
        copyPose(out, sample);
        sig = 'air:land';
      }
    }
    this.layStance(inst, tpl, a, dt, out);
    return sig;
  }

  /**
   * An enemy's locomotion: one loop at a time, played at the rate that matches its ground
   * speed (`pickGait`), with the body turned along a sideways path and the loop reversed
   * when it backs away (`gaitFacing`). Each switch of loop crossfades by its signature.
   */
  private enemyGait(inst: Instance, tpl: Template, a: DrawActor, dt: number, out: Pose): string {
    // The AI rethinks every few ticks and can stand still for one, which dips the measured
    // speed. The average follows a speed-up at once but a slowdown only gradually, and a
    // switch to a slower loop waits out a short hold, so such a dip doesn't flip the loop.
    if (inst.speed > inst.gaitSpeed) inst.gaitSpeed = inst.speed;
    else inst.gaitSpeed += (inst.speed - inst.gaitSpeed) * (1 - Math.exp(-dt / GAIT_SPEED_SMOOTH_SECONDS));
    const pick = pickGait(inst.gaitSpeed, tpl.bands, inst.gait);
    const order: Record<Gait, number> = { idle: 0, walk: 1, run: 2 };
    if (pick.gait === inst.gait) inst.gaitWantT = 0;
    else {
      inst.gaitWantT = pick.gait === inst.gaitWant ? inst.gaitWantT + dt : 0;
      inst.gaitWant = pick.gait;
      if (order[pick.gait] > order[inst.gait] || inst.gaitWantT >= GAIT_HOLD_SECONDS) {
        // A new loop starts where its first step lands, not part way through the old one's.
        inst.gaitT = 0;
        inst.gait = pick.gait;
        inst.gaitWantT = 0;
      }
    }
    const gait = inst.gait;
    const rate = gait === 'idle' ? 1 : pickGait(Math.max(inst.gaitSpeed, IDLE_GAIT_FLOOR), tpl.bands, gait).rate;
    if (gait === 'idle') {
      inst.idleT += dt;
      sampleClip(tpl.clips.get(tpl.loops.idle)!, inst.idleT, true, out.q, out.hips);
      return 'loco:idle';
    }
    const heading = Math.atan2(inst.vx, inst.vz);
    const face = gaitFacing(a.yaw, heading, Math.hypot(inst.vx, inst.vz));
    inst.faceYaw = face.yaw;
    inst.faceRate = GAIT_TURN_RATE;
    inst.gaitT += dt * rate * (face.reverse ? -1 : 1);
    sampleClip(tpl.clips.get(gait === 'walk' ? tpl.loops.walk : tpl.loops.run)!, inst.gaitT, true, out.q, out.hips);
    return `loco:${gait}`;
  }

  /**
   * Fills `inst.tmp` with an art enemy's pose and returns its signature. In order: the death
   * (played once and held), waking (lying still until it first stirs, then the `rise` clip),
   * a move retimed to its frame data, a stagger while locked out, a hit reaction, and
   * locomotion. A missing clip skips its state.
   */
  private composeEnemyPose(inst: Instance, a: DrawActor, dt: number, moveFrac: number): string {
    const tpl = inst.tpl;
    const out = inst.tmp;
    const clip = (id: string) => tpl.clips.get(id);
    // Every state but locomotion faces where the simulation faces.
    inst.faceYaw = a.yaw;
    inst.faceRate = ATTACK_TURN_RATE;
    const once = (id: string, t: number) => {
      const c = clip(id)!;
      sampleClip(c, Math.min(t, c.duration), false, out.q, out.hips);
    };

    if ((a.flags & FLAG_DEAD) !== 0) {
      inst.deathT = inst.deathT < 0 ? 0 : inst.deathT + dt;
      if (clip('death')) {
        once('death', inst.deathT);
        return 'death';
      }
    } else if (inst.deathT >= 0) {
      // Back from the dead (the Lantern woke the cave): lie down again until stirred.
      inst.deathT = -1;
      inst.hitT = null;
      inst.rise = clip('rise') ? 'dormant' : 'up';
    }

    const info = a.moveId !== 0 ? this.resolveMove(a.moveId) : undefined;
    const staggered = (a.flags & FLAG_STAGGERED) !== 0;

    if (inst.rise === 'dormant' && (info || staggered || inst.hitT !== null || inst.speed > RISE_SPEED)) {
      inst.rise = 'rising';
      inst.riseT = 0;
      inst.hitT = null;
    }
    if (inst.rise === 'dormant') {
      once('rise', 0);
      return 'rise';
    }
    if (inst.rise === 'rising') {
      // A move cuts the waking short; the crossfade covers it.
      // The simulation sends a waking enemy after its target at once, so while it moves the
      // whole clip plays in RISE_MOVING_SECONDS: a lurch up rather than a slow slide.
      const riseClip = clip('rise')!;
      const moving = inst.speed > RISE_SPEED;
      inst.riseT += dt * (moving ? Math.max(RISE_RATE, riseClip.duration / RISE_MOVING_SECONDS) : RISE_RATE);
      if (!info && inst.riseT < riseClip.duration) {
        once('rise', inst.riseT);
        return 'rise';
      }
      inst.rise = 'up';
    }

    if (info) {
      inst.hitT = null;
      const frame = Math.max(0, a.moveFrame + moveFrac - inst.lagFrames);
      const choice = artSpecFor(info.anim, info.strikeStart !== undefined, tpl.art!.sidecar.clips);
      inst.drawnMove = info;
      inst.drawnFrame = frame;
      if (choice) {
        const frames = (c: string) => tpl.clips.get(c)!.frames;
        const len = specLength(choice, frames);
        const at = locateSpec(choice, moveClipFrame(len, choice.contact, info, frame), frames, BAKE_FPS);
        sampleClip(tpl.clips.get(at.clip)!, at.time, false, out.q, out.hips);
      } else {
        this.warnMissing(info.anim, tpl);
        sampleClip(tpl.clips.get(tpl.loops.idle)!, inst.idleT, true, out.q, out.hips);
      }
      return `move:${info.anim}:${inst.moveRun}`;
    }

    if (staggered) {
      inst.staggerT = inst.staggerT < 0 ? 0 : inst.staggerT + dt;
      const id = clip('stagger') ? 'stagger' : clip('hit') ? 'hit' : null;
      if (id) {
        inst.hitT = null;
        once(id, inst.staggerT);
        return 'stagger';
      }
    } else inst.staggerT = -1;

    if (inst.hitT !== null) {
      const hit = clip('hit');
      inst.hitT += dt * HIT_RATE;
      const moving = inst.hitT > HIT_MIN_SECONDS && inst.speed > HIT_MOVING_SPEED;
      if (hit && inst.hitT < hit.duration && !moving) {
        once('hit', inst.hitT);
        return `hit:${inst.hitRun}`;
      }
      inst.hitT = null;
    }

    return this.enemyGait(inst, tpl, a, dt, out);
  }

  /**
   * Blends the upper body in `out` toward the guard and two-hand stances (`stancePose.ts`),
   * eased toward the snapshot's `GUARDING` and `TWO_HANDING` flags. The legs keep the walk or
   * run. Moves and airborne poses skip this, so the weights also ease out while they play.
   */
  private layStance(inst: Instance, tpl: Template, a: DrawActor, dt: number, out: Pose): void {
    inst.guardW = easeStance(inst.guardW, (a.flags & FLAG_GUARDING) !== 0, dt);
    inst.twoHandW = easeStance(inst.twoHandW, (a.flags & FLAG_TWO_HANDING) !== 0, dt);
    if (inst.guardW <= 0 && inst.twoHandW <= 0) return;
    const w = stanceWeights(inst.guardW, inst.twoHandW);
    if (w.twoHand > 0) {
      // An art character has a two-handed idle; the fitted offsets are for the Wayfarer's bones.
      const target = tpl.art?.twoHand ? this.clipPose(tpl, tpl.art.twoHand, inst.idleT) : this.offsetPose(tpl, 'twohand', TWO_HAND);
      this.blendBones(out, target, tpl, ARM_BONES, w.twoHand);
    }
    if (w.guard > 0) this.blendBones(out, this.guardPose(inst, tpl, a), tpl, GUARD_BONES, w.guard);
    if (inst.hips.parent) {
      out.hips[1] -= (GUARD_HIPS_DROP * w.guard) / inst.hips.parent.getWorldScale(gestureScale).y;
    }
  }

  /** Slerps the named bones of `out` toward `target` by `w`. */
  private blendBones(out: Pose, target: Pose, tpl: Template, bones: readonly string[], w: number): void {
    for (const name of bones) {
      const i = tpl.boneIndex.get(name);
      if (i === undefined) continue;
      qa.fromArray(out.q, i * 4);
      qb.fromArray(target.q, i * 4);
      qa.slerp(qb, w).toArray(out.q, i * 4);
    }
  }

  /**
   * The guard's target: the shield guard with a shield in the off hand (and not two-handing),
   * else the weapon guard, else the fitted offsets when the library lacks both clips.
   */
  private guardPose(inst: Instance, tpl: Template, a: DrawActor): Pose {
    if (tpl.art?.guard) return this.clipPose(tpl, tpl.art.guard, inst.idleT);
    const left = a.id === this.playerId && this.equipped ? weaponByNum(this.equipped.left) : undefined;
    const shield = left?.class === 'Shield' && (a.flags & FLAG_TWO_HANDING) === 0;
    const pick = shield && tpl.clips.has(SHIELD_GUARD_CLIP.clip) ? SHIELD_GUARD_CLIP : WEAPON_GUARD_CLIP;
    if (tpl.clips.has(pick.clip)) return this.clipPose(tpl, pick, inst.idleT);
    return this.offsetPose(tpl, 'guard', GUARD_FALLBACK);
  }

  private stanceCache = new Map<string, Pose>();
  private stanceLoop: Pose | null = null;

  /** A stance clip's pose: a cached held frame, or the looping clip at time `t`. */
  private clipPose(tpl: Template, s: StanceClip, t: number): Pose {
    const clip = tpl.clips.get(s.clip)!;
    if (s.frame === undefined) {
      const p = (this.stanceLoop ??= newPose(tpl.boneNames.length));
      sampleClip(clip, t % clip.duration, true, p.q, p.hips);
      return p;
    }
    const key = `${s.clip}@${s.frame}`;
    let p = this.stanceCache.get(key);
    if (!p) {
      p = newPose(tpl.boneNames.length);
      sampleClip(clip, s.frame / BAKE_FPS, false, p.q, p.hips);
      this.stanceCache.set(key, p);
    }
    return p;
  }

  /** The idle clip's first frame with `offsets` laid over it, cached. */
  private offsetPose(tpl: Template, key: string, offsets: readonly BoneOffset[]): Pose {
    let p = this.stanceCache.get(key);
    if (p) return p;
    p = newPose(tpl.boneNames.length);
    sampleClip(tpl.clips.get(tpl.loops.idle)!, 0, true, p.q, p.hips);
    const e = new Euler();
    for (const o of offsets) {
      const i = tpl.boneIndex.get(o.bone);
      if (i === undefined) continue;
      qa.fromArray(p.q, i * 4).multiply(qb.setFromEuler(e.set(o.x, o.y, o.z)));
      qa.toArray(p.q, i * 4);
    }
    this.stanceCache.set(key, p);
    return p;
  }

  /** The idle pose with gesture `id`'s offsets laid over it (`ui/gestures/gesturePose.ts`). */
  private gestureFrame(inst: Instance, tpl: Template, out: Pose, id: number, t: number): void {
    const idle = tpl.clips.get(tpl.loops.idle)!;
    sampleClip(idle, t % idle.duration, true, out.q, out.hips);
    const g = gesturePose(id, t);
    const e = new Euler();
    for (const o of g.bones) {
      const i = tpl.boneIndex.get(o.bone);
      if (i === undefined) continue;
      qa.fromArray(out.q, i * 4).multiply(qb.setFromEuler(e.set(o.x, o.y, o.z)));
      qa.toArray(out.q, i * 4);
    }
    // The hips position is local to a scaled parent, so a drop in meters is divided by the
    // parent's world scale to land in the pose's units.
    if (g.hipsDrop > 0 && inst.hips.parent) out.hips[1] -= g.hipsDrop / inst.hips.parent.getWorldScale(gestureScale).y;
  }

  private locomotion(inst: Instance, tpl: Template, dt: number, frozen: boolean, out: Pose): void {
    const w = locomotionBlend(inst.speed, tpl.bands);
    if (!frozen) {
      inst.phase = (inst.phase + w.cyclesPerSec * dt) % 1;
      inst.idleT += dt;
    }
    const loops = tpl.loops;
    const layers: Array<[string, number, number]> = [
      [loops.idle, w.idle, inst.idleT],
      [loops.walk, w.walk, 0],
      [loops.run, w.run, 0],
      [loops.sprint, w.sprint, 0]
    ];
    const scratch = (this.sample ??= newPose(inst.bones.length));
    let total = 0;
    for (const [name, weight, idleT] of layers) {
      if (weight <= 0) continue;
      const clip = tpl.clips.get(name)!;
      const t = name === loops.idle ? idleT : inst.phase * clip.duration;
      if (total === 0) {
        sampleClip(clip, t, true, out.q, out.hips);
      } else {
        sampleClip(clip, t, true, scratch.q, scratch.hips);
        blendPose(out, scratch, weight / (total + weight));
      }
      total += weight;
    }
  }

  /** Crossfades from the last applied pose when the pose source changes. */
  private crossfade(inst: Instance, sig: string, dt: number): void {
    if (sig !== inst.signature) {
      if (inst.signature !== '') {
        copyPose(inst.from, inst.cur);
        inst.fadeT = 0;
        inst.fadeLen =
          inst.signature.startsWith('move:') && !sig.startsWith('move:')
            ? MOVE_EXIT_FADE_SECONDS
            : inst.signature.startsWith('loco:') && sig.startsWith('loco:')
              ? GAIT_FADE_SECONDS
              : FADE_SECONDS;
      }
      inst.signature = sig;
    }
    copyPose(inst.cur, inst.tmp);
    if (inst.fadeT < inst.fadeLen) {
      inst.fadeT += dt;
      const w = smooth(inst.fadeT / inst.fadeLen);
      // cur = slerp(from, target, w)
      const tgt = inst.tmp;
      copyPose(inst.cur, inst.from);
      blendPose(inst.cur, tgt, w);
    }
  }

  private apply(inst: Instance): void {
    const { q, hips } = inst.cur;
    for (let i = 0; i < inst.bones.length; i++) inst.bones[i].quaternion.fromArray(q, i * 4);
    inst.hips.position.set(hips[0], hips[1], hips[2]);
  }

  /**
   * The generic pose for a move with no clip: idle with the torso and sword arm swung through
   * wind-up, strike and follow-through keys, eased between them so the swing moves instead of
   * snapping. The strike key sits on the move's hitbox when content gives it. Crude on
   * purpose; it keeps gameplay unblocked by art.
   */
  private genericPose(out: Pose, tpl: Template, info: MoveAnimInfo, frame: number): void {
    const idle = tpl.clips.get(tpl.loops.idle)!;
    sampleClip(idle, 0, true, out.q, out.hips);
    const k = genericKeys(info);
    let a = GENERIC_KEYS[0];
    let b = GENERIC_KEYS[0];
    let w = 0;
    for (let i = 0; i < k.length - 1; i++) {
      if (frame <= k[i + 1] || i === k.length - 2) {
        a = GENERIC_KEYS[i];
        b = GENERIC_KEYS[i + 1];
        w = smooth((frame - k[i]) / Math.max(k[i + 1] - k[i], 1e-3));
        break;
      }
    }
    const e = new Euler();
    for (let n = 0; n < GENERIC_BONES.length; n++) {
      const i = tpl.boneIndex.get(GENERIC_BONES[n]);
      if (i === undefined) continue;
      const lerp = (c: number) => a[n][c] + (b[n][c] - a[n][c]) * w;
      qa.fromArray(out.q, i * 4).multiply(qb.setFromEuler(e.set(lerp(0), lerp(1), lerp(2))));
      qa.toArray(out.q, i * 4);
    }
  }

  /**
   * The first clip choice for `anim` whose clips are all loaded. The dev preview also takes a
   * raw clip name (`&anim=move:Sword_Regular_A`).
   */
  private pickSpec(tpl: Template, anim: string, preview: boolean, info?: MoveAnimInfo, twoHanded = false): ClipSpec | undefined {
    if (tpl.art) return artSpecFor(anim, info?.strikeStart !== undefined, tpl.art.sidecar.clips, twoHanded);
    const found = clipsForAnim(anim)?.find((c) => c.segs.every((g) => tpl.clips.has(g.clip)));
    if (found || !preview || !tpl.clips.has(anim)) return found;
    return { segs: [{ clip: anim }], contact: 0 };
  }

  private warnMissing(anim: string, tpl: Template): void {
    if (!import.meta.env.DEV || this.warned.has(anim)) return;
    this.warned.add(anim);
    if (tpl.art) {
      console.warn(`[latticefall] art character ${tpl.art.sidecar.name} has no clip for move anim "${anim}" (see ART_ANIMS in artClips.ts); holding the idle`);
      return;
    }
    const mapped = clipsForAnim(anim);
    const clips = mapped?.flatMap((c) => c.segs.map((g) => g.clip)).join(', ');
    console.warn(`[latticefall] no clip for move anim "${anim}"${clips ? ` (mapped to ${clips}, not all loaded)` : ''}; using the generic pose`);
  }
}
