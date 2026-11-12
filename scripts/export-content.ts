// Exports the portfolio sections from src/lib/content.ts to JSON for the game's Rust
// validator. Run from the repo root: `bun scripts/export-content.ts`.
// Items without a slug are exported with `slug: null` so the validator can report them.
import { sections } from '../src/lib/content';

const out = {
  sections: sections.map((s) => ({
    id: s.id,
    items: s.items.map((i) => ({ slug: i.slug ?? null, title: i.title }))
  }))
};

const dest = new URL('../game/data/gen/portfolio_content.json', import.meta.url);
await Bun.write(dest, JSON.stringify(out, null, 2) + '\n');
console.log(`wrote ${dest.pathname}`);
