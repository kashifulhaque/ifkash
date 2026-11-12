# World

This document defines the realm of Vael: its regions, dungeons, checkpoints, gates,
shortcuts, and secrets, plus the rules for building an area from data. Read it before
any WP in the `world` lane.

All names here are original. Don't rename them to anything from another game. See
[the IP rules](00-vision-and-scope.md#ip-rules).

## Design rules

Every area follows these rules. They come from what makes small souls-likes feel large;
for the reasoning and sources, see [the research notes](12-research-notes.md#world-design).

- **One landmark per region,** visible from that region's entry route. The Lattice Spire
  is visible from every outdoor area.
- **One loop per region.** Each region has at least one shortcut that opens from the
  far side and connects back to a Lantern the player already found.
- **Vertical stacking.** At least one place per region sits directly above or below
  another place the player has been. Hollowdeep lies under the Verge and Mirrormere.
- **Mechanics teach once, then become clues.** Each dungeon type has one signature
  mechanic. The first time the player meets it, the space teaches it safely.
- **Visible threat before hidden threat.** Show an enemy type in the open before it
  ambushes. At most one ambush per dungeon.
- **Hard gates are rare.** There are exactly three in the whole game, listed in
  [Gates](#gates-and-progression). Everything else is a soft gate: the area is open,
  but its enemies signal that it's late-game.
- **No dead time.** If a walk between two interesting things takes more than 45 seconds
  at a run, add something: an enemy camp, an item, a vista, or a shortcut.

## World map

The realm is about 1.2 by 1.2 kilometers of outdoor space, plus an underground layer.
The following diagram shows the layout, with north at the top:

```text
                     +---------------------------+
                     |        RIMESHELF          |   snow highland, lv 30-40
                     |  Bound Ring   Bellwright's |
                     |  Whitewarden  Steps (GS4) |
                     +-------------+-------------+
                                   | castle back gate (one-way at first)
                     +-------------+-------------+
                     |    CASTLE GRAYMANTLE      |   legacy dungeon, lv 20-30
                     |  3 tiers on a sea cliff   |
                     +-------------+-------------+
                                   | Graymantle Bridge (field boss B03)
+----------------------------------+----------------------------------+
|              THE VERGE           |            MIRRORMERE            |
|  lv 1-15                         |  lv 10-25                        |
|  Waking Hollow (start)           |  Reedwater, Stillwater Village   |
|  Ninefold Chapel, Aqueduct       |  Drowned Cathedral (GS2)         |
|  Mossgut Cave ---lift---+        |  Sunken Ossuary, Drowned Belfry  |
|               +--------[LATTICE SPIRE plateau]--------+              |
+-----------------------+----------+---------------------+------------+
                        | Cinderdeep Mine (tunnel)        | Belfry lift
                        v                                 v
              +-----------------------------------------------+
              |            HOLLOWDEEP (underground)           |  lv 18-30
              |  Fungal Shore, Glasswing Grotto,              |
              |  Lamplight Hollow (GS3)                       |
              +-----------------------------------------------+

HOLLOWHEARTH (hub) is reached by fast travel from any Lantern after boss B01.
```

## Regions

| Region | Theme | Level range | Landmark | Size | Lanterns |
|---|---|---|---|---|---|
| Hollowhearth (hub) | Candle-lit refuge inside a hollow tree | n/a | The Hearth | 60 x 60 m | 1 |
| The Verge | Ashen meadow, broken aqueduct, windmill ruins | 1-15 | The broken aqueduct arch | 400 x 400 m | 5 |
| Mirrormere | Mirror-still lake, flooded village, reed marsh | 10-25 | Drowned Cathedral on its island | 350 x 300 m | 4 |
| Hollowdeep | Fungal caverns, glowing lake, abandoned mine | 18-30 | The Lamp Tree, a giant glowing fungus | 300 x 300 m, 60 m below ground | 4 |
| Castle Graymantle | Cliff fortress over the sea | 20-30 | The keep's broken bell tower | 220 x 200 m, 3 tiers | 4 |
| Rimeshelf | Frozen highland, bell foundry, catacombs | 30-40 | The Great Bell frozen in the cliff | 300 x 250 m | 4 |
| The Lattice Spire | A tower of woven light, partly unraveled | 40+ | Itself | 80 m tall | 2 |

### Hollowhearth (hub)

A hollow in the roots of a dead giant tree, lit by candles. It's the safe place where
the portfolio's people live.

- **Unlock:** Defeat B01. A root-door opens in Waking Hollow; afterward, any Lantern can
  travel here.
- **NPCs:** the Archivist (education, Letters, codex), the Smith (upgrades), the
  Cartographer (map, work history), the Courier (contact), and the Peddler (shop). See
  [NPCs and questlines](06-progression-and-systems.md#npcs-and-questlines).
- **Features:** Lantern L00, the Archive shelves (blog Letters), the Mirror (change
  appearance preset and dye), and the Ledger Stand (résumé).
- **Rules:** No combat. Weapons are sheathed. Movement is walk-only.

### The Verge

The starting region. Open, readable, and forgiving. Teaches exploration.

- **Sub-areas:** Waking Hollow (tutorial cave), Verge Crossroads, the Aqueduct and its
  Steps, Ninefold Chapel (ruins), the Windmill Rise, the Briar Thicket, and the Spire
  Approach.
- **Enemies:** Husks, Thorn Hounds, Lantern Thieves, Chapel Acolytes, one Bramble Brute
  elite at the windmill.
- **Field boss:** B02 the Briar Stag, roaming the Briar Thicket.
- **Mini-dungeons:** Mossgut Cave.
- **Loop:** The aqueduct top runs from Aqueduct Steps (L03) to the chapel roof. Kick down
  the chapel ladder to connect L04 back to L03.
- **Vertical stack:** The Mossgut Cave lift descends into Hollowdeep.
- **Secrets:** A hollow wall in the chapel crypt hides the *banana.cpp* curved sword. A
  windmill rooftop chest holds the *Shrike-Lite Circuit* charm.
- **Portfolio:** Chronicle Page slot 1 (the earliest work entry) at Ninefold Chapel. For
  how work entries fill Chronicle Page slots, see
  [the portfolio spec](08-portfolio-integration.md#chronicle-pages).

### Mirrormere

A lake so still it reflects the Spire. Teaches status effects and water hazards.

- **Sub-areas:** Reedwater Landing, Stillwater Village (flooded to the knees), the Reed
  Marsh (Blight water), the Causeway, the Drowned Cathedral island.
- **Enemies:** Drowned (slow, grab), Reed Stalkers (ambush from reeds), Marsh Crawlers
  (poison), Bell Acolytes (casters), Choir Knights (elite).
- **Main boss:** B06 the Drowned Choir (Great Sigil 2) in the Drowned Cathedral.
- **Mini-dungeons:** Sunken Ossuary (catacomb), Drowned Belfry (sunken ruin).
- **Loop:** A sluice lever in the Belfry lowers the lake level by 1.5 meters. That opens
  a dry path from Stillwater Village to the Cathedral's side door near L09.
- **Vertical stack:** The Belfry's bell shaft is a lift down to Hollowdeep's Fungal Shore.
- **Secrets:** A rowboat moored behind the village leads to an islet with the
  *NoPokeDB* shield.
- **Portfolio:** Chronicle Page slot 2 in the Stillwater Village chief's house.

### Hollowdeep

An underground region below the Verge and Mirrormere. Dark, beautiful, and the most
vertical place in the game. Teaches light and verticality.

- **Sub-areas:** Cinderdeep Mine (tunnel from the Verge), the Fungal Shore, the Glowing
  Lake, Glasswing Grotto, the Root Stair, and Lamplight Hollow.
- **Lighting rule:** Outside the glowing zones it's dark. The *smoltorch* relic (found
  in the mine) or the Lantern Sword's Lantern Flare Art lights the way. Darkness is never required to be solved:
  the critical path stays inside glowing zones.
- **Enemies:** Cave Moths, Lamp Mites (swarm), Miners (lantern-carrying husks), Shard
  Golems (tank), Glasswing Stalkers (ambush).
- **Main boss:** B09 the Lamplight Matriarch (Great Sigil 3).
- **Mini-dungeons:** Cinderdeep Mine (with B08), Glasswing Grotto.
- **Loop:** The Root Stair climbs from the Glowing Lake up to the Spire plateau's
  underside and opens a one-way trapdoor next to L22.

### Castle Graymantle (legacy dungeon)

See [Legacy dungeons](#legacy-dungeons).

### Rimeshelf

A frozen highland reached through Castle Graymantle's back gate. The late-game region.
Teaches stamina management in deep snow and frost buildup.

- **Sub-areas:** Rimeshelf Pass, the Frozen Foundry, Whitewarden Catacombs, the Bound
  Ring, and Bellwright's Steps.
- **Terrain rule:** Deep snow slows movement by 25 percent. Packed paths don't.
- **Enemies:** Rime Wolves (pack), Frost Husks, Foundry Smiths (heavy hammer elites),
  Bell Wardens (elite, bell-strike stun), Whitewarden Skeletons (revive unless struck with
  Radiance or a critical).
- **Main boss:** B10 the Bellwright (Great Sigil 4).
- **Mini-dungeons:** Whitewarden Catacombs, the Bound Ring.
- **Loop:** A collapsed foundry chimney drops the player back into the castle's upper
  tier at L16.
- **Portfolio:** Chronicle Page slot 4 in the Frozen Foundry's office. It stays empty
  until the portfolio has five or more work entries.

### The Lattice Spire (legacy dungeon)

See [Legacy dungeons](#legacy-dungeons).

## Legacy dungeons

Legacy dungeons are long, interconnected, hand-designed spaces. Each has a main route,
an alternative route, and at least two shortcuts.

### Castle Graymantle

A three-tier fortress on a sea cliff. About 25-35 minutes on a first visit.

The following list describes the castle's tiers in order:

1. **Bridge and Barbican (L14, L15).** The Graymantle Bridge holds B03, Bridgewarden
   Corvane. A cliff path under the bridge lets players skip him and climb into the
   Barbican's cistern. The Barbican has a gatehouse lever that opens the main gate as
   the first shortcut back to L14.
2. **Lower Ward.** A courtyard with a ballista on the wall, an armory (heavy armor set),
   a kennel of Thorn Hounds, and a chapel with Chapel Acolytes. Two routes go up: the
   main stair (guarded by two Gray Knights) and the rampart route (archers, narrow
   walkways, a drop to an item ledge).
3. **Ramparts (L16).** Wind, archers, and a Gray Knight captain (elite). A lift from the
   ramparts down to the Lower Ward is the second shortcut.
4. **The Keep.** Library (Formulae and the *Gradient Rod* stylus), the dining hall
   (ambush of Husk servants), and the bell tower climb.
5. **Castellan's Hall (L17).** B07 Castellan Voss, the Overfit Warden (Great Sigil 1).
   After the fight, the back gate to Rimeshelf opens.

**Secrets:** A hollow wall in the library leads to the *Boo* mask. The bell-tower roof
has the Art Scroll for *Windward Parry*.

**Portfolio:** The Castellan's desk holds Chronicle Page slot 3.

### The Lattice Spire

The final dungeon, at the center of the realm. About 15-20 minutes.

- **Unlock:** The Spire Gate opens when the player holds any two Great Sigils (hard gate).
- **Structure:** An outer spiral ramp that wraps the tower. It's partly unraveled, so
  gaps need short jumps; each gap has a fallback ladder for players who struggle with
  jumping. Three floors inside hold an enemy gauntlet of elite versions of every earlier
  region's enemies (one floor per region you've cleared).
- **Lanterns:** L22 at the foot, outside the Spire Gate (so the Root Stair shortcut
  works before the gate opens), and L23 before the final arena.
- **Final boss:** B12 the Unspooled, two phases.
- **Portfolio:** The top floor holds the last Chronicle Page slot, which always holds
  the current role. The ending hands over the Courier's Letter (contact).

## Mini-dungeons

Each mini-dungeon takes 5-12 minutes, has one signature mechanic, and ends in a boss or
an elite guardian plus a reward.

| ID | Name | Region | Type | Signature mechanic | End encounter | Reward |
|---|---|---|---|---|---|---|
| D1 | Mossgut Cave | Verge | Cave | Low ceilings block overhead attacks; fight with thrusts and horizontal swings | B04 Mossgut Brood | *Fungal Knife* dagger; lift to Hollowdeep |
| D2 | Sunken Ossuary | Mirrormere | Catacomb | Water-level lever: lowering water opens rooms but wakes the Drowned | B05 Ossuary Keeper | Bone Sentinel (Echo) |
| D3 | Drowned Belfry | Mirrormere | Sunken ruin | Ringing the bell stuns nearby enemies and opens the bell-shaft lift | Elite: Belfry Choir Knight | Lake sluice lever, *Choir Litany* |
| D4 | Cinderdeep Mine | Verge to Hollowdeep | Tunnel | Mine carts on rails: ride one down, push one to break a wall | B08 Forgeheart Colossus | *smoltorch* relic, Forgeglass Shards |
| D5 | Glasswing Grotto | Hollowdeep | Cave | Crystal mirrors reflect light beams; aim them to open sealed doors | Guardian: two Glasswing Stalkers with elite stats | *Shardglass Spear* |
| D6 | Whitewarden Catacombs | Rimeshelf | Catacomb | Skeletons revive until a Warden Brazier is lit; light braziers to stop revives | Elite: Twin Wardens | *Whitewarden Seal* (Litanies) |
| D7 | The Bound Ring | Rimeshelf | Sealed arena | A ring of standing stones; touching the sigil seals you in with its prisoner | B11 the Bound Exile | *Exile's Stylus*, Formula *Unbinding Lance* |

### Dungeon layout template

Every mini-dungeon follows this template, so agents can build new ones consistently:

1. **Entrance** with a Lantern outside (or just inside) the door.
2. **Teaching room:** introduces the signature mechanic with no enemies, or one weak
   enemy.
3. **Test room:** uses the mechanic with enemies.
4. **Branch:** a side path to an optional item, ending in a dead end or a secret.
5. **Twist room:** combines the mechanic with an ambush or a hazard.
6. **Shortcut:** a door or lift that opens back to the entrance.
7. **Boss or elite room,** behind a fog gate, with the reward on a pedestal after it.

Rooms are 12-30 meters across, built from kit pieces on the 2-meter grid. A
mini-dungeon has 5-9 rooms.

## Checkpoints

There are 24 Lanterns. Each one is a rest point, a respawn point, a level-up point, and
a fast-travel node once lit. The following table lists them all:

| ID | Name | Region | Notes |
|---|---|---|---|
| L00 | The Hearth | Hollowhearth | Hub |
| L01 | Waking Hollow | Verge | Start point |
| L02 | Verge Crossroads | Verge | First outdoor Lantern |
| L03 | Aqueduct Steps | Verge | |
| L04 | Ninefold Chapel | Verge | |
| L05 | Mossgut Mouth | Verge | Outside D1 |
| L06 | Reedwater Landing | Mirrormere | |
| L07 | Stillwater Village | Mirrormere | Also serves D3, which opens off the village shore |
| L08 | Ossuary Stair | Mirrormere | Outside D2 |
| L09 | Cathedral Causeway | Mirrormere | Before B06 |
| L10 | Cinderdeep Shaft | Hollowdeep | Inside D4 |
| L11 | Fungal Shore | Hollowdeep | |
| L12 | Glasswing Mouth | Hollowdeep | Outside D5 |
| L13 | Lamplight Hollow | Hollowdeep | Before B09 |
| L14 | Graymantle Bridge | Castle | Before B03 |
| L15 | Barbican Cistern | Castle | |
| L16 | Windward Rampart | Castle | |
| L17 | Castellan's Antechamber | Castle | Before B07 |
| L18 | Rimeshelf Pass | Rimeshelf | |
| L19 | Whitewarden Gate | Rimeshelf | Outside D6 |
| L20 | Bellwright's Steps | Rimeshelf | Before B10 |
| L21 | The Bound Ring | Rimeshelf | Outside D7 |
| L22 | Spire Foot | Spire | |
| L23 | The Loom | Spire | Before B12 |

Placement rules:

- One Lantern per 3-5 minutes of expected field travel.
- One Lantern within 20 seconds of every required boss's fog gate.
- No Lantern inside a mini-dungeon's interior, except L10 (the mine is long).
- Resting respawns all non-boss enemies in the world and refills flasks.

## Gates and progression

There are exactly three hard gates. Everything else is open from the start and gated
only by difficulty.

| Gate | Requirement | Opens |
|---|---|---|
| Waking Hollow root-door | Defeat B01 Hollow Sentry | The Verge and the hub |
| Castle back gate | Defeat B07 Castellan Voss | Rimeshelf |
| Spire Gate | Hold any 2 of the 4 Great Sigils | The Lattice Spire |

The following diagram shows the intended routes. Arrows mean "leads to." Every region
except Rimeshelf connects to at least two others, so players can always turn back.
Rimeshelf has two routes to the castle: the back gate and the foundry chimney.

```text
Waking Hollow --B01--> Verge --+--> Mirrormere --Belfry lift--> Hollowdeep
                               |                                  ^
                               +--Mossgut lift / Cinderdeep Mine--+
                               |
                               +--> Graymantle Bridge --> Castle --B07--> Rimeshelf
                               |
                               +--> Spire Gate (2 Great Sigils) --> Lattice Spire --> B12
```

### Expected critical path

| Step | Content | Minutes |
|---|---|---|
| 1 | Waking Hollow tutorial and B01 | 8 |
| 2 | Verge exploration to L02-L04 | 10 |
| 3 | Mirrormere and B06, or Castle and B07 | 25 |
| 4 | Second Great Sigil (any other) | 25 |
| 5 | Spire and B12 | 15 |
|  | **Main path total** | **83** |

A full clear adds all mini-dungeons (about 55 minutes), the other two Great Sigils
(about 40 minutes), and secrets.

## Traversal and world mechanics

The following mechanics are available to every area. Each lives in
`game/crates/core/src/world_state/` and is placed through `entities.ron`.

| Mechanic | Behavior | Notes |
|---|---|---|
| Jump | 1.1 m height, 3.5 m running distance | Gaps on the critical path are 2.5 m or less |
| Ladder | Climb state; can't attack; drop off at any rung | Kick-down ladders start raised and drop when kicked from above |
| Lift | Kinematic platform between two stops; called by a lever at each stop | Actors on it move with it; never strand the player: both stops have a call lever |
| Door | Locked, one-way (opens from one side), or keyed | One-way doors show a "Opens from the other side" prompt |
| Lever | Toggles a linked entity: door, lift, water level, bridge | State saved in world flags |
| Hollow wall | Looks like a wall; one hit reveals a passage | Up to 1 per region; a faint hum near it is the hint |
| Fog gate | Boss arena entrance; walk through to enter; sealed during the fight | Shows the Echo Stone prompt if one is near |
| Water | Shallow (under 0.6 m): slows 20%. Deep (over 1.4 m): blocks with an invisible wall at the edge, no swimming | |
| Blight water | Shallow water that builds Blight status | Reed Marsh only |
| Water level | A pool's depth follows a lever (`raised_depth_m`) or a world flag (`lowered_by`, `lowered_depth_m`); a pool's surface is `floor_y_m` plus its depth | Mirrormere's lake drops 1.5 m when the Belfry raises `sluice.mirrormere.open`; see `mechanics/water_level.rs` |
| Flag lever | A lever with target `Flag(name)` raises a world flag for good (WP-061) | The Ossuary's drain levers (`ossuary.drain_t`, `ossuary.drain_f`) and the Belfry's sluice; see `mechanics/bell.rs` |
| Bell | A lever with target `Bell(radius_m:, stun_ticks:, opens:)`: ringing it stuns every living enemy in range (they kneel, open to a riposte) and opens the door it names (WP-061) | The Belfry's four bells; the bell-shaft lift's gate opens to bell C; see `mechanics/bell.rs` |
| Sleeping enemy | An enemy spawn with `wakes_on: Some(flag)` doesn't think until the flag is set, so it can be backstabbed asleep (WP-061) | The Ossuary's Drowned wake when lever 2 drains the Flooded Crypt; see `mechanics/slumber.rs` |
| Deep snow | Slows 25%, sprint costs 1.5x stamina | Rimeshelf |
| Fall damage | Over 8 m: 4% of max HP per meter above 8; over 20 m: death | Except into deep water, which isn't reachable |
| Darkness | Visibility reduced to 6 m outside light sources (14 m with the *smoltorch* relic or a Lantern Flare); a `Darkness` rule and `Glow` zones in `entities.ron`; a Lantern lights 9 m around it | Hollowdeep only, never on the critical path: `lf-tools bake` warns (`dark-critical-path`) when the walk from the spawn through the Lanterns leaves the glowing zones; see `mechanics/darkness.rs` |
| Mine cart | A `Cart` on a rail: `Ride` carries the player down and back (held in place, no attacks); `Push(wall)` rolls into a hollow wall and breaks it | Cinderdeep Mine; a `sturdy` hollow wall opens only to a pushed cart, not to a swing; see `mechanics/mine_cart.rs` |
| Trapdoor | A hatch overhead that opens with `INTERACT` from below and drops a kick-down ladder; never opens from above | The Root Stair; see `mechanics/trapdoor.rs` |
| Messages | Readable ground messages (authored and, later, player-written) | See [async features](06-progression-and-systems.md#asynchronous-features) |
| Treasure chest | Opens with `INTERACT` | One mimic chest exists, in the castle armory |
| Item glint | A pickup on the ground | Glints are visible from 25 m |

## Procedural dressing

Areas look handcrafted at the scale of the player's path and use rules everywhere else.
Agents must not hand-place more than about 50 decorative props per area. Use scatter
rules in `dressing.ron` instead.

The following code sample shows a scatter rule:

```ron
// game/data/levels/verge/dressing.ron
(
  rules: [
    (
      tag: "ash_grass",           // kit catalog tag; picks a random variant
      density: 0.6,               // per square meter
      min_spacing: 0.8,           // Poisson-disk radius in meters
      max_slope_deg: 30,
      avoid: [Path(width: 2.0), Lantern(radius: 4.0), Spawn(radius: 2.0)],
      region_mask: "meadow",      // named polygon in area.ron
      scale: (0.8, 1.3),
      collider: false,
    ),
    (
      tag: "dead_tree",
      density: 0.01,
      min_spacing: 7.0,
      max_slope_deg: 20,
      avoid: [Path(width: 4.0)],
      region_mask: "meadow",
      scale: (0.9, 1.4),
      collider: true,             // trunk capsule from the kit catalog
    ),
  ],
)
```

Scatter uses the area's own seed, so dressing is the same every time and the colliders
it adds are deterministic.

## Build an area

When a WP builds or rebuilds an area, follow these steps:

1. Write `area.ron` with bounds, the cell grid, named region polygons, and connections.
2. Write `terrain.ron`. Start from a noise preset, then add sculpt operations until the
   landmark and the paths read clearly from the entry point.
3. Lay out the critical path in `layout.ron` with kit pieces. Greybox first: use only
   the `block_*` kit pieces until the layout plays well.
4. Place Lanterns, spawns, items, and mechanics in `entities.ron`.
5. Run `cargo run -p lf-tools -- bake AREA` and fix every warning (unreachable Lantern,
   item off the nav grid, spawn inside a collider, and so on).
6. Play it with `/game?dev=1&spawn=LANTERN_ID`. Time the walk between Lanterns and fix
   any stretch over 45 seconds without content.
7. Swap greybox pieces for art kit pieces and add `dressing.ron`.
8. Take screenshots from at least these viewpoints: each Lantern, the landmark from the
   entry, and each boss arena. Use the
   [visual verification protocol](11-agent-playbook.md#verify-visually).
9. Add or update a golden replay that walks the critical path through the area.
