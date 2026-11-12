// The codex catalogue: every portfolio entry the codex can show, keyed `SECTION.SLUG`
// (for example `projects.banana-cpp`), which is also the suffix of the core's
// `codex.<key>` world flag. Facts come from `$lib/content` and are never copied here.
// No Three.js or WASM import, so the home page may read this module.

import { sections, type LootItem, type SectionId } from '$lib/content';

export interface CodexEntry {
  /** `SECTION.SLUG`. */
  key: string;
  section: SectionId | 'stub';
  slug: string;
  /** The facts, as written in `content.ts` (the stub's are placeholder text). */
  item: LootItem;
  /** One line of in-world framing; plain facts never carry it. */
  flavor?: string;
}

export function entryKey(section: string, slug: string): string {
  return `${section}.${slug}`;
}

/** Sections whose items become codex pages the player finds in the world. */
const FOUND_SECTIONS: readonly string[] = ['work', 'projects'];

/** The stub relic the test field places (WP-049): proves the pickup-to-codex path. */
const STUB: CodexEntry = {
  key: 'projects.test-relic',
  section: 'stub',
  slug: 'test-relic',
  flavor: 'A plain brass token, left on the field to prove that a pickup unlocks a page.',
  item: {
    slug: 'test-relic',
    title: 'Test relic',
    body: 'A stand-in relic on the test field. Taking it unlocks this codex entry.',
    links: []
  }
};

const ALL: CodexEntry[] = [
  ...sections.flatMap((s) =>
    s.items.flatMap((item): CodexEntry[] =>
      item.slug ? [{ key: entryKey(s.id, item.slug), section: s.id, slug: item.slug, item }] : []
    )
  ),
  STUB
];

const BY_KEY = new Map(ALL.map((e) => [e.key, e]));

/** Every entry, portfolio order, the stub last. */
export function allEntries(): readonly CodexEntry[] {
  return ALL;
}

export function entryByKey(key: string): CodexEntry | undefined {
  return BY_KEY.get(key);
}

/** Entries of the sections a player finds in the world: Chronicle Pages and relics. */
export function findableEntries(): CodexEntry[] {
  return ALL.filter((e) => e.section !== 'stub' && FOUND_SECTIONS.includes(e.section));
}

/**
 * The codex entry a pickup's item unlocks, by registry number. The core raises the same
 * `codex.<key>` flag from the scenario's pickup; this table lets the browser name the entry
 * from the event, which carries only the number. Later WPs fill it from `portfolio.ron`.
 */
export const PICKUP_ENTRY: Readonly<Record<number, string>> = {
  11210: 'projects.banana-cpp',
  11211: 'projects.vicharak-micro-llama',
  11212: 'work.corteva',
  12000: 'projects.test-relic'
};
