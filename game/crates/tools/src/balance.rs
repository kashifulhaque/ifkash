//! `lf-tools balance`: the headless balance report (WP-089).
//!
//! The report plays each of the nine builds (docs/game/04-arsenal-and-builds.md, "Builds")
//! against each required boss (docs/game/05-enemies-and-bosses.md, "Boss roster") at the
//! expected level (docs/game/06-progression-and-systems.md, "Expected level curve"). The
//! player is the boss's M7 scripted bot (`lf_core::ai::*_tests::Bot`, exposed by the `bots`
//! feature): it rolls through each hit and punishes in the gaps with `LIGHT`. Only the
//! loadout changes from build to build: the attributes, the weapons, the temper and upgrade
//! level, the armor, the Charms, the attuned spells, and the flasks.
//!
//! For each pair, the report plays a fixed list of seeds and prints the time to kill (the
//! ticks from the fight's start to the boss's defeat, averaged over the wins), the hits the
//! player took and landed, and the win rate. A pair passes when its time to kill is within
//! [`TOLERANCE_PERCENT`] of [`TARGET_TICKS`]. Everything is deterministic: the seeds are
//! fixed and the output is sorted by boss, then build.

use std::path::Path;
use std::process::ExitCode;

use lf_content::{Content, NumId};
use lf_core::actor::stats::Attrs;
use lf_core::ai::boss::Stage;
use lf_core::combat::spells;
use lf_core::events::{Event, HitResult, LifeEvent};
use lf_core::input::{ActionFrame, Command, TickInput, buttons};
use lf_core::items::{gear, slot};
use lf_core::tick::tick;
use lf_core::world::World;

/// The target time to kill, in ticks: docs/game/03-combat.md, "Tuning targets", sets a boss
/// fight at 2-4 minutes. The target is the middle, 3 minutes at 30 ticks a second.
const TARGET_TICKS: u64 = 5400;
/// How far from the target a pair may be and pass (the WP-089 acceptance bullet).
const TOLERANCE_PERCENT: u64 = 30;
/// The seeds each pair plays. Fixed, so the report is the same on every run.
const SEEDS: [u64; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
/// The longest fight the report plays before it calls the run a loss (a stuck run).
const MAX_TICKS: u64 = 4 * TARGET_TICKS;
/// Ticks played after the fight ends, so the last events land.
const AFTER_TICKS: u32 = 30;

/// Attribute positions, in `Attrs` order.
const VIT: usize = 0;
const FOC: usize = 1;
const END: usize = 2;
const MIG: usize = 3;
const FIN: usize = 4;
const INS: usize = 5;
const DEV: usize = 6;

/// The point in the game a boss is met at: the expected level and what the player has
/// found by then.
#[derive(Clone, Copy)]
struct Tier {
    /// docs/game/06-progression-and-systems.md, "Expected level curve".
    level: u32,
    /// The upgrade level of a common weapon, and the Starglass level of a unique one. The
    /// main path has no Forgeheart Ember (B08 is optional), so +5 and Starglass +2 are the
    /// most a main-path player reaches (docs/game/04-arsenal-and-builds.md, "Upgrades").
    common: u8,
    starglass: u8,
    /// Flask charges (Wick Beads found) and potency (Flask Dew found).
    charges: u8,
    flask_level: u8,
    /// Whether the build has its core kit yet; B01 is fought with the starting class's gear.
    core: bool,
}

/// B01: the start of the game, at the starting class's level of about 10.
const OPENING: Tier = Tier {
    level: 10,
    common: 0,
    starglass: 0,
    charges: 4,
    flask_level: 0,
    core: false,
};
/// A Great Sigil boss: between the first (level 28) and the second (level 38).
const SIGIL: Tier = Tier {
    level: 33,
    common: 3,
    starglass: 1,
    charges: 6,
    flask_level: 2,
    core: true,
};
/// B12: "Final boss, main path", level 45.
const FINALE: Tier = Tier {
    level: 45,
    common: 5,
    starglass: 2,
    charges: 8,
    flask_level: 3,
    core: true,
};

/// The bosses the report plays. All are on the main path: B01, the four Great Sigil bosses
/// (a run needs two of them), and B12.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Boss {
    B01,
    B06,
    B07,
    B09,
    B10,
    B12,
}

