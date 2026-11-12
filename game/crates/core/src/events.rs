//! Event queue drained by the host every tick. Excluded from the state hash.

use serde::{Deserialize, Serialize};

use crate::arena::Id;
use crate::combat::moves::MoveNum;

/// A simulation event for the host. WP-004 and later add variants.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// A hitbox connected with a hurtbox (`combat::resolve`).
    Hit(Hit),
    /// The player's death, respawn, resting, or Remnant (`world_state`).
    Life(LifeEvent),
}

/// What happened to the player's life cycle (docs/game/06-progression-and-systems.md,
/// "Glimmer and death" and "Lanterns"). `pos` is the player's feet, except for the
/// Remnant events, where it is the Remnant's position.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LifeEvent {
    /// The player died.
    Died { pos: [f32; 3] },
    /// The player respawned at the last rested Lantern (or the spawn).
    Respawned { pos: [f32; 3] },
    /// The player rested at the Lantern at this index of the scenario's Lantern list.
    Rested { lantern: u16, pos: [f32; 3] },
    /// A death dropped a Remnant holding this much Glimmer.
    RemnantDropped { glimmer: u32, pos: [f32; 3] },
    /// The player touched the Remnant and got its Glimmer back.
    RemnantRecovered { glimmer: u32, pos: [f32; 3] },
    /// A death destroyed the unrecovered Remnant.
    RemnantLost { glimmer: u32, pos: [f32; 3] },
}

/// One hitbox connecting with one target: the raw inputs to damage, poise, status, and
/// knockback (docs/game/03-combat.md, "Damage dealt"). The simulation reads none of it
/// back; `combat::damage` computes damage from the same inputs inside the core, and the
/// last five fields report the result.
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
    /// HP the target lost: the damage after the guard's absorption (WP-023).
    pub damage: u32,
    /// How the target took the hit.
    pub result: HitResult,
    /// The target was in the startup or active frames of its own attack, so the hit
    /// had the counter-hit multiplier.
    pub counter: bool,
    /// This hit took the target from above 0 HP to 0.
    pub killed: bool,
    /// The target's HP after the hit.
    pub hp: u32,
}

/// How a target took a landed hit (`combat::guard::HitOutcome` without the numbers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitResult {
    /// Not guarding: the full damage landed.
    Connected,
    /// Blocked: reduced damage, and the block cost stamina.
    Guarded,
    /// The block cost more stamina than the target had.
    GuardBroken,
}

impl HitResult {
    /// The code the WASM event stream uses.
    pub fn code(self) -> u32 {
        match self {
            Self::Connected => 0,
            Self::Guarded => 1,
            Self::GuardBroken => 2,
        }
    }
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
