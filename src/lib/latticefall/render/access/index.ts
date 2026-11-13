// Accessibility state the renderer shares (WP-082): reduced motion, the color-blind mode, and
// the shape cues for telegraphs. The settings menu pushes the values in through
// `Engine.access.apply`; the effects read them each frame. Nothing here touches the simulation.
//
// Every telegraph already differs by shape from the thing it warns about (a ring that gathers
// in versus a wave that runs out, a lane versus a patch). The two that were only a colored
// line, B02's charge lane and B12's loom lines, now draw as a striped pattern while they warn,
// and turn solid when the danger is live. The color-blind modes swap their tint for one that
// stays apart from the arena's colors; the pattern carries the meaning either way.

import { CanvasTexture, RepeatWrapping, SRGBColorSpace } from 'three';

export type ColorBlindMode = 'off' | 'protanopia' | 'deuteranopia' | 'tritanopia';

export interface RenderAccess {
  /** No camera shake, no flashes, no flickering pulses. */
  reducedMotion: boolean;
  colorBlind: ColorBlindMode;
}

const state: RenderAccess = { reducedMotion: false, colorBlind: 'off' };

export function renderAccess(): Readonly<RenderAccess> {
  return state;
}

export function setRenderAccess(next: Partial<RenderAccess>): void {
  if (next.reducedMotion !== undefined) state.reducedMotion = next.reducedMotion;
  if (next.colorBlind !== undefined) state.colorBlind = next.colorBlind;
}

export interface TelegraphColors {
  /** A danger that is coming: the striped warning. */
  warn: number;
  /** A danger that is live: the solid beam or lane. */
  live: number;
}

/**
 * Warning and live colors per mode. Red and green merge for protanopia and deuteranopia, so
 * those modes use blue and white against the arenas' warm browns; tritanopia confuses blue and
 * yellow, so it uses a pink and a near-white pair.
 */
const COLORS: Record<ColorBlindMode, TelegraphColors> = {
  off: { warn: 0xffd98a, live: 0xfff6e0 },
  protanopia: { warn: 0x6cc4ff, live: 0xf2fbff },
  deuteranopia: { warn: 0x6cc4ff, live: 0xf2fbff },
  tritanopia: { warn: 0xff9fb8, live: 0xfff0f4 }
};

export function telegraphColors(): TelegraphColors {
  return COLORS[state.colorBlind];
}

/** Meters one stripe period covers; set a stripe texture's `repeat` to length / this. */
export const STRIPE_METERS = 1.2;

/**
 * Slanted stripes with clear gaps, white so a material's color tints them. The period runs
 * along the texture's U axis, or its V axis for `'v'`. Returns null where there is no canvas
 * (headless builds), and the effect keeps its flat color.
 */
export function stripeTexture(axis: 'u' | 'v' = 'u'): CanvasTexture | null {
  if (typeof document === 'undefined') return null;
  const c = document.createElement('canvas');
  const u = axis === 'u';
  c.width = u ? 64 : 32;
  c.height = u ? 32 : 64;
  const g = c.getContext('2d');
  if (!g) return null;
  g.fillStyle = '#fff';
  g.beginPath();
  // A slanted bar spanning the cross axis, about 40% of the period wide.
  const pts: [number, number][] = [
    [0, 32],
    [28, 32],
    [46, 0],
    [18, 0]
  ];
  pts.forEach(([x, y], i) => {
    const px = u ? x : y;
    const py = u ? y : x;
    if (i === 0) g.moveTo(px, py);
    else g.lineTo(px, py);
  });
  g.closePath();
  g.fill();
  const t = new CanvasTexture(c);
  t.wrapS = t.wrapT = RepeatWrapping;
  t.colorSpace = SRGBColorSpace;
  return t;
}