impl Boss {
    const ALL: [Boss; 6] = [
        Boss::B01,
        Boss::B06,
        Boss::B07,
        Boss::B09,
        Boss::B10,
        Boss::B12,
    ];

    fn code(self) -> &'static str {
        match self {
            Boss::B01 => "B01",
            Boss::B06 => "B06",
            Boss::B07 => "B07",
            Boss::B09 => "B09",
            Boss::B10 => "B10",
            Boss::B12 => "B12",
        }
    }

    fn scenario(self) -> &'static str {
        match self {
            Boss::B01 => "boss_b01",
            Boss::B06 => "boss_b06",
            Boss::B07 => "boss_b07",
            Boss::B09 => "boss_b09",
            Boss::B10 => "boss_b10",
            Boss::B12 => "boss_b12",
        }
    }

    fn tier(self) -> Tier {
        match self {
            Boss::B01 => OPENING,
            Boss::B12 => FINALE,
            _ => SIGIL,
        }
    }

    fn bot(self) -> AnyBot {
        use lf_core::ai::{b01_tests, bosses};
        match self {
            Boss::B01 => AnyBot::B01(b01_tests::Bot::default()),
            Boss::B06 => AnyBot::B06(bosses::b06_tests::Bot::default()),
            Boss::B07 => AnyBot::B07(bosses::b07_tests::Bot::default()),
            Boss::B09 => AnyBot::B09(bosses::b09_tests::Bot::default()),
            Boss::B10 => AnyBot::B10(bosses::b10_tests::Bot::default()),
            Boss::B12 => AnyBot::B12(bosses::b12_tests::Bot::default()),
        }
    }
}

/// The boss's scripted bot.
enum AnyBot {
    B01(lf_core::ai::b01_tests::Bot),
    B06(lf_core::ai::bosses::b06_tests::Bot),
    B07(lf_core::ai::bosses::b07_tests::Bot),
    B09(lf_core::ai::bosses::b09_tests::Bot),
    B10(lf_core::ai::bosses::b10_tests::Bot),
    B12(lf_core::ai::bosses::b12_tests::Bot),
}

impl AnyBot {
    fn act(&mut self, w: &World) -> ActionFrame {
        match self {
            AnyBot::B01(b) => b.act(w),
            AnyBot::B06(b) => b.act(w),
            AnyBot::B07(b) => b.act(w),
            AnyBot::B09(b) => b.act(w),
            AnyBot::B10(b) => b.act(w),
            AnyBot::B12(b) => b.act(w),
        }
    }
}

/// A set of gear.
struct Kit {
    right: &'static str,
    left: Option<&'static str>,
    /// The temper's ID in `upgrades.ron`, for a common right-hand weapon.
    temper: Option<&'static str>,
    armor: &'static str,
    charms: &'static [&'static str],
    /// Attuned in order; the first is selected, and the bot casts it with `LIGHT` when the
    /// right hand holds a catalyst.
    spells: &'static [&'static str],
    arrows: bool,
}

/// One of the nine builds.
struct Build {
    name: &'static str,
    /// The starting class's attributes (docs/game/04-arsenal-and-builds.md, "Starting
    /// classes"), in `Attrs` order.
    class: [u16; 7],
    /// Where each level goes, in turn, once the gear's requirements are met: the build's
    /// main attributes, Vitality, and its third attribute.
    pattern: &'static [usize],
    /// The starting class's gear (B01).
    early: Kit,
    /// The build's core kit: the first weapon, Art, Charms, armor, and spells of its row.
    core: Kit,
}

const WAYFARER: [u16; 7] = [12, 10, 12, 12, 12, 6, 6];
const WARDEN: [u16; 7] = [14, 6, 14, 14, 8, 7, 7];
const DUELIST: [u16; 7] = [10, 8, 12, 8, 16, 9, 6];
const SCHOLAR: [u16; 7] = [9, 14, 9, 7, 10, 16, 4];
const ACOLYTE: [u16; 7] = [11, 12, 9, 10, 8, 4, 16];
const HUNTER: [u16; 7] = [10, 8, 13, 9, 15, 8, 6];

const NO_SPELLS: &[&str] = &[];
const NO_CHARMS: &[&str] = &[];

