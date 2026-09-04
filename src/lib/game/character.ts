// The explorer and the animals that share the planet with them.
//
// Every model is built from primitives at load time. Animals merge their bodies
// into a single vertex-coloured mesh so a planet full of them still costs only
// a handful of draw calls; the parts that have to move — an explorer's limbs, a
// quadruped's legs — stay separate.
//
// The explorer's walk is driven by distance travelled rather than by time; see
// `Character.update`. That is what keeps the feet planted instead of skating.

import * as THREE from 'three';
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';

const SKIN = 0xf2c9a4;
const JACKET = 0xf3b53a;
const JACKET_D = 0xe5a52f;
const TROUSERS = 0x3b4a7a;
const CAP = 0xd9402a;
const PACK = 0x3d8f8a;
const BOOT = 0x5a3b2a;

/** Hip height, which is also the length of a straightened leg. */
const LEG = 0.62;
/** Thigh and shin each take half of it. */
const SEG = LEG / 2;
/** Ground speed, in surface units per second, that counts as a full sprint. */
const SPRINT = 8.2;
/** How deeply the knee flexes over a planted foot. */
const STANCE_KNEE = 0.24;

function box(w: number, h: number, d: number, color: number): THREE.Mesh {
  const m = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), new THREE.MeshStandardMaterial({ color, roughness: 0.85 }));
  m.castShadow = true;
  return m;
}

/** Ease that starts and ends at rest, for the swing leg's reach forward. */
function smoothstep(t: number): number {
  return t * t * (3 - 2 * t);
}

/**
 * Blocky explorer: red cap, yellow jacket, teal backpack. Local +Y is up and
 * +Z is forward. Roughly 1.8 units tall.
 *
 * The limbs hang off jointed pivots — hip, knee, shoulder, elbow — so the walk
 * can plant a foot and bend the trailing knee to clear the ground.
 */
export class Character {
  group = new THREE.Group();
  /** Ground speed that `speed01 = 1` stands for. The Game sets it from `RUN_SPEED`. */
  topSpeed = SPRINT;
  /** True on the frame a foot lands, for the footstep sound. */
  footStrike = false;
  /** Root of everything above the feet; its height follows the stance leg. */
  private body = new THREE.Group();
  private hipL: THREE.Group;
  private hipR: THREE.Group;
  private kneeL: THREE.Group;
  private kneeR: THREE.Group;
  private shoulderL: THREE.Group;
  private shoulderR: THREE.Group;
  private elbowL: THREE.Group;
  private elbowR: THREE.Group;
  private torso: THREE.Group;
  /** Gait cycle position in [0, 1). One cycle is two steps. */
  private phase = 0;
  /** Which half of the cycle the last frame was in, for foot-strike detection. */
  private lastHalf = 0;

  constructor() {
    this.group.add(this.body);

    const legL = this.makeLeg(-0.15);
    const legR = this.makeLeg(0.15);
    this.hipL = legL.hip;
    this.hipR = legR.hip;
    this.kneeL = legL.knee;
    this.kneeR = legR.knee;
    this.body.add(this.hipL, this.hipR);

    this.torso = new THREE.Group();
    this.torso.position.y = LEG;
    const body = box(0.62, 0.58, 0.4, JACKET);
    body.position.y = 0.29;
    const belt = box(0.64, 0.09, 0.42, 0x8a5a2c);
    belt.position.y = 0.04;
    const collar = box(0.64, 0.09, 0.42, JACKET_D);
    collar.position.y = 0.56;
    const scarf = box(0.5, 0.12, 0.44, 0xc9524a);
    scarf.position.set(0, 0.63, 0.02);
    const scarfTail = box(0.14, 0.34, 0.08, 0xc9524a);
    scarfTail.position.set(0.12, 0.5, -0.24);
    scarfTail.rotation.x = -0.3;
    const pack = box(0.44, 0.44, 0.22, PACK);
    pack.position.set(0, 0.32, -0.29);
    const packLid = box(0.46, 0.12, 0.24, 0x2f6f6b);
    packLid.position.set(0, 0.52, -0.29);
    const bedroll = new THREE.Mesh(
      new THREE.CylinderGeometry(0.09, 0.09, 0.46, 6),
      new THREE.MeshStandardMaterial({ color: 0xd9c9a8, roughness: 0.9 })
    );
    bedroll.castShadow = true;
    bedroll.rotation.z = Math.PI / 2;
    bedroll.position.set(0, 0.12, -0.36);
    const packStrap = box(0.1, 0.42, 0.06, 0x2f6f6b);
    packStrap.position.set(0, 0.32, 0.21);
    this.torso.add(body, belt, collar, scarf, scarfTail, pack, packLid, bedroll, packStrap);

    const armL = this.makeArm(-0.39);
    const armR = this.makeArm(0.39);
    this.shoulderL = armL.shoulder;
    this.shoulderR = armR.shoulder;
    this.elbowL = armL.elbow;
    this.elbowR = armR.elbow;
    this.torso.add(this.shoulderL, this.shoulderR);

    const head = box(0.46, 0.42, 0.44, SKIN);
    head.position.y = 0.82;
    const eyeL = box(0.06, 0.08, 0.02, 0x2a2a2a);
    eyeL.position.set(-0.11, 0.84, 0.225);
    const eyeR = eyeL.clone();
    eyeR.position.x = 0.11;
    const smile = box(0.14, 0.03, 0.02, 0x8a4a3a);
    smile.position.set(0, 0.72, 0.225);
    const capTop = new THREE.Mesh(
      new THREE.CylinderGeometry(0.27, 0.29, 0.18, 8),
      new THREE.MeshStandardMaterial({ color: CAP, roughness: 0.8 })
    );
    capTop.castShadow = true;
    capTop.position.y = 1.07;
    const button = new THREE.Mesh(
      new THREE.SphereGeometry(0.05, 6, 5),
      new THREE.MeshStandardMaterial({ color: 0xf5e9c8, roughness: 0.8 })
    );
    button.position.y = 1.18;
    const brim = box(0.32, 0.05, 0.28, CAP);
    brim.position.set(0, 1.0, 0.31);
    this.torso.add(head, eyeL, eyeR, smile, capTop, button, brim);

    this.body.add(this.torso);
  }

