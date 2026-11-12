# Golden replays

A `.lfr` file is one recorded run of the simulation. The test
`game/crates/tools/tests/golden.rs` replays every `.lfr` file in this folder and fails,
naming the file and the first mismatching tick, if any recorded state hash differs.

## Format

The file is [`postcard`](https://docs.rs/postcard) bytes of `lf_core::replay::Replay`
(format version 1):

- Header: `format_version` (u16), `content_hash` (u64), `seed` (u64), `scenario`
  (string, `m0_flat` for now), and `start_tick` (u64, must be 0).
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

# Record a new random replay.
lf-tools replay record-random --seed 20260101 --ticks 3000 --out tests/replays/m0_random_3000.lfr
```

Rehash only when a change to the simulation or the content data is intended. Review the
old and new final hash it prints, and commit the updated file with that change.
