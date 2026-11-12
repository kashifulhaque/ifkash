# Vision and scope

*Latticefall* (working title) is a small, original, souls-like 3D action RPG that runs
in the browser at `ifkash.dev/game`. It replaces the tiny-planet explorer that lives in
`src/lib/game/` today. It has two jobs:

1. **Be the portfolio.** Every work entry, project, education entry, blog post, the
   résumé, and contact details are discoverable inside the world, and every one of them
   stays reachable as plain HTML without playing.
2. **Be an RL environment.** The same simulation that runs in the browser runs headless
   from Python, deterministically, thousands of steps per second, so the owner can train
   and evaluate agents on it: classic deep RL, LLM or VLM agents, or sequence models.

## Pillars

Every design decision is checked against these five pillars, in priority order. When two
conflict, the earlier one wins.

1. **Content is never locked.** A recruiter who never presses a key still gets the name,
   role, résumé, and contact in under three seconds. The game is a second way to read the
   portfolio, not a gate in front of it.
2. **One simulation, many players.** Humans, scripted bots, RL policies, and LLMs all
   drive the same deterministic core through the same `ActionFrame`. No gameplay logic
   lives in the renderer.
3. **Small, dense, interconnected.** One compact world that loops back on itself beats a
   large empty one. Every area has a landmark, a secret, and a shortcut.
4. **Readable, fair combat.** Every enemy attack is telegraphed. Death teaches. The player
   always knows why they died.
5. **Buildable by agents.** Content is data, levels are data, and every system has a
   headless test. An AI coding agent with no art skills can add a weapon, an enemy, or a
   dungeon by editing data files and checking screenshots.

## Audience

| Audience | What they want | How the game serves them |
|---|---|---|
| Recruiters and hiring managers | Facts, fast | Landing card with name, role, résumé, and contact before any 3D loads; **Read the plain site** link always visible |
| Engineers browsing the site | Something impressive and technically interesting | A real souls-like with a Rust and WASM core, an open RL environment, and a dev overlay |
| Gamers | A satisfying short souls-like | 2-3 hours to full clear, 12 bosses, 9 builds, secrets |
| Non-gamers who are curious | To see the content without dying 40 times | **Wayfarer mode** (story mode), damage scale, boss skip |
| The owner, as an ML engineer | A research environment | Gymnasium, vector, text, and replay interfaces; human-demo datasets |

## Scope budget

The scope budget keeps the game small enough to finish. Treat these numbers as caps,
not targets. A WP that pushes a number past its cap needs an ADR.

