// `&dev=1&ktx2test=1`: draws the KTX2 test model (a Poly Haven stone fire pit, ADR 0010) in
// front of the player's start position and reports how its textures loaded. The dev tools
// import this module dynamically, so normal play never fetches it, the model, or the Basis
// transcoder.
//
// The report goes to the console and to `window.__lfKtx2Test`. A texture that reached the GPU
// in a compressed format has `compressed: true` and a block format such as `ASTC 4x4`. A
// texture that was decoded to plain RGBA has `compressed: false`.

import {
  RGBA_ASTC_4x4_Format,
  RGBA_BPTC_Format,
  RGBA_ETC2_EAC_Format,
  RGBA_PVRTC_4BPPV1_Format,
  RGBA_S3TC_DXT1_Format,
  RGBA_S3TC_DXT5_Format,
  RGBAFormat,
  RGB_ETC1_Format,
  RGB_ETC2_Format,
  RGB_PVRTC_4BPPV1_Format,
  type CompressedTexture,
  type Material,
  type Object3D,
  type Scene,
  type Texture,
  type WebGLRenderer
} from 'three';
import { createGltfLoader } from '../render/assets/gltfLoader';

/** Written by `lf-tools assets` from `game/assets-src/polyhaven-stone-fire-pit/`. */
const MODEL_URL = '/latticefall/dev/stone_fire_pit.glb';
/** How far in front of the start position the model stands, in meters. */
const DISTANCE_M = 3;

const FORMAT_NAMES: Record<number, string> = {
  [RGBA_ASTC_4x4_Format]: 'ASTC 4x4',
  [RGBA_BPTC_Format]: 'BC7',
  [RGBA_S3TC_DXT1_Format]: 'BC1 (DXT1)',
  [RGBA_S3TC_DXT5_Format]: 'BC3 (DXT5)',
  [RGB_ETC2_Format]: 'ETC2 RGB',
  [RGBA_ETC2_EAC_Format]: 'ETC2 RGBA',
  [RGB_ETC1_Format]: 'ETC1',
  [RGB_PVRTC_4BPPV1_Format]: 'PVRTC RGB',
  [RGBA_PVRTC_4BPPV1_Format]: 'PVRTC RGBA',
  [RGBAFormat]: 'RGBA8 (not compressed)'
};

const SLOTS = ['map', 'normalMap', 'roughnessMap', 'metalnessMap', 'aoMap', 'emissiveMap'] as const;

export interface Ktx2TextureReport {
  /** The material slots that use the texture, for example `roughnessMap+metalnessMap`. */
  slots: string;
  compressed: boolean;
  format: string;
  colorSpace: string;
  width: number;
  height: number;
  mipLevels: number;
  /** Bytes the texture holds on the GPU, every mip level included. */
  gpuBytes: number;
}

export interface Ktx2TestReport {
  url: string;
  /** The GLB's size in bytes, or null when the server sent no `Content-Length`. */
  fileBytes: number | null;
  /** First load, in milliseconds: the download, the transcoder's download and start, and the transcode. */
  coldLoadMs: number;
  /** Second load, in milliseconds, with the transcoder workers already running. */
  warmLoadMs: number;
  triangles: number;
  textures: Ktx2TextureReport[];
  gpuTextureBytes: number;
  allCompressed: boolean;
}

declare global {
  interface Window {
    /** Set only for `&dev=1&ktx2test=1`, when the test model has loaded. */
    __lfKtx2Test?: Ktx2TestReport;
  }
}

