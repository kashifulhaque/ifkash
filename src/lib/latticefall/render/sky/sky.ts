// The sky: a gradient dome that follows the camera, plus a sun disc. The horizon always
// takes the current fog color so distant ground melts into it; the zenith comes from the
// area's `sky` preset name. Unknown presets fall back to `day`.

import {
  BackSide,
  BufferAttribute,
  CircleGeometry,
  Color,
  Mesh,
  MeshBasicMaterial,
  PerspectiveCamera,
  Scene,
  SphereGeometry,
  Vector3
} from 'three';

/** Must stay under the camera's far plane (400). */
const RADIUS = 300;

interface SkyPreset {
  zenith: string;
  /** How much of the gradient is horizon color near the bottom, 0..1. */
  haze: number;
  sun: boolean;
}

export const SKY_PRESETS: Readonly<Record<string, SkyPreset>> = {
  day: { zenith: '#6d93b3', haze: 0.35, sun: true },
  overcast: { zenith: '#8e979c', haze: 0.55, sun: false },
  dusk: { zenith: '#3d3f6b', haze: 0.4, sun: true },
  night: { zenith: '#0c1022', haze: 0.3, sun: false },
  dark: { zenith: '#070612', haze: 0.2, sun: false }
};

export class Sky {
  private dome: Mesh<SphereGeometry, MeshBasicMaterial>;
  private sunDisc: Mesh<CircleGeometry, MeshBasicMaterial>;
  private glow: Mesh<CircleGeometry, MeshBasicMaterial>;
  private preset: SkyPreset;
  private zenith = new Color();
  private lastHorizon = new Color(-1, -1, -1);
  private dir = new Vector3();

  constructor(
    private scene: Scene,
    presetName: string,
    sunDir: Vector3
  ) {
    this.preset = SKY_PRESETS[presetName] ?? SKY_PRESETS.day;
    this.zenith.set(this.preset.zenith);
    // `sunDir` is the way the light travels; the sun sits the opposite way.
    this.dir.copy(sunDir).negate().normalize();

    const geo = new SphereGeometry(RADIUS, 24, 12);
    geo.setAttribute('color', new BufferAttribute(new Float32Array(geo.getAttribute('position').count * 3), 3));
    this.dome = new Mesh(geo, new MeshBasicMaterial({ vertexColors: true, side: BackSide, fog: false, depthWrite: false, depthTest: false }));
    this.dome.renderOrder = -1000;
    this.dome.frustumCulled = false;

    const disc = (radius: number, opacity: number, order: number) => {
      const m = new Mesh(
        new CircleGeometry(radius, 24),
        new MeshBasicMaterial({ fog: false, depthWrite: false, depthTest: false, transparent: true, opacity })
      );
      m.renderOrder = order;
      m.frustumCulled = false;
      m.visible = this.preset.sun;
      return m;
    };
    this.glow = disc(46, 0.22, -998);
    this.sunDisc = disc(10, 0.95, -997);
    scene.add(this.dome, this.glow, this.sunDisc);
  }

  /** Recolors the dome when the horizon color moves, and recenters everything on the camera. */
  update(camera: PerspectiveCamera, horizon: Color, sunColor: Color): void {
    this.dome.position.copy(camera.position);
    const p = new Vector3().copy(this.dir).multiplyScalar(RADIUS * 0.92).add(camera.position);
    for (const m of [this.glow, this.sunDisc]) {
      m.position.copy(p);
      m.lookAt(camera.position);
      m.material.color.copy(sunColor);
    }
    if (this.lastHorizon.equals(horizon)) return;
    this.lastHorizon.copy(horizon);
    const geo = this.dome.geometry;
    const pos = geo.getAttribute('position');
    const col = geo.getAttribute('color') as BufferAttribute;
    const c = new Color();
    for (let i = 0; i < pos.count; i++) {
      const t = Math.min(1, Math.max(0, pos.getY(i) / RADIUS));
      // Stay on the horizon color through the haze band, then rise to the zenith.
      const k = Math.pow(Math.max(0, t - this.preset.haze * 0.4) / (1 - this.preset.haze * 0.4), 0.75);
      c.copy(horizon).lerp(this.zenith, k);
      col.setXYZ(i, c.r, c.g, c.b);
    }
    col.needsUpdate = true;
  }

  dispose(): void {
    this.scene.remove(this.dome, this.glow, this.sunDisc);
    for (const m of [this.dome, this.glow, this.sunDisc]) {
      m.geometry.dispose();
      m.material.dispose();
    }
  }
}
