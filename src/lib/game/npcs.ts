import * as THREE from 'three';
import type { Section } from '$lib/content';
import { loadModel, cloneRigged, NPC_MODELS } from './assets';
import type { ChunkManager } from './chunks';
import { terrainHeight } from './terrain';

const NPC_HEIGHT = 1.85;
const HEADSHOT_Y = 1.45; // hit point above this (group-relative) counts as a headshot
const DESPAWN_DIST = 52;
const SPAWN_MIN = 18;
const SPAWN_MAX = 36;

// ── Enemy archetypes ─────────────────────────────────────────────────────────
// Per-type tuning. The `grunt` row is the original rifle NPC; the others trade
// speed/range/toughness for variety. `tint` multiplies the rig's mesh colors so
// each type reads at a glance. `melee` enemies carry no gun and deal contact
// damage instead of firing tracers.
export type NpcType = 'grunt' | 'rusher' | 'heavy' | 'sniper';

type NpcConfig = {
  hp: [number, number];
  wanderSpeed: number;
  chaseSpeed: number;
  engageRange: number;
  shootRange: number;
  keepDistance: number;
  shotCooldown: [number, number];
  shotDamage: [number, number];
  /** Probability a shot connects when the player is standing (crouch lowers it). */
  accuracy: number;
  scale: number;
  tint: number;
  score: number;
  melee?: boolean;
};

export const NPC_TYPES: Record<NpcType, NpcConfig> = {
  grunt: {
    hp: [2, 6],
    wanderSpeed: 1.6,
    chaseSpeed: 3.2,
    engageRange: 17,
    shootRange: 15,
    keepDistance: 8,
    shotCooldown: [1.7, 3.0],
    shotDamage: [6, 14],
    accuracy: 0.55,
    scale: 1,
    tint: 0xffffff,
    score: 100
  },
  rusher: {
    hp: [1, 2],
    wanderSpeed: 2.4,
    chaseSpeed: 5.4,
    engageRange: 22,
    shootRange: 1.7, // "shoot" range doubles as the melee reach
    keepDistance: 0, // never backs off — charges into contact
    shotCooldown: [0.8, 1.2],
    shotDamage: [10, 18],
    accuracy: 1,
    scale: 0.85,
    tint: 0xff6a5a,
    score: 120,
    melee: true
  },
  heavy: {
    hp: [10, 14],
    wanderSpeed: 1.0,
    chaseSpeed: 2.0,
    engageRange: 16,
    shootRange: 14,
    keepDistance: 9,
    shotCooldown: [2.4, 3.6],
    shotDamage: [14, 24],
    accuracy: 0.5,
    scale: 1.28,
    tint: 0x6b7080,
    score: 250
  },
  sniper: {
    hp: [4, 6],
    wanderSpeed: 0.8,
    chaseSpeed: 1.4,
    engageRange: 34,
    shootRange: 30,
    keepDistance: 24, // holds its distance and picks you off
    shotCooldown: [3.2, 4.6],
    shotDamage: [22, 34],
    accuracy: 0.82,
    scale: 1,
    tint: 0x7ad8ff,
    score: 200
  }
};

export function makeLabelSprite(text: string, color: number): THREE.Sprite {
  const canvas = document.createElement('canvas');
  canvas.width = 512;
  canvas.height = 128;
  const ctx = canvas.getContext('2d')!;
  ctx.font = "72px 'VT323', monospace";
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillStyle = 'rgba(13, 11, 8, 0.55)';
  const w = ctx.measureText(text).width + 48;
  ctx.fillRect((canvas.width - w) / 2, 16, w, 96);
  ctx.fillStyle = '#' + new THREE.Color(color).offsetHSL(0, 0, 0.25).getHexString();
  ctx.fillText(text, canvas.width / 2, canvas.height / 2 + 4);
  const texture = new THREE.CanvasTexture(canvas);
  const sprite = new THREE.Sprite(
    new THREE.SpriteMaterial({ map: texture, transparent: true, depthTest: false })
  );
  sprite.scale.set(3.2, 0.8, 1);
  return sprite;
}

type NpcState = 'wander' | 'engage' | 'dead';

// ── NPC rifle ──────────────────────────────────────────────────────────────
// Tiny procedural gun (same technique as the first-person viewmodel) held in
// the right hand; the muzzle Object3D is the tracer origin when the NPC fires.

