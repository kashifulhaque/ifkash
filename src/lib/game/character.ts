import * as THREE from 'three';

const SKIN = 0xf2c9a4;
const JACKET = 0xf3b53a;
const TROUSERS = 0x3b4a7a;
const CAP = 0xd9402a;
const PACK = 0x3d8f8a;
const BOOT = 0x5a3b2a;

function box(w: number, h: number, d: number, color: number): THREE.Mesh {
  const m = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), new THREE.MeshStandardMaterial({ color, roughness: 0.85 }));
  m.castShadow = true;
  return m;
}

/**
 * Blocky explorer: red cap, yellow jacket, teal backpack. Local +Y is up and
 * +Z is forward. Roughly 1.6 units tall.
 */
export class Character {
  group = new THREE.Group();
  private legL: THREE.Group;
  private legR: THREE.Group;
  private armL: THREE.Group;
  private armR: THREE.Group;
  private torso: THREE.Group;
  private phase = 0;

  constructor() {
    const g = this.group;

    this.legL = this.makeLeg(-0.14);
    this.legR = this.makeLeg(0.14);
    g.add(this.legL, this.legR);

    this.torso = new THREE.Group();
    this.torso.position.y = 0.5;
    const body = box(0.6, 0.56, 0.38, JACKET);
    body.position.y = 0.28;
    const collar = box(0.62, 0.08, 0.4, 0xe5a52f);
    collar.position.y = 0.53;
    const pack = box(0.42, 0.42, 0.2, PACK);
    pack.position.set(0, 0.3, -0.27);
    const packStrap = box(0.1, 0.4, 0.06, 0x2f6f6b);
    packStrap.position.set(0, 0.3, 0.2);
    this.torso.add(body, collar, pack, packStrap);

    this.armL = this.makeArm(-0.37);
    this.armR = this.makeArm(0.37);
    this.torso.add(this.armL, this.armR);

    const head = box(0.44, 0.4, 0.42, SKIN);
    head.position.y = 0.78;
    const eyeL = box(0.05, 0.07, 0.02, 0x2a2a2a);
    eyeL.position.set(-0.1, 0.8, 0.215);
    const eyeR = eyeL.clone();
    eyeR.position.x = 0.1;
    const capTop = new THREE.Mesh(
      new THREE.CylinderGeometry(0.26, 0.28, 0.17, 8),
      new THREE.MeshStandardMaterial({ color: CAP, roughness: 0.8 })
    );
    capTop.castShadow = true;
    capTop.position.y = 1.02;
    const brim = box(0.3, 0.05, 0.26, CAP);
    brim.position.set(0, 0.96, 0.3);
    this.torso.add(head, eyeL, eyeR, capTop, brim);

    g.add(this.torso);
  }

  private makeLeg(x: number): THREE.Group {
    const pivot = new THREE.Group();
    pivot.position.set(x, 0.5, 0);
    const leg = box(0.22, 0.42, 0.22, TROUSERS);
    leg.position.y = -0.21;
    const boot = box(0.24, 0.12, 0.28, BOOT);
    boot.position.set(0, -0.45, 0.03);
    pivot.add(leg, boot);
    return pivot;
  }

  private makeArm(x: number): THREE.Group {
    const pivot = new THREE.Group();
    pivot.position.set(x, 0.5, 0);
    const arm = box(0.16, 0.44, 0.16, JACKET);
    arm.position.y = -0.2;
    const hand = box(0.14, 0.12, 0.14, SKIN);
    hand.position.y = -0.46;
    pivot.add(arm, hand);
    return pivot;
  }

  /**
   * Animate limbs. `speed01` is 0 when idle and 1 at full run;
   * `airborne` freezes the legs in a tuck; `seated` folds the explorer onto a
   * bench with the legs forward and the hands out, as if holding oars;
   * `petting` crouches with a hand out to an animal.
   */
  update(dt: number, speed01: number, airborne: boolean, time: number, seated = false, petting = false): void {
    if (petting && !seated) {
      const pat = Math.sin(time * 7) * 0.18;
      this.legL.rotation.x = -0.45;
      this.legR.rotation.x = 0.2;
      this.armL.rotation.x = -1.15 + pat;
      this.armR.rotation.x = -1.55 + pat * 1.4;
      this.torso.position.y = 0.32;
      this.torso.rotation.x = 0.34;
      return;
    }
    if (seated) {
      this.legL.rotation.x = -1.42;
      this.legR.rotation.x = -1.42;
      const row = Math.sin(time * 2.4) * 0.12;
      this.armL.rotation.x = -0.95 + row;
      this.armR.rotation.x = -0.95 + row;
      this.torso.position.y = 0.5 + Math.sin(time * 2.2) * 0.008;
      this.torso.rotation.x = 0.08;
      return;
    }
    if (speed01 > 0.02) this.phase += dt * (7 + speed01 * 7);
    const swing = Math.sin(this.phase) * 0.75 * speed01;
    if (airborne) {
      this.legL.rotation.x = 0.5;
      this.legR.rotation.x = -0.2;
      this.armL.rotation.x = -2.2;
      this.armR.rotation.x = -2.2;
      this.torso.position.y = 0.5;
    } else {
      this.legL.rotation.x = swing;
      this.legR.rotation.x = -swing;
      this.armL.rotation.x = -swing * 0.8;
      this.armR.rotation.x = swing * 0.8;
      const bob = Math.abs(Math.sin(this.phase)) * 0.06 * speed01;
      const breathe = speed01 < 0.02 ? Math.sin(time * 2.2) * 0.008 : 0;
      this.torso.position.y = 0.5 + bob + breathe;
    }
    this.torso.rotation.x = speed01 * 0.12;
  }
}

