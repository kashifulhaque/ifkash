//! B04, the Mossgut Brood: the egg sacs and the weak point. See
//! docs/game/05-enemies-and-bosses.md, "B04 Mossgut Brood", and `data/bosses/b04_*.ron`.
//!
//! The mother, her moves, her phase 2 crawl, and the sacs as parts are all ordinary boss
//! data. Two rules aren't, and live here:
//!
//! - **Egg sacs spawn Marsh Crawlers every 300 ticks.** Each living sac, counted from the
//!   start of the fight, spawns one Marsh Crawler beside itself (the sacs are staggered so
//!   they don't all spawn together, and at most [`MAX_ALIVE`] spawned Crawlers live at once).
//!   A spawned Crawler is an ordinary AI actor whose spawn index is its sac's, so it walks
//!   home to the sac if it ever loses the player. This module keeps them in Combat while the
//!   fight lasts, and removes them when the fight resets or ends.
//! - **Destroying all four sacs exposes a weak point:** the mother takes double damage. The
//!   core does this by doubling her per-type weakness percents (a Fire hit goes from 1.5x to
//!   3x), which `combat::damage` already reads, so no damage code changes.
//!
//! The sacs' clocks are their own `AiState::state_ticks`: the fight puts every member into
//! Combat on its first tick, which sets the count to 0, so it reads "ticks since the fight
//! began". The mother's and the sacs' `dead_ticks` (a counter only revivers use) hold one
//! flag each, "its event was announced"; both are hashed with the actor.
//!
//! Events are `Event::Mechanic` with the sub-kinds below (WASM event kind 14).

use lf_content::scenario::EnemySpawn;

use crate::actor::stats::Defense;
use crate::ai::boss::{self, Stage};
use crate::ai::{Awareness, set_awareness};
use crate::arena::Id;
use crate::combat::hurtbox::Hurtbox;
use crate::events::{Event, MechanicEvent};
use crate::math::dmath::sqrt;
use crate::world::World;

/// The boss's ID (`data/bosses/b04_mossgut_brood.ron`).
pub const BOSS_ID: &str = "b04_mossgut_brood";
/// The enemy a sac spawns.
const CRAWLER_ID: &str = "marsh_crawler";

/// `MechanicEvent` sub-kind: egg sac `index` (0 to 3, in the boss's part order) was
/// destroyed.
pub const SAC_BURST: u8 = 130;
/// `MechanicEvent` sub-kind: the last sac fell and the mother's weak point is open (`index` 0).
pub const WEAK_POINT_EXPOSED: u8 = 131;
/// `MechanicEvent` sub-kind: egg sac `index` spawned a Marsh Crawler.
pub const CRAWLER_SPAWNED: u8 = 132;

/// A named number from the boss's `params` (0 if the definition lacks it, which turns the
/// rule that reads it off). `spawn_every_ticks` is the spec's 300 ("Egg sacs spawn Marsh
/// Crawlers every 300 ticks"); `weak_point_percent` is its "2x damage" (200); `sac_stagger_ticks`
/// (75), `max_alive` (6), and `spawn_offset_m` (1.2) are tuning.
fn param(world: &World, b: usize, key: &str) -> f32 {
    world.layout.bosses[b]
        .def
        .params
        .get(key)
        .copied()
        .unwrap_or(0.0)
}

/// The index of B04 among the scenario's bosses, if it is there.
fn find(world: &World) -> Option<usize> {
    world.layout.bosses.iter().position(|l| l.def.id == BOSS_ID)
}

/// The mother's actor, and her sacs with their spawn indexes.
fn cast(world: &World, b: usize) -> Option<(Id, Vec<(Id, u16)>)> {
    let members = &world.layout.bosses[b].members;
    let at = |spawn: u16| world.state.ai_enemies.get(usize::from(spawn)).copied();
    let body = at(*members.first()?)?;
    let sacs = members
        .iter()
        .skip(1)
        .filter_map(|&spawn| at(spawn).map(|id| (id, spawn)))
        .collect();
    Some((body, sacs))
}

