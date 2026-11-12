//! Formula reflection (docs/game/05-enemies-and-bosses.md, "Enemy roster", Shard Golem): an
//! enemy with `reflects_formulae` sends a Formula back at its shooter instead of taking it,
//! unless a Blunt hit cracked its crystals within `cracked_ticks`.
//!
//! A Formula is an elemental projectile with no physical type: the seeker bolt and the
//! Acolyte's Radiance bolt are, an arrow is not. `combat::projectile` asks [`reflects`] when
//! a projectile touches an actor, and `combat::resolve` calls [`note_blunt`] for each hit.

use lf_content::ProjectileDef;
use lf_content::moves::PhysType;

use crate::arena::Id;
use crate::world::World;

/// True if `def` is a Formula.
pub fn is_formula(def: &ProjectileDef) -> bool {
    def.element.is_some() && def.phys == PhysType::Neutral
}

/// True if `target` reflects a Formula `def` right now: it has the rule and isn't cracked.
pub fn reflects(world: &World, target: Id, def: &ProjectileDef) -> bool {
    if !is_formula(def) {
        return false;
    }
    let Some(b) = world.actors.get(target).and_then(|a| a.ai.as_ref()) else {
        return false;
    };
    b.cracked == 0
        && world.ai.defs[usize::from(b.def)]
            .data
            .reflects_formulae
            .is_some()
}

/// Records that a Blunt hit landed on `target`: its crystals crack.
pub fn note_blunt(world: &mut World, target: Id) {
    let Some(b) = world.actors.get(target).and_then(|a| a.ai.as_ref()) else {
        return;
    };
    let Some(rule) = world.ai.defs[usize::from(b.def)].data.reflects_formulae else {
        return;
    };
    if let Some(b) = world.actors.get_mut(target).and_then(|a| a.ai.as_mut()) {
        b.cracked = rule.cracked_ticks;
    }
}

/// Counts the crack down.
pub fn tick(world: &mut World, id: Id) {
    if let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
        b.cracked = b.cracked.saturating_sub(1);
    }
}
