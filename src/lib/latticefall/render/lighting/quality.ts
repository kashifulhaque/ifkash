// Quality presets (docs/game/07-presentation.md, "Quality presets"). No Three.js import, so
// the menus can read and set the preset without pulling in the renderer. The renderer
// subscribes and re-applies the numbers when the preset changes.
//
// The world renderer applies render scale, shadows, draw distance, and grass density.
// `postProcessing` and `reflections` are carried for the post-processing and Mirrormere
// work; nothing here reads them.

export type QualityName = 'low' | 'medium' | 'high';

export interface QualityPreset {
  name: QualityName;
  /** Pixel ratio as a fraction of CSS pixels; `null` means the device pixel ratio, capped at 2. */
  renderScale: number | null;
  /** Sun shadow map size, or 0 for none. Low draws no sun shadow. */
  shadowMapSize: number;
  /** Shadow cascades; the sun shadow uses one until the high preset's second cascade lands. */
  shadowCascades: number;
  /** Meters from the player at which chunks of scatter and kit stop drawing. */
  drawDistance: number;
  /** Fraction of grass scatter drawn, 0 to 1. */
  grassDensity: number;
  postProcessing: 'fog' | 'fog-bloom' | 'fog-bloom-ao';
  reflections: 'environment-map' | 'planar';
}

export const QUALITY_PRESETS: Readonly<Record<QualityName, QualityPreset>> = {
  low: {
    name: 'low',
    renderScale: 0.75,
    shadowMapSize: 0,
    shadowCascades: 0,
    drawDistance: 120,
    grassDensity: 0.25,
    postProcessing: 'fog',
    reflections: 'environment-map'
  },
  medium: {
    name: 'medium',
    renderScale: 1,
    shadowMapSize: 2048,
    shadowCascades: 1,
    drawDistance: 200,
    grassDensity: 0.6,
    postProcessing: 'fog-bloom',
    reflections: 'environment-map'
  },
  high: {
    name: 'high',
    renderScale: null,
    shadowMapSize: 4096,
    shadowCascades: 2,
    drawDistance: 320,
    grassDensity: 1,
    postProcessing: 'fog-bloom-ao',
    reflections: 'planar'
  }
};

const STORAGE_KEY = 'latticefall-quality';
export const DEFAULT_QUALITY: QualityName = 'medium';

export function isQualityName(v: unknown): v is QualityName {
  return v === 'low' || v === 'medium' || v === 'high';
}

function readStored(): QualityName {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (isQualityName(v)) return v;
  } catch {
    // Storage can be blocked; fall through to the default.
  }
  return DEFAULT_QUALITY;
}

let current: QualityName | null = null;
const listeners = new Set<(p: QualityPreset) => void>();

/** The active preset: the saved one, else medium. */
export function getQuality(): QualityPreset {
  current ??= readStored();
  return QUALITY_PRESETS[current];
}

/** Switches preset, saves it, and tells every subscriber. Safe to call from the menus at any time. */
export function setQuality(name: QualityName): void {
  if (!isQualityName(name)) return;
  const changed = getQuality().name !== name;
  current = name;
  try {
    localStorage.setItem(STORAGE_KEY, name);
  } catch {
    // Not persisted; the preset still applies for this session.
  }
  if (changed) for (const fn of [...listeners]) fn(QUALITY_PRESETS[name]);
}

/** Calls `fn` on every preset change (not immediately). Returns the unsubscribe function. */
export function subscribeQuality(fn: (p: QualityPreset) => void): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

/** The renderer pixel ratio for a preset on a display with the given device pixel ratio. */
export function pixelRatioFor(p: QualityPreset, devicePixelRatio: number): number {
  return p.renderScale ?? Math.min(devicePixelRatio, 2);
}
