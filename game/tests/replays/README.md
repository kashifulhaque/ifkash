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
- `m2_parry_riposte.lfr`: the WP-024 golden on `duel_parry`, from `m2_parry_riposte.script`.
  The player holds guard with the parrying dagger as the scripted duelist swings, parries
  it, then ripostes. `m2_parry_riposte_parries_then_ripostes` checks the `Parry`,
  `Critical`, and `Hit` events, and that the parry cost nothing.
- `m2_stance_break.lfr`: the WP-024 golden on `duel_stance`, from `m2_stance_break.script`.
  Two charged heavies break the elite dummy's stance. `m2_stance_break_breaks_with_charged_heavies`
  checks the `StanceBreak` event arrives with the second hit.
- `m2_backstab.lfr`: the WP-024 golden on `duel_backstab`, from `m2_backstab.script`. The
  player presses `LIGHT` behind an unaware dummy that faces away, and a backstab lands.
  `m2_backstab_stabs_the_unaware` checks it.

- `m2_projectiles.lfr`: the WP-027 golden on `m2_range`, recorded from
  `m2_projectiles.script`. The player fires an arrow that falls into the floor, locks on
  and fires an arrow at the dummy, then a seeker bolt that curves onto it. The
  `m2_projectiles_arrow_and_seeker_do_what_the_script_says` test checks the projectile
  and `Hit` events.

- `m3_ai_notice.lfr`: the WP-030 golden on `ai_yard`, recorded from
  `m3_ai_notice.script`. Three test grunts notice the player, close in, and attack it at
  most two at a time (the attack tokens). The player then sprints away, and the grunts
  leash, walk home, and heal. `m3_ai_notice_grunts_notice_attack_and_leash` checks the
  awareness events and the hits; `game/crates/core/src/ai/tests.rs` tests the rules.

- `m3_husk.lfr`: the WP-032 golden on `fight_husk`, from `m3_husk.script`. A Husk walks up
  and lands both swings of its combo (62 and 79 damage), and the player's light kills it.
  `m3_husk_lands_its_two_hit_combo_and_dies` checks it.
- `m3_husk_variants.lfr`: the WP-032 golden on `fight_husk_variants`, from
  `m3_husk_variants.script`. The Servant, Miner (pickaxe), and Frost Husk (Rime) each
  swing. `m3_husk_variants_each_swing` checks it.
- `m3_thorn_hounds.lfr`: the WP-032 golden on `fight_thorn_hounds`, from
  `m3_thorn_hounds.script`. A pack of three snaps and lunges at the player, who then kills
  them. `m3_thorn_hounds_bite_and_die` checks it.
- `m3_gray_knight.lfr`: the WP-032 golden on `fight_gray_knight`, from
  `m3_gray_knight.script`. The knight guards the player's lights (80% absorbed), then cuts
  with its two-hit combo. `m3_gray_knight_guards_then_cuts` checks it.
- `m3_gray_archer.lfr`: the WP-032 golden on `fight_gray_archer`, from
  `m3_gray_archer.script`. The archer on its platform looses arrows about every 90 ticks.
  `m3_gray_archer_shoots_on_a_cadence` checks it.
- `m3_whitewarden_revive.lfr`: the WP-032 golden on `fight_skeleton`, from
  `m3_whitewarden_revive.script`. The player kills a Whitewarden Skeleton, which lies down
  for 180 ticks and revives at half HP (WASM event kind 13). `m3_whitewarden_revive_gets_back_up`
  checks it.

Rehash only when a change to the simulation or the content data is intended. Review the
old and new final hash it prints, and commit the updated file with that change.
