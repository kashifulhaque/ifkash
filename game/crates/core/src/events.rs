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
    /// A hit staggered its target (`combat::poise`).
    Stagger(Stagger),
    /// A hit broke its target's stance (`combat::poise`).
    StanceBreak(StanceBreak),
    /// A parry turned an attack aside (`combat::resolve`).
    Parry(Parry),
    /// A critical hit started (`combat::crit`).
    Critical(Critical),
    /// A projectile spawned, struck an actor, hit the level, or expired
    /// (`combat::projectile`). A strike also emits a `Hit` for the damage.
    Projectile(ProjectileEvent),
    /// A status meter filled and the effect took hold (`combat::status`).
    StatusProc(StatusProc),
    Awareness(AwarenessEvent),
}

/// An AI actor changed awareness state (WP-030). `from` and `to` are
/// `ai::Awareness::code` values: 0 Idle, 1 Suspicious, 2 Aware, 3 Combat, 4 Return.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AwarenessEvent {
    pub actor: Id,
    pub from: u8,
    pub to: u8,
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

/// `target` staggered: its move ended and it can't act for `ticks`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stagger {
    pub target: Id,
    /// A heavy stagger (24 ticks) rather than a light one (12).
    pub heavy: bool,
    pub ticks: u8,
}

/// `target`'s stance broke: it kneels for `ticks`, and a riposte can start.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StanceBreak {
    pub target: Id,
    pub ticks: u8,
}

/// `target` parried `attacker`'s hit: no damage, and the attacker is in the parried state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parry {
    pub attacker: Id,
    pub target: Id,
    /// The attacker's parried move.
    pub move_num: MoveNum,
}

/// Which critical hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CritKind {
    /// Against a parried or stance-broken actor, from the front.
    Riposte,
    /// Against an unaware actor, from behind.
    Backstab,
}

impl CritKind {
    /// The code the WASM event stream uses.
    pub fn code(self) -> u32 {
        match self {
            Self::Riposte => 0,
            Self::Backstab => 1,
        }
    }
}

/// `attacker` started a paired critical move on `target`. The damage arrives later, as a
/// `Hit` from `move_num`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Critical {
    pub attacker: Id,
    pub target: Id,
    pub kind: CritKind,
    /// The attacker's crit move.
    pub move_num: MoveNum,
}

/// What happened to a projectile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectileEventKind {
    /// It left the shooter. `pos` is the spawn point.
    Spawned,
    /// It struck `target`. If `ended` is false it flew on through (pierce).
    Struck,
    /// It hit level geometry and stopped.
    Impact,
    /// Its lifetime ran out.
    Expired,
}

impl ProjectileEventKind {
    /// The code the WASM event stream uses.
    pub fn code(self) -> u32 {
        match self {
            Self::Spawned => 0,
            Self::Struck => 1,
            Self::Impact => 2,
            Self::Expired => 3,
        }
    }
}

/// One step in a projectile's life, for the renderer's effects and sounds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectileEvent {
    pub kind: ProjectileEventKind,
    /// The projectile's arena handle. The same handle appears in every event of its life.
    pub projectile: Id,
    /// The projectile's registry number (`proj_arrow` is 5000).
    pub def: u32,
    pub shooter: Id,
    /// The actor struck, for `Struck`.
    pub target: Option<Id>,
    /// Where it was: the spawn point, the contact point, or where it expired.
    pub pos: [f32; 3],
    /// The projectile is gone after this event (the handle may be reused later).
    pub ended: bool,
}

/// `target`'s `status` meter filled: the effect started. `damage` is the HP the burst took
/// (Rupture, Rime, Delirium), 0 for the other statuses. The meter has reset and the
/// status is locked out for a while.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusProc {
    pub target: Id,
    pub status: lf_content::moves::StatusKind,
    pub damage: u32,
}

impl StatusProc {
    /// The code the WASM event stream uses: the status's index in
    /// [`crate::combat::status::KINDS`] (0 Poison, 1 Blight, 2 Drowse, 3 Delirium, 4 Rupture,
    /// 5 Rime).
    pub fn code(&self) -> u32 {
        crate::combat::status::index(self.status) as u32
    }
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
