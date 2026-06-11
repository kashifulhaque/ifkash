import * as THREE from 'three';
import type { Section } from '$lib/content';
import { InputManager } from './input';
import { Player } from './player';
import { Environment } from './world';
import { ChunkManager } from './chunks';
import { NpcManager, type Npc } from './npcs';
import { LootCrate, INTERACT_RANGE, CRATE_DESPAWN_DIST } from './crates';
import { Shooter } from './shooting';
import { GameAudio } from './audio';
import { ViewModel } from './viewmodel';
import { preloadAll } from './assets';

export const MAG_SIZE = 12;

export type GameCallbacks = {
  onLockChange: (locked: boolean) => void;
  onInteractPrompt: (label: string | null) => void;
  onOpenSection: (section: Section) => void;
  onHealthChange: (health: number) => void;
  onPlayerHit: () => void;
  onPlayerDeath: () => void;
  onHitMarker: (headshot: boolean, killed: boolean) => void;
  onAmmoChange: (ammo: number, reloading: boolean) => void;
  onScoreChange: (score: number, streak: number) => void;
  onAimChange: (aiming: boolean) => void;
  onYawChange: (yaw: number) => void;
};

export class Game {
  input: InputManager;
  audio = new GameAudio();

  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private player: Player;
  private environment: Environment;
  private chunks: ChunkManager;
  private npcs: NpcManager;
  private crates: LootCrate[] = [];
  private shooter: Shooter;
  private viewModel: ViewModel;
  private ammo = MAG_SIZE;
  private wasReloading = false;
  private score = 0;
  private streak = 0;
  private aimShown = false;
  private yawShown = 999;
  private callbacks: GameCallbacks;
  private canvas: HTMLCanvasElement;
  private raf = 0;
  private lastTime = 0;
  private paused = false;
  private promptShown: string | null = null;
  private disposed = false;

  constructor(canvas: HTMLCanvasElement, sections: Section[], callbacks: GameCallbacks) {
    this.canvas = canvas;
    this.callbacks = callbacks;

    // Throws if WebGL is unavailable — caller catches and shows fallback.
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

    this.player = new Player(canvas.clientWidth / Math.max(canvas.clientHeight, 1));
    this.player.onHealthChange = (h) => callbacks.onHealthChange(h);
    this.player.onDeath = () => {
      this.streak = 0;
      callbacks.onScoreChange(this.score, this.streak);
      callbacks.onPlayerDeath();
    };
    this.player.onFootstep = (running) => this.audio.footstep(running);
    // The camera must live in the scene graph so the gun can be parented to it
    this.scene.add(this.player.camera);
    this.viewModel = new ViewModel(this.player.camera);

    this.environment = new Environment(this.scene);
    this.chunks = new ChunkManager(this.scene);
    this.chunks.update(this.player.position.x, this.player.position.z);

    this.npcs = new NpcManager(this.scene, sections);
    this.npcs.onDeath = (npc) => this.dropCrate(npc);

    this.shooter = new Shooter(this.scene);
    this.input = new InputManager(canvas, (locked) => callbacks.onLockChange(locked));

    preloadAll().catch((e) => console.warn('model preload failed', e));

    window.addEventListener('resize', this.onResize);
    this.onResize();
    this.lastTime = performance.now();
    this.raf = requestAnimationFrame(this.loop);
  }

  requestLock() {
    this.input.requestLock();
  }

  startAudio() {
    this.audio.start();
  }

  toggleMute(): boolean {
    return this.audio.toggleMute();
  }

  setPaused(paused: boolean) {
    this.paused = paused;
    this.input.enabled = !paused;
    if (paused) this.input.exitLock();
  }

  private tryFire() {
    if (this.viewModel.reloading) return;
    if (this.ammo <= 0) {
      this.audio.dryFire();
      this.tryReload(true);
      return;
    }
    this.ammo--;
    this.callbacks.onAmmoChange(this.ammo, false);
    this.audio.gunshot();
    this.player.kickRecoil();
    this.viewModel.kick();
    const muzzle = this.viewModel.muzzleWorld(new THREE.Vector3());
    const result = this.shooter.fire(this.player.camera, this.npcs.hittables(), muzzle);
    if (result) {
      if (result.headshot) this.audio.headshotConfirm();
      else this.audio.hitConfirm();
      this.callbacks.onHitMarker(result.headshot, result.killed);
      if (result.killed) {
        this.score += result.headshot ? 150 : 100;
        this.streak++;
        if (this.streak >= 2) this.audio.streak(this.streak);
        this.callbacks.onScoreChange(this.score, this.streak);
      }
    }
    if (this.ammo === 0) this.tryReload(true);
  }

  private tryReload(auto: boolean) {
    if (this.ammo >= MAG_SIZE && !auto) return;
    if (this.viewModel.startReload()) {
      this.audio.reload();
      this.callbacks.onAmmoChange(this.ammo, true);
    }
  }