const BUILDS: [Build; 9] = [
    Build {
        name: "Colossus",
        class: WARDEN,
        pattern: &[MIG, VIT, MIG, END, MIG, VIT],
        early: Kit {
            right: "wpn_wayfarer_blade",
            left: Some("wpn_castle_kite"),
            temper: None,
            armor: "miner",
            charms: NO_CHARMS,
            spells: NO_SPELLS,
            arrows: false,
        },
        core: Kit {
            right: "wpn_forgeheart_maul",
            left: None,
            temper: None,
            armor: "graymantle",
            charms: &["chm_ox", "chm_last_stand"],
            spells: NO_SPELLS,
            arrows: false,
        },
    },
    Build {
        name: "Bloodletter",
        class: DUELIST,
        pattern: &[FIN, VIT, FIN, END, FIN, VIT],
        early: Kit {
            right: "wpn_fungal_knife",
            left: Some("wpn_parrying_dirk"),
            temper: None,
            armor: "reedwalker",
            charms: NO_CHARMS,
            spells: NO_SPELLS,
            arrows: false,
        },
        core: Kit {
            right: "wpn_banana_cpp",
            left: None,
            temper: None,
            armor: "reedwalker",
            charms: &["chm_thornheart", "chm_quickening"],
            spells: NO_SPELLS,
            arrows: false,
        },
    },
    Build {
        name: "Scholar",
        class: SCHOLAR,
        pattern: &[INS, VIT, INS, FOC, INS, VIT],
        early: Kit {
            right: "wpn_scholars_stylus",
            left: None,
            temper: None,
            armor: "exile",
            charms: NO_CHARMS,
            spells: &["spl_aether_bolt", "spl_aether_dart"],
            arrows: false,
        },
        core: Kit {
            right: "wpn_gradient_rod",
            left: None,
            temper: None,
            armor: "exile",
            charms: &["chm_scholars_seal", "chm_shrike_lite"],
            spells: &["spl_aether_bolt", "spl_homing_motes", "spl_mirror_ward"],
            arrows: false,
        },
    },
    Build {
        name: "Paladin",
        class: ACOLYTE,
        pattern: &[DEV, MIG, VIT, DEV, MIG, VIT],
        early: Kit {
            right: "wpn_wayfarer_blade",
            left: Some("wpn_chapel_reliquary"),
            temper: None,
            armor: "chapel",
            charms: NO_CHARMS,
            spells: NO_SPELLS,
            arrows: false,
        },
        core: Kit {
            right: "wpn_chapel_longsword",
            left: Some("wpn_castle_kite"),
            temper: None,
            armor: "graymantle",
            charms: &["chm_vital", "chm_bulwark"],
            spells: NO_SPELLS,
            arrows: false,
        },
    },
    Build {
        name: "Frostbinder",
        class: WAYFARER,
        pattern: &[MIG, INS, VIT, MIG, INS, VIT],
        early: Kit {
            right: "wpn_wayfarer_blade",
            left: Some("wpn_plank_buckler"),
            temper: None,
            armor: "wayfarer",
            charms: NO_CHARMS,
            spells: NO_SPELLS,
            arrows: false,
        },
        core: Kit {
            right: "wpn_rimeforged_claymore",
            left: None,
            temper: None,
            armor: "rimeforged",
            charms: &["chm_rimeward", "chm_ox"],
            spells: NO_SPELLS,
            arrows: false,
        },
    },
    Build {
        name: "Spellblade",
        class: SCHOLAR,
        pattern: &[FIN, INS, VIT, FIN, INS, VIT],
        early: Kit {
            right: "wpn_wayfarer_blade",
            left: None,
            temper: None,
            armor: "exile",
            charms: NO_CHARMS,
            spells: NO_SPELLS,
            arrows: false,
        },
        core: Kit {
            right: "wpn_stillwater_rapier",
            left: None,
            temper: Some("lucid"),
            armor: "reedwalker",
            charms: &["chm_shrike_lite", "chm_quickening"],
            spells: NO_SPELLS,
            arrows: false,
        },
    },
    Build {
        name: "Bulwark",
        class: WARDEN,
        pattern: &[END, MIG, VIT, MIG, END, VIT],
        early: Kit {
            right: "wpn_wayfarer_blade",
            left: Some("wpn_castle_kite"),
            temper: None,
            armor: "miner",
            charms: NO_CHARMS,
            spells: NO_SPELLS,
            arrows: false,
        },
        core: Kit {
            right: "wpn_gray_halberd",
            left: Some("wpn_graymantle_greatshield"),
            temper: None,
            armor: "graymantle",
            charms: &["chm_bulwark", "chm_ox"],
            spells: NO_SPELLS,
            arrows: false,
        },
    },
    Build {
        name: "Ranger",
        class: HUNTER,
        pattern: &[FIN, END, VIT, FIN, END, VIT],
        early: Kit {
            right: "wpn_hunter_shortbow",
            left: Some("wpn_fungal_knife"),
            temper: None,
            armor: "reedwalker",
            charms: NO_CHARMS,
            spells: NO_SPELLS,
            arrows: true,
        },
        core: Kit {
            right: "wpn_hunter_shortbow",
            left: Some("wpn_fungal_knife"),
            temper: None,
            armor: "reedwalker",
            charms: &["chm_quickening", "chm_vital"],
            spells: NO_SPELLS,
            arrows: true,
        },
    },
    Build {
        name: "Tempest Lancer",
        class: ACOLYTE,
        pattern: &[DEV, FIN, VIT, FIN, DEV, VIT],
        early: Kit {
            right: "wpn_wayfarer_blade",
            left: Some("wpn_chapel_reliquary"),
            temper: None,
            armor: "chapel",
            charms: NO_CHARMS,
            spells: NO_SPELLS,
            arrows: false,
        },
        core: Kit {
            right: "wpn_ferray",
            left: Some("wpn_chapel_reliquary"),
            temper: None,
            armor: "chapel",
            charms: &["chm_ember_locket", "chm_lantern_moth"],
            spells: NO_SPELLS,
            arrows: false,
        },
    },
];

