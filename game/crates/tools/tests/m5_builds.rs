//! M5 exit gate: "All nine builds are equippable" (docs/game/10-roadmap.md, Milestones).
//!
//! The nine builds are the table in docs/game/04-arsenal-and-builds.md, "Builds". Each test
//! here takes one row and drives a real `World` the way the game does:
//!
//! 1. resolves every display name in the row to its content entry (weapon, Art, spell, Charm,
//!    armor piece, temper), so a renamed or missing item fails with its name;
//! 2. rests at a Lantern and raises attributes with `Command::LevelUp` until the build's
//!    weapon and spell requirements and its spell slots are met;
//! 3. gives each item and equips it with `Command::Equip` (hand, armor, and Charm slots, and
//!    spell positions 32 and up), tempers a weapon with `Command::Temper` where the row says
//!    so, and asserts each item sits in its slot;
//! 4. checks that each Art in the row is the active Art of one of the build's weapons.
//!
//! Item gives use the inventory API (`give_gear`, `pick_up`), as the existing gear and spell
//! tests do. No game data or simulation code is changed here.

use lf_content::Content;
use lf_core::combat::arts::weapon_art;
use lf_core::combat::spells::MAX_ATTUNED;
use lf_core::input::{Command, TickInput};
use lf_core::items::{slot, upgrade};
use lf_core::{World, tick};

/// Attribute indexes for `Command::LevelUp`, in the order of `Attrs`.
const FOCUS: usize = 1;
const MIGHT: usize = 3;
const FINESSE: usize = 4;
const INSIGHT: usize = 5;
const DEVOTION: usize = 6;

/// One weapon in a build: its display name in the table, the hand slot it goes in, and its
/// temper (a display name from `upgrades.ron`) where the table names one.
struct Wpn {
    name: &'static str,
    slot: u8,
    temper: Option<&'static str>,
}

/// One row of the "Builds" table. Display names are exactly as the table spells them,
/// except that a Charm may drop its " Charm" suffix, as the table does.
struct Build<'a> {
    name: &'static str,
    weapons: &'a [Wpn],
    arts: &'a [&'a str],
    spells: &'a [&'a str],
    charms: &'a [&'a str],
    /// Armor pieces, equipped one after another into their own slots. Two pieces for the
    /// same slot are alternatives (the table's "or", or a relic that replaces the set's
    /// body), so each is checked as it goes on.
    armor: &'a [&'a str],
}

const fn w(name: &'static str, slot: u8) -> Wpn {
    Wpn {
        name,
        slot,
        temper: None,
    }
}

const fn wt(name: &'static str, slot: u8, temper: &'static str) -> Wpn {
    Wpn {
        name,
        slot,
        temper: Some(temper),
    }
}

use slot::{LEFT_1, LEFT_2, RIGHT_1, RIGHT_2};

fn content() -> Content {
    Content::load_embedded().expect("embedded content")
}

fn step(w: &mut World, commands: Vec<Command>) {
    tick(
        w,
        &TickInput {
            commands,
            ..TickInput::default()
        },
    );
}

fn attrs(w: &World) -> [u16; 7] {
    let a = &w.actors.get(w.player).unwrap().stats.attrs;
    [
        a.vitality,
        a.focus,
        a.endurance,
        a.might,
        a.finesse,
        a.insight,
        a.devotion,
    ]
}

/// A world on `test_field`, rested at its Lantern (3 m behind the spawn) with plenty of
/// Glimmer, so Lantern commands apply.
fn resting_world(c: &Content) -> World {
    let mut w = World::new(5, c, "test_field").expect("test_field exists");
    let id = w.player;
    let p = w.actors.get_mut(id).unwrap();
    p.pos = [0.0, 0.0, -2.0];
    p.inventory.glimmer = 3_000_000_000;
    step(&mut w, vec![Command::Rest]);
    assert!(w.state.resting.is_some(), "the player rests at the Lantern");
    w
}