  private dropCrate(npc: Npc) {
    if (this.crates.some((cr) => cr.section.id === npc.section.id)) return;
    // Probabilistic drops — tankier NPCs are likelier to drop. Sections respawn
    // round-robin, so a missed drop only delays access to that content.
    const chance = 0.35 + 0.06 * npc.maxHp;
    if (Math.random() > chance) return;
    const crate = new LootCrate(npc.section, npc.group.position.x, npc.group.position.z);
    this.scene.add(crate.group);
    this.crates.push(crate);
  }

  private removeCrate(crate: LootCrate) {
    this.scene.remove(crate.group);
    this.crates = this.crates.filter((c) => c !== crate);
  }

  private onResize = () => {
    const w = this.canvas.clientWidth || window.innerWidth;
    const h = this.canvas.clientHeight || window.innerHeight;
    this.renderer.setSize(w, h, false);
    this.player.camera.aspect = w / h;
    this.player.camera.updateProjectionMatrix();
  };

  private loop = (now: number) => {
    if (this.disposed) return;
    this.raf = requestAnimationFrame(this.loop);

    const dt = Math.min((now - this.lastTime) / 1000, 0.05);
    this.lastTime = now;

    const input = this.input.read();
    const ppos = this.player.position;

    if (!this.paused) {
      this.chunks.update(ppos.x, ppos.z);
      this.player.update(dt, input, this.chunks.collidersNear(ppos.x, ppos.z));

      if (input.fireQueued) this.tryFire();
      if (input.reloadQueued) this.tryReload(false);

      this.viewModel.update(dt, input, this.player);
      // Reload finished this frame → top off the mag
      if (this.wasReloading && !this.viewModel.reloading) {
        this.ammo = MAG_SIZE;
        this.callbacks.onAmmoChange(this.ammo, false);
      }
      this.wasReloading = this.viewModel.reloading;

      // HUD signals: ADS crosshair + compass yaw
      if (this.player.aiming !== this.aimShown) {
        this.aimShown = this.player.aiming;
        this.callbacks.onAimChange(this.aimShown);
      }
      if (Math.abs(this.player.yaw - this.yawShown) > 0.01) {
        this.yawShown = this.player.yaw;
        this.callbacks.onYawChange(this.yawShown);
      }

      // NPC AI: returns any shots fired at the player this frame
      const shots = this.npcs.update(dt, ppos, this.player.crouching, this.chunks);
      for (const shot of shots) {
        this.audio.enemyShot();
        this.shooter.enemyShot(shot.from, shot.to, shot.hit);
        if (shot.hit) {
          this.player.takeDamage(shot.damage);
          this.audio.playerHurt();
          this.callbacks.onPlayerHit();
        }
      }

      // Crates: proximity prompt, interact, despawn when far
      let nearest: LootCrate | null = null;
      let nearestDist = INTERACT_RANGE;
      for (let i = this.crates.length - 1; i >= 0; i--) {
        const crate = this.crates[i];
        const d = crate.distanceTo(ppos.x, ppos.z);
        if (d > CRATE_DESPAWN_DIST) {
          this.removeCrate(crate);
          continue;
        }
        if (d < nearestDist) {
          nearest = crate;
          nearestDist = d;
        }
      }
      const prompt = nearest ? nearest.section.label : null;
      if (prompt !== this.promptShown) {
        this.promptShown = prompt;
        this.callbacks.onInteractPrompt(prompt);
      }
      if (nearest && input.interactQueued) {
        this.audio.crateOpen();
        this.callbacks.onOpenSection(nearest.section);
        this.removeCrate(nearest);
        this.promptShown = null;
        this.callbacks.onInteractPrompt(null);
      }
    }

    // Day/night clock and cloud drift freeze with the rest of the sim
    this.environment.update(this.paused ? 0 : dt, ppos.x, ppos.z);
    for (const crate of this.crates) crate.update(dt);
    this.shooter.update(dt);
    this.input.consumeFrame();

    this.renderer.render(this.scene, this.player.camera);
  };

  dispose() {
    this.disposed = true;
    cancelAnimationFrame(this.raf);
    window.removeEventListener('resize', this.onResize);
    this.input.dispose();
    this.audio.dispose();
    this.viewModel.dispose();
    this.npcs.dispose();
    this.chunks.dispose();
    this.scene.traverse((obj) => {
      const mesh = obj as THREE.Mesh;
      if (mesh.geometry) mesh.geometry.dispose();
      const mat = (mesh as THREE.Mesh).material as THREE.Material | THREE.Material[];
      if (Array.isArray(mat)) mat.forEach((m) => m.dispose());
      else if (mat) {
        const tex = (mat as THREE.MeshBasicMaterial).map;
        if (tex) tex.dispose();
        mat.dispose();
      }
    });
    this.renderer.dispose();
  }
}
