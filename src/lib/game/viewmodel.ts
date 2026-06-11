// First-person gun: a procedural low-poly rifle parented to the camera, with
// look sway, walk bob, recoil and iron-sight ADS — no model downloads.
import * as THREE from 'three';
import type { Player } from './player';
import type { InputState } from './input';

const HIP_POS = new THREE.Vector3(0.28, -0.24, -0.5);
const ADS_POS = new THREE.Vector3(0, -0.155, -0.42);
const RELOAD_TIME = 1.2;

const gunmetal = new THREE.MeshLambertMaterial({ color: 0x37393f, flatShading: true });
const darkmetal = new THREE.MeshLambertMaterial({ color: 0x24262b, flatShading: true });
const wood = new THREE.MeshLambertMaterial({ color: 0x6d4c33, flatShading: true });

function box(w: number, h: number, d: number, mat: THREE.Material): THREE.Mesh {
  const m = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), mat);
  m.frustumCulled = false;
  return m;
}

export class ViewModel {
  gun = new THREE.Group();
  adsT = 0; // 0 hip → 1 fully aimed
  reloading = false;
  private reloadT = 0;
  private muzzle = new THREE.Object3D();
  private flash: THREE.Mesh;
  private flashTime = 0;
  private swayX = 0;
  private swayY = 0;
  private recoil = 0;
  private recoilVel = 0;

  constructor(camera: THREE.PerspectiveCamera) {
    // Receiver + barrel along -z
    const receiver = box(0.07, 0.1, 0.34, gunmetal);
    receiver.position.set(0, 0, -0.08);
    const barrel = new THREE.Mesh(new THREE.CylinderGeometry(0.016, 0.016, 0.34, 8), darkmetal);
    barrel.rotation.x = Math.PI / 2;
    barrel.position.set(0, 0.02, -0.42);
    barrel.frustumCulled = false;
    const handguard = box(0.06, 0.07, 0.26, wood);
    handguard.position.set(0, -0.01, -0.37);
    const stock = box(0.06, 0.09, 0.22, wood);
    stock.position.set(0, -0.02, 0.18);
    const grip = box(0.05, 0.12, 0.06, wood);
    grip.position.set(0, -0.1, 0.04);
    grip.rotation.x = 0.35;
    const mag = box(0.045, 0.14, 0.07, darkmetal);
    mag.position.set(0, -0.11, -0.12);
    mag.rotation.x = -0.18;
    // Iron sights: front post + rear notch posts
    const frontSight = box(0.012, 0.05, 0.012, darkmetal);
    frontSight.position.set(0, 0.07, -0.56);
    const rearL = box(0.012, 0.035, 0.015, darkmetal);
    rearL.position.set(-0.022, 0.065, 0.06);
    const rearR = rearL.clone();
    rearR.position.x = 0.022;

    this.muzzle.position.set(0, 0.02, -0.6);
    this.flash = new THREE.Mesh(
      new THREE.SphereGeometry(0.05, 6, 6),
      new THREE.MeshBasicMaterial({ color: 0xffe082, transparent: true, opacity: 0.95 })
    );
    this.flash.position.copy(this.muzzle.position);
    this.flash.visible = false;
    this.flash.frustumCulled = false;

    this.gun.add(
      receiver,
      barrel,
      handguard,
      stock,
      grip,
      mag,
      frontSight,
      rearL,
      rearR,
      this.muzzle,
      this.flash
    );
    this.gun.position.copy(HIP_POS);
    camera.add(this.gun);
  }

  muzzleWorld(out: THREE.Vector3): THREE.Vector3 {
    return this.muzzle.getWorldPosition(out);
  }

  kick() {
    this.recoilVel += 1.6 + Math.random() * 0.5;
    this.flashTime = 0.05;
    this.flash.visible = true;
    this.flash.rotation.z = Math.random() * Math.PI;
  }

  startReload(): boolean {
    if (this.reloading) return false;
    this.reloading = true;
    this.reloadT = 0;
    return true;
  }

  update(dt: number, input: InputState, player: Player) {
    // ADS blend (sprinting or reloading forces hip)
    const wantAds = player.aiming && !this.reloading;
    this.adsT += ((wantAds ? 1 : 0) - this.adsT) * Math.min(1, dt * 9);
    const t = this.adsT * this.adsT * (3 - 2 * this.adsT);

    // Look sway, springs back to center
    this.swayX += (Math.max(-0.035, Math.min(0.035, -input.lookDX * 0.0007)) - this.swayX) * Math.min(1, dt * 10);
    this.swayY += (Math.max(-0.03, Math.min(0.03, input.lookDY * 0.0006)) - this.swayY) * Math.min(1, dt * 10);
    const swayScale = 1 - t * 0.8;

    // Walk bob: figure-8 driven by the player's distance-based cycle
    let bobX = 0;
    let bobY = 0;
    if (player.grounded && player.groundSpeed > 0.5) {
      const amp = (player.running ? 0.022 : 0.013) * (1 - t * 0.9);
      bobX = Math.sin(player.bobPhase) * amp;
      bobY = -Math.abs(Math.cos(player.bobPhase)) * amp;
    }

    // Recoil spring (gun shoves back and rotates up, then recovers)
    this.recoil += this.recoilVel * dt;
    this.recoilVel += (-this.recoil * 140 - this.recoilVel * 16) * dt;

    // Reload: dip the gun down and roll it, then bring it back
    let reloadDip = 0;
    let reloadRoll = 0;
    if (this.reloading) {
      this.reloadT += dt / RELOAD_TIME;
      if (this.reloadT >= 1) this.reloading = false;
      else {
        const r = Math.sin(Math.PI * this.reloadT);
        reloadDip = r * 0.22;
        reloadRoll = r * 0.7;
      }
    }

    this.gun.position.set(
      HIP_POS.x + (ADS_POS.x - HIP_POS.x) * t + this.swayX * swayScale + bobX,
      HIP_POS.y + (ADS_POS.y - HIP_POS.y) * t + this.swayY * swayScale + bobY - reloadDip,
      HIP_POS.z + (ADS_POS.z - HIP_POS.z) * t + this.recoil * 0.06
    );
    this.gun.rotation.set(
      this.recoil * 0.05 + this.swayY * 1.5 * swayScale - reloadRoll * 0.5,
      this.swayX * 1.2 * swayScale,
      reloadRoll * 0.4
    );

    if (this.flashTime > 0) {
      this.flashTime -= dt;
      if (this.flashTime <= 0) this.flash.visible = false;
    }
  }

  dispose() {
    this.gun.parent?.remove(this.gun);
  }
}