/// Raises each attribute to at least `want` with `Command::LevelUp`, and checks it got there.
fn level_to(w: &mut World, want: [u16; 7]) {
    let now = attrs(w);
    let mut commands = Vec::new();
    for (i, (&have, &need)) in now.iter().zip(&want).enumerate() {
        for _ in have..need {
            commands.push(Command::LevelUp { attribute: i as u8 });
        }
    }
    step(w, commands);
    let got = attrs(w);
    for i in 0..7 {
        assert!(
            got[i] >= want[i],
            "attribute {i} is {} after leveling, wanted {}",
            got[i],
            want[i]
        );
    }
}

/// Resolves display names to content entries, or panics naming the missing one.
struct Names<'a>(&'a Content);

impl Names<'_> {
    fn weapon(&self, name: &str) -> &lf_content::Weapon {
        self.0
            .all_weapons()
            .map(|(_, w)| w)
            .find(|w| w.name == name)
            .unwrap_or_else(|| panic!("no weapon named {name:?}"))
    }

    fn art(&self, name: &str) -> &lf_content::Art {
        self.0
            .arts
            .iter()
            .find(|a| a.name == name)
            .unwrap_or_else(|| panic!("no Art named {name:?}"))
    }

    fn spell(&self, name: &str) -> &lf_content::SpellDef {
        self.0
            .spells
            .iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("no spell named {name:?}"))
    }

    fn charm(&self, name: &str) -> &lf_content::Charm {
        let suffixed = format!("{name} Charm");
        self.0
            .charms
            .iter()
            .find(|c| c.name == name || c.name == suffixed)
            .unwrap_or_else(|| panic!("no Charm named {name:?}"))
    }

    fn armor(&self, name: &str) -> &lf_content::ArmorPiece {
        self.0
            .armor
            .iter()
            .find(|a| a.name == name)
            .unwrap_or_else(|| panic!("no armor piece named {name:?}"))
    }

    /// The index of a temper in `upgrades.ron`, which is what `Command::Temper` takes.
    fn temper(&self, name: &str) -> u8 {
        let rules = self.0.upgrades.as_ref().expect("upgrades.ron");
        rules
            .tempers
            .iter()
            .position(|t| t.name == name)
            .unwrap_or_else(|| panic!("no temper named {name:?}")) as u8
    }
}

fn num(c: &Content, id: &str) -> u32 {
    c.registry
        .num(id)
        .unwrap_or_else(|| panic!("{id} is registered"))
}

fn give_gear(w: &mut World, item: u32) {
    let id = w.player;
    assert!(
        w.actors.get_mut(id).unwrap().inventory.give_gear(item),
        "gear {item} goes in the pouch"
    );
}

fn equipped(w: &World, slot: u8) -> Option<u32> {
    w.actors.get(w.player).unwrap().inventory.equipment.slots[usize::from(slot)]
}

/// Gives `item` and equips it in `slot` through `Command::Equip`, then checks it is there.
fn equip_gear(w: &mut World, slot: u8, item: u32, what: &str) {
    give_gear(w, item);
    step(w, vec![Command::Equip { slot, item }]);
    assert_eq!(
        equipped(w, slot),
        Some(item),
        "{what} is equipped in slot {slot}"
    );
}

/// Gives and attunes `spell` at position `index`, and checks it is attuned there.
fn attune_spell(w: &mut World, index: u8, spell: u32, what: &str) {
    give_gear(w, spell);
    step(
        w,
        vec![Command::Equip {
            slot: slot::SPELL_1 + index,
            item: spell,
        }],
    );
    assert_eq!(
        w.spells.attuned[usize::from(index)],
        Some(spell),
        "{what} is attuned at position {index}"
    );
}

/// The smallest Focus whose slots (plus the Charms' bonus) hold `slots` spell slots.
fn focus_for(w: &World, slots: u32, bonus: u32) -> u16 {
    (w.attributes.min..=w.attributes.max)
        .find(|&f| u32::from(w.attributes.spell_slots(f)) + bonus >= slots)
        .unwrap_or_else(|| panic!("no Focus holds {slots} spell slots (with {bonus} bonus)"))
}

