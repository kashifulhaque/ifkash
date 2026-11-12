# Arsenal and builds

This document lists every weapon class, weapon, Art, spell, Charm, armor set,
consumable, starting class, and supported build. Read it before any WP that adds
equipment. Rules for how these items behave live in [the combat spec](03-combat.md).

Numbers are starting values. The source of truth is `game/data/`; when a number
changes there, this document doesn't need to change unless the item's role changes.

## Design rules

- **Every weapon class has a distinct verb.** If two classes play the same, cut one.
- **Every build is supported end to end:** its core kit (the first weapon in the
  build's row, its Art, one Charm, and, for caster builds, four spells) is reachable
  before the second Great Sigil on at least one route. Items marked *late* are upgrades.
- **Unique weapons have fixed Arts and can't be tempered.** Common weapons can.
- **Relic items** are portfolio projects. They must be good, not joke items, so
  players actually use them and read them. See
  [the portfolio spec](08-portfolio-integration.md#relics).
- **Fixed action IDs across classes.** Every class maps to the same input verbs (light,
  heavy, charged heavy, guard or parry, Art). This keeps the RL action space stable.

## Weapon classes

The game has 14 weapon classes. The following table describes how each one plays:

| Class | Verb and feel | Speed | Reach | Typical scaling | Moveset file |
|---|---|---|---|---|---|
| Dagger | Fast stabs, best criticals (crit 1.4) | Very fast | 1.6 m | Finesse | `dagger.ron` |
| Straight sword | Balanced slashes and thrusts; the baseline | Fast | 2.0 m | Might and Finesse | `straight_sword.ron` |
| Greatsword | Wide, slow sweeps with hyperarmor | Slow | 2.6 m | Might | `greatsword.ron` |
| Curved sword | Flowing combos, chained spins | Fast | 1.9 m | Finesse | `curved_sword.ron` |
| Thrusting sword | Long pierces, counter-hit specialist | Fast | 2.4 m | Finesse | `thrusting_sword.ron` |
| Axe | Heavy chops, status-friendly | Medium | 1.9 m | Might | `axe.ron` |
| Hammer | Blunt damage, high poise damage | Medium | 1.8 m | Might | `hammer.ron` |
| Colossal | Huge two-handed slams; breaks stance | Very slow | 3.0 m | Might 30+ | `colossal.ron` |
| Spear | Pierce at range; attacks while guarding | Medium | 2.8 m | Might and Finesse | `spear.ron` |
| Halberd | Wide sweeps at range | Medium | 2.9 m | Might | `halberd.ron` |
| Fist and claw | Rapid strings, rolling attacks | Very fast | 1.3 m | Finesse | `fist.ron` |
| Bow | Ranged; aim while locked on or free-aim | Medium | 40 m | Finesse | `bow.ron` |
| Shield | Guard, parry (bucklers), bash | n/a | 1.2 m | Might | `shield.ron` |
| Catalyst | Casts Formulae (stylus) or Litanies (reliquary) | n/a | n/a | Insight or Devotion | `catalyst.ron` |

### Temper

At the Smith, a common weapon can be tempered to change its scaling and add an element.
Each temper needs a Temper Whetstone, found once in the world (5 in total).

| Temper | Effect | Whetstone location |
|---|---|---|
| Plain | Default scaling | n/a |
| Heavy | Might scaling x1.5, Finesse scaling removed | Verge, windmill |
| Keen | Finesse scaling x1.5, Might scaling x0.5 | Mirrormere, village |
| Lucid | 40% of base becomes Aether, adds Insight scaling 0.7 | Castle library |
| Sacred | 40% of base becomes Radiance, adds Devotion scaling 0.7 | Whitewarden Catacombs |
| Rimed | 25% of base becomes Aether, adds Rime buildup 40 | Rimeshelf foundry |

## Weapon list

The game has 36 weapons. Relic weapons (marked R) hold a portfolio project. **Unique**
weapons are relics, boss rewards, and weapons with a unique Art (Lantern Sword,
Glassneedle Estoc, Shardglass Spear). Every other weapon is **common**. Content data
marks unique weapons with `unique: true`.

| ID | Name | Class | Base | Scaling M/F/I/D | Requirement | Weight | Status | Default Art | Where |
|---|---|---|---|---|---|---|---|---|---|
| `wpn_fungal_knife` | Fungal Knife | Dagger | 70 | -/0.8/-/- | F 9 | 2 | Poison 50 | Feint Step | D1 boss reward |
| `wpn_parrying_dirk` | Parrying Dirk | Dagger (parry tool) | 65 | 0.3/0.7/-/- | F 10 | 2 | none | Windward Parry | Verge, Briar Thicket chest |
| `wpn_whisperfang` | Whisperfang | Dagger | 72 | -/0.9/-/- | F 14 | 2 | Rupture 45 | Feint Step | Castle, Lower Ward |
| `wpn_wayfarer_blade` | Wayfarer's Blade | Straight sword | 105 | 0.6/0.6/-/- | M 10, F 10 | 4 | none | Rising Cleave | Starting gear |
| `wpn_chapel_longsword` | Chapel Longsword | Straight sword | 95 + 40 Radiance | 0.5/0.3/-/0.6 | M 12, D 12 | 5 | none | Radiant Blade | Ninefold Chapel |
| `wpn_lantern_sword` | Lantern Sword | Straight sword | 90 + 45 Fire | 0.5/0.5/-/0.3 | M 12, F 12 | 4 | none | Lantern Flare | Hollowdeep, Fungal Shore |
| `wpn_gray_greatsword` | Gray Knight Greatsword | Greatsword | 140 | 1.0/0.3/-/- | M 18, F 10 | 10 | none | Rising Cleave | Castle, ramparts captain |
| `wpn_rimeforged_claymore` | Rimeforged Claymore | Greatsword | 120 + 40 Aether | 0.8/-/0.5/- | M 18, I 14 | 11 | Rime 55 | Rime Edge | Rimeshelf, Frozen Foundry |
| `wpn_banana_cpp` | banana.cpp (R) | Curved sword | 98 | 0.3/1.0/-/- | F 13 | 3 | Rupture 50 | Rending Sweep | Ninefold Chapel, hollow wall |
| `wpn_reedcutter` | Reedcutter Scimitar | Curved sword | 95 | 0.4/0.8/-/- | F 11 | 3 | none | Rending Sweep | Mirrormere, Reed Marsh |
| `wpn_stillwater_rapier` | Stillwater Rapier | Thrusting sword | 100 | 0.2/0.9/-/- | F 13 | 3 | none | Lunging Thrust | Stillwater Village |
| `wpn_glassneedle_estoc` | Glassneedle Estoc | Thrusting sword | 85 + 40 Aether | -/0.7/0.6/- | F 14, I 14 | 4 | none | Glass Barrage | Glasswing Grotto |
| `wpn_woodcutter_axe` | Woodcutter's Axe | Axe | 115 | 0.8/0.3/-/- | M 12 | 5 | none | Rallying Roar | Verge, windmill |
| `wpn_briar_axe` | Briar Axe | Axe | 112 | 0.8/0.4/-/- | M 14, F 10 | 6 | Rupture 55 | Rending Sweep | B02 reward |
| `wpn_miners_pick` | Miner's Pick | Hammer | 118 (Blunt and Pierce) | 0.9/0.3/-/- | M 14 | 6 | none | Earthsplit | Cinderdeep Mine |
| `wpn_morning_star` | Morning Star | Hammer | 112 | 0.8/0.3/-/- | M 14 | 6 | Rupture 40 | Rallying Roar | Castle, kennel |
| `wpn_bellwright_mallet` | Bellwright's Mallet | Hammer | 110 + 50 Aether | 1.0/-/0.4/- | M 22, I 12 | 9 | Rime 50 | Bell Toll | B10 reward |
| `wpn_forgeheart_maul` | Forgeheart Maul | Colossal | 160 + 60 Fire | 1.2/-/-/0.3 | M 30 | 18 | none | Earthsplit | B08 reward |
| `wpn_greatanchor` | Greatanchor | Colossal | 175 | 1.3/-/-/- | M 28 | 20 | none | Rallying Roar | Drowned Belfry |
| `wpn_ferray` | ferray (R) | Spear | 95 + 40 Storm | 0.5/0.7/-/0.3 | M 12, F 14 | 5 | none | Lunging Thrust | Hollowdeep, Glowing Lake |
| `wpn_shardglass_spear` | Shardglass Spear | Spear | 90 + 40 Aether | 0.4/0.5/0.6/- | F 12, I 14 | 6 | none | Glass Barrage | D5 reward |
| `wpn_gray_halberd` | Gray Halberd | Halberd | 125 | 0.9/0.4/-/- | M 16, F 10 | 9 | none | Rising Cleave | Castle, Lower Ward |
| `wpn_choir_glaive` | Choir Glaive | Halberd | 110 + 50 Radiance | 0.6/0.3/-/0.7 | M 16, D 16 | 9 | none | Choir Toll | B06 reward |
| `wpn_bramble_claws` | Bramble Claws | Fist and claw | 80 | 0.3/0.9/-/- | F 12 | 2 | Rupture 35 | Feint Step | Windmill Rise elite |
| `wpn_iron_knuckles` | Ironbound Knuckles | Fist and claw | 85 (Blunt) | 0.7/0.6/-/- | M 12, F 12 | 3 | none | Rallying Roar | Rimeshelf Pass |
| `wpn_hunter_shortbow` | Hunter's Shortbow | Bow | 80 | 0.2/0.8/-/- | F 10 | 3 | none | Spark Volley | The Peddler |
| `wpn_spire_longbow` | Spire Longbow | Bow | 100 | 0.3/1.0/-/- | F 18 | 5 | none | Spark Volley | Lattice Spire |
| `wpn_plank_buckler` | Plank Buckler | Shield (buckler, parry) | 60 | 0.5/-/-/- | M 8 | 2 | none | Windward Parry | Starting gear |
| `wpn_nopokedb` | NoPokeDB (R) | Shield (medium) | 75 | 0.5/-/-/- | M 12 | 5 | none | Bastion | Mirrormere islet |
| `wpn_castle_kite` | Castle Kite Shield | Shield (medium) | 75 | 0.5/-/-/- | M 12 | 5 | none | Bastion | Castle Barbican |
| `wpn_graymantle_greatshield` | Graymantle Greatshield | Shield (greatshield) | 90 | 0.6/-/-/- | M 24 | 14 | none | Bastion | Castle armory (past the mimic) |
| `wpn_gradient_rod` | Gradient Rod | Catalyst (stylus) | Spell power 100 | -/-/1.0/- | I 14 | 2 | none | n/a | Castle library |
| `wpn_scholar_stylus` | Scholar's Stylus | Catalyst (stylus) | Spell power 90 | -/-/0.9/- | I 10 | 2 | none | n/a | Starting gear (Scholar) |
| `wpn_exile_stylus` | Exile's Stylus | Catalyst (stylus) | Spell power 110 | -/-/1.2/- | I 24 | 3 | none | n/a | B11 reward |
| `wpn_chapel_reliquary` | Chapel Reliquary | Catalyst (reliquary) | Spell power 95 | -/-/-/1.0 | D 10 | 2 | none | n/a | Starting gear (Acolyte); Ninefold Chapel |
| `wpn_whitewarden_seal` | Whitewarden Seal | Catalyst (reliquary) | Spell power 110 | -/-/-/1.2 | D 22 | 2 | none | n/a | D6 reward |

Shield guard values: bucklers absorb 70% physical with guard boost 30; medium shields
absorb 100% physical with guard boost 50; the greatshield absorbs 100% physical, 60%
elemental, with guard boost 80. NoPokeDB absorbs 100% physical and additionally takes
no chip damage from Pierce.

### Upgrades

- Common weapons upgrade from +0 to +10 with **Forgeglass**: +1 to +5 costs 1-3
  Forgeglass Shards; +6 to +10 costs 1-3 Forgeglass Chunks.
- Unique and relic weapons upgrade from +0 to +5 with **Starglass**. Each Starglass
  level counts as two common levels in the AR formula, and costs one Starglass.
- Levels +6 to +10, and Starglass levels +3 to +5, unlock after the player brings the
  Smith the **Forgeheart Ember**, B08's reward.
- Only the Smith in Hollowhearth upgrades. Upgrades cost Glimmer too: `200 * level^1.5`.

## Arts

An Art is a weapon skill that costs FP. Common weapons can swap their Art with an
**Art Scroll** if the class is compatible.

| ID | Art | Classes | FP | Effect | Scroll location |
|---|---|---|---|---|---|
| `art_feint_step` | Feint Step | Dagger, straight, curved, thrusting, fist | 6 | Short dash with i-frames 1-8; can follow with a light attack | Starting Art (Duelist) |
| `art_rending_sweep` | Rending Sweep | Curved, axe | 10 | Spinning slash, double status buildup | Mirrormere, Reed Marsh |
| `art_bastion` | Bastion | Shield | 0 (stamina 20) | Guard stance; next blocked hit triggers an automatic guard counter | Starting Art (Warden) |
| `art_windward_parry` | Windward Parry | Dagger, straight, shield | 0 | A parry with a 5-tick window for weapons that can't parry | Castle bell tower |
| `art_rising_cleave` | Rising Cleave | Straight, greatsword, halberd | 10 | Upward slash with hyperarmor and 2.0x stance damage | Starting Art (Wayfarer) |
| `art_rallying_roar` | Rallying Roar | Axe, hammer, colossal, fist | 12 | Body buff: +15% damage, +20% poise for 900 ticks | Verge, windmill |
| `art_lunging_thrust` | Lunging Thrust | Spear, thrusting | 9 | Charged forward thrust, 4 m, counter-hit 1.3x | Stillwater Village |
| `art_earthsplit` | Earthsplit | Hammer, colossal | 14 | Shockwave, 3 m radius, knocks down small enemies | Cinderdeep Mine |
| `art_rime_edge` | Rime Edge | Any melee except fist | 15 | Weapon buff: +40 Rime buildup and +10% Aether for 900 ticks | Rimeshelf |
| `art_radiant_blade` | Radiant Blade | Any melee except fist | 15 | Weapon buff: adds Radiance at 25% of AR for 900 ticks | Ninefold Chapel |
| `art_spark_volley` | Spark Volley | Bow | 10 | Three Storm-infused arrows | Starting Art (Hunter) |
| `art_lantern_flare` | Lantern Flare | Straight sword (Lantern Sword only) | 12 | Fire burst in a cone, lights darkness for 600 ticks | Unique |
| `art_glass_barrage` | Glass Barrage | Unique (Glassneedle, Shardglass) | 14 | Fires 5 Aether shards | Unique |
| `art_toll` | Toll (Bell Toll and Choir Toll variants) | Unique (B06, B10 weapons) | 18 | Stunning shockwave; Rime or Radiance | Unique |

## Spells

### Formulae (Insight, stylus)

| ID | Formula | Slots | FP | Requirement | Effect | Where |
|---|---|---|---|---|---|---|
| `spl_aether_dart` | Aether Dart | 1 | 5 | I 10 | Fast, weak projectile | Starting (Scholar) |
| `spl_aether_bolt` | Aether Bolt | 1 | 12 | I 14 | Strong, slower projectile | Starting (Scholar) |
| `spl_homing_motes` | Homing Motes | 1 | 16 | I 16 | 5 motes orbit, then home in on the target | Castle library |
| `spl_aether_wave` | Aether Wave | 1 | 20 | I 18 | Short-range cone of Aether | Castle library |
| `spl_mirror_ward` | Mirror Ward | 1 | 14 | I 16 | Body buff: absorbs 40% of the next 3 hits | Drowned Belfry |
| `spl_veil` | Veil | 1 | 10 | I 12 | Body buff: enemies notice you at half range for 900 ticks | Mossgut Cave |
| `spl_gradient_descent` | Gradient Descent | 2 | 24 | I 22 | A bolt that arcs up and accelerates down onto the target; high stance damage | Castle keep, Castellan's desk |
| `spl_glasswing_swarm` | Glasswing Swarm | 1 | 18 | I 18, F 12 | Shards in a spread; good against groups | Glasswing Grotto |
| `spl_unbinding_lance` | Unbinding Lance | 2 | 32 | I 28 | Charged beam that pierces through enemies | D7 reward |

### Litanies (Devotion, reliquary)

| ID | Litany | Slots | FP | Requirement | Effect | Where |
|---|---|---|---|---|---|---|
| `spl_mend` | Mend | 1 | 18 | D 10 | Heal 35% of max HP | Starting (Acolyte) |
| `spl_wide_mend` | Wide Mend | 2 | 32 | D 20 | Heal 30% in a 6 m radius, including Echoes | Whitewarden Catacombs |
| `spl_ember_toss` | Ember Toss | 1 | 8 | D 10 | Arcing Fire projectile | Starting (Acolyte) |
| `spl_pyre_breath` | Pyre Breath | 1 | 6 per second | D 16 | Channeled Fire cone | Cinderdeep Mine |
| `spl_storm_spear` | Storm Spear | 1 | 16 | D 16 | Storm projectile; 1.5x against targets in water | Mirrormere, Causeway |
| `spl_radiant_edge` | Radiant Edge | 1 | 14 | D 12 | Weapon buff: adds Radiance | Ninefold Chapel |
| `spl_choir_litany` | Choir Litany | 2 | 28 | D 20 | Radiance burst, 5 m radius | Drowned Belfry |
| `spl_purge` | Purge | 1 | 12 | D 14 | Cures all statuses and resets buildup meters | Stillwater Village |
| `spl_steadfast` | Steadfast | 1 | 14 | D 14 | Body buff: +50% poise for 600 ticks | Castle chapel |

## Charms

The player has 2 Charm slots at the start and gains one at each of the first two Great
Sigils (4 at most).

| ID | Charm | Effect | Where |
|---|---|---|---|
| `chm_vital` | Vital Charm | +8% max HP | Verge Crossroads |
| `chm_ox` | Ox Charm | +15% max equip load | Castle armory |
| `chm_quickening` | Quickening Charm | +12% stamina regeneration | Mossgut Cave |
| `chm_parriers_eye` | Parrier's Eye | Parry window +2 ticks | Castle ramparts |
| `chm_thornheart` | Thornheart | +12% damage for 600 ticks after a Rupture proc near you | Briar Thicket |
| `chm_ember_locket` | Ember Locket | +12% Fire damage | Cinderdeep Mine |
| `chm_scholars_seal` | Scholar's Seal (R, education) | +10% Formula damage, +1 spell slot | The Archivist, Hollowhearth |
| `chm_shrike_lite` | Shrike-Lite Circuit (R) | Spell startup -12% | Windmill rooftop |
| `chm_smol_llama` | smol-llama (R) | Equipped weapons weigh 25% less | Glowing Lake islet |
| `chm_bulwark` | Bulwark Charm | Guard stamina cost -15% | Castle Barbican |
| `chm_last_stand` | Last Stand Charm | +15% damage while under 20% HP | Bound Ring |
| `chm_lantern_moth` | Lantern Moth Charm | Restores 1 FP every 2 seconds | B09 area |
| `chm_rimeward` | Rimeward Charm | +60 Robustness | Rimeshelf Pass |
| `chm_clearblood` | Clearblood Charm | +60 Immunity | Reed Marsh |

## Armor

Armor uses two slots: **head** and **body**. A set is one of each. One relic head and
one relic body are unique pieces outside the sets.

| Set | Weight (head + body) | Physical defense | Absorption highlights | Poise | Where |
|---|---|---|---|---|---|
| Wayfarer's Coat | 2 + 5 | Low | Balanced | 8 | Starting gear |
| Chapel Vestments | 1 + 4 | Low | Radiance 20% | 4 | Ninefold Chapel |
| Reedwalker Garb | 2 + 5 | Low | Immunity +40 | 8 | Mirrormere |
| Miner's Harness | 3 + 9 | Medium | Fire 15% | 18 | Cinderdeep Mine |
| Exile's Robe | 1 + 4 | Low | Aether 25% | 4 | Bound Ring |
| Graymantle Plate | 7 + 18 | High | Physical 30% | 45 | Castle armory |
| Rimeforged Mail | 6 + 16 | High | Robustness +80 | 40 | Frozen Foundry |
| *Boo* (R, head) | 1 | n/a | Enemies notice you at 80% range | 0 | Castle library, hollow wall |
| *endark* (R, body) | 3 | Low | In darkness, enemies notice you at 50% range | 6 | Hollowdeep, Root Stair |

The *Opencode* relic is a **dye**, not armor. It recolors any armor set and the HUD
accent. It's in the Cartographer's quest reward.

## Consumables

| ID | Item | Effect | Stack | Source |
|---|---|---|---|---|
| `itm_vigil_flask` | Vigil Flask | Heal 300 HP at +0, +50 per upgrade (max +6) | Shared charges | Start |
| `itm_lucid_flask` | Lucid Flask | Restore 60 FP at +0, +10 per upgrade | Shared charges | Start |
| `itm_pitch_bomb` | Pitch Bomb | Throw: 120 Fire in a 2 m radius | 10 | Peddler, world |
| `itm_frostjar` | Frostjar | Throw: 60 Aether, Rime 80 | 10 | Rimeshelf |
| `itm_blight_pot` | Blight Pot | Throw: Blight 120 | 5 | Reed Marsh |
| `itm_throwing_knife` | Poison Knife | Throw: 40 Pierce, Poison 40 | 20 | Peddler |
| `itm_pitch_oil` | Pitch Oil | Weapon buff: Fire at 20% of AR, 900 ticks | 5 | Peddler |
| `itm_spark_oil` | Spark Oil | Weapon buff: Storm at 20% of AR, 900 ticks | 5 | Mirrormere |
| `itm_bloodbloom_oil` | Bloodbloom Oil | Weapon buff: Rupture 40, 900 ticks | 5 | Briar Thicket |
| `itm_clotweed` | Clotweed | Cure Rupture buildup | 10 | World |
| `itm_antidote_root` | Antidote Root | Cure Poison | 10 | Peddler |
| `itm_clearwater` | Clearwater Draught | Cure Blight | 5 | Mirrormere |
| `itm_hearth_ember` | Hearth Ember | Cure Rime | 5 | Rimeshelf |
| `itm_waking_salts` | Waking Salts | Cure Drowse and Delirium buildup | 5 | Peddler |
| `itm_glimmer_vial` | Glimmer Vial (small, large) | Grants 200 or 1,000 Glimmer | 99 | World |
| `itm_lantern_ash` | Lantern Ash | Return to the last Lantern rested at | 5 | Peddler |
| `itm_arrows` | Iron, Fire, and Storm Arrows | Ammunition | 99 | Peddler, world |

## Materials and key items

The following table lists every material and key item and where each one comes from.
Region WPs place them; see [M6](10-roadmap.md#m6-world-build-out).

| Item | Count | Sources |
|---|---|---|
| Wick Bead | 6 (start with 4 flask charges, max 10) | B01, B03, Mossgut Cave, the Stillwater chief's house, the Castle Barbican, Rimeshelf Pass |
| Flask Dew | 6 | The aqueduct top, the Reedwater islet, the Fungal Shore, the castle ramparts, the Frozen Foundry, Spire floor 2 |
| Forgeglass Shard | About 60 | Drops from Miners and Gray Knights; chests in every region; sold by the Peddler |
| Forgeglass Chunk | About 30 | Hollowdeep, the castle keep, Rimeshelf, the Spire; sold by the Smith after the Forgeheart Ember |
| Starglass | 14 | One from each of B02-B12, plus Glasswing Grotto, the Bound Ring, and Spire floor 3 |
| Temper Whetstone | 5 | See [Temper](#temper) |
| Rewrite Quill | 3 | The Ninefold Chapel crypt, the castle library's upper shelf, the Whitewarden Catacombs' ossuary |
| Lost Folio | 3 | Ninefold Chapel, the castle library, Whitewarden Catacombs (the Archivist's quest) |
| Mail satchel | 3 | The Verge windmill, Stillwater Village, Rimeshelf Pass (the Courier's quest) |
| Forgeheart Ember | 1 | B08 |
| Echo Bell | 1 | The Archivist, on the first visit to Hollowhearth |
| *smoltorch* | 1 | Cinderdeep Mine |
| Wayfarer's Ledger | 1 | Starting inventory |
| Great Sigil | 4 | B06, B07, B09, B10 |

## Starting classes

| Class | Level | Vit | Foc | End | Mig | Fin | Ins | Dev | Gear |
|---|---|---|---|---|---|---|---|---|---|
| Wayfarer | 10 | 12 | 10 | 12 | 12 | 12 | 6 | 6 | Wayfarer's Blade, Plank Buckler, Wayfarer's Coat |
| Warden | 10 | 14 | 6 | 14 | 14 | 8 | 7 | 7 | Wayfarer's Blade, Castle Kite Shield, Miner's Harness |
| Duelist | 9 | 10 | 8 | 12 | 8 | 16 | 9 | 6 | Fungal Knife (unupgraded copy), Parrying Dirk, Reedwalker Garb |
| Scholar | 9 | 9 | 14 | 9 | 7 | 10 | 16 | 4 | Scholar's Stylus, short sword (Wayfarer's Blade), Exile's Robe, Aether Dart, Aether Bolt |
| Acolyte | 9 | 11 | 12 | 9 | 10 | 8 | 4 | 16 | Chapel Reliquary, Wayfarer's Blade, Chapel Vestments, Mend, Ember Toss |
| Hunter | 9 | 10 | 8 | 13 | 9 | 15 | 8 | 6 | Hunter's Shortbow, Fungal Knife, Reedwalker Garb, 40 Iron Arrows |

Where a starting item also appears in the world, the world copy still drops. Starting
copies are separate instances.

## Builds

The following nine builds are supported end to end. The game's balance pass
([WP-089](10-roadmap.md#wp-089)) runs each one through every required boss.

| Build | Main attributes | Weapons | Art | Spells | Charms | Armor |
|---|---|---|---|---|---|---|
| Colossus | Might, Vitality | Forgeheart Maul, Greatanchor, Gray Knight Greatsword | Rallying Roar, Earthsplit | Steadfast (optional) | Ox, Last Stand | Graymantle Plate |
| Bloodletter | Finesse, Endurance | banana.cpp, Whisperfang, Bramble Claws | Rending Sweep, Feint Step | none | Thornheart, Quickening | Reedwalker Garb |
| Scholar | Insight, Focus | Gradient Rod, Exile's Stylus, Glassneedle Estoc | Glass Barrage | All Formulae | Scholar's Seal, Shrike-Lite Circuit | Exile's Robe |
| Paladin | Devotion, Might | Chapel Longsword, Choir Glaive, Castle Kite Shield | Radiant Blade, Bastion | Mend, Radiant Edge, Steadfast, Purge | Vital, Bulwark | Graymantle Plate or Chapel Vestments |
| Frostbinder | Might, Insight | Rimeforged Claymore, Gray Halberd (Rimed temper), Bellwright's Mallet (late) | Rime Edge | Mirror Ward | Rimeward, Ox | Rimeforged Mail |
| Spellblade | Finesse, Insight | Stillwater Rapier (Lucid temper), Glassneedle Estoc, Gradient Rod | Lunging Thrust | Aether Bolt, Homing Motes, Mirror Ward, Veil | Shrike-Lite Circuit, Quickening | Reedwalker Garb, *Boo* |
| Bulwark | Endurance, Might | Graymantle Greatshield, Gray Halberd, NoPokeDB | Bastion | Steadfast | Bulwark, Ox | Graymantle Plate |
| Ranger | Finesse, Endurance | Hunter's Shortbow, Fungal Knife, Spire Longbow (late) | Spark Volley | none | Quickening, Vital | Reedwalker Garb, *endark* |
| Tempest Lancer | Devotion, Finesse | ferray, Chapel Reliquary, Whitewarden Seal | Lunging Thrust | Storm Spear, Ember Toss, Mend, Purge, Wide Mend (late) | Ember Locket, Lantern Moth | Chapel Vestments |

### Respec

The Archivist offers one free respec after the first Great Sigil, then one respec per **Rewrite Quill** (3 in the world; see
[materials and key items](#materials-and-key-items)). Respec redistributes all
attributes above the starting class's values.
