# Enemies and bosses

This document defines the AI framework, the enemy roster, and all 12 bosses. Read it
before any WP in the `ai` lane.

## AI framework

Enemy AI lives in `game/crates/core/src/ai/` and is driven by data in
`game/data/enemies/` and `game/data/bosses/`. It runs inside the core, so the RL
environment faces exactly the enemies a human faces.

### Awareness states

Every AI actor is in one of the following states:

| State | Behavior | Exits to |
|---|---|---|
| Idle | Stands, sits, or patrols waypoints | Suspicious on a sound; Aware on sight |
| Suspicious | Turns to the last noise, walks toward it for up to 6 seconds | Aware on sight; Idle after timeout |
| Aware | Plays a short notice animation (15-30 ticks), then engages | Combat |
| Combat | Picks options (attacks, spacing) until the target is lost | Return when leashed |
| Return | Walks back to its spawn point and heals to full | Idle |

### Perception

- **Sight:** A cone (default 110 degrees, 18 m) with a line-of-sight ray to the target's
  chest. In darkness, the range drops to 6 m unless the player carries a light.
  Equipment such as *Boo* and *endark* scales the range.
- **Hearing:** Player actions emit noise with a radius: walk 2 m, run 6 m, sprint 10 m,
  roll 8 m, attack 10 m, landing from a fall 12 m. Crouching isn't in the game.
- **Leash:** An actor in Combat returns home when it's more than 30 m from its spawn
  (configurable) or hasn't seen the player for 240 ticks.
- **Unaware:** An actor in Idle or Suspicious that doesn't face the player is unaware,
  which allows backstabs.

### Decision making

In Combat, the AI picks **options** with a weighted, utility-style choice. Each option in
the data has conditions and a weight. Every AI think step (10 Hz) evaluates the
available options, filters by conditions, and samples one by weight from the `ai` random
stream.

The following code sample shows an enemy's options:

```ron
// game/data/enemies/gray_knight.ron (excerpt)
(
  id: "gray_knight",
  model: "knight_gray",
  hp: 520, poise: 40, stance: None,
  defense: (phys: 60, aether: 30, fire: 30, storm: 30, radiance: 30),
  weakness: (pierce: 1.15),
  resist: (immunity: 150, robustness: 180, composure: 120),
  glimmer: 140,
  perception: (sight_deg: 110, sight_m: 18, hearing_mult: 1.0),
  moveset: "gray_knight.ron",
  options: [
    (move: "gk_shield_advance", when: [DistBetween(3.0, 9.0)], weight: 3, cooldown: 60),
    (move: "gk_slash_combo",    when: [DistBelow(2.4), FacingWithin(45)], weight: 5, cooldown: 0),
    (move: "gk_thrust",         when: [DistBetween(2.0, 3.4)], weight: 3, cooldown: 30),
    (move: "gk_shield_bash",    when: [DistBelow(1.8), TargetGuarding], weight: 4, cooldown: 90),
    (behavior: Strafe(dur: 45), when: [DistBetween(2.5, 6.0)], weight: 2),
    (behavior: Guard(dur: 30),  when: [TargetAttacking, DistBelow(4.0)], weight: 4),
  ],
  drops: [(item: "itm_glimmer_vial_s", chance: 0.08)],
)
```

Behaviors (approach, strafe, retreat, guard, wait, flee, and patrol) are built into the
core. Conditions include distance bands, facing angles, HP fractions, the target's
current action (attacking, healing, guarding, casting), cooldowns, and the boss phase.

### Attack tokens

To keep group fights fair, at most **two** enemies can be in an attack move against the
player at once. An enemy without a token strafes or waits at range. Archers and casters
use a separate pool of one ranged token. Bosses and their adds ignore tokens.

### Navigation

