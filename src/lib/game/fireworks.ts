// Fireworks for the big moments: the seventh wonder, the last shard. A show is
// a string of bursts launched above the explorer over a few seconds. Every
// spark lives in one `Points` pool, so a full sky costs a single draw call.

import * as THREE from 'three';
import { tangentBasis } from './planet';
import { softDot } from './weather';

const SPARKS = 900;
const PER_BURST = 70;
/** Seconds a spark lasts. */
const LIFE = 1.7;
/** Pull toward the ground, in world units per second squared. */
const GRAVITY = 3.2;
/** Seconds between bursts during a show. */
const CADENCE = 0.42;

const PALETTE = [0xffd166, 0xff7b9c, 0x9fe8ff, 0xc9a4ff, 0xa8f0c6, 0xfff3c4];

const _right = new THREE.Vector3();
const _fwd = new THREE.Vector3();
const _c = new THREE.Color();

export class Fireworks {
  readonly group = new THREE.Group();
  private points: THREE.Points;
  private positions: Float32Array;
  private colors: Float32Array;
  private velocity = new Float32Array(SPARKS * 3);
  private life = new Float32Array(SPARKS);
  private tint: THREE.Color[] = [];
  private next = 0;
  /** Seconds of show left, and the countdown to the next burst. */
  private show = 0;
  private cooldown = 0;
  private up = new THREE.Vector3(0, 1, 0);
  private origin = new THREE.Vector3();
  /** Called on each launch so the Game can play a pop. */
  onBurst: (() => void) | null = null;

  constructor() {
    this.positions = new Float32Array(SPARKS * 3);
    this.colors = new Float32Array(SPARKS * 3);
    for (let i = 0; i < SPARKS; i++) this.tint.push(new THREE.Color());
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
    geo.setAttribute('color', new THREE.BufferAttribute(this.colors, 3));
    this.points = new THREE.Points(
      geo,
      new THREE.PointsMaterial({
        size: 0.42,
        map: softDot(),
        vertexColors: true,
        transparent: true,
        depthWrite: false,
        blending: THREE.AdditiveBlending,
        sizeAttenuation: true,
        toneMapped: false
      })
    );
    this.points.frustumCulled = false;
    this.group.add(this.points);
    this.group.visible = false;
  }

  /** Start a show of `seconds` above `origin`, which should be a point on the surface. */
  celebrate(origin: THREE.Vector3, seconds = 7): void {
    this.origin.copy(origin);
    this.up.copy(origin).normalize();
    this.show = seconds;
    this.cooldown = 0;
  }

  get active(): boolean {
    return this.show > 0 || this.group.visible;
  }

  private burst(): void {
    tangentBasis(this.up, Math.random() * Math.PI * 2, _right, _fwd);
    // Somewhere over the explorer's head, off to one side.
    const cx = this.origin.x + this.up.x * (7 + Math.random() * 4) + _right.x * (Math.random() - 0.5) * 9 + _fwd.x * (Math.random() - 0.5) * 9;
    const cy = this.origin.y + this.up.y * (7 + Math.random() * 4) + _right.y * (Math.random() - 0.5) * 9 + _fwd.y * (Math.random() - 0.5) * 9;
    const cz = this.origin.z + this.up.z * (7 + Math.random() * 4) + _right.z * (Math.random() - 0.5) * 9 + _fwd.z * (Math.random() - 0.5) * 9;
    _c.setHex(PALETTE[Math.floor(Math.random() * PALETTE.length)]);
    const speed = 3.2 + Math.random() * 1.8;
    for (let n = 0; n < PER_BURST; n++) {
      const i = this.next;
      this.next = (this.next + 1) % SPARKS;
      // Even spread over a sphere, with a little jitter in the speed.
      const z = Math.random() * 2 - 1;
      const phi = Math.random() * Math.PI * 2;
      const r = Math.sqrt(1 - z * z);
      const s = speed * (0.7 + Math.random() * 0.3);
      this.velocity[i * 3] = r * Math.cos(phi) * s;
      this.velocity[i * 3 + 1] = r * Math.sin(phi) * s;
      this.velocity[i * 3 + 2] = z * s;
      this.positions[i * 3] = cx;
      this.positions[i * 3 + 1] = cy;
      this.positions[i * 3 + 2] = cz;
      this.life[i] = LIFE * (0.75 + Math.random() * 0.25);
      this.tint[i].copy(_c);
    }
    this.onBurst?.();
  }

  update(dt: number): void {
    if (this.show > 0) {
      this.show -= dt;
      this.cooldown -= dt;
      if (this.cooldown <= 0) {
        this.burst();
        this.cooldown = CADENCE * (0.7 + Math.random() * 0.6);
      }
    }
    let any = false;
    for (let i = 0; i < SPARKS; i++) {
      if (this.life[i] <= 0) {
        this.colors[i * 3] = 0;
        this.colors[i * 3 + 1] = 0;
        this.colors[i * 3 + 2] = 0;
        continue;
      }
      any = true;
      this.life[i] -= dt;
      const k = i * 3;
      // Drag slows the spread; gravity pulls the fall toward the planet.
      const drag = 1 - dt * 1.4;
      this.velocity[k] = this.velocity[k] * drag - this.up.x * GRAVITY * dt;
      this.velocity[k + 1] = this.velocity[k + 1] * drag - this.up.y * GRAVITY * dt;
      this.velocity[k + 2] = this.velocity[k + 2] * drag - this.up.z * GRAVITY * dt;
      this.positions[k] += this.velocity[k] * dt;
      this.positions[k + 1] += this.velocity[k + 1] * dt;
      this.positions[k + 2] += this.velocity[k + 2] * dt;
      // Bright for the first moment, then a twinkling fade.
      const age = 1 - Math.max(0, this.life[i]) / LIFE;
      const twinkle = 0.7 + 0.3 * Math.sin(age * 40 + i);
      const v = (1 - age) * (1 - age) * twinkle * 1.6;
      const c = this.tint[i];
      this.colors[k] = c.r * v;
      this.colors[k + 1] = c.g * v;
      this.colors[k + 2] = c.b * v;
    }
    this.group.visible = any;
    if (any) {
      (this.points.geometry.attributes.position as THREE.BufferAttribute).needsUpdate = true;
      (this.points.geometry.attributes.color as THREE.BufferAttribute).needsUpdate = true;
    }
  }
}