const npcGunmetal = new THREE.MeshLambertMaterial({ color: 0x37393f, flatShading: true });
const npcWood = new THREE.MeshLambertMaterial({ color: 0x6d4c33, flatShading: true });

function gunBox(w: number, h: number, d: number, mat: THREE.Material): THREE.Mesh {
  const m = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), mat);
  m.frustumCulled = false; // parented to skinned-rig bones; bounds lag the pose
  return m;
}

function makeNpcGun(): { gun: THREE.Group; muzzle: THREE.Object3D } {
  const gun = new THREE.Group();
  const body = gunBox(0.05, 0.08, 0.32, npcGunmetal);
  body.position.z = -0.06;
  const barrel = new THREE.Mesh(new THREE.CylinderGeometry(0.014, 0.014, 0.24, 6), npcGunmetal);
  barrel.rotation.x = Math.PI / 2;
  barrel.position.set(0, 0.015, -0.32);
  barrel.frustumCulled = false;
  const stock = gunBox(0.045, 0.07, 0.16, npcWood);
  stock.position.set(0, -0.015, 0.16);
  const grip = gunBox(0.04, 0.1, 0.05, npcWood);
  grip.position.set(0, -0.08, 0.02);
  grip.rotation.x = 0.3;
  const muzzle = new THREE.Object3D();
  muzzle.position.set(0, 0.015, -0.46);
  gun.add(body, barrel, stock, grip, muzzle);
  return { gun, muzzle };
}

const _aimV1 = new THREE.Vector3();
const _aimV2 = new THREE.Vector3();
const _aimQ1 = new THREE.Quaternion();
const _aimQ2 = new THREE.Quaternion();

export type NpcShot = {
  from: THREE.Vector3;
  to: THREE.Vector3;
  hit: boolean;
  damage: number;
  melee?: boolean;
};

// All six Quaternius rigs share these bone names (GLTFLoader strips the '.'
// from e.g. 'UpperArm.L'); none ship animation clips, so we animate bones
// procedurally.
const BONE_NAMES = [
  'Hips',
  'Torso',
  'Head',
  'UpperArmL',
  'UpperArmR',
  'LowerArmL',
  'LowerArmR',
  'UpperLegL',
  'UpperLegR',
  'LowerLegL',
  'LowerLegR'
] as const;
type BoneName = (typeof BONE_NAMES)[number];

export class Npc {
  section: Section;
  type: NpcType;
  cfg: NpcConfig;
  scoreValue: number;
  group = new THREE.Group();
  hittable: THREE.Object3D[] = [];
  state: NpcState = 'wander';
  // Random toughness per NPC, bounded by the archetype's HP range
  maxHp: number;
  hp: number;
  private wanderTarget = new THREE.Vector2();
  private wanderTimer = 0;
  private shotTimer = 2 + Math.random() * 2;
  private deadTimer = 0;
  private flinchTimer = 0;
  removeMe = false;
  onDeath?: (npc: Npc) => void;

  private model: THREE.Object3D;
  private bones = new Map<BoneName, THREE.Bone>();
  private restX = new Map<BoneName, number>();
  private walkPhase = 0;
  private moveBlend = 0; // 0 idle → 1 full stride, smoothed
  private idleTime = Math.random() * 10;
  private aimBlend = 0; // 0 relaxed → 1 gun raised at the player, smoothed
  private muzzle: THREE.Object3D | null = null;
  private armAxis = new THREE.Vector3(0, 1, 0); // upper-arm bone-local "along the arm"