  /** Hip pivot with a knee inside it, so the shin can fold under the thigh. */
  private makeLeg(x: number): { hip: THREE.Group; knee: THREE.Group } {
    const hip = new THREE.Group();
    hip.position.set(x, LEG, 0);
    const thigh = box(0.23, SEG, 0.23, TROUSERS);
    thigh.position.y = -SEG / 2;
    const knee = new THREE.Group();
    knee.position.y = -SEG;
    const shin = box(0.21, SEG, 0.21, TROUSERS);
    shin.position.y = -SEG / 2;
    const boot = box(0.25, 0.13, 0.3, BOOT);
    boot.position.set(0, -SEG + 0.06, 0.04);
    knee.add(shin, boot);
    hip.add(thigh, knee);
    return { hip, knee };
  }

  private makeArm(x: number): { shoulder: THREE.Group; elbow: THREE.Group } {
    const shoulder = new THREE.Group();
    shoulder.position.set(x, 0.5, 0);
    const upper = box(0.16, 0.24, 0.16, JACKET);
    upper.position.y = -0.12;
    const elbow = new THREE.Group();
    elbow.position.y = -0.24;
    const fore = box(0.15, 0.22, 0.15, JACKET_D);
    fore.position.y = -0.11;
    const hand = box(0.15, 0.13, 0.15, SKIN);
    hand.position.y = -0.28;
    elbow.add(fore, hand);
    shoulder.add(upper, elbow);
    return { shoulder, elbow };
  }

