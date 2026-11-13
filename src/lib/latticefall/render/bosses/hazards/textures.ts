// Small procedural textures the hazard effects share. Each is created once per effects object and
// disposed with it. They need a 2D canvas, so a headless build returns null and the effects fall
// back to flat colors.

import { CanvasTexture, RepeatWrapping, SRGBColorSpace } from 'three';

function canvas(size: number): { c: HTMLCanvasElement; g: CanvasRenderingContext2D } | null {
  if (typeof document === 'undefined') return null;
  const c = document.createElement('canvas');
  c.width = c.height = size;
  const g = c.getContext('2d');
  return g ? { c, g } : null;
}

/** A soft round glow: white in the middle, clear at the rim. Tint it with the material color. */
export function glowTexture(): CanvasTexture | null {
  const made = canvas(64);
  if (!made) return null;
  const grad = made.g.createRadialGradient(32, 32, 0, 32, 32, 32);
  grad.addColorStop(0, 'rgba(255,255,255,1)');
  grad.addColorStop(0.35, 'rgba(255,255,255,0.45)');
  grad.addColorStop(1, 'rgba(255,255,255,0)');
  made.g.fillStyle = grad;
  made.g.fillRect(0, 0, 64, 64);
  const t = new CanvasTexture(made.c);
  t.colorSpace = SRGBColorSpace;
  return t;
}

/** Cooling lava: dark crust cut by bright orange seams. Tiles, so a scroll never shows an edge. */
export function lavaTexture(): CanvasTexture | null {
  const made = canvas(128);
  if (!made) return null;
  const { g } = made;
  g.fillStyle = '#c8350a';
  g.fillRect(0, 0, 128, 128);
  // A fixed pseudo-random sequence, so every run draws the same crust.
  let seed = 12345;
  const rand = () => {
    seed = (seed * 1664525 + 1013904223) >>> 0;
    return seed / 4294967296;
  };
  for (let i = 0; i < 46; i++) {
    const x = rand() * 128;
    const y = rand() * 128;
    const r = 8 + rand() * 18;
    g.fillStyle = `rgba(${28 + rand() * 24}, ${10 + rand() * 8}, 6, ${0.55 + rand() * 0.35})`;
    // Drawn at the tile's wrapped positions too, so the edges meet.
    for (const [ox, oy] of [
      [0, 0],
      [-128, 0],
      [128, 0],
      [0, -128],
      [0, 128]
    ]) {
      g.beginPath();
      g.ellipse(x + ox, y + oy, r, r * (0.55 + rand() * 0.3), rand() * Math.PI, 0, Math.PI * 2);
      g.fill();
    }
  }
  const t = new CanvasTexture(made.c);
  t.wrapS = t.wrapT = RepeatWrapping;
  t.colorSpace = SRGBColorSpace;
  return t;
}
