//! The pack rule (docs/game/05-enemies-and-bosses.md, "Enemy roster", Rime Wolf: "Packs of
//! 3-4; circles, then one lunges at a time"). An enemy with `pack` has packmates: the other
//! living enemies of its definition in Combat within `radius_m`.
//!
//! - Circling: a packmate that's already strafing sets the direction, so the pack runs one
//!   way round the target instead of milling about ([`circle_left`]).
//! - One at a time: an attack move can't start while a packmate is in one ([`may_attack`]).
//!
//! The rule keeps no state of its own; it reads the packmates' `AiState`.

use lf_content::enemy::Behavior;

use crate::ai::perception::dist_xz;
use crate::ai::{Awareness, Doing};
use crate::arena::Id;
use crate::world::World;

/// `me`'s packmates, in arena order. Empty if its definition has no `pack`.
fn mates(world: &World, me: Id) -> Vec<Id> {
    let Some(a) = world.actors.get(me) else {
        return Vec::new();
    };
    let Some(b) = a.ai.as_ref() else {
        return Vec::new();
    };
    let Some(rule) = world.ai.def(b.def).data.pack else {
        return Vec::new();
    };
    world
        .actors
        .iter()
        .filter(|&(id, o)| {
            id != me
                && !o.dead
                && o.ai.as_ref().is_some_and(|ob| {
                    ob.def == b.def
                        && ob.awareness == Awareness::Combat
                        && dist_xz(o.pos, a.pos) <= rule.radius_m
                })
        })
        .map(|(id, _)| id)
        .collect()
}

/// True if `me` may start an attack move: no packmate is in one. Always true without `pack`.
pub fn may_attack(world: &World, me: Id) -> bool {
    !mates(world, me).into_iter().any(|m| {
        world
            .actors
            .get(m)
            .and_then(|a| a.ai.as_ref())
            .is_some_and(|b| matches!(b.doing, Doing::Attack { .. }))
    })
}

/// The strafe direction a pack member should take: that of the first packmate that is
/// strafing now (true is to its left). `None` if none is (or `me` has no pack): the caller
/// keeps its own draw.
pub fn circle_left(world: &World, me: Id) -> Option<bool> {
    mates(world, me).into_iter().find_map(|m| {
        let b = world.actors.get(m)?.ai.as_ref()?;
        (matches!(b.doing, Doing::Behavior(Behavior::Strafe { .. })) && b.doing_ticks > 0)
            .then_some(b.strafe_left)
    })
}