  constructor(section: Section, model: THREE.Object3D, x: number, z: number, type: NpcType = 'grunt') {
    this.section = section;
    this.type = type;
    const cfg = NPC_TYPES[type];
    this.cfg = cfg;
    this.scoreValue = cfg.score;
    this.maxHp = cfg.hp[0] + Math.floor(Math.random() * (cfg.hp[1] - cfg.hp[0] + 1));
    this.hp = this.maxHp;
    // Quaternius characters render at ~1.85u natively (armature compensates its
    // own x100 scale) — do NOT bbox-normalize; the bind-pose bbox lies.
    this.model = model;
    this.group.add(model);

    const label = makeLabelSprite(section.label, section.color);
    label.position.y = NPC_HEIGHT + 0.5;
    this.group.add(label);
    this.group.position.set(x, terrainHeight(x, z), z);

    const tint = new THREE.Color(cfg.tint);
    model.traverse((o) => {
      const mesh = o as THREE.Mesh;
      if (mesh.isMesh || (o as THREE.SkinnedMesh).isSkinnedMesh) {
        o.frustumCulled = false; // skinned bounds lag the rig; avoid pop-out
        // Tint the archetype so it reads at a glance (clone material first so we
        // don't bleed color into the shared cached one).
        if (cfg.tint !== 0xffffff && mesh.material) {
          const recolor = (m: THREE.Material) => {
            const lm = (m as THREE.MeshStandardMaterial).clone();
            const col = (lm as THREE.MeshStandardMaterial).color;
            if (col) col.multiply(tint);
            return lm;
          };
          mesh.material = Array.isArray(mesh.material)
            ? mesh.material.map(recolor)
            : recolor(mesh.material);
        }
      }
      if ((o as THREE.Bone).isBone && BONE_NAMES.includes(o.name as BoneName)) {
        // Relax the T-pose: arms down at the sides
        if (/^UpperArm[LR]$/.test(o.name)) o.rotation.x -= 1.3;
        this.bones.set(o.name as BoneName, o as THREE.Bone);
        this.restX.set(o.name as BoneName, o.rotation.x);
      }
    });

    // Put a rifle in the right hand (melee archetypes go unarmed). The Quaternius
    // armature carries an internal x100 scale, so the gun sits in a holder that
    // cancels the bone's world scale; the lower arm's child bone marks the hand.
    const lowerArm = this.bones.get('LowerArmR');
    const upperArm = this.bones.get('UpperArmR');
    if (!cfg.melee && lowerArm && upperArm) {
      this.model.updateMatrixWorld(true);
      const ws = lowerArm.getWorldScale(new THREE.Vector3());
      const holder = new THREE.Group();
      holder.scale.setScalar(1 / (ws.x || 1));
      const handBone = lowerArm.children.find((c) => (c as THREE.Bone).isBone);
      const along = handBone ? handBone.position.clone() : new THREE.Vector3(0, 0.26 / (ws.x || 1), 0);
      holder.position.copy(along);
      // Barrel (-z) follows the arm direction so the aimed pose points the gun
      holder.quaternion.setFromUnitVectors(new THREE.Vector3(0, 0, -1), along.clone().normalize());
      const { gun, muzzle } = makeNpcGun();
      holder.add(gun);
      lowerArm.add(holder);
      this.muzzle = muzzle;
      // Aim rotations point the upper arm along the shoulder→hand direction
      const elbow = lowerArm.position.clone();
      if (elbow.lengthSq() > 1e-6) this.armAxis.copy(elbow.normalize());
    }

    // Raycasting skinned meshes is unreliable (bind-pose bounds), so shots
    // test against an invisible capsule-ish hitbox instead.
    const hitbox = new THREE.Mesh(
      new THREE.BoxGeometry(0.9, NPC_HEIGHT, 0.9),
      new THREE.MeshBasicMaterial({ visible: false })
    );
    hitbox.position.y = NPC_HEIGHT / 2;
    hitbox.userData.npc = this;
    this.group.add(hitbox);
    this.hittable.push(hitbox);

    this.pickWanderTarget();
  }

  private pickWanderTarget() {
    const a = Math.random() * Math.PI * 2;
    const d = 4 + Math.random() * 8;
    this.wanderTarget.set(this.group.position.x + Math.cos(a) * d, this.group.position.z + Math.sin(a) * d);
    this.wanderTimer = 4 + Math.random() * 4;
  }

  /** Hit point above HEADSHOT_Y (group-relative) is a headshot. Returns true if killed. */
  hit(damage: number): boolean {
    if (this.state === 'dead') return false;
    this.hp -= damage;
    if (this.hp <= 0) {
      this.state = 'dead';
      this.deadTimer = 0;
      this.onDeath?.(this);
      return true;
    }
    this.flinchTimer = 0.28;
    return false;
  }

  isHeadshot(hitPoint: THREE.Vector3): boolean {
    return hitPoint.y - this.group.position.y > HEADSHOT_Y;
  }