| Item | Budget | Spec |
|---|---|---|
| Playtime, main path | 60-90 minutes | [World](02-world.md) |
| Playtime, full clear | 2-3 hours | [World](02-world.md) |
| Regions | 6, including the 2 legacy dungeons, plus the hub | [World](02-world.md#regions) |
| Legacy dungeons | 2 (Castle Graymantle, the Lattice Spire) | [World](02-world.md#legacy-dungeons) |
| Mini-dungeons | 7 (2 caves, 2 catacombs, 1 mine tunnel, 1 sunken ruin, 1 sealed arena) | [World](02-world.md#mini-dungeons) |
| Checkpoints (Lanterns) | 24 | [World](02-world.md#checkpoints) |
| Enemy types | 16, plus 5 elites | [Enemies](05-enemies-and-bosses.md#enemy-roster) |
| Bosses | 12 (4 required) | [Bosses](05-enemies-and-bosses.md#boss-roster) |
| Attributes | 7 | [Combat](03-combat.md#attributes) |
| Weapon classes | 14 | [Arsenal](04-arsenal-and-builds.md#weapon-classes) |
| Weapons | 36, including 4 shields and 5 catalysts | [Arsenal](04-arsenal-and-builds.md#weapon-list) |
| Arts (weapon skills) | 14 | [Arsenal](04-arsenal-and-builds.md#arts) |
| Spells | 18 (9 Formulae, 9 Litanies) | [Arsenal](04-arsenal-and-builds.md#spells) |
| Charms | 14 | [Arsenal](04-arsenal-and-builds.md#charms) |
| Armor sets | 7 | [Arsenal](04-arsenal-and-builds.md#armor) |
| Status effects | 6 | [Combat](03-combat.md#status-effects) |
| Echo summons | 4 | [Enemies](05-enemies-and-bosses.md#echo-summons) |
| NPCs | 5, of which 4 have questlines | [Systems](06-progression-and-systems.md#npcs-and-questlines) |
| Build archetypes supported end to end | 9 | [Arsenal](04-arsenal-and-builds.md#builds) |

## Non-goals

The following are out of scope for version 1. Each one is a trap that would multiply the
work without serving either job:

- Real-time multiplayer, invasions, and co-op. Asynchronous messages and ghost replays
  give most of the feeling for a fraction of the cost.
- Mounts and horseback combat.
- Crafting beyond a single smith (upgrades only).
- Character creation beyond a few presets (body, palette, starting class).
- Procedurally generated worlds. The world is authored. Procedural tools only dress it.
- Hand-authored cutscenes. Bosses get an intro camera move and a title card, nothing
  more.
- Voice acting.
- A mobile-first control scheme. Phones get a read-only codex and an optional touch mode,
  not a tuned touch combat experience. See
  [the presentation spec](07-presentation.md#mobile).

## World premise

The following premise is the whole story. Everything else is told through item
descriptions, NPC lines, and the environment.

> The Lattice was a great woven structure of light that held the realm of Vael together:
> each thread a memory, each knot a law. Something pulled one thread too hard. The
> Lattice fell, and its knots scattered into the land as **Sigils**, each held by
> whoever caught it and twisted by holding it too long.
>
> You are a **Wayfarer**, one of the people the Lattice forgot to remember. Wayfarers
> don't stay dead; they wake at the nearest **Lantern**. Gather the Great Sigils (any two open the way),
> climb the Lattice Spire, and decide what the realm remembers.

The premise has a quiet second reading for the portfolio: the Lattice is a model's
weights, the Sigils are the owner's career chapters, and remembering is training. The
game never says this out loud. Item descriptions wink at it, for example a stylus named
*Gradient Rod* or a boss called *the Overfit Warden*. Keep the winks rare (about one in
five items) so the world still reads as a fantasy world.

## Vocabulary

The game uses its own vocabulary. Always use these terms in code, data, and UI copy.
The [README glossary](README.md#glossary) lists them with their genre equivalents.

| Term | Meaning |
|---|---|
| Wayfarer | The player character |
| Lantern | Checkpoint: rest, level up, refill flasks, respawn enemies, fast-travel node |
| Glimmer | Currency and experience |
| Remnant | The glimmer you drop on death; touch it to recover it |
| Vigil Flask | Healing flask |
| Lucid Flask | FP restore flask |
| Sigil | A boss's reward that advances the main path |
| Great Sigil | One of the four required Sigils |
| Echo | A summoned spirit ally |
| Echo Stone | A spot near a boss where you can summon an Echo |
| Art | A weapon skill that costs FP |
| Art Scroll | An item that teaches a weapon a different Art |
| Formula | An Insight spell, cast with a stylus |
| Litany | A Devotion spell, cast with a reliquary |
| Charm | A passive equipment slot item |
| Relic | A unique item that holds a portfolio project |
| Chronicle Page | A readable item that holds a work-history entry |
| Wayfarer's Ledger | The key item that opens the résumé |
| Codex | The in-game and on-site index of everything discovered |
| Wayfarer mode | Story mode for non-gamers |

## IP rules

*Latticefall* is a genre homage, not a copy. Mechanics and genre conventions are fine to
use: stamina combat, lock-on, rolls with invincibility frames, checkpoints that respawn
enemies, dropping currency on death, asynchronous messages, and boss fog gates are
shared across dozens of games.

The following are banned everywhere, including code identifiers, data IDs, comments,
commit messages, and UI copy:

- Names of the source games and their publishers, except in this sentence, in source
  links in the research notes, and in a "not affiliated" credits line: Elden Ring, Dark
  Souls, Demon's Souls, Bloodborne, Sekiro, FromSoftware, Bandai Namco.
- Their proper nouns and coined terms, including: Tarnished, Erdtree, Elden, Grace, Site
  of Grace, Rune (as currency), Great Rune, Estus, Crimson Tears, Cerulean Tears, Flask
  of Wondrous Physick, Ashes of War, Spirit Ashes, Torrent, Spirit Calling Bell, Hollow
  or Hollowing (as a name for a cursed undead state), Bonfire, Humanity (as an item),
  Lordvessel, Summon Sign, Stake of Marika, Evergaol, Roundtable Hold, Two Fingers,
  Crucible, Scarlet Rot, Hemorrhage (as the status name), Deathblight, Golden Order,
  Shardbearer, Demigod, Kindling (as a mechanic). Ordinary English words such as
  *hollow* or *covenant* are fine in original names, for example Hollowdeep.
- Their characters, bosses, creatures, item names, place names, and quotes.
- Death-screen and victory-screen treatments that imitate theirs: large centered serif
  text in red or gold with a slow fade. Use the original treatment in
  [the presentation spec](07-presentation.md#death-and-victory).
- Their fonts, logos, music, sound effects, icons, and UI layouts.
- Their names in page metadata, SEO keywords, or social cards. Describe the game as
  "a small action RPG" or "a souls-like."

Before launch, the owner runs a trademark search on the final title. Keep a dated log of
original names in `docs/game/decisions/` so originality is easy to show.

## Success criteria

Version 1 is done when all of the following are true:

- A first-time visitor sees the name, role, résumé link, and contact link in under three
  seconds on a mid-range laptop, before the game bundle loads.
- A gamer can finish the main path in 60-90 minutes and the full clear in 2-3 hours.
- A non-gamer in Wayfarer mode can reach every portfolio item in under 30 minutes.
- Every portfolio item has a stable codex URL that renders as plain HTML.
- The game holds 60 frames per second on a 2021-era laptop with integrated graphics at
  1080p, and 30 frames per second on a mid-range phone in touch mode.
- `pip install` of the Python package, followed by `gymnasium.make("Latticefall/BossArena-v0")`,
  runs at 5,000 or more steps per second per CPU core with vector observations.
- A replay recorded in the browser reproduces bit-for-bit in the Python environment.