  /**
   * Animate the explorer. `speed01` is 0 when idle and 1 at `topSpeed`;
   * `airborne` tucks the legs for a hop; `seated` folds them onto a bench with
   * the hands out, as if holding oars; `petting` crouches with a hand out.
   *
   * The walk cycle advances with **distance covered**, not with time, and the
   * stance leg's hip angle is solved from where the planted foot has to be. A
   * step therefore always covers exactly the ground the explorer travels, at
   * any speed, which is what stops the feet from skating. Faster running
   * shortens the share of the cycle a foot spends on the ground and lengthens
   * the reach, so the stride opens up into a run with a moment of flight.
   */
  update(dt: number, speed01: number, airborne: boolean, time: number, seated = false, petting = false): void {
    this.footStrike = false;
    if (petting && !seated) {
      this.pose({ hip: [-0.5, 0.24], knee: [0.95, 0.5], shoulder: [-1.15, -1.55], elbow: [-0.5, -0.75], lift: -0.28, pitch: 0.34 });
      const pat = Math.sin(time * 7) * 0.18;
      this.shoulderL.rotation.x += pat;
      this.shoulderR.rotation.x += pat * 1.4;
      return;
    }
    if (seated) {
      const row = Math.sin(time * 2.4) * 0.12;
      this.pose({ hip: [-1.42, -1.42], knee: [0.9, 0.9], shoulder: [-0.95 + row, -0.95 + row], elbow: [-0.35, -0.35], lift: 0, pitch: 0.08 });
      this.body.position.y += Math.sin(time * 2.2) * 0.008;
      return;
    }
    if (airborne) {
      this.pose({ hip: [-0.55, 0.35], knee: [1.1, 0.45], shoulder: [-2.0, -2.2], elbow: [-0.6, -0.4], lift: 0, pitch: 0.06 });
      return;
    }

    const speed = Math.max(0, speed01) * this.topSpeed;
    const gait = Math.min(1, speed / SPRINT);
    if (speed < 0.05) {
      // Standing: settle out of whatever pose the last step left behind.
      const breathe = Math.sin(time * 2.2) * 0.01;
      this.relax(dt, breathe);
      return;
    }

    // Reach of the swing, and the share of the cycle a foot stays planted.
    // Both open up with speed: a sprint reaches further and lifts off sooner.
    const reach = 0.58 + 0.42 * gait;
    const stance = 0.58 - 0.30 * gait;
    // Ground the foot covers while planted, and therefore the whole cycle.
    const sweep = 2 * LEG * Math.sin(reach);
    const cycle = sweep / stance;

    this.phase = (this.phase + (speed * dt) / cycle) % 1;
    const half = this.phase < 0.5 ? 0 : 1;
    if (half !== this.lastHalf) this.footStrike = true;
    this.lastHalf = half;

    const right = this.leg(this.phase, reach, stance, sweep);
    const left = this.leg((this.phase + 0.5) % 1, reach, stance, sweep);

    this.hipR.rotation.x = right.hip;
    this.kneeR.rotation.x = right.knee;
    this.hipL.rotation.x = left.hip;
    this.kneeL.rotation.x = left.knee;

    // Hip height follows whichever foot is on the ground, so the body rises
    // over mid-stance and dips at the hand-over. With both feet off the ground
    // there is nothing holding it up, so it arcs between the two hand-overs
    // instead of snapping to full height.
    let support = Math.max(right.support, left.support);
    if (support <= 0) {
      const low = LEG * Math.cos(reach);
      const u = ((this.phase % 0.5) - stance) / (0.5 - stance);
      support = low + (LEG - low) * Math.sin(Math.PI * u) * (0.35 + 0.5 * gait);
    }
    this.body.position.y = support - LEG;

    // Arms counter-swing against the legs and bend more the harder we run.
    const swing = Math.sin(this.phase * Math.PI * 2) * (0.5 + 0.6 * gait);
    const bend = -0.25 - 0.9 * gait;
    this.shoulderR.rotation.x = -swing - 0.1 * gait;
    this.shoulderL.rotation.x = swing - 0.1 * gait;
    this.elbowR.rotation.x = bend - Math.max(0, -swing) * 0.5;
    this.elbowL.rotation.x = bend - Math.max(0, swing) * 0.5;
    // Lean into the run, and roll the shoulders a little with each stride.
    this.torso.rotation.x = 0.05 + 0.22 * gait;
    this.torso.rotation.y = swing * 0.12;
    this.torso.rotation.z = Math.sin(this.phase * Math.PI * 4) * 0.03 * gait;
  }

  /**
   * Hip and knee angles for one leg at cycle position `p`.
   *
   * While the foot is planted its position is fixed in the world, so in the
   * explorer's own frame it slides straight backwards through `sweep` units.
   * The stance half solves for the joint angles that put the sole exactly
   * there. The swing half is free to be shaped for looks: the knee folds to
   * clear the ground and the foot eases forward for the next contact.
   *
   * Both segments are the same length, so a bent knee is easy to solve for in
   * closed form. Writing the two segments as one leg bisecting the knee angle,
   * `sole = 2 * SEG * cos(k/2)` long and pointing `hip + k/2` from vertical,
   * the reach shortens by `cos(k/2)` and the whole leg swings half the bend
   * forward — so backing both out of the straight-leg answer keeps the sole
   * where the stride wants it, however deeply the knee flexes.
   */
  private leg(p: number, reach: number, stance: number, sweep: number): { hip: number; knee: number; support: number } {
    if (p < stance) {
      const t = p / stance;
      // Foot travels from `+sweep/2` (ahead) to `-sweep/2` (behind).
      const z = sweep * (0.5 - t);
      // The knee takes the landing and straightens into the push off.
      const knee = STANCE_KNEE * Math.sin(t * Math.PI);
      const bisect = knee / 2;
      const line = Math.asin(THREE.MathUtils.clamp(-z / LEG / Math.cos(bisect), -1, 1));
      return { hip: line - bisect, knee, support: LEG * Math.cos(line) * Math.cos(bisect) };
    }
    const u = (p - stance) / (1 - stance);
    const hip = -reach * (2 * smoothstep(u) - 1);
    // Fold the knee hardest early in the swing, when the foot has to clear.
    const knee = Math.sin(Math.pow(u, 0.7) * Math.PI) * (0.9 + reach * 0.6);
    return { hip, knee, support: 0 };
  }

