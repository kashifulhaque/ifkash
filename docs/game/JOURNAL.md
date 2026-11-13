# Journal

This file is the handoff log between sessions. Every orchestrator (or single-WP)
session adds one entry and keeps it current while it works. The newest entry goes at
the top. Never edit an earlier session's entry; correct it in your own.

[`STATUS.md`](STATUS.md) says *what* state each WP is in. This journal says *who* did
it, *how*, at what context cost, and *what the next session must do first*.

## Entry template

Copy this block to the top of the entries and fill it in as you go:

```markdown
### YYYY-MM-DD: SESSION_NAME (TARGET)

- **Orchestrator:** model and effort; session ID if the app shows one
- **Branch:** session branch name; first and last commit pushed to `main`
- **Started from:** `main` at COMMIT, or the previous session's branch
- **WPs:**
  - WP-NNN: done by WORKER (escalated from ...?); commit SHA; worker tokens
  - WP-NNN: blocked: reason
  - WP-051: haiku, worktree; `b2cc543`; 346K. Orchestrator regenerated `gen/` types for WP-050 `level_cost` (WP-050 missed `gen-ts`).
  - WP-057: sonnet, worktree; `80dd69f`; 366K. Merge onto WP-051: registry keep-both, goldens rehashed (45, all A -> A).
  - WP-055: sonnet, worktree; `fa79fac`; 371K.
  - WP-059: sonnet, worktree; `342753c`; 406K.
  - Merge of WP-055 and WP-059 plus integration commit `bbb46e1` (gear refresh after level-up and respec; Scholar's Seal slot; Shrike-Lite startup): sonnet merge worker in the main checkout; 154K. Every golden A -> A.
  - WP-052: sonnet, worktree; `abc2479`; 261K. Orchestrator merged it onto WP-055 and WP-059 (4 code conflicts) and fixed the test catalyst's missing `catalyst_school`.
  - WP-056: haiku, worktree; `12fb40d`; 395K. Clean cherry-pick; 71 goldens A -> A.
  - WP-053: haiku, worktree; `3903440`; 377K. Behavior change: seven `m5_spell_*` goldens re-recorded (stylus now at spec scaling and catalyst moveset).
  - WP-058: sonnet, worktree; `472c995`; 278K. Orchestrator opened the armory (armor, catalysts) on port 5180; it renders. Only console errors: the pre-existing `m0_flat.json` 404.
  - WP-054: haiku, worktree; `bdd3384`; 394K. Behavior change: `m0_random_3000` re-recorded (random inputs press the new `ART` button). Orchestrator ran `cargo fmt` on three files. The worker's stale `lf-tools` binary briefly rewrote WP-059's finished worktree; that worktree shows no changes.
- **Context:** orchestrator NNK; each worker NNK (budget 500K per context window)
- **Not verified:** anything the session couldn't check
- **Waiting on the owner:** decisions, credentials, playtests, merges
- **Next steps:** what the next session does first, in order
```

## Entries

### 2026-11-13: latticefall-m8 (M8)

- **Orchestrator:** Opus 5.5, low effort
- **Branch:** `game/m8-launch`
- **Started from:** `origin/main` at `3e64199`
- **Gates:** G2 and G3 are still open. The owner asked for M8 anyway, as in M5-M7.
  WP-086 (Launch) waits for the owner's go-ahead before it's pushed, because it
  replaces the live `/game`.
- **Reservations:** registry numbers: WP-081 42000-42999, WP-082 43000-43999,
  WP-083 44000-44999, WP-084 45000-45999, WP-085 46000-46999, WP-086 47000-47999,
  WP-087 48000-48999, WP-088 49000-49999, WP-089 50000-50999, WP-100 51000-51999,
  WP-101 52000-52999. WASM event kinds: WP-081 52, WP-082 53, WP-083 54, WP-084 55,
  WP-087 56, WP-088 57-58, WP-100 59, WP-101 60. `MechanicEvent` sub-kinds: WP-101
  220-229, WP-088 230-239. HUD prompt codes: WP-082 82-83, WP-083 84-85, WP-084 86-87,
  WP-088 88-89, WP-100 90-91.
- **Plan:** the M7 journal's boss follow-ups become WP-100 (render) and WP-101 (sim).
  Wave 1: WP-081, 084, 085, 101. Wave 2: WP-082, 083, 087, 089 (after WP-101, which
  re-records boss goldens). Wave 3: WP-088 (after WP-082; both edit `ui/`), WP-100
  (after WP-081; both edit `render/`). Wave 4: WP-086. At most four workers at once.
- **WPs:**
  - WP-085: haiku, worktree; `e5eb911`; 158K. Clean cherry-pick. Orchestrator re-ran `bun run check` and `bun run build` (15 codex pages prerendered) and stopped two `workerd` processes left in the worker's worktree.
  - WP-084: sonnet, worktree; `b604f05`; 163K. Clean cherry-pick. Orchestrator applied migration 0013 locally and smoke-tested the Worker on port 8799 (reads 200, writes 401 without a token). The worker ran `pkill -f workerd` late in its run, which may have hit another worker's Worker.
  - Started WP-083 (sonnet) and WP-087 (haiku) while WP-081 and WP-101 run.
  - WP-083: sonnet, worktree; `3e20d7f`; 142K. Clean cherry-pick. Orchestrator re-ran `bun run check` and took screenshots at 375 x 812 on port 5180 (phone view, touch mode).
  - Started WP-088 early (it was wave 3), moved up from haiku to sonnet: it adds Marks to the simulation state, gestures to the input frame, and UI. WP-082 now waits for both WP-081 and WP-088. Orchestrator context 150K.
  - WP-081: sonnet, worktree; `32233fd`; 296K. Clean cherry-pick; no Rust changes. Orchestrator re-ran `bun run check` and the benchmark for the Verge and the castle (matches the report) and took a gameplay screenshot.
  - WP-101: sonnet, worktree; `d5f15c0`; 171K. Clean cherry-pick. Deliberate: `m6_bound_ring`, `m7_b11_bot_win`, `m7_b11_idle_dies`. Orchestrator fixed a stale B11 comment in `scenarios/boss_b11.ron` (content hash unchanged).
  - WP-087: haiku, worktree; `cf78346`; 242K. Clean cherry-pick. Merged tree with WP-101: 877 tests, clippy, fmt, validate, 120 goldens A -> A, `bun run check` clean, `lf-tools credits` writes 3 packs.