/// One fight's result.
struct Run {
    won: bool,
    /// Ticks from the fight's start to the boss's defeat (or to the end of the run).
    ticks: u64,
    hits_taken: u32,
    hits_landed: u32,
    /// Damage the player dealt, and critical hits (ripostes and stance-break crits) started.
    damage: u64,
    crits: u32,
}

/// One build against one boss, over every seed.
struct Row {
    boss: Boss,
    build: &'static str,
    level: u32,
    runs: Vec<Run>,
    /// Setup problems: gear that couldn't be equipped, a requirement the level can't meet.
    notes: Vec<String>,
}

impl Row {
    fn wins(&self) -> Vec<&Run> {
        self.runs.iter().filter(|r| r.won).collect()
    }

    /// The mean time to kill over the wins.
    fn ttk(&self) -> Option<u64> {
        let wins = self.wins();
        (!wins.is_empty()).then(|| wins.iter().map(|r| r.ticks).sum::<u64>() / wins.len() as u64)
    }

    fn mean(&self, f: impl Fn(&Run) -> u32) -> f32 {
        let n = self.runs.len().max(1) as f32;
        self.runs.iter().map(|r| f(r) as f32).sum::<f32>() / n
    }

    fn passes(&self) -> bool {
        self.ttk()
            .is_some_and(|t| t.abs_diff(TARGET_TICKS) * 100 <= TARGET_TICKS * TOLERANCE_PERCENT)
    }
}

/// The registry number of a content ID.
fn num(content: &Content, id: &str) -> Option<NumId> {
    content.registry.num(id)
}

