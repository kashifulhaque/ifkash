# 0007: Name the game Cinderheart

- **Status:** Accepted
- **Date:** 2026-11-12
- **WP:** None (an open owner decision listed in the design docs README)

## Context

The design docs called the game *Latticefall* as a working title. The docs README listed
"confirm the working title after a trademark search" as an owner decision needed before
[WP-086](../10-roadmap.md#wp-086), the launch.

## Decision

The owner named the game *Cinderheart* on 2026-11-12. The design docs use the new name
in prose from this date.

*Cinderheart* is the public-facing name: page titles, menus, the HUD, the landing card,
and anything else a player reads. Internal code keeps *Latticefall*, and the owner
confirmed it stays: the Rust crates (`lf-core`, `lf-tools`, and the rest),
`src/lib/latticefall/`, the `engine=lf` URL flag, the `latticefall` Python package, the
`Latticefall/*` Gymnasium IDs in [the RL spec](../09-rl-environment.md), and console
messages.

## Consequences

- The game page's title and its loading and error messages now say *Cinderheart*. New
  player-facing text must use *Cinderheart*; new code identifiers keep the `latticefall`
  and `lf` prefixes.
- Earlier ADRs, `JOURNAL.md` entries, and `STATUS.md` notes keep the old name, because
  they're historical records.
- This ADR records no trademark search. Do one before launch if the owner wants it.

## Alternatives considered

- Keep *Latticefall*: the owner chose a new name.
- Rename every code identifier: the owner decided against it. Players never see them,
  and a rename across crates, paths, and the Python package would churn every branch.