### 2026-11-13: latticefall-m7 (M7)

- **Orchestrator:** Opus 5.5, low effort
- **Branch:** `game/m7-bosses`; first commit on `main` `b21a925`, last `09e0922` plus this close-out
- **Started from:** `origin/main` at `2fe6615`
- **Gate G2:** still open. The owner asked for M7 anyway.
- **Reservations:** registry numbers: WP-070 31000-31999, WP-071 32000-32999,
  WP-072 33000-33999, WP-073 34000-34999, WP-074 35000-35999, WP-075 36000-36999,
  WP-076 37000-37999, WP-077 38000-38999, WP-078 39000-39999, WP-079 40000-40999,
  WP-080 41000-41999. WASM event kinds: WP-070 40, WP-071 41, WP-072 42, WP-073 43,
  WP-074 44, WP-075 45, WP-076 46, WP-077 47, WP-078 48, WP-079 49, WP-080 50-51.
  `MechanicEvent` sub-kinds: WP-070 110-119, then 10 per WP in card order up to
  WP-080 210-219. HUD prompt codes: WP-070 60-61, then 2 per WP in card order up to
  WP-080 80-81.
- **Plan:** wave 1 is WP-070, 071, 072, 073, 074, and 076, each in its own worktree.
  Wave 2 is WP-075 (after WP-071: both edit `levels/castle/entities.ron`), 077, 078,
  and 079. Wave 3 is WP-080. WP-071 moved up from haiku to sonnet: B01 needed the same
  move, and every boss WP needs a scripted bot that wins.
- **WPs:**
  - WP-071: sonnet (moved up from haiku), worktree; `63961ab`; 295K. Clean cherry-pick; 100 goldens A -> A on the merged tree.
  - WP-076: sonnet, worktree; `2f57de8`; 463K. Merged onto WP-071 (keep-both). Deliberate: `m6_cinderdeep_mine` (boss placed), `m4_mechanics_yard` (new `MechState` field). No boss screenshots: the worker ran out of budget, and the orchestrator only confirmed the scenario loads.
  - WP-074: sonnet, worktree; `42a1ed2`; 426K. Merged onto WP-076 with one boss-mechanic shape: the `BossDef.params` map (B06's `mechanics` renamed) and both hooks in `ai/bosses/mod.rs` (`run` for the world, `tick` per fight). Deliberate: `m6_mirrormere` (boss and water), `m7_b06_bot_win` (WP-076's fix now pays the Choir Glaive as gear).
  - WP-072: sonnet, worktree; `7babbde`; 470K (its own count 650K). Merged by a sonnet merge worker in the main checkout (174K for the three merges). Deliberate: `m6_mossgut_cave`.
  - WP-070: sonnet, worktree; `b55759e`; 580K, over budget. Merged by the merge worker. Deliberate: `m4_verge` (boss placed, two Husks removed), `m7_b02_bot_win` (weapon rewards paid as gear). Keeps a `before_locomotion` hook for the patch slow.
  - WP-073: sonnet, worktree; `ef355d8`; 514K, over budget. Merged by the merge worker, who added an `after` hook (B05 must run after `boss::run`). Deliberate: `m6_sunken_ossuary`, `m7_b05_idle_dies` (one-tick shift from the new hook). Orchestrator re-checked the merged tree: 110 goldens A -> A, 766 tests pass, clippy, fmt, validate, and `bun run check` clean.
  - Wave 2, all merged by the same sonnet merge worker (262K for four merges) and re-checked by the orchestrator on the final tree (118 goldens A -> A, 853 tests, clippy, fmt, validate, `bun run check` clean):
    - WP-075: sonnet, worktree; `912399b`; 443K. Deliberate: `m6_castle_part1`, `m6_castle_part2`.
    - WP-077: sonnet, worktree; `a2fece1`; 393K. Merge moved its state to `World::b09`. Deliberate: `m6_hollowdeep`.
    - WP-078: sonnet, worktree; `11950b2`; 432K. Deliberate: `m6_rimeshelf`.
    - WP-079: sonnet, worktree; `5945b80`; 424K. Merge swapped the glyph rings to match the spec. Deliberate: `m6_bound_ring` (re-recorded).
  - Wave 1 ran six workers at once; machine load reached 37 and three workers ended at or over the 500K budget (WP-070 580K, WP-073 514K, WP-072 470K by the tool, 650K by its own count). Wave 2 runs four.
  - WP-080: opus, worktree; `09e0922`; 393K. Clean cherry-pick. Deliberate: `m6_spire` (re-recorded), `m6_castle_part1`/`part2` (new 3 m fog-gate height limit in `boss.rs`). Merged tree: 120 goldens A -> A, 869 tests.
  - Worktrees: all eleven worker worktrees removed after their WPs were pushed.
- **Context:** orchestrator about 290K at the end (budget 500K per context window). Workers 295K-580K; three went over 500K in wave 1 (six at once).
- **Lessons:**
  - Six parallel boss workers pushed load to 37 and three over budget; four at once went cleanly.
  - Parallel workers each invent the same shared scaffold (here `ai/bosses/mod.rs`, a per-boss number map, and hook call sites). Merge the first two into one shape early and put it in the brief before more workers start.
  - A sonnet merge worker in the main checkout handled three and four cherry-picks for 174K and 262K, sparing the orchestrator's context; the orchestrator re-ran the full checks on the final tree.
  - Per-boss state in `MechState` moves `m4_mechanics_yard` and `m6_cinderdeep_mine`; put it in a `World::bNN` field hashed only during its fight.
  - Worker dev servers in worktrees 403 on symlinked `node_modules`; `bun install` in the worktree fixes it.
- **Not verified:**
  - Any boss in play by a human: every fairness check, tuning, and fight length. Bots roll through every telegraph.
  - On screen: B08 and B09 have no boss screenshots; most bosses have no live-play shots of later phases or the defeat (the hidden Browser pane runs at about 4-5 ticks a second).
  - Hazards with no visuals: B08's lava, B10's toll shockwave and ice, B09's lit lamps (fog lifts, no flame), B04's weak point (always glowing), B12's loom lines (code exists, unseen).
  - A full playthrough (gate G3).
