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
    /// A dead enemy stood up again (`ai::special::revive`, WP-032).
    Revive(ReviveEvent),
    /// A boss fight began, changed phase, ended, or reset (`ai::boss`).
    Boss(BossEvent),
    /// The player took a pickup (`pickups`, WP-049).
    Pickup(PickupEvent),
    /// A door, lever, lift, ladder, or hollow wall did something
    /// (`world_state::mechanics`, WP-042; WASM event kind 14).
    Mechanic(MechanicEvent),
    /// The player opened a chest (WP-042; WASM event kind 15).
    Loot(LootEvent),
    /// The player read an authored message (WP-042; WASM event kind 16).
    Message(MessageEvent),
    /// An Echo was summoned, fell, was dismissed, was bowed to, or a summoning was refused
    /// (`ai::echo`, WP-059; WASM event kind 21).
    Echo(EchoEvent),
    /// A conversation opened, changed topic, or closed (`world_state::quests`, WP-068; WASM
    /// event kind 31).
    Dialogue(DialogueEvent),
    /// A questline was taken up or finished (WP-068; WASM event kind 32).
    Quest(QuestEvent),
    /// The Mirror changed the appearance preset or the dye (WP-068; WASM event kind 33).
    Appearance(AppearanceEvent),
    /// The Briar Stag's bramble patches grew or went (`ai::bosses::b02`, WP-070; WASM event
    /// kind 40).
    Briar(BriarEvent),
}

/// What happened to B02's bramble patches (WASM event kind 40). `what` is one of the consts;
/// `boss` is the boss's index in the scenario's bosses, `index` the patch's id (unique
/// among the patches on the ground, see `World::briar_patches`), `pos` where it lies, `radius` how far it slows, and `ticks`
/// how long it lasts (0 for `FADED` and `CLEARED`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BriarEvent {
    pub what: u8,
    pub boss: u16,
    pub index: u16,
    pub pos: [f32; 3],
    pub radius: f32,
    pub ticks: u16,
}

impl BriarEvent {
    /// A patch grew.
    pub const SPAWNED: u8 = 0;
    /// A patch ran out of time and withered.
    pub const FADED: u8 = 1;
    /// Every patch went at once: the fight ended or reset.
    pub const CLEARED: u8 = 2;
}

/// What happened to a conversation (WASM event kind 31). `what` is one of the consts;
/// `target` says whether it is an NPC (0) or a station (1); `index` is its index in the
/// scenario's `npcs` or `stations`; `topic` is the open topic (0 for the menu, a topic's
/// index plus one, or 255 for the quest line).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DialogueEvent {
    pub what: u8,
    pub target: u8,
    pub index: u16,
    pub topic: u8,
}

impl DialogueEvent {
    /// The player started talking.
    pub const OPENED: u8 = 0;
    /// The conversation ended: the player closed it, walked off, or died.
    pub const CLOSED: u8 = 1;
    /// The player asked about a topic, or went back to the menu.
    pub const TOPIC: u8 = 2;
}

/// A quest's state changed (WASM event kind 32): `npc` is the giver's index in the scenario's
/// `npcs`, `state` the new state (1 taken up, 2 done).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuestEvent {
    pub npc: u16,
    pub state: u8,
}

/// The Mirror's preset and dye changed (WASM event kind 33).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppearanceEvent {
    pub preset: u8,
    pub dye: u8,
}

/// What happened to an Echo (WASM event kind 21). `kind` is one of the consts. `echo` is the
/// Echo's registry number (0 when none applies, for a refusal before one was picked).
/// `actor` is the body that summoned, fell, or was bowed to (the first body for a summoning).
/// `reason` is why a summoning was refused, one of the `REASON_*` consts, and 0 otherwise.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EchoEvent {
    pub kind: u8,
    pub echo: u32,
    pub actor: Option<Id>,
    pub reason: u8,
}

impl EchoEvent {
    /// The Echo's bodies appeared at an Echo Stone.
    pub const SUMMONED: u8 = 0;
    /// One body fell.
    pub const FELL: u8 = 1;
    /// The Echo's bodies were removed: all of them had fallen, the player died, or the
    /// player rested.
    pub const DISMISSED: u8 = 2;
    /// The player bowed to it in time: it fights harder.
    pub const BOWED: u8 = 3;
    /// The Bell was rung and nothing came; `reason` says why.
    pub const REFUSED: u8 = 4;

    /// Refused: no Echo Stone within reach.
    pub const REASON_NO_STONE: u8 = 1;
    /// Refused: the player owns no Echo.
    pub const REASON_NO_ECHO: u8 = 2;
    /// Refused: not enough FP.
    pub const REASON_NO_FP: u8 = 3;
    /// Refused: an Echo is already summoned.
    pub const REASON_ALREADY: u8 = 4;
}

/// The player took the pickup at `index` of `Layout::pickups` (WASM event kind 17).
/// `item` is the item's registry number.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PickupEvent {
    pub index: u16,
    pub item: u32,
}

/// What a world mechanic did. `index` is the entity's index in its scenario list: doors
/// for the door kinds, levers for `LEVER_PULLED`, lifts for the lift kinds, ladders for the
/// ladder kinds, hollow walls for `WALL_REVEALED`. `kind` is one of the consts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MechanicEvent {
    pub kind: u8,
    pub index: u16,
}

