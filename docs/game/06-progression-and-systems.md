# Progression and systems

This document covers everything outside a single fight: leveling, the Glimmer economy,
death, flasks, Lanterns, equipment slots, NPCs and their questlines, the map, the codex,
achievements, and the asynchronous online features. Read it before any WP that touches
these systems.

## Leveling

The player levels up at any Lantern by spending Glimmer. Each level adds one point to
one attribute.

```text
cost(L) = floor(100 + 20 * L + 1.2 * L^2)      # Glimmer to go from level L to L + 1
```

| Level | Cost to next | Cumulative from level 10 |
|---|---|---|
| 10 | 420 | 0 |
| 20 | 980 | 6,518 |
| 30 | 1,780 | 19,716 |
| 45 | 3,430 | 57,288 |
| 60 | 5,620 | 123,390 |

### Expected level curve

The economy is tuned so that the following levels are typical for a player who explores
moderately. Glimmer drops from enemies and items are tuned against this table in
[WP-089](10-roadmap.md#wp-089).

| Point in the game | Expected level |
|---|---|
| After B01 | 11 |
| Leaving the Verge | 18 |
| First Great Sigil | 28 |
| Second Great Sigil | 38 |
| Final boss, main path | 45 |
| Full clear | 60-65 |

## Glimmer and death

- Enemies drop Glimmer on death: fodder 30-80, elites 600-1,500, bosses 3,000-20,000.
- On death, the player drops **all** held Glimmer as a **Remnant** at their last safe
  position (the last tick their feet touched walkable, non-hazard ground).
- Touching the Remnant recovers it. Dying again before recovering it destroys it.
- A Remnant inside a boss arena is placed just outside the fog gate.
- The player respawns at the last Lantern they rested at, with full HP, FP, and flasks.
  Non-boss enemies respawn.
- With **Keep Glimmer on death** enabled (on in Wayfarer mode), nothing is dropped.

## Flasks

- The player starts with **4 flask charges**, split between the Vigil Flask (HP) and the
  Lucid Flask (FP). The split is set while resting at a Lantern.
- Each **Wick Bead** adds one charge, up to 10. There are 6 in the world.
- Each **Flask Dew** raises both flasks' potency by one level, up to +6. There are 6 in
  the world.

## Lanterns

Resting at a Lantern does all of the following:

- Restores HP, FP, and flask charges, and cures statuses.
- Respawns non-boss enemies.
- Sets the respawn point.
- Opens the Lantern menu: **Level up**, **Attune spells**, **Split flasks**, **Travel**,
  and **Change equipment loadouts**.

Fast travel between lit Lanterns is available only from the Lantern menu, which keeps
the world feeling physical. The one exception is Lantern Ash, which returns the player to
the last Lantern they rested at from anywhere outside a boss arena.

## Equipment slots

| Slot | Count | Notes |
|---|---|---|
| Right hand | 2 | Cycle with `CYCLE_RIGHT` |
| Left hand | 2 | Cycle with `CYCLE_LEFT`; shields, catalysts, or a second weapon |
| Head | 1 | |
| Body | 1 | |
| Charms | 2, then 3 and 4 after the first and second Great Sigils | |
| Quick items | 5 | Cycle with `CYCLE_ITEM` |
| Arrows | 1 | Used when a bow is equipped |
| Spells | Attuned at Lanterns, limited by spell slots | Cycle with `CYCLE_SPELL` |

The menu shows each change's effect on AR, defense, equip load, and roll tier before the
player confirms it. Equipment changes are `Command` values in the core, so they replay.

## NPCs and questlines

Five NPCs live in Hollowhearth. Four have a three-state questline (before, during,
after), and each delivers part of the portfolio or a service. **No portfolio fact is locked behind a quest.**
Quests only add flavor, rewards, and codex illustrations; the facts are available from
the first conversation.

| NPC | Portfolio role | Quest | Reward |
|---|---|---|---|
| The Archivist | Education; blog Letters; respec | Find three Lost Folios (Ninefold Chapel, the castle library, Whitewarden Catacombs) | Scholar's Seal (education relic) |
| The Cartographer | Work history | Bring every Chronicle Page; the Cartographer binds them into *The Wayfarer's Chronicle* | *Opencode* dye (relic) |
| The Smith | Upgrades | Bring the Forgeheart Ember from B08 | Unlocks weapon levels +6 to +10 and Starglass +3 to +5 |
| The Courier | Contact | Recover three lost mail satchels from the Verge, Mirrormere, and Rimeshelf | *PoliteLlama* Echo (relic) |
| The Peddler | Shop | none | Sells consumables, arrows, Lantern Ash, Forgeglass Shards, and the Hunter's Shortbow |

Until the player first visits Hollowhearth, the Peddler stands at Verge Crossroads.

### Dialogue rules

- Each NPC has a greeting, one line per quest state, and a **Ask about...** menu that
  shows the portfolio content directly. For example, the Courier's menu has **How to
  reach the Wayfarer** with the real contact links.
- NPC lines are 1-3 sentences. The voice is warm and a little dry. Facts (role, dates,
  outcomes) are stated plainly in the portfolio panels, not in the fantasy voice. See
  [the portfolio spec](08-portfolio-integration.md#tone).
- Dialogue lives in `game/data/npcs/*.ron` with portfolio fields pulled from
  `src/lib/content.ts` at build time.

### How dialogue and quests are built

[WP-068](10-roadmap.md#wp-068) built Hollowhearth, the five NPCs, and the stations.
The following list states what is decided:

- **The simulation owns the conversation.** Pressing Interact near an NPC or a station
  opens it; `Command::Ask`, `Command::EndTalk`, and `Command::Appearance` drive it. The
  state is world flags (`dialogue.*`, `quest.<id>.state`, `mirror.*`), so it saves and
  replays with the world. The DOM reads the state and sends commands.
- **A topic names facts by section and slug**, never by text. A slug of `*` names every
  item of the section, so an entry added to `content.ts` reaches the hub without editing
  the data. `lf-tools validate` fails if a portfolio entry is not shown by some NPC or
  station, and a core test walks the dialogue tree to prove it.
- **A quest has three states** (0 before, 1 during, 2 after). Asking about the quest takes
  it up, and asking again with everything brought turns it in. A quest that is already
  satisfied when taken up finishes at once. A quest's `needs` are world flags (a pickup
  raises `pickup.<id>.taken`) or a held item. `at_least` lets the Cartographer finish
  with four of the five Chronicle Pages, because four work entries leave one slot empty.
- **Quest pickups belong to the areas that hold them.** The flag names the NPC data waits
  for are `pickup.folio_ninefold`, `pickup.folio_library`, `pickup.folio_whitewarden`,
  `pickup.page_chronicle_1` to `5`, `pickup.satchel_verge`, `pickup.satchel_mirrormere`,
  and `pickup.satchel_rimeshelf` (each with the suffix `.taken`).
- **Hollowhearth is a safe zone** (`safe_zone: true` in `area.ron`): the core clears the
  attack, guard, dodge, and Art buttons and caps the stick at a walk.

## Map

- The map reveals as the player explores: the core records visited 16-meter map cells
  in world flags.
- The map shows lit Lanterns, defeated bosses, discovered dungeon entrances, the
  player's Remnant, and up to 20 player-placed markers.
- After the Cartographer's quest starts, Chronicle Page locations show as faint marks.
- The map has a **list view** that lists every discovered place as text, for
  screen-reader users and for players who don't read maps well.
- **Built (WP-069):** the core keeps the visited cells as bit rows in flags
  (`map.<area>.row.<z>.<chunk>`), the markers as flags (`map.<area>.marker.<slot>`), and a
  pending trip as `map.travel` (`world_state/map.rs`). The DOM map (`ui/map/`) draws the
  visited cells plus a one-cell halo, and the list view says each place with its distance and
  compass direction from the player. Only lit Lanterns, defeated bosses, the Remnant, and
  markers show; dungeon entrances and Chronicle Page marks wait for their systems.
- **Fast travel crosses areas.** `Command::Travel` moves within the area. `Command::TravelFar`
  starts a trip to a lit Lantern in another area: the host replaces the world with the
  destination area's (`map::tick_world`), keeping the character, flags, and tick, and rests
  at the Lantern. A trip is refused while a Remnant lies behind, because the Remnant has no
  area of its own.

## Codex

The codex is the shared content layer between the game and the plain site.

- It holds entries for every region, boss, enemy, weapon, spell, Charm, armor piece,
  relic, Chronicle Page, Letter, and NPC.
- Inside the game, it's a menu. Outside the game, it's a set of plain HTML pages under
  `/codex`. Portfolio entries are always visible on the site, unlocked or not. Game-lore
  entries show as titles only until discovered.
- Unlocks are stored by `src/lib/latticefall/codex/` in `localStorage`, so the codex
  pages and the home page can read them without loading the game.

For the details, see [the portfolio spec](08-portfolio-integration.md#codex).

## Achievements

Achievements are called **Marks**. Each one is stored locally and, for signed-in
players, in D1. The following list defines them:

- **First Light**: Rest at a Lantern.
- **Woken**: Defeat B01.
- **Reader**: Open the Wayfarer's Ledger (the résumé).
- **Correspondent**: Talk to the Courier.
- **Chronicler**: Collect every Chronicle Page.
- **Collector**: Find all relics.
- **Great Sigil** marks: One per Great Sigil boss.
- **Ever After**: Defeat B12.
- **Thorough**: Defeat all 12 bosses.
- **Pacifist Path**: Reach the Spire Gate having killed no non-boss enemies after the
  Verge.
- **Generalist**: Defeat B07 with no action category above 35% in phase 1 (beat the
  Overfit Warden without being overfit).
- **Speedrunner**: Defeat B12 in under 45 minutes of in-game time.
- **Polite**: Use the bow gesture in front of the PoliteLlama Echo.
- **Unbroken**: Defeat any Great Sigil boss without taking damage.

## Gestures and photo mode

- Six gestures: wave, bow, point forward, sit, shrug, and beckon. A gesture is a
  `Command::Gesture` chosen from a gesture menu. It starts a move with no hitboxes, so
  gestures are deterministic and replay.
- Photo mode pauses the game and frees the camera within 15 m of the player. It can
  hide the HUD, apply three color grades, and adds a small `ifkash.dev` watermark to
  saved images.

## Timer and new game plus

- An in-game time counter (ticks while not paused) is always recorded and optionally
  shown. It's the basis of the **Speedrunner** Mark and an optional leaderboard.
- **Reweave** (new game plus) is available after B12: enemies get +30% HP and +15%
  damage; the player keeps level and equipment; quest and world flags reset. Only one
  cycle exists. This is a stretch feature in [milestone M8](10-roadmap.md#m8-polish-and-launch).

## Asynchronous features

These features use Cloudflare D1 through the existing Worker. They're optional: the
game is complete without them, and they're off in the RL environment.

| Feature | Behavior | Storage | Moderation |
|---|---|---|---|
| Messages | Signed-in players leave a message built from templates and a fixed word list, for example "Try [rolling] ahead." Others rate them. | Area, position, template ID, word IDs, rating, author's anonymous ID | No free text, so nothing to moderate; low-rated messages hide |
| Afterimages | A red mark where another player died. Touching it plays a 5-second ghost of their last moments. | Area, position, a 150-tick snippet of positions and move IDs, cause of death | Sampled; at most 20 per area shown |
| Wanderers | Faint ghosts of other players walking past, replayed from position trails | Area, a short trail of positions and move IDs | Sampled; no identity shown |

- Reads are anonymous and cached at the edge. Writes need a Google sign-in, and each
  user has a write rate limit.
- The data uses an anonymous per-install ID stored in `localStorage`. It's linked to the
  Google account only when the player signs in to write.
- Records older than 90 days are deleted by a scheduled cleanup.
- The Worker endpoints are under `/api/latticefall/` and follow the existing pattern:
  a handler in `api/src/handlers/latticefall.rs`, a line in `api/src/routes/mod.rs`, and
  a migration in `api/migrations/`.

## Telemetry

Telemetry helps tune the game and seeds later RL work. It's opt-in, anonymous, and
batched.

- **Events:** session start and end, area entered, Lantern lit, death (area, 8-meter
  grid cell, cause), boss attempt (boss, phase reached, duration, result), difficulty
  settings, and plain-site link clicks.
- **Transport:** One batched request per session end (or every 10 minutes) to
  `POST /api/latticefall/telemetry`, stored in D1, retained 90 days.
- **Replays for research:** a separate, explicit opt-in. See
  [the RL spec](09-rl-environment.md#human-demonstrations).
- **Site analytics:** Cloudflare Web Analytics, which doesn't use cookies, covers page
  views and load timing.