- **Waiting on the owner:**
  - **Gate G2** is still open (from M4). **Gate G3** (content complete, a full playthrough) is next.
  - Placement deviations: B05 Keeper at z 41.5 (the placeholder point was inside the dais); B11 on the plinth at y 0.5, blink stones 8 points on 15 m (the level has 12 stones at 42 m); B02's arena lost two Husks.
  - B11: the seal lifts when the player is 30 m away with no fight (not in the notes). B12: the Weaver's defeat flag is the ending trigger; Echoes summoned in phase 2 are felled, losing their FP.
  - B10's state is `#[serde(skip)]`: a mid-fight save would restart the beat.
  - Six validator warnings for B11's image moves (no hitbox, intended).
- **Next steps:**
  1. Owner: gate G2, then play every boss for gate G3 and the decisions above.
  2. Follow-ups found in M7 (one sonnet WP each, or fold into M8): draw the boss hazards (B08 lava, B10 toll and ice, B09 lamp flames, B04 weak point state, B12 loom lines verified on screen); reset B02 thorns and B08 ember veins on a fight reset; the B03 bridge's broken section; B02's intro camera move; B11's stones to match the spec or the spec to the level; B12's ending UI for the Courier's Letter. The M6 follow-ups still stand.
  3. Then M8 (WP-081 onward). Free after M7: registry 42000 and up; WASM kinds 10, 12, 18-20, 22-30, 34-39, 41-49, 51 and up; `MechanicEvent` sub-kinds 220 and up; HUD prompt codes 57-59 and 82 and up.

### 2026-11-12: latticefall-m6 (M6)

- **Orchestrator:** Opus 5.5, low effort
- **Branch:** `game/m6-world`; first commit on `main` `dba21d4`, last `b555f9e` plus this close-out
- **Started from:** `origin/main` at `3680331`
- **Gate G2:** still open. The owner asked for M6 anyway.
- **Reservations:** registry numbers: WP-060 21000-21999, WP-061 22000-22999,
  WP-062 23000-23999, WP-063 24000-24999, WP-064 25000-25999, WP-065 26000-26999,
  WP-066 27000-27999, WP-067 28000-28999, WP-068 29000-29999, WP-069 30000-30499.
  WASM event kinds: WP-060 27, WP-062 28, WP-064 29, WP-067 30, WP-068 31-33,
  WP-061 34, WP-063 35, WP-065 36, WP-066 37, WP-069 38.
- **Plan:** wave 1 is WP-060 and WP-068. WP-060 also adds Echo Stones to the level
  format (an M5 follow-up), which every later region WP uses, so the other region WPs
  wait for it. New world mechanics go in their own files under
  `world_state/mechanics/`; `mechanics.rs`, `world_state/mod.rs`, `lib.rs`,
  `registry.ron`, and the WASM event list take append-only edits.
