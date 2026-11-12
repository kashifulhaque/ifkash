//! Scripted dummies: the stand-in for enemy AI until WP-030 (docs/game/10-roadmap.md).
//!
//! A scenario dummy can carry a [`DummyScript`]: a move it starts on a fixed schedule
//! whenever it is free to act. It doesn't aim, dodge, or react; its facing is the
//! scenario yaw. WP-030's AI replaces [`run`] (the `ai_think` system calls it) and the
//! `script` field of the actor; the move it plays is ordinary content.

use serde::{Deserialize, Serialize};

use super::moves::{ActiveMove, MoveNum};
use crate::arena::Id;
use crate::world::World;

/// A dummy's attack loop. Simulation state: part of the state hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DummyScript {
    /// The move it plays.
    pub move_num: MoveNum,
    /// The first tick it starts the move.
    pub first_tick: u64,
    /// Ticks between starts. At least 1.
    pub every_ticks: u64,
}

impl DummyScript {
    /// True if the move is due on `tick`.
    pub fn due(&self, tick: u64) -> bool {
        tick >= self.first_tick && (tick - self.first_tick).is_multiple_of(self.every_ticks.max(1))
    }
}

/// Starts each scripted dummy's move on its due ticks. A dummy that is dead, mid-move,
/// staggered, kneeling, or parried skips the start; the next due tick tries again.
pub fn run(world: &mut World) {
    let ids: Vec<Id> = world.actors.iter().map(|(id, _)| id).collect();
    for id in ids {
        let tick = world.tick;
        let Some(a) = world.actors.get(id) else {
            continue;
        };
        let Some(script) = a.script else {
            continue;
        };
        if a.dead || !script.due(tick) || a.moves.current.is_some() || !a.moves.guard.can_act() {
            continue;
        }
        let Some(d) = world.moves.get(script.move_num) else {
            continue;
        };
        let (num, free_from) = (d.num, d.data.cancel_any_from.unwrap_or(d.data.total_frames));
        if let Some(a) = world.actors.get_mut(id) {
            a.moves.current = Some(ActiveMove {
                num,
                frame: 0,
                charge_ticks: 0,
                hold_button: 0,
                free_from,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_on_the_first_tick_and_every_period_after() {
        let s = DummyScript {
            move_num: 1,
            first_tick: 40,
            every_ticks: 90,
        };
        let due: Vec<u64> = (0..300).filter(|&t| s.due(t)).collect();
        assert_eq!(due, vec![40, 130, 220]);
    }
}
