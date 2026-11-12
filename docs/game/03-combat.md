# Combat

This document defines every combat rule as data and formulas: verbs and their frame
data, attributes, damage, poise and stance, guarding and parrying, status effects,
spells, and lock-on. Read it before any WP in the `combat` lane.

All timings are in **ticks at 30 ticks per second** (one tick is 33.3 ms). All numbers
are starting values to tune in play; tuning lives in `game/data/`, never in code.

## Tuning targets

These targets drive every number in this document. When a number changes, check it
against them.

| Target | Value |
|---|---|
| Light attacks to kill fodder, on-level | 2-3 |
| Light attacks to kill an elite, on-level | 10-14 |
| Hits to kill a boss, on-level, quality build | 45-70 (a 2-4 minute fight) |
| Hits from fodder to kill the player, on-level | 6-8 |
| Hits from a boss to kill the player, on-level | 3-4 |
| Heavy boss attack wind-up (readability) | 18-36 ticks (0.6-1.2 s) |
| Player light-attack startup | 9-14 ticks |
| Roll invincibility, light and medium load | 13 ticks |

## Combat verbs

Every action is a **move**: a data record with phases, hitboxes, i-frames, costs, and
cancel windows. The core advances every actor's move one frame per tick.

The following table lists the player verbs and their default frame data for a
straight sword. Other weapon classes override these in their moveset files.

