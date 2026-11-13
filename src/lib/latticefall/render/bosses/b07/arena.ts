// B07's arena layer (WP-075): the chandeliers that fall in phase 2, and, with `&dev=1`, a small
// readout of the counts the core keeps of the player's actions. Render-only: it reads the
// mechanic events (WASM kind 14, sub-kinds 160-165; game/crates/core/src/events.rs) and never
// writes simulation state.
//
// A chandelier marker (sub-kind 163) names its slot and its center as half-meter offsets from
// Voss's spawn, plus 64 on each axis. The renderer doesn't know the spawn, so it takes it from
// where it last saw Voss's body before the gate sealed (he stands at his spawn until then). The marker is
// a ring on the floor that brightens over the 45 ticks of the telegraph, while the chandelier
// hangs shaking at the ceiling; in the last ticks it drops, and sub-kind 164 (it fell) bursts it
// into a flash. Sub-kind 165 (it hit the player) flashes the ring red.

import {
  AdditiveBlending,
  CylinderGeometry,
  DoubleSide,
  Group,
  Mesh,
  MeshBasicMaterial,
  MeshStandardMaterial,
  RingGeometry,
  SphereGeometry,
  TorusGeometry,
  type Scene
} from 'three';
import { EVENT_WORDS } from '../../../engine/events';
import { TICK_HZ } from '../../../engine/loop';
import type { DrawActor } from '../../../engine/snapshot';

/** Registry number of Voss's body (`b07_castellan_voss_body`). */
const BODY_KIND = 36011;
const EVENT_MECHANIC = 14;
const EVENT_BOSS = 11;
const BOSS_SEALED = 0;
const BOSS_RESET = 4;
const COUNTED = 160;
const OVERFIT = 161;
const BLOCKED = 162;
const MARKED = 163;
const FELL = 164;
const HIT = 165;

/** The telegraph, in ticks, and the circle's radius, in meters (`chandelier_*` in the boss data). */
const TELEGRAPH_TICKS = 45;
const RADIUS_M = 2.2;
/** How high the chandeliers hang above the floor, and how long the drop lasts, in ticks. */
const HANG_M = 9;
const DROP_TICKS = 10;
/** A burst's length in seconds. */
const BURST_S = 0.45;
const SLOTS = 2;
const CATEGORY_NAMES = ['roll left', 'roll right', 'roll back', 'parry', 'guard', 'ranged'];

interface Slot {
  group: Group;
  ring: Mesh;
  fill: Mesh;
  chandelier: Group;
  /** Seconds since the marker appeared, or -1 when free. */
  age: number;
  /** Seconds since it fell, or -1 while it hangs. */
  burst: number;
  /** Seconds since the player was hit, or -1. */
  hit: number;
}

export class VossArena {
  private slots: Slot[] = [];
  /** Voss's spawn: where his body stood until the gate sealed (see `update`). */
  private origin: [number, number, number] | null = null;
  private originLocked = false;
  private counts = [0, 0, 0, 0, 0, 0];
  private favored: number[] = [];
  private blocked = 0;
  private readout: HTMLDivElement | null = null;

  constructor(private scene: Scene, parent?: HTMLElement | null) {
    for (let i = 0; i < SLOTS; i++) this.slots.push(this.makeSlot());
    if (parent && typeof location !== 'undefined' && new URLSearchParams(location.search).get('dev') === '1') {
      const el = document.createElement('div');
      el.style.cssText =
        'position:absolute;left:8px;bottom:8px;padding:6px 8px;font:11px/1.35 ui-monospace,monospace;' +
        'color:#e8e0cc;background:rgba(16,14,22,0.72);border:1px solid rgba(201,185,154,0.4);' +
        'pointer-events:none;white-space:pre;display:none;z-index:5';
      parent.appendChild(el);
      this.readout = el;
      // Console access in dev mode, for checking the markers: `__lfVoss.events(words)`.
      (window as unknown as { __lfVoss?: VossArena }).__lfVoss = this;
    }
  }

