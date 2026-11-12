// Resolves the portfolio entries a dialogue topic or station names into the facts panel's
// items (docs/game/08-portfolio-integration.md, rule 1): the data names an entry by section and
// slug, and the text always comes from `src/lib/content.ts`, so no fact is copied into game
// data and an entry added to `content.ts` reaches the hub by itself. No Three.js or WASM import.

import { sections, type LootItem } from '$lib/content';
// The relic mapping is read from the same file the validator reads, so the keepsake table can
// never disagree with it. Vite inlines the text at build time.
import portfolioRon from '../../../../../game/data/portfolio.ron?raw';
import { ALL_SLUGS, type FactRef } from './dialogueState';

/** The slugs of the projects that `portfolio.ron` turns into relics. */
export function mappedProjects(text: string = portfolioRon): Set<string> {
  const out = new Set<string>();
  for (const m of text.matchAll(/project:\s*"([^"]+)"/g)) out.add(m[1]);
  return out;
}

/** The items the references name, in the order of `content.ts`, each once. */
export function resolveFacts(refs: readonly FactRef[]): LootItem[] {
  const out: LootItem[] = [];
  for (const section of sections) {
    for (const item of section.items) {
      if (!item.slug) continue;
      const named = refs.some((r) => r.section === section.id && (r.slug === ALL_SLUGS || r.slug === item.slug));
      if (named && !out.includes(item)) out.push(item);
    }
  }
  return out;
}

/** The keepsake table's items: the named projects that have no relic mapping. */
export function keepsakes(refs: readonly FactRef[], mapped: ReadonlySet<string> = mappedProjects()): LootItem[] {
  return resolveFacts(refs).filter((item) => item.slug !== undefined && !mapped.has(item.slug));
}
