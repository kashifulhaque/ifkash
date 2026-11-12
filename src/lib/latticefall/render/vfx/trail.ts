// Weapon trails: a fading ribbon between the weapon's base and tip, one per actor. Samples
// are taken from the animated skeleton while the actor swings, so the ribbon follows what
// is drawn, not the hitbox.

import { BufferAttribute, BufferGeometry, DoubleSide, Mesh, MeshBasicMaterial, NormalBlending, type Vector3 } from 'three';

/** Samples kept per ribbon; at 60 fps this is far more than `LIFE` needs. */
const MAX_SAMPLES = 24;
/** Seconds a sample lives. */
const LIFE = 0.3;
/** Below this tip speed in m/s the weapon is not swinging, so no sample is added. */
const MIN_TIP_SPEED = 3;

/** How a weapon class draws its trail. One entry per class in the spec's "Weapon trail" row. */
export interface TrailStyle {
  /** Linear 0-1 RGB. */
  color: [number, number, number];
  /** Peak alpha at the newest sample. */
  alpha: number;
}

/** Move `anim` prefix to trail style. Only straight-sword moves exist so far. */
const TRAIL_STYLES: ReadonlyArray<readonly [string, TrailStyle]> = [['sword_', { color: [0.5, 0.76, 0.92], alpha: 0.9 }]];

/** The trail style for a move `anim`, or null when that move leaves no trail (rolls, flasks). */
export function trailStyleForAnim(anim: string): TrailStyle | null {
  for (const [prefix, style] of TRAIL_STYLES) if (anim.startsWith(prefix)) return style;
  return null;
}

export class Trail {
  readonly mesh: Mesh;
  private geometry = new BufferGeometry();
  private pos = new Float32Array(MAX_SAMPLES * 2 * 3);
  private col = new Float32Array(MAX_SAMPLES * 2 * 4);
  /** Ring of samples, oldest first: tip xyz, base xyz, age. */
  private samples: Array<{ tip: [number, number, number]; base: [number, number, number]; age: number }> = [];
  private lastTip: [number, number, number] | null = null;
  private style: TrailStyle = { color: [1, 1, 1], alpha: 0.7 };

  constructor(material: MeshBasicMaterial) {
    this.geometry.setAttribute('position', new BufferAttribute(this.pos, 3).setUsage(35048));
    this.geometry.setAttribute('color', new BufferAttribute(this.col, 4).setUsage(35048));
    const index: number[] = [];
    for (let i = 0; i < MAX_SAMPLES - 1; i++) {
      const a = i * 2;
      index.push(a, a + 1, a + 2, a + 1, a + 3, a + 2);
    }
    this.geometry.setIndex(index);
    this.geometry.setDrawRange(0, 0);
    this.mesh = new Mesh(this.geometry, material);
    this.mesh.frustumCulled = false;
    this.mesh.renderOrder = 9;
  }

  get sampleCount(): number {
    return this.samples.length;
  }

  /**
   * Ages the ribbon by `dt` seconds and, when `emit` is given, adds a sample from the weapon's
   * current tip and base. A weapon that is barely moving adds none, so a held pose draws no
   * ribbon. `dt` of 0 (hitstop, frozen) adds nothing and ages nothing.
   */
  step(dt: number, emit: { tip: Vector3; base: Vector3; style: TrailStyle } | null): void {
    for (const s of this.samples) s.age += dt;
    while (this.samples.length && this.samples[0].age > LIFE) this.samples.shift();
    if (emit && dt > 0) {
      const t = emit.tip;
      const speed = this.lastTip ? Math.hypot(t.x - this.lastTip[0], t.y - this.lastTip[1], t.z - this.lastTip[2]) / dt : Infinity;
      if (speed >= MIN_TIP_SPEED || this.samples.length > 0) {
        this.style = emit.style;
        this.samples.push({ tip: [t.x, t.y, t.z], base: [emit.base.x, emit.base.y, emit.base.z], age: 0 });
        if (this.samples.length > MAX_SAMPLES) this.samples.shift();
      }
      this.lastTip = [t.x, t.y, t.z];
    } else if (!emit) {
      this.lastTip = null;
    }
    this.fill();
  }

  private fill(): void {
    const n = this.samples.length;
    const [r, g, b] = this.style.color;
    for (let i = 0; i < n; i++) {
      const s = this.samples[i];
      const life = 1 - Math.min(s.age / LIFE, 1);
      // The tip edge is brighter than the base edge, which reads as a swept blade.
      const a = life * this.style.alpha;
      const o = i * 2;
      this.pos.set(s.tip, o * 3);
      this.pos.set(s.base, (o + 1) * 3);
      this.col.set([r, g, b, a], o * 4);
      this.col.set([r, g, b, a * 0.4], (o + 1) * 4);
    }
    this.geometry.setDrawRange(0, n >= 2 ? (n - 1) * 6 : 0);
    (this.geometry.getAttribute('position') as BufferAttribute).needsUpdate = true;
    (this.geometry.getAttribute('color') as BufferAttribute).needsUpdate = true;
  }

  dispose(): void {
    this.geometry.dispose();
  }
}

export function makeTrailMaterial(): MeshBasicMaterial {
  return new MeshBasicMaterial({
    vertexColors: true,
    transparent: true,
    depthWrite: false,
    side: DoubleSide,
    blending: NormalBlending
  });
}
