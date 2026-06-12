import * as THREE from 'three';
import type { InputState } from './input';
import { resolveCircleAABB, type AABB } from './collision';
import { terrainHeight } from './terrain';

const STAND_EYE = 1.7;
const CROUCH_EYE = 1.0;
const RADIUS = 0.5;
const WALK_SPEED = 5;
const RUN_SPEED = 9;
const CROUCH_SPEED = 2.5;
const ADS_SPEED = 3.5;
const ACCEL = 45;
const FRICTION = 12;
const GRAVITY = -24;
const JUMP_VELOCITY = 8;
const SENSITIVITY = 0.0023;
const ADS_SENSITIVITY = 0.0012;
const PITCH_LIMIT = (85 * Math.PI) / 180;
const MAX_HEALTH = 100;
const REGEN_DELAY = 4;
const REGEN_RATE = 12;
const BASE_FOV = 75;
const SPRINT_FOV = 82;
const ADS_FOV = 55;

export class Player {
  camera: THREE.PerspectiveCamera;
  yaw = 0;
  pitch = 0;
  position = new THREE.Vector3(0, STAND_EYE, 14);
  velocity = new THREE.Vector3();
  health = MAX_HEALTH;
  crouching = false;
  running = false;
  grounded = true;
  aiming = false;
  /** Walk-cycle phase, distance-driven; viewmodel bob and footsteps key off it. */
  bobPhase = 0;
  /** Horizontal speed this frame (for bob amplitude elsewhere). */
  groundSpeed = 0;
  onHealthChange?: (health: number) => void;
  onDeath?: () => void;
  onFootstep?: (running: boolean) => void;
  private eyeHeight = STAND_EYE;
  private feetY = 0;
  private vy = 0;
  private sinceDamage = 999;
  private recoilPitch = 0;
  private recoilYaw = 0;
  private landDip = 0;
  private landDipVel = 0;

  constructor(aspect: number) {
    this.camera = new THREE.PerspectiveCamera(BASE_FOV, aspect, 0.1, 300);
    this.feetY = terrainHeight(this.position.x, this.position.z);
    this.syncCamera();
  }

  takeDamage(amount: number) {
    if (this.health <= 0) return;
    this.health = Math.max(0, this.health - amount);
    this.sinceDamage = 0;
    this.onHealthChange?.(this.health);
    if (this.health === 0) this.onDeath?.();
  }

  /** Back to life after the death screen. */
  revive() {
    this.health = MAX_HEALTH;
    this.sinceDamage = 999;
    this.onHealthChange?.(this.health);
  }

  /** Teleport to a spot on the ground (e.g. stepping out of a car). */
  placeAt(x: number, z: number) {
    this.feetY = terrainHeight(x, z);
    this.vy = 0;
    this.grounded = true;
    this.velocity.set(0, 0, 0);
    this.position.set(x, this.feetY + this.eyeHeight, z);
    this.syncCamera();
  }

  /** Camera kick on firing; recovers via spring in syncCamera. */
  kickRecoil() {
    this.recoilPitch += 0.014 + Math.random() * 0.006;
    this.recoilYaw += (Math.random() - 0.5) * 0.006;
  }

