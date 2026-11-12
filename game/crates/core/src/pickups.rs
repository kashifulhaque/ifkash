//! Pickups: items lying in the level that the player takes by walking up to them
//! (WP-049). Relics and Chronicle Pages are pickups that unlock codex entries
//! (docs/game/08-portfolio-integration.md, "Codex").
//!
//! A pickup is taken once. Taking it raises the flag `pickup.<id>.taken` (so it never
//! reappears, survives a save, and is in the state hash), raises `codex.<entry>` when the
//! scenario names a codex entry, and emits [`Event::Pickup`] (WASM event kind 17) for the
//! host. The host's codex store records the unlock from that event; the decision is made
//! here, not in the renderer.
//!
//! WP-042 builds general pickups and chests on `world_state/`; this module only covers
//! the scenario-placed pickups the portfolio needs, and the state lives in flags so it
//! doesn't touch the world state's layout.

use lf_content::{Content, Scenario};
use serde::{Deserialize, Serialize};

use crate::events::{Event, PickupEvent};
use crate::world::World;
use crate::world_state::{Layout, flags};

/// How far above or below the player's feet a pickup still counts as in reach, in meters.
const REACH_HEIGHT_M: f32 = 1.5;

/// A scenario pickup with its item resolved to a registry number.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PickupSpot {
    pub id: String,
    pub item: u32,
    pub pos: [f32; 3],
    pub radius_m: f32,
    pub codex: Option<String>,
    /// How many of the item (WP-042).
    pub count: u16,
    /// Taken with `INTERACT` instead of by walking up (WP-042).
    pub interact: bool,
}

/// Flag name for "this pickup has been taken".
pub fn taken_flag(id: &str) -> String {
    format!("pickup.{id}.taken")
}

/// Fills `layout.pickups` from the scenario. Fails on an item the registry doesn't know.
pub fn extend_layout(
    layout: &mut Layout,
    level: &Scenario,
    content: &Content,
) -> Result<(), String> {
    for p in &level.pickups {
        let item = content
            .registry
            .num(&p.item)
            .ok_or_else(|| format!("pickup {:?}: unknown item \"{}\"", p.id, p.item))?;
        layout.pickups.push(PickupSpot {
            id: p.id.clone(),
            item,
            pos: p.pos,
            radius_m: p.radius_m,
            codex: p.codex.clone(),
            count: p.count.max(1),
            interact: p.interact,
        });
    }
    Ok(())
}

/// Takes pickup `i`: puts the item in the inventory if the item table knows it (a relic
/// or page is only a flag and an event), raises the flags, and emits the event. Returns
/// false, taking nothing, if the inventory can't hold the item or it was already taken.
pub fn take(world: &mut World, i: usize) -> bool {
    let Some(s) = world.layout.pickups.get(i) else {
        return false;
    };
    let (id, item, count, codex) = (s.id.clone(), s.item, s.count, s.codex.clone());
    let flag = taken_flag(&id);
    if world.state.flags.is_set(&flag) {
        return false;
    }
    if world.items.get(item).is_some() || world.items.is_weapon(item) {
        let player = world.player;
        let Some(p) = world.actors.get_mut(player) else {
            return false;
        };
        let took = if world.items.is_weapon(item) {
            u16::from(p.inventory.give_gear(item))
        } else {
            p.inventory.pick_up(item, count, &world.items)
        };
        if took == 0 {
            return false;
        }
    }
    world.state.flags.raise(&flag);
    if let Some(entry) = codex {
        world.state.flags.raise(&flags::codex(&entry));
    }
    world.events.push(Event::Pickup(PickupEvent {
        index: i as u16,
        item,
    }));
    true
}

/// Part of tick system 10 (`systems::world_logic`): takes every untaken walk-up pickup
/// (not an `interact` one; see `world_state::mechanics`) the living player is standing
/// within reach of.
pub fn run(world: &mut World) {
    if world.layout.pickups.is_empty() {
        return;
    }
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    if p.dead {
        return;
    }
    let feet = p.pos;
    for i in 0..world.layout.pickups.len() {
        let s = &world.layout.pickups[i];
        let (dx, dz) = (s.pos[0] - feet[0], s.pos[2] - feet[2]);
        let near = dx * dx + dz * dz <= s.radius_m * s.radius_m
            && (s.pos[1] - feet[1]).abs() <= REACH_HEIGHT_M;
        if near && !s.interact {
            take(world, i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::TickInput;

    fn field() -> World {
        let content = Content::load_embedded().unwrap();
        World::new(1, &content, "test_field").unwrap()
    }

    fn put_player(w: &mut World, pos: [f32; 3]) {
        let id = w.player;
        w.actors.get_mut(id).unwrap().pos = pos;
    }

    #[test]
    fn the_test_field_has_the_stub_relic() {
        let w = field();
        let s = &w.layout.pickups[0];
        assert_eq!(s.id, "relic_test_stub");
        assert_eq!(s.item, 12000);
        assert_eq!(s.codex.as_deref(), Some("projects.test-relic"));
    }

    #[test]
    fn far_away_takes_nothing() {
        let mut w = field();
        crate::tick(&mut w, &TickInput::default());
        assert!(!w.state.flags.is_set("pickup.relic_test_stub.taken"));
        assert!(w.events.drain().is_empty());
    }

    #[test]
    fn walking_up_unlocks_the_codex_entry_once() {
        let mut w = field();
        let spot = w.layout.pickups[0].pos;
        put_player(&mut w, [spot[0], spot[1], spot[2] - 0.5]);
        crate::tick(&mut w, &TickInput::default());
        assert!(w.state.flags.is_set("pickup.relic_test_stub.taken"));
        assert!(w.state.flags.is_set("codex.projects.test-relic"));
        let taken: Vec<_> = w
            .events
            .drain()
            .into_iter()
            .filter_map(|e| match e {
                Event::Pickup(p) => Some(p),
                _ => None,
            })
            .collect();
        assert_eq!(
            taken,
            vec![PickupEvent {
                index: 0,
                item: 12000
            }]
        );
        // Standing there again takes nothing more.
        crate::tick(&mut w, &TickInput::default());
        assert!(
            w.events
                .drain()
                .iter()
                .all(|e| !matches!(e, Event::Pickup(_)))
        );
    }

    #[test]
    fn taking_it_changes_the_state_hash_and_is_deterministic() {
        let run = |near: bool| {
            let mut w = field();
            if near {
                let spot = w.layout.pickups[0].pos;
                put_player(&mut w, spot);
            }
            crate::tick(&mut w, &TickInput::default());
            w.state_hash()
        };
        assert_eq!(run(true), run(true));
        assert_ne!(run(true), run(false));
    }
}
