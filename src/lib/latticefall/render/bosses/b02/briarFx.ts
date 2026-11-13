// B02's ground effects (render-only; docs/game/05-enemies-and-bosses.md, "B02 The Briar Stag"):
//
//   - Bramble patches: the core's `Briar` events (WASM kind 40) say where they grow, how far they
//     slow the player, and when they wither. Each draws as a dark thorny disc that grows in over
//     half a second, pulses in its last three seconds, and shrinks away.
//   - The charge line: a streak on the ground along the Stag's facing while it scrapes, so the
//     straight line of the charge is a readable ground marker (docs "Fairness rules": every
//     attack has a telegraph; this is one for the charge).
//   - The leap's shadow marker: where the stomp will land, drawn from the leap's first frames.
//   - The thorn burst's ring: a faint ring at the aura's radius while the thorns bristle, then a
//     bright one that expands and fades on the burst (the `rupture_aura` vfx).
//   - Camera shake: a small rumble when a charge starts, a heavy one when it strikes a tree.
//
// The numbers below mirror game/data/moves/bosses/b02.ron (the charge's run frames and
// distances, the leap's root motion and hitbox); the core owns the real ones.

import {
  CircleGeometry,
  ConeGeometry,
  DoubleSide,
  Group,
  Mesh,
  MeshBasicMaterial,
  PlaneGeometry,
  RingGeometry,
  type Scene
} from 'three';
import {
  BRIAR_CLEARED,
  BRIAR_FADED,
  BRIAR_SPAWNED,
  MECHANIC_BRIAR_CHARGE_STARTED,
  MECHANIC_BRIAR_TREE_STRUCK,
  type GameEvent
} from '../../../engine/events';
import type { DrawActor } from '../../../engine/snapshot';
import type { MoveResolver } from '../../actors/moveClips';
import { B02_KIND } from './briarStag';

const TICK_SECONDS = 1 / 30;
/** Seconds a patch takes to grow in, and to shrink away. */
const GROW_SECONDS = 0.5;
const SHRINK_SECONDS = 0.6;
/** Seconds before it withers that a patch starts to pulse. */
const PULSE_SECONDS = 3;
/** Thorns on a patch. */
const PATCH_THORNS = 9;

/** Charge anims: the frame the run starts (also the end of the scrape) and its length in meters. */
const CHARGES: Record<string, { runFrom: number; length: number }> = {
  stag_charge: { runFrom: 30, length: 17 },
  stag_charge_2: { runFrom: 26, length: 14 },
  stag_charge_again: { runFrom: 26, length: 14 }
};
/** The leap: root motion `(frame, meters)` keys and the hitbox (a sphere at 0.5 m ahead of the landing, 2.6 m). */
const LEAP_FROM = 10;
const LEAP_TO = 30;
const LEAP_METERS = 6.5;
const LEAP_HIT_AHEAD = 0.5;
const LEAP_RADIUS = 2.6;
const LEAP_END = 36;
/** Thorn bursts: the aura's radius in meters by anim, and the frame it bursts on. */
const BURSTS: Record<string, number> = { stag_thorn_burst: 3.4, stag_thorn_burst_2: 4.0 };
const BURST_FRAME = 36;
const BURST_END = 48;

/** What the effects need to shake the camera. */
export interface ShakeHook {
  add(amount: number): void;
}

interface PatchView {
  group: Group;
  /** Seconds since it grew. */
  age: number;
  /** Seconds it lasts (0 while unknown). */
  life: number;
  /** Seconds since it started to wither; null while it lives. */
  fade: number | null;
}

const smooth = (x: number) => {
  const t = Math.min(Math.max(x, 0), 1);
  return t * t * (3 - 2 * t);
};

export class BriarFx {
  readonly group = new Group();
  private patches = new Map<number, PatchView>();
  private actors: DrawActor[] = [];
  private clock = 0;

  private discGeom = new CircleGeometry(1, 28);
  private ringGeom = new RingGeometry(0.86, 1, 36);
  private thornGeom = new ConeGeometry(0.07, 0.6, 6);
  private lineGeom = new PlaneGeometry(1, 1);
  private patchMat = new MeshBasicMaterial({ color: 0x1c2a17, transparent: true, opacity: 0.78, depthWrite: false, side: DoubleSide });
  private patchEdgeMat = new MeshBasicMaterial({ color: 0x4f7a3a, transparent: true, opacity: 0.8, depthWrite: false, side: DoubleSide });
  private thornMat = new MeshBasicMaterial({ color: 0x8a3a22 });
  private lineMat = new MeshBasicMaterial({ color: 0xb85a2e, transparent: true, opacity: 0.4, depthWrite: false, side: DoubleSide });
  private markerMat = new MeshBasicMaterial({ color: 0x15100c, transparent: true, opacity: 0.5, depthWrite: false, side: DoubleSide });
  private markerEdgeMat = new MeshBasicMaterial({ color: 0xd9531e, transparent: true, opacity: 0.8, depthWrite: false, side: DoubleSide });
  private auraMat = new MeshBasicMaterial({ color: 0xe0562a, transparent: true, opacity: 0.4, depthWrite: false, side: DoubleSide });
  private aura: Mesh;
  private line: Mesh;
  private marker: Group;
  private markerDisc: Mesh;
  private markerRing: Mesh;