impl MechanicEvent {
    /// A door opened.
    pub const DOOR_OPENED: u8 = 0;
    /// A lever shut a door.
    pub const DOOR_CLOSED: u8 = 1;
    /// The player tried a locked door that only a lever opens.
    pub const DOOR_LOCKED: u8 = 2;
    /// The player tried a one-way door from the wrong side ("Opens from the other
    /// side").
    pub const DOOR_ONE_WAY: u8 = 3;
    /// The player tried a keyed door without the key.
    pub const DOOR_NEEDS_KEY: u8 = 4;
    /// The player pulled a lever.
    pub const LEVER_PULLED: u8 = 5;
    /// A lift set off.
    pub const LIFT_STARTED: u8 = 6;
    /// A lift reached a stop.
    pub const LIFT_ARRIVED: u8 = 7;
    /// The player grabbed a ladder.
    pub const LADDER_GRABBED: u8 = 8;
    /// The player let go of a ladder, or climbed off it.
    pub const LADDER_RELEASED: u8 = 9;
    /// The player kicked a raised ladder down.
    pub const LADDER_KICKED: u8 = 10;
    /// A hit revealed a hollow wall's passage.
    pub const WALL_REVEALED: u8 = 11;
    /// The player travelled to another area (WP-069); `index` is the place they went to
    /// (`world_state::map::places`). The world was replaced, so the host rebuilds its view.
    pub const AREA_CHANGED: u8 = 90;
    /// The player placed a map marker (WP-069).
    pub const MARKER_PLACED: u8 = 91;
    /// The player removed a map marker (WP-069).
    pub const MARKER_REMOVED: u8 = 92;
    /// B08, the Forgeheart Colossus (WP-076): a leg's stance broke and the Colossus kneels,
    /// its furnace open for a riposte. `index` is the leg: 0 the left, 1 the right.
    pub const COLOSSUS_LEG_BROKE: u8 = 170;
    /// B08: the heart ignited and lava began to creep in from the walls. `index` is the boss's
    /// index in the scenario.
    pub const COLOSSUS_LAVA_STARTED: u8 = 171;
    /// B08: the lava burned the player. `index` is the boss's index in the scenario.
    pub const COLOSSUS_LAVA_BURNED: u8 = 172;
    /// B06's survivors sang a new verse (WP-074); `index` is the member that fell (0 the
    /// Cantor, 1 the Bellbearer, 2 the Lamenter). WASM kind 14, sub-kinds 150-159 are B06's.
    pub const CHOIR_VERSE: u8 = 150;
    /// The Cantor's mending hymn finished and healed a member (WP-074); `index` is the member
    /// healed.
    pub const CHOIR_MEND: u8 = 151;
    /// The Lamenter reappeared behind the player after fading (WP-074); `index` is her
    /// member number (2).
    pub const CHOIR_TELEPORT: u8 = 152;
    /// A Briar Stag charge started (WP-070); `index` is the boss's index. The renderer
    /// kicks up dust and the camera rumbles.
    pub const BRIAR_CHARGE_STARTED: u8 = 110;
    /// A Briar Stag charge ran into a tree or a hedge and stunned the Stag (WP-070);
    /// `index` is the boss's index.
    pub const BRIAR_TREE_STRUCK: u8 = 111;
    /// B05 (WP-073): the Keeper's Raise stood fallen thralls up; `index` is how many.
    pub const B05_RAISED: u8 = 140;
    /// B05: the water began to rise (phase 2).
    pub const B05_TIDE_RISING: u8 = 141;
    /// B05: the Drowned stood up at the vault's edges; `index` is how many.
    pub const B05_DROWNED_JOIN: u8 = 142;
    /// B05: the Keeper took a fallen thrall's weapon.
    pub const B05_WEAPON_TAKEN: u8 = 143;
}

/// The player opened a chest (`source` is always 1; pickups send `PickupEvent`), and
/// `index` is its index in the scenario's chest list. `item` is the registry number (0 if none) and `count` how
/// many were taken.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LootEvent {
    pub source: u8,
    pub index: u16,
    pub item: u32,
    pub count: u16,
    pub glimmer: u32,
}

/// The player read the authored message at `index` of the scenario's messages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessageEvent {
    pub index: u16,
}

/// A reviver got back up with `hp` HP (WASM event kind 13).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviveEvent {
    pub actor: Id,
    pub hp: u32,
}

/// What happened in a boss fight (WP-033). `boss` indexes the scenario's bosses
/// (`Layout::bosses`). `kind` is one of the consts; `phase` is the phase it concerns
/// (the new phase for `PHASE_STARTED`, 1 for `FIGHT_STARTED`, otherwise 0).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BossEvent {
    pub boss: u16,
    pub kind: u8,
    pub phase: u8,
}

impl BossEvent {
    /// The player walked through the fog gate: it sealed and the intro wait began.
    pub const SEALED: u8 = 0;
    /// The intro ended and the boss fights.
    pub const FIGHT_STARTED: u8 = 1;
    /// A phase threshold was crossed: the transition move starts.
    pub const PHASE_STARTED: u8 = 2;
    /// The boss fell: the gate opened and the rewards paid out.
    pub const DEFEATED: u8 = 3;
    /// The player died: the boss was rebuilt and the gate opened.
    pub const RESET: u8 = 4;
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
