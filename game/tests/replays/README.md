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
- `m3_boss_phase_reset.lfr`: the WP-033 golden on `boss_gate`, recorded from
  `m3_boss_phase_reset.script`. The player walks through the test boss's fog gate and
  waits out the 90-tick intro. Then they lock on and land three light hits, which take the
  boss under 60% HP: it plays the 45-tick phase 2 transition, during which it takes no
  damage. The player then sprints off the arena's edge and dies, and the boss resets to
  full HP and Dormant while the Remnant drops outside the gate.
  `the_golden_replay_changes_phase_and_resets_after_the_players_death` in
  `game/crates/core/src/ai/boss.rs` checks it; the rest of that file tests the rules.
- `m3_b01_bot_win.lfr`: the WP-034 golden on `boss_b01`, from `m3_b01_bot_win.script`. The
  scripted bot (`Bot` in `game/crates/core/src/ai/b01_tests.rs`) walks through the Hollow
  Sentry's fog gate, locks on, rolls through each hit, and punishes after the combo
  finisher until the boss falls (flag `boss.b01_hollow_sentry.defeated`) with the player
  alive. `the_bot_golden_replay_beats_b01_with_the_player_alive` checks it.
- `m3_b01_idle_dies.lfr`: the WP-034 golden on `boss_b01`, from `m3_b01_idle_dies.script`.
  The player walks in and does nothing, and the Sentry kills them.
  `the_idle_golden_replay_kills_the_player` checks it.
  Both `.script` files are generated by the ignored test `record_the_b01_goldens`; record
  the `.lfr` files with `lf-tools replay record-script --out <lfr> <script>`.

- `m3_acolyte`, `m3_drowned`, `m3_reed_stalker`, `m3_marsh_crawlers`, `m3_cave_moth`,
  `m3_lamp_mites`, `m3_shard_golem`, `m3_glasswing`: the WP-036 goldens, one per batch-2
  enemy, each from its `.script` on the matching `fight_*` scenario. The assertion tests
  are in `game/crates/tools/tests/golden_batch2.rs`. The special rules have their own unit
  tests in `game/crates/core/src/ai/batch2_tests.rs`.

- `wp051_dagger`, `wp051_greatsword`, `wp051_curved_sword`, `wp051_thrusting_sword`,
  `wp051_axe`, `wp051_hammer`: the WP-051 goldens, one per weapon class, each from its
  `.script` on the matching `wp051_*` scenario. The player wields the class's test weapon
  (`wpn_test_*` in `data/weapons/test_classes.ron`) against a padded dummy, and plays the
  three-hit light chain and then the heavy. `game/crates/tools/tests/wp051_movesets.rs` checks
  that each of those four swings hits the dummy, in order.
- `m5_spell_aether_dart`, `m5_spell_aether_bolt`, `m5_spell_homing_motes`,
  `m5_spell_aether_wave`, `m5_spell_mirror_ward`, `m5_spell_veil`,
  `m5_spell_gradient_descent`, `m5_spell_glasswing_swarm`, `m5_spell_unbinding_lance`: the
  WP-055 goldens, one per Formula, each from its `.script`. The player rests at the Lantern,
  levels Insight to the spell's requirement, attunes it with `cmd Equip 32 <spell number>`,
  and casts it on `spell_range` (the ward and the Veil on `spell_ward`, against a Husk). The
  assertion tests are in `game/crates/tools/tests/golden_spells.rs`; the rules have unit tests
  in `game/crates/core/src/combat/spells/tests.rs`.
- `m5_litany_mend`, `m5_litany_wide_mend`, `m5_litany_ember_toss`, `m5_litany_pyre_breath`,
  `m5_litany_storm_spear`, `m5_litany_radiant_edge`, `m5_litany_choir_litany`,
  `m5_litany_purge`, `m5_litany_steadfast`: the WP-056 goldens, one per Litany, each from its
  `.script`. The player rests at the Lantern, raises Devotion to 20, attunes the Litany with
  `cmd Equip 32 <spell number>`, and casts it on `litany_range` (Mend and Steadfast on
  `litany_ward`, against a Husk; Purge on `litany_hounds`, against thorn hounds; Wide Mend on
  `litany_echo`, where the Echo Bell is rung with `cmd UseItem 2`). The assertion tests are in
  `game/crates/tools/tests/golden_litanies.rs`; the rules have unit tests in
  `game/crates/core/src/combat/spells/tests.rs`.

- `m5_echo_helps_beat_test_boss.lfr`: the WP-059 golden on `echo_boss_gate`, recorded by the
  ignored test `record_the_echo_golden` (`game/crates/core/src/ai/echo/tests.rs`) because the
  script format has no `cmd` lines for `Equip`, `UseItem`, and `Gesture`. The scripted bot
  picks up the Echo Bell, equips it in a quick slot, walks through the test boss's fog gate to
  its Echo Stone, rings the Bell (summoning the PoliteLlama), bows, locks on, and fights. The
  Llama lands hits on the boss and the boss falls with the player alive.
  `the_golden_replay_has_an_echo_help_beat_the_test_boss` checks it, and
  `the_bot_beats_the_test_boss_faster_with_an_echo` checks the same bot is slower without the
  Echo.

- `wp052_colossal`, `wp052_spear`, `wp052_halberd`, `wp052_fist`, `wp052_bow`, `wp052_shield`,
  `wp052_catalyst`: the WP-052 goldens, one per remaining weapon class, each from its `.script`
  on the matching `wp052_*` scenario with the class's test weapon (`wpn_test_*`, registry
  18900-18906). Each plays the class's light chain and heavy; the colossal sword then
  two-hands (`TWO_HAND`) and swings again, the spear and the shield bash from a raised guard
  (`GUARD+LIGHT`), and the bow shoots free-aimed, locked on, drawn, and fully drawn.
  `game/crates/tools/tests/wp052_movesets.rs` checks the hits, in order, and the guard staying
  up through the spear's and shield's guard attacks.

Rehash only when a change to the simulation or the content data is intended. Review the
old and new final hash it prints, and commit the updated file with that change.
