//! The revive rule (docs/game/05-enemies-and-bosses.md, "Enemy roster", Whitewarden
//! Skeleton): an enemy with `revive` stands up `after_ticks` after dying with
//! `hp_percent` of its HP, unless its death was one of the `stopped_by` kinds.
//!
//! The combat pipeline calls [`note_death_blow`] when a hit kills an AI actor, so the rule
//! can see how it died (Radiance, a critical, or near a lit Lantern). [`tick_dead`] runs
//! each tick for a dead AI actor, counts the ticks, and revives it.

use lf_content::enemy::ReviveStop;

use crate::actor::Actor;
use crate::ai::{Awareness, Doing, perception::dist_xz};
use crate::arena::Id;
use crate::combat::guard::GuardState;
use crate::events::{Event, ReviveEvent};
use crate::world::World;
use crate::world_state::flags::lantern_lit;

/// How the blow that killed an actor was dealt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeathBlow {
    /// A hitbox with the Radiance element.
    pub radiance: bool,
    /// A riposte or a backstab.
    pub critical: bool,
}

/// True if `stop` keeps the actor `a` dead given `blow`.
fn stops(world: &World, a: &Actor, stop: ReviveStop, blow: DeathBlow) -> bool {
    match stop {
        ReviveStop::Radiance => blow.radiance,
        ReviveStop::Critical => blow.critical,
        ReviveStop::LitLantern(m) => {
            world.layout.lanterns.iter().any(|l| {
                world.state.flags.is_set(&lantern_lit(&l.id)) && dist_xz(l.pos, a.pos) <= m
            })
        }
    }
}

/// Records how `id` died. If the definition's rule says this death is final, the actor
/// stays dead; otherwise its revive countdown starts.
pub fn note_death_blow(world: &mut World, id: Id, blow: DeathBlow) {
    let Some(a) = world.actors.get(id) else {
        return;
    };
    let Some(b) = a.ai.as_ref() else {
        return;
    };
    let Some(rule) = &world.ai.def(b.def).data.revive else {
        return;
    };
    let final_death = rule.stopped_by.iter().any(|&s| stops(world, a, s, blow));
    if let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
        b.dead_ticks = 0;
        b.stays_dead = final_death;
    }
}

/// One tick of a dead AI actor: counts up and, if it has a revive rule that wasn't
/// stopped, brings it back at `after_ticks`.
pub fn tick_dead(world: &mut World, id: Id) {
    let Some(a) = world.actors.get(id) else {
        return;
    };
    let Some(b) = a.ai.as_ref() else {
        return;
    };
    let Some(rule) = &world.ai.def(b.def).data.revive else {
        return;
    };
    if b.stays_dead {
        return;
    }
    let (after, percent) = (rule.after_ticks, rule.hp_percent);
    let ticks = b.dead_ticks.saturating_add(1);
    if ticks < after {
        if let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
            b.dead_ticks = ticks;
        }
        return;
    }
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    let hp = (u64::from(a.max_hp) * u64::from(percent) / 100).max(1) as u32;
    a.dead = false;
    a.hp = hp;
    a.vel = [0.0; 3];
    a.moves.interrupt();
    a.moves.guard = GuardState::default();
    a.poise.accum = 0;
    a.poise.stance_dmg = 0;
    a.poise.kneel = 0;
    a.poise.parried = 0;
    a.poise.unaware = false;
    a.hit.prev = None;
    crate::combat::status::cure_all(a);
    if let Some(b) = a.ai.as_mut() {
        b.dead_ticks = 0;
        b.last_hp = hp;
        b.doing = Doing::Nothing;
        b.doing_ticks = 0;
        b.guarding = false;
        b.path.clear();
        b.path_ticket = None;
        b.cooldowns.iter_mut().for_each(|c| *c = 0);
        // It comes back in the fight: if it had a target it keeps it, and otherwise it
        // is Idle at the place it fell.
        if b.target.is_some() {
            b.awareness = Awareness::Combat;
            b.state_ticks = 0;
        } else {
            b.awareness = Awareness::Idle;
        }
    }
    world
        .events
        .push(Event::Revive(ReviveEvent { actor: id, hp }));
}