fn check_build(b: &Build<'_>) {
    let c = content();
    let names = Names(&c);

    // 1. Every name in the row resolves.
    let weapons: Vec<_> = b.weapons.iter().map(|x| names.weapon(x.name)).collect();
    let arts: Vec<_> = b.arts.iter().map(|a| names.art(a)).collect();
    let spells: Vec<_> = b.spells.iter().map(|s| names.spell(s)).collect();
    let charms: Vec<_> = b.charms.iter().map(|x| names.charm(x)).collect();
    let armor: Vec<_> = b.armor.iter().map(|a| names.armor(a)).collect();
    let tempers: Vec<_> = b
        .weapons
        .iter()
        .map(|x| x.temper.map(|t| names.temper(t)))
        .collect();

    // 2. Level to what the build needs: weapon requirements, spell requirements, and the
    // Focus that opens enough spell slots (Charms like the Scholar's Seal add one).
    let mut w = resting_world(&c);
    let mut want = attrs(&w);
    let raise = |want: &mut [u16; 7], i: usize, v: u16| want[i] = want[i].max(v);
    for x in &weapons {
        raise(&mut want, MIGHT, x.requirements.might);
        raise(&mut want, FINESSE, x.requirements.finesse);
        raise(&mut want, INSIGHT, x.requirements.insight);
        raise(&mut want, DEVOTION, x.requirements.devotion);
    }
    for s in &spells {
        raise(&mut want, MIGHT, s.requirements.might);
        raise(&mut want, FINESSE, s.requirements.finesse);
        raise(&mut want, INSIGHT, s.requirements.insight);
        raise(&mut want, DEVOTION, s.requirements.devotion);
    }
    let bonus: u32 = charms
        .iter()
        .map(|ch| {
            ch.effects
                .iter()
                .map(|e| match e {
                    lf_content::GearEffect::SpellSlots(n) => u32::from(*n),
                    _ => 0,
                })
                .sum::<u32>()
        })
        .sum();
    let slots: u32 = spells.iter().map(|s| u32::from(s.slots)).sum();
    if slots > 0 {
        raise(&mut want, FOCUS, focus_for(&w, slots, bonus));
    }
    level_to(&mut w, want);

    // 3. Equip. Weapons first, tempering where the row says so (Temper needs a rest, which
    // the world is still in, and a Temper Whetstone).
    for (x, (def, temper)) in b.weapons.iter().zip(weapons.iter().zip(&tempers)) {
        equip_gear(&mut w, x.slot, num(&c, &def.id), x.name);
        if let Some(t) = *temper {
            let whetstone = num(&c, &c.upgrades.as_ref().unwrap().whetstone_item);
            let items = w.items.clone();
            let id = w.player;
            let got = w
                .actors
                .get_mut(id)
                .unwrap()
                .inventory
                .pick_up(whetstone, 1, &items);
            assert_eq!(got, 1, "a Temper Whetstone goes in the pouch");
            step(
                &mut w,
                vec![Command::Temper {
                    slot: x.slot,
                    temper: t,
                }],
            );
            let applied = upgrade::temper_of(&w, num(&c, &def.id))
                .unwrap_or_else(|| panic!("{} takes the {} temper", x.name, x.temper.unwrap()));
            assert_eq!(applied.name, x.temper.unwrap());
        }
    }
    for (i, ch) in charms.iter().enumerate() {
        equip_gear(&mut w, slot::CHARM_1 + i as u8, num(&c, &ch.id), &ch.name);
    }
    for a in &armor {
        let at = match a.slot {
            lf_content::gear::ArmorSlot::Head => slot::HEAD,
            lf_content::gear::ArmorSlot::Body => slot::BODY,
        };
        equip_gear(&mut w, at, num(&c, &a.id), &a.name);
    }
    for (i, s) in spells.iter().enumerate() {
        attune_spell(&mut w, i as u8, num(&c, &s.id), &s.name);
    }

    // Every weapon is still in its slot, and its requirements are met.
    let have = attrs(&w);
    for (x, def) in b.weapons.iter().zip(&weapons) {
        assert_eq!(equipped(&w, x.slot), Some(num(&c, &def.id)), "{}", x.name);
        let r = &def.requirements;
        assert!(
            have[MIGHT] >= r.might
                && have[FINESSE] >= r.finesse
                && have[INSIGHT] >= r.insight
                && have[DEVOTION] >= r.devotion,
            "{} requirements are met",
            x.name
        );
    }

    // 4. Each Art in the row is the active Art of one of the build's weapons.
    let active: Vec<(&str, String)> = b
        .weapons
        .iter()
        .zip(&weapons)
        .map(|(x, def)| {
            let art = weapon_art(&w, num(&c, &def.id))
                .map(|r| r.id.clone())
                .unwrap_or_default();
            (x.name, art)
        })
        .collect();
    for a in &arts {
        assert!(
            active.iter().any(|(_, id)| *id == a.id),
            "{}: no weapon has {} as its Art; weapons have {active:?}",
            b.name,
            a.name
        );
    }
}