/// The attributes for `build` at `tier`: the class's, then the gear's requirements, then
/// the build's pattern until the expected level. Notes a requirement the level can't meet.
fn attributes(
    build: &Build,
    kit: &Kit,
    tier: Tier,
    w: &World,
    content: &Content,
    notes: &mut Vec<String>,
) -> Attrs {
    let rules = &w.attributes;
    let max = rules.max;
    let mut v = build.class;
    let level =
        |v: &[u16; 7]| v.iter().map(|&a| u32::from(a)).sum::<u32>() - u32::from(rules.level_offset);
    // The requirements of the gear in hand and the attuned spells, as [M, F, I, D].
    let mut need = [0u16; 4];
    let mut add = |req: &[u16; 4]| {
        for (n, r) in need.iter_mut().zip(req) {
            *n = (*n).max(*r);
        }
    };
    for id in [Some(kit.right), kit.left].into_iter().flatten() {
        if let Some(s) = num(content, id).and_then(|n| w.damage.weapon(n)) {
            add(&s.requirements);
        }
    }
    for id in kit.spells {
        if let Some(s) = w
            .spells_table
            .num_of(id)
            .and_then(|n| w.spells_table.get(n))
        {
            add(&s.requirements);
        }
    }
    for (i, &r) in [MIG, FIN, INS, DEV].iter().zip(&need) {
        while v[*i] < r && level(&v) < tier.level {
            v[*i] += 1;
        }
        if v[*i] < r {
            notes.push(format!("attribute {i} short of {r}"));
        }
    }
    let mut k = 0usize;
    let mut stuck = 0usize;
    while level(&v) < tier.level && stuck < build.pattern.len() {
        let a = build.pattern[k % build.pattern.len()];
        k += 1;
        if v[a] < max {
            v[a] += 1;
            stuck = 0;
        } else {
            stuck += 1;
        }
    }
    lf_core::actor::level::from_values(v)
}

/// A world for `build` against `boss` on `seed`, with the loadout set up before tick 0.
fn setup(
    content: &Content,
    boss: Boss,
    build: &Build,
    seed: u64,
    notes: &mut Vec<String>,
) -> Option<World> {
    let mut w = match World::new(seed, content, boss.scenario()) {
        Ok(w) => w,
        Err(e) => {
            notes.push(format!("world: {e:?}"));
            return None;
        }
    };
    let tier = boss.tier();
    let kit = if tier.core { &build.core } else { &build.early };
    let attrs = attributes(build, kit, tier, &w, content, notes);
    let player = w.player;
    let caster = !kit.spells.is_empty();

    // Attributes; `gear::refresh` below writes what they drive.
    w.actors.get_mut(player)?.stats.attrs = attrs;

    // Gear.
    let items = w.items.clone();
    let mut equip = |w: &mut World, s: u8, id: &str| {
        let Some(n) = num(content, id) else {
            notes.push(format!("unknown item {id}"));
            return;
        };
        let p = w.actors.get_mut(player).expect("player");
        p.inventory.give_gear(n);
        if !p.inventory.equip(s, n, &items) {
            notes.push(format!("couldn't equip {id}"));
        }
    };
    {
        let p = w.actors.get_mut(player)?;
        for s in [slot::RIGHT_1, slot::RIGHT_2, slot::LEFT_1, slot::LEFT_2] {
            p.inventory.unequip(s);
        }
    }
    equip(&mut w, slot::RIGHT_1, kit.right);
    if let Some(l) = kit.left {
        equip(&mut w, slot::LEFT_1, l);
    }
    equip(&mut w, slot::HEAD, &format!("arm_{}_head", kit.armor));
    equip(&mut w, slot::BODY, &format!("arm_{}_body", kit.armor));
    for (i, c) in kit.charms.iter().enumerate() {
        equip(&mut w, slot::CHARM_1 + i as u8, c);
    }
    if kit.arrows
        && let Some(n) = num(content, "itm_arrows")
    {
        let p = w.actors.get_mut(player)?;
        p.inventory.pick_up(n, 99, &items);
        if !p.inventory.equip(slot::ARROWS, n, &items) {
            notes.push("couldn't nock arrows".into());
        }
    }

    // Upgrades and temper, in the world's flags (`items::upgrade`).
    for id in [Some(kit.right), kit.left].into_iter().flatten() {
        let Some(n) = num(content, id) else { continue };
        let unique = w.damage.weapon(n).is_some_and(|s| s.unique);
        let steps = if unique { tier.starglass } else { tier.common };
        if steps > 0 {
            w.state
                .flags
                .set(&format!("weapon.level.{n}"), u32::from(steps));
        }
    }
    if let Some(t) = kit.temper
        && let Some(n) = num(content, kit.right)
    {
        let index = w
            .upgrades
            .as_ref()
            .and_then(|u| u.tempers.iter().position(|x| x.id == t));
        match index {
            Some(i) => w
                .state
                .flags
                .set(&format!("weapon.temper.{n}"), i as u32 + 1),
            None => notes.push(format!("unknown temper {t}")),
        }
    }

    w.sync_player_moveset();
    gear::refresh(&mut w);

    // Spells: known (in the gear), then attuned in order.
    for (i, id) in kit.spells.iter().enumerate() {
        let Some(n) = w.spells_table.num_of(id) else {
            notes.push(format!("unknown spell {id}"));
            continue;
        };
        w.actors.get_mut(player)?.inventory.give_gear(n);
        if !spells::attune(&mut w, i as u8, n) {
            notes.push(format!("couldn't attune {id}"));
        }
    }

    // Flasks, then full HP, FP, and stamina.
    let derived = attrs.derived(&w.attributes);
    let p = w.actors.get_mut(player)?;
    let f = &mut p.inventory.flasks;
    f.max_charges = tier.charges;
    // A caster splits the charges evenly; anyone else keeps one Lucid charge for an Art.
    f.vigil_max = if caster {
        tier.charges - tier.charges / 2
    } else {
        tier.charges - 1
    };
    f.level = tier.flask_level;
    f.refill();
    // `gear::refresh` set the HP maximum (with the gear's bonus); the attributes set the rest.
    p.max_fp = derived.max_fp;
    p.max_stamina = derived.max_stamina;
    p.hp = p.max_hp;
    p.fp = p.max_fp;
    p.stamina = p.max_stamina;
    Some(w)
}