Enemies use the area's navigation grid (see
[the architecture spec](01-architecture.md#navigation)). Enemies can't climb ladders or
use lifts; a player who escapes that way is leashed out. Some enemies (Drowned, Reed
Stalkers) are confined to water, marked by a `water_only` flag.

## Enemy roster

The game has 16 enemy types and 5 elites. Each type is one data file and one moveset.

| ID | Enemy | Region | Role | HP | Notes |
|---|---|---|---|---|---|
| `husk` | Husk | Everywhere | Fodder | 120 | Slow two-hit combo. Variants: Servant (castle), Miner (Hollowdeep, carries a lantern, can't be backstabbed while lit), Frost Husk (Rimeshelf, Rime buildup) |
| `thorn_hound` | Thorn Hound | Verge, castle kennel | Fast pack | 90 | Packs of 3; lunges, retreats; Rupture buildup |
| `lantern_thief` | Lantern Thief | Verge | Ambusher and thrower | 140 | Hides on roofs and in ruins; throws pitch bombs, flees when approached |
| `acolyte` | Acolyte | Verge chapel, castle chapel, Belfry | Caster | 160 | Casts a slow Radiance bolt with a visible 30-tick glow; Bell variant casts Rime |
| `drowned` | Drowned | Mirrormere | Grab | 220 | Water only; slow; a grab costs 20% HP unless dodged; weak to Storm |
| `reed_stalker` | Reed Stalker | Mirrormere | Ambush | 180 | Hidden in reeds; a single heavy lunge, then retreats into the reeds |
| `marsh_crawler` | Marsh Crawler | Mirrormere, D1 | Swarm | 60 | Groups of 4-6; Poison bite |
| `cave_moth` | Cave Moth | Hollowdeep | Flyer | 110 | Hovers; drops Drowse dust; drawn to light sources |
| `lamp_mite` | Lamp Mite | Hollowdeep | Swarm | 30 | Groups of 6-10; explodes for Fire after 60 ticks of contact |
| `shard_golem` | Shard Golem | Hollowdeep, D5 | Tank | 600 | Slow, heavy hits; crystals reflect Formulae unless struck with Blunt first |
| `glasswing_stalker` | Glasswing Stalker | Hollowdeep, D5 | Ambusher | 260 | Nearly invisible when still; shimmers when it moves |
| `gray_knight` | Gray Knight | Castle | Shield-bearer | 520 | Guards high; punished by guard counters, bashes, and Pierce |
| `gray_archer` | Gray Archer | Castle ramparts | Ranged | 200 | Fires on a 90-tick cadence from high ground; repositions after being hit |
| `rime_wolf` | Rime Wolf | Rimeshelf | Pack | 160 | Packs of 3-4; circles, then one lunges at a time |
| `whitewarden_skeleton` | Whitewarden Skeleton | Rimeshelf, D6 | Reviver | 180 | Revives after 180 ticks unless killed by Radiance, a critical, or near a lit brazier |
| `mimic_chest` | Mimic Chest | Castle armory | Surprise | 900 | One in the game; the chest's latch is on the wrong side as a hint |

| ID | Elite | Region | HP | Notes |
|---|---|---|---|---|
| `bramble_brute` | Bramble Brute | Verge, Windmill Rise | 1,100 | A huge husk wrapped in thorns; slow, long combos; drops Bramble Claws |
| `choir_knight` | Choir Knight | Mirrormere, D3 | 1,400 | Halberd and bell; a bell ring stuns within 4 m (telegraphed by lifting the bell) |
| `gray_captain` | Gray Knight Captain | Castle ramparts | 1,600 | Greatsword duelist; parryable delayed swings |
| `foundry_smith` | Foundry Smith | Rimeshelf | 1,500 | Hammer and tongs; hammer slams leave burning metal on the ground |
| `bell_warden` | Bell Warden | Rimeshelf, D6 (as the Twin Wardens) | 1,300 | Strikes a hand bell; each third strike is a stun wave |

Dungeon guardians reuse an existing moveset with elite stats: a pair of Glasswing
Stalkers (D5) and the Twin Wardens, a pair of Bell Wardens (D6). They don't count toward
the elite budget.

## Boss framework

Bosses share the following systems in `game/crates/core/src/ai/boss.rs`:

- **Fog gate.** Walking through seals the arena. The fight starts after a skippable
  3-second intro camera move (render-only; the core waits 90 ticks).
- **Boss bar.** Shows the name and HP. Stance is hidden.
- **Phases.** A phase triggers at an HP threshold. The boss plays a 45-tick transition
  move with full invulnerability, then switches to the phase's option list. A phase
  must add new behavior, not only more damage.
- **Echo Stone.** Present at every boss except B01 and B12's second phase.
- **Reset.** Dying resets the boss fully, including adds and hazards.
- **Rewards.** Defeating a boss grants Glimmer, its listed items, a world flag, and a
  codex entry.
- **Music.** Each boss has an intro stem and a loop; phase 2 adds a stem.

### Fairness rules

Every boss in the game must satisfy these rules. The boss review in
[WP-089](10-roadmap.md#wp-089) checks each one.

- Every attack has a telegraph of at least 12 ticks: a pose, a glow, a sound, or a
  ground marker. Heavy attacks telegraph for 18-36 ticks.
- A delayed attack only delays the second or later hit of a combo, never the first.
- Every combo ends in a punish window of at least 20 ticks.
- No attack tracks through the whole of a roll. Tracking stops at least 4 ticks before
  the active frames.
- Grabs and unblockable attacks have a unique color flash.
- The arena has at least two escape directions and no geometry that traps the camera.
- The player can always see the boss: no arena is darker than the darkness rule allows,
  and very large bosses get a camera pull-back.

## Boss roster

The game has 12 bosses. Four are required: B01, two of the four Great Sigil bosses
(B06, B07, B09, B10), and B12.

| ID | Boss | Location | Archetype | Required | HP | Phases |
|---|---|---|---|---|---|---|
| B01 | Hollow Sentry | Waking Hollow | Tutorial knight | Yes | 1,200 | 1 |
| B02 | The Briar Stag | Verge, Briar Thicket | Tethered beast | No | 2,600 | 2 |
| B03 | Bridgewarden Corvane | Graymantle Bridge | Duelist | No | 2,200 | 1 |
| B04 | Mossgut Brood | D1 Mossgut Cave | Swarm nest | No | 2,400 | 2 |
| B05 | Ossuary Keeper | D2 Sunken Ossuary | Puppeteer | No | 3,000 | 2 |
| B06 | The Drowned Choir | Drowned Cathedral | Trio | Great Sigil | 3 x 1,800 | Per member |
| B07 | Castellan Voss, the Overfit Warden | Castellan's Hall | Adaptive duelist | Great Sigil | 6,000 | 2 |
| B08 | Forgeheart Colossus | D4 Cinderdeep Mine | Colossus | No | 7,000 | 2 |
| B09 | The Lamplight Matriarch | Lamplight Hollow | Flyer | Great Sigil | 6,500 | 2 |
| B10 | The Bellwright | Bellwright's Steps | Rhythm giant | Great Sigil | 8,000 | 2 |
| B11 | The Bound Exile | D7 Bound Ring | Teleporting caster | No | 4,500 | 2 |
| B12 | The Unspooled | Lattice Spire | Final, rebirth | Yes | 9,000 + 9,000 | 2 |

### B01 Hollow Sentry

An empty suit of armor animated by a lantern in its chest. It teaches the core loop:
read the telegraph, roll through, punish.

- **Arena:** A round chamber, 18 m across, with four pillars.
- **Moves:** Overhead slam (30-tick wind-up, lantern flares), horizontal sweep (roll
  through it, not away), shield bash at close range (14-tick wind-up), and a 3-hit
  combo ending in a 40-tick punish window.
- **Weakness:** Blunt 1.2x. **Immunities:** Poison, Blight (it's empty).
- **Reward:** A Wick Bead; opens the root-door to the hub.

### B02 The Briar Stag

A giant stag grown over with thorns, roaming the Briar Thicket.

- **Arena:** Open thicket with thick trees.
- **Moves:** Hoof scrape then charge along a straight line (scrapes twice, 30 ticks; a
  charge that hits a tree stuns it for 60 ticks), antler toss, leap stomp (shadow
  marker), and thorn burst (bristles for 36 ticks, then a Rupture aura).
- **Phase 2 at 50%:** Thorns grow longer, charges chain twice, and the thorn burst
  leaves bramble patches that slow.
- **Immunities:** Rupture. **Reward:** Briar Axe, Bloodbloom Oil x3.

### B03 Bridgewarden Corvane

A knight who holds the bridge out of habit. A pure duel and the game's parry teacher.

- **Arena:** The bridge, 10 m wide with railings and a broken section.
- **Moves:** Shield advance, three-hit halberd combo (every hit parryable, the third
  delayed), sweeping turn, and a lunge that punishes healing within 6 m (telegraphed by
  a shout, 18 ticks).
- **Note:** The cliff path under the bridge lets players skip him.
- **Immunities:** Delirium. **Reward:** Starglass, a Wick Bead.

### B04 Mossgut Brood

A bloated fungal mother embedded in the cave wall, surrounded by egg sacs.

- **Arena:** A low cave chamber; the ceiling blocks overhead attacks.
- **Moves:** The mother doesn't move. She spits Poison globs (arcing, ground marker) and
  sweeps a tendril at close range. Egg sacs spawn Marsh Crawlers every 300 ticks.
- **Phase 2 at 50%:** She tears free and crawls slowly, spraying Poison in a cone.
- **Mechanic:** Destroying all four egg sacs exposes a weak point (2x damage).
- **Weakness:** Fire 1.5x. **Immunities:** Poison, Blight. **Reward:** Fungal Knife; the
  lift to Hollowdeep.

### B05 Ossuary Keeper

A skeletal warden with a bell-lantern who wakes the dead.

- **Arena:** A flooded crypt with raised platforms.
- **Moves:** Bell swing combo, lantern beam (sweeping, 24-tick charge), and Raise:
  revives up to three skeletons from the floor.
- **Phase 2 at 50%:** The water rises 0.4 m (slows the player) and Drowned join from the
  edges. The Keeper picks up a fallen skeleton's weapon and gains its moves.
- **Weakness:** Blunt, Radiance. **Immunities:** Poison, Rupture.
- **Reward:** the Bone Sentinel Echo.

### B06 The Drowned Choir

Three singers who drowned in their own cathedral, still singing. Kill order matters.

- **Members:** The **Cantor** (caster; heals the others, Radiance beams), the
  **Bellbearer** (tank; bell slam, ring stun wave), and the **Lamenter** (fast flanker;
  Rime knives).
- **Arena:** A flooded nave. Side aisles hold waist-deep water, where Storm deals 1.5x.
- **Per-member phases:** When a member dies, the others sing a new verse and gain one
  move each: the Cantor gains a homing chorus, the Bellbearer gains a charge, and the
  Lamenter gains a teleport-stab.
- **Immunities:** Each member resists a different status.
- **Reward:** Great Sigil of Stillwater, Choir Glaive.

### B07 Castellan Voss, the Overfit Warden

The castle's lord, who memorized every duel he ever fought. Phase 2 adapts to you.

- **Arena:** A great hall with banners and chandeliers.
- **Phase 1 moves:** Greatsword combos (one delayed), dagger off-hand stab, guard stance
  with a counter, and a leaping slam.
- **Phase 2 at 60%:** He *overfits*. During phase 1, the core counts the player's actions
  in six categories: roll left, roll right, roll back, parry, guard, and ranged or spell.
  In phase 2 he gains a counter for the top two: wider sweeps toward the favored roll
  side, delayed swings against parry, guard-break kicks against guarding, and closing
  dashes against ranged. Chandeliers fall on telegraphed markers.
- **Design intent:** The fight rewards varied play and is a quiet joke about overfitting.
  The counting is deterministic and visible in the dev overlay.
- **Immunities:** Drowse. **Reward:** Great Sigil of Graymantle; the way to the
  Castellan's desk and its Chronicle Page.

### B08 Forgeheart Colossus

A forge golem as tall as the cavern, its heart a furnace.

- **Arena:** A forge cavern with lava channels at the edges.
- **Moves:** Stomp (molten ground marker glows for 30 ticks), arm sweep, furnace breath
  (Fire cone, 36-tick wind-up with the chest opening), and a two-handed slam that
  sends a shockwave along the ground.
- **Mechanic:** Each leg has its own stance meter. Breaking one makes it kneel and
  exposes the furnace for a critical.
- **Phase 2 at 40%:** The heart ignites; lava creeps across the floor over time.
- **Weakness:** Blunt 1.2x. **Absorption:** Fire 80%. **Immunities:** Poison, Blight,
  Rupture. Its Rime resistance is low, so Rime procs quickly.
- **Reward:** Forgeheart Maul, the Forgeheart Ember (a key item for the Smith).

### B09 The Lamplight Matriarch

A vast moth, mother of every light in Hollowdeep.

- **Arena:** A round cavern ringed by six unlit lamps.
- **Air phase:** Dives (a shadow marks the landing for 24 ticks), Drowse dust clouds,
  and light beams. After four dives, or a stance break, she lands.
- **Ground phase:** Wing buffets, a lunge, and a spinning wing sweep. Returns to the air
  after 600 ticks.
- **Phase 2 at 50%:** She drains the cavern's light; only her glowing abdomen is visible.
  The player can light the six lamps with Fire, the *smoltorch*, or Lantern Flare;
  each lit lamp restores light and stuns her briefly when she flies past it.
- **Weakness:** Fire 1.3x. **Immunities:** Drowse.
- **Reward:** Great Sigil of the Deep Lamp.

### B10 The Bellwright

A giant smith who hauls a frozen bell on a chain, ringing it in rhythm.

- **Arena:** The Bellwright's open-air forge at the top of Bellwright's Steps, on frozen
  ground. Cracked ice zones break after two bell tolls.
- **Rhythm:** The fight runs on a beat, one every 40 ticks. Hammer combos land on beats,
  and every fourth beat the bell tolls: a Rime shockwave that can be jumped or rolled.
  The music plays the beat, so players can fight by ear.
- **Phase 2 at 50%:** The chain snaps; he swings the bell as a flail with long reach,
  and the beat speeds up to one every 32 ticks.
- **Weakness:** Fire 1.3x. **Immunities:** Rime, Drowse.
- **Reward:** Great Sigil of Rimeshelf, Bellwright's Mallet.

### B11 The Bound Exile

A sorcerer sealed in a ring of stones for a crime no one remembers.

- **Arena:** A ring of twelve standing stones, about 42 m across.
- **Moves:** Teleports between the stones (it appears on the ground in front of one), summons orbiting orbs, fires an Aether beam with
  a visible glyph ring (the cast bar). Hitting the Exile during the glyph ring
  interrupts and staggers.
- **Phase 2 at 50%:** Splits into two images. Only the real one casts; its glyph ring
  is a slightly different color (and a different shape, for colorblind players).
- **Immunities:** Delirium. **Reward:** Exile's Stylus, Unbinding Lance.

### B12 The Unspooled

The thing that pulled the thread. The last fight.

- **Phase 1, the Weaver:** A tall figure of threads. Thread lashes with whip reach,
  loom lines (glowing lines across the arena that become beams after 30 ticks), a
  stitching grab (white flash), and a spinning cut.
- **Phase 2, the Unspooled:** After its first death, the Weaver unravels and reforms
  as a larger, many-limbed shape. It uses one signature move from each Great Sigil boss
  the player defeated (borrowed from their move tables), plus its own phase moves. The
  fight is different depending on which Sigils the player chose.
- **Arena:** The top of the Spire: a round loom floor, open sky, Lattice threads above.
- **Immunities:** Drowse, Delirium. **Reward:** The ending; the Courier's Letter (contact);
  the credits, which are the portfolio's About section.

## Echo summons

Echoes are summoned at an Echo Stone with the Echo Bell and cost FP. The Archivist gives
the Echo Bell on the first visit to Hollowhearth. They have their own
AI files and follow the same attack-token rules as enemies (in reverse).

| ID | Echo | Role | FP | Source |
|---|---|---|---|---|
| `echo_politellama` | PoliteLlama (R) | Tank; bows before each attack and taunts. If you bow to it after summoning, it fights at +20% damage (you asked nicely) | 40 | The Courier's quest |
| `echo_bone_sentinel` | Bone Sentinel | Shield and spear; holds aggro | 50 | B05 reward |
| `echo_choir_wisp` | Choir Wisp | Caster; heals the player slowly | 60 | Drowned Belfry |
| `echo_briar_hounds` | Briar Hounds | Three fast hounds; Rupture | 45 | Briar Thicket, after B02 |