  update(dt: number, input: InputState, colliders: AABB[]) {
    this.aiming = input.aim && !input.run;
    const sens = this.aiming ? ADS_SENSITIVITY : SENSITIVITY;
    this.yaw -= input.lookDX * sens;
    this.pitch -= input.lookDY * sens;
    this.pitch = Math.max(-PITCH_LIMIT, Math.min(PITCH_LIMIT, this.pitch));

    this.crouching = input.crouch;
    this.running = input.run && !input.crouch && !this.aiming;
    const targetEye = this.crouching ? CROUCH_EYE : STAND_EYE;
    this.eyeHeight += (targetEye - this.eyeHeight) * Math.min(1, dt * 12);

    // Jump + gravity over rolling terrain
    const groundY = terrainHeight(this.position.x, this.position.z);
    if (input.jumpQueued && this.grounded && !this.crouching) {
      this.vy = JUMP_VELOCITY;
      this.grounded = false;
    }
    if (this.grounded) {
      // Snap to the slope while walking up/down hills
      this.feetY = groundY;
      this.vy = 0;
    } else {
      this.vy += GRAVITY * dt;
      this.feetY += this.vy * dt;
      if (this.feetY <= groundY) {
        // Landing dip scaled by impact speed (CoD-style)
        this.landDipVel -= Math.min(2.2, Math.abs(this.vy) * 0.16);
        this.feetY = groundY;
        this.vy = 0;
        this.grounded = true;
      }
    }

    const maxSpeed = this.aiming
      ? ADS_SPEED
      : this.crouching
        ? CROUCH_SPEED
        : this.running
          ? RUN_SPEED
          : WALK_SPEED;

    const sin = Math.sin(this.yaw);
    const cos = Math.cos(this.yaw);
    const wishX = input.moveX * cos - input.moveY * sin;
    const wishZ = -input.moveY * cos - input.moveX * sin;
    const len = Math.hypot(wishX, wishZ);

    if (len > 0.01) {
      this.velocity.x += (wishX / len) * ACCEL * dt;
      this.velocity.z += (wishZ / len) * ACCEL * dt;
      const sp = Math.hypot(this.velocity.x, this.velocity.z);
      if (sp > maxSpeed) {
        this.velocity.x = (this.velocity.x / sp) * maxSpeed;
        this.velocity.z = (this.velocity.z / sp) * maxSpeed;
      }
    } else {
      const damp = Math.max(0, 1 - FRICTION * dt);
      this.velocity.x *= damp;
      this.velocity.z *= damp;
    }

    let nx = this.position.x + this.velocity.x * dt;
    let nz = this.position.z + this.velocity.z * dt;
    for (const box of colliders) {
      [nx, nz] = resolveCircleAABB(nx, nz, RADIUS, box);
    }

    this.position.set(nx, this.feetY + this.eyeHeight, nz);

    // Distance-driven walk cycle: stops with the player, speeds up sprinting
    this.groundSpeed = Math.hypot(this.velocity.x, this.velocity.z);
    if (this.grounded && this.groundSpeed > 0.5) {
      const prev = this.bobPhase;
      this.bobPhase += this.groundSpeed * dt * 1.6;
      // Footstep on each half-cycle (one per foot)
      if (Math.floor(prev / Math.PI) !== Math.floor(this.bobPhase / Math.PI)) {
        this.onFootstep?.(this.running);
      }
    }

    // Recoil recovery + landing dip spring
    this.recoilPitch *= Math.max(0, 1 - dt * 8);
    this.recoilYaw *= Math.max(0, 1 - dt * 8);
    this.landDipVel += -this.landDip * 90 * dt - this.landDipVel * 11 * dt;
    this.landDip += this.landDipVel * dt;

    // One FOV target per frame: ADS < base < sprint
    const sprinting = this.running && this.groundSpeed > WALK_SPEED + 0.5;
    const targetFov = this.aiming ? ADS_FOV : sprinting ? SPRINT_FOV : BASE_FOV;
    if (Math.abs(this.camera.fov - targetFov) > 0.01) {
      this.camera.fov += (targetFov - this.camera.fov) * Math.min(1, dt * 8);
      this.camera.updateProjectionMatrix();
    }

    // Health regen
    this.sinceDamage += dt;
    if (this.sinceDamage > REGEN_DELAY && this.health > 0 && this.health < MAX_HEALTH) {
      this.health = Math.min(MAX_HEALTH, this.health + REGEN_RATE * dt);
      this.onHealthChange?.(this.health);
    }

    this.syncCamera();
  }

  private syncCamera() {
    this.camera.position.copy(this.position);
    let roll = 0;
    // Walk/sprint bob: vertical double-frequency + lateral sway + a hint of roll
    if (this.grounded && this.groundSpeed > 0.5) {
      const amp = (this.running ? 0.055 : 0.03) * (this.aiming ? 0.25 : 1);
      this.camera.position.y += Math.sin(this.bobPhase * 2) * amp;
      const lat = Math.sin(this.bobPhase) * amp * 0.6;
      this.camera.position.x += Math.cos(this.yaw) * lat;
      this.camera.position.z -= Math.sin(this.yaw) * lat;
      roll = Math.sin(this.bobPhase) * 0.006;
    }
    this.camera.position.y += this.landDip;
    this.camera.rotation.set(
      this.pitch + this.recoilPitch,
      this.yaw + this.recoilYaw,
      roll,
      'YXZ'
    );
  }
}
