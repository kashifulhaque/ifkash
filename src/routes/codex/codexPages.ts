// Helpers for the codex routes: which entries get a page, how the index groups them, and the
// metadata text. Reads only `$lib/latticefall/codex/entries` (no Three.js or WASM import), so
// the codex pages prerender without the game bundle.

import { error } from '@sveltejs/kit';
import { entryByKey, findableEntries, type CodexEntry } from '$lib/latticefall/codex/entries';

export type CodexSection = 'work' | 'projects';

export const INDEX_TITLE = 'Codex | Kashiful Haque';
export const INDEX_DESCRIPTION = 'Kashiful Haque: ML engineer. Portfolio, and a small action RPG.';

/** The index's groups, in the order the codex lists them: portfolio sections first. */
export const GROUPS: readonly { section: CodexSection; label: string; kind: string }[] = [
  { section: 'work', label: 'Chronicle Pages', kind: 'Chronicle Page' },
  { section: 'projects', label: 'Relics', kind: 'Relic' }
];

/** The entries of one section that get a codex page, in `content.ts` order. */
export function entriesIn(section: CodexSection): CodexEntry[] {
  return findableEntries().filter((e) => e.section === section);
}

export function entryHref(entry: CodexEntry): string {
  return `/codex/${entry.section}/${entry.slug}`;
}

/** The meta description for an entry page: its own text, else its title, then the owner's line. */
export function entryDescription(entry: CodexEntry): string {
  const text = entry.item.body ?? entry.item.title;
  const short = text.length > 120 ? `${text.slice(0, 117).trimEnd()}...` : text;
  const sentence = /[.!?]$/.test(short) ? short : `${short}.`;
  return `${sentence} Codex entry from the portfolio of Kashiful Haque, ML engineer.`;
}

/** The entry a codex route names, or a 404 when the slug is not a page of that section. */
export function entryOr404(section: CodexSection, slug: string): CodexEntry {
  const entry = entryByKey(`${section}.${slug}`);
  if (!entry || entry.section !== section) {
    throw error(404, `No codex entry at ${section}/${slug}`);
  }
  return entry;
}