| Verb | Input | Startup | Active | Recovery | Stamina | Notes |
|---|---|---|---|---|---|---|
| Light attack 1-3 | `LIGHT` | 10 | 3 | 14 | 12 | Chains to the next light if pressed from frame 8 of recovery |
| Heavy attack | `HEAVY` tap | 18 | 4 | 20 | 22 | Hyperarmor from startup frame 6 |
| Charged heavy | `HEAVY` hold | 18 + up to 24 charge | 4 | 22 | 28 | Charge scales motion value from 1.3 to 1.8 |
| Running attack | `LIGHT` while sprinting | 8 | 4 | 18 | 14 | Lunges 2 m |
| Rolling attack | `LIGHT` during roll recovery | 7 | 3 | 15 | 12 | |
| Backstep attack | `LIGHT` during backstep recovery | 6 | 3 | 15 | 12 | |
| Jump attack | `LIGHT` or `HEAVY` in the air | 10 | 4 | 16 landing | 18 | Heavy version has hyperarmor; strong poise damage |
| Plunge attack | Any attack while falling more than 3 m onto a target | 4 | 4 | 20 landing | 0 | Motion value 2.0; negates fall damage up to 12 m |
| Guard | `GUARD` hold | 3 to raise | held | 4 to lower | per hit | See [Guard](#guard-and-guard-counter) |
| Guard counter | `HEAVY` within 12 ticks after blocking a hit | 8 | 4 | 18 | 18 | Motion value 1.4, stance damage 2.0x |
| Parry | `GUARD` with a parry tool | 2 | 6 (window) | 20 | 10 | See [Parry](#parry-and-critical-hits) |
| Roll | `DODGE` tap with a direction | 0 | i-frames 2-14 | to 24 | 14 | Travels 3.2 m |
| Backstep | `DODGE` tap, no direction | 0 | i-frames 2-7 | to 15 | 10 | Travels 1.8 m |
| Sprint | `DODGE` hold | n/a | n/a | n/a | 0.2 per tick | Starts after 8 ticks held |
| Jump | `JUMP` | 4 | n/a | 6 landing | 10 | |
| Art | `ART` | per Art | per Art | per Art | per Art | Costs FP |
| Use item | `USE_ITEM` | per item | n/a | per item | 0 | Movement slowed to a walk |
| Drink a flask | `USE_ITEM` with a flask selected | 16 (heal lands) | n/a | 14 | 0 | Walk speed 40% |

### Equip load and roll tiers

Equip load is the total weight of equipped weapons and armor divided by the maximum
equip load. The tier changes the roll:

| Load | Tier | Roll | I-frames | Total frames | Distance |
|---|---|---|---|---|---|
| Under 30% | Light | Quick roll | 2-14 | 20 | 3.4 m |
| 30% to under 70% | Medium | Roll | 2-14 | 24 | 3.2 m |
| 70% to 100% | Heavy | Heavy roll | 2-13 | 30 | 2.4 m |
| Over 100% | Overloaded | None; walk only | n/a | n/a | n/a |

At heavy load, stamina regeneration is reduced by 20 percent.

### Input buffer

An action pressed while the current move can't be canceled is **buffered** for up to
8 ticks. It fires on the first frame the move allows that kind of cancel. Only the most
recent buffered action is kept. Moves declare these cancel windows:

- `cancel_attack_from`: the frame from which a chained attack can start.
- `cancel_dodge_from`: the frame from which a roll or backstep can start. For most
  attacks this is the first recovery frame plus 4.
- `cancel_any_from`: the frame from which any action, including walking, interrupts.

Generous buffering is what makes combat feel responsive. The RL environment sees the
same buffer, so an agent that presses early isn't punished differently from a human.

## Attributes

The Wayfarer has seven attributes. Each starts between 4 and 16 depending on the
starting class, and caps at 60. The character level is the sum of attributes minus 60,
so a fresh character is level 9 or 10.

| Attribute | Drives | Resistance it raises |
|---|---|---|
| Vitality | Max HP | Immunity (Poison, Blight) |
| Focus | Max FP, spell slots | Composure (Drowse, Delirium) |
| Endurance | Max stamina, max equip load | Robustness (Rupture, Rime) |
| Might | Might-scaling damage, Might requirements | none |
| Finesse | Finesse-scaling damage, Formula and Litany cast speed | none |
| Insight | Insight-scaling damage, Formula damage, Aether defense | none |
| Devotion | Devotion-scaling damage, Litany damage, Radiance defense | none |

### Derived stats

The following curves are piecewise linear with breakpoints at 20 and 40. Write them as
data in `game/data/attributes.ron`, not as code. The first value in each row is the stat
at attribute 1; each point above 1 adds the listed increment.

| Stat | 1 to 20 | 20 to 40 | 40 to 60 | Value at 10 | Value at 40 |
|---|---|---|---|---|---|
| Max HP (Vitality) | 300 + 30 per point | +25 per point | +8 per point | 570 | 1,370 |
| Max FP (Focus) | 40 + 5 per point | +3 per point | +1 per point | 85 | 195 |
| Max stamina (Endurance) | 80 + 2 per point | +1 per point | +0.5 per point | 98 | 138 |
| Max equip load (Endurance) | 40 + 1.2 per point | +1.2 per point | +0.6 per point | 50.8 | 86.8 |
| Spell slots (Focus) | 2 at 10, +1 every 8 points | | | 2 | 5 |
| Resistance (any) | 83 + 3 per point of its attribute | | | 110 | 200 |

### Regeneration

- Stamina regenerates at 1.5 per tick after 15 ticks without spending stamina.
  While guarding, it regenerates at 40 percent of that rate.
- FP doesn't regenerate on its own. Lucid Flasks, some Charms, and resting restore it.
- HP doesn't regenerate on its own. Vigil Flasks, Litanies, and some Charms restore it.

## Damage

### Damage types

There are four physical subtypes and four elements. Every hitbox deals one physical
subtype and up to one element.

| Type | Kind | Typical source | Typical weakness |
|---|---|---|---|
| Neutral | Physical | Straight swords, most weapons | n/a |
| Slash | Physical | Curved swords, axes | Unarmored beasts |
| Blunt | Physical | Hammers, fists | Skeletons, golems, armored knights |
| Pierce | Physical | Spears, daggers, thrusting swords | Large beasts, counter hits |
| Aether | Element | Formulae, Insight weapons | Knights, constructs |
| Fire | Element | Fire weapons, Litanies, pitch bombs | Beasts, husks, fungus |
| Storm | Element | Storm Litanies, storm oils | Anything standing in water |
| Radiance | Element | Litanies, Devotion weapons | Undead, Whitewarden Skeletons |

### Attack rating

A weapon's attack rating (AR) for each damage type it deals is computed as follows:

```text
base(U)      = base_damage * (1 + 0.06 * U)                    # U = upgrade level, 0-10
k_a(U)       = scaling_a * (1 + 0.03 * U)                      # per attribute a
curve(x)     = 0.40 * x / 20                     if x <= 20
             = 0.40 + 0.35 * (x - 20) / 20       if x <= 40
             = 0.75 + 0.25 * (x - 40) / 20       otherwise     # 1.0 at 60
AR           = base(U) * (1 + sum over a of k_a(U) * curve(attr_a))
```

The scaling letter shown in menus is display-only and derived from `scaling_a`:
S at 1.4 or more, A at 1.0, B at 0.7, C at 0.45, D at 0.25, and E above 0.

**Requirements.** If any attribute requirement isn't met, all scaling is zero, base
damage is multiplied by 0.6, and the weapon's recovery frames increase by 20 percent.
The menu shows the weapon as *unwieldy*.

**Two-handing.** While two-handing, Might counts as 1.5 times its value (capped at 60)
for both requirements and scaling.

### Damage dealt

When a hitbox connects, the core computes damage per type and sums the results:

```text
raw        = AR_type * motion_value * counter_mult * buff_mult
after_def  = raw * raw / (raw + defense_type)        # never zero, favors big hits
final_type = after_def * (1 - absorption_type) * weakness_type
damage     = max(1, round(sum of final_type))
```

Replace the following:

- `motion_value`: Per hitbox, from the move. Light attacks are about 1.0; heavy
  attacks 1.3; charged attacks 1.8; criticals 3.0 or more.
- `counter_mult`: 1.15 when the target is in the startup or active frames of its own
  attack, otherwise 1.0.
- `buff_mult`: The product of active damage buffs, capped at 1.5.
- `defense_type`: Flat defense from level and armor.
- `absorption_type`: Percentage absorption from armor, capped at 0.7.
- `weakness_type`: From the target's content data, for example 1.25 for Blunt against
  skeletons.

## Poise and stance

**Poise** decides whether a hit interrupts an actor.

- Every hitbox carries `poise_damage`. Every actor has `poise`, from armor (player) or
  content data (enemy).
- Poise damage accumulates. When it exceeds the actor's poise, the actor staggers and
  the accumulator resets. A margin under 50 percent of poise is a light stagger
  (12 ticks); over 50 percent is a heavy stagger (24 ticks).
- The accumulator resets after 90 ticks without taking poise damage.
- **Hyperarmor**: during frames the move marks as hyperarmor, the actor's poise counts
  as doubled. Heavy attacks with heavy weapons have hyperarmor from mid-startup.

**Stance** is a second meter that only elites and bosses have.

- Stance damage is the hitbox's `poise_damage` times the move's `stance_mult` (1.0 by
  default; 2.0 for guard counters, charged heavies, and jump heavies).
- When stance damage reaches the actor's `stance` value, the actor is **stance-broken**:
  it kneels for 60 ticks, and the attacker can perform a frontal critical.
- Stance damage decays by 2 percent of the meter per tick after 120 ticks without a hit.

## Guard and guard counter

- While guarding, a blocked hit deals `damage * (1 - guard_absorption_type)` and costs
  stamina equal to `poise_damage * 100 / guard_boost`.
- If stamina would drop below zero, the guard breaks: the actor staggers for 30 ticks
  and takes the remaining damage unblocked.
- Guard absorption comes from the shield's data: bucklers absorb 70 percent physical,
  medium shields and greatshields 100 percent. Weapons guard at about 50 percent physical.
- After a block, `HEAVY` within 12 ticks performs a guard counter.

## Parry and critical hits

- A **parry** has a 6-tick window for bucklers and parry daggers, 4 ticks for medium
  shields. Greatshields and most weapons can't parry.
- A successful parry puts the attacker in the *parried* state for 60 ticks. Some boss
  attacks are marked unparryable in data and show a red flash at startup.
- **Critical hits** are paired moves: the attacker plays a `crit` move and the victim
  plays a matching `crit_receive` move, both locked until the crit ends. Damage is
  `AR * 3.0 * crit_mult`, where `crit_mult` comes from the weapon (daggers 1.4,
  most weapons 1.0).
- Triggers: `LIGHT` within 1.5 m of a parried or stance-broken actor's front
  (riposte), or within 1.2 m of an unaware actor's back inside a 90-degree rear arc
  (backstab). Backstabs on unaware enemies also add 20 percent status buildup.

## Status effects

Every actor has a buildup meter for each status. Hits add buildup; when a meter reaches
the actor's resistance, the effect **procs**, the meter resets to zero, and that status
can't build up again for 60 ticks.

```text
buildup_per_hit = status_value_of_hit * (1 + 0.3 * crit_or_stealth)
decay           = 1.5 per tick, starting 60 ticks after the last buildup
boss_tolerance  = resistance * (1 + 0.25 * times_proc'd_this_fight)   # bosses only
```

| Status | Resistance | On proc | Duration | Counter |
|---|---|---|---|---|
| Rupture | Robustness | Burst: 12% of max HP + 80 | Instant | Clotweed cures Rupture buildup |
| Rime | Robustness | Burst: 8% of max HP + 50, then +15% damage taken and -50% stamina regeneration | 600 ticks | Hearth Ember consumable cures |
| Poison | Immunity | 0.2% of max HP + 7 per second | 1,800 ticks | Antidote Root cures |
| Blight | Immunity | 0.6% of max HP + 14 per second | 900 ticks | Clearwater Draught cures |
| Drowse | Composure | Enemies: sleep for 300 ticks or until hit (hit while asleep deals 1.2x). Player: knocked down for 60 ticks and loses 10% of FP | as listed | Waking Salts |
| Delirium | Composure | Player: loses 30% of max FP and 10% of max HP, staggers for 40 ticks. Enemies that can go mad: stagger for 60 ticks | Instant | Waking Salts clear buildup |

Bosses list immunities in data. Every boss is immune to at least one status, and no
boss is immune to more than three.

### Buffs

- An actor holds at most one **weapon buff** (an oil or a buff spell on the weapon)
  and one **body buff** (a buff spell or consumable on the body) at a time. Applying a
  second of the same kind replaces the first.
- Charms are passive and always stack.
- A buff is data: a stat delta or multiplier, a duration in ticks, and a VFX tag.

## Spells

There are two schools:

- **Formulae** scale with Insight and are cast with a **stylus**. They lean toward
  projectiles and Aether damage.
- **Litanies** scale with Devotion and are cast with a **reliquary**. They lean toward
  healing, buffs, Fire, Storm, and Radiance.

Spells are moves too. Each spell has an FP cost, a slot cost (1 or 2), attribute
requirements, frame data, and either a projectile, an area, a buff, or a heal. The
catalyst's *spell power* replaces AR in the damage formula. Finesse shortens cast
startup by up to 25 percent at 60 Finesse.

Spells are attuned at Lanterns. The number of slots comes from Focus.

**Casting input.** With a catalyst as the active right-hand item, `LIGHT` casts the
selected spell and `HEAVY` casts its charged form, if it has one. With a catalyst in the
left hand, `GUARD` casts instead, because catalysts can't guard. `CYCLE_SPELL` selects
the spell.

## Lock-on

- `LOCK` picks the target with the smallest angle to the camera's forward direction
  (the player's facing, for RL hosts) within 20 m and a 60-degree cone, breaking ties by
  distance. Line of sight is required to acquire, not to keep.
- The lock breaks when the target dies (the core then auto-picks the next target within
  8 m), goes beyond 30 m, or the player presses `LOCK` again.
- `SWITCH_L` and `SWITCH_R` move to the nearest target to the left or right in screen
  space (or facing space, for RL).
- While locked on, movement strafes around the target, and attacks turn to face it with
  a turn-rate limit set per move (`track_deg_per_tick`).

## Difficulty settings

Difficulty is part of the world configuration, so the RL environment can use it too.
None of these settings changes game rules beyond the listed scale.

| Setting | Range | Wayfarer mode preset |
|---|---|---|
| Damage taken | 25% to 200% | 25% |
| Enemy HP | 50% to 200% | 60% |
| Infinite stamina | On or off | On |
| Game speed | 50% to 100% | 100% |
| Keep Glimmer on death | On or off | On |
| Boss skip after 3 deaths | On or off | On |

Game speed is the one exception to world configuration: it scales how fast the browser's
loop feeds ticks to the core, so it doesn't change the simulation, isn't recorded in
replays, and isn't an RL option.

Saves record the lowest damage-taken setting ever used, and the codex shows a small
lantern icon on runs that used Wayfarer mode. Nothing is locked behind difficulty.

## Move data schema

The following code sample shows a complete move in RON. The schema is defined in
`game/crates/content/src/moves.rs`.

```ron
// game/data/moves/straight_sword.ron (excerpt)
(
  id: "ss_light_1",
  anim: "sword_slash_r",              // renderer clip; visual only
  total_frames: 27,
  stamina: 12,
  fp: 0,
  root_motion: [(0, 0.0), (8, 0.6), (13, 0.9), (27, 0.9)],  // (frame, meters forward)
  track_deg_per_tick: 12.0,           // turn toward lock target, startup only
  hyperarmor: None,                   // or Some((from: 6, to: 14))
  iframes: None,
  cancel_attack_from: 21,
  cancel_dodge_from: 17,
  cancel_any_from: 24,
  next_chain: Some("ss_light_2"),
  hitboxes: [
    (
      frames: (10, 12),
      shape: Capsule(a: (0.2, 1.2, 0.3), b: (0.2, 1.0, 1.6), radius: 0.25),
      motion_value: 1.0,
      phys: Neutral,
      element: None,
      poise_damage: 12,
      status: [],
      knockback: 0.3,
      hitstop: 3,                      // render-only freeze frames
      tags: [],                        // for example [Unparryable, Unblockable, Grab]
    ),
  ],
  sfx: [(9, "swing_light")],
  vfx: [(10, "trail_steel")],
)
```

The shape coordinates are in the actor's local space: x right, y up, z forward, with the
origin at the actor's feet.