/// The whole-world hook: outside a fight, nothing of the old one stays.
pub fn run(world: &mut World) {
    let Some(b) = find(world) else {
        return;
    };
    let Some((_, sacs)) = cast(world, b) else {
        return;
    };
    match world.bosses.get(b).map(|r| r.stage) {
        Some(Stage::Defeated) => fell_crawlers(world, &sacs),
        // Dormant (a new fight, or the player died) or the intro.
        Some(Stage::Dormant | Stage::Intro { .. }) => clear_crawlers(world, &sacs),
        _ => {}
    }
}

/// One tick of B04's rules while boss `b` is in its Fight stage.
pub fn tick(world: &mut World, b: usize) {
    if world.layout.bosses[b].def.id != BOSS_ID {
        return;
    }
    if let Some((body, sacs)) = cast(world, b) {
        fight(world, b, body, &sacs);
    }
}

fn is_down(world: &World, id: Id) -> bool {
    world.actors.get(id).is_none_or(|a| a.dead || a.hp == 0)
}

fn emit(world: &mut World, kind: u8, index: u16) {
    world
        .events
        .push(Event::Mechanic(MechanicEvent { kind, index }));
}

/// The Crawlers the sacs spawned: every AI actor of the Crawler definition whose spawn index
/// is a sac's. In arena order.
fn crawlers(world: &World, sacs: &[(Id, u16)]) -> Vec<Id> {
    let Some(num) = world.ai.index_of(CRAWLER_ID).map(|i| world.ai.defs[i].num) else {
        return Vec::new();
    };
    world
        .actors
        .iter()
        .filter(|(_, a)| {
            a.ai.as_ref()
                .is_some_and(|b| b.def == num && sacs.iter().any(|&(_, s)| s == b.spawn))
        })
        .map(|(id, _)| id)
        .collect()
}

/// The fight is on: spawns, announcements, the weak point, and holding the Crawlers in the
/// fight.
fn fight(world: &mut World, b: usize, body: Id, sacs: &[(Id, u16)]) {
    let player = world.player;
    let Some(at) = world.actors.get(player).map(|p| p.pos) else {
        return;
    };

    // Each sac that has just been destroyed announces it once.
    for (i, &(id, _)) in sacs.iter().enumerate() {
        if is_down(world, id)
            && let Some(brain) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut())
            && brain.dead_ticks == 0
        {
            brain.dead_ticks = 1;
            emit(world, SAC_BURST, i as u16);
        }
    }

    // The weak point: every sac down, the mother up.
    let exposed = !sacs.is_empty() && sacs.iter().all(|&(id, _)| is_down(world, id));
    set_weak_point(world, b, body, exposed);

    // Spawns: a living sac whose clock reads a multiple of the period (after its stagger).
    let (every, stagger_step) = (
        param(world, b, "spawn_every_ticks") as u16,
        param(world, b, "sac_stagger_ticks") as u16,
    );
    let max_alive = param(world, b, "max_alive") as usize;
    let mut alive = crawlers(world, sacs)
        .into_iter()
        .filter(|&id| !is_down(world, id))
        .count();
    for (i, &(id, spawn)) in sacs.iter().enumerate() {
        let stagger = stagger_step * i as u16;
        let Some(a) = world.actors.get(id).filter(|a| !a.dead && a.hp > 0) else {
            continue;
        };
        let Some(ticks) = a.ai.as_ref().map(|b| b.state_ticks) else {
            continue;
        };
        // `state_ticks` saturates; a saturated clock never matches again.
        let due = every > 0
            && ticks < u16::MAX
            && ticks >= every + stagger
            && (ticks - stagger).is_multiple_of(every);
        if due && alive < max_alive && spawn_crawler(world, b, id, spawn, at) {
            alive += 1;
            emit(world, CRAWLER_SPAWNED, i as u16);
        }
    }

    // Every spawned Crawler fights on, wherever the player is.
    for id in crawlers(world, sacs) {
        hold_in_combat(world, id, player, at);
    }
}