  /** Ease every joint back toward standing. Used while idle. */
  private relax(dt: number, breathe: number): void {
    const k = Math.min(1, dt * 9);
    const ease = (o: THREE.Object3D, x: number) => (o.rotation.x += (x - o.rotation.x) * k);
    ease(this.hipL, 0);
    ease(this.hipR, 0);
    ease(this.kneeL, 0);
    ease(this.kneeR, 0);
    ease(this.shoulderL, 0.06);
    ease(this.shoulderR, 0.06);
    ease(this.elbowL, -0.18);
    ease(this.elbowR, -0.18);
    this.torso.rotation.x += (0 - this.torso.rotation.x) * k;
    this.torso.rotation.y += (0 - this.torso.rotation.y) * k;
    this.torso.rotation.z += (0 - this.torso.rotation.z) * k;
    this.body.position.y += (breathe - this.body.position.y) * k;
    this.phase = 0;
    this.lastHalf = 0;
  }

  /** Snap every joint to a fixed pose. */
  private pose(o: {
    hip: [number, number];
    knee: [number, number];
    shoulder: [number, number];
    elbow: [number, number];
    lift: number;
    pitch: number;
  }): void {
    this.hipL.rotation.x = o.hip[0];
    this.hipR.rotation.x = o.hip[1];
    this.kneeL.rotation.x = o.knee[0];
    this.kneeR.rotation.x = o.knee[1];
    this.shoulderL.rotation.x = o.shoulder[0];
    this.shoulderR.rotation.x = o.shoulder[1];
    this.elbowL.rotation.x = o.elbow[0];
    this.elbowR.rotation.x = o.elbow[1];
    this.torso.rotation.set(o.pitch, 0, 0);
    this.body.position.y = o.lift;
    this.phase = 0;
  }
}

// ---------------------------------------------------------------- animals

/**
 * How an animal carries itself. `Critter` in `critters.ts` reads this off the
 * model's `userData` and animates accordingly.
 */
export type Gait = 'walk' | 'hop' | 'waddle' | 'scuttle';

export type AnimalData = {
  gait: Gait;
  /** Surface units covered by one full leg cycle, or one hop. */
  cycle: number;
  /** Everything above the ground; rocked and bounced without moving the model. */
  pivot: THREE.Group;
  /** Leg pivots, for the `walk` gait. */
  legs: THREE.Object3D[];
};

/** Unit primitives, centred on the origin, shared by every animal. */
const U = {
  box: new THREE.BoxGeometry(1, 1, 1),
  sphere: new THREE.SphereGeometry(0.5, 8, 6),
  ico: new THREE.IcosahedronGeometry(0.5, 0),
  dodeca: new THREE.DodecahedronGeometry(0.5, 0),
  cone: new THREE.ConeGeometry(0.5, 1, 6),
  cyl: new THREE.CylinderGeometry(0.5, 0.5, 1, 8)
};

type Piece = {
  geo: THREE.BufferGeometry;
  color: number;
  p?: [number, number, number];
  s?: number | [number, number, number];
  r?: [number, number, number];
};

const _mp = new THREE.Vector3();
const _mq = new THREE.Quaternion();
const _ms = new THREE.Vector3();
const _me = new THREE.Euler();
const _mm = new THREE.Matrix4();
const _mc = new THREE.Color();

/**
 * Bake a list of primitives into one flat-shaded, vertex-coloured mesh. An
 * animal body is a dozen boxes; merging them keeps the planet's animals at one
 * draw call each instead of a dozen.
 */
function merge(pieces: Piece[]): THREE.Mesh {
  const parts: THREE.BufferGeometry[] = [];
  for (const piece of pieces) {
    const g = piece.geo.index ? piece.geo.toNonIndexed() : piece.geo.clone();
    const p = piece.p ?? [0, 0, 0];
    const r = piece.r ?? [0, 0, 0];
    const s = piece.s ?? 1;
    _mp.set(p[0], p[1], p[2]);
    _me.set(r[0], r[1], r[2]);
    _mq.setFromEuler(_me);
    if (typeof s === 'number') _ms.set(s, s, s);
    else _ms.set(s[0], s[1], s[2]);
    g.applyMatrix4(_mm.compose(_mp, _mq, _ms));
    for (const name of Object.keys(g.attributes)) if (name !== 'position') g.deleteAttribute(name);
    const n = g.attributes.position.count;
    const arr = new Float32Array(n * 3);
    _mc.setHex(piece.color);
    for (let i = 0; i < n; i++) {
      arr[i * 3] = _mc.r;
      arr[i * 3 + 1] = _mc.g;
      arr[i * 3 + 2] = _mc.b;
    }
    g.setAttribute('color', new THREE.BufferAttribute(arr, 3));
    parts.push(g);
  }
  const merged = mergeGeometries(parts, false)!;
  merged.computeVertexNormals();
  for (const p of parts) p.dispose();
  const mesh = new THREE.Mesh(merged, new THREE.MeshStandardMaterial({ vertexColors: true, flatShading: true, roughness: 0.9 }));
  mesh.castShadow = true;
  return mesh;
}

type Leg = { x: number; z: number; w: number; h: number; color: number };

