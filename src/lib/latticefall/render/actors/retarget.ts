// Retargets the Universal Animation Library (Unreal-style rig) onto the Quaternius modular
// humanoid by bone name, and bakes each clip to 30 Hz rotation arrays the renderer samples
// itself. Baking happens once at load. The method is world-space rotation deltas:
//
//   targetWorldAnim = sourceWorldAnim * inverse(sourceWorldRest) * targetWorldRest
//
// so the two rigs may have different local axes and rest poses, and only need to agree on
// which way the character faces and which way is up.

import { type AnimationClip, AnimationMixer, type Bone, Matrix3, type Object3D, PropertyBinding, Quaternion, Vector3 } from 'three';

/** Source bone to target bone, parents first. Fingers and toes are left at their rest pose. */
export const BONE_MAP: ReadonlyArray<readonly [string, string]> = [
  ['pelvis', 'Hips'],
  ['spine_01', 'Abdomen'],
  ['spine_02', 'Torso'],
  ['spine_03', 'Chest'],
  ['neck_01', 'Neck'],
  ['Head', 'Head'],
  ['clavicle_l', 'Shoulder.L'],
  ['upperarm_l', 'UpperArm.L'],
  ['lowerarm_l', 'LowerArm.L'],
  ['hand_l', 'Hand.L'],
  ['clavicle_r', 'Shoulder.R'],
  ['upperarm_r', 'UpperArm.R'],
  ['lowerarm_r', 'LowerArm.R'],
  ['hand_r', 'Hand.R'],
  ['thigh_l', 'UpperLeg.L'],
  ['calf_l', 'LowerLeg.L'],
  ['foot_l', 'Foot.L'],
  ['thigh_r', 'UpperLeg.R'],
  ['calf_r', 'LowerLeg.R'],
  ['foot_r', 'Foot.R']
];

/** Baked sample rate in Hz. The library's clips are authored at 30 fps. */
export const BAKE_FPS = 30;

export interface BakedClip {
  name: string;
  duration: number;
  frames: number;
  /** `frames * boneCount * 4` quaternions (x, y, z, w) in each target bone's local space. */
  q: Float32Array;
  /** `frames * 3` hip-bone local positions, in the target scene's units. */
  hips: Float32Array;
  /**
   * Ground speed in m/s the clip's planted foot implies on the source rig: how fast the
   * stance ankle travels backward through the in-place clip. 0 when no foot is planted.
   * Measured, not authored: the library's root-motion variant doesn't match its feet.
   */
  stanceSpeed: number;
}

export interface RetargetResult {
  /** Target bone names in the order `BakedClip.q` stores them. */
  boneNames: string[];
  clips: Map<string, BakedClip>;
  /** Ratio of the target's hip height to the source's, in meters. Scales stride length. */
  strideScale: number;
}

const sanitize = (n: string) => PropertyBinding.sanitizeNodeName(n);

function find(root: Object3D, name: string): Object3D {
  const node = root.getObjectByName(sanitize(name));
  if (!node) throw new Error(`retarget: bone "${name}" not found`);
  return node;
}

/**
 * @param srcScene The animation library's scene (skeleton nodes only is fine).
 * @param srcClips The library's clips.
 * @param tgtScene The character's scene, at its rest pose, with no transform on it.
 * @param tgtMetersPerUnit Meters per unit of `tgtScene` once the renderer scales the model.
 */
