// Draws actors as the Wayfarer model and animates them from snapshots. Render-only: it reads
// snapshot fields and never writes simulation state.
//
// Pose priority each frame: a move (by `move_id`, scrubbed by `move_frame`), then airborne
// states derived from position deltas, then locomotion blended by ground speed. A short
// crossfade hides every switch between those.

import {
  Box3,
  type Bone,
  Group,
  type Object3D,
  PropertyBinding,
  Quaternion,
  type Scene,
  Euler,
  Vector3,
  type WebGLRenderer
} from 'three';
import { GLTFLoader, type GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { KTX2Loader } from 'three/examples/jsm/loaders/KTX2Loader.js';
import { MeshoptDecoder } from 'three/examples/jsm/libs/meshopt_decoder.module.js';
import { clone as cloneSkinned } from 'three/examples/jsm/utils/SkeletonUtils.js';
import type { DrawActor } from '../../engine/snapshot';
import { AirTracker, LAND_SECONDS } from './airState';
import { parsePreview, type PreviewRequest } from './devPreview';
import { locomotionBlend, type LocoBands } from './locomotion';
import { clipForAnim, movePhase, moveClipTime, type MoveResolver } from './moveClips';
import { BONE_MAP, type BakedClip, retargetClips, sampleClip } from './retarget';

/** Where the optimized assets live (`lf-tools assets` writes them). */
export const ACTOR_ASSET_BASE = '/latticefall/actors/';
/** Where KTX2 loading looks for the Basis transcoder. No current actor asset has textures. */
const BASIS_PATH = '/latticefall/basis/';

/** Standing height of the Wayfarer in meters. The sim capsule is 1.7 m; the model adds hair. */
export const WAYFARER_HEIGHT_M = 1.75;
/** Added to the sim yaw. Both the sim (yaw 0 faces +Z) and the model face +Z. */
const MODEL_YAW_OFFSET = 0;

/** Crossfade between pose sources, in seconds. */
const FADE_SECONDS = 0.12;
/** Jump_Start is played this much faster than authored: the jump has already begun in the sim. */
const JUMP_START_RATE = 2.5;
/** Simulation tick length. Matches `TICK_MS` in `engine/loop.ts` (30 Hz). */
const TICK_SECONDS = 1 / 30;
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
  signature: string;
  lastPos: [number, number, number] | null;
  speed: number;
  phase: number;
  idleT: number;
  lastMoveFrame: number;
  moveFrameAt: number;
  previewT: number;
}

const newPose = (n: number): Pose => ({ q: new Float32Array(n * 4), hips: new Float32Array(3) });

function copyPose(dst: Pose, src: Pose): void {
  dst.q.set(src.q);
  dst.hips.set(src.hips);
}