/// Plays one fight.
fn fight(mut w: World, boss: Boss) -> Run {
    let mut bot = boss.bot();
    let player = w.player;
    let mut start: Option<u64> = None;
    let mut end: Option<u64> = None;
    let mut died = false;
    let mut hits_taken = 0;
    let mut hits_landed = 0;
    let mut damage = 0u64;
    let mut crits = 0;
    let mut after = 0u32;
    while w.tick < MAX_TICKS + 2_000 && after < AFTER_TICKS {
        let mut frame = bot.act(&w);
        let mut commands = Vec::new();
        // The bots were written for a melee weapon. A caster out of FP drinks a Lucid Flask
        // (quick slot 2) in a gap where the bot would have attacked instead.
        if frame.buttons & buttons::LIGHT != 0 && out_of_fp(&w) {
            frame.buttons &= !buttons::LIGHT;
            commands.push(Command::UseItem { slot: 1 });
        }
        tick(&mut w, &TickInput { frame, commands });
        for e in w.events.drain() {
            match e {
                Event::Hit(h) if h.target == player && h.result != HitResult::Guarded => {
                    hits_taken += 1;
                }
                Event::Hit(h) if h.attacker == player && h.damage > 0 => {
                    hits_landed += 1;
                    damage += u64::from(h.damage);
                }
                Event::Critical(c) if c.attacker == player => crits += 1,
                Event::Life(LifeEvent::Died { .. }) => died = true,
                _ => {}
            }
        }
        let stage = w.bosses[0].stage;
        if start.is_none() && stage == Stage::Fight {
            start = Some(w.tick);
        }
        if end.is_none() && stage == Stage::Defeated {
            end = Some(w.tick);
        }
        if died || end.is_some() {
            after += 1;
        }
        if start.is_some_and(|s| w.tick - s > MAX_TICKS) {
            break;
        }
    }
    let s = start.unwrap_or(0);
    Run {
        won: end.is_some() && !died,
        ticks: end.unwrap_or(w.tick) - s,
        hits_taken,
        hits_landed,
        damage,
        crits,
    }
}

/// Whether the player holds a catalyst, can't pay for the selected spell, and has a Lucid
/// Flask charge left.
fn out_of_fp(w: &World) -> bool {
    let Some(p) = w.actors.get(w.player) else {
        return false;
    };
    let catalyst = p
        .inventory
        .equipment
        .right_weapon()
        .is_some_and(|n| w.spells_table.catalyst(n).is_some());
    let cost = w
        .spells
        .selected_spell()
        .and_then(|s| w.spells_table.get(s))
        .and_then(|s| w.moves.get(s.cast))
        .map_or(0, |m| m.data.fp);
    catalyst && p.inventory.flasks.lucid > 0 && p.fp < f32::from(cost)
}

