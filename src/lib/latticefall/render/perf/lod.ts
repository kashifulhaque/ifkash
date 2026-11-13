// Level-of-detail and culling distances (WP-081), all derived from the quality preset's draw
// distance so one slider moves them together. No Three.js import.

import type { QualityPreset } from '../lighting/quality';

/** Meters beyond which an enemy is not posed or drawn: it would be a few pixels in the fog. */
export function actorCullDistance(p: QualityPreset): number {
  return p.drawDistance * 0.4;
}

/** Meters beyond which scatter and kit chunks stop casting shadows (they stay drawn). */
export function shadowCastDistance(p: QualityPreset): number {
  return Math.min(p.drawDistance * 0.3, 70);
}

/**
 * Terrain chunk resolution by distance: how many heightfield cells one rendered quad spans.
 * Chunks inside the first ring draw every cell; each ring beyond doubles the stride, up to 8.
 * The rings scale with the draw distance so a low preset gets coarse ground sooner.
 */
export function terrainStride(distance: number, p: QualityPreset): 1 | 2 | 4 | 8 {
  const d = distance / p.drawDistance;
  if (d < 0.22) return 1;
  if (d < 0.42) return 2;
  if (d < 0.7) return 4;
  return 8;
}