  // Procedural skeletal animation: stride from walkPhase, breathing at idle,
  // flinch as an additive torso kick.
  private animate(dt: number, moving: boolean, speed: number, chasing: boolean, aimTarget: THREE.Vector3) {
    this.idleTime += dt;
    const targetBlend = moving ? 1 : 0;
    this.moveBlend += (targetBlend - this.moveBlend) * Math.min(1, dt * 8);
    if (moving) this.walkPhase += dt * speed * 3.2;

    const s = Math.sin(this.walkPhase) * this.moveBlend;
    const stride = chasing ? 0.62 : 0.5;
    const set = (name: BoneName, offset: number) => {
      const bone = this.bones.get(name);
      const rest = this.restX.get(name);
      if (bone && rest !== undefined) {
        bone.rotation.x += (rest + offset - bone.rotation.x) * Math.min(1, dt * 12);
      }
    };

    // Knees only bend backward, on the leg swinging back; offset phase per side
    const kneeL = Math.max(0, -Math.sin(this.walkPhase - 0.5)) * 0.55 * this.moveBlend;
    const kneeR = Math.max(0, Math.sin(this.walkPhase - 0.5)) * 0.55 * this.moveBlend;
    set('UpperLegL', s * stride);
    set('UpperLegR', -s * stride);
    set('LowerLegL', kneeL);
    set('LowerLegR', kneeR);
    // Arms counter-swing the legs; the right arm straightens while aiming
    set('UpperArmL', -s * 0.38);
    set('UpperArmR', s * 0.38 * (1 - this.aimBlend));
    set('LowerArmL', -0.15 * this.moveBlend);
    set('LowerArmR', -0.15 * this.moveBlend * (1 - this.aimBlend));

    // Aim: blend the upper-arm bone from the walk pose toward pointing the
    // arm (and the gun parented to it) straight at the player, in world space.
    const upper = this.bones.get('UpperArmR');
    if (upper && upper.parent && this.aimBlend > 0.01) {
      upper.updateWorldMatrix(true, false);
      const shoulder = _aimV1.setFromMatrixPosition(upper.matrixWorld);
      const dir = _aimV2.copy(aimTarget).sub(shoulder).normalize();
      // World direction → upper-arm parent space, then rotate the arm axis onto it
      dir.applyQuaternion(upper.parent.getWorldQuaternion(_aimQ1).invert());
      _aimQ2.setFromUnitVectors(this.armAxis, dir);
      upper.quaternion.slerp(_aimQ2, this.aimBlend);
    }

    // Torso: run lean + idle breathing + flinch kick
    let torso = chasing ? 0.12 * this.moveBlend : 0;
    torso += Math.sin(this.idleTime * 2) * 0.02 * (1 - this.moveBlend);
    if (this.flinchTimer > 0) torso -= (this.flinchTimer / 0.28) * 0.35;
    set('Torso', torso);

    // Bounce in the step
    this.model.position.y = Math.abs(Math.sin(this.walkPhase)) * 0.06 * this.moveBlend;
  }

