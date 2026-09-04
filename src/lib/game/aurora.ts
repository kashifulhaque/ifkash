// Auroras. Two of them: a close curtain that stands over the arctic once the
// education wonder is found, and a wide band that wraps the whole night sky.
// Both are additive ribbons whose vertex colours run through a green, cyan, and
// violet ramp that drifts slowly, so the light never settles on one hue.

import * as THREE from 'three';
import { seededRng } from './noise';
import { tangentBasis } from './planet';

/** Vertices across a ribbon's length and height. */
const W = 48;
const H = 6;
/** Hue at the foot and the crown of a ribbon; the span between them is the colour ramp. */
const HUE_FOOT = 0.36;
const HUE_CROWN = 0.82;
/** How far the whole ramp drifts, in hue units, over a full drift cycle. */
const HUE_DRIFT = 0.09;
/** Opacity the arctic curtain settles at once it is woken. */
const FULL = 0.52;
/** Opacity the sky-wide bands settle at. Deliberately faint: three of them overlap. */
const SKY_FULL = 0.23;

const _c = new THREE.Color();

/** Ribbon colour at height `t` (0 at the foot, 1 at the crown) at time `time`. */
function rampColor(t: number, time: number, offset: number, bright: number): THREE.Color {
  const hue = (HUE_FOOT + (HUE_CROWN - HUE_FOOT) * t + Math.sin(time * 0.07 + offset) * HUE_DRIFT + 1) % 1;
  // Kept dim on purpose: several additive ribbons overlap, and bright ones stack up to white.
  return _c.setHSL(hue, 0.9, (0.2 + t * 0.16) * bright);
}

/**
 * Curtains of light above the arctic. Hidden until the aurora is woken, then
 * fades in, ripples, and cycles through the colour ramp.
 */
export class Aurora {
  group = new THREE.Group();
  private ribbons: { mesh: THREE.Mesh; base: Float32Array; phase: number }[] = [];
  private opacity = 0;
  private active = false;