#[test]
fn m5_build_colossus_is_equippable() {
    check_build(&Build {
        name: "Colossus",
        weapons: &[
            w("Forgeheart Maul", RIGHT_1),
            w("Greatanchor", RIGHT_2),
            w("Gray Knight Greatsword", LEFT_1),
        ],
        arts: &["Rallying Roar", "Earthsplit"],
        spells: &["Steadfast"],
        charms: &["Ox", "Last Stand"],
        armor: &["Graymantle Plate"],
    });
}

#[test]
fn m5_build_bloodletter_is_equippable() {
    check_build(&Build {
        name: "Bloodletter",
        weapons: &[
            w("banana.cpp", RIGHT_1),
            w("Whisperfang", RIGHT_2),
            w("Bramble Claws", LEFT_1),
        ],
        arts: &["Rending Sweep", "Feint Step"],
        spells: &[],
        charms: &["Thornheart", "Quickening"],
        armor: &["Reedwalker Garb"],
    });
}

#[test]
fn m5_build_scholar_is_equippable() {
    // "All Formulae" is checked in `m5_build_scholar_attunes_every_formula_individually`
    // and `m5_build_scholar_attunes_all_nine_formulae_at_once`.
    check_build(&Build {
        name: "Scholar",
        weapons: &[
            w("Gradient Rod", RIGHT_1),
            w("Exile's Stylus", RIGHT_2),
            w("Glassneedle Estoc", LEFT_1),
        ],
        arts: &["Glass Barrage"],
        spells: &[],
        charms: &["Scholar's Seal", "Shrike-Lite Circuit"],
        armor: &["Exile's Robe"],
    });
}

/// The nine Formulae, by display name.
const FORMULAE: [&str; 9] = [
    "Aether Dart",
    "Aether Bolt",
    "Homing Motes",
    "Aether Wave",
    "Mirror Ward",
    "Veil",
    "Gradient Descent",
    "Glasswing Swarm",
    "Unbinding Lance",
];

/// A Scholar world: levels for every Formula and the Scholar's Seal and Shrike-Lite Circuit
/// worn, at `focus`.
fn scholar_world(c: &Content, focus: u16) -> World {
    let names = Names(c);
    let mut w = resting_world(c);
    let mut want = attrs(&w);
    want[FOCUS] = focus;
    for f in FORMULAE {
        let r = names.spell(f).requirements;
        want[MIGHT] = want[MIGHT].max(r.might);
        want[FINESSE] = want[FINESSE].max(r.finesse);
        want[INSIGHT] = want[INSIGHT].max(r.insight);
        want[DEVOTION] = want[DEVOTION].max(r.devotion);
    }
    level_to(&mut w, want);
    for (i, ch) in ["Scholar's Seal", "Shrike-Lite Circuit"].iter().enumerate() {
        let d = names.charm(ch);
        equip_gear(&mut w, slot::CHARM_1 + i as u8, num(c, &d.id), ch);
    }
    w
}

#[test]
fn m5_build_scholar_attunes_every_formula_individually() {
    let c = content();
    let names = Names(&c);
    let mut w = scholar_world(&c, c.attributes.max);
    for f in FORMULAE {
        let n = num(&c, &names.spell(f).id);
        attune_spell(&mut w, 0, n, f);
        step(
            &mut w,
            vec![Command::Unequip {
                slot: slot::SPELL_1,
            }],
        );
        assert_eq!(w.spells.attuned[0], None, "{f} is freed");
    }
}

