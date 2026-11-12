# 0004: Keep portfolio data out of the content hash

- **Status:** Accepted
- **Date:** 2026-11-12
- **WP:** [WP-008](../10-roadmap.md#wp-008)

## Context

The [determinism contract](../01-architecture.md#determinism-contract) says the content
hash covers all `game/data/` files after parsing, and every replay header stores it. A
replay only runs against the same content hash.

WP-008 adds `game/data/portfolio.ron` and the generated
`game/data/gen/portfolio_content.json`, which is exported from `src/lib/content.ts`.
`content.ts` changes whenever the owner edits a job, project, or link. If the portfolio
were in the content hash, each of those edits would invalidate every golden replay, saved
replay, and RL dataset, although none of them changes the simulation.

## Decision

`lf-content` loads and validates the portfolio separately from `Content`. The content
hash covers only simulation data (attributes, moves, weapons, and the registry), not
`portfolio.ron` or `gen/`. `lf-tools validate` checks both.

## Consequences

- Editing `content.ts` never breaks replays.
- If a later WP makes portfolio data affect the simulation (for example, relic stats),
  that data must live in a simulation file such as `registry.ron` or a weapons file, so
  the content hash covers it. The portfolio files stay text and mappings only.
- The [determinism contract](../01-architecture.md#determinism-contract) now names this
  exception.

## Alternatives considered

- **Hash everything in `game/data/`:** matches the original wording, but turns every
  portfolio text edit into a determinism-breaking change.
- **Move the portfolio files out of `game/data/`:** clearer, but the WP-008 card fixes
  their paths, and the validator already reads them from there.
