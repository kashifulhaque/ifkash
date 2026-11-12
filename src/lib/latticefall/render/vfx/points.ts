// A pool of soft point sprites drawn in one call: hit sparks, flashes, aura motes.
// The owner refills it every frame (`clear`, `push` many, `commit`), so particles keep their
// state in plain arrays and the GPU buffers are just a mirror.

import { BufferAttribute, BufferGeometry, NormalBlending, Points, ShaderMaterial } from 'three';

const VERT = /* glsl */ `
  attribute float aSize;
  attribute vec4 aColor;
  uniform float uScale;
  varying vec4 vColor;
  void main() {
    vColor = aColor;
    vec4 mv = modelViewMatrix * vec4(position, 1.0);
    gl_Position = projectionMatrix * mv;
    // aSize is a diameter in meters; uScale turns meters at 1 m depth into pixels.
    gl_PointSize = max(aSize * uScale / max(-mv.z, 0.1), 1.0);
  }
`;

const FRAG = /* glsl */ `
  varying vec4 vColor;
  void main() {
    float d = length(gl_PointCoord - vec2(0.5)) * 2.0;
    if (d > 1.0) discard;
    float a = 1.0 - d;
    a *= a;
    gl_FragColor = vec4(vColor.rgb, vColor.a * a);
  }
`;

export function makePointMaterial(): ShaderMaterial {
  return new ShaderMaterial({
    uniforms: { uScale: { value: 600 } },
    vertexShader: VERT,
    fragmentShader: FRAG,
    transparent: true,
    depthWrite: false,
    // Normal blending: additive light vanishes against the site's pale ground.
    blending: NormalBlending
  });
}

export class PointCloud {
  readonly points: Points;
  private geometry = new BufferGeometry();
  private pos: Float32Array;
  private size: Float32Array;
  private col: Float32Array;
  private n = 0;

  constructor(
    readonly capacity: number,
    material: ShaderMaterial
  ) {
    this.pos = new Float32Array(capacity * 3);
    this.size = new Float32Array(capacity);
    this.col = new Float32Array(capacity * 4);
    this.geometry.setAttribute('position', new BufferAttribute(this.pos, 3).setUsage(35048));
    this.geometry.setAttribute('aSize', new BufferAttribute(this.size, 1).setUsage(35048));
    this.geometry.setAttribute('aColor', new BufferAttribute(this.col, 4).setUsage(35048));
    this.geometry.setDrawRange(0, 0);
    this.points = new Points(this.geometry, material);
    // The bounding sphere is never recomputed as particles move.
    this.points.frustumCulled = false;
    this.points.renderOrder = 10;
  }

  /** Points pushed since the last `clear`. */
  get count(): number {
    return this.n;
  }

  clear(): void {
    this.n = 0;
  }

  push(x: number, y: number, z: number, size: number, r: number, g: number, b: number, a: number): void {
    if (this.n >= this.capacity) return;
    const i = this.n++;
    this.pos[i * 3] = x;
    this.pos[i * 3 + 1] = y;
    this.pos[i * 3 + 2] = z;
    this.size[i] = size;
    this.col[i * 4] = r;
    this.col[i * 4 + 1] = g;
    this.col[i * 4 + 2] = b;
    this.col[i * 4 + 3] = a;
  }

  commit(): void {
    this.geometry.setDrawRange(0, this.n);
    for (const name of ['position', 'aSize', 'aColor']) {
      (this.geometry.getAttribute(name) as BufferAttribute).needsUpdate = true;
    }
  }

  dispose(): void {
    this.geometry.dispose();
  }
}