/**
 * Assemble an animal: a merged body, optional legs, and its gait.
 *
 * A walking quadruped moves its legs in diagonal pairs, and the two hips of a
 * pair share one axis — so each pair is merged into a single mesh on a single
 * pivot. That is three draw calls for an animal instead of nine, which is what
 * lets the planet carry a few dozen of them.
 */
function animal(gait: Gait, cycle: number, body: Piece[], legs: Leg[] = []): THREE.Group {
  const g = new THREE.Group();
  const pivot = new THREE.Group();
  g.add(pivot);
  pivot.add(merge(body));
  const pivots: THREE.Object3D[] = [];
  const pairs = legs.length === 4 ? [[0, 3], [1, 2]] : legs.map((_, i) => [i]);
  for (const pair of pairs) {
    const hip = new THREE.Group();
    hip.position.y = legs[pair[0]].h;
    hip.add(
      merge(
        pair.map((i) => {
          const l = legs[i];
          return { geo: U.box, color: l.color, p: [l.x, -l.h / 2, l.z], s: [l.w, l.h, l.w] } as Piece;
        })
      )
    );
    pivot.add(hip);
    pivots.push(hip);
  }
  const data: AnimalData = { gait, cycle, pivot, legs: pivots };
  g.userData = data;
  return g;
}

/** Four legs at the corners of a body. */
function fourLegs(x: number, z: number, w: number, h: number, color: number) {
  return [
    { x: -x, z, w, h, color },
    { x, z, w, h, color },
    { x: -x, z: -z, w, h, color },
    { x, z: -z, w, h, color }
  ];
}

/** Low-poly sheep for the farm. */
export function buildSheep(): THREE.Group {
  return animal(
    'walk',
    1.0,
    [
      { geo: U.dodeca, color: 0xf5f1e8, p: [0, 0.58, 0], s: [0.98, 0.78, 1.16] },
      { geo: U.ico, color: 0xfffdf6, p: [0, 0.8, -0.16], s: [0.6, 0.5, 0.6] },
      { geo: U.box, color: 0x2b2b30, p: [0, 0.63, 0.5], s: [0.26, 0.24, 0.3] },
      { geo: U.box, color: 0x2b2b30, p: [-0.15, 0.72, 0.46], s: [0.07, 0.13, 0.06] },
      { geo: U.box, color: 0x2b2b30, p: [0.15, 0.72, 0.46], s: [0.07, 0.13, 0.06] },
      { geo: U.box, color: 0x1c1c20, p: [0, 0.58, 0.65], s: [0.1, 0.07, 0.05] },
      { geo: U.box, color: 0xf5f1e8, p: [0, 0.5, -0.6], s: [0.16, 0.16, 0.14] }
    ],
    fourLegs(0.19, 0.24, 0.11, 0.32, 0x2b2b30)
  );
}

/** Low-poly polar bear for the arctic. */
export function buildBear(): THREE.Group {
  const coat = 0xf6f7f4;
  return animal(
    'walk',
    1.5,
    [
      { geo: U.dodeca, color: coat, p: [0, 0.7, 0], s: [1.14, 0.94, 1.7] },
      { geo: U.dodeca, color: coat, p: [0, 0.8, 0.86], s: [0.62, 0.58, 0.6] },
      { geo: U.box, color: 0xe9e6dc, p: [0, 0.72, 1.1], s: [0.21, 0.17, 0.2] },
      { geo: U.box, color: 0x222226, p: [0, 0.75, 1.2], s: [0.11, 0.08, 0.05] },
      { geo: U.box, color: 0x222226, p: [-0.12, 0.85, 1.14], s: [0.06, 0.05, 0.03] },
      { geo: U.box, color: 0x222226, p: [0.12, 0.85, 1.14], s: [0.06, 0.05, 0.03] },
      { geo: U.box, color: 0xe9e6dc, p: [-0.21, 1.02, 0.8], s: [0.12, 0.11, 0.09] },
      { geo: U.box, color: 0xe9e6dc, p: [0.21, 1.02, 0.8], s: [0.12, 0.11, 0.09] }
    ],
    fourLegs(0.33, 0.46, 0.25, 0.4, 0xefeee8)
  );
}