  constructor(
    scene: Scene,
    private resolveMove: MoveResolver,
    private shake?: ShakeHook
  ) {
    this.line = new Mesh(this.lineGeom, this.lineMat);
    this.line.rotation.x = -Math.PI / 2;
    this.line.visible = false;
    this.line.renderOrder = 2;
    this.marker = new Group();
    this.markerDisc = new Mesh(this.discGeom, this.markerMat);
    this.markerRing = new Mesh(this.ringGeom, this.markerEdgeMat);
    for (const m of [this.markerDisc, this.markerRing]) {
      m.rotation.x = -Math.PI / 2;
      m.renderOrder = 2;
      this.marker.add(m);
    }
    this.marker.visible = false;
    this.aura = new Mesh(this.ringGeom, this.auraMat);
    this.aura.rotation.x = -Math.PI / 2;
    this.aura.visible = false;
    this.aura.renderOrder = 2;
    this.group.add(this.line, this.marker, this.aura);
    scene.add(this.group);
  }

  /** Reacts to this tick's events: patches grow and wither, the camera rumbles. */
  handleEvents(events: GameEvent[]): void {
    for (const e of events) {
      if (e.kind === 'briar') {
        if (e.what === BRIAR_SPAWNED) this.grow(e.patch, e.pos, e.radius, e.ticks * TICK_SECONDS);
        else if (e.what === BRIAR_FADED) this.wither(e.patch);
        else if (e.what === BRIAR_CLEARED) for (const id of [...this.patches.keys()]) this.wither(id);
      } else if (e.kind === 'mechanic') {
        if (e.what === MECHANIC_BRIAR_CHARGE_STARTED) this.shake?.add(0.18);
        else if (e.what === MECHANIC_BRIAR_TREE_STRUCK) this.shake?.add(0.7);
      }
    }
  }

  /** The actors this frame, for the charge line and the shadow marker. */
  setActors(actors: DrawActor[]): void {
    this.actors = actors;
  }

  private grow(id: number, pos: [number, number, number], radius: number, life: number): void {
    this.patches.get(id)?.group.removeFromParent();
    const g = new Group();
    g.position.set(pos[0], pos[1] + 0.05, pos[2]);
    const disc = new Mesh(this.discGeom, this.patchMat);
    disc.rotation.x = -Math.PI / 2;
    disc.scale.set(radius, radius, 1);
    const edge = new Mesh(this.ringGeom, this.patchEdgeMat);
    edge.rotation.x = -Math.PI / 2;
    edge.position.y = 0.01;
    edge.scale.set(radius, radius, 1);
    g.add(disc, edge);
    // Thorns standing in a ring and a few in the middle, placed from the id so a patch always
    // looks the same.
    for (let i = 0; i < PATCH_THORNS; i++) {
      const a = (i / PATCH_THORNS) * Math.PI * 2 + (id % 7) * 0.3;
      const r = radius * (i < 6 ? 0.7 + 0.2 * (((id >> i) & 1) - 0.5) : 0.25);
      const t = new Mesh(this.thornGeom, this.thornMat);
      const h = 0.7 + 0.5 * (((id * 31 + i * 17) % 10) / 10);
      t.scale.set(1, h, 1);
      t.position.set(Math.sin(a) * r, 0.3 * h, Math.cos(a) * r);
      t.rotation.set(Math.cos(a) * 0.35, 0, -Math.sin(a) * 0.35);
      g.add(t);
    }
    g.scale.setScalar(0.001);
    this.group.add(g);
    this.patches.set(id, { group: g, age: 0, life, fade: null });
  }

  private wither(id: number): void {
    const p = this.patches.get(id);
    if (p && p.fade === null) p.fade = 0;
  }

