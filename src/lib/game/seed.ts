// Stable labels used to reproduce generated planets from shared URLs.

const MAX_LABEL = 48;

/** Trim and sanitise a raw label; returns `null` when nothing usable is left. */
export function normalizeSeedLabel(raw: string | null | undefined): string | null {
  if (!raw) return null;
  const label = raw.trim().replace(/\s+/g, '-').slice(0, MAX_LABEL);
  return label.length > 0 ? label : null;
}

/** FNV-1a over the UTF-16 code units, so any label becomes a 32-bit seed. */
export function hashSeed(label: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < label.length; i++) {
    h ^= label.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

/** Shareable URL for a generated world; depth preserves its onward route graph. */
export function seedLink(
  label: string,
  base: string = typeof location === 'undefined' ? '' : location.origin,
  depth = 0
): string {
  const route = depth > 0 ? `&depth=${Math.floor(depth)}` : '';
  return `${base}/game?seed=${encodeURIComponent(label)}${route}`;
}
