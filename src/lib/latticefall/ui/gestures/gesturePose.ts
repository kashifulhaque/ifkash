// Gesture poses (WP-088): bone offsets, by gesture and time, that the actor renderer lays over
// the idle clip. The animation library has no gesture clips (only Idle, Walk, Jog, Sprint, Jump,
// Roll, Interact, Sword and hit clips), so the poses are built here from rotations. Pure math:
// no Three.js import, and nothing writes simulation state. The core only says which gesture is
// on (`GestureEvent`, WASM event kind 58); how it looks is presentation.

/** One bone's offset: Euler angles in radians, applied on top of the idle pose. */
export interface BoneOffset {
  bone: string;
  x: number;
  y: number;
  z: number;
}

export interface GesturePose {
  bones: BoneOffset[];
  /** Meters the hips drop below standing. */
  hipsDrop: number;
}

/** Seconds the pose takes to come in. */
const EASE_IN = 0.3;

const smooth = (x: number): number => {
  const t = Math.min(Math.max(x, 0), 1);
  return t * t * (3 - 2 * t);
};

/**
 * The pose of gesture `id` (0 wave, 1 bow, 2 point forward, 3 sit, 4 shrug, 5 beckon) `t`
 * seconds after it started. An unknown gesture leaves the idle pose alone.
 */
export function gesturePose(id: number, t: number): GesturePose {
  const e = smooth(t / EASE_IN);
  const bones: BoneOffset[] = [];
  const add = (bone: string, x: number, y: number, z: number) => bones.push({ bone, x: x * e, y: y * e, z: z * e });
  let hipsDrop = 0;
  switch (id) {
    case 0: // Wave: the right arm up, the forearm swinging.
      add('UpperArm.R', 0, 0, -2.5);
      add('LowerArm.R', 0, 0, -0.35 + Math.sin(t * 9) * 0.5);
      add('Head', 0, 0.12, 0);
      break;
    case 1: // Bow: from the waist, the head following.
      add('Abdomen', 0.45, 0, 0);
      add('Torso', 0.35, 0, 0);
      add('Head', 0.25, 0, 0);
      add('UpperArm.L', 0, 0, 0.15);
      add('UpperArm.R', 0, 0, -0.15);
      break;
    case 2: // Point forward: the right arm straight out.
      add('UpperArm.R', -1.45, 0, 0);
      add('LowerArm.R', 0, 0, 0);
      add('Torso', 0, -0.12, 0);
      break;
    case 3: // Sit: down on the heels, a little slumped.
      hipsDrop = 0.5 * e;
      add('UpperLeg.L', -1.45, 0, 0);
      add('UpperLeg.R', -1.45, 0, 0);
      add('LowerLeg.L', 1.55, 0, 0);
      add('LowerLeg.R', 1.55, 0, 0);
      add('Abdomen', 0.18, 0, 0);
      add('Head', -0.12, 0, 0);
      break;
    case 4: // Shrug: shoulders up, hands open, the head to one side.
      add('Shoulder.L', 0, 0, 0.45);
      add('Shoulder.R', 0, 0, -0.45);
      add('LowerArm.L', -0.8, 0, 0);
      add('LowerArm.R', -0.8, 0, 0);
      add('Head', 0, 0, 0.14);
      break;
    case 5: // Beckon: the right forearm raised, the hand curling toward the chest.
      add('UpperArm.R', -1.2, 0, 0);
      add('LowerArm.R', -1.1, 0, 0);
      add('Hand.R', Math.sin(t * 8) * 0.55 - 0.2, 0, 0);
      break;
    default:
      break;
  }
  return { bones, hipsDrop };
}