  private makeSlot(): Slot {
    const group = new Group();
    group.visible = false;
    const ring = new Mesh(
      new RingGeometry(RADIUS_M - 0.14, RADIUS_M, 48),
      new MeshBasicMaterial({ color: 0xff9a3c, transparent: true, opacity: 0.8, side: DoubleSide, depthWrite: false })
    );
    ring.rotation.x = -Math.PI / 2;
    ring.position.y = 0.06;
    const fill = new Mesh(
      new RingGeometry(0.01, RADIUS_M - 0.14, 48),
      new MeshBasicMaterial({
        color: 0xff6a2a,
        transparent: true,
        opacity: 0.1,
        side: DoubleSide,
        depthWrite: false,
        blending: AdditiveBlending
      })
    );
    fill.rotation.x = -Math.PI / 2;
    fill.position.y = 0.055;
    const chandelier = new Group();
    const iron = new MeshStandardMaterial({ color: 0x3a3640, roughness: 0.7, metalness: 0.5 });
    const flame = new MeshBasicMaterial({ color: 0xffd27a });
    const hoop = new Mesh(new TorusGeometry(1.1, 0.09, 8, 28), iron);
    hoop.rotation.x = Math.PI / 2;
    chandelier.add(hoop);
    const inner = new Mesh(new TorusGeometry(0.55, 0.06, 8, 20), iron);
    inner.rotation.x = Math.PI / 2;
    inner.position.y = 0.25;
    chandelier.add(inner);
    for (let i = 0; i < 8; i++) {
      const a = (i / 8) * Math.PI * 2;
      const candle = new Mesh(new CylinderGeometry(0.05, 0.05, 0.28, 6), new MeshStandardMaterial({ color: 0xe8dfc8 }));
      candle.position.set(Math.cos(a) * 1.1, 0.2, Math.sin(a) * 1.1);
      const f = new Mesh(new SphereGeometry(0.07, 6, 5), flame);
      f.position.set(Math.cos(a) * 1.1, 0.4, Math.sin(a) * 1.1);
      chandelier.add(candle, f);
    }
    const chain = new Mesh(new CylinderGeometry(0.03, 0.03, 4, 5), iron);
    chain.position.y = 2.1;
    chandelier.add(chain);
    group.add(ring, fill, chandelier);
    this.scene.add(group);
    return { group, ring, fill, chandelier, age: -1, burst: -1, hit: -1 };
  }

  /** Scans one tick's event words for B07's mechanic events. */
  events(words: ArrayLike<number>): void {
    let i = 0;
    while (i < words.length) {
      const kind = words[i];
      const len = EVENT_WORDS[kind];
      if (len === undefined || i + len > words.length) break;
      if (kind === EVENT_MECHANIC) this.mechanic(words[i + 1] >>> 0, words[i + 2] >>> 0);
      // A fight starting (the gate sealed) or ending in the player's death clears the readout.
      else if (kind === EVENT_BOSS && words[i + 2] === BOSS_SEALED) {
        this.clear();
        this.originLocked = true;
      } else if (kind === EVENT_BOSS && words[i + 2] === BOSS_RESET) {
        this.clear();
        this.originLocked = false;
      }
      i += len;
    }
  }

  private clear(): void {
    this.counts = [0, 0, 0, 0, 0, 0];
    this.favored = [];
    this.blocked = 0;
    for (const s of this.slots) {
      s.age = -1;
      s.group.visible = false;
    }
  }

  private mechanic(sub: number, index: number): void {
    switch (sub) {
      case COUNTED:
        this.counts[(index >> 8) & 7] = index & 255;
        break;
      case OVERFIT:
        this.favored = [index >> 8, index & 255].filter((c) => c < 6);
        break;
      case BLOCKED:
        this.blocked++;
        break;
      case MARKED:
        this.mark(index);
        break;
      case FELL: {
        const s = this.slots[index >> 14];
        if (s && s.age >= 0) s.burst = 0;
        break;
      }
      case HIT: {
        const s = this.slots[index];
        if (s) s.hit = 0;
        break;
      }
    }
  }

