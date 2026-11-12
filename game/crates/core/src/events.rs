//! Event queue drained by the host every tick. Excluded from the state hash.

use serde::{Deserialize, Serialize};

use crate::arena::Id;
use crate::combat::moves::MoveNum;

/// A simulation event for the host. WP-004 and later add variants.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// A hitbox connected with a hurtbox (`combat::resolve`).
    Hit(Hit),
}

/// One hitbox connecting with one target: the raw inputs to damage, poise, status, and
/// knockback (docs/game/03-combat.md, "Damage dealt"). The simulation reads none of it
/// back; WP-023 computes damage from the same inputs inside the core.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub attacker: Id,
    pub target: Id,
    /// The attacker's move, as its registry number.
    pub move_num: MoveNum,
    /// Index of the hitbox in the move's `hitboxes` list.
    pub hitbox: u8,
    /// The hitbox's motion value, scaled by the charge: lerped from the listed value to
    /// `charge.motion_value_max` by `charge_frac` for a chargeable move.
    pub motion_value: f32,
    /// How far the move was charged, 0 to 1 (0 for a move without a charge).
    pub charge_frac: f32,
    /// Contact point in world space, between the hitbox and hurtbox surfaces.
    pub point: [f32; 3],
    /// Unit direction in the XZ plane from the attacker to the target (the attacker's
    /// facing if they stand on the same spot): the knockback direction.
    pub dir: [f32; 3],
}

/// Events emitted since the host last drained the queue.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventQueue {
    events: Vec<Event>,
}

impl EventQueue {
    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Removes and returns all queued events in emission order.
    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
