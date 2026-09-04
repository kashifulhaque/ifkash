// World seeds. A seed is a short human-readable label ("2026-09-04",
// "quiet-fox-73") that the URL carries as `?seed=`. The label is hashed to the
// integer that drives `noise.ts`. Without a `?seed=`, the planet of the day is
// used, so every visitor sees the same world on a given date and a shared link
// reproduces it later.

const MAX_LABEL = 48;

const ADJECTIVES = [
  'quiet', 'amber', 'misty', 'sunny', 'mossy', 'windy', 'frosty', 'golden',
  'sleepy', 'hidden', 'gentle', 'lonely', 'cozy', 'rosy', 'dusty', 'silver',
  'briny', 'wild', 'humble', 'lucky', 'velvet', 'ember', 'pale', 'little'
];
const NOUNS = [
  'fox', 'harbour', 'meadow', 'lantern', 'pebble', 'orchard', 'comet', 'sparrow',
  'anchor', 'willow', 'kettle', 'saddle', 'thistle', 'otter', 'beacon', 'clover',
  'glacier', 'canyon', 'marsh', 'tide', 'hollow', 'summit', 'reef', 'dune'
];

/** Trim and sanitise a raw label; returns `null` when nothing usable is left. */
export function normalizeSeedLabel(raw: string | null | undefined): string | null {
  if (!raw) return null;
  const label = raw.trim().replace(/\s+/g, '-').slice(0, MAX_LABEL);
  return label.length > 0 ? label : null;
}

/** Label for today's planet, in the visitor's local date. */
export function dailySeedLabel(date = new Date()): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

/** A fresh, friendly label such as "quiet-fox-73". */
export function randomSeedLabel(): string {
  const pick = <T>(arr: T[]) => arr[Math.floor(Math.random() * arr.length)];
  const n = Math.floor(Math.random() * 90) + 10;
  return `${pick(ADJECTIVES)}-${pick(NOUNS)}-${n}`;
}

/** Seed label for a page load: `?seed=` from `search` if present, otherwise today's. */
export function resolveSeedLabel(search: string): string {
  return normalizeSeedLabel(new URLSearchParams(search).get('seed')) ?? dailySeedLabel();
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

/** Shareable URL for the given seed on the current origin. */
export function seedLink(label: string, base: string = typeof location === 'undefined' ? '' : location.origin): string {
  return `${base}/game?seed=${encodeURIComponent(label)}`;
}
