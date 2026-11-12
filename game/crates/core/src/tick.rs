//! The tick schedule. The order is part of the determinism contract.

use crate::input::TickInput;
use crate::systems;
use crate::world::World;

/// Advances the world by one tick, running the eleven systems in spec order.
pub fn tick(world: &mut World, input: &TickInput) {
    systems::apply_commands::run(world, input);
    // A player on a ladder (or who just let go) can't attack, dodge, or jump (WP-042).
    let masked = crate::world_state::mechanics::mask_input(world, input);
    let input = masked.as_ref().unwrap_or(input);
    systems::player_intent::run(world, input);
    systems::ai_think::run(world, input);
    systems::move_state::run(world, input);
    systems::locomotion::run(world, input);
    systems::projectiles::run(world, input);
    systems::hit_resolution::run(world, input);
    systems::status_regen::run(world, input);
    systems::deaths_drops::run(world, input);
    systems::world_logic::run(world, input);
    systems::snapshot::run(world, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh(seed: u64) -> World {
        let content = lf_content::Content::load_embedded().unwrap();
        World::new(seed, &content, "m0_flat").unwrap()
    }

    fn run(seed: u64, ticks: u64) -> World {
        let mut w = fresh(seed);
        let input = TickInput::default();
        for _ in 0..ticks {
            tick(&mut w, &input);
        }
        w
    }

    #[test]
    fn ten_thousand_ticks_are_reproducible() {
        let start = fresh(42).state_hash();
        let a = run(42, 10_000);
        let b = run(42, 10_000);
        assert_eq!(a.tick, 10_000);
        assert_eq!(a.state_hash(), b.state_hash());
        assert_ne!(a.state_hash(), start);
        assert_ne!(run(43, 10_000).state_hash(), a.state_hash());
    }

    #[test]
    fn cloned_world_stays_in_lockstep() {
        let mut a = run(42, 500);
        let mut b = a.clone();
        assert_eq!(a.state_hash(), b.state_hash());
        let input = TickInput::default();
        for _ in 0..500 {
            tick(&mut a, &input);
            tick(&mut b, &input);
        }
        assert_eq!(a.state_hash(), b.state_hash());
    }
}
