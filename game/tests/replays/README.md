# Golden replays

A `.lfr` file is one recorded run of the simulation. The test
`game/crates/tools/tests/golden.rs` replays every `.lfr` file in this folder and fails,
naming the file and the first mismatching tick, if any recorded state hash differs.

## Format

The file is [`postcard`](https://docs.rs/postcard) bytes of `lf_core::replay::Replay`
(format version 1):

- Header: `format_version` (u16), `content_hash` (u64), `seed` (u64), `scenario`
  (string: a file in `game/data/scenarios/` without the extension, such as `m0_flat`
  or `test_field`), and `start_tick` (u64, must be 0).
- Frames: run-length-encoded `ActionFrame` values, as `(frame, run)` pairs. A frame is
  `move_x: i8`, `move_z: i8`, `buttons: u16`.
- Commands: `(tick, Command)` pairs, applied on the tick whose number matches.
- Hashes: `(tick, state_hash)` pairs every 30 ticks, plus one after the final tick.

## Commands

Run these from `game/`, with `cargo run -q -p lf-tools --` before each command (or use
`mise run game:test`):

```sh
# Run a replay. Prints OK, or the first mismatch and exits 1.
lf-tools replay run tests/replays/m0_random_3000.lfr

# Re-record the expected hashes and content hash in place (all .lfr files if no FILE).
lf-tools replay rehash [FILE]

# Record a new random replay (on m0_flat).
lf-tools replay record-random --seed 20260101 --ticks 3000 --out tests/replays/m0_random_3000.lfr

# Record a replay from an input script (`cmd Rest` and `cmd SplitFlasks <vigil>` lines
# queue commands).
lf-tools replay record-script tests/replays/m1_movement.script --out tests/replays/m1_movement.lfr
```

## Replays

- `m0_random_3000.lfr`: 3,000 ticks of random input on `m0_flat`.
- `m1_movement.lfr`: the WP-010 route on `test_field`, recorded from
  `m1_movement.script`. It walks, sprints up the slope, jumps the 1 m ledge, and walks
  off the 10 m drop. The `m1_movement_route_does_what_it_says` test in
  `game/crates/tools/tests/golden.rs` checks each of those happens.
- `m2_three_hit_chain.lfr`: the WP-021 golden on `test_field`, recorded from
  `m2_three_hit_chain.script`. The player walks up to `dummy_ahead` and attacks with
  the three-hit light chain. The `m2_three_hit_chain_lands_exactly_three_hits` test
  checks it produces exactly three `Hit` events, one per light.
- `m2_death_recover.lfr`: the WP-029 golden on `test_field`, recorded from
  `m2_death_recover.script`. The player rests at `lantern_field`, runs off the edge of the
  world, dies (500 Glimmer drops as a Remnant at the edge), respawns at the Lantern, and
  runs back to recover it. The `m2_death_recover_gets_the_remnant_back` test checks the
  sequence of life events.
- `m2_death_lose_remnant.lfr`: the WP-029 golden recorded from
  `m2_death_lose_remnant.script`. The same first death, then a second death elsewhere
  before the Remnant is touched, which destroys it. The
  `m2_death_lose_remnant_loses_the_glimmer` test checks it.

Rehash only when a change to the simulation or the content data is intended. Review the
old and new final hash it prints, and commit the updated file with that change.