fn play(content: &Content, boss: Boss, build: &'static Build) -> Row {
    let mut notes = Vec::new();
    let mut runs = Vec::new();
    for (i, &seed) in SEEDS.iter().enumerate() {
        // Setup notes are the same on every seed; keep the first seed's.
        let mut n = Vec::new();
        let Some(w) = setup(content, boss, build, seed, &mut n) else {
            notes.extend(n);
            break;
        };
        if i == 0 {
            notes.extend(n);
        }
        runs.push(fight(w, boss));
    }
    Row {
        boss,
        build: build.name,
        level: boss.tier().level,
        runs,
        notes,
    }
}

fn seconds(ticks: u64) -> String {
    format!("{}:{:02}", ticks / 1800, ticks / 30 % 60)
}

/// Runs the report. `boss` and `build` filter it (case-insensitive prefixes).
pub fn run(data: &Path, boss: Option<&str>, build: Option<&str>) -> ExitCode {
    let content = match Content::load_dir(data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("balance: {e}");
            return ExitCode::FAILURE;
        }
    };
    let matches = |filter: Option<&str>, name: &str| {
        filter.is_none_or(|f| name.to_lowercase().starts_with(&f.to_lowercase()))
    };
    let mut jobs: Vec<(Boss, &'static Build)> = Vec::new();
    for b in Boss::ALL {
        for bd in &BUILDS {
            if matches(boss, b.code()) && matches(build, bd.name) {
                jobs.push((b, bd));
            }
        }
    }
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut rows: Vec<Row> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                s.spawn(|| {
                    let mut out = Vec::new();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(&(b, bd)) = jobs.get(i) else { break };
                        out.push(play(&content, b, bd));
                    }
                    out
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("balance worker"))
            .collect()
    });
    rows.sort_by(|a, b| {
        (a.boss, BUILDS.iter().position(|x| x.name == a.build))
            .cmp(&(b.boss, BUILDS.iter().position(|x| x.name == b.build)))
    });

    let lo = TARGET_TICKS - TARGET_TICKS * TOLERANCE_PERCENT / 100;
    let hi = TARGET_TICKS + TARGET_TICKS * TOLERANCE_PERCENT / 100;
    println!(
        "Balance report: target time to kill {} ({} ticks), pass band {}-{} ({}%), seeds {:?}",
        seconds(TARGET_TICKS),
        TARGET_TICKS,
        seconds(lo),
        seconds(hi),
        TOLERANCE_PERCENT,
        SEEDS
    );
    println!();
    println!(
        "| Boss | Build | Level | TTK | vs target | Hits taken | Hits landed | Damage/hit | Crits | Win rate | Result |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|");
    let mut failed = 0;
    for r in &rows {
        let wins = r.wins().len();
        let (ttk, delta) = match r.ttk() {
            Some(t) => (
                format!("{} ({t})", seconds(t)),
                format!(
                    "{:+}%",
                    (t as i64 - TARGET_TICKS as i64) * 100 / TARGET_TICKS as i64
                ),
            ),
            None => ("no win".into(), "n/a".into()),
        };
        let ok = r.passes();
        if !ok {
            failed += 1;
        }
        let landed: u64 = r.runs.iter().map(|x| u64::from(x.hits_landed)).sum();
        let dealt: u64 = r.runs.iter().map(|x| x.damage).sum();
        println!(
            "| {} | {} | {} | {} | {} | {:.1} | {:.1} | {} | {:.1} | {}/{} | {} |",
            r.boss.code(),
            r.build,
            r.level,
            ttk,
            delta,
            r.mean(|x| x.hits_taken),
            r.mean(|x| x.hits_landed),
            dealt / landed.max(1),
            r.mean(|x| x.crits),
            wins,
            r.runs.len(),
            if ok { "pass" } else { "FAIL" },
        );
    }
    let noted: Vec<&Row> = rows.iter().filter(|r| !r.notes.is_empty()).collect();
    if !noted.is_empty() {
        println!();
        println!("Setup notes:");
        for r in noted {
            println!("- {} {}: {}", r.boss.code(), r.build, r.notes.join("; "));
        }
    }
    println!();
    println!(
        "{} of {} pairs within the target.",
        rows.len() - failed,
        rows.len()
    );
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