/** Fox for the forest, the grove, and the shore. `coat` recolours it. */
export function buildFox(coat = 0xe0782f): THREE.Group {
  return animal(
    'walk',
    0.85,
    [
      { geo: U.box, color: coat, p: [0, 0.42, 0], s: [0.3, 0.28, 0.62] },
      { geo: U.box, color: coat, p: [0, 0.57, 0.4], s: [0.26, 0.22, 0.26] },
      { geo: U.box, color: 0xf5efe3, p: [0, 0.52, 0.57], s: [0.12, 0.1, 0.14] },
      { geo: U.box, color: 0x2a2124, p: [0, 0.53, 0.65], s: [0.07, 0.05, 0.04] },
      { geo: U.box, color: 0x2a2124, p: [-0.08, 0.62, 0.52], s: [0.05, 0.04, 0.03] },
      { geo: U.box, color: 0x2a2124, p: [0.08, 0.62, 0.52], s: [0.05, 0.04, 0.03] },
      { geo: U.cone, color: coat, p: [-0.09, 0.73, 0.36], s: [0.13, 0.18, 0.06] },
      { geo: U.cone, color: coat, p: [0.09, 0.73, 0.36], s: [0.13, 0.18, 0.06] },
      { geo: U.box, color: 0xf5efe3, p: [0, 0.3, 0.2], s: [0.22, 0.12, 0.3] },
      { geo: U.box, color: coat, p: [0, 0.46, -0.44], s: [0.15, 0.15, 0.36], r: [-0.4, 0, 0] },
      { geo: U.box, color: 0xf5efe3, p: [0, 0.56, -0.6], s: [0.14, 0.14, 0.16], r: [-0.4, 0, 0] }
    ],
    fourLegs(0.1, 0.2, 0.09, 0.28, 0x3a2a22)
  );
}

/** Red deer for the grove, with a small rack of antlers. */
export function buildDeer(): THREE.Group {
  const coat = 0xa9713f;
  const body: Piece[] = [
    { geo: U.box, color: coat, p: [0, 0.78, 0], s: [0.38, 0.42, 0.86] },
    { geo: U.box, color: 0xc99a6a, p: [0, 0.62, 0.1], s: [0.34, 0.16, 0.6] },
    { geo: U.box, color: coat, p: [0, 1.0, 0.42], s: [0.24, 0.42, 0.24], r: [-0.35, 0, 0] },
    { geo: U.box, color: coat, p: [0, 1.24, 0.6], s: [0.24, 0.24, 0.34] },
    { geo: U.box, color: 0x4a3226, p: [0, 1.2, 0.78], s: [0.14, 0.12, 0.12] },
    { geo: U.box, color: 0x24201e, p: [-0.08, 1.3, 0.72], s: [0.05, 0.05, 0.03] },
    { geo: U.box, color: 0x24201e, p: [0.08, 1.3, 0.72], s: [0.05, 0.05, 0.03] },
    { geo: U.cone, color: coat, p: [-0.14, 1.36, 0.56], s: [0.14, 0.16, 0.07], r: [0, 0, -0.3] },
    { geo: U.cone, color: coat, p: [0.14, 1.36, 0.56], s: [0.14, 0.16, 0.07], r: [0, 0, 0.3] },
    { geo: U.box, color: 0xf2e6d2, p: [0, 0.86, -0.44], s: [0.16, 0.18, 0.12], r: [0.5, 0, 0] }
  ];
  // Antlers: a beam each side with two tines.
  for (const s of [-1, 1]) {
    body.push({ geo: U.box, color: 0x8a6a48, p: [0.1 * s, 1.5, 0.58], s: [0.05, 0.34, 0.05], r: [0, 0, -0.35 * s] });
    body.push({ geo: U.box, color: 0x8a6a48, p: [0.22 * s, 1.66, 0.58], s: [0.04, 0.2, 0.04], r: [0, 0, -0.8 * s] });
    body.push({ geo: U.box, color: 0x8a6a48, p: [0.18 * s, 1.62, 0.72], s: [0.04, 0.18, 0.04], r: [-0.5, 0, -0.4 * s] });
  }
  return animal('walk', 1.4, body, fourLegs(0.15, 0.32, 0.1, 0.57, 0x6f4a30));
}

/** Farmyard goat. Small, stubborn, and horned. */
export function buildGoat(): THREE.Group {
  const coat = 0xd8cfbe;
  return animal(
    'walk',
    0.95,
    [
      { geo: U.box, color: coat, p: [0, 0.56, 0], s: [0.32, 0.34, 0.66] },
      { geo: U.box, color: coat, p: [0, 0.72, 0.42], s: [0.24, 0.24, 0.3] },
      { geo: U.box, color: 0x8e8375, p: [0, 0.66, 0.58], s: [0.16, 0.14, 0.12] },
      { geo: U.box, color: 0x24201e, p: [-0.08, 0.78, 0.54], s: [0.05, 0.05, 0.03] },
      { geo: U.box, color: 0x24201e, p: [0.08, 0.78, 0.54], s: [0.05, 0.05, 0.03] },
      { geo: U.box, color: 0x6f6355, p: [-0.09, 0.9, 0.38], s: [0.05, 0.22, 0.05], r: [-0.5, 0, -0.2] },
      { geo: U.box, color: 0x6f6355, p: [0.09, 0.9, 0.38], s: [0.05, 0.22, 0.05], r: [-0.5, 0, 0.2] },
      { geo: U.box, color: 0xf2ece0, p: [0, 0.56, 0.6], s: [0.09, 0.2, 0.08], r: [0.3, 0, 0] },
      { geo: U.box, color: coat, p: [0, 0.7, -0.36], s: [0.1, 0.16, 0.1], r: [-0.7, 0, 0] }
    ],
    fourLegs(0.14, 0.24, 0.09, 0.36, 0x4a4038)
  );
}