/// Sets the mother's weakness percents to her definition's, doubled while the weak point is
/// open, and announces the opening once.
fn set_weak_point(world: &mut World, b: usize, body: Id, exposed: bool) {
    let Some(a) = world.actors.get(body) else {
        return;
    };
    let Some(brain) = a.ai.as_ref() else {
        return;
    };
    let base = Defense::from_data(&world.ai.def(brain.def).data.defense).weakness_percent;
    let announced = brain.dead_ticks > 0;
    let percent = param(world, b, "weak_point_percent") as u32;
    let want = base.map(|w| {
        if exposed {
            // Percents multiply: 150 (Fire) doubles to 300, 100 (everything else) to 200.
            (u32::from(w) * percent / 100).min(u32::from(u16::MAX)) as u16
        } else {
            w
        }
    });
    let Some(a) = world.actors.get_mut(body) else {
        return;
    };
    a.stats.defense.weakness_percent = want;
    let announce = exposed && !announced && !a.dead;
    if announce && let Some(brain) = a.ai.as_mut() {
        brain.dead_ticks = 1;
    }
    if announce {
        emit(world, WEAK_POINT_EXPOSED, 0);
    }
}

/// Spawns a Marsh Crawler beside sac `sac` (spawn index `spawn`), in Combat against the
/// player. False if it couldn't be made.
fn spawn_crawler(world: &mut World, b: usize, sac: Id, spawn: u16, toward: [f32; 3]) -> bool {
    let offset = param(world, b, "spawn_offset_m");
    let Some(from) = world.actors.get(sac).map(|a| (a.pos, a.yaw)) else {
        return false;
    };
    let (dx, dz) = (toward[0] - from.0[0], toward[2] - from.0[2]);
    let len = sqrt(dx * dx + dz * dz).max(0.001);
    let at = [
        from.0[0] + dx / len * offset,
        from.0[1],
        from.0[2] + dz / len * offset,
    ];
    let enemy = EnemySpawn {
        name: format!("{}.crawler", world.layout.enemies[usize::from(spawn)].name),
        enemy: CRAWLER_ID.to_owned(),
        pos: at,
        yaw: from.1,
        patrol: Vec::new(),
        wakes_on: None,
    };
    let hurtbox = Hurtbox::from_locomotion(&world.locomotion);
    let Some(mut actor) = crate::ai::enemy_actor(&enemy, spawn, &world.ai, hurtbox) else {
        return false;
    };
    actor.poise.unaware = false;
    actor.lock_target = Some(world.player);
    if let Some(brain) = actor.ai.as_mut() {
        brain.target = Some(world.player);
        brain.last_seen = toward;
    }
    let id = world.actors.insert(actor);
    set_awareness(world, id, Awareness::Combat);
    true
}

/// Keeps a spawned Crawler fighting, as `boss::run` does for the members: in Combat, aware
/// of the player wherever they are, and not tracking in the last ticks of a telegraph.
fn hold_in_combat(world: &mut World, id: Id, player: Id, at: [f32; 3]) {
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    if a.dead {
        return;
    }
    match a.moves.current {
        Some(c) => {
            if world
                .moves
                .get(c.num)
                .is_some_and(|d| c.frame + 1 + boss::TRACK_STOP_TICKS >= d.track_until)
            {
                a.lock_target = None;
            }
        }
        None => a.lock_target = Some(player),
    }
    let Some(brain) = a.ai.as_mut() else {
        return;
    };
    brain.target = Some(player);
    brain.last_seen = at;
    brain.unseen_ticks = 0;
    if brain.awareness != Awareness::Combat {
        set_awareness(world, id, Awareness::Combat);
    }
}

/// The fight is over (or hasn't begun): every spawned Crawler is removed.
fn clear_crawlers(world: &mut World, sacs: &[(Id, u16)]) {
    for id in crawlers(world, sacs) {
        world.actors.remove(id);
    }
}

/// The boss fell: the spawned Crawlers fall with her.
fn fell_crawlers(world: &mut World, sacs: &[(Id, u16)]) {
    for id in crawlers(world, sacs) {
        if let Some(a) = world.actors.get_mut(id)
            && !a.dead
        {
            a.hp = 0;
            a.dead = true;
            a.moves.current = None;
        }
    }
}
