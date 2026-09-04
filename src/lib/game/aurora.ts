import * as THREE from 'three';
import { tangentBasis } from './planet';

const W = 48;
const H = 6;

/**
 * Curtains of light above the arctic. Hidden until the aurora is woken, then
 * fades in and ripples.
 */
export class Aurora {
  group = new THREE.Group();
  private ribbons: { mesh: THREE.Mesh; base: Float32Array; phase: number }[] = [];
  private opacity = 0;
  private active = false;
  /** 0 by day, 1 at midnight; the ribbons glow brighter after dark. */
  night = 0;

  constructor(center: THREE.Vector3, radius: number) {
    const right = new THREE.Vector3();
    const fwd = new THREE.Vector3();
    tangentBasis(center, 0, right, fwd);
    this.group.position.copy(center).multiplyScalar(radius + 5.5);
    const m = new THREE.Matrix4().makeBasis(right, center, fwd);
    this.group.quaternion.setFromRotationMatrix(m);

    for (let i = 0; i < 3; i++) {
      const geo = new THREE.PlaneGeometry(26 + i * 4, 6 + i, W - 1, H - 1);
      const colors = new Float32Array(geo.attributes.position.count * 3);
      const c = new THREE.Color();
      for (let v = 0; v < geo.attributes.position.count; v++) {
        const y = geo.attributes.position.getY(v);
        const t = (y + 3) / 6;
        c.setHSL(0.42 - t * 0.2 + i * 0.05, 0.9, 0.55 + t * 0.15);
        colors[v * 3] = c.r;
        colors[v * 3 + 1] = c.g;
        colors[v * 3 + 2] = c.b;
      }
      geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
      const mat = new THREE.MeshBasicMaterial({
        vertexColors: true,
        transparent: true,
        opacity: 0,
        side: THREE.DoubleSide,
        depthWrite: false,
        blending: THREE.AdditiveBlending
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.set(0, 6 + i * 1.4, -4 + i * 4);
      mesh.rotation.y = (i - 1) * 0.35;
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
    const target = this.active ? 0.3 + this.night * 0.4 : 0;
    this.opacity += (target - this.opacity) * Math.min(1, dt * 1.2);
    if (this.opacity < 0.005 && !this.active) {
      this.group.visible = false;
      return;
    }
    for (const r of this.ribbons) {
      (r.mesh.material as THREE.MeshBasicMaterial).opacity = this.opacity;
      const pos = r.mesh.geometry.attributes.position as THREE.BufferAttribute;
      for (let v = 0; v < pos.count; v++) {
        const x = r.base[v * 3];
        const y = r.base[v * 3 + 1];
        const wave = Math.sin(x * 0.35 + time * 0.9 + r.phase) * 0.8 + Math.sin(x * 0.9 - time * 0.6) * 0.3;
        pos.setXYZ(v, x, y + wave * ((y + 3) / 6), wave * 0.6);
      }
      pos.needsUpdate = true;
    }
  }
}