/** Low-poly sheep for the farm. */
export function buildSheep(): THREE.Group {
  const g = new THREE.Group();
  const wool = new THREE.Mesh(
    new THREE.DodecahedronGeometry(0.42, 0),
    new THREE.MeshStandardMaterial({ color: 0xf5f1e8, roughness: 1, flatShading: true })
  );
  wool.castShadow = true;
  wool.scale.set(1.15, 0.9, 1.35);
  wool.position.y = 0.55;
  const head = box(0.26, 0.24, 0.28, 0x2b2b30);
  head.position.set(0, 0.62, 0.5);
  const earL = box(0.06, 0.12, 0.05, 0x2b2b30);
  earL.position.set(-0.15, 0.7, 0.48);
  const earR = earL.clone();
  earR.position.x = 0.15;
  g.add(wool, head, earL, earR);
  for (const [x, z] of [
    [-0.18, 0.22],
    [0.18, 0.22],
    [-0.18, -0.22],
    [0.18, -0.22]
  ]) {
    const leg = box(0.1, 0.3, 0.1, 0x2b2b30);
    leg.position.set(x, 0.15, z);
    g.add(leg);
  }
  return g;
}

/** Low-poly polar bear for the arctic. */
export function buildBear(): THREE.Group {
  const g = new THREE.Group();
  const mat = new THREE.MeshStandardMaterial({ color: 0xf6f7f4, roughness: 0.95, flatShading: true });
  const body = new THREE.Mesh(new THREE.DodecahedronGeometry(0.55, 0), mat);
  body.castShadow = true;
  body.scale.set(1.1, 0.9, 1.6);
  body.position.y = 0.66;
  const head = new THREE.Mesh(new THREE.DodecahedronGeometry(0.3, 0), mat);
  head.castShadow = true;
  head.position.set(0, 0.78, 0.85);
  const snout = box(0.2, 0.16, 0.18, 0xe9e6dc);
  snout.position.set(0, 0.7, 1.08);
  const nose = box(0.1, 0.07, 0.04, 0x222226);
  nose.position.set(0, 0.72, 1.18);
  const earL = box(0.1, 0.1, 0.08, 0xe9e6dc);
  earL.position.set(-0.2, 1.0, 0.8);
  const earR = earL.clone();
  earR.position.x = 0.2;
  g.add(body, head, snout, nose, earL, earR);
  for (const [x, z] of [
    [-0.32, 0.45],
    [0.32, 0.45],
    [-0.32, -0.45],
    [0.32, -0.45]
  ]) {
    const leg = box(0.24, 0.36, 0.26, 0xefeee8);
    leg.position.set(x, 0.18, z);
    leg.castShadow = true;
    g.add(leg);
  }
  return g;
}

/** Fox for the forest and shore. */
export function buildFox(): THREE.Group {
  const g = new THREE.Group();
  const body = box(0.3, 0.28, 0.62, 0xe0782f);
  body.position.y = 0.4;
  body.castShadow = true;
  const head = box(0.26, 0.22, 0.26, 0xe0782f);
  head.position.set(0, 0.55, 0.4);
  const snout = box(0.12, 0.1, 0.12, 0xf5efe3);
  snout.position.set(0, 0.5, 0.56);
  const earL = box(0.07, 0.14, 0.05, 0xe0782f);
  earL.position.set(-0.09, 0.72, 0.36);
  const earR = earL.clone();
  earR.position.x = 0.09;
  const tail = box(0.14, 0.14, 0.4, 0xf5efe3);
  tail.position.set(0, 0.42, -0.48);
  tail.rotation.x = -0.4;
  g.add(body, head, snout, earL, earR, tail);
  for (const [x, z] of [
    [-0.1, 0.2],
    [0.1, 0.2],
    [-0.1, -0.2],
    [0.1, -0.2]
  ]) {
    const leg = box(0.08, 0.26, 0.08, 0x3a2a22);
    leg.position.set(x, 0.13, z);
    g.add(leg);
  }
  return g;
}