/** Rabbit. Hops rather than walks. */
export function buildRabbit(coat = 0xbfae9a): THREE.Group {
  return animal('hop', 0.9, [
    { geo: U.ico, color: coat, p: [0, 0.26, -0.04], s: [0.34, 0.32, 0.44] },
    { geo: U.ico, color: coat, p: [0, 0.4, 0.22], s: [0.24, 0.24, 0.24] },
    { geo: U.box, color: 0x2a2226, p: [-0.07, 0.42, 0.32], s: [0.05, 0.05, 0.03] },
    { geo: U.box, color: 0x2a2226, p: [0.07, 0.42, 0.32], s: [0.05, 0.05, 0.03] },
    { geo: U.box, color: 0xdcc9b4, p: [-0.07, 0.6, 0.14], s: [0.07, 0.28, 0.05], r: [-0.15, 0, -0.15] },
    { geo: U.box, color: 0xdcc9b4, p: [0.07, 0.6, 0.14], s: [0.07, 0.28, 0.05], r: [-0.15, 0, 0.15] },
    { geo: U.sphere, color: 0xfdf6ec, p: [0, 0.28, -0.28], s: 0.14 },
    { geo: U.box, color: coat, p: [-0.14, 0.11, -0.06], s: [0.09, 0.2, 0.24] },
    { geo: U.box, color: coat, p: [0.14, 0.11, -0.06], s: [0.09, 0.2, 0.24] },
    { geo: U.box, color: coat, p: [-0.1, 0.07, 0.18], s: [0.07, 0.13, 0.14] },
    { geo: U.box, color: coat, p: [0.1, 0.07, 0.18], s: [0.07, 0.13, 0.14] }
  ]);
}

/** Duck for the marsh. Waddles. */
export function buildDuck(): THREE.Group {
  return animal('waddle', 0.5, [
    { geo: U.ico, color: 0xf2efe6, p: [0, 0.26, 0], s: [0.3, 0.28, 0.44] },
    { geo: U.box, color: 0xd8d2c2, p: [0, 0.3, -0.24], s: [0.18, 0.14, 0.2], r: [0.5, 0, 0] },
    { geo: U.cyl, color: 0x3f6f4a, p: [0, 0.44, 0.14], s: [0.14, 0.24, 0.14], r: [-0.2, 0, 0] },
    { geo: U.ico, color: 0x3f6f4a, p: [0, 0.56, 0.2], s: [0.2, 0.2, 0.22] },
    { geo: U.box, color: 0xe8a33a, p: [0, 0.54, 0.34], s: [0.12, 0.06, 0.14] },
    { geo: U.box, color: 0x201d1c, p: [-0.07, 0.6, 0.26], s: [0.04, 0.04, 0.03] },
    { geo: U.box, color: 0x201d1c, p: [0.07, 0.6, 0.26], s: [0.04, 0.04, 0.03] },
    { geo: U.box, color: 0xe8a33a, p: [-0.08, 0.05, 0.04], s: [0.08, 0.08, 0.16] },
    { geo: U.box, color: 0xe8a33a, p: [0.08, 0.05, 0.04], s: [0.08, 0.08, 0.16] }
  ]);
}

/** Penguin for the arctic. Waddles. */
export function buildPenguin(): THREE.Group {
  return animal('waddle', 0.42, [
    { geo: U.ico, color: 0x2b3440, p: [0, 0.36, 0], s: [0.34, 0.6, 0.32] },
    { geo: U.ico, color: 0xf4f1e8, p: [0, 0.34, 0.09], s: [0.24, 0.46, 0.22] },
    { geo: U.ico, color: 0x2b3440, p: [0, 0.68, 0], s: [0.26, 0.24, 0.26] },
    { geo: U.box, color: 0xf4f1e8, p: [0, 0.64, 0.1], s: [0.16, 0.12, 0.08] },
    { geo: U.cone, color: 0xe8a33a, p: [0, 0.66, 0.17], s: [0.09, 0.14, 0.08], r: [1.4, 0, 0] },
    { geo: U.box, color: 0x14181d, p: [-0.07, 0.72, 0.11], s: [0.04, 0.04, 0.03] },
    { geo: U.box, color: 0x14181d, p: [0.07, 0.72, 0.11], s: [0.04, 0.04, 0.03] },
    { geo: U.box, color: 0x2b3440, p: [-0.19, 0.36, 0], s: [0.06, 0.34, 0.14], r: [0, 0, 0.18] },
    { geo: U.box, color: 0x2b3440, p: [0.19, 0.36, 0], s: [0.06, 0.34, 0.14], r: [0, 0, -0.18] },
    { geo: U.box, color: 0xe8a33a, p: [-0.08, 0.03, 0.05], s: [0.09, 0.06, 0.16] },
    { geo: U.box, color: 0xe8a33a, p: [0.08, 0.03, 0.05], s: [0.09, 0.06, 0.16] }
  ]);
}