  /** Advances the patches and draws the markers. Once per frame, before the scene renders. */
  update(dt: number): void {
    this.clock += dt;
    for (const [id, p] of this.patches) {
      p.age += dt;
      let s = smooth(p.age / GROW_SECONDS);
      if (p.fade !== null) {
        p.fade += dt;
        s *= 1 - smooth(p.fade / SHRINK_SECONDS);
        if (p.fade >= SHRINK_SECONDS) {
          p.group.removeFromParent();
          this.patches.delete(id);
          continue;
        }
      } else if (p.life > 0 && p.life - p.age < PULSE_SECONDS) {
        s *= 1 + 0.05 * Math.sin(this.clock * 10);
      }
      p.group.scale.setScalar(Math.max(s, 0.001));
    }
    this.drawLine();
    this.drawMarker();
    this.drawAura();
  }

  /** The stag actors that are in a move: `[actor, anim, frame, total]`. */
  private *stagMoves(): Generator<[DrawActor, string, number]> {
    for (const a of this.actors) {
      if (a.kind !== B02_KIND || a.moveId === 0) continue;
      const info = this.resolveMove(a.moveId);
      if (info) yield [a, info.anim, a.moveFrame];
    }
  }

  private drawLine(): void {
    this.line.visible = false;
    for (const [a, anim, frame] of this.stagMoves()) {
      const c = CHARGES[anim];
      if (!c || frame < 4 || frame > c.runFrom + 2) continue;
      // From the Stag along its facing: yaw 0 faces +Z. The plane is laid flat (x rotation) after
      // turning about its own normal by the yaw, so its length runs along (sin yaw, cos yaw).
      const len = c.length;
      this.line.visible = true;
      this.line.position.set(a.x + Math.sin(a.yaw) * (len / 2 + 0.8), a.y + 0.06, a.z + Math.cos(a.yaw) * (len / 2 + 0.8));
      this.line.rotation.set(-Math.PI / 2, 0, a.yaw);
      this.line.scale.set(1.8, len, 1);
      // Brightens through the scrapes, and is full by the time it runs.
      this.lineMat.opacity = 0.18 + 0.4 * smooth(frame / c.runFrom);
      return;
    }
  }

  private drawMarker(): void {
    this.marker.visible = false;
    for (const [a, anim, frame] of this.stagMoves()) {
      if (anim !== 'stag_leap' || frame < 2 || frame > LEAP_END) continue;
      // Where the Stag will be at the landing: the rest of its root motion ahead of it, plus the
      // hitbox's own offset.
      const done = frame <= LEAP_FROM ? 0 : (LEAP_METERS * (Math.min(frame, LEAP_TO) - LEAP_FROM)) / (LEAP_TO - LEAP_FROM);
      const ahead = LEAP_METERS - done + LEAP_HIT_AHEAD;
      this.marker.visible = true;
      this.marker.position.set(a.x + Math.sin(a.yaw) * ahead, a.y + 0.06, a.z + Math.cos(a.yaw) * ahead);
      this.marker.scale.set(LEAP_RADIUS, 1, LEAP_RADIUS);
      // It darkens as the Stag comes down, and flashes on the landing.
      const u = smooth(frame / LEAP_TO);
      this.markerMat.opacity = 0.15 + 0.5 * u;
      this.markerEdgeMat.opacity = frame >= LEAP_TO ? 0.9 : 0.35 + 0.45 * u;
      return;
    }
  }

  private drawAura(): void {
    this.aura.visible = false;
    for (const [a, anim, frame] of this.stagMoves()) {
      const radius = BURSTS[anim];
      if (radius === undefined || frame < 8 || frame > BURST_END) continue;
      this.aura.visible = true;
      this.aura.position.set(a.x, a.y + 0.07, a.z);
      if (frame < BURST_FRAME) {
        // The warning: the whole radius, filling in as the thorns stand up.
        this.aura.scale.set(radius, radius, 1);
        this.auraMat.opacity = 0.08 + 0.4 * smooth((frame - 8) / (BURST_FRAME - 8));
      } else {
        // The burst: a bright ring that races out to the radius and fades.
        const u = (frame - BURST_FRAME) / (BURST_END - BURST_FRAME);
        const r = radius * (0.4 + 0.6 * smooth(u * 3));
        this.aura.scale.set(r, r, 1);
        this.auraMat.opacity = 0.95 * (1 - smooth(u));
      }
      return;
    }
  }

  dispose(): void {
    this.group.removeFromParent();
    for (const g of [this.discGeom, this.ringGeom, this.thornGeom, this.lineGeom]) g.dispose();
    for (const m of [this.patchMat, this.patchEdgeMat, this.thornMat, this.lineMat, this.markerMat, this.markerEdgeMat, this.auraMat]) m.dispose();
    this.patches.clear();
  }
}