  constructor(center: THREE.Vector3, radius: number) {
    const right = new THREE.Vector3();
    const fwd = new THREE.Vector3();
    tangentBasis(center, 0, right, fwd);
    this.group.position.copy(center).multiplyScalar(radius + 9);
    const m = new THREE.Matrix4().makeBasis(right, center, fwd);
    this.group.quaternion.setFromRotationMatrix(m);

    for (let i = 0; i < 3; i++) {
      const geo = new THREE.PlaneGeometry(26 + i * 4, 6 + i, W - 1, H - 1);
      geo.setAttribute('color', new THREE.BufferAttribute(new Float32Array(geo.attributes.position.count * 3), 3));
      const mat = new THREE.MeshBasicMaterial({
        vertexColors: true,
        transparent: true,
        opacity: 0,
        side: THREE.DoubleSide,
        depthWrite: false,
        toneMapped: false,
        blending: THREE.AdditiveBlending
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.set(0, 6 + i * 1.6, -6 + i * 5);
      mesh.rotation.y = (i - 1) * 0.42;
      this.group.add(mesh);
      this.ribbons.push({ mesh, base: (geo.attributes.position.array as Float32Array).slice(), phase: i * 1.7 });
    }
    this.group.visible = false;
  }

  setActive(on: boolean): void {
    this.active = on;
    if (on) this.group.visible = true;
  }

  update(dt: number, time: number): void {
    const target = this.active ? FULL : 0;
    this.opacity += (target - this.opacity) * Math.min(1, dt * 1.2);
    if (this.opacity < 0.005 && !this.active) {
      this.group.visible = false;
      return;
    }
    for (let r = 0; r < this.ribbons.length; r++) {
      const ribbon = this.ribbons[r];
      (ribbon.mesh.material as THREE.MeshBasicMaterial).opacity = this.opacity;
      const pos = ribbon.mesh.geometry.attributes.position as THREE.BufferAttribute;
      const col = ribbon.mesh.geometry.attributes.color as THREE.BufferAttribute;
      // Colours only change per row, so sample the ramp once per row and reuse it.
      for (let row = 0; row < H; row++) {
        const t = 1 - row / (H - 1);
        const c = rampColor(t, time, r * 1.3, 0.55 + t * 0.5);
        for (let ix = 0; ix < W; ix++) col.setXYZ(row * W + ix, c.r, c.g, c.b);
      }
      for (let v = 0; v < pos.count; v++) {
        const x = ribbon.base[v * 3];
        const y = ribbon.base[v * 3 + 1];
        const wave = Math.sin(x * 0.35 + time * 0.9 + ribbon.phase) * 0.8 + Math.sin(x * 0.9 - time * 0.6) * 0.3;
        pos.setXYZ(v, x, y + wave * ((y + 3) / 6), wave * 0.6);
      }
      pos.needsUpdate = true;
      col.needsUpdate = true;
    }
  }
}

// ---------------------------------------------------------------- sky band

/** Vertices along and across a sky band. */
const SW = 56;
const SH = 7;
/** Radius of the shell the bands hang on: outside the clouds, inside the stars. */
const SKY_RADIUS = 300;

/**
 * Rings of light wrapped right around the night sky, so one always arcs
 * overhead wherever the player is standing. Each ring is a band of a sphere
 * around its own tilted axis; brightness swells and ebbs along its length in
 * whole cycles, which keeps the two ends of the loop seamless.
 */
export class SkyAurora {
  group = new THREE.Group();
  private bands: {
    mesh: THREE.Mesh;
    axis: THREE.Vector3;
    e1: THREE.Vector3;
    e2: THREE.Vector3;
    theta0: number;
    phi0: number;
    phiSpan: number;
    drift: number;
    phase: number;
  }[] = [];
  private opacity = 0;
  private hidden = false;

  constructor(seed: number) {
    const rng = seededRng(seed);
    const e1 = new THREE.Vector3();
    const e2 = new THREE.Vector3();
    for (let i = 0; i < 3; i++) {
      // A tilted axis per band, so the three arcs cross the sky at their own angles.
      const axis = new THREE.Vector3(rng() * 2 - 1, rng() * 2 - 1, rng() * 2 - 1).normalize();
      tangentBasis(axis, rng() * Math.PI * 2, e1, e2);
      const geo = new THREE.BufferGeometry();
      geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(SW * SH * 3), 3));
      geo.setAttribute('color', new THREE.BufferAttribute(new Float32Array(SW * SH * 3), 3));
      geo.setIndex(gridIndices(SW, SH));
      const mat = new THREE.MeshBasicMaterial({
        vertexColors: true,
        transparent: true,
        opacity: 0,
        side: THREE.DoubleSide,
        depthWrite: false,
        toneMapped: false,
        blending: THREE.AdditiveBlending
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.frustumCulled = false;
      mesh.renderOrder = -1;
      this.group.add(mesh);
      this.bands.push({
        mesh,
        axis,
        e1: e1.clone(),
        e2: e2.clone(),
        theta0: rng() * Math.PI * 2,
        phi0: 0.55 + rng() * 0.55,
        phiSpan: 0.16 + rng() * 0.12,
        drift: (rng() < 0.5 ? -1 : 1) * (0.006 + rng() * 0.008),
        phase: rng() * 6.28
      });
    }
    this.group.visible = false;
  }

  /** Hide the bands, for example in globe view, where they would swamp the planet. */
  setHidden(hidden: boolean): void {
    this.hidden = hidden;
  }

  update(dt: number, time: number): void {
    const target = this.hidden ? 0 : SKY_FULL;
    this.opacity += (target - this.opacity) * Math.min(1, dt * 1.5);
    const on = this.opacity > 0.004;
    this.group.visible = on;
    if (!on) return;

    for (const b of this.bands) {
      (b.mesh.material as THREE.MeshBasicMaterial).opacity = this.opacity;
      const pos = b.mesh.geometry.attributes.position as THREE.BufferAttribute;
      const col = b.mesh.geometry.attributes.color as THREE.BufferAttribute;
      const start = b.theta0 + time * b.drift;
      for (let row = 0; row < SH; row++) {
        const v = row / (SH - 1);
        // Fade the crown out so the band dissolves into the sky rather than ending flat.
        const c = rampColor(v, time, b.phase, 0.5 + 0.5 * Math.sin(v * Math.PI));
        for (let ix = 0; ix < SW; ix++) {
          const u = ix / (SW - 1);
          const theta = start + u * Math.PI * 2;
          // Whole numbers of cycles only, so the ring closes without a seam.
          const wave = Math.sin(theta * 3 + time * 0.35 + b.phase) * 0.6 + Math.sin(theta * 7 - time * 0.22) * 0.22;
          const phi = b.phi0 + v * b.phiSpan + wave * 0.045 * (0.4 + v);
          const s = Math.sin(phi) * SKY_RADIUS;
          const y = Math.cos(phi) * SKY_RADIUS;
          const ct = Math.cos(theta);
          const st = Math.sin(theta);
          pos.setXYZ(
            row * SW + ix,
            b.e1.x * ct * s + b.e2.x * st * s + b.axis.x * y,
            b.e1.y * ct * s + b.e2.y * st * s + b.axis.y * y,
            b.e1.z * ct * s + b.e2.z * st * s + b.axis.z * y
          );
          // Bright stretches and faint ones along the ring, rather than one even glow.
          const swell = 0.5 + 0.5 * Math.sin(theta + b.phase + time * 0.03);
          const ripple = 0.5 + 0.5 * Math.sin(theta * 3 - b.phase * 1.7);
          const fade = 0.12 + 0.88 * (swell * 0.65 + ripple * 0.35);
          col.setXYZ(row * SW + ix, c.r * fade, c.g * fade, c.b * fade);
        }
      }
      pos.needsUpdate = true;
      col.needsUpdate = true;
    }
  }
}

/** Triangle indices for a `w` by `h` vertex grid. */
function gridIndices(w: number, h: number): THREE.BufferAttribute {
  const idx = new Uint16Array((w - 1) * (h - 1) * 6);
  let k = 0;
  for (let row = 0; row < h - 1; row++) {
    for (let ix = 0; ix < w - 1; ix++) {
      const a = row * w + ix;
      idx[k++] = a;
      idx[k++] = a + w;
      idx[k++] = a + 1;
      idx[k++] = a + 1;
      idx[k++] = a + w;
      idx[k++] = a + w + 1;
    }
  }
  return new THREE.BufferAttribute(idx, 1);
}