/** Crab for the shore. Scuttles sideways. */
export function buildCrab(): THREE.Group {
  const shell = 0xd9542f;
  const body: Piece[] = [
    { geo: U.ico, color: shell, p: [0, 0.16, 0], s: [0.44, 0.2, 0.34] },
    { geo: U.box, color: 0xf6e9d8, p: [-0.1, 0.24, 0.13], s: [0.06, 0.06, 0.04] },
    { geo: U.box, color: 0xf6e9d8, p: [0.1, 0.24, 0.13], s: [0.06, 0.06, 0.04] },
    { geo: U.box, color: shell, p: [-0.28, 0.18, 0.16], s: [0.16, 0.1, 0.12], r: [0, 0.5, 0] },
    { geo: U.box, color: shell, p: [0.28, 0.18, 0.16], s: [0.16, 0.1, 0.12], r: [0, -0.5, 0] }
  ];
  for (const s of [-1, 1]) {
    for (let i = 0; i < 3; i++) {
      body.push({ geo: U.box, color: 0xb8431f, p: [0.24 * s, 0.08, -0.06 - i * 0.1], s: [0.18, 0.05, 0.05], r: [0, 0, 0.3 * s] });
    }
  }
  return animal('scuttle', 0.36, body);
}

/** Turtle for the shore and the marsh. Plods. */
export function buildTurtle(): THREE.Group {
  return animal('waddle', 0.4, [
    { geo: U.sphere, color: 0x6a7d3c, p: [0, 0.18, 0], s: [0.48, 0.34, 0.6] },
    { geo: U.sphere, color: 0x8a9b52, p: [0, 0.2, 0], s: [0.36, 0.3, 0.46] },
    { geo: U.box, color: 0xc6b98c, p: [0, 0.1, 0], s: [0.46, 0.1, 0.56] },
    { geo: U.ico, color: 0x9aa86a, p: [0, 0.18, 0.34], s: [0.18, 0.16, 0.2] },
    { geo: U.box, color: 0x231f1c, p: [-0.06, 0.22, 0.42], s: [0.04, 0.04, 0.03] },
    { geo: U.box, color: 0x231f1c, p: [0.06, 0.22, 0.42], s: [0.04, 0.04, 0.03] },
    { geo: U.box, color: 0x9aa86a, p: [-0.22, 0.08, 0.2], s: [0.14, 0.08, 0.16], r: [0, 0.5, 0] },
    { geo: U.box, color: 0x9aa86a, p: [0.22, 0.08, 0.2], s: [0.14, 0.08, 0.16], r: [0, -0.5, 0] },
    { geo: U.box, color: 0x9aa86a, p: [-0.2, 0.08, -0.2], s: [0.13, 0.08, 0.15], r: [0, -0.4, 0] },
    { geo: U.box, color: 0x9aa86a, p: [0.2, 0.08, -0.2], s: [0.13, 0.08, 0.15], r: [0, 0.4, 0] }
  ]);
}

/** Frog for the marsh. Hops. */
export function buildFrog(): THREE.Group {
  return animal('hop', 0.7, [
    { geo: U.ico, color: 0x5f9e4a, p: [0, 0.18, 0], s: [0.34, 0.24, 0.4] },
    { geo: U.ico, color: 0x5f9e4a, p: [0, 0.26, 0.16], s: [0.28, 0.2, 0.24] },
    { geo: U.sphere, color: 0xf2e9c8, p: [-0.09, 0.36, 0.16], s: 0.11 },
    { geo: U.sphere, color: 0xf2e9c8, p: [0.09, 0.36, 0.16], s: 0.11 },
    { geo: U.box, color: 0x1d1a18, p: [-0.09, 0.38, 0.21], s: [0.05, 0.05, 0.03] },
    { geo: U.box, color: 0x1d1a18, p: [0.09, 0.38, 0.21], s: [0.05, 0.05, 0.03] },
    { geo: U.box, color: 0xd8e3a8, p: [0, 0.09, 0.06], s: [0.24, 0.08, 0.24] },
    { geo: U.box, color: 0x4d8a3c, p: [-0.16, 0.1, -0.1], s: [0.09, 0.14, 0.24], r: [0.4, 0, 0] },
    { geo: U.box, color: 0x4d8a3c, p: [0.16, 0.1, -0.1], s: [0.09, 0.14, 0.24], r: [0.4, 0, 0] },
    { geo: U.box, color: 0x4d8a3c, p: [-0.14, 0.05, 0.18], s: [0.07, 0.07, 0.14] },
    { geo: U.box, color: 0x4d8a3c, p: [0.14, 0.05, 0.18], s: [0.07, 0.07, 0.14] }
  ]);
}