  private mark(index: number): void {
    const s = this.slots[index >> 14];
    if (!s || !this.origin) return;
    const x = this.origin[0] + (((index >> 7) & 127) - 64) * 0.5;
    const z = this.origin[2] + ((index & 127) - 64) * 0.5;
    s.group.position.set(x, this.origin[1], z);
    s.group.visible = true;
    s.age = 0;
    s.burst = -1;
    s.hit = -1;
    s.chandelier.visible = true;
    s.chandelier.scale.setScalar(1);
  }

  /** Per frame: `dtMs` of frame time. Pass the drawn actors so Voss's spawn is found. */
  update(dtMs: number, actors: DrawActor[]): void {
    // Until the gate seals, Voss stands at his spawn; after a reset he is rebuilt there.
    if (!this.originLocked) {
      const body = actors.find((a) => a.kind === BODY_KIND);
      if (body) this.origin = [body.x, body.y, body.z];
    }
    const dt = dtMs / 1000;
    for (const s of this.slots) {
      if (s.age < 0) continue;
      s.age += dt;
      const t = s.age * TICK_HZ;
      const frac = Math.min(t / TELEGRAPH_TICKS, 1);
      const ringMat = s.ring.material as MeshBasicMaterial;
      const fillMat = s.fill.material as MeshBasicMaterial;
      if (s.burst < 0) {
        // Telegraph: the ring pulses faster as it nears, the fill brightens, and the chandelier
        // shakes at the ceiling, then drops in the last DROP_TICKS.
        const pulse = 0.55 + 0.45 * Math.sin(t * (0.5 + frac * 1.2));
        ringMat.opacity = 0.35 + 0.65 * pulse * (0.4 + frac * 0.6);
        fillMat.opacity = 0.06 + 0.4 * frac * frac;
        const drop = Math.max(0, (t - (TELEGRAPH_TICKS - DROP_TICKS)) / DROP_TICKS);
        s.chandelier.position.y = HANG_M * (1 - drop * drop) + 0.5;
        s.chandelier.rotation.z = (1 - drop) * Math.sin(t * 2.1) * 0.05 * (0.3 + frac);
        s.chandelier.position.x = (1 - drop) * Math.sin(t * 3.3) * 0.03 * frac;
      } else {
        // Landed: the chandelier is gone in a flash, and the ring blows outward and fades.
        s.burst += dt;
        const u = Math.min(s.burst / BURST_S, 1);
        s.chandelier.visible = u < 0.15;
        s.chandelier.position.y = 0.5;
        const scale = 1 + u * 0.5;
        s.ring.scale.set(scale, scale, 1);
        ringMat.opacity = (1 - u) * 0.9;
        fillMat.opacity = (1 - u) * 0.5;
        if (s.hit >= 0) ringMat.color.setHex(0xff3030);
        if (u >= 1) {
          s.age = -1;
          s.group.visible = false;
          s.ring.scale.set(1, 1, 1);
          ringMat.color.setHex(0xff9a3c);
        }
      }
      if (s.hit >= 0) s.hit += dt;
    }
    this.draw();
  }

  /** The dev readout: the counts, the two most-counted (the ones with a counter), and blocks. */
  private draw(): void {
    if (!this.readout) return;
    const any = this.counts.some((c) => c > 0) || this.favored.length > 0;
    this.readout.style.display = any ? 'block' : 'none';
    if (!any) return;
    const lines = ['Voss: counted actions'];
    this.counts.forEach((c, i) => {
      const mark = this.favored.includes(i) ? ' <- counter' : '';
      lines.push(`  ${CATEGORY_NAMES[i].padEnd(10)} ${String(c).padStart(3)}${mark}`);
    });
    if (this.blocked) lines.push(`  guard blocks ${this.blocked}`);
    this.readout.textContent = lines.join('\n');
  }

  dispose(): void {
    for (const s of this.slots) {
      this.scene.remove(s.group);
      s.group.traverse((o) => {
        const m = o as Mesh;
        if (m.geometry) m.geometry.dispose();
        if (m.material) (m.material as MeshBasicMaterial).dispose();
      });
    }
    this.slots = [];
    this.readout?.remove();
    if (typeof window !== 'undefined') delete (window as unknown as { __lfVoss?: VossArena }).__lfVoss;
  }
}
