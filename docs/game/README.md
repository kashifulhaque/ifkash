# Latticefall design docs

*Latticefall* (working title) is the souls-like action RPG that replaces the
tiny-planet game at `ifkash.dev/game`. It's a portfolio you can play, and an RL
environment you can train agents on. These documents are the complete plan, written so
AI coding agents can build it one work package at a time.

## Start here

- **If you're an AI coding agent** with a work package to do: read
  [the agent playbook](11-agent-playbook.md), then [`STATUS.md`](STATUS.md), then your
  card in [the roadmap](10-roadmap.md). Read only the sections your card lists.
- **If you're the owner** planning the next session: check [`STATUS.md`](STATUS.md),
  pick the next WPs from [the roadmap's waves](10-roadmap.md#critical-path-and-parallelism),
  and paste the session prompt from [the playbook](11-agent-playbook.md#start-a-session).
- **If you're reviewing the design:** read in the order of the following table.

## Documents

| Doc | What it decides |
|---|---|
| [00 Vision and scope](00-vision-and-scope.md) | Pillars, audience, scope budget, non-goals, premise, vocabulary, IP rules, success criteria |
| [01 Architecture](01-architecture.md) | Rust core, WASM and Python hosts, determinism contract, inputs, data pipeline, budgets |
| [02 World](02-world.md) | Regions, legacy dungeons, mini-dungeons, Lanterns, gates, traversal, building an area |
| [03 Combat](03-combat.md) | Verbs and frame data, attributes, damage formulas, poise, guard, parry, status, spells, lock-on |
| [04 Arsenal and builds](04-arsenal-and-builds.md) | Weapon classes, 36 weapons, Arts, 18 spells, Charms, armor, consumables, classes, 9 builds |
| [05 Enemies and bosses](05-enemies-and-bosses.md) | AI framework, 16 enemies and 5 elites, 12 bosses, Echoes |
| [06 Progression and systems](06-progression-and-systems.md) | Leveling, economy, death, NPCs, map, codex, Marks, async features, telemetry |
| [07 Presentation](07-presentation.md) | Art, assets and licenses, animation, VFX, camera, controls, HUD, audio, accessibility, mobile |
| [08 Portfolio integration](08-portfolio-integration.md) | Landing card, content mapping, relics, Chronicle Pages, codex pages, SEO |
| [09 RL environment](09-rl-environment.md) | Scenarios, observations, actions, rewards, Python API, LLM interface, datasets, experiments |
| [10 Roadmap](10-roadmap.md) | Milestones M0-M9 and about 90 work package cards |
| [11 Agent playbook](11-agent-playbook.md) | How a session runs, definition of done, determinism rules, visual checks, lanes |
| [12 Research notes](12-research-notes.md) | What the plan's research found, with sources and open questions |
| [STATUS.md](STATUS.md) | Live progress tracker |
| [decisions/](decisions/) | Architecture decision records (ADRs) |

## Open decisions for the owner

These need an answer before the WPs that depend on them:

| Decision | Needed before | Default if no answer |
|---|---|---|
| Finish the [ADR 0003](decisions/0003-frontend-deploy.md) rollout (approved 2026-11-12): set the Pages project name, add the secrets, then disconnect the Pages Git integration | Merging the M0 branch to `main` | None. Until it's done, merging breaks Pages deploys, because the WASM module isn't committed |
| Confirm the working title *Latticefall*, after a trademark search | [WP-086](10-roadmap.md#wp-086) (launch) | Keep it as a working title |
| Confirm the Rust-core decision at gate G1 | [M2](10-roadmap.md#m2-combat-core) | Continue with Rust |
| Approve each new asset pack (license check) | The WP that adds it | Use only packs already verified in [the presentation spec](07-presentation.md#sources) |
| Choose whether research replays are collected at all | [WP-098](10-roadmap.md#wp-098) | Don't collect |

## Glossary

The game uses its own vocabulary. The following table maps each term to the generic
genre concept, so a reader new to souls-likes can follow the specs:

| Term | Generic concept |
|---|---|
| Wayfarer | Player character |
| Lantern | Checkpoint and rest point |
| Glimmer | Currency and experience points |
| Remnant | Dropped currency, recoverable once after death |
| Vigil Flask, Lucid Flask | Healing flask, mana flask |
| Wick Bead, Flask Dew | Flask charge upgrade, flask potency upgrade |
| Sigil, Great Sigil | Boss reward; main-path key |
| Echo, Echo Stone, Echo Bell | Summoned AI ally; summoning spot; summoning tool |
| Art, Art Scroll | Weapon skill; item that changes a weapon's skill |
| Formula, Litany | Intelligence-style spell; faith-style spell |
| Stylus, reliquary | Spell catalysts |
| Charm | Passive accessory slot |
| Temper | Weapon affinity |
| Rewrite Quill | Respec item |
| Forgeheart Ember | Key item that unlocks higher upgrade levels at the Smith |
| Oil | Weapon coating that adds an element or status for a while |
| Forgeglass, Starglass | Common and rare upgrade materials |
| Relic | Unique item that holds a portfolio project |
| Chronicle Page | Readable item that holds a work-history entry |
| Codex | In-game and on-site encyclopedia |
| Wayfarer mode | Story or assist mode |
| Hollow wall | Hidden wall revealed by a hit |
| Afterimage | Another player's death replay |
| Wanderer | Another player's ghost walking past |
| Mark | Achievement |
| Reweave | New game plus |
| Tick | One simulation step; 30 per second |
| WP | Work package: one AI coding session of work |
| Lane | Area of the codebase a WP owns |
| Golden replay | Committed input log with expected state hashes, used as a regression test |