  /** Returns a shot description when the NPC fires this frame. */
  update(
    dt: number,
    playerPos: THREE.Vector3,
    playerCrouching: boolean,
    chunks: ChunkManager
  ): NpcShot | null {
    if (this.state === 'dead') {
      this.deadTimer += dt;
      // Tween a topple
      const t = Math.min(this.deadTimer / 0.5, 1);
      this.group.rotation.x = (t * (2 - t) * Math.PI) / 2;
      if (this.deadTimer > 1.6) this.removeMe = true;
      return null;
    }

    this.flinchTimer = Math.max(0, this.flinchTimer - dt);

    const cfg = this.cfg;
    const pos = this.group.position;
    const distToPlayer = Math.hypot(playerPos.x - pos.x, playerPos.z - pos.z);
    this.state = distToPlayer < cfg.engageRange ? 'engage' : 'wander';

    // Raise/lower the gun (firing is gated on being mostly aimed-in). Melee
    // archetypes never raise a gun — they just charge.
    const wantAim =
      !cfg.melee &&
      this.state === 'engage' &&
      distToPlayer < cfg.shootRange * 1.2 &&
      this.flinchTimer <= 0;
    this.aimBlend += ((wantAim ? 1 : 0) - this.aimBlend) * Math.min(1, dt * 6);

    let moveX = 0;
    let moveZ = 0;
    let speed = cfg.wanderSpeed;
    let shot: NpcShot | null = null;

    if (this.state === 'wander') {
      this.wanderTimer -= dt;
      const dx = this.wanderTarget.x - pos.x;
      const dz = this.wanderTarget.y - pos.z;
      if (this.wanderTimer <= 0 || Math.hypot(dx, dz) < 0.5) this.pickWanderTarget();
      else {
        const d = Math.hypot(dx, dz);
        moveX = dx / d;
        moveZ = dz / d;
        this.group.rotation.y = Math.atan2(moveX, moveZ);
      }
    } else {
      // Face the player
      const dx = playerPos.x - pos.x;
      const dz = playerPos.z - pos.z;
      this.group.rotation.y = Math.atan2(dx, dz);
      // Close distance until comfortable, then hold
      if (distToPlayer > cfg.keepDistance) {
        moveX = dx / distToPlayer;
        moveZ = dz / distToPlayer;
        speed = cfg.chaseSpeed;
      }

      // Attack: melee lands a contact hit at point-blank; ranged fires a tracer
      // (flinching interrupts aim; must be visibly aimed-in first).
      this.shotTimer -= dt;
      const baseAcc = playerCrouching ? cfg.accuracy * 0.6 : cfg.accuracy;
      if (cfg.melee) {
        if (this.shotTimer <= 0 && distToPlayer < cfg.shootRange && this.flinchTimer <= 0) {
          this.shotTimer =
            cfg.shotCooldown[0] + Math.random() * (cfg.shotCooldown[1] - cfg.shotCooldown[0]);
          const at = pos.clone();
          at.y += 1.2;
          shot = {
            from: at,
            to: playerPos.clone(),
            hit: true,
            melee: true,
            damage: Math.round(cfg.shotDamage[0] + Math.random() * (cfg.shotDamage[1] - cfg.shotDamage[0]))
          };
        }
      } else if (
        this.shotTimer <= 0 &&
        distToPlayer < cfg.shootRange &&
        this.flinchTimer <= 0 &&
        this.aimBlend > 0.85
      ) {
        this.shotTimer =
          cfg.shotCooldown[0] + Math.random() * (cfg.shotCooldown[1] - cfg.shotCooldown[0]);
        const hit = Math.random() < baseAcc;
        // Tracer starts at the gun barrel
        const from = this.muzzle ? this.muzzle.getWorldPosition(new THREE.Vector3()) : pos.clone();
        if (!this.muzzle) from.y = pos.y + 1.4;
        const to = playerPos.clone();
        if (!hit) {
          to.x += (Math.random() - 0.5) * 3;
          to.y += (Math.random() - 0.2) * 2;
          to.z += (Math.random() - 0.5) * 3;
        }
        shot = {
          from,
          to,
          hit,
          damage: Math.round(cfg.shotDamage[0] + Math.random() * (cfg.shotDamage[1] - cfg.shotDamage[0]))
        };
      }
    }

    // Flinching pauses movement
    const moving = (moveX !== 0 || moveZ !== 0) && this.flinchTimer <= 0;
    if (moving) {
      const nx = pos.x + moveX * speed * dt;
      const nz = pos.z + moveZ * speed * dt;
      if (chunks.isFree(nx, nz, 0.5)) {
        pos.x = nx;
        pos.z = nz;
        pos.y = terrainHeight(nx, nz);
      } else {
        this.pickWanderTarget();
      }
    }
    this.animate(dt, moving, speed, this.state === 'engage', playerPos);
    return shot;
  }
}

export type WaveInfo = { wave: number; kills: number; quota: number };

export class NpcManager {
  npcs: Npc[] = [];
  onDeath?: (npc: Npc) => void;
  /** Fired when wave progress changes (a kill counts, or a new wave begins). */
  onWave?: (info: WaveInfo, started: boolean) => void;
  private scene: THREE.Scene;
  private sections: Section[];
  private spawnCursor = 0;
  private pendingSpawns = 0;
  // Wave state: kill `quota` enemies to advance; each wave raises the live
  // population cap and shifts the type mix toward tougher archetypes.
  private wave = 1;
  private waveKills = 0;

