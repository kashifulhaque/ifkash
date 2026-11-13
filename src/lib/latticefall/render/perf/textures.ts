// Texture budgets (WP-081). The art direction is vertex colors and small gradient textures, so
// the world draws almost no textures; this module keeps it that way. It estimates GPU texture
// memory for a scene, and can shrink an oversize texture to the longest edge a quality preset
// allows. `render/lighting/quality.ts` stays free of Three.js, so the edge limits live in
// `budgets.ts`.

import type { Material, Object3D, Texture } from 'three';
import { TEXTURE_BUDGET_BYTES, TEXTURE_MAX_EDGE } from './budgets';
import type { QualityName } from '../lighting/quality';

export interface TextureAudit {
  count: number;
  /** Estimated GPU bytes, mipmaps included. */
  bytes: number;
  /** Longest edge among them, in pixels. */
  largest: number;
}

const MAP_KEYS = ['map', 'normalMap', 'roughnessMap', 'metalnessMap', 'emissiveMap', 'aoMap', 'alphaMap', 'bumpMap'] as const;

function sizeOf(t: Texture): { w: number; h: number } {
  const image = t.image as { width?: number; height?: number; videoWidth?: number } | undefined;
  return { w: image?.width ?? 0, h: image?.height ?? 0 };
}

/** Bytes a texture holds on the GPU: RGBA8, a third more with mipmaps. Compressed ones are quartered. */
export function estimateTextureBytes(t: Texture): number {
  const { w, h } = sizeOf(t);
  const compressed = (t as Texture & { isCompressedTexture?: boolean }).isCompressedTexture === true;
  let bytes = w * h * (compressed ? 1 : 4);
  if (t.generateMipmaps || (t.mipmaps && t.mipmaps.length > 1)) bytes *= 4 / 3;
  return Math.round(bytes);
}

/** Every distinct texture the scene's materials reference. */
export function sceneTextures(root: Object3D): Set<Texture> {
  const found = new Set<Texture>();
  root.traverse((o) => {
    const mat = (o as { material?: Material | Material[] }).material;
    if (!mat) return;
    for (const m of Array.isArray(mat) ? mat : [mat]) {
      for (const k of MAP_KEYS) {
        const t = (m as unknown as Record<string, Texture | null | undefined>)[k];
        if (t && t.isTexture) found.add(t);
      }
    }
  });
  return found;
}

export function auditTextures(root: Object3D): TextureAudit {
  const audit: TextureAudit = { count: 0, bytes: 0, largest: 0 };
  for (const t of sceneTextures(root)) {
    const { w, h } = sizeOf(t);
    audit.count++;
    audit.bytes += estimateTextureBytes(t);
    audit.largest = Math.max(audit.largest, w, h);
  }
  return audit;
}

/** True when the audit fits the GPU texture budget. */
export function withinTextureBudget(a: TextureAudit): boolean {
  return a.bytes <= TEXTURE_BUDGET_BYTES;
}

/**
 * Shrinks `t` in place so its longest edge is at most `maxEdge`, by drawing it onto a smaller
 * canvas. Does nothing for a texture with no readable image (compressed GPU textures are already
 * small and cannot be redrawn) or that already fits. Returns true when it shrank.
 */
export function capTextureEdge(t: Texture, maxEdge: number): boolean {
  const image = t.image as CanvasImageSource & { width: number; height: number };
  if (!image || !image.width || !image.height) return false;
  const edge = Math.max(image.width, image.height);
  if (edge <= maxEdge) return false;
  if (typeof document === 'undefined') return false;
  const k = maxEdge / edge;
  const canvas = document.createElement('canvas');
  canvas.width = Math.max(1, Math.round(image.width * k));
  canvas.height = Math.max(1, Math.round(image.height * k));
  const ctx = canvas.getContext('2d');
  if (!ctx) return false;
  try {
    ctx.drawImage(image, 0, 0, canvas.width, canvas.height);
  } catch {
    return false; // An ImageBitmap that was already transferred, or a cross-origin image.
  }
  t.image = canvas;
  t.needsUpdate = true;
  return true;
}

/** Caps every texture under `root` to the preset's longest edge. Returns how many it shrank. */
export function capSceneTextures(root: Object3D, preset: QualityName): number {
  let n = 0;
  for (const t of sceneTextures(root)) if (capTextureEdge(t, TEXTURE_MAX_EDGE[preset])) n++;
  return n;
}