function textureReports(root: Object3D): Ktx2TextureReport[] {
  const slotsByTexture = new Map<Texture, string[]>();
  root.traverse((o) => {
    const mat = (o as { material?: Material | Material[] }).material;
    if (!mat) return;
    for (const m of Array.isArray(mat) ? mat : [mat]) {
      for (const slot of SLOTS) {
        const t = (m as unknown as Record<string, Texture | null | undefined>)[slot];
        if (!t?.isTexture) continue;
        const slots = slotsByTexture.get(t) ?? [];
        if (!slots.includes(slot)) slots.push(slot);
        slotsByTexture.set(t, slots);
      }
    }
  });
  return [...slotsByTexture].map(([t, slots]) => {
    const compressed = (t as CompressedTexture).isCompressedTexture === true;
    const image = t.image as { width?: number; height?: number } | undefined;
    const width = image?.width ?? 0;
    const height = image?.height ?? 0;
    const mips = compressed ? (t as CompressedTexture).mipmaps ?? [] : [];
    const gpuBytes = compressed
      ? mips.reduce((sum, m) => sum + ((m.data as ArrayBufferView | null)?.byteLength ?? 0), 0)
      : Math.round(width * height * 4 * (t.generateMipmaps ? 4 / 3 : 1));
    return {
      slots: slots.join('+'),
      compressed,
      format: FORMAT_NAMES[t.format as number] ?? `format ${t.format}`,
      colorSpace: t.colorSpace || 'linear',
      width,
      height,
      mipLevels: compressed ? mips.length : 1,
      gpuBytes
    };
  });
}

function countTriangles(root: Object3D): number {
  let n = 0;
  root.traverse((o) => {
    const geom = (o as { geometry?: { index: { count: number } | null; attributes: { position?: { count: number } } } }).geometry;
    if (geom) n += (geom.index?.count ?? geom.attributes.position?.count ?? 0) / 3;
  });
  return n;
}

/**
 * Loads the test model, adds it to the scene in front of `start`, and returns the report.
 * Rejects when the model or the transcoder can't be loaded.
 */
export async function spawnKtx2Test(
  scene: Scene,
  renderer: WebGLRenderer,
  start: { x: number; y: number; z: number; yaw: number } | null
): Promise<Ktx2TestReport> {
  const loader = createGltfLoader(renderer);
  const t0 = performance.now();
  const gltf = await loader.loadAsync(MODEL_URL);
  const coldLoadMs = performance.now() - t0;

  // Load a second copy to time a load with the workers warm, and then drop it.
  const t1 = performance.now();
  const again = await loader.loadAsync(`${MODEL_URL}?warm=1`);
  const warmLoadMs = performance.now() - t1;
  again.scene.traverse((o) => {
    const mesh = o as { geometry?: { dispose(): void }; material?: Material | Material[] };
    mesh.geometry?.dispose();
    for (const m of mesh.material ? (Array.isArray(mesh.material) ? mesh.material : [mesh.material]) : []) {
      for (const slot of SLOTS) (m as unknown as Record<string, Texture | null | undefined>)[slot]?.dispose();
      m.dispose();
    }
  });

  const s = start ?? { x: 0, y: 0, z: 0, yaw: 0 };
  gltf.scene.name = 'ktx2-test-model';
  // Yaw 0 faces +Z and turns toward +X, as in the simulation.
  gltf.scene.position.set(s.x + Math.sin(s.yaw) * DISTANCE_M, s.y, s.z + Math.cos(s.yaw) * DISTANCE_M);
  gltf.scene.traverse((o) => {
    o.castShadow = true;
    o.receiveShadow = true;
  });
  scene.add(gltf.scene);

  const entry = performance.getEntriesByName(new URL(MODEL_URL, window.location.href).href)[0] as
    | PerformanceResourceTiming
    | undefined;
  const textures = textureReports(gltf.scene);
  const report: Ktx2TestReport = {
    url: MODEL_URL,
    fileBytes: entry?.encodedBodySize || entry?.decodedBodySize || null,
    coldLoadMs: Math.round(coldLoadMs),
    warmLoadMs: Math.round(warmLoadMs),
    triangles: countTriangles(gltf.scene),
    textures,
    gpuTextureBytes: textures.reduce((sum, t) => sum + t.gpuBytes, 0),
    allCompressed: textures.length > 0 && textures.every((t) => t.compressed)
  };
  window.__lfKtx2Test = report;
  console.info('[latticefall dev] ktx2test', report);
  console.table(textures);
  return report;
}