export function retargetClips(
  srcScene: Object3D,
  srcClips: AnimationClip[],
  tgtScene: Object3D,
  tgtMetersPerUnit: number
): RetargetResult {
  srcScene.updateMatrixWorld(true);
  tgtScene.updateMatrixWorld(true);

  const pairs = BONE_MAP.map(([s, t]) => ({ src: find(srcScene, s), tgt: find(tgtScene, t) as Bone }));
  const boneNames = pairs.map((p) => p.tgt.name);
  const n = pairs.length;

  const srcRest = pairs.map((p) => p.src.getWorldQuaternion(new Quaternion()));
  const tgtRest = pairs.map((p) => p.tgt.getWorldQuaternion(new Quaternion()));
  const tgtIndex = new Map<Object3D, number>(pairs.map((p, i) => [p.tgt, i]));

  const srcHipsRest = pairs[0].src.getWorldPosition(new Vector3());
  const tgtHipsRest = pairs[0].tgt.getWorldPosition(new Vector3());
  const strideScale = (tgtHipsRest.y * tgtMetersPerUnit) / srcHipsRest.y;
  const hipsParentInv3 = new Matrix3().setFromMatrix4(pairs[0].tgt.parent!.matrixWorld.clone().invert());
  const hipsRestLocal = pairs[0].tgt.position.clone();

  // Animated world rotation of any target node: mapped bones come from the source, others keep
  // their rest local rotation under an animated parent.
  const animWorld: Array<Quaternion | undefined> = new Array(n);
  const plainCache = new Map<Object3D, Quaternion>();
  const worldOf = (node: Object3D): Quaternion => {
    const i = tgtIndex.get(node);
    if (i !== undefined) return animWorld[i]!;
    let q = plainCache.get(node);
    if (!q) {
      q = node.parent ? worldOf(node.parent).clone().multiply(node.quaternion) : node.quaternion.clone();
      plainCache.set(node, q);
    }
    return q;
  };

  const srcFeet = [find(srcScene, 'foot_l'), find(srcScene, 'foot_r')];
  const footPos = new Vector3();
  const mixer = new AnimationMixer(srcScene);
  const clips = new Map<string, BakedClip>();
  const tmp = new Quaternion();
  const local = new Vector3();
  const srcPos = new Vector3();

  for (const clip of srcClips) {
    const frames = Math.max(2, Math.round(clip.duration * BAKE_FPS) + 1);
    const q = new Float32Array(frames * n * 4);
    const hips = new Float32Array(frames * 3);
    const feetY: [number[], number[]] = [[], []];
    const feetZ: [number[], number[]] = [[], []];
    const action = mixer.clipAction(clip);
    action.play();
    for (let f = 0; f < frames; f++) {
      mixer.setTime(Math.min(f / BAKE_FPS, clip.duration));
      srcScene.updateMatrixWorld(true);
      plainCache.clear();
      for (let i = 0; i < n; i++) {
        const world = pairs[i].src
          .getWorldQuaternion(new Quaternion())
          .multiply(tmp.copy(srcRest[i]).invert())
          .multiply(tgtRest[i]);
        animWorld[i] = world;
        const parentWorld = worldOf(pairs[i].tgt.parent!);
        const rel = tmp.copy(parentWorld).invert().multiply(world);
        q.set([rel.x, rel.y, rel.z, rel.w], (f * n + i) * 4);
      }
      pairs[0].src.getWorldPosition(srcPos);
      [srcFeet[0], srcFeet[1]].forEach((foot, k) => {
        foot.getWorldPosition(footPos);
        feetY[k].push(footPos.y);
        feetZ[k].push(footPos.z);
      });
      // World-space hip travel in meters, scaled by leg length, into the hip bone's parent frame.
      local.copy(srcPos).sub(srcHipsRest).multiplyScalar(strideScale / tgtMetersPerUnit);
      local.applyMatrix3(hipsParentInv3).add(hipsRestLocal);
      hips.set([local.x, local.y, local.z], f * 3);
    }
    action.stop();
    const stanceSpeed = measureStanceSpeed(feetY, feetZ, clip.duration / (frames - 1));
    clips.set(clip.name, { name: clip.name, duration: clip.duration, frames, q, hips, stanceSpeed });
  }
  mixer.stopAllAction();
  return { boneNames, clips, strideScale };
}

/** A foot counts as planted in the lowest quarter of its ankle's height range while it moves backward. */
const STANCE_FRACTION = 0.25;

/**
 * Median backward speed of the planted ankle, along +Z (the rig's forward), in m/s. Height
 * alone isn't enough: a sprinting kick-back dips as low as the stance, so a foot has to be
 * low and moving backward.
 */
function measureStanceSpeed(y: [number[], number[]], z: [number[], number[]], dt: number): number {
  const v: number[] = [];
  for (let k = 0; k < 2; k++) {
    const lo = Math.min(...y[k]);
    const low = lo + (Math.max(...y[k]) - lo) * STANCE_FRACTION;
    for (let i = 1; i < y[k].length; i++) {
      const back = -(z[k][i] - z[k][i - 1]) / dt;
      if (y[k][i] < low && y[k][i - 1] < low && back > 0) v.push(back);
    }
  }
  if (v.length < 3) return 0;
  v.sort((a, b) => a - b);
  return v[Math.floor(v.length / 2)];
}

const qa = new Quaternion();
const qb = new Quaternion();

/**
 * Samples a baked clip at `time` seconds into `outQ` (n*4) and `outHips` (3). Loops wrap.
 * Linear in time, spherical between the two frames.
 */
export function sampleClip(clip: BakedClip, time: number, loop: boolean, outQ: Float32Array, outHips: Float32Array): void {
  const n = outQ.length / 4;
  let t = time;
  if (loop) t = ((t % clip.duration) + clip.duration) % clip.duration;
  else t = Math.min(Math.max(t, 0), clip.duration);
  const f = (t / clip.duration) * (clip.frames - 1);
  const f0 = Math.min(Math.floor(f), clip.frames - 2);
  const u = f - f0;
  for (let i = 0; i < n; i++) {
    const o0 = (f0 * n + i) * 4;
    const o1 = ((f0 + 1) * n + i) * 4;
    qa.fromArray(clip.q, o0);
    qb.fromArray(clip.q, o1);
    qa.slerp(qb, u);
    qa.toArray(outQ, i * 4);
  }
  for (let k = 0; k < 3; k++) outHips[k] = clip.hips[f0 * 3 + k] * (1 - u) + clip.hips[(f0 + 1) * 3 + k] * u;
}