/// "All Formulae" read as "all nine attuned together". The nine take 11 spell slots (seven
/// take one, Gradient Descent and Unbinding Lance take two). Focus 60, the attribute
/// maximum, gives 8 slots and the Scholar's Seal adds 1, so 9 is the most a build can hold.
/// If the spec means a Scholar swaps Formulae at Lanterns, this is not a gap.
#[test]
#[ignore = "all nine Formulae need 11 spell slots; Focus 60 (max) plus the Scholar's Seal gives 9"]
fn m5_build_scholar_attunes_all_nine_formulae_at_once() {
    let c = content();
    let names = Names(&c);
    let mut w = scholar_world(&c, c.attributes.max);
    for (pos, f) in FORMULAE.iter().enumerate() {
        assert!(pos < MAX_ATTUNED);
        attune_spell(&mut w, pos as u8, num(&c, &names.spell(f).id), f);
    }
}

#[test]
fn m5_build_paladin_is_equippable() {
    check_build(&Build {
        name: "Paladin",
        weapons: &[
            w("Chapel Longsword", RIGHT_1),
            w("Choir Glaive", RIGHT_2),
            w("Castle Kite Shield", LEFT_1),
        ],
        arts: &["Radiant Blade", "Bastion"],
        spells: &["Mend", "Radiant Edge", "Steadfast", "Purge"],
        charms: &["Vital", "Bulwark"],
        armor: &["Graymantle Plate", "Chapel Vestments"],
    });
}

#[test]
fn m5_build_frostbinder_is_equippable() {
    check_build(&Build {
        name: "Frostbinder",
        weapons: &[
            w("Rimeforged Claymore", RIGHT_1),
            wt("Gray Halberd", RIGHT_2, "Rimed"),
            w("Bellwright's Mallet", LEFT_1),
        ],
        arts: &["Rime Edge"],
        spells: &["Mirror Ward"],
        charms: &["Rimeward", "Ox"],
        armor: &["Rimeforged Mail"],
    });
}

#[test]
fn m5_build_spellblade_is_equippable() {
    check_build(&Build {
        name: "Spellblade",
        weapons: &[
            wt("Stillwater Rapier", RIGHT_1, "Lucid"),
            w("Glassneedle Estoc", RIGHT_2),
            w("Gradient Rod", LEFT_1),
        ],
        arts: &["Lunging Thrust"],
        spells: &["Aether Bolt", "Homing Motes", "Mirror Ward", "Veil"],
        charms: &["Shrike-Lite Circuit", "Quickening"],
        armor: &["Reedwalker Garb", "Boo"],
    });
}

#[test]
fn m5_build_bulwark_is_equippable() {
    check_build(&Build {
        name: "Bulwark",
        weapons: &[
            w("Graymantle Greatshield", LEFT_1),
            w("Gray Halberd", RIGHT_1),
            w("NoPokeDB", LEFT_2),
        ],
        arts: &["Bastion"],
        spells: &["Steadfast"],
        charms: &["Bulwark", "Ox"],
        armor: &["Graymantle Plate"],
    });
}

#[test]
fn m5_build_ranger_is_equippable() {
    check_build(&Build {
        name: "Ranger",
        weapons: &[
            w("Hunter's Shortbow", RIGHT_1),
            w("Fungal Knife", LEFT_1),
            w("Spire Longbow", RIGHT_2),
        ],
        arts: &["Spark Volley"],
        spells: &[],
        charms: &["Quickening", "Vital"],
        // Both are body armor, so the table's "Reedwalker Garb, endark" is two loadouts.
        armor: &["Reedwalker Garb", "endark"],
    });
}

#[test]
fn m5_build_tempest_lancer_is_equippable() {
    check_build(&Build {
        name: "Tempest Lancer",
        weapons: &[
            w("ferray", RIGHT_1),
            w("Chapel Reliquary", RIGHT_2),
            w("Whitewarden Seal", LEFT_1),
        ],
        arts: &["Lunging Thrust"],
        spells: &["Storm Spear", "Ember Toss", "Mend", "Purge", "Wide Mend"],
        charms: &["Ember Locket", "Lantern Moth"],
        armor: &["Chapel Vestments"],
    });
}