const qa = new Quaternion();
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

  constructor(scene: Scene, renderer: WebGLRenderer, opts: { resolveMove?: MoveResolver } = {}) {
    this.scene = scene;
    this.renderer = renderer;
    this.resolveMove = opts.resolveMove ?? (() => undefined);
    if (typeof location !== 'undefined') this.preview = parsePreview(location.search);
    if (this.preview || import.meta.env.DEV) (globalThis as Record<string, unknown>).__lfActors = this;
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
    const [wayfarer, ual] = await Promise.all([
      loader.loadAsync(`${ACTOR_ASSET_BASE}wayfarer.glb`),
      loader.loadAsync(`${ACTOR_ASSET_BASE}ual.glb`)
    ]);
    ktx2.dispose();

    // Measure the rest pose with no transform on the scene.
    wayfarer.scene.updateMatrixWorld(true);
    const box = new Box3().setFromObject(wayfarer.scene, true);
    const height = box.max.y - box.min.y;
    const scale = WAYFARER_HEIGHT_M / height;

    const rt = retargetClips(ual.scene, ual.animations, wayfarer.scene, scale);
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
    const tpl = this.tpl;
    if (!tpl) return false;
    const dt = Math.max(frameMs, 0) / 1000;
    const inst = this.instances.get(a.id) ?? this.spawn(a.id, tpl);

    inst.holder.position.set(a.x, a.y, a.z);
    inst.holder.rotation.y = a.yaw + MODEL_YAW_OFFSET;

    // Ground speed and airborne state from position deltas (the snapshot has neither).
    if (inst.lastPos && dt > 0) {
      const d = Math.hypot(a.x - inst.lastPos[0], a.z - inst.lastPos[2]);
      const raw = d > TELEPORT_METERS ? 0 : d / dt;
      const k = 1 - Math.exp(-dt / SPEED_SMOOTH_SECONDS);
      inst.speed += (raw - inst.speed) * k;
    }
    inst.lastPos = [a.x, a.y, a.z];
    if (dt > 0) inst.air.update(a.y, dt);

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
    if (pv && (pv.state === 'jump' || pv.state === 'fall' || pv.state === 'land')) {
      inst.holder.position.y += pv.state === 'land' ? 0 : PREVIEW_AIR_LIFT_M;
    }
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
    for (const [id, inst] of this.instances) {
      if (!live.has(id)) {
        this.scene.remove(inst.holder);
        this.instances.delete(id);
      }
    }
  }

  dispose(): void {
    for (const inst of this.instances.values()) this.scene.remove(inst.holder);
    this.instances.clear();
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
      signature: '',
      lastPos: null,
      speed: 0,
      phase: 0,
      idleT: 0,
      lastMoveFrame: -1,
      moveFrameAt: 0,
      previewT: 0
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

    // 1. A move, from move_id and move_frame.
    let info = a.moveId !== 0 ? this.resolveMove(a.moveId) : undefined;
    let frame = a.moveFrame + moveFrac;
    if (pv && typeof pv.state === 'object') {
      const total = 30;
      info = { anim: pv.state.move, totalFrames: total };
      frame = (pv.t ?? ((performance.now() / 1000) % 1)) * total;
    }
    if (info) {
      const clipName = clipForAnim(info.anim);
      const clip = clipName ? tpl.clips.get(clipName) : undefined;
      if (clip) {
        sampleClip(clip, moveClipTime(clip.duration, frame, info.totalFrames), false, out.q, out.hips);
        return `move:${info.anim}`;
      }
      this.warnMissing(info.anim, clipName);
      this.genericPose(out, tpl, info, frame);
      return `move:${info.anim}`;
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
    // 3. Locomotion, with the landing blended into it.
    this.locomotion(inst, tpl, dt, frozen, out);
    if (air.phase === 'landing') {
      const land = tpl.clips.get('Jump_Land');
      if (land) {
        const sample = (this.sample ??= newPose(inst.bones.length));
        sampleClip(land, (air.t / LAND_SECONDS) * land.duration * 0.5, false, sample.q, sample.hips);
        // Full landing pose at touchdown, easing into locomotion over LAND_SECONDS.
        blendPose(sample, out, smooth(air.t / LAND_SECONDS));
        copyPose(out, sample);
        return 'air:land';
      }
    }
    return 'loco';
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
      }
      inst.signature = sig;
    }
    copyPose(inst.cur, inst.tmp);
    if (inst.fadeT < FADE_SECONDS) {
      inst.fadeT += dt;
      const w = smooth(inst.fadeT / FADE_SECONDS);
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
   * The generic pose for a move with no clip: idle with the torso and sword arm swung by
   * phase. Crude on purpose; it keeps gameplay unblocked by art.
   */
  private genericPose(out: Pose, tpl: Template, info: { anim: string; totalFrames: number }, frame: number): void {
    const idle = tpl.clips.get(LOOP_CLIPS.idle)!;
    sampleClip(idle, 0, true, out.q, out.hips);
    const phase = movePhase(info, frame);
    const e = new Euler();
    const apply = (bone: string, x: number, y: number, z: number) => {
      const i = tpl.boneIndex.get(bone);
      if (i === undefined) return;
      qa.fromArray(out.q, i * 4).multiply(qb.setFromEuler(e.set(x, y, z)));
      qa.toArray(out.q, i * 4);
    };
    if (phase === 'windup') {
      apply('Torso', -0.2, -0.45, 0);
      apply('UpperArm.R', 0, 0, -1.1);
      apply('LowerArm.R', -0.9, 0, 0);
    } else if (phase === 'strike') {
      apply('Torso', 0.25, 0.55, 0);
      apply('UpperArm.R', 0, 0, 0.5);
      apply('LowerArm.R', -0.2, 0, 0);
    } else {
      apply('Torso', 0.1, 0.15, 0);
      apply('UpperArm.R', 0, 0, 0.2);
    }
  }

  private warnMissing(anim: string, clip: string | undefined): void {
    if (!import.meta.env.DEV || this.warned.has(anim)) return;
    this.warned.add(anim);
    console.warn(`[latticefall] no clip for move anim "${anim}"${clip ? ` (mapped to ${clip}, not in the library)` : ''}; using the generic pose`);
  }
}
