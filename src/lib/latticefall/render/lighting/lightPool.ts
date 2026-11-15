// A small pool of point lights for an area's fixed light spots (the art manifest's `lights`):
// torches, candles, braziers. Every lit material pays for every point light in the scene, so
// the renderer keeps a fixed handful of lights (the quality preset's `lights`) and moves them
// to the spots nearest the player each frame. A spot that joins the pool fades in, so the
// swap reads as a flame catching rather than a pop. No light casts a shadow.
//
// Read-only: nothing here writes simulation state.

import { Color, PointLight, type Scene } from 'three';
import type { ArtLight } from '../kits/art';

/** The renderer's usual falloff for flames (the Lantern uses the same). */
const DECAY = 1.6;
/** Milliseconds for a light that joins the pool to reach full strength. */
const FADE_MS = 350;

interface Slot {
  light: PointLight;
  /** Index into the spots, or -1 when free. */
  spot: number;
  fade: number;
}

export class LightPool {
  private slots: Slot[] = [];
  private colors: Color[];
  private phases: number[];
  private clock = 0;
  private order: number[] = [];

  constructor(
    private scene: Scene,
    private spots: readonly ArtLight[],
    size: number
  ) {
    this.colors = spots.map((s) => new Color(s.color));
    this.phases = spots.map((s, i) => s.pos[0] * 1.7 + s.pos[2] * 0.9 + i * 2.3);
    this.setSize(size);
  }

  /** How many point lights the pool holds. Changing it recompiles the lit materials once. */
  get size(): number {
    return this.slots.length;
  }

  setSize(n: number): void {
    const size = Math.max(0, Math.min(n, this.spots.length));
    if (size === this.slots.length) return;
    for (const s of this.slots) this.scene.remove(s.light);
    this.slots = [];
    for (let i = 0; i < size; i++) {
      const light = new PointLight(0xffffff, 0, 1, DECAY);
      light.castShadow = false;
      this.scene.add(light);
      this.slots.push({ light, spot: -1, fade: 0 });
    }
  }

  /** Moves the pool to the spots nearest `(fx, fy, fz)` and animates the flicker. */
  update(fx: number, fy: number, fz: number, dtMs: number): void {
    if (this.slots.length === 0) return;
    this.clock += dtMs;
    // Rank spots by how far the player is beyond the near half of their reach.
    const score = (i: number): number => {
      const s = this.spots[i];
      return Math.hypot(s.pos[0] - fx, s.pos[1] - fy, s.pos[2] - fz) - s.distance * 0.5;
    };
    if (this.order.length !== this.spots.length) this.order = this.spots.map((_, i) => i);
    this.order.sort((a, b) => score(a) - score(b));
    const wanted = new Set(this.order.slice(0, this.slots.length));
    for (const slot of this.slots) {
      if (slot.spot >= 0 && !wanted.has(slot.spot)) slot.spot = -1;
      else if (slot.spot >= 0) wanted.delete(slot.spot);
    }
    for (const slot of this.slots) {
      if (slot.spot >= 0) continue;
      const next = wanted.values().next();
      if (next.done) break;
      wanted.delete(next.value);
      slot.spot = next.value;
      slot.fade = 0;
    }
    const t = this.clock / 1000;
    for (const slot of this.slots) {
      const light = slot.light;
      if (slot.spot < 0) {
        light.intensity = 0;
        continue;
      }
      const s = this.spots[slot.spot];
      slot.fade = Math.min(1, slot.fade + dtMs / FADE_MS);
      const p = this.phases[slot.spot];
      const f = s.flicker ?? 0;
      const wave = 0.5 * Math.sin(t * 9.1 + p) + 0.3 * Math.sin(t * 15.7 + p * 1.9) + 0.2 * Math.sin(t * 3.3 + p * 0.7);
      light.position.set(s.pos[0], s.pos[1] + f * 0.03 * Math.sin(t * 6.1 + p), s.pos[2]);
      light.color.copy(this.colors[slot.spot]);
      light.distance = s.distance;
      light.intensity = s.intensity * slot.fade * (1 + f * 0.35 * wave);
    }
  }

  dispose(): void {
    for (const s of this.slots) this.scene.remove(s.light);
    this.slots = [];
  }
}