- **WPs:**
  - WP-068: sonnet, worktree; `f64edc9`; 470K. Clean cherry-pick. Orchestrator checked tests, lint, validate, `bun run check`, and the hub loads.
  - WP-060: sonnet, worktree; `fbf71cf`; 331K. Orchestrator merged it onto WP-068 (keep-both in `level.rs`, content test, spawns, props index), ran `gen-ts` and `cargo fmt`, rehashed goldens (86 lines, all A -> A). Screenshots: L09 causeway and Cathedral; L06 landmark with a fixed camera (the follow camera pitches down and hides the tower).
  - WP-061: sonnet, worktree; `fdd72b8`; 330K. Clean cherry-pick. Its `MechanicEvent` sub-kinds 30-39 aren't WASM kinds; the orchestrator gave running workers sub-kind ranges (WP-062 40-49, WP-064 50-59, WP-067 60-69).
  - WP-064: sonnet, worktree; `c4e2ba7`; 326K. Merged onto WP-061 (keep-both: embedded levels, content test, registry, spawns).
  - WP-062: sonnet, worktree; `d0ad877`; 442K. Merged onto WP-064. Deliberate rehash: `m4_mechanics_yard` (new `MechState` fields are hashed once its lift moves). The merge left level manifests stale; the orchestrator rebaked all of them.
  - WP-067: sonnet, worktree; `2e2a678`; 415K. Merged onto WP-062; removed a duplicate `SpawnPoint.y` (WP-064 and WP-067 both added it).
  - Wave 2 screenshots by the orchestrator (dev WASM, port 5180): Belfry Tide Hall, castle bridge deck, Hollowdeep L11, Spire Gate; no console errors.
  - WP-063: sonnet, worktree; `1024fe4`; 514K, **over the 500K budget** (it finished and committed). Clean cherry-pick; orchestrator rehashed after the worker's late elite-stat change (94 lines, A -> A).
  - WP-065: sonnet, worktree; `9bd1f3e`; 391K. Worker said `m6_castle_part1` was unchanged; it wasn't (its part-1 geometry edits move the hash). The orchestrator proved it by patching the content hash in the worker's own recording and replaying it OK on the merged code. Deliberate.
  - WP-069: sonnet, worktree; `8bf7ac9`; 289K. Merged last so WP-063 and WP-065 could be checked A -> A first. Deliberate rehash of 55 goldens (visited-cell flags). Map checked in the browser (opens with M; labelled tablist).
  - WP-066: sonnet, worktree; `b555f9e`; 432K. Clean cherry-pick; 98 goldens A -> A. Worker screenshots only.
  - Owner request mid-session: removed 34 leftover worktrees from M0-M6 (none had uncommitted work; branches kept), freeing about 125 GB. The playbook now has [clean up worktrees](11-agent-playbook.md#clean-up-worktrees) (`7107f3a`).
- **Context:** orchestrator about 300K at the end (budget 500K per context window). Largest worker: WP-063 at 514K, over budget; it finished and committed.
- **Lessons:**
  - Workers misreport golden changes (WP-065 called a moved hash unchanged). Always rehash after a merge and count the `A != B` lines yourself. To find which side moved a hash, patch the content hash in a worker's own `.lfr` (a LEB128 varint after the version byte) and `replay run` it on the merged code.
  - Merge a WP that deliberately re-records many goldens (WP-069) last in its wave, so the others can be checked A -> A first.
  - Merges that change shared level types leave baked manifests stale; rebake every area after such a merge (the bake test catches it).
  - Two workers added the same optional field (`SpawnPoint.y`); a keep-both merge duplicated it. Read merged type declarations, not only the conflict markers.
  - Give each worker its own ranges for `MechanicEvent` sub-kinds and HUD prompt codes, not only WASM event kinds.
  - Each worker worktree is 3-6 GB; remove it once its WP is pushed.
- **Not verified:**
  - Any region in play: feel, pacing, combat in the new areas. Fights the bots avoid: the castle ward, the Spire halls, the foundry Smiths.
  - On screen: drained water, moving carts and lifts, the trapdoor, mirrors on sockets, lit braziers, the sealed sigil.
  - A cross-area trip in the browser (Rust tests only).
  - Walking between areas: `area.ron` connections are never read, so Lanterns (WP-069) are the only way to change area.
- **Waiting on the owner:**
  - **Gate G2** is still open (from M4).
  - Spire: the gauntlet halls are optional (behind one-way doors); force them? `levels/spire/area.ron` grants two Great Sigils until saves carry inventory.
  - A cross-area trip is refused while a Remnant is unrecovered (WP-069).
  - The castle's rampart lift loop from L16 to L15 (49.4 s) isn't faster than the main route.
  - Chronicle Page 4 and 5 slots need a fifth work entry in the portfolio, or a different use.
- **Next steps:**
  1. Owner: gate G2 and the decisions above.
  2. M7 (bosses WP-070 to WP-080). Every dependency is done except WP-080's other bosses. Each boss WP replaces its arena's commented `Boss` placeholder and fog gate: B03 `castle`, B04 `mossgut_cave`, B05 `sunken_ossuary`, B06 `mirrormere`, B07 `castle` (also raise `door.gate_castle_back.open` on defeat), B08 `cinderdeep_mine`, B09 `hollowdeep`, B10 `rimeshelf`, B11 `bound_ring` (also set the sigil `armed: true`), B12 `spire`. WP-071 and WP-075 both edit `levels/castle/entities.ron`; don't run them at the same time.
  3. Follow-ups found in M6 (one sonnet WP each, or fold into M8): walking between areas (read `area.ron` connections, including the chimney drop and the Belfry and Mossgut lifts); open the Verge's `mossgut_plug` and connect it to Mossgut; a castle connection back into Rimeshelf; the follow camera clipping low ceilings; darkness affecting enemies; remove the Spire's Great Sigil grant; a "ring the bell" prompt; Mirrormere's sound bed; the Mirror's visual effect; a nav-grid link for jumps and ladders (the Spire's L23 bake warning).
  4. Registry blocks used in M6: 21000-30499. WASM event kinds used: 31-33 (WP-068). Free WASM kinds: 10, 12, 18-20, 22-30, 34 and up. `MechanicEvent` sub-kinds used: 30-39, 40-43, 70-72, 90-92, 100-103. HUD prompt codes used: 8, 9, 30-32, 40, 55, 56.

### 2026-11-12: latticefall-m5 (M5)

- **Orchestrator:** Opus 5.5, low effort
- **Branch:** `game/m5-builds`; first commit on `main` `a33ad3c`, last `e7769a9` plus this close-out
- **Started from:** `origin/main` at `d1c264a`
- **Gate G2:** still open. The owner asked for M5 anyway; M5 adds build content, which
  G2's tuning answers don't block.
- **Reservations:** registry numbers: WP-050 14000-14099, WP-051 13000-13999,
  WP-055 15000-15499 (catalyst weapons 1100-1149), WP-057 16000-16999,
  WP-059 17000-17499, WP-052 18000-18999, WP-053 19000-19999, WP-056 15020-15499 (sub-blocks per WP-055). WASM event kinds: WP-052 23, WP-053 24, WP-056 25, WP-055 18-19, WP-057 20, WP-059 21-22;
  WP-050 and WP-051 add none.
- **WPs:**
  - WP-050: haiku, worktree; `cbccd54`; 189K. Touched `wasm/src/menu.rs` (signature change). Goldens rehashed (content hash only).
  - M5 exit gate: sonnet, main checkout; `e7769a9`; 153K. `tools/tests/m5_builds.rs`: all nine builds equip, temper, and attune through commands (10 pass, 1 ignored: Scholar's nine Formulae need 11 slots, the most is 9).
- **Context:** orchestrator about 290K at the end (budget 500K per context window). Largest worker: WP-059 at 406K.
- **Lessons:**
  - Workers in worktrees can't run `mise run` (untrusted config) or `bun run check` cleanly (no WASM glue); verify both in the main checkout after each merge.
  - Workers skipped `cargo fmt` (WP-054) and `gen-ts` (WP-050); run both after every merge.
  - Two content rules bit merges: a Catalyst needs `catalyst_school` (WP-055), and the first weapon file in load order is the player's default.
  - In zsh, `R="mise exec -- ..."; $R cargo ...` doesn't word-split; a failed rehash then looks like "0 changed". Count the `final hash` lines.
- **Not verified:**
  - Any build in play: feel, balance, and equip load (equipping doesn't enforce load). That's WP-089.
  - The browser can't send `Respec`, `Upgrade`, or `Temper` yet (menu encoding), and menu equipping of the new gear; WP-058's armory was checked by screenshots only.
  - Weapons during attack animations; mobile; spell VFX from real casts (combat events carry no spell element).
- **Owner decision:** the game is named *Cinderheart* (ADR 0007); public-facing name only; internal code keeps `latticefall` by the owner's choice. Docs prose and the game page's title and messages renamed.
- **Waiting on the owner:**
  - **Gate G2** is still open (from M4).
  - Two deliberate golden behavior changes to accept: seven `m5_spell_*` goldens (stylus at spec scaling, WP-053) and `m0_random_3000` (random inputs press the new `ART` button, WP-054).
  - Builds table readings: does Scholar's "All Formulae" mean all attuned at once (impossible: 11 slots, max 9) or swapped at Lanterns? Colossus lists Rallying Roar, which none of its weapons carry by default.
- **Next steps:**
  1. Owner: gate G2 and the decisions above.
  2. Small follow-ups found in M5 (fold into M6 or M8): a menu encoding for `Respec`, `Upgrade`, and `Temper`; spell loadout and weapon upgrades in `SaveData`; renderer spell events; weapon and temper status buildup on hits; per-class crit moves; the Archivist free respec; Echo Stones in the level format (WP-060); a class picker (respec floor uses the stub loadout).
  3. Then M6 (WP-060 onward). Reserved this session: registry 13000-20499 blocks; WASM kinds 18-26 (used: 21). Free WASM kinds: 10, 12, 18-20, 22-26 unless a WP claimed them, and 27 and up.

### 2026-11-12: latticefall-m4 (M4)

- **Orchestrator:** Opus 5.5, low effort
- **Branch:** `game/m4-slice`; first commit on `main` `0b4815b`, last WP commit `518bac3`
- **Started from:** `origin/main` at `54850f0`
- **Reservations:** registry numbers: WP-040 10000-10499, WP-042 10500-10999 (items
  4500-4599), WP-043 11000-11199, WP-044 11200-11499, WP-049 12000-12099. WASM event
  kinds: WP-042 14-16, WP-049 17; WP-047 and WP-035.a add none.
- **Added cards:** WP-030.a (the M3 journal's `AiState.def` fix) and WP-035.a (models for
  enemy batches 2 and 3 and the elites).
- **WPs:**
  - WP-030.a: sonnet; `c2bae96`; 98K. All 27 m3 goldens rehashed once.
  - WP-048: sonnet, worktree; `185f063`; 108K. Not listened to; only hits make sound so far.
  - WP-047: sonnet, worktree; `a241b74`; 133K. Autosave from a real rest not exercised in play.
  - WP-045: sonnet, worktree; `f438ef1`; 200K. Merged onto WP-047 by the orchestrator (2 conflicts: WASM struct, engine loop).
  - WP-035.a: sonnet, worktree; `2ec2d05`; 281K. No WASM or simulation change.
  - WP-040: opus, worktree; `02ec373`; 248K. Golden conflict with WP-030.a resolved by one rehash (content hash only).
  - WP-049: sonnet, worktree; `fd0287e`; 234K. Orchestrator gated the landing card behind `engine=lf`.
  - WP-041: sonnet, worktree; `5ec93b7`; 172K.
  - WP-046: sonnet, worktree; `7f8d8a7`; 335K. Edited `world_state/` (LevelUp, Travel) while WP-042 held it; WP-042 rebases onto it.
  - WP-042: sonnet, worktree; `12ca046`; 363K. Its report named a test that doesn't exist under that name; the equivalent core test does.
  - WP-043: sonnet, main checkout; `870e5d3`; 241K.
  - WP-044: sonnet, main checkout; `518bac3`; 220K. Greybox only; the card allows a separate art pass.
- **Context:** orchestrator about 310K at the end (budget 500K per context window). Largest worker: WP-042 at 363K.
- **Lessons:**
  - Agent worktrees branch from `origin/main`, not the session branch, so push each
    verified WP before starting the next worker, and tell running workers to rebase
    onto the session branch before they commit. That kept every cherry-pick clean
    except two (WP-045's WASM struct and engine loop; WP-040's goldens).
  - The Browser pane's `preview_start` only reads the main checkout's `launch.json`.
    Worktree workers ran Vite on their own ports and opened them by URL.
  - A golden that only re-records content hashes after a merge is safe to rehash once;
    compare `final hash A -> B` lines from `replay rehash` to prove no state hash moved.
  - Workers' reports can name tests wrongly (WP-042); grep for the test before trusting it.
  - The landing card (WP-049) would have changed the public `/game` page; the
    orchestrator gated it behind `engine=lf`. Check every UI WP for that.
- **Not verified:**
  - Any real playthrough: combat feel, pacing, whether the tutorial teaches without
    outside help, and whether the Verge's routes feel right. That's gate G2.
  - Real gamepad and screen reader in the menus; audio by ear; the low and high
    quality presets; mobile.
  - Fast 4G load time of the landing card (measured locally only).
- **Waiting on the owner:**
  - **Gate G2:** play Waking Hollow and the Verge at `/game?engine=lf` (Play on the
    landing card; `&scenario=waking_hollow` or `&scenario=verge`). The levels aren't
    linked yet: no area transition joins the cave to the Verge.
  - Waking Hollow has one Lantern about 98 m from B01's fog gate; the spec wants one
    within 20 s. Add a Lantern (changes the Lantern table) or accept the run.
  - The starting blade kills Husks in one or two hits; the bot took no damage
    through the cave. Tune now or in WP-089 (with B01 from M3).
  - WP-040 readings recorded in `01-architecture.md` Levels without an ADR: levels are
    compiled at content load, kit colliders are boxes only, Lanterns live only in
    `entities.ron`.
  - Settings saved but not read yet: difficulty, graphics quality in the sim,
    color-blind mode, subtitles, reduced motion (WP-046).
- **Next steps:**
  1. Owner: gate G2 playtest and its decisions.
  2. Small follow-ups found in M4 (one sonnet WP each, or fold into M5/M6):
     an area transition from Waking Hollow's root-door to the Verge (and the
     root-door kind that opens after B01); a Lantern model; the Verge art pass; menu
     load and save; reading the saved difficulty, quality, and accessibility settings.
  3. Then M5 (WP-050 onward). Keep reserving registry blocks and event kinds per WP.
     Used this session: 14-17. Free WASM kinds: 10, 12, and 18 and up.

### 2026-11-12: latticefall-m3 (M3)

- **Orchestrator:** Opus 5.5, low effort
- **Branch:** `game/m3-enemies`; first commit on `main` `8836362`, last WP commit `c40bdfb`
- **Started from:** `origin/main` at `4a41d3e`
- **Reservations (to avoid M2's merge clashes):** registry numbers: enemies 7000-7099,
  WP-030 test enemies 7100-7199, enemy moves WP-030 8900-8999, WP-032 8000-8299,
  WP-036 8300-8599, WP-037 8600-8899; bosses 9000-9099, WP-033 dummy boss moves
  9200-9299, B01 moves 9100-9199. WASM event kinds: WP-030 9-10, WP-033 11-12, WP-032
  13, WP-036 14-15, WP-037 16.
- **WPs:**
  - WP-030: opus; 264K worker tokens (reported 190K).
  - WP-031: sonnet, worktree; 139K. Cherry-picked onto WP-030 by the orchestrator, no conflicts.
  - WP-032: sonnet; `93f07b7`; 313K.
  - WP-033: sonnet, worktree; `cc074cd`; 295K. Merged onto WP-032 by the orchestrator (18 conflicts, keep-both, then a rehash; only the boss golden's final hash moved).
  - WP-034: sonnet (moved up from haiku: first real boss on a new framework, plus a scripted bot), worktree; `35c7083`; 169K.
  - WP-035: sonnet; `81fd4c0`, plus the orchestrator's fix `fb4d1e4` (B01 keyed to its body's number 9011); 194K.
  - WP-039: opus; `f3519e0`; 216K.
  - WP-036: sonnet, worktree; `850590a`, `0919903`; 320K, including a follow-up after the orchestrator sent it back once for the missing Bell Acolyte.
  - WP-037: sonnet (moved up from haiku: wolf packs and the mimic need new AI code outside its Touch list), worktree; `c40bdfb`; 252K.
  - Merge of WP-036 and WP-037: sonnet merge worker in a worktree; 100K. No conflicts when the result was cherry-picked onto WP-039.
- **Context:** orchestrator 173K after WP-033, about 215K after WP-035, about 275K at the end (budget 500K per context window). Largest worker: WP-036 at 320K.
- **Lessons:**
  - `AiState.def` stores an index into the enemy table, not the registry number, so
    adding any enemy shifts it and changes golden hashes of unrelated fights (it
    happened three times this session). Store the registry number instead.
  - Reserving registry blocks and WASM event kinds per WP worked: no number clashes.
  - A merge worker in its own worktree, while another worker held the main checkout,
    kept the orchestrator's context low (one merge took the orchestrator about 20K
    by hand, the delegated one about 5K).
- **Not verified:**
  - The 30-minute PPO run itself (the orchestrator re-ran 1 minute).
  - Browser checks for batches 2 and 3 and the elites: they have no models yet.
    The renderer doesn't read the new `CLOAKED` and `HIDDEN` flags.
  - Combat feel of any enemy, and enemy strike alignment against the hitbox overlay.
  - The M3 exit gate is met by WP-039's numbers, but B01 is beaten by guarding and
    heavy attacks without dodging.
- **Waiting on the owner (spec readings the workers chose):**
  - B01 tuning: PPO and the scripted bot both win easily; the fight doesn't force
    rolling (WP-034, WP-039). Decide whether to tune it now or in WP-089.
  - ADR 0006 (Python extras `env` and `train`: gymnasium, numpy, torch) and its spec
    edits to `09-rl-environment.md` (four move-timing slots on `target`, button rules).
  - Enemy readings: the hearing model and return-and-heal (WP-030), the revive rule's
    lit-Lantern stand-in for braziers and unlimited revives (WP-032), boss phase
    numbering and runtime tracking stop (WP-033), the Lamp Mite fuse split and the
    thief not stealing (WP-036, WP-037), and elite approximations (WP-037).
- **Next steps:**
  1. Make `AiState.def` a registry number (one small core WP or a fix at the start of
     M4), then rehash the goldens once.
  2. Start M4: WP-040 (opus, level format) first; it must take over scenario-data
     Lanterns, water and reed zones, and fog gates. Then WP-041, WP-042, WP-045,
     WP-047, WP-048 in parallel where Touch lists allow.
  3. Procedural models for enemy batches 2 and 3 and the elites have no card yet;
     add one to the roadmap (render lane) or fold it into WP-041 or WP-058.
  4. Keep reserving registry blocks and event kinds per WP. Free WASM kinds: 10, 12,
     14, 15, 16 (reserved this session but unused), and 17 and up.

### 2026-11-12: latticefall-m2 (M2)

- **Orchestrator:** Opus 5.5, low effort
- **Branch:** `game/m2-combat`; first commit on `main` `ddf64a7`, last WP commit `da1fba4`
- **Started from:** `origin/main` at `1e61e40`
- **WPs:** all ten M2 cards are done. No worker failed verification, so nothing was escalated.
  - WP-020: opus; `9858e66`; 181K worker tokens. The orchestrator regenerated
    `gen/Moveset.ts`.
  - WP-022: sonnet; `398952c`; 160K.
  - WP-021: opus, worktree; `5e4f88b`; 181K. Rebased onto WP-022 by the orchestrator.
  - WP-026: sonnet; `2964b82`; 180K. Rebased onto WP-021 by the orchestrator.
  - WP-023: sonnet; `0e40d33`; 190K. Also wired WP-022's guard into WP-021's resolver.
  - WP-029: sonnet, worktree; `f1922f8`; 211K. Rebased onto WP-023 by the orchestrator.
  - WP-028: sonnet, worktree; `29d6863` and `9c3b04f`; 226K. The orchestrator replaced
    its guessed event-length decoder with a per-kind table and took the browser check.
  - WP-024: sonnet; `8bfb854`; 230K. The orchestrator merged it onto WP-029 (13
    conflicts; WASM event kinds renumbered to 3-6).
  - WP-027: sonnet, worktree; `25526c3`; 303K, plus 115K for a sonnet merge worker.
  - WP-025: sonnet; `da1fba4`; 225K, plus 111K for a sonnet merge worker.
- **Context:** orchestrator about 320K at the end (budget 500K per context window, per
  the owner's clarification this session). Largest worker: WP-027 at 303K.
- **Lessons:** parallel workers on the combat lane all edit `resolve.rs`, `actor/mod.rs`,
  `world.rs`, the WASM encoder, `registry.ron`, and the content crate. Merges cost
  15-30K each and twice hit registry-number clashes and WASM event-kind clashes. A
  keep-both merge also nested new fields inside the wrong struct several times. Next
  time, reserve registry blocks and event kinds per WP in the brief.
- **Event kinds (WASM `drain_events`, mirrored in `engine/events.ts` `EVENT_WORDS`):**
  1 Hit 17 words, 2 Life 6, 3 Stagger 4, 4 StanceBreak 3, 5 Parry 4, 6 Critical 5,
  7 StatusProc 4, 8 Projectile 10. The stream has no length prefix, so a new kind must
  be added to `EVENT_WORDS` in the same change.
- **Not verified:**
  - The M2 exit gate, "every formula in the combat spec has a passing test", wasn't
    audited formula by formula. Each card's acceptance tests pass.
  - Combat feel in the browser beyond one VFX hit check. Projectiles aren't drawn
    (no WASM export or TS reader for `ProjectileSnap`).
  - Real `prefers-reduced-motion` emulation (the setter it calls was tested).
- **Waiting on the owner (spec readings the workers chose):**
  - Guard break: stamina pays its share of the hit and the rest lands unblocked
    (WP-022). Blocks work from any side.
  - Parry costs no stamina and a missed parry has no extra recovery; only the test
    parrying dagger can parry (WP-024).
  - Status buildup: backstab +20%, riposte +30% (the spec gives both numbers); blocked
    hits add none; projectile hits get weapon-buff buildup but no hitbox buildup (WP-025).
  - Dummies use flat defense 500 so a light does 29 (WP-023 tuning).
- **Next steps:**
  1. Optional: audit the M2 exit gate against `03-combat.md` and add any missing formula
     tests (one sonnet worker).
  2. Start M3 with WP-030 (opus, AI framework), then WP-031 (navigation grid) in
     parallel. WP-030 replaces the scripted dummy hook (`combat/dummy.rs`,
     `Actor.script`) and `world_state::lantern::respawn_enemies`.
  3. Give every parallel worker its own registry-number block and WASM event kind.

### 2026-11-12: latticefall-m1c (M1: WP-014, gate G1)

- **Orchestrator:** Sonnet 5.5; side session of the M1 series
- **Branch:** `game/m1-devtools`; claim commit `5e6c1b5` on `main`, WP-014 commit pushed after it
- **Started from:** `origin/main` at `9263599` (local `main` was behind; fast-forwarded)
- **WPs:**
  - WP-014: done by sonnet (no escalation); 154K worker tokens
- **Context:** orchestrator about 60K + worker 154K = about 215K (budget 500K).
- **G1 numbers:** WASM 366,194 B gzipped (936,707 raw). Native `game:bench`: test_field
  49,019 ticks/s, m0_flat 482,910. WASM under Node 25 (30,000 ticks, scripted walk input):
  test_field 30,131 ticks/s, m0_flat 563,253. Goldens match native and WASM
  (`golden.mjs`); Python wasn't re-run this session, last green after WP-015.
  Toolchain time: not measured; M1 sessions lost time to `mise` refusing the worktree's
  `mise.toml` and to a hung WP-013 script (see the m1 entry).
- **Not verified:** the orchestrator did not re-run the browser screenshots (the worker did);
  `overlay=hitboxes|hurtboxes|nav|ai` draw nothing until those systems exist; pointer lock,
  gamepad, mouse-wheel target switching; the `game` CI workflow on GitHub.
- **Gate G1:** the owner confirmed the Rust core (ADR 0001) on 2026-11-12.
- **Waiting on the owner:** the open context-budget
  rule; the lock-on cone reading and camera-direction acquisition (see the m1b entry).
- **Next steps:**
  1. Start M2 with WP-020 (opus).

### 2026-11-12: latticefall-m1b (M1: WP-015, WP-012, WP-014, gate G1)

- **Orchestrator:** Opus 5.5, low effort; side session started from `latticefall-m1`
  (`local_e9f21284-d98a-499d-989a-a9b9aee9498c`)
- **Branch:** `game/m1-camera`; first commit on `main` `7db8780`, last `c6a0228` plus this SHA fix
- **Started from:** `origin/main` at `a5d0e60`
- **WPs:**
  - WP-015: done by sonnet (moved up from haiku: the card names one file, but the work
    also changes the shared actor module, the tick systems, the snapshot flags, the test
    field, and the golden hashes); commit `fa12966`; 128K worker tokens
  - WP-012: done by sonnet in its own worktree, in parallel with WP-015; commit
    `e04b675` on `main`; 212K worker tokens
  - WP-014: not started (budget)
- **Snapshot contract for lock-on:** `ActorSnap.flags` bits follow the architecture
  doc's order: dead `1<<0`, staggered `1<<1`, guarding `1<<2`, invulnerable `1<<3`,
  locked target `1<<4`, two-handing `1<<5`. Constants are in `snapshot.rs` and
  `engine/snapshot.ts`.
- **Context:** orchestrator 136K (at the check after WP-012) + workers 340K = 476K
  (budget 500K). Past the 400K handoff line, so WP-014 and G1 go to the next session.
  As in the previous session, two workers used most of the budget.
- **G1 numbers so far:** WASM 366,194 bytes gzipped (936,707 raw; budget 1.5 MB).
  Native `game:bench`: 50,403 ticks/s on `test_field`, 502,040 on `m0_flat`. Golden
  replays green on native, WASM, and Python after the WP-015 rehash. WASM ticks per
  second not measured yet.
- **Not verified:** target switching with the mouse wheel in the browser (core unit
  tests cover it); pointer lock and gamepad; dark theme (no DOM UI changed).
- **Toolchain notes:** `mise` refuses this worktree's `mise.toml` as untrusted, so the
  session ran cargo from `~/.cargo/bin` with `RUSTUP_TOOLCHAIN=1.99.1` and the mise tasks'
  commands by hand. The untracked `.claude/launch.json` in the main checkout points at
  the old `game-m0-1730d5` worktree on port 5193; this session used its own local entry
  on port 5181.
- **Waiting on the owner:** the context-budget rule (still open; see the previous
  entry). Lock-on reads: is the 60-degree cone the full apex angle (as built) or 60
  degrees each side? Acquisition uses the player's facing because the browser doesn't
  send the camera direction to the core; the spec asks for the camera's.
- **Next steps:**
  1. Run WP-014 (sonnet; `dev/` and `game/crates/wasm/src/debug.rs`). It should fold
     WP-012's ad hoc `cam`, `scenario`, and `fakelock` parsing in `engine/index.ts` and
     `render/camera/devCamera.ts` into the dev URL parser, and add `cam=fixed:`.
  2. Measure WASM ticks per second, then stop at gate G1 with the numbers above.

### 2026-11-12: latticefall-m1 (M1)

- **Orchestrator:** Opus 5.5, low effort; session `local_e9f21284-d98a-499d-989a-a9b9aee9498c`
- **Branch:** `game/m1-movement`, opened as
  [kashifulhaque/ifkash#15](https://github.com/kashifulhaque/ifkash/pull/15). The owner
  then decided that sessions commit to `main` directly, so the branch was pushed to
  `main` and the PR closed as merged.
- **Started from:** `origin/main` at `daffff9` (M0 and the playbook are merged)
- **WPs:**
  - WP-010: done by opus; commit `99b31ed`; 235K worker tokens
  - WP-011: done by sonnet in its own worktree, merged in `543fdb4`, with a follow-up
    fix for WP-010's new `Replay::record` signature; 126K worker tokens
  - WP-013: done by sonnet in its own worktree (the owner approved the animation in
    motion; fine-tuning comes later), merged in `9a34f77` (resolved a
    `scene.ts` conflict with WP-011's camera); 215K worker tokens. The owner approved
    downloading the Quaternius Universal Animation Library, `@gltf-transform/cli`, and
    toktx.
  - WP-012, WP-014, WP-015: not started (budget)
- **Context:** orchestrator 163K + workers 576K = 739K total (budget 500K). The three
  workers alone passed the hard stop, so this session stopped after verifying them.
  The playbook's "6-10 WPs per session" estimate doesn't hold when worker tokens count:
  two L workers and one M worker used 576K. Either raise the budget or count only the
  orchestrator's context; that's the owner's call.
- **Not verified:** pointer lock and gamepad input
  (WP-011 was checked with synthetic key events); x86 determinism (macOS arm64 only);
  the `game` CI workflow on GitHub for this branch.
- **Leftover process:** a WP-013 worker script stopped responding and ran for about
  1.5 hours after the worker reported. The orchestrator stopped it. The playbook now has
  a process rule for workers and a
  [leftover-process check](11-agent-playbook.md#check-for-leftover-processes).
- **Waiting on the owner:** decide the context-budget rule above.
- **Next steps:**
  1. Start a session branch from `origin/main` and push each verified WP to `main`
     (see the playbook's "Branches and commits").
  2. Run WP-015 (haiku; it touches `actor/targeting.rs` and `test_field.ron`) and
     WP-012 (sonnet; render/camera) in parallel. Agree on the snapshot's lock-target
     field first: WP-012 reads what WP-015 writes. WP-012 must take over the orbit yaw
     and pitch that WP-011 put in `engine/scene.ts`.
  3. Then WP-014. Then stop at gate G1 with the numbers: WASM 356,445 bytes gzipped;
     native bench 51,242 ticks/s on `test_field`; golden replays green on native, WASM,
     and Python.

### 2026-11-12: latticefall-m0 (M0)

- **Orchestrator:** Opus 5.5, medium effort; session `local_9d89a627-8c84-46ee-a494-c285e2afed9c`
- **Branch and PR:** `claude/game-m0-1730d5`;
  [kashifulhaque/ifkash#13](https://github.com/kashifulhaque/ifkash/pull/13)
- **Started from:** `main`, with the plan copied in from the planning session as the
  first commit
- **WPs:** WP-001 to WP-008 are all done. Every worker was Sonnet 5.5 at medium effort,
  and the orchestrator re-ran every verify command. Commits and details are in
  `STATUS.md`. ADR 0004 (portfolio text isn't part of the content hash) was added.
- **Context:** not recorded. This entry predates the context budget.
- **Not verified:** 60 fps in a visible browser; a build from a fresh clone.
- **Waiting on the owner:** none for M0 itself; see the planning session's entry.

### 2026-11-12: planning and deploy follow-up (plan, ADR 0003, PR #13 CI)

- **Orchestrator:** Opus 5.5 (the session that wrote the plan)
- **Branch and PR:** follow-up commits on `claude/game-m0-1730d5`, PR #13
- **What happened:**
  - Wrote the plan in `docs/game/`, with research from five Haiku agents and a
    consistency review by a separate agent.
  - The owner approved ADR 0003. `web-deploy.yml` deploys the site to the `ifkash`
    Pages project from GitHub Actions. The `CLOUDFLARE_PAGES_API_TOKEN` secret exists.
  - Fixed two preview-deploy failures on PR #13: the deploy jobs now get the full
    SvelteKit output and install `node_modules`. The preview deployed and serves
    `/`, `/game`, `/game?engine=lf`, `/work`, and `/projects`.
  - Added the orchestrator playbook, a worker model on every WP card, this journal, and
    the context budget.
- **Context:** about 470K. This session ran past the 400K handoff point while writing
  these rules, and handed off at the end.
- **Not verified:** that the game renders in a real browser on the preview, and the
  Google sign-in pages on the preview.
- **Owner actions, done:**
  - PR #13 was squash-merged to `main` as `0834919`, and the first production deploy
    from GitHub Actions succeeded. `ifkash.dev` serves `/`, `/game`,
    `/game?engine=lf`, and `/work`.
  - The Pages Git integration's automatic builds are off: production and preview
    deployments and PR comments are disabled. The Git connection itself was kept,
    because `cf pages projects disconnect-source` warns that it permanently deletes
    the resource. GitHub Actions is now the only deploy path.
  - The plan update (this playbook, worker models, and this journal) missed the squash
    merge and went to `main` in a follow-up pull request.
- **Still waiting on the owner:** a review of the relic text in
  `game/data/portfolio.ron`. It doesn't block M1.
- **Next steps:**
  1. Branch M1 from `main`. The owner's main checkout may be behind `origin/main`;
     fetch first and branch from `origin/main`.
  2. Run M1: WP-010 (opus), then WP-011, WP-012, WP-013, and WP-015 in parallel where
     Touch lists allow, then WP-014.
  3. Stop at gate G1 and give the owner the G1 numbers from the roadmap.