  constructor(scene: THREE.Scene, sections: Section[]) {
    this.scene = scene;
    this.sections = sections;
  }

  private get quota(): number {
    return 6 + this.wave * 2;
  }

  /** Live population cap grows with the wave. */
  private get desired(): number {
    return Math.min(14, 6 + Math.floor(this.wave * 1.2));
  }

  waveInfo(): WaveInfo {
    return { wave: this.wave, kills: this.waveKills, quota: this.quota };
  }

  resetWaves() {
    this.wave = 1;
    this.waveKills = 0;
    this.onWave?.(this.waveInfo(), false);
  }

  /** Called by the owner when an NPC dies, to drive wave progression. */
  registerKill() {
    this.waveKills++;
    if (this.waveKills >= this.quota) {
      this.wave++;
      this.waveKills = 0;
      this.onWave?.(this.waveInfo(), true);
    } else {
      this.onWave?.(this.waveInfo(), false);
    }
  }

  /** Bias the spawn mix toward tougher types as waves climb. */
  private pickType(): NpcType {
    const w = this.wave;
    const r = Math.random();
    // Snipers appear from wave 3, sparsely.
    if (w >= 3 && r < 0.12) return 'sniper';
    // Heavies ramp in from wave 2.
    if (w >= 2 && r < 0.12 + Math.min(0.25, w * 0.04)) return 'heavy';
    // Rushers ramp harder — they drive the pressure.
    if (r < 0.5 + Math.min(0.3, w * 0.06)) return 'rusher';
    return 'grunt';
  }

  update(
    dt: number,
    playerPos: THREE.Vector3,
    playerCrouching: boolean,
    chunks: ChunkManager
  ): NpcShot[] {
    const shots: NpcShot[] = [];
    for (const npc of this.npcs) {
      const shot = npc.update(dt, playerPos, playerCrouching, chunks);
      if (shot) shots.push(shot);
    }

    // Remove finished corpses and far NPCs (Minecraft-style despawn)
    for (let i = this.npcs.length - 1; i >= 0; i--) {
      const npc = this.npcs[i];
      const dist = Math.hypot(playerPos.x - npc.group.position.x, playerPos.z - npc.group.position.z);
      if (npc.removeMe || (npc.state !== 'dead' && dist > DESPAWN_DIST)) {
        this.scene.remove(npc.group);
        this.npcs.splice(i, 1);
      }
    }

    // Top up population near the player
    if (this.npcs.length + this.pendingSpawns < this.desired) {
      this.pendingSpawns++;
      this.spawn(playerPos, chunks).finally(() => this.pendingSpawns--);
    }
    return shots;
  }

  private async spawn(playerPos: THREE.Vector3, chunks: ChunkManager) {
    // Round-robin sections so every portfolio section keeps appearing
    const section = this.sections[this.spawnCursor % this.sections.length];
    this.spawnCursor++;
    const type = this.pickType();

    let x = 0;
    let z = 0;
    let ok = false;
    for (let tries = 0; tries < 10 && !ok; tries++) {
      const a = Math.random() * Math.PI * 2;
      const d = SPAWN_MIN + Math.random() * (SPAWN_MAX - SPAWN_MIN);
      x = playerPos.x + Math.cos(a) * d;
      z = playerPos.z + Math.sin(a) * d;
      ok = chunks.isFree(x, z, 1);
    }
    if (!ok) return;

    const variants = NPC_MODELS[section.id];
    const gltf = await loadModel(variants[Math.floor(Math.random() * variants.length)]);
    const model = cloneRigged(gltf);
    const npc = new Npc(section, model, x, z, type);
    // Archetype base size × slight per-NPC variation (label/hitbox scale along)
    npc.group.scale.setScalar(npc.cfg.scale * (0.92 + Math.random() * 0.16));
    npc.onDeath = (n) => this.onDeath?.(n);
    this.scene.add(npc.group);
    this.npcs.push(npc);
  }

  hittables(): THREE.Object3D[] {
    const out: THREE.Object3D[] = [];
    for (const npc of this.npcs) if (npc.state !== 'dead') out.push(...npc.hittable);
    return out;
  }

  dispose() {
    for (const npc of this.npcs) this.scene.remove(npc.group);
    this.npcs = [];
  }
}
