// Draws actors as the Wayfarer model and animates them from snapshots. Render-only: it reads
// snapshot fields and never writes simulation state.
//
// Pose priority each frame: a move (by `move_id`, scrubbed by `move_frame`), then airborne
// states from the snapshot's `AIRBORNE` flag, then locomotion blended by ground speed. A short
// crossfade hides every switch between those.

import {
  Box3,
  type Bone,
  Group,
  type Material,
  Matrix4,
  type Mesh,
  type Object3D,
  type PerspectiveCamera,
  PropertyBinding,
  Quaternion,
  type Scene,
  type SkinnedMesh,
  Euler,
  Vector3,
  type WebGLRenderer
} from 'three';
import { GLTFLoader, type GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { KTX2Loader } from 'three/examples/jsm/loaders/KTX2Loader.js';
import { MeshoptDecoder } from 'three/examples/jsm/libs/meshopt_decoder.module.js';
import { clone as cloneSkinned } from 'three/examples/jsm/utils/SkeletonUtils.js';
import type { DrawActor } from '../../engine/snapshot';
import { PlayerGear, type Equipped, type RigFrame } from '../equipment/playerGear';
import { SpellFx } from '../equipment/spellFx';
import type { ModelDef } from '../procedural';
import { getQuality } from '../lighting/quality';
import { actorCullDistance } from '../perf/lod';
import { capSceneTextures } from '../perf/textures';
import { AirTracker, LAND_SECONDS } from './airState';
import { easeStance, GUARD_HIPS_DROP, stancePose } from './stancePose';
import { parsePreview, type PreviewRequest } from './devPreview';
import { digestAnimInfo } from './moveTable';
import { locomotionBlend, type LocoBands } from './locomotion';
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
/** Where KTX2 loading looks for the Basis transcoder. No current actor asset has textures. */
const BASIS_PATH = '/latticefall/basis/';

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

interface Template {
  gltf: GLTF;
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

  constructor(scene: Scene, renderer: WebGLRenderer, opts: { resolveMove?: MoveResolver } = {}) {
    this.scene = scene;
    this.renderer = renderer;
    this.resolveMove = opts.resolveMove ?? (() => undefined);
    if (typeof location !== 'undefined') {
      this.preview = parsePreview(location.search);
      this.propPreview = parsePropPreview(location.search);
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
    this.loading ??= this.doLoad();
    return this.loading;
  }

  private async doLoad(): Promise<void> {
    const loader = new GLTFLoader();
    loader.setMeshoptDecoder(MeshoptDecoder);
    const ktx2 = new KTX2Loader().setTranscoderPath(BASIS_PATH).detectSupport(this.renderer);
    loader.setKTX2Loader(ktx2);
    const [wayfarer, ual, ual2] = await Promise.all([
      loader.loadAsync(`${ACTOR_ASSET_BASE}wayfarer.glb`),
      loader.loadAsync(`${ACTOR_ASSET_BASE}ual.glb`),
      loader.loadAsync(`${ACTOR_ASSET_BASE}ual2.glb`).catch(() => null)
    ]);
    ktx2.dispose();
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

  /** Draws one actor. Returns false (and draws nothing) until the assets are loaded. */
  update(a: DrawActor, frameMs: number): boolean {
    const def = a.kind ? modelForKind(a.kind) : undefined;
    if (def) {
      this.updateEnemy(a, def, frameMs);
      return true;
    }
    const tpl = this.tpl;
    if (!tpl) return false;
    const dt = Math.max(frameMs, 0) / 1000;
    const inst = this.instances.get(a.id) ?? this.spawn(a.id, tpl);

    inst.holder.position.set(a.x, a.y, a.z);
    inst.holder.rotation.y = a.yaw + MODEL_YAW_OFFSET;

    // Ground speed from position deltas (the snapshot has none).
    if (inst.lastPos && dt > 0) {
      const d = Math.hypot(a.x - inst.lastPos[0], a.z - inst.lastPos[2]);
      const raw = d > TELEPORT_METERS ? 0 : d / dt;
      const k = 1 - Math.exp(-dt / SPEED_SMOOTH_SECONDS);
      inst.speed += (raw - inst.speed) * k;
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
      return true;
    }
    inst.lagFrames = Math.max(0, inst.lagFrames - (dt / TICK_SECONDS) * HITSTOP_CATCHUP);
    if (dt > 0) inst.air.update(a.y, dt, (a.flags & FLAG_AIRBORNE) !== 0);

    const pv = this.preview;
    if (pv) this.applyPreview(inst, pv, dt);

    // Move frame, made fractional between ticks.
    const now = performance.now();
    if (a.moveFrame !== inst.lastMoveFrame) {
      inst.lastMoveFrame = a.moveFrame;
      inst.moveFrameAt = now;
    }
    const moveFrac = Math.min((now - inst.moveFrameAt) / 1000 / TICK_SECONDS, 1);

    const sig = this.composePose(inst, a, tpl, dt, moveFrac);
    this.crossfade(inst, sig, dt);
    this.apply(inst);
    if (this.equipped && a.id === this.playerId) {
      this.applyGear(a.id, inst, tpl);
      this.gear.get(a.id)?.setOffHandStowed((a.flags & FLAG_TWO_HANDING) !== 0);
    }
    if (pv && (pv.state === 'jump' || pv.state === 'fall' || pv.state === 'land')) {
      inst.holder.position.y += pv.state === 'land' ? 0 : PREVIEW_AIR_LIFT_M;
    }
    return true;
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

  private applyGear(id: number, inst: Instance, tpl: Template): void {
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
        .multiply(new Matrix4().makeScale(1 / tpl.scale, 1 / tpl.scale, 1 / tpl.scale)),
      restBone: (n) => tpl.gltf.scene.getObjectByName(PropertyBinding.sanitizeNodeName(named(n)))!,
      bone: (n) => inst.bones[tpl.boneIndex.get(n)!],
      materials
    };
    tpl.gltf.scene.updateMatrixWorld(true);
    g.apply(this.equipped!, frame);
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
    const tpl = this.tpl;
    if (!inst || !tpl) return false;
    inst.holder.updateMatrixWorld(true);
    const hand = inst.bones[tpl.boneIndex.get('Hand.R')!];
    const fore = inst.bones[tpl.boneIndex.get('LowerArm.R')!];
    hand.getWorldPosition(base);
    fore.getWorldPosition(tip);
    tip.sub(base).negate().normalize().multiplyScalar(WEAPON_LENGTH_M).add(base);
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
        this.scene.remove(inst.holder);
        disposeSkeletons(inst.holder);
        this.instances.delete(id);
      }
    }
  }

  dispose(): void {
    for (const g of this.gear.values()) g.dispose();
    this.gear.clear();
    this.scene.remove(this.spellFx.points);
    this.spellFx.dispose();
    for (const inst of this.instances.values()) {
      this.scene.remove(inst.holder);
      disposeSkeletons(inst.holder);
    }
    this.instances.clear();
    for (const e of [...this.enemies.values(), ...this.propActors]) {
      this.scene.remove(e.holder);
      e.dispose();
    }
    this.enemies.clear();
    this.propActors = [];
    this.tpl?.gltf.scene.traverse((o) => {
      const m = o as { geometry?: { dispose(): void }; material?: { dispose(): void } | Array<{ dispose(): void }> };
      m.geometry?.dispose();
      if (Array.isArray(m.material)) m.material.forEach((x) => x.dispose());
      else m.material?.dispose();
    });
    this.tpl = null;
    if ((globalThis as Record<string, unknown>).__lfActors === this) delete (globalThis as Record<string, unknown>).__lfActors;
  }

  /** Dev helper: world positions of an actor's feet, to check for sliding. */
  debugFeet(id: number): { l: [number, number, number]; r: [number, number, number]; speed: number } | null {
    const inst = this.instances.get(id);
    if (!inst) return null;
    inst.holder.updateMatrixWorld(true);
    const get = (name: string): [number, number, number] => {
      const b = inst.bones[this.tpl!.boneIndex.get(name)!];
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

  private spawn(id: number, tpl: Template): Instance {
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
      twoHandW: 0
    };
    this.instances.set(id, inst);
    return inst;
  }

  private applyPreview(inst: Instance, pv: PreviewRequest, dt: number): void {
    const tpl = this.tpl!;
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
      const choice = this.pickSpec(tpl, info.anim, !!pv);
      if (choice) {
        const frames = (c: string) => tpl.clips.get(c)!.frames;
        const len = specLength(choice, frames);
        const at = locateSpec(choice, moveClipFrame(len, choice.contact, info, frame), frames, BAKE_FPS);
        sampleClip(tpl.clips.get(at.clip)!, at.time, false, out.q, out.hips);
        return sig;
      }
      this.warnMissing(info.anim);
      this.genericPose(out, tpl, info, frame);
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
   * Lays the guard and two-hand stances (`stancePose.ts`) over the pose in `out`, eased toward
   * the snapshot's `GUARDING` and `TWO_HANDING` flags. Moves and airborne poses skip this, so
   * the weights also ease out while they play.
   */
  private layStance(inst: Instance, tpl: Template, a: DrawActor, dt: number, out: Pose): void {
    inst.guardW = easeStance(inst.guardW, (a.flags & FLAG_GUARDING) !== 0, dt);
    inst.twoHandW = easeStance(inst.twoHandW, (a.flags & FLAG_TWO_HANDING) !== 0, dt);
    if (inst.guardW <= 0 && inst.twoHandW <= 0) return;
    const e = new Euler();
    for (const o of stancePose(inst.guardW, inst.twoHandW)) {
      const i = tpl.boneIndex.get(o.bone);
      if (i === undefined) continue;
      qa.fromArray(out.q, i * 4).multiply(qb.setFromEuler(e.set(o.x, o.y, o.z)));
      qa.toArray(out.q, i * 4);
    }
    if (inst.guardW > 0 && inst.hips.parent) {
      out.hips[1] -= (GUARD_HIPS_DROP * smooth(inst.guardW)) / inst.hips.parent.getWorldScale(gestureScale).y;
    }
  }

  /** The idle pose with gesture `id`'s offsets laid over it (`ui/gestures/gesturePose.ts`). */
  private gestureFrame(inst: Instance, tpl: Template, out: Pose, id: number, t: number): void {
    const idle = tpl.clips.get(LOOP_CLIPS.idle)!;
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
    const layers: Array<[string, number, number]> = [
      [LOOP_CLIPS.idle, w.idle, inst.idleT],
      [LOOP_CLIPS.walk, w.walk, 0],
      [LOOP_CLIPS.run, w.run, 0],
      [LOOP_CLIPS.sprint, w.sprint, 0]
    ];
    const scratch = (this.sample ??= newPose(inst.bones.length));
    let total = 0;
    for (const [name, weight, idleT] of layers) {
      if (weight <= 0) continue;
      const clip = tpl.clips.get(name)!;
      const t = name === LOOP_CLIPS.idle ? idleT : inst.phase * clip.duration;
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
        inst.fadeLen = inst.signature.startsWith('move:') && !sig.startsWith('move:') ? MOVE_EXIT_FADE_SECONDS : FADE_SECONDS;
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
    const idle = tpl.clips.get(LOOP_CLIPS.idle)!;
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
  private pickSpec(tpl: Template, anim: string, preview: boolean): ClipSpec | undefined {
    const found = clipsForAnim(anim)?.find((c) => c.segs.every((g) => tpl.clips.has(g.clip)));
    if (found || !preview || !tpl.clips.has(anim)) return found;
    return { segs: [{ clip: anim }], contact: 0 };
  }

  private warnMissing(anim: string): void {
    if (!import.meta.env.DEV || this.warned.has(anim)) return;
    this.warned.add(anim);
    const mapped = clipsForAnim(anim);
    const clips = mapped?.flatMap((c) => c.segs.map((g) => g.clip)).join(', ');
    console.warn(`[latticefall] no clip for move anim "${anim}"${clips ? ` (mapped to ${clips}, not all loaded)` : ''}; using the generic pose`);
  }
}
