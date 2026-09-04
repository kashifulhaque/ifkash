// Light sources on the surface. Under a permanent night the planet has far more
// glowing things — the campfire, the forge, the crater, lit windows, the
// lighthouse, every wonder marker — than a forward renderer can afford to light
// with at once. Each one registers an `Emitter` instead, and a fixed pool of
// point lights is lent to the handful nearest the player, fading in and out at
// the edge of their reach so nothing pops.

import * as THREE from 'three';

export type EmitterOptions = {
  /** Flicker depth: 0 for a steady lamp, 1 for an open flame. */
  flicker?: number;
  /** Flicker offset, so two fires never pulse together. */
  phase?: number;
  /** Start switched off, for example an unlit lighthouse. */
  on?: boolean;
};

export type Emitter = {
  /** World position. Moving emitters, such as the floating markers, rewrite it. */
  pos: THREE.Vector3;
  color: THREE.Color;
  /** Peak intensity, before the flicker and the distance fade. */
  intensity: number;
  /** Reach of the light, in world units. */
  distance: number;
  flicker: number;
  phase: number;
  on: boolean;
};

export function emitter(
  pos: THREE.Vector3,
  color: number,
  intensity: number,
  distance: number,
  o: EmitterOptions = {}
): Emitter {
  return {
    pos,
    color: new THREE.Color(color),
    intensity,
    distance,
    flicker: o.flicker ?? 0,
    phase: o.phase ?? 0,
    on: o.on ?? true
  };
}

/**
 * How far past its own reach a source is still worth a light. A point light
 * contributes nothing at the focus point once it is `distance` away, but it can
 * still light the scenery in between, so candidates are kept a little longer.
 */
const REACH = 1.6;
/** Fraction of that reach at which a source starts fading out of the pool. */
const FADE_FROM = 0.72;

type Candidate = { e: Emitter; key: number; fade: number };

export class LightPool {
  readonly group = new THREE.Group();
  readonly emitters: Emitter[] = [];
  private lights: THREE.PointLight[] = [];
  private near: Candidate[] = [];

  constructor(count: number) {
    for (let i = 0; i < count; i++) {
      const light = new THREE.PointLight(0xffffff, 0, 10, 2);
      // The pool never hides a light. Three.js keys its shader programs on the
      // number of visible lights, so toggling one would recompile every
      // material in the scene; an unused slot sits at zero intensity instead.
      this.group.add(light);
      this.lights.push(light);
    }
  }

  add(e: Emitter): Emitter {
    this.emitters.push(e);
    return e;
  }

  /** Hand the lights to the sources nearest `focus`, usually the player. */
  update(time: number, focus: THREE.Vector3): void {
    this.near.length = 0;
    for (const e of this.emitters) {
      if (!e.on || e.intensity <= 0) continue;
      const key = e.pos.distanceTo(focus) / (e.distance * REACH);
      if (key >= 1) continue;
      this.near.push({ e, key, fade: 1 - THREE.MathUtils.smoothstep(key, FADE_FROM, 1) });
    }
    this.near.sort((a, b) => a.key - b.key);

    for (let i = 0; i < this.lights.length; i++) {
      const light = this.lights[i];
      const c = this.near[i];
      if (!c) {
        light.intensity = 0;
        continue;
      }
      const e = c.e;
      // Two detuned waves so a flame never settles into an obvious rhythm.
      const flicker = e.flicker
        ? 1 + (Math.sin(time * 11 + e.phase) * 0.18 + Math.sin(time * 23.3 + e.phase * 1.7) * 0.1) * e.flicker
        : 1;
      light.position.copy(e.pos);
      light.color.copy(e.color);
      light.distance = e.distance;
      light.intensity = e.intensity * c.fade * flicker;
    }
  }
}
