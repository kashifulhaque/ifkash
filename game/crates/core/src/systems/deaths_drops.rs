//! Deaths and drops: the player's death, Remnant, and respawn (see
//! `world_state::death::run`), and the Glimmer an enemy pays out when it dies.

use crate::arena::Id;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    crate::world_state::death::run(world);
}

/// The AI actors alive now that pay Glimmer when they die. `tick` takes this list before
/// the systems run and hands it to `pay_enemy_glimmer` after them, so a death from any
/// cause in the tick (a hit, a status, a fall, a script) pays once, with no extra state.
pub fn living_enemies(world: &World) -> Vec<Id> {
    world
        .actors
        .iter()
        .filter(|(id, a)| {
            *id != world.player
                && !a.dead
                && a.ai
                    .as_ref()
                    .is_some_and(|b| !a.is_ally() && glimmer_of(world, b.def) > 0)
        })
        .map(|(id, _)| id)
        .collect()
}

/// Pays the player each enemy's `glimmer` (docs/game/06-progression-and-systems.md,
/// "Glimmer and death": enemies drop Glimmer on death) for every actor in `alive` that died
/// this tick for good. Bosses and their adds pay through the boss reward (`ai::boss`).
/// Issue #75.
pub fn pay_enemy_glimmer(world: &mut World, alive: &[Id]) {
    for &id in alive {
        if world.actors.get(id).is_some_and(|a| a.dead) {
            pay(world, id);
        }
    }
}

/// Pays the player for the dead enemy `id`, unless it will stand up again: an enemy with a
/// revive rule pays only when its death is final (`special::revive`), as Elden Ring's
/// skeletons do. A dead player has already dropped their Glimmer as a Remnant, so a trade
/// kill pays nothing.
pub fn pay(world: &mut World, id: Id) {
    let Some(b) = world.actors.get(id).and_then(|a| a.ai.as_ref()) else {
        return;
    };
    // A boss fight's members and the adds it spawns pay nothing: the boss's reward covers
    // the fight (`ai::boss`), and endless adds can't be farmed.
    if crate::ai::boss::boss_of_spawn(&world.layout, b.spawn).is_some() {
        return;
    }
    let def = world.ai.get(b.def);
    let revives = def.is_some_and(|d| d.data.revive.is_some());
    if revives && !b.stays_dead {
        return;
    }
    let amount = def.map_or(0, |d| d.data.glimmer);
    if amount > 0
        && let Some(p) = world.actors.get_mut(world.player)
        && !p.dead
    {
        p.inventory.glimmer = p.inventory.glimmer.saturating_add(amount);
    }
}

fn glimmer_of(world: &World, def: lf_content::NumId) -> u32 {
    world.ai.get(def).map_or(0, |d| d.data.glimmer)
}

#[cfg(test)]
mod tests {
    use crate::input::TickInput;
    use crate::tick::tick;
    use crate::world::World;
    use lf_content::Content;

    fn fight_husk() -> (World, crate::arena::Id) {
        let content = Content::load_embedded().unwrap();
        let w = World::new(3, &content, "fight_husk").unwrap();
        let husk = w.state.ai_enemies[0];
        (w, husk)
    }

    fn glimmer(w: &World) -> u32 {
        w.actors.get(w.player).unwrap().inventory.glimmer
    }

    /// A Husk that dies in a tick, here off the kill plane, pays its 15 Glimmer once.
    #[test]
    fn a_dying_enemy_pays_its_glimmer_once() {
        let (mut w, husk) = fight_husk();
        let before = glimmer(&w);
        let pay =
            w.ai.get(w.actors.get(husk).unwrap().ai.as_ref().unwrap().def)
                .unwrap()
                .data
                .glimmer;
        assert_eq!(pay, 15);
        w.actors.get_mut(husk).unwrap().pos[1] = w.physics.kill_y - 10.0;
        tick(&mut w, &TickInput::default());
        assert!(w.actors.get(husk).unwrap().dead);
        assert_eq!(glimmer(&w), before + pay);
        for _ in 0..30 {
            tick(&mut w, &TickInput::default());
        }
        assert_eq!(glimmer(&w), before + pay);
    }

    /// A dead player gets nothing for an enemy that dies in the same tick.
    #[test]
    fn a_dead_player_is_not_paid() {
        let (mut w, husk) = fight_husk();
        let before = glimmer(&w);
        let living = super::living_enemies(&w);
        assert_eq!(living, vec![husk]);
        w.actors.get_mut(husk).unwrap().dead = true;
        w.actors.get_mut(w.player).unwrap().dead = true;
        super::pay_enemy_glimmer(&mut w, &living);
        assert_eq!(glimmer(&w), before);
    }

    /// A Whitewarden Skeleton killed by an ordinary blow stands up again and pays nothing;
    /// killed by a critical (a final death), it pays.
    #[test]
    fn a_reviving_enemy_pays_only_for_its_final_death() {
        use crate::ai::special::revive::{DeathBlow, note_death_blow};
        let content = Content::load_embedded().unwrap();
        let mut w = World::new(1, &content, "fight_skeleton").unwrap();
        let bones = w.state.ai_enemies[0];
        let before = glimmer(&w);
        let living = super::living_enemies(&w);
        assert!(living.contains(&bones));
        w.actors.get_mut(bones).unwrap().dead = true;
        note_death_blow(&mut w, bones, DeathBlow::default());
        super::pay_enemy_glimmer(&mut w, &living);
        assert_eq!(glimmer(&w), before);
        note_death_blow(
            &mut w,
            bones,
            DeathBlow {
                critical: true,
                ..DeathBlow::default()
            },
        );
        super::pay_enemy_glimmer(&mut w, &living);
        assert!(glimmer(&w) > before);
    }
}
